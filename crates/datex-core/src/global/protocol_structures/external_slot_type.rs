use binrw::io::{Read, Seek, Write};
use binrw::{BinRead, BinResult, BinWrite, Endian};
use crate::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SharedSlotType {
    // shared x
    Move,
    // 'shared x
    Ref,
    // 'mut shared mut x
    RefMut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LocalSlotType {
    /// The value is moved into the external slot and no longer used afterward
    Move,
    /// The value is moved into the external slot but still used afterward (clone or immutable ref (&x))
    Copy,
    /// The value is temporarily borrowed in the external slot - the changed value must be written back to the local slot afterward
    RefMut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExternalSlotType {
    Local(LocalSlotType),
    Shared(SharedSlotType),
}

impl From<ExternalSlotType> for u8 {
    fn from(slot_type: ExternalSlotType) -> Self {
        match slot_type {
            ExternalSlotType::Local(local_type) => match local_type {
                LocalSlotType::Move => 0,
                LocalSlotType::Copy => 1,
                LocalSlotType::RefMut => 2,
            },
            ExternalSlotType::Shared(shared_type) => match shared_type {
                SharedSlotType::Move => 3,
                SharedSlotType::Ref => 4,
                SharedSlotType::RefMut => 5,
            },
        }
    }
}

impl TryFrom<u8> for ExternalSlotType {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ExternalSlotType::Local(LocalSlotType::Move)),
            1 => Ok(ExternalSlotType::Local(LocalSlotType::Copy)),
            2 => Ok(ExternalSlotType::Local(LocalSlotType::RefMut)),
            3 => Ok(ExternalSlotType::Shared(SharedSlotType::Move)),
            4 => Ok(ExternalSlotType::Shared(SharedSlotType::Ref)),
            5 => Ok(ExternalSlotType::Shared(SharedSlotType::RefMut)),
            _ => Err(()),
        }
    }
}

impl BinWrite for ExternalSlotType {
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
                message: "Only little-endian is supported for ExternalSlotType"
                    .to_string(),
            });
        }
        // write type
        writer.write_all(&[u8::from(*self)])?;

        Ok(())
    }
}

impl BinRead for ExternalSlotType {
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
                message: "Only little-endian is supported for ExternalSlotType"
                    .to_string(),
            });
        }
        // read type
        let mut type_buf = [0u8; 1];
        reader.read_exact(&mut type_buf)?;
        let slot_type = ExternalSlotType::try_from(type_buf[0]).map_err(|_| binrw::Error::AssertFail {
            pos: reader.stream_position().unwrap_or(0),
            message: format!("Invalid ExternalSlotType value: {}", type_buf[0]),
        })?;
        Ok(slot_type)
    }
}