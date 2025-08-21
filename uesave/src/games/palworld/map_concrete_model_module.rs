use crate::{Context, Readable, TResult, Writable};
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use serde::{Deserialize, Serialize};
use std::io::{Read, Seek, Write};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PalModuleSlotIndexes {
    pub attribute: u8,
    pub indexes: Vec<i32>,
}

impl<R: Read + Seek, V: crate::VersionInfo> Readable<R, V> for PalModuleSlotIndexes {
    fn read(reader: &mut Context<R, V>) -> TResult<Self> {
        let attribute = reader.read_u8()?;
        let indexes_count = reader.read_u32::<LE>()?;
        let mut indexes = Vec::with_capacity(indexes_count as usize);
        for _ in 0..indexes_count {
            indexes.push(reader.read_i32::<LE>()?);
        }
        Ok(PalModuleSlotIndexes { attribute, indexes })
    }
}

impl<W: Write, V: crate::VersionInfo> Writable<W, V> for PalModuleSlotIndexes {
    fn write(&self, writer: &mut Context<W, V>) -> TResult<()> {
        writer.write_u8(self.attribute)?;
        writer.write_u32::<LE>(self.indexes.len() as u32)?;
        for index in &self.indexes {
            writer.write_i32::<LE>(*index)?;
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PalPlayerLockInfo {
    pub player_uid: uuid::Uuid,
    pub try_failed_count: i32,
    pub try_success_cache: bool,
}

impl<R: Read + Seek, V: crate::VersionInfo> Readable<R, V> for PalPlayerLockInfo {
    fn read(reader: &mut Context<R, V>) -> TResult<Self> {
        Ok(PalPlayerLockInfo {
            player_uid: uuid::Uuid::read(reader)?,
            try_failed_count: reader.read_i32::<LE>()?,
            try_success_cache: reader.read_u32::<LE>()? > 0,
        })
    }
}

impl<W: Write, V: crate::VersionInfo> Writable<W, V> for PalPlayerLockInfo {
    fn write(&self, writer: &mut Context<W, V>) -> TResult<()> {
        self.player_uid.write(writer)?;
        writer.write_i32::<LE>(self.try_failed_count)?;
        writer.write_u32::<LE>(if self.try_success_cache { 1 } else { 0 })?;
        Ok(())
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum PalMapConcreteModelModuleData {
    ItemContainer {
        target_container_id: uuid::Uuid,
        slot_attribute_indexes: Vec<PalModuleSlotIndexes>,
        all_slot_attribute: Vec<u8>,
        drop_item_at_disposed: bool,
        usage_type: u8,
        trailing_bytes: [u8; 4],
    },
    CharacterContainer {
        target_container_id: uuid::Uuid,
        trailing_bytes: [u8; 4],
    },
    Workee {
        target_work_id: uuid::Uuid,
        trailing_bytes: [u8; 4],
    },
    Energy,
    StatusObserver,
    ItemStack,
    Switch {
        switch_state: u8,
        trailing_bytes: [u8; 4],
    },
    PlayerRecord,
    BaseCampPassiveEffect,
    PasswordLock {
        lock_state: u8,
        password: String,
        player_infos: Vec<PalPlayerLockInfo>,
        trailing_bytes: [u8; 4],
    },
    RequireElementalAction {
        unlock_item: String,
        trailing_bytes: [u8; 12],
    },
    Unknown {
        module_type: String,
        raw_bytes: Vec<u8>,
    },
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PalMapConcreteModelModule {
    pub module_type: String,
    pub data: PalMapConcreteModelModuleData,
    pub custom_version_data: Vec<u8>,
}

impl PalMapConcreteModelModule {
    pub(crate) fn read_with_module_type<R: Read + Seek, V: crate::VersionInfo>(
        reader: &mut Context<R, V>,
        module_type: &str,
        bytes: Vec<u8>,
        custom_version_data: Vec<u8>,
    ) -> TResult<Self> {
        if bytes.is_empty() {
            return Ok(PalMapConcreteModelModule {
                module_type: module_type.to_string(),
                data: match module_type {
                    "EPalMapObjectConcreteModelModuleType::Energy" => {
                        PalMapConcreteModelModuleData::Energy
                    }
                    "EPalMapObjectConcreteModelModuleType::StatusObserver" => {
                        PalMapConcreteModelModuleData::StatusObserver
                    }
                    "EPalMapObjectConcreteModelModuleType::ItemStack" => {
                        PalMapConcreteModelModuleData::ItemStack
                    }
                    "EPalMapObjectConcreteModelModuleType::PlayerRecord" => {
                        PalMapConcreteModelModuleData::PlayerRecord
                    }
                    "EPalMapObjectConcreteModelModuleType::BaseCampPassiveEffect" => {
                        PalMapConcreteModelModuleData::BaseCampPassiveEffect
                    }
                    _ => PalMapConcreteModelModuleData::Unknown {
                        module_type: module_type.to_string(),
                        raw_bytes: bytes,
                    },
                },
                custom_version_data,
            });
        }

        let data: PalMapConcreteModelModuleData = reader.with_stream(
            &mut std::io::Cursor::new(bytes),
            |byte_reader| -> TResult<PalMapConcreteModelModuleData> {
                match module_type {
                    "EPalMapObjectConcreteModelModuleType::ItemContainer" => {
                        let target_container_id = uuid::Uuid::read(byte_reader)?;
                        let slot_count = byte_reader.read_u32::<LE>()?;
                        let mut slot_attribute_indexes = Vec::with_capacity(slot_count as usize);
                        for _ in 0..slot_count {
                            slot_attribute_indexes.push(PalModuleSlotIndexes::read(byte_reader)?);
                        }
                        let all_slot_count = byte_reader.read_u32::<LE>()?;
                        let mut all_slot_attribute = Vec::with_capacity(all_slot_count as usize);
                        for _ in 0..all_slot_count {
                            all_slot_attribute.push(byte_reader.read_u8()?);
                        }
                        let drop_item_at_disposed = byte_reader.read_u32::<LE>()? > 0;
                        let usage_type = byte_reader.read_u8()?;
                        let mut trailing_bytes = [0u8; 4];
                        byte_reader.read_exact(&mut trailing_bytes)?;

                        Ok(PalMapConcreteModelModuleData::ItemContainer {
                            target_container_id,
                            slot_attribute_indexes,
                            all_slot_attribute,
                            drop_item_at_disposed,
                            usage_type,
                            trailing_bytes,
                        })
                    }
                    "EPalMapObjectConcreteModelModuleType::CharacterContainer" => {
                        let target_container_id = uuid::Uuid::read(byte_reader)?;
                        let mut trailing_bytes = [0u8; 4];
                        byte_reader.read_exact(&mut trailing_bytes)?;
                        Ok(PalMapConcreteModelModuleData::CharacterContainer {
                            target_container_id,
                            trailing_bytes,
                        })
                    }
                    "EPalMapObjectConcreteModelModuleType::Workee" => {
                        let target_work_id = uuid::Uuid::read(byte_reader)?;
                        let mut trailing_bytes = [0u8; 4];
                        byte_reader.read_exact(&mut trailing_bytes)?;
                        Ok(PalMapConcreteModelModuleData::Workee {
                            target_work_id,
                            trailing_bytes,
                        })
                    }
                    "EPalMapObjectConcreteModelModuleType::Switch" => {
                        let switch_state = byte_reader.read_u8()?;
                        let mut trailing_bytes = [0u8; 4];
                        byte_reader.read_exact(&mut trailing_bytes)?;
                        Ok(PalMapConcreteModelModuleData::Switch {
                            switch_state,
                            trailing_bytes,
                        })
                    }
                    "EPalMapObjectConcreteModelModuleType::PasswordLock" => {
                        let lock_state = byte_reader.read_u8()?;
                        let password = crate::read_string(byte_reader)?;
                        let player_count = byte_reader.read_u32::<LE>()?;
                        let mut player_infos = Vec::with_capacity(player_count as usize);
                        for _ in 0..player_count {
                            player_infos.push(PalPlayerLockInfo::read(byte_reader)?);
                        }
                        let mut trailing_bytes = [0u8; 4];
                        byte_reader.read_exact(&mut trailing_bytes)?;
                        Ok(PalMapConcreteModelModuleData::PasswordLock {
                            lock_state,
                            password,
                            player_infos,
                            trailing_bytes,
                        })
                    }
                    "EPalMapObjectConcreteModelModuleType::RequireElementalAction" => {
                        let unlock_item = crate::read_string(byte_reader)?;
                        let mut trailing_bytes = [0u8; 12];
                        byte_reader.read_exact(&mut trailing_bytes)?;
                        Ok(PalMapConcreteModelModuleData::RequireElementalAction {
                            unlock_item,
                            trailing_bytes,
                        })
                    }
                    "EPalMapObjectConcreteModelModuleType::Energy" => {
                        Ok(PalMapConcreteModelModuleData::Energy)
                    }
                    "EPalMapObjectConcreteModelModuleType::StatusObserver" => {
                        Ok(PalMapConcreteModelModuleData::StatusObserver)
                    }
                    "EPalMapObjectConcreteModelModuleType::ItemStack" => {
                        Ok(PalMapConcreteModelModuleData::ItemStack)
                    }
                    "EPalMapObjectConcreteModelModuleType::PlayerRecord" => {
                        Ok(PalMapConcreteModelModuleData::PlayerRecord)
                    }
                    "EPalMapObjectConcreteModelModuleType::BaseCampPassiveEffect" => {
                        Ok(PalMapConcreteModelModuleData::BaseCampPassiveEffect)
                    }
                    _ => {
                        let mut raw_bytes = Vec::new();
                        byte_reader.read_to_end(&mut raw_bytes)?;
                        Ok(PalMapConcreteModelModuleData::Unknown {
                            module_type: module_type.to_string(),
                            raw_bytes,
                        })
                    }
                }
            },
        )?;

        Ok(PalMapConcreteModelModule {
            module_type: module_type.to_string(),
            data,
            custom_version_data,
        })
    }
}

impl<R: Read + Seek, V: crate::VersionInfo> Readable<R, V> for PalMapConcreteModelModule {
    fn read(_reader: &mut Context<R, V>) -> TResult<Self> {
        Err(crate::Error::Other(
            "PalMapConcreteModelModule::read called without module_type context. Use read_with_module_type instead."
                .to_string(),
        ))
    }
}

impl<W: Write, V: crate::VersionInfo> Writable<W, V> for PalMapConcreteModelModule {
    fn write(&self, writer: &mut Context<W, V>) -> TResult<()> {
        match &self.data {
            PalMapConcreteModelModuleData::ItemContainer {
                target_container_id,
                slot_attribute_indexes,
                all_slot_attribute,
                drop_item_at_disposed,
                usage_type,
                trailing_bytes,
            } => {
                target_container_id.write(writer)?;
                writer.write_u32::<LE>(slot_attribute_indexes.len() as u32)?;
                for slot in slot_attribute_indexes {
                    slot.write(writer)?;
                }
                writer.write_u32::<LE>(all_slot_attribute.len() as u32)?;
                for attr in all_slot_attribute {
                    writer.write_u8(*attr)?;
                }
                writer.write_u32::<LE>(if *drop_item_at_disposed { 1 } else { 0 })?;
                writer.write_u8(*usage_type)?;
                writer.write_all(trailing_bytes)?;
            }
            PalMapConcreteModelModuleData::CharacterContainer {
                target_container_id,
                trailing_bytes,
            } => {
                target_container_id.write(writer)?;
                writer.write_all(trailing_bytes)?;
            }
            PalMapConcreteModelModuleData::Workee {
                target_work_id,
                trailing_bytes,
            } => {
                target_work_id.write(writer)?;
                writer.write_all(trailing_bytes)?;
            }
            PalMapConcreteModelModuleData::Switch {
                switch_state,
                trailing_bytes,
            } => {
                writer.write_u8(*switch_state)?;
                writer.write_all(trailing_bytes)?;
            }
            PalMapConcreteModelModuleData::PasswordLock {
                lock_state,
                password,
                player_infos,
                trailing_bytes,
            } => {
                writer.write_u8(*lock_state)?;
                crate::write_string(writer, password)?;
                writer.write_u32::<LE>(player_infos.len() as u32)?;
                for player_info in player_infos {
                    player_info.write(writer)?;
                }
                writer.write_all(trailing_bytes)?;
            }
            PalMapConcreteModelModuleData::RequireElementalAction {
                unlock_item,
                trailing_bytes,
            } => {
                crate::write_string(writer, unlock_item)?;
                writer.write_all(trailing_bytes)?;
            }
            PalMapConcreteModelModuleData::Energy
            | PalMapConcreteModelModuleData::StatusObserver
            | PalMapConcreteModelModuleData::ItemStack
            | PalMapConcreteModelModuleData::PlayerRecord
            | PalMapConcreteModelModuleData::BaseCampPassiveEffect => {}
            PalMapConcreteModelModuleData::Unknown { raw_bytes, .. } => {
                writer.write_all(raw_bytes)?;
            }
        }
        Ok(())
    }
}
