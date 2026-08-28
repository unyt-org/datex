/// Generates the [DatexNative] implementation, including [AsBorrowed] and [AsBorrowedMut] implementations for the given structure data.
/// Returns a TokenStream of the implementations.
pub fn generate_datex_native(structure_data: &StructureData) -> TokenStream {
    let StructureData {
        ident, generics, ..
    } = structure_data;

    quote! {
        use core::any::Any;

        #[automatically_derived]
        impl #generics DatexNative for #ident #generics {
            fn as_any(&self) -> &dyn Any {
                self
            }
            fn as_any_mut(&mut self) -> &mut dyn Any {
                self
            }
            fn boxed_to_datex_native_value(self: Box<Self>, cache: &mut SharedReferencesCache) -> Value {
                Value::native_boxed(self, cache)
            }
        }

        #[automatically_derived]
        impl<'a> AsBorrowed<'a> for #ident #generics { // FIXME: generics
            fn as_borrowed(&'a self) -> BorrowedValueContainer<'a> {
                BorrowedValueContainer::native_borrowed(self)
            }
        }

        #[automatically_derived]
        impl<'a> AsBorrowedMut<'a> for #ident #generics { // FIXME: generics
            fn as_borrowed_mut(&'a mut self) -> BorrowedValueContainerMut<'a> {
                BorrowedValueContainerMut::native_borrowed(self)
            }
        }
    }
}
