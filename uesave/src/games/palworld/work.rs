use super::types::PalInstanceId;
use crate::{ArchiveReader, ArchiveWriter, FGuid, Quat, Result, Vector};
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PalWork {
    pub work_type: String,
    pub base_data: Option<PalWorkBase>,
    pub assign_data: Option<PalWorkAssign>,
    pub transform: PalWorkTransform,
    pub work_specific_data: PalWorkTypeSpecificData,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PalWorkBase {
    pub id: FGuid,
    pub workable_bounds: PalWorkableBounds,
    pub base_camp_id_belong_to: FGuid,
    pub owner_map_object_model_id: FGuid,
    pub owner_map_object_concrete_model_id: FGuid,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PalWorkAssign {
    pub handle_id: FGuid,
    pub location_index: i32,
    pub assign_type: u8,
    pub assigned_individual_id: PalInstanceId,
    pub state: u8,
    pub fixed: u32,
    pub target_map_object_model_id: Option<FGuid>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PalWorkableBounds {
    pub location: Vector,
    pub rotation: Quat,
    pub box_sphere_bounds: PalBoxSphereBounds,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PalBoxSphereBounds {
    pub origin: Vector,
    pub box_extent: Vector,
    pub sphere_radius: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PalAssignLocation {
    pub location: Vector,
    pub facing_direction: Vector,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PalWorkTransform {
    pub transform_type: u8,
    pub map_object_instance_id: Option<FGuid>,
    pub trailing_bytes: Option<[u8; 8]>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

impl PalWork {
    pub fn read<A: ArchiveReader>(ar: &mut A) -> Result<Self> {
        let mut data = Vec::new();
        ar.read_to_end(&mut data)?;

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
    pub fn write<A: ArchiveWriter>(&self, ar: &mut A) -> Result<()> {
        if let PalWorkTypeSpecificData::Unknown { data } = &self.work_specific_data {
            ar.write_all(data)?;
        }
        Ok(())
    }
}

impl PalWorkableBounds {
    pub fn read<A: ArchiveReader>(ar: &mut A) -> Result<Self> {
        Ok(PalWorkableBounds {
            location: Vector::read(ar)?,
            rotation: Quat::read(ar)?,
            box_sphere_bounds: PalBoxSphereBounds::read(ar)?,
        })
    }
    pub fn write<A: ArchiveWriter>(&self, ar: &mut A) -> Result<()> {
        self.location.write(ar)?;
        self.rotation.write(ar)?;
        self.box_sphere_bounds.write(ar)?;
        Ok(())
    }
}

impl PalBoxSphereBounds {
    pub fn read<A: ArchiveReader>(ar: &mut A) -> Result<Self> {
        Ok(PalBoxSphereBounds {
            origin: Vector::read(ar)?,
            box_extent: Vector::read(ar)?,
            sphere_radius: ar.read_f64::<LE>()?,
        })
    }
    pub fn write<A: ArchiveWriter>(&self, ar: &mut A) -> Result<()> {
        self.origin.write(ar)?;
        self.box_extent.write(ar)?;
        ar.write_f64::<LE>(self.sphere_radius)?;
        Ok(())
    }
}

impl PalAssignLocation {
    pub fn read<A: ArchiveReader>(ar: &mut A) -> Result<Self> {
        Ok(PalAssignLocation {
            location: Vector::read(ar)?,
            facing_direction: Vector::read(ar)?,
        })
    }
    pub fn write<A: ArchiveWriter>(&self, ar: &mut A) -> Result<()> {
        self.location.write(ar)?;
        self.facing_direction.write(ar)?;
        Ok(())
    }
}
