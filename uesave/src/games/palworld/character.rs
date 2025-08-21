use crate::{Context, Properties, Readable, TResult, Writable};
use byteorder::ReadBytesExt;
use serde::{Deserialize, Serialize};
use std::io::{Read, Seek, Write};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PalCharacterData {
    pub object: Properties,
    pub unknown_bytes: [u8; 4],
    pub group_id: uuid::Uuid,
    pub trailing_bytes: [u8; 4],
}

impl<R: Read + Seek, V: crate::VersionInfo> Readable<R, V> for PalCharacterData {
    fn read(reader: &mut Context<R, V>) -> TResult<Self> {
        Ok(PalCharacterData {
            object: crate::read_properties_until_none(reader)?,
            unknown_bytes: {
                let mut bytes = [0; 4];
                reader.read_exact(&mut bytes)?;
                bytes
            },
            group_id: uuid::Uuid::read(reader)?,
            trailing_bytes: {
                let mut bytes = [0; 4];
                reader.read_exact(&mut bytes)?;
                bytes
            },
        })
    }
}

impl<W: Write, V: crate::VersionInfo> Writable<W, V> for PalCharacterData {
    fn write(&self, writer: &mut Context<W, V>) -> TResult<()> {
        crate::write_properties_none_terminated(writer, &self.object)?;
        writer.write_all(&self.unknown_bytes)?;
        self.group_id.write(writer)?;
        writer.write_all(&self.trailing_bytes)?;
        Ok(())
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PalCharacterContainer {
    pub player_uid: uuid::Uuid,
    pub instance_id: uuid::Uuid,
    pub permission_tribe_id: u8,
    pub trailing_bytes: Option<Vec<u8>>,
}

impl<R: Read + Seek, V: crate::VersionInfo> Readable<R, V> for PalCharacterContainer {
    fn read(reader: &mut Context<R, V>) -> TResult<Self> {
        let player_uid = uuid::Uuid::read(reader)?;
        let instance_id = uuid::Uuid::read(reader)?;
        let permission_tribe_id = reader.read_u8()?;
        let mut trailing_bytes = Vec::new();
        if reader.read_to_end(&mut trailing_bytes)? > 0 {
            Ok(PalCharacterContainer {
                player_uid,
                instance_id,
                permission_tribe_id,
                trailing_bytes: Some(trailing_bytes),
            })
        } else {
            Ok(PalCharacterContainer {
                player_uid,
                instance_id,
                permission_tribe_id,
                trailing_bytes: None,
            })
        }
    }
}

impl<W: Write, V: crate::VersionInfo> Writable<W, V> for PalCharacterContainer {
    fn write(&self, writer: &mut Context<W, V>) -> TResult<()> {
        self.player_uid.write(writer)?;
        self.instance_id.write(writer)?;
        writer.write_all(&[self.permission_tribe_id])?;
        if let Some(trailing) = &self.trailing_bytes {
            writer.write_all(trailing)?;
        }
        Ok(())
    }
}
