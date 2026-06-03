use binrw::{
    BinRead, BinReaderExt, BinResult, BinWrite, Endian,
    io::{Read, Seek, Write},
    meta::{EndianKind, ReadEndian},
};
use num::BigInt;

use crate::values::core_values::integer::Integer;

impl BinWrite for Integer {
    type Args<'a> = ();

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        _endian: Endian,
        _: Self::Args<'_>,
    ) -> BinResult<()> {
        let (sign, bytes) = self.0.to_bytes_be();
        let len = bytes.len() as u32;
        writer.write_all(&[sign as u8])?;
        writer.write_all(&len.to_le_bytes())?;
        writer.write_all(&bytes)?;

        Ok(())
    }
}
impl BinRead for Integer {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        _endian: Endian,
        _: Self::Args<'_>,
    ) -> BinResult<Self> {
        let sign = reader.read_le::<u8>()?;
        let len = reader.read_le::<u32>()? as usize;
        let mut bytes = vec![0; len];
        reader.read_exact(&mut bytes)?;

        let big_int = BigInt::from_bytes_be(
            match sign {
                0 => num::bigint::Sign::Minus,
                1 => num::bigint::Sign::NoSign,
                2 => num::bigint::Sign::Plus,
                _ => {
                    return Err(binrw::Error::AssertFail {
                        pos: reader.stream_position()?,
                        message: "Invalid sign byte".into(),
                    });
                }
            },
            &bytes,
        );
        Ok(Integer(big_int))
    }
}

impl ReadEndian for Integer {
    const ENDIAN: EndianKind = EndianKind::Endian(Endian::Little);
}
