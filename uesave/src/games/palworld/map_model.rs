use super::types::PalTransform;
use crate::{Context, Readable, TResult, Writable};
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use serde::{Deserialize, Serialize};
use std::io::{Read, Seek, Write};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PalMapObjectHp {
    pub current: i32,
    pub max: i32,
}

impl<R: Read + Seek, V: crate::VersionInfo> Readable<R, V> for PalMapObjectHp {
    fn read(reader: &mut Context<R, V>) -> TResult<Self> {
        Ok(PalMapObjectHp {
            current: reader.read_i32::<LE>()?,
            max: reader.read_i32::<LE>()?,
        })
    }
}

impl<W: Write, V: crate::VersionInfo> Writable<W, V> for PalMapObjectHp {
    fn write(&self, writer: &mut Context<W, V>) -> TResult<()> {
        writer.write_i32::<LE>(self.current)?;
        writer.write_i32::<LE>(self.max)?;
        Ok(())
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PalStageInstanceId {
    pub id: uuid::Uuid,
    pub valid: u32,
}

impl<R: Read + Seek, V: crate::VersionInfo> Readable<R, V> for PalStageInstanceId {
    fn read(reader: &mut Context<R, V>) -> TResult<Self> {
        let id = uuid::Uuid::read(reader)?;
        let valid = reader.read_u32::<LE>()?;
        Ok(PalStageInstanceId {
            id: id,
            valid: valid,
        })
    }
}

impl<W: Write, V: crate::VersionInfo> Writable<W, V> for PalStageInstanceId {
    fn write(&self, writer: &mut Context<W, V>) -> TResult<()> {
        self.id.write(writer)?;
        writer.write_u32::<LE>(self.valid)?;
        Ok(())
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PalMapModel {
    pub instance_id: uuid::Uuid,
    pub concrete_model_instance_id: uuid::Uuid,
    pub base_camp_id_belong_to: uuid::Uuid,
    pub group_id_belong_to: uuid::Uuid,
    pub hp: PalMapObjectHp,
    pub initial_transform_cache: PalTransform,
    pub repair_work_id: uuid::Uuid,
    pub owner_spawner_level_object_instance_id: uuid::Uuid,
    pub owner_instance_id: uuid::Uuid,
    pub build_player_uid: uuid::Uuid,
    pub interact_restrict_type: u8,
    pub deterioration_damage: f32,
    pub stage_instance_id_belong_to: PalStageInstanceId,
    pub unknown_bytes: Vec<u8>,
}

impl<R: Read + Seek, V: crate::VersionInfo> Readable<R, V> for PalMapModel {
    fn read(reader: &mut Context<R, V>) -> TResult<Self> {
        Ok(PalMapModel {
            instance_id: uuid::Uuid::read(reader)?,
            concrete_model_instance_id: uuid::Uuid::read(reader)?,
            base_camp_id_belong_to: uuid::Uuid::read(reader)?,
            group_id_belong_to: uuid::Uuid::read(reader)?,
            hp: PalMapObjectHp::read(reader)?,
            initial_transform_cache: PalTransform::read(reader)?,
            repair_work_id: uuid::Uuid::read(reader)?,
            owner_spawner_level_object_instance_id: uuid::Uuid::read(reader)?,
            owner_instance_id: uuid::Uuid::read(reader)?,
            build_player_uid: uuid::Uuid::read(reader)?,
            interact_restrict_type: reader.read_u8()?,
            deterioration_damage: reader.read_f32::<LE>()?,
            stage_instance_id_belong_to: PalStageInstanceId::read(reader)?,
            unknown_bytes: {
                let mut bytes = Vec::new();
                reader.read_to_end(&mut bytes)?;
                bytes
            },
        })
    }
}

impl<W: Write, V: crate::VersionInfo> Writable<W, V> for PalMapModel {
    fn write(&self, writer: &mut Context<W, V>) -> TResult<()> {
        self.instance_id.write(writer)?;
        self.concrete_model_instance_id.write(writer)?;
        self.base_camp_id_belong_to.write(writer)?;
        self.group_id_belong_to.write(writer)?;
        self.hp.write(writer)?;
        self.initial_transform_cache.write(writer)?;
        self.repair_work_id.write(writer)?;
        self.owner_spawner_level_object_instance_id.write(writer)?;
        self.owner_instance_id.write(writer)?;
        self.build_player_uid.write(writer)?;
        writer.write_u8(self.interact_restrict_type)?;
        writer.write_f32::<LE>(self.deterioration_damage)?;
        self.stage_instance_id_belong_to.write(writer)?;
        writer.write_all(&self.unknown_bytes)?;
        Ok(())
    }
}
