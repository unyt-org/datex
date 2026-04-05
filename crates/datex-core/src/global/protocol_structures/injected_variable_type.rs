use binrw::io::{Read, Seek, Write};
use binrw::{BinRead, BinResult, BinWrite, Endian};
use crate::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SharedInjectedVariableType {
    // shared x
    Move,
    // 'shared x
    Ref,
    // 'mut shared mut x
    RefMut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LocalInjectedVariableType {
    /// The value is moved into the child scope and no longer used afterward
    Move,
    /// The value is moved into the child scope but still used afterward (clone or immutable ref (&x))
    Copy,
    /// The value is temporarily borrowed in the child scope - the changed value must be written back to the parent scope afterward
    RefMut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InjectedVariableType {
    Local(LocalInjectedVariableType),
    Shared(SharedInjectedVariableType),
}

impl From<InjectedVariableType> for u8 {
    fn from(injected_value_type: InjectedVariableType) -> Self {
        match injected_value_type {
            InjectedVariableType::Local(local_type) => match local_type {
                LocalInjectedVariableType::Move => 0,
                LocalInjectedVariableType::Copy => 1,
                LocalInjectedVariableType::RefMut => 2,
            },
            InjectedVariableType::Shared(shared_type) => match shared_type {
                SharedInjectedVariableType::Move => 3,
                SharedInjectedVariableType::Ref => 4,
                SharedInjectedVariableType::RefMut => 5,
            },
        }
    }
}

impl TryFrom<u8> for InjectedVariableType {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(InjectedVariableType::Local(LocalInjectedVariableType::Move)),
            1 => Ok(InjectedVariableType::Local(LocalInjectedVariableType::Copy)),
            2 => Ok(InjectedVariableType::Local(LocalInjectedVariableType::RefMut)),
            3 => Ok(InjectedVariableType::Shared(SharedInjectedVariableType::Move)),
            4 => Ok(InjectedVariableType::Shared(SharedInjectedVariableType::Ref)),
            5 => Ok(InjectedVariableType::Shared(SharedInjectedVariableType::RefMut)),
            _ => Err(()),
        }
    }
}

impl BinWrite for InjectedVariableType {
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
                message: "Only little-endian is supported for InjectedVariableType"
                    .to_string(),
            });
        }
        // write type
        writer.write_all(&[u8::from(*self)])?;

        Ok(())
    }
}

impl BinRead for InjectedVariableType {
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
                message: "Only little-endian is supported for InjectedVariableType"
                    .to_string(),
            });
        }
        // read type
        let mut type_buf = [0u8; 1];
        reader.read_exact(&mut type_buf)?;
        let value_type = InjectedVariableType::try_from(type_buf[0]).map_err(|_| binrw::Error::AssertFail {
            pos: reader.stream_position().unwrap_or(0),
            message: format!("Invalid InjectedVariableType value: {}", type_buf[0]),
        })?;
        Ok(value_type)
    }
}