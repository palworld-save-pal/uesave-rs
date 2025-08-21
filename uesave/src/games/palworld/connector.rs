use crate::{Context, Readable, TResult, Writable};
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use serde::{Deserialize, Serialize};
use std::io::{Read, Seek, Write};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PalConnectInfoItem {
    pub connect_to_model_instance_id: uuid::Uuid,
    pub index: u8,
}

impl<R: Read + Seek, V: crate::VersionInfo> Readable<R, V> for PalConnectInfoItem {
    fn read(reader: &mut Context<R, V>) -> TResult<Self> {
        Ok(PalConnectInfoItem {
            connect_to_model_instance_id: uuid::Uuid::read(reader)?,
            index: reader.read_u8()?,
        })
    }
}

impl<W: Write, V: crate::VersionInfo> Writable<W, V> for PalConnectInfoItem {
    fn write(&self, writer: &mut Context<W, V>) -> TResult<()> {
        self.connect_to_model_instance_id.write(writer)?;
        writer.write_u8(self.index)?;
        Ok(())
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PalConnect {
    pub index: u8,
    pub any_place: Vec<PalConnectInfoItem>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PalConnector {
    pub supported_level: i32,
    pub connect: PalConnect,
    pub unknown_bytes: Vec<u8>,
}

impl<R: Read + Seek, V: crate::VersionInfo> Readable<R, V> for PalConnector {
    fn read(reader: &mut Context<R, V>) -> TResult<Self> {
        let supported_level = reader.read_i32::<LE>()?;
        let connect_index = reader.read_u8()?;

        let any_place_count = reader.read_u32::<LE>()?;
        let mut any_place = Vec::with_capacity(any_place_count as usize);

        for _ in 0..any_place_count {
            any_place.push(PalConnectInfoItem::read(reader)?);
        }

        let mut unknown_bytes = Vec::new();
        reader.read_to_end(&mut unknown_bytes)?;

        Ok(PalConnector {
            supported_level,
            connect: PalConnect {
                index: connect_index,
                any_place,
            },
            unknown_bytes,
        })
    }
}

impl<W: Write, V: crate::VersionInfo> Writable<W, V> for PalConnector {
    fn write(&self, writer: &mut Context<W, V>) -> TResult<()> {
        writer.write_i32::<LE>(self.supported_level)?;
        writer.write_u8(self.connect.index)?;

        writer.write_u32::<LE>(self.connect.any_place.len() as u32)?;
        for item in &self.connect.any_place {
            item.write(writer)?;
        }

        writer.write_all(&self.unknown_bytes)?;
        Ok(())
    }
}
