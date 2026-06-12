use binrw::{
    BinRead, BinReaderExt, BinResult, BinWrite, Endian,
    io::{Read, Seek, Write},
    meta::{EndianKind, ReadEndian},
};
use num::BigInt;

use crate::values::core_values::decimal::{
    BigDecimalType, Decimal, rational::Rational,
};
use crate::prelude::*;

impl BinRead for Decimal {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        _: Self::Args<'_>,
    ) -> BinResult<Self> {
        // only handle le for now
        if endian != Endian::Little {
            return Err(binrw::Error::AssertFail {
                pos: reader.stream_position().unwrap_or(0),
                message: "Only little-endian is supported for Decimal"
                    .to_string(),
            });
        }
        let big_decimal_type =
            BigDecimalType::try_from(reader.read_le::<u8>()?);

        match big_decimal_type {
            Ok(BigDecimalType::Finite) => {
                let numerator_len = reader.read_le::<u32>()? as usize;
                let denominator_len = reader.read_le::<u32>()? as usize;

                let mut numerator_bytes = vec![0; numerator_len];
                let mut denominator_bytes = vec![0; denominator_len];

                reader.read_exact(&mut numerator_bytes)?;
                reader.read_exact(&mut denominator_bytes)?;

                let numerator = BigInt::from_signed_bytes_le(&numerator_bytes);
                let denominator =
                    BigInt::from_signed_bytes_le(&denominator_bytes);

                Ok(Decimal::Finite(Rational::new(numerator, denominator)))
            }
            Ok(big_decimal_type) => Ok(big_decimal_type.try_into().unwrap()),
            Err(_) => Err(binrw::Error::AssertFail {
                pos: reader.stream_position().unwrap_or(0),
                message: "Invalid BigDecimalType".to_string(),
            }),
        }
    }
}

impl BinWrite for Decimal {
    type Args<'a> = ();

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        _: Self::Args<'_>,
    ) -> BinResult<()> {
        // only handle le for now
        if endian != Endian::Little {
            return Err(binrw::Error::AssertFail {
                pos: writer.stream_position().unwrap_or(0),
                message: "Only little-endian is supported for Decimal"
                    .to_string(),
            });
        }
        // write type
        writer.write_all(&[BigDecimalType::from(self) as u8])?;

        // if finite, add value
        if let Decimal::Finite(value) = self {
            let numerator = value.numer();
            let denominator = value.denom();
            let numerator_bytes = numerator.to_signed_bytes_le();
            let denominator_bytes = denominator.to_signed_bytes_le();
            let numerator_len = numerator_bytes.len() as u32;
            let denominator_len = denominator_bytes.len() as u32;
            // write lengths
            writer.write_all(&numerator_len.to_le_bytes())?;
            writer.write_all(&denominator_len.to_le_bytes())?;
            // write numerator and denominator
            writer.write_all(&numerator_bytes)?;
            writer.write_all(&denominator_bytes)?;
        }

        Ok(())
    }
}

impl ReadEndian for Decimal {
    const ENDIAN: EndianKind = EndianKind::Endian(Endian::Little);
}
