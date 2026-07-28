use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    Attribute, Data, DeriveInput, ExprPath, Fields, Path, Result, Variant,
};

pub fn derive_instruction(input: DeriveInput) -> TokenStream {
    expand(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

enum Mode {
    Wire(ExprPath),
    Skip,
}

struct VariantInfo {
    ident: syn::Ident,
    payload: Option<syn::Type>,
    mode: Mode,
    cfg: Vec<Attribute>,
}

fn expand(input: DeriveInput) -> Result<TokenStream2> {
    let name = input.ident;
    if !input.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            input.generics,
            "Instruction does not support generic enums",
        ));
    }

    // check that the enum is repr(u8) or repr(u16) or repr(u32) or repr(u64)
    // note: not needed, when we dont assign arbitrary values to skipped fields
    // {
    //     let mut has_repr = false;
    //     for attr in &input.attrs {
    //         if attr.path().is_ident("repr") {
    //             let meta = attr.parse_nested_meta(|meta| {
    //                 if meta.path.is_ident("u8")
    //                     || meta.path.is_ident("u16")
    //                     || meta.path.is_ident("u32")
    //                     || meta.path.is_ident("u64")
    //                 {
    //                     has_repr = true;
    //                     Ok(())
    //                 } else {
    //                     Err(meta.error(
    //                         "expected repr(u8), repr(u16), repr(u32) or repr(u64)",
    //                     ))
    //                 }
    //             })?;
    //         }
    //     }
    //     if !has_repr {
    //         return Err(syn::Error::new_spanned(
    //             name,
    //             "Instruction must have a repr attribute",
    //         ));
    //     }
    // }
    let data = match input.data {
        Data::Enum(data) => data,
        _ => {
            return Err(syn::Error::new_spanned(
                name,
                "Instruction can only be derived for enums",
            ));
        }
    };

    let mut variants = Vec::new();

    // parse the variants and their attributes
    for variant in data.variants {
        variants.push(VariantInfo {
            mode: parse_mode(&variant)?,
            payload: parse_payload(&variant)?,
            cfg: variant
                .attrs
                .iter()
                .filter(|attr| {
                    attr.path().is_ident("cfg")
                        || attr.path().is_ident("cfg_attr")
                })
                .cloned()
                .collect(),
            ident: variant.ident,
        });
    }

    let code_type = find_code_type(&name, &variants)?;

    let code_arms = variants.iter().map(|variant| {
        let ident = &variant.ident;
        let cfg = &variant.cfg;

        let pattern = if variant.payload.is_some() {
            quote!(Self::#ident(..))
        } else {
            quote!(Self::#ident)
        };

        match &variant.mode {
            Mode::Wire(magic) => quote! {
                #(#cfg)*
                #pattern => Some(#magic),
            },

            Mode::Skip => quote! {
                #(#cfg)*
                #pattern => None,
            },
        }
    });

    let read_arms = variants.iter().filter_map(|variant| {
        let Mode::Wire(magic) = &variant.mode else {
            return None;
        };

        let ident = &variant.ident;
        let cfg = &variant.cfg;

        Some(match &variant.payload {
            Some(payload) => quote! {
                #(#cfg)*
                code if code == #magic => {
                    let value =
                        <#payload as ::binrw::BinRead>::read_options(
                            reader,
                            endian,
                            (),
                        )?;

                    Ok(Self::#ident(value))
                }
            },

            None => quote! {
                #(#cfg)*
                code if code == #magic => Ok(Self::#ident)
            },
        })
    });

    let write_arms = variants.iter().map(|variant| {
        let ident = &variant.ident;
        let cfg = &variant.cfg;

        match (&variant.mode, &variant.payload) {
            (Mode::Wire(magic), Some(payload)) => quote! {
                #(#cfg)*
                Self::#ident(value) => {
                    <#code_type as ::binrw::BinWrite>::write_options(
                        &(#magic),
                        writer,
                        endian,
                        (),
                    )?;

                    <#payload as ::binrw::BinWrite>::write_options(
                        value,
                        writer,
                        endian,
                        (),
                    )
                }
            },
            (Mode::Wire(magic), None) => quote! {
                #(#cfg)*
                Self::#ident => {
                    <#code_type as ::binrw::BinWrite>::write_options(
                        &(#magic),
                        writer,
                        endian,
                        (),
                    )
                }
            },
            (Mode::Skip, Some(_)) => skipped_write_arm(ident, cfg, true),
            (Mode::Skip, None) => skipped_write_arm(ident, cfg, false),
        }
    });

    Ok(quote! {
        impl #name {
            pub fn code(&self) -> Option<#code_type> {
                match self {
                    #(#code_arms)*
                }
            }
        }

        impl ::binrw::BinRead for #name {
            type Args<'a> = ();

            fn read_options<R>(
                reader: &mut R,
                endian: ::binrw::Endian,
                (): Self::Args<'_>,
            ) -> ::binrw::BinResult<Self>
            where
                R: ::binrw::io::Read
                    + ::binrw::io::Seek,
            {
                use ::binrw::io::Seek as _;
                let pos = reader.stream_position()?;
                let code =
                    <#code_type as ::binrw::BinRead>::read_options(
                        reader,
                        endian,
                        (),
                    )?;
                match code {
                    #(#read_arms,)*
                    _ => Err(
                        ::binrw::Error::NoVariantMatch { pos },
                    ),
                }
            }
        }

        impl ::binrw::BinWrite for #name {
            type Args<'a> = ();

            fn write_options<W>(
                &self,
                writer: &mut W,
                endian: ::binrw::Endian,
                (): Self::Args<'_>,
            ) -> ::binrw::BinResult<()>
            where
                W: ::binrw::io::Write
                    + ::binrw::io::Seek,
            {
                use ::binrw::io::Seek as _;
                match self {
                    #(#write_arms,)*
                }
            }
        }

        impl ::binrw::meta::ReadEndian for #name {
            const ENDIAN: ::binrw::meta::EndianKind =
                ::binrw::meta::EndianKind::Endian(
                    ::binrw::Endian::Little,
                );
        }

        impl ::binrw::meta::WriteEndian for #name {
            const ENDIAN: ::binrw::meta::EndianKind =
                ::binrw::meta::EndianKind::Endian(
                    ::binrw::Endian::Little,
                );
        }
    })
}

