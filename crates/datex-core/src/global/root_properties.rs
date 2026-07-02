use binrw::{BinRead, BinWrite};
use num_enum::TryFromPrimitive;
use strum::Display;
use strum_macros::EnumString;

/// internal slots address space, starting at 0xffffff_00
#[derive(
    BinRead,
    BinWrite,
    Debug,
    Eq,
    PartialEq,
    TryFromPrimitive,
    Copy,
    Clone,
    Display,
    num_enum::IntoPrimitive,
    EnumString,
)]
#[brw(little, repr(u8))]
#[strum(serialize_all = "lowercase")]
#[repr(u8)]
pub enum RootProperty {
    ENDPOINT = 0x01,
    ENV = 0x02,
    CALLER = 0x03,
    CONFIG = 0x04,
}
