use core::fmt::Display;
use crate::global::protocol_structures::instruction_data::{ImplTypeData, IntegerData, ListData, TextData, TypeReferenceData};
use crate::global::type_instruction_codes::TypeInstructionCode;
use crate::shared_values::pointer_address::PointerAddress;
use crate::values::core_values::r#type::TypeMetadata;

#[derive(Clone, Debug, PartialEq)]
pub enum TypeInstruction {
    ImplType(ImplTypeData),
    SharedTypeReference(TypeReferenceData),
    LiteralText(TextData),
    LiteralInteger(IntegerData),
    List(ListData),
    Range, // TODO #670: add more type instructions
}

impl From<&TypeInstruction> for TypeInstructionCode {
    fn from(instruction: &TypeInstruction) -> Self {
        match instruction {
            TypeInstruction::ImplType(_) => TypeInstructionCode::TYPE_WITH_IMPLS,
            TypeInstruction::SharedTypeReference(_) => TypeInstructionCode::SHARED_TYPE_REFERENCE,
            TypeInstruction::LiteralText(_) => TypeInstructionCode::TYPE_LITERAL_TEXT,
            TypeInstruction::LiteralInteger(_) => TypeInstructionCode::TYPE_LITERAL_INTEGER,
            TypeInstruction::List(_) => TypeInstructionCode::TYPE_LIST,
            TypeInstruction::Range => TypeInstructionCode::TYPE_RANGE,
        }
    }
}

impl Display for TypeInstruction {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let code = TypeInstructionCode::from(self);
        write!(f, "{} ", code)?;

        match self {
            TypeInstruction::LiteralText(data) => {
                write!(f, "{}", data.0)
            }
            TypeInstruction::LiteralInteger(data) => {
                write!(f, "{}", data.0)
            }
            TypeInstruction::List(data) => {
                write!(f, "{}", data.element_count)
            }
            TypeInstruction::SharedTypeReference(reference_data) => {
                write!(
                    f,
                    "(mutability: {:?}, address: {})",
                    TypeMetadata::from(&reference_data.metadata),
                    PointerAddress::from(&reference_data.address)
                )
            }
            TypeInstruction::ImplType(data) => {
                write!(f, "({} impls)", data.impl_count)
            }
            _ => {
                // no custom disassembly
                Ok(())
            }
        }
    }
}
