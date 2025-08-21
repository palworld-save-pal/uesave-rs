use crate::{Context, Readable, TResult, Writable};
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use serde::{Deserialize, Serialize};
use std::io::{Read, Seek, Write};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PalItemId {
    pub static_id: String,
    pub dynamic_id: PalDynamicId,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PalDynamicId {
    pub created_world_id: uuid::Uuid,
    pub local_id_in_created_world: uuid::Uuid,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PalItemAndNum {
    pub item_id: PalItemId,
    pub num: u32,
}

impl<R: Read + Seek, V: crate::VersionInfo> Readable<R, V> for PalItemId {
    fn read(reader: &mut Context<R, V>) -> TResult<Self> {
        Ok(PalItemId {
            static_id: crate::read_string(reader)?,
            dynamic_id: PalDynamicId::read(reader)?,
        })
    }
}

impl<W: Write, V: crate::VersionInfo> Writable<W, V> for PalItemId {
    fn write(&self, writer: &mut Context<W, V>) -> TResult<()> {
        crate::write_string(writer, &self.static_id)?;
        self.dynamic_id.write(writer)?;
        Ok(())
    }
}

impl<R: Read + Seek, V: crate::VersionInfo> Readable<R, V> for PalDynamicId {
    fn read(reader: &mut Context<R, V>) -> TResult<Self> {
        Ok(PalDynamicId {
            created_world_id: uuid::Uuid::read(reader)?,
            local_id_in_created_world: uuid::Uuid::read(reader)?,
        })
    }
}

impl<W: Write, V: crate::VersionInfo> Writable<W, V> for PalDynamicId {
    fn write(&self, writer: &mut Context<W, V>) -> TResult<()> {
        self.created_world_id.write(writer)?;
        self.local_id_in_created_world.write(writer)?;
        Ok(())
    }
}

impl<R: Read + Seek, V: crate::VersionInfo> Readable<R, V> for PalItemAndNum {
    fn read(reader: &mut Context<R, V>) -> TResult<Self> {
        Ok(PalItemAndNum {
            item_id: PalItemId::read(reader)?,
            num: reader.read_u32::<LE>()?,
        })
    }
}

impl<W: Write, V: crate::VersionInfo> Writable<W, V> for PalItemAndNum {
    fn write(&self, writer: &mut Context<W, V>) -> TResult<()> {
        self.item_id.write(writer)?;
        writer.write_u32::<LE>(self.num)?;
        Ok(())
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PalInstanceId {
    pub guid: uuid::Uuid,
    pub instance_id: uuid::Uuid,
}

impl<R: Read + Seek, V: crate::VersionInfo> Readable<R, V> for PalInstanceId {
    fn read(reader: &mut Context<R, V>) -> TResult<Self> {
        Ok(PalInstanceId {
            guid: uuid::Uuid::read(reader)?,
            instance_id: uuid::Uuid::read(reader)?,
        })
    }
}

impl<W: Write, V: crate::VersionInfo> Writable<W, V> for PalInstanceId {
    fn write(&self, writer: &mut Context<W, V>) -> TResult<()> {
        self.guid.write(writer)?;
        self.instance_id.write(writer)?;
        Ok(())
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PalPlayerInfo {
    pub player_uid: uuid::Uuid,
    pub player_info: PalPlayerInfoDetails,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PalPlayerInfoDetails {
    pub last_online_real_time: i64,
    pub player_name: String,
}

impl<R: Read + Seek, V: crate::VersionInfo> Readable<R, V> for PalPlayerInfo {
    fn read(reader: &mut Context<R, V>) -> TResult<Self> {
        Ok(PalPlayerInfo {
            player_uid: uuid::Uuid::read(reader)?,
            player_info: PalPlayerInfoDetails {
                last_online_real_time: reader.read_i64::<LE>()?,
                player_name: crate::read_string(reader)?,
            },
        })
    }
}

impl<W: Write, V: crate::VersionInfo> Writable<W, V> for PalPlayerInfo {
    fn write(&self, writer: &mut Context<W, V>) -> TResult<()> {
        self.player_uid.write(writer)?;
        writer.write_i64::<LE>(self.player_info.last_online_real_time)?;
        crate::write_string(writer, &self.player_info.player_name)?;
        Ok(())
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PalTransform {
    pub rotation: crate::Quat,
    pub translation: crate::Vector,
    pub scale: crate::Vector,
}

impl<R: Read + Seek, V: crate::VersionInfo> Readable<R, V> for PalTransform {
    fn read(reader: &mut Context<R, V>) -> TResult<Self> {
        Ok(PalTransform {
            rotation: crate::Quat::read(reader)?,
            translation: crate::Vector::read(reader)?,
            scale: crate::Vector::read(reader)?,
        })
    }
}

impl<W: Write, V: crate::VersionInfo> Writable<W, V> for PalTransform {
    fn write(&self, writer: &mut Context<W, V>) -> TResult<()> {
        self.rotation.write(writer)?;
        self.translation.write(writer)?;
        self.scale.write(writer)?;
        Ok(())
    }
}