fn parse_mode(variant: &Variant) -> Result<Mode> {
    let mut magic = None;
    let mut skip = false;

    for attr in &variant.attrs {
        if attr.path().is_ident("magic") {
            if magic.is_some() {
                return Err(syn::Error::new_spanned(
                    attr,
                    "duplicate #[magic(...)]",
                ));
            }

            magic = Some(attr.parse_args::<ExprPath>()?);
        } else if attr.path().is_ident("instruction") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("skip") {
                    if skip {
                        return Err(meta.error("duplicate `skip`"));
                    }

                    skip = true;
                    Ok(())
                } else {
                    Err(meta.error("expected `skip`"))
                }
            })?;
        }
    }

    match (magic, skip) {
        (Some(magic), false) => Ok(Mode::Wire(magic)),
        (None, true) => Ok(Mode::Skip),
        (Some(_), true) => Err(syn::Error::new_spanned(
            &variant.ident,
            "use either #[magic(...)] or #[instruction(skip)]",
        )),
        (None, false) => Err(syn::Error::new_spanned(
            &variant.ident,
            "missing #[magic(...)] or #[instruction(skip)]",
        )),
    }
}

fn parse_payload(variant: &Variant) -> Result<Option<syn::Type>> {
    match &variant.fields {
        Fields::Unit => Ok(None),

        Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
            Ok(Some(fields.unnamed.first().unwrap().ty.clone()))
        }

        _ => Err(syn::Error::new_spanned(
            &variant.fields,
            "variants must be unit variants or have one tuple field",
        )),
    }
}

/// Find the code type from the magic values of the variants
fn find_code_type(
    enum_name: &syn::Ident,
    variants: &[VariantInfo],
) -> Result<Path> {
    let mut result: Option<Path> = None;

    for variant in variants {
        let Mode::Wire(magic) = &variant.mode else {
            continue;
        };

        // If the magic inner val is a path, we can extract the code type from it
        let current = code_type_from_magic(magic)?;
        if let Some(expected) = &result {
            if quote!(#expected).to_string() != quote!(#current).to_string() {
                return Err(syn::Error::new_spanned(
                    magic,
                    "all magic values must use the same code type",
                ));
            }
        } else {
            result = Some(current);
        }
    }

    result.ok_or_else(|| {
        syn::Error::new_spanned(
            enum_name,
            "at least one variant needs #[magic(...)]",
        )
    })
}

/// Convert a magic value to a path representing the code type. For example, if the magic value is `InstructionCode::VALUE`, this function will return `InstructionCode`.
fn code_type_from_magic(magic: &ExprPath) -> Result<Path> {
    if magic.qself.is_some() || magic.path.segments.len() < 2 {
        return Err(syn::Error::new_spanned(
            magic,
            "expected #[magic(InstructionCode::VALUE)]",
        ));
    }
    let mut path = magic.path.clone();
    path.segments.pop();
    if path.segments.trailing_punct() {
        path.segments.pop_punct();
    }
    Ok(path)
}

/// Generate a match arm for a skipped variant in the write implementation
fn skipped_write_arm(
    ident: &syn::Ident,
    cfg: &[Attribute],
    has_payload: bool,
) -> TokenStream2 {
    let pattern = if has_payload {
        quote!(Self::#ident(..))
    } else {
        quote!(Self::#ident)
    };

    quote! {
        #(#cfg)*
        #pattern => {
            let pos = writer.stream_position()?;
            Err(::binrw::Error::AssertFail {
                pos,
                message: concat!(
                    stringify!(#ident),
                    " is not a writable instruction variant",
                )
                .to_owned(),
            })
        }
    }
}
