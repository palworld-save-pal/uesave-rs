use crate::games::palworld::types::PalInstanceId;
use crate::{Context, Readable, TResult, Writable};
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use serde::{Deserialize, Serialize};
use std::io::{Read, Seek, Write};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PalGroupData {
    pub group_id: uuid::Uuid,
    pub group_name: String,
    pub individual_character_handle_ids: Vec<PalInstanceId>,
    pub remaining_data: Vec<u8>,
}

impl<R: Read + Seek, V: crate::VersionInfo> Readable<R, V> for PalGroupData {
    fn read(reader: &mut Context<R, V>) -> TResult<Self> {
        let group_id = uuid::Uuid::read(reader)?;
        let group_name = crate::read_string(reader)?;

        let handle_ids_count = reader.read_u32::<LE>()?;
        let individual_character_handle_ids =
            crate::read_array(handle_ids_count, reader, PalInstanceId::read)?;

        let mut remaining_data = Vec::new();
        reader.read_to_end(&mut remaining_data)?;

        Ok(PalGroupData {
            group_id,
            group_name,
            individual_character_handle_ids,
            remaining_data,
        })
    }
}

impl<W: Write, V: crate::VersionInfo> Writable<W, V> for PalGroupData {
    fn write(&self, writer: &mut Context<W, V>) -> TResult<()> {
        self.group_id.write(writer)?;
        crate::write_string(writer, &self.group_name)?;

        writer.write_u32::<LE>(self.individual_character_handle_ids.len() as u32)?;
        for handle_id in &self.individual_character_handle_ids {
            handle_id.write(writer)?;
        }

        writer.write_all(&self.remaining_data)?;

        Ok(())
    }
}
