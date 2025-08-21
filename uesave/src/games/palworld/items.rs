use crate::games::palworld::types::PalDynamicId;
use crate::games::palworld::PalItemId;
use crate::{Context, Readable, TResult, Writable};
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use serde::{Deserialize, Serialize};
use std::io::{Read, Seek, Write};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PalItemContainer {
    pub permission: PalItemContainerPermission,
    pub trailing_unparsed_data: Option<Vec<u8>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PalItemContainerPermission {
    pub type_a: Vec<u8>,
    pub type_b: Vec<u8>,
    pub item_static_ids: Vec<String>,
}

impl<R: Read + Seek, V: crate::VersionInfo> Readable<R, V> for PalItemContainer {
    fn read(reader: &mut Context<R, V>) -> TResult<Self> {
        let type_a_count = reader.read_u32::<LE>()?;
        let mut type_a = Vec::with_capacity(type_a_count as usize);
        for _ in 0..type_a_count {
            type_a.push(reader.read_u8()?);
        }

        let type_b_count = reader.read_u32::<LE>()?;
        let mut type_b = Vec::with_capacity(type_b_count as usize);
        for _ in 0..type_b_count {
            type_b.push(reader.read_u8()?);
        }

        let item_static_ids_count = reader.read_u32::<LE>()?;
        let mut item_static_ids = Vec::with_capacity(item_static_ids_count as usize);
        for _ in 0..item_static_ids_count {
            item_static_ids.push(crate::read_string(reader)?);
        }

        let mut trailing_unparsed_data = Vec::new();
        if reader.read_to_end(&mut trailing_unparsed_data)? > 0 {
            Ok(PalItemContainer {
                permission: PalItemContainerPermission {
                    type_a,
                    type_b,
                    item_static_ids,
                },
                trailing_unparsed_data: Some(trailing_unparsed_data),
            })
        } else {
            Ok(PalItemContainer {
                permission: PalItemContainerPermission {
                    type_a,
                    type_b,
                    item_static_ids,
                },
                trailing_unparsed_data: None,
            })
        }
    }
}

impl<W: Write, V: crate::VersionInfo> Writable<W, V> for PalItemContainer {
    fn write(&self, writer: &mut Context<W, V>) -> TResult<()> {
        writer.write_u32::<LE>(self.permission.type_a.len() as u32)?;
        for byte in &self.permission.type_a {
            writer.write_u8(*byte)?;
        }

        writer.write_u32::<LE>(self.permission.type_b.len() as u32)?;
        for byte in &self.permission.type_b {
            writer.write_u8(*byte)?;
        }

        writer.write_u32::<LE>(self.permission.item_static_ids.len() as u32)?;
        for id in &self.permission.item_static_ids {
            crate::write_string(writer, id)?;
        }

        if let Some(trailing_data) = &self.trailing_unparsed_data {
            writer.write_all(trailing_data)?;
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PalItemContainerSlot {
    pub slot_index: i32,
    pub count: i32,
    pub item: PalItemId,
    pub trailing_bytes: Vec<u8>,
}

impl<R: Read + Seek, V: crate::VersionInfo> Readable<R, V> for PalItemContainerSlot {
    fn read(reader: &mut Context<R, V>) -> TResult<Self> {
        let slot_index = reader.read_i32::<LE>()?;
        let count = reader.read_i32::<LE>()?;
        let item = PalItemId::read(reader)?;

        let mut trailing_bytes = Vec::new();
        reader.read_to_end(&mut trailing_bytes)?;

        Ok(PalItemContainerSlot {
            slot_index,
            count,
            item,
            trailing_bytes,
        })
    }
}

impl<W: Write, V: crate::VersionInfo> Writable<W, V> for PalItemContainerSlot {
    fn write(&self, writer: &mut Context<W, V>) -> TResult<()> {
        writer.write_i32::<LE>(self.slot_index)?;
        writer.write_i32::<LE>(self.count)?;
        self.item.write(writer)?;
        writer.write_all(&self.trailing_bytes)?;
        Ok(())
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PalDynamicItem {
    pub id: PalDynamicId,
    pub static_id: String,
    pub item_type: PalDynamicItemType,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum PalDynamicItemType {
    Unknown {
        trailer: Vec<u8>,
    },
    Egg {
        leading_bytes: [u8; 4],
        character_id: String,
        object: crate::Properties,
        trailing_bytes: [u8; 28],
    },
    Armor {
        leading_bytes: [u8; 4],
        durability: f32,
        trailing_bytes: [u8; 4],
    },
    Weapon {
        leading_bytes: [u8; 4],
        durability: f32,
        remaining_bullets: i32,
        passive_skill_list: Vec<String>,
        trailing_bytes: [u8; 4],
    },
}

impl<R: Read + Seek, V: crate::VersionInfo> Readable<R, V> for PalDynamicItem {
    fn read(reader: &mut Context<R, V>) -> TResult<Self> {
        let id = PalDynamicId::read(reader)?;
        let static_id = crate::read_string(reader)?;

        let remaining_start = reader.stream_position()?;

        if let Ok(egg_data) = try_parse_egg(reader) {
            return Ok(PalDynamicItem {
                id,
                static_id,
                item_type: egg_data,
            });
        }

        reader.seek(std::io::SeekFrom::Start(remaining_start))?;
        if let Ok(weapon_data) = try_parse_weapon(reader) {
            return Ok(PalDynamicItem {
                id,
                static_id,
                item_type: weapon_data,
            });
        }

        reader.seek(std::io::SeekFrom::Start(remaining_start))?;
        if let Ok(armor_data) = try_parse_armor(reader) {
            return Ok(PalDynamicItem {
                id,
                static_id,
                item_type: armor_data,
            });
        }

        reader.seek(std::io::SeekFrom::Start(remaining_start))?;
        let mut trailer = Vec::new();
        reader.read_to_end(&mut trailer)?;

        Ok(PalDynamicItem {
            id,
            static_id,
            item_type: PalDynamicItemType::Unknown { trailer },
        })
    }
}

impl<W: Write, V: crate::VersionInfo> Writable<W, V> for PalDynamicItem {
    fn write(&self, writer: &mut Context<W, V>) -> TResult<()> {
        self.id.write(writer)?;
        crate::write_string(writer, &self.static_id)?;

        match &self.item_type {
            PalDynamicItemType::Unknown { trailer } => {
                writer.write_all(trailer)?;
            }
            PalDynamicItemType::Egg {
                leading_bytes,
                character_id,
                object,
                trailing_bytes,
            } => {
                writer.write_all(leading_bytes)?;
                crate::write_string(writer, character_id)?;
                crate::write_properties_none_terminated(writer, object)?;
                writer.write_all(trailing_bytes)?;
            }
            PalDynamicItemType::Armor {
                leading_bytes,
                durability,
                trailing_bytes,
            } => {
                writer.write_all(leading_bytes)?;
                writer.write_f32::<LE>(*durability)?;
                writer.write_all(trailing_bytes)?;
            }
            PalDynamicItemType::Weapon {
                leading_bytes,
                durability,
                remaining_bullets,
                passive_skill_list,
                trailing_bytes,
            } => {
                writer.write_all(leading_bytes)?;
                writer.write_f32::<LE>(*durability)?;
                writer.write_i32::<LE>(*remaining_bullets)?;

                writer.write_u32::<LE>(passive_skill_list.len() as u32)?;
                for skill in passive_skill_list {
                    crate::write_string(writer, skill)?;
                }

                writer.write_all(trailing_bytes)?;
            }
        }

        Ok(())
    }
}

fn try_parse_egg<R: Read + Seek, V: crate::VersionInfo>(
    reader: &mut Context<R, V>,
) -> TResult<PalDynamicItemType> {
    let start_pos = reader.stream_position()?;

    let mut leading_bytes = [0u8; 4];
    reader.read_exact(&mut leading_bytes)?;

    let character_id = crate::read_string(reader)?;
    let object = crate::read_properties_until_none(reader)?;

    let mut trailing_bytes = [0u8; 28];
    reader.read_exact(&mut trailing_bytes)?;

    let mut test_byte = [0u8; 1];
    if reader.read(&mut test_byte)? != 0 {
        reader.seek(std::io::SeekFrom::Start(start_pos))?;
        return Err(crate::Error::Other("Not an egg".to_string()));
    }

    Ok(PalDynamicItemType::Egg {
        leading_bytes,
        character_id,
        object,
        trailing_bytes,
    })
}

fn try_parse_weapon<R: Read + Seek, V: crate::VersionInfo>(
    reader: &mut Context<R, V>,
) -> TResult<PalDynamicItemType> {
    let start_pos = reader.stream_position()?;

    let mut leading_bytes = [0u8; 4];
    reader.read_exact(&mut leading_bytes)?;

    let durability = reader.read_f32::<LE>()?;
    let remaining_bullets = reader.read_i32::<LE>()?;

    let skill_count = reader.read_u32::<LE>()?;
    let mut passive_skill_list = Vec::with_capacity(skill_count as usize);
    for _ in 0..skill_count {
        passive_skill_list.push(crate::read_string(reader)?);
    }

    let mut trailing_bytes = [0u8; 4];
    reader.read_exact(&mut trailing_bytes)?;

    let mut test_byte = [0u8; 1];
    if reader.read(&mut test_byte)? != 0 {
        reader.seek(std::io::SeekFrom::Start(start_pos))?;
        return Err(crate::Error::Other("Not a weapon".to_string()));
    }

    Ok(PalDynamicItemType::Weapon {
        leading_bytes,
        durability,
        remaining_bullets,
        passive_skill_list,
        trailing_bytes,
    })
}

fn try_parse_armor<R: Read + Seek, V: crate::VersionInfo>(
    reader: &mut Context<R, V>,
) -> TResult<PalDynamicItemType> {
    let start_pos = reader.stream_position()?;

    let mut leading_bytes = [0u8; 4];
    reader.read_exact(&mut leading_bytes)?;

    let durability = reader.read_f32::<LE>()?;

    let mut trailing_bytes = [0u8; 4];
    reader.read_exact(&mut trailing_bytes)?;

    let mut test_byte = [0u8; 1];
    if reader.read(&mut test_byte)? != 0 {
        reader.seek(std::io::SeekFrom::Start(start_pos))?;
        return Err(crate::Error::Other("Not armor".to_string()));
    }

    Ok(PalDynamicItemType::Armor {
        leading_bytes,
        durability,
        trailing_bytes,
    })
}
