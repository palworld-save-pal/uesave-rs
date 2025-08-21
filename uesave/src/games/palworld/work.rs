use super::types::PalInstanceId;
use crate::{Context, Quat, Readable, TResult, Vector, Writable};
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use serde::{Deserialize, Serialize};
use std::io::{Read, Seek, Write};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PalWork {
    pub work_type: String,
    pub base_data: Option<PalWorkBase>,
    pub assign_data: Option<PalWorkAssign>,
    pub transform: PalWorkTransform,
    pub work_specific_data: PalWorkTypeSpecificData,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PalWorkBase {
    pub id: uuid::Uuid,
    pub workable_bounds: PalWorkableBounds,
    pub base_camp_id_belong_to: uuid::Uuid,
    pub owner_map_object_model_id: uuid::Uuid,
    pub owner_map_object_concrete_model_id: uuid::Uuid,
    pub current_state: u8,
    pub assign_locations: Vec<PalAssignLocation>,
    pub behaviour_type: u8,
    pub assign_define_data_id: String,
    pub override_work_type: u8,
    pub assignable_fixed_type: u8,
    pub assignable_otomo: bool,
    pub can_trigger_worker_event: bool,
    pub can_steal_assign: bool,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PalWorkAssign {
    pub handle_id: uuid::Uuid,
    pub location_index: i32,
    pub assign_type: u8,
    pub assigned_individual_id: PalInstanceId,
    pub state: u8,
    pub fixed: u32,
    pub target_map_object_model_id: Option<uuid::Uuid>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PalWorkableBounds {
    pub location: Vector,
    pub rotation: Quat,
    pub box_sphere_bounds: PalBoxSphereBounds,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PalBoxSphereBounds {
    pub origin: Vector,
    pub box_extent: Vector,
    pub sphere_radius: f64,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PalAssignLocation {
    pub location: Vector,
    pub facing_direction: Vector,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PalWorkTransform {
    pub transform_type: u8,
    pub map_object_instance_id: Option<uuid::Uuid>,
    pub trailing_bytes: Option<[u8; 8]>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum PalWorkTypeSpecificData {
    Unknown {
        data: Vec<u8>,
    },
    Defense {
        leading_bytes: [u8; 4],
        defense_combat_type: u8,
        trailing_bytes: [u8; 4],
    },
    Progress {
        required_work_amount: f32,
        current_work_amount: f32,
        work_exp: i32,
        work_exp_calc_type: u8,
        auto_work_self_amount_by_sec: f32,
        progress_time_since_last_tick: f32,
        tick_process_min_interval: f32,
    },
    ReviveCharacter {
        target_individual_id: PalInstanceId,
    },
    SimpleWork {
        required_work_amount: f32,
    },
}

impl<R: Read + Seek, V: crate::VersionInfo> Readable<R, V> for PalWork {
    fn read(reader: &mut Context<R, V>) -> TResult<Self> {
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;

        Ok(PalWork {
            work_type: "Unknown".to_string(),
            base_data: None,
            assign_data: None,
            transform: PalWorkTransform {
                transform_type: 0,
                map_object_instance_id: None,
                trailing_bytes: None,
            },
            work_specific_data: PalWorkTypeSpecificData::Unknown { data },
        })
    }
}

impl<W: Write, V: crate::VersionInfo> Writable<W, V> for PalWork {
    fn write(&self, writer: &mut Context<W, V>) -> TResult<()> {
        if let PalWorkTypeSpecificData::Unknown { data } = &self.work_specific_data {
            writer.write_all(data)?;
        }
        Ok(())
    }
}

impl<R: Read + Seek, V: crate::VersionInfo> Readable<R, V> for PalWorkableBounds {
    fn read(reader: &mut Context<R, V>) -> TResult<Self> {
        Ok(PalWorkableBounds {
            location: Vector::read(reader)?,
            rotation: Quat::read(reader)?,
            box_sphere_bounds: PalBoxSphereBounds::read(reader)?,
        })
    }
}

impl<W: Write, V: crate::VersionInfo> Writable<W, V> for PalWorkableBounds {
    fn write(&self, writer: &mut Context<W, V>) -> TResult<()> {
        self.location.write(writer)?;
        self.rotation.write(writer)?;
        self.box_sphere_bounds.write(writer)?;
        Ok(())
    }
}

impl<R: Read + Seek, V: crate::VersionInfo> Readable<R, V> for PalBoxSphereBounds {
    fn read(reader: &mut Context<R, V>) -> TResult<Self> {
        Ok(PalBoxSphereBounds {
            origin: Vector::read(reader)?,
            box_extent: Vector::read(reader)?,
            sphere_radius: reader.read_f64::<LE>()?,
        })
    }
}

impl<W: Write, V: crate::VersionInfo> Writable<W, V> for PalBoxSphereBounds {
    fn write(&self, writer: &mut Context<W, V>) -> TResult<()> {
        self.origin.write(writer)?;
        self.box_extent.write(writer)?;
        writer.write_f64::<LE>(self.sphere_radius)?;
        Ok(())
    }
}

impl<R: Read + Seek, V: crate::VersionInfo> Readable<R, V> for PalAssignLocation {
    fn read(reader: &mut Context<R, V>) -> TResult<Self> {
        Ok(PalAssignLocation {
            location: Vector::read(reader)?,
            facing_direction: Vector::read(reader)?,
        })
    }
}

impl<W: Write, V: crate::VersionInfo> Writable<W, V> for PalAssignLocation {
    fn write(&self, writer: &mut Context<W, V>) -> TResult<()> {
        self.location.write(writer)?;
        self.facing_direction.write(writer)?;
        Ok(())
    }
}
