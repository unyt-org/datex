use crate::{
    libs::core::type_id::{
        CoreLibBaseTypeId, CoreLibTypeId, CoreLibVariantTypeId,
    },
    types::{
        literal_type_definition::LiteralTypeDefinition,
        shared_container_containing_nominal_type::SharedContainerContainingNominalType,
        shared_container_containing_type::SharedContainerContainingType,
        r#type::Type,
        type_definition::{
            callable::CallableTypeDefinition,
            intersection::IntersectionTypeDefinition, list::ListTypeDefinition,
            map::MapTypeDefinition, union::UnionTypeDefinition,
        },
        type_definition_with_metadata::TypeDefinitionWithMetadata,
        visitor::TypeFolder,
    },
};

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
enum AliasState {
    Visiting,
    Complete(String),
}

#[derive(Debug, Default)]
pub struct TsTypeFolder {
    aliases: HashMap<String, AliasState>,

    declaration_order: Vec<String>,
}

impl TsTypeFolder {
    pub fn new() -> Self {
        Self {
            aliases: HashMap::new(),
            declaration_order: Vec::new(),
        }
    }

    fn alias_name(&self, alias: &TypeDefinitionWithMetadata) -> String {
        santiize_ts_identifier("FIXME")
    }

    pub fn render_declarations(&self) -> String {
        self.declaration_order
            .iter()
            .filter_map(|name| {
                let AliasState::Complete(definition) =
                    self.aliases.get(name)?
                else {
                    return None;
                };
                Some(format!("type {name} = {definition};"))
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn render_module_with_root(&self, root_reference: &str) -> String {
        let declarations = self.render_declarations();
        if declarations.is_empty() {
            root_reference.to_string()
        } else {
            format!("{declarations}\n\n{root_reference}")
        }
    }
}

fn santiize_ts_identifier(name: &str) -> String {
    let mut output = String::new();

    for (index, character) in name.chars().enumerate() {
        let valid = if index == 0 {
            character == '_'
                || character == '$'
                || character.is_ascii_alphabetic()
        } else {
            character == '_'
                || character == '$'
                || character.is_ascii_alphanumeric()
        };

        if valid {
            output.push(character);
        } else {
            output.push('_');
        }
    }

    if output.is_empty() {
        "TOOD".to_string()
    } else {
        output
    }
}

impl TypeFolder for TsTypeFolder {
    type Output = String;
    type Error = ();

    fn begin_alias(
        &mut self,
        alias: &TypeDefinitionWithMetadata,
    ) -> Result<bool, Self::Error> {
        let name = self.alias_name(alias);

        match self.aliases.get(&name) {
            None => {
                self.aliases.insert(name, AliasState::Visiting);
                Ok(true)
            }
            Some(AliasState::Visiting | AliasState::Complete(_)) => Ok(false),
        }
    }

    fn end_alias(
        &mut self,
        alias: &TypeDefinitionWithMetadata,
        definition: Self::Output,
    ) -> Result<(), Self::Error> {
        let name = self.alias_name(alias);

        self.aliases
            .insert(name.clone(), AliasState::Complete(definition));

        Ok(())
    }

    fn fold_alias_reference(
        &mut self,
        alias: &TypeDefinitionWithMetadata,
    ) -> Result<Self::Output, Self::Error> {
        Ok(self.alias_name(alias))
    }

    fn fold_literal(
        &mut self,
        literal: &LiteralTypeDefinition,
    ) -> Result<Self::Output, Self::Error> {
        match literal {
            LiteralTypeDefinition::Text(text) => Ok(format!("\"{}\"", text.0)),

            LiteralTypeDefinition::Integer(integer) => Ok(integer.to_string()),

            LiteralTypeDefinition::TypedInteger(integer) => {
                Ok(integer.to_string())
            }

            LiteralTypeDefinition::Decimal(decimal) => Ok(decimal.to_string()),

            LiteralTypeDefinition::TypedDecimal(decimal) => {
                Ok(decimal.to_string())
            }

            LiteralTypeDefinition::Boolean(boolean) => Ok(boolean.to_string()),

            LiteralTypeDefinition::Endpoint(endpoint) => {
                Ok(format!("Endpoint<\"{}\">", endpoint))
            }
        }
    }

    fn fold_list(
        &mut self,
        _source: &ListTypeDefinition,
        elements: Vec<Self::Output>,
    ) -> Result<Self::Output, Self::Error> {
        Ok(format!("[{}]", elements.join(", ")))
    }

    fn fold_map(
        &mut self,
        _source: &MapTypeDefinition,
        entries: Vec<(Self::Output, Self::Output)>,
    ) -> Result<Self::Output, Self::Error> {
        let entries = entries
            .into_iter()
            .map(|(key, value)| format!("{key}: {value}"))
            .collect::<Vec<_>>()
            .join(", ");

        Ok(format!("{{ {entries} }}"))
    }

    fn fold_nested(
        &mut self,
        _source: &Type,
        inner: Self::Output,
    ) -> Result<Self::Output, Self::Error> {
        Ok(inner)
    }

    fn fold_union(
        &mut self,
        _source: &UnionTypeDefinition,
        members: Vec<Self::Output>,
    ) -> Result<Self::Output, Self::Error> {
        Ok(members.join(" | "))
    }

    fn fold_intersection(
        &mut self,
        _source: &IntersectionTypeDefinition,
        members: Vec<Self::Output>,
    ) -> Result<Self::Output, Self::Error> {
        Ok(members.join(" & "))
    }

    fn fold_callable(
        &mut self,
        _source: &CallableTypeDefinition,
        parameters: Vec<(Option<String>, Self::Output)>,
        rest_parameter: Option<(Option<String>, Self::Output)>,
        return_type: Option<Self::Output>,
        yeet_type: Option<Self::Output>,
    ) -> Result<Self::Output, Self::Error> {
        let mut rendered_parameters = parameters
            .into_iter()
            .enumerate()
            .map(|(index, (name, ty))| {
                let name = name
                    .as_deref()
                    .map(santiize_ts_identifier)
                    .unwrap_or_else(|| format!("arg{index}"));

                format!("{name}: {ty}")
            })
            .collect::<Vec<_>>();

        if let Some((name, ty)) = rest_parameter {
            let name = name
                .as_deref()
                .map(santiize_ts_identifier)
                .unwrap_or_else(|| "rest".to_string());

            rendered_parameters.push(format!("...{name}: {ty}"));
        }

        let _ = yeet_type;

        let return_type = return_type.unwrap_or_else(|| "void".to_string());

        Ok(format!(
            "({}) => {}",
            rendered_parameters.join(", "),
            return_type,
        ))
    }

    fn fold_shared_reference(
        &mut self,
        _shared: &SharedContainerContainingType,
    ) -> Result<Self::Output, Self::Error> {
        Ok("unknown".to_string())
    }

    fn fold_nominal_reference(
        &mut self,
        _nominal: &SharedContainerContainingNominalType,
    ) -> Result<Self::Output, Self::Error> {
        Ok("unknown".to_string())
    }

    fn fold_core_type(
        &mut self,
        core_type: CoreLibTypeId,
    ) -> Result<Self::Output, Self::Error> {
        match core_type {
            CoreLibTypeId::Base(base) => match base {
                CoreLibBaseTypeId::Boolean => Ok("boolean".to_string()),
                CoreLibBaseTypeId::Text => Ok("string".to_string()),
                CoreLibBaseTypeId::Integer => Ok("number".to_string()),
                CoreLibBaseTypeId::Decimal => Ok("number".to_string()),
                CoreLibBaseTypeId::Null => Ok("null".to_string()),
                CoreLibBaseTypeId::Endpoint => Ok("Endpoint".to_string()),
                CoreLibBaseTypeId::Unit => Ok("void".to_string()),
                CoreLibBaseTypeId::Never => Ok("never".to_string()),
                CoreLibBaseTypeId::Unknown => Ok("unknown".to_string()),
                CoreLibBaseTypeId::List => Ok("unknown[]".to_string()),
                CoreLibBaseTypeId::Map => {
                    Ok("Map<unknown, unknown>".to_string())
                }
                CoreLibBaseTypeId::Callable => {
                    Ok("(...args: unknown[]) => unknown".to_string())
                }
                CoreLibBaseTypeId::Range => Ok("unknown".to_string()),
                CoreLibBaseTypeId::Type => Ok("unknown".to_string()),
            },
            CoreLibTypeId::Variant(variant) => match variant {
                CoreLibVariantTypeId::Decimal(_)
                | CoreLibVariantTypeId::Integer(_) => Ok("number".to_string()),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use datex_macros_internal::Datex;

    use crate::{
        datex_proxy::DatexProxyTypes,
        runtime::memory::Memory,
        types::visitor::{self, ts::TsTypeFolder},
        values::core_values::endpoint::Endpoint,
    };

    #[derive(Datex, Debug, Clone, PartialEq)]
    struct Example {
        a: u8,
        b: String,
        c: Endpoint,
    }

    #[test]
    fn test_simple_struct() {
        let ty = Example::datex_type(&mut Memory::default());
        println!("Datex type: {:#?}", ty);

        let mut folder = TsTypeFolder::default();

        let root = visitor::fold_type(&mut folder, &ty).unwrap();

        assert_eq!(root, "Example");

        assert_eq!(
            folder.render_declarations(),
            r#"type Example = { "a": number, "b": string, "c": Endpoint };"#
        );
    }

    #[derive(Datex, Debug, Clone, PartialEq)]
    struct WrappedStruct {
        inner: Example,
    }

    #[test]
    fn test_nested_struct() {
        let ty = WrappedStruct::datex_type(&mut Memory::default());
        let mut folder = TsTypeFolder::default();
        let root = visitor::fold_type(&mut folder, &ty).unwrap();

        assert_eq!(root, "WrappedStruct");

        assert_eq!(
            folder.render_declarations(),
            concat!(
                r#"type Example = { "a": number, "b": string, "c": Endpoint };"#,
                "\n",
                r#"type WrappedStruct = { "inner": Example };"#,
            )
        );
    }
}
