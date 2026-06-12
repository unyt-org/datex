// use binrw::{
//     BinResult, BinWrite, Endian,
//     io::{Read, Seek, Write},
// };
// use strum::IntoDiscriminant;

// use crate::{
//     global::type_instruction_codes::TypeInstructionCode,
//     types::type_definition::TypeDefinition,
// };
// impl BinWrite for TypeDefinition {
//     type Args<'a> = ();

//     fn write_options<W: Write + Seek>(
//         &self,
//         writer: &mut W,
//         endian: Endian,
//         args: Self::Args<'_>,
//     ) -> BinResult<()> {
//         (self.discriminant() as u8).write_options(writer, endian, args)?;
//         match self {
//             TypeDefinition::CoreType(core_lib_id) => {
//                 core_lib_id.write_options(writer, endian, args)
//             }
//             TypeDefinition::Shared(type_ref) => {
//                 core_libtype_ref_id.write_options(writer, endian, args)
//             }
//             TypeDefinition::List(structural_def) => {
//                 structural_def.write_options(writer, endian, args)
//             }
//             TypeDefinition::Map(structural_def) => {
//                 structural_def.write_options(writer, endian, args)
//             }
//             TypeDefinition::(alias_def) => {
//                 alias_def.write_options(writer, endian, args)
//             }
//             TypeDefinition::Impl(impl_def) => {
//                 impl_def.write_options(writer, endian, args)
//             }
//             TypeDefinition::Literal(literal_def) => {
//                 literal_def.write_options(writer, endian, args)
//             }
//         }
//     }
// }

// impl Bin
