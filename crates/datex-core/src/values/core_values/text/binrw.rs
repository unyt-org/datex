use crate::{
    instruction::instruction_data::TextData, prelude::*,
    values::core_values::text::Text,
};
use binrw::{
    BinRead, BinResult, BinWrite, Endian,
    io::{Read, Seek, Write},
};

impl BinRead for Text {
    type Args<'a> = <TextData as BinRead>::Args<'a>;

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<Self> {
        TextData::read_options(reader, endian, args).map(|t| Text::String(t.0))
    }
}

impl BinWrite for Text {
    type Args<'a> = <TextData as BinWrite>::Args<'a>;

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<()> {
        TextData(self.as_str().to_owned()).write_options(writer, endian, args)
    }
}
