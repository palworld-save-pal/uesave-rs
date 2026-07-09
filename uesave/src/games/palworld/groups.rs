use crate::games::palworld::types::PalInstanceId;
use crate::{ArchiveReader, ArchiveWriter, FGuid, Result};
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PalGroupData {
    pub group_id: FGuid,
    pub group_name: String,
    pub individual_character_handle_ids: Vec<PalInstanceId>,
    pub remaining_data: Vec<u8>,
}

impl PalGroupData {
    pub fn read<A: ArchiveReader>(ar: &mut A) -> Result<Self> {
        let group_id = FGuid::read(ar)?;
        let group_name = ar.read_string()?;

        let handle_ids_count = ar.read_u32::<LE>()?;
        let individual_character_handle_ids =
            crate::read_array(handle_ids_count, ar, PalInstanceId::read)?;

        let mut remaining_data = Vec::new();
        ar.read_to_end(&mut remaining_data)?;

        Ok(PalGroupData {
            group_id,
            group_name,
            individual_character_handle_ids,
            remaining_data,
        })
    }
    pub fn write<A: ArchiveWriter>(&self, ar: &mut A) -> Result<()> {
        self.group_id.write(ar)?;
        ar.write_string(&self.group_name)?;

        ar.write_u32::<LE>(self.individual_character_handle_ids.len() as u32)?;
        for handle_id in &self.individual_character_handle_ids {
            handle_id.write(ar)?;
        }

        ar.write_all(&self.remaining_data)?;

        Ok(())
    }
}
