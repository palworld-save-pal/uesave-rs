use crate::games::palworld::types::PalTransform;
use crate::{Context, Readable, TResult, Writable};
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use serde::{Deserialize, Serialize};
use std::io::{Read, Seek, Write};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PalBaseCamp {
    pub id: uuid::Uuid,
    pub name: String,
    pub state: u8,
    pub transform: PalTransform,
    pub area_range: f32,
    pub group_id_belong_to: uuid::Uuid,
    pub fast_travel_local_transform: PalTransform,
    pub owner_map_object_instance_id: uuid::Uuid,
    pub trailing_bytes: [u8; 4],
}

impl<R: Read + Seek, V: crate::VersionInfo> Readable<R, V> for PalBaseCamp {
    fn read(reader: &mut Context<R, V>) -> TResult<Self> {
        Ok(PalBaseCamp {
            id: uuid::Uuid::read(reader)?,
            name: crate::read_string(reader)?,
            state: reader.read_u8()?,
            transform: PalTransform::read(reader)?,
            area_range: reader.read_f32::<LE>()?,
            group_id_belong_to: uuid::Uuid::read(reader)?,
            fast_travel_local_transform: PalTransform::read(reader)?,
            owner_map_object_instance_id: uuid::Uuid::read(reader)?,
            trailing_bytes: {
                let mut bytes = [0u8; 4];
                reader.read_exact(&mut bytes)?;
                bytes
            },
        })
    }
}

impl<W: Write, V: crate::VersionInfo> Writable<W, V> for PalBaseCamp {
    fn write(&self, writer: &mut Context<W, V>) -> TResult<()> {
        self.id.write(writer)?;
        crate::write_string(writer, &self.name)?;
        writer.write_u8(self.state)?;
        self.transform.write(writer)?;
        writer.write_f32::<LE>(self.area_range)?;
        self.group_id_belong_to.write(writer)?;
        self.fast_travel_local_transform.write(writer)?;
        self.owner_map_object_instance_id.write(writer)?;
        writer.write_all(&self.trailing_bytes)?;
        Ok(())
    }
}
