use crate::{
    global::protocol_structures::instruction_data::StackIndex, prelude::*,
};
use binrw::{
    BinRead, BinResult, BinWrite, Endian,
    io::{Read, Seek, Write},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SharedInjectedValueType {
    // shared x
    Move,
    // 'shared x
    Ref,
    // 'mut shared mut x
    RefMut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LocalInjectedValueType {
    /// The value is moved into the child scope and no longer used afterward
    Move,
    /// The value is moved into the child scope but still used afterward (clone or immutable ref (&x))
    Copy,
    /// The value is temporarily borrowed in the child scope - the changed value must be written back to the parent scope afterward
    RefMut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InjectedValueType {
    Local(LocalInjectedValueType),
    Shared(SharedInjectedValueType),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, BinRead, BinWrite)]
pub struct InjectedValueDeclaration {
    pub(crate) index: StackIndex,
    pub(crate) ty: InjectedValueType,
}

impl From<InjectedValueType> for u8 {
    fn from(injected_value_type: InjectedValueType) -> Self {
        match injected_value_type {
            InjectedValueType::Local(local_type) => match local_type {
                LocalInjectedValueType::Move => 0,
                LocalInjectedValueType::Copy => 1,
                LocalInjectedValueType::RefMut => 2,
            },
            InjectedValueType::Shared(shared_type) => match shared_type {
                SharedInjectedValueType::Move => 3,
                SharedInjectedValueType::Ref => 4,
                SharedInjectedValueType::RefMut => 5,
            },
        }
    }
}

impl TryFrom<u8> for InjectedValueType {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(InjectedValueType::Local(LocalInjectedValueType::Move)),
            1 => Ok(InjectedValueType::Local(LocalInjectedValueType::Copy)),
            2 => Ok(InjectedValueType::Local(LocalInjectedValueType::RefMut)),
            3 => Ok(InjectedValueType::Shared(SharedInjectedValueType::Move)),
            4 => Ok(InjectedValueType::Shared(SharedInjectedValueType::Ref)),
            5 => Ok(InjectedValueType::Shared(SharedInjectedValueType::RefMut)),
            _ => Err(()),
        }
    }
}

impl BinWrite for InjectedValueType {
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
                message:
                    "Only little-endian is supported for InjectedValueType"
                        .to_string(),
            });
        }
        // write type
        writer.write_all(&[u8::from(*self)])?;

        Ok(())
    }
}

impl BinRead for InjectedValueType {
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
                message:
                    "Only little-endian is supported for InjectedValueType"
                        .to_string(),
            });
        }
        // read type
        let mut type_buf = [0u8; 1];
        reader.read_exact(&mut type_buf)?;
        let value_type =
            InjectedValueType::try_from(type_buf[0]).map_err(|_| {
                binrw::Error::AssertFail {
                    pos: reader.stream_position().unwrap_or(0),
                    message: format!(
                        "Invalid InjectedValueType value: {}",
                        type_buf[0]
                    ),
                }
            })?;
        Ok(value_type)
    }
}
