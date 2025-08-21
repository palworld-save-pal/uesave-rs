use crate::{Context, Readable, TResult, Writable};
use byteorder::{ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use std::io::{Read, Seek, Write};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PalBuildProcess {
    pub state: u8,
    pub id: uuid::Uuid,
    pub trailing_bytes: [u8; 4],
}

impl<R: Read + Seek, V: crate::VersionInfo> Readable<R, V> for PalBuildProcess {
    fn read(reader: &mut Context<R, V>) -> TResult<Self> {
        let state = reader.read_u8()?;
        let id = uuid::Uuid::read(reader)?;

        let mut trailing_bytes = [0u8; 4];
        reader.read_exact(&mut trailing_bytes)?;

        let mut remaining = Vec::new();
        if reader.read_to_end(&mut remaining)? > 0 {
            return Err(crate::Error::Other("Warning: EOF not reached".to_string()));
        }

        Ok(PalBuildProcess {
            state,
            id,
            trailing_bytes,
        })
    }
}

impl<W: Write, V: crate::VersionInfo> Writable<W, V> for PalBuildProcess {
    fn write(&self, writer: &mut Context<W, V>) -> TResult<()> {
        writer.write_u8(self.state)?;
        self.id.write(writer)?;
        writer.write_all(&self.trailing_bytes)?;
        Ok(())
    }
}
