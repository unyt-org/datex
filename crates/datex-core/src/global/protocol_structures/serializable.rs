use crate::prelude::*;
use binrw::{BinWrite, io::Cursor};
pub trait Serializable: BinWrite {
    fn to_bytes(&self) -> Vec<u8>
    where
        for<'a> Self::Args<'a>: Default,
    {
        let mut writer = Cursor::new(Vec::new());
        self.write_le(&mut writer)
            .expect("Failed to write to buffer");
        writer.into_inner()
    }
}
