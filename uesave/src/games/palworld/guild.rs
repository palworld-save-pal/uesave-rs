use crate::{Context, Readable, TResult, Writable};
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use serde::{Deserialize, Serialize};
use std::io::{Read, Seek, Write};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PalLabResearchInfo {
    pub research_id: String,
    pub work_amount: f32,
}

impl<R: Read + Seek, V: crate::VersionInfo> Readable<R, V> for PalLabResearchInfo {
    fn read(reader: &mut Context<R, V>) -> TResult<Self> {
        Ok(PalLabResearchInfo {
            research_id: crate::read_string(reader)?,
            work_amount: reader.read_f32::<LE>()?,
        })
    }
}

impl<W: Write, V: crate::VersionInfo> Writable<W, V> for PalLabResearchInfo {
    fn write(&self, writer: &mut Context<W, V>) -> TResult<()> {
        crate::write_string(writer, &self.research_id)?;
        writer.write_f32::<LE>(self.work_amount)?;
        Ok(())
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PalGuildLab {
    pub research_info: Vec<PalLabResearchInfo>,
    pub current_research_id: String,
    pub trailing_bytes: Vec<u8>,
}

impl<R: Read + Seek, V: crate::VersionInfo> Readable<R, V> for PalGuildLab {
    fn read(reader: &mut Context<R, V>) -> TResult<Self> {
        let research_count = reader.read_u32::<LE>()?;
        let mut research_info = Vec::with_capacity(research_count as usize);
        for _ in 0..research_count {
            research_info.push(PalLabResearchInfo::read(reader)?);
        }

        let current_research_id = crate::read_string(reader)?;

        let mut trailing_bytes = Vec::new();
        reader.read_to_end(&mut trailing_bytes)?;

        Ok(PalGuildLab {
            research_info,
            current_research_id,
            trailing_bytes,
        })
    }
}

impl<W: Write, V: crate::VersionInfo> Writable<W, V> for PalGuildLab {
    fn write(&self, writer: &mut Context<W, V>) -> TResult<()> {
        writer.write_u32::<LE>(self.research_info.len() as u32)?;
        for info in &self.research_info {
            info.write(writer)?;
        }

        crate::write_string(writer, &self.current_research_id)?;

        writer.write_all(&self.trailing_bytes)?;

        Ok(())
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PalGuildItemStorage {
    pub container_id: uuid::Uuid,
    pub trailing_bytes: Vec<u8>,
}

impl<R: Read + Seek, V: crate::VersionInfo> Readable<R, V> for PalGuildItemStorage {
    fn read(reader: &mut Context<R, V>) -> TResult<Self> {
        let container_id = uuid::Uuid::read(reader)?;

        let mut trailing_bytes = Vec::new();
        reader.read_to_end(&mut trailing_bytes)?;

        Ok(PalGuildItemStorage {
            container_id,
            trailing_bytes,
        })
    }
}

impl<W: Write, V: crate::VersionInfo> Writable<W, V> for PalGuildItemStorage {
    fn write(&self, writer: &mut Context<W, V>) -> TResult<()> {
        self.container_id.write(writer)?;
        writer.write_all(&self.trailing_bytes)?;
        Ok(())
    }
}
