//! Support for Palworld save files.
//!
//! Palworld embeds much of its data as opaque byte arrays (typically properties
//! named `RawData`) inside regular GVAS property trees. When the paths of these
//! properties are registered in [`Types`] (see [`palworld_types`]), the byte
//! arrays are transparently parsed into typed [`StructValue`]s on read and
//! serialized back to byte arrays on write.

pub mod base_camp;
pub mod build_process;
pub mod character;
pub mod connector;
pub mod groups;
pub mod guild;
pub mod items;
pub mod map_concrete_model;
pub mod map_concrete_model_module;
pub mod map_model;
pub mod map_object;
pub mod types;
pub mod work;

pub use base_camp::*;
pub use build_process::*;
pub use character::*;
pub use connector::*;
pub use groups::*;
pub use guild::*;
pub use items::*;
pub use map_concrete_model::*;
pub use map_concrete_model_module::*;
pub use map_model::*;
pub use types::*;
pub use work::*;

use crate::{
    ByteArray, Property, PropertyKey, PropertyTagDataPartial, PropertyTagPartial, Result,
    SaveGameArchive, StructType, StructValue, Types, ValueVec,
};
use std::io::{Cursor, Read, Seek, Write};

/// Path of the map object save data which needs context-dependent parsing of
/// the embedded data of its elements.
const MAP_OBJECT_SAVE_DATA_PATH: &str = "worldSaveData.MapObjectSaveData";

pub(crate) fn is_pal_struct_type(t: &StructType) -> bool {
    matches!(
        t,
        StructType::PalCharacterData
            | StructType::PalItemContainer
            | StructType::PalGroupData
            | StructType::PalDynamicItem
            | StructType::PalBuildProcess
            | StructType::PalGuildItemStorage
            | StructType::PalGuildLab
            | StructType::PalItemContainerSlots
            | StructType::PalCharacterContainer
            | StructType::PalConnector
            | StructType::PalBaseCamp
            | StructType::PalWork
            | StructType::PalMapModel
            | StructType::PalMapConcreteModel
            | StructType::PalMapConcreteModelModule
    )
}

/// Build a [`Types`] specification for parsing Palworld save files (Level.sav,
/// LevelMeta.sav, Players/\*.sav, ...).
pub fn palworld_types() -> Types {
    let mut types = Types::new();

    let struct_hints = [
        "worldSaveData.CharacterContainerSaveData.Key",
        "worldSaveData.CharacterSaveParameterMap.Key",
        "worldSaveData.CharacterSaveParameterMap.Value",
        "worldSaveData.FoliageGridSaveDataMap.Key",
        "worldSaveData.FoliageGridSaveDataMap.Value",
        "worldSaveData.FoliageGridSaveDataMap.ModelMap.Value",
        "worldSaveData.FoliageGridSaveDataMap.ModelMap.InstanceDataMap.Key",
        "worldSaveData.FoliageGridSaveDataMap.ModelMap.InstanceDataMap.Value",
        "worldSaveData.ItemContainerSaveData.Key",
        "worldSaveData.ItemContainerSaveData.Value",
        "worldSaveData.MapObjectSaveData.ConcreteModel.ModuleMap.Value",
        "worldSaveData.MapObjectSaveData.Model.EffectMap.Value",
        "worldSaveData.MapObjectSpawnerInStageSaveData.Key",
        "worldSaveData.MapObjectSpawnerInStageSaveData.Value",
        "worldSaveData.MapObjectSpawnerInStageSaveData.Value.SpawnerDataMapByLevelObjectInstanceId.Value",
        "worldSaveData.MapObjectSpawnerInStageSaveData.Value.SpawnerDataMapByLevelObjectInstanceId.Value.ItemMap.Value",
        "worldSaveData.WorkSaveData.WorkAssignMap.Value",
        "worldSaveData.BaseCampSaveData.Value",
        "worldSaveData.BaseCampSaveData.ModuleMap.Value",
        "worldSaveData.CharacterContainerSaveData.Value",
        "worldSaveData.GroupSaveDataMap.Value",
        "worldSaveData.EnemyCampSaveData.EnemyCampStatusMap.Value",
        "worldSaveData.EnemyCampSaveData.EnemyCampStatusMap.Value.TreasureBoxInfoMapBySpawnerName.Value",
        "worldSaveData.DungeonSaveData.MapObjectSaveData.Model.EffectMap.Value",
        "worldSaveData.DungeonSaveData.MapObjectSaveData.ConcreteModel.ModuleMap.Value",
        "worldSaveData.InvaderSaveData.Value",
        "worldSaveData.OilrigSaveData.OilrigMap.Value",
        "worldSaveData.SupplySaveData.SupplyInfos.Value",
        "worldSaveData.GuildExtraSaveDataMap.Value",
        "SaveData.Local_MaxFriendshipPalIds.Value",
        "worldSaveData.MapObjectSpawnerInStageSaveData.SpawnerDataMapByLevelObjectInstanceId.Value",
        "worldSaveData.MapObjectSpawnerInStageSaveData.SpawnerDataMapByLevelObjectInstanceId.ItemMap.Value",
        "worldSaveData.DungeonSaveData.RewardSaveDataMap.Value",
        "worldSaveData.InLockerCharacterInstanceIDArray",
        "worldSaveData.EnemyCampSaveData.EnemyCampStatusMap.TreasureBoxInfoMapBySpawnerName.Value",
        "worldSaveData.FoliageGridSaveDataMap.ModelMap.RawData",
        "worldSaveData.FoliageGridSaveDataMap.ModelMap.InstanceDataMap.RawData",
        "worldSaveData.BaseCampSaveData.WorkerDirector.RawData",
        "worldSaveData.BaseCampSaveData.WorkCollection.RawData",
        // Marker enabling the context-dependent parsing of MapObjectSaveData elements
        MAP_OBJECT_SAVE_DATA_PATH,
    ];
    for path in struct_hints {
        types.add(path.to_string(), StructType::Struct(None));
    }

    let guid_hints = [
        "worldSaveData.MapObjectSpawnerInStageSaveData.Value.SpawnerDataMapByLevelObjectInstanceId.Key",
        "worldSaveData.BaseCampSaveData.Key",
        "worldSaveData.GroupSaveDataMap.Key",
        "worldSaveData.InvaderSaveData.Key",
        "worldSaveData.SupplySaveData.SupplyInfos.Key",
        "worldSaveData.GuildExtraSaveDataMap.Key",
        "SaveData.Local_MaxFriendshipPalIds.Key",
        "worldSaveData.MapObjectSpawnerInStageSaveData.SpawnerDataMapByLevelObjectInstanceId.Key",
        "worldSaveData.DungeonSaveData.RewardSaveDataMap.Key",
    ];
    for path in guid_hints {
        types.add(path.to_string(), StructType::Guid);
    }

    // Embedded (RawData) properties parsed into typed Palworld structs
    let pal_hints = [
        (
            "worldSaveData.GroupSaveDataMap.RawData",
            StructType::PalGroupData,
        ),
        (
            "worldSaveData.CharacterSaveParameterMap.RawData",
            StructType::PalCharacterData,
        ),
        (
            "worldSaveData.ItemContainerSaveData.RawData",
            StructType::PalItemContainer,
        ),
        (
            "worldSaveData.ItemContainerSaveData.Slots.RawData",
            StructType::PalItemContainerSlots,
        ),
        (
            "worldSaveData.CharacterContainerSaveData.Slots.RawData",
            StructType::PalCharacterContainer,
        ),
        (
            "worldSaveData.DynamicItemSaveData.RawData",
            StructType::PalDynamicItem,
        ),
        (
            "worldSaveData.BaseCampSaveData.RawData",
            StructType::PalBaseCamp,
        ),
        ("worldSaveData.WorkSaveData", StructType::PalWork),
        (
            "worldSaveData.GuildExtraSaveDataMap.GuildItemStorage.RawData",
            StructType::PalGuildItemStorage,
        ),
        (
            "worldSaveData.GuildExtraSaveDataMap.Lab.RawData",
            StructType::PalGuildLab,
        ),
    ];
    for (path, t) in pal_hints {
        types.add(path.to_string(), t);
    }

    types
}

/// Called for every property after it has been read (with the property name on
/// the scope). Converts Palworld data embedded as byte arrays into typed struct
/// values when the current path is registered with a Pal struct type in the
/// [`Types`] specification, updating `tag` (and thereby the recorded schema) to
/// match.
pub(crate) fn process_property_for_read<R: Read + Seek>(
    ar: &mut SaveGameArchive<R>,
    tag: &mut PropertyTagPartial,
    value: Property,
) -> Result<Property> {
    let Some(hint) = ar.get_type().cloned() else {
        return Ok(value);
    };

    // MapObjectSaveData elements need context-dependent parsing (the embedded
    // data formats depend on sibling properties such as MapObjectId)
    if ar.scope.path() == MAP_OBJECT_SAVE_DATA_PATH {
        if let Property::Array(ValueVec::Struct(mut values)) = value {
            for (i, struct_value) in values.iter_mut().enumerate() {
                match struct_value {
                    StructValue::Struct(properties) => {
                        map_object::parse_map_object_with_context(ar, properties)?;
                    }
                    other => {
                        return Err(crate::Error::Other(format!(
                            "Expected struct value for MapObjectSaveData element {}, got: {:?}",
                            i + 1,
                            other
                        )))
                    }
                }
            }
            return Ok(Property::Array(ValueVec::Struct(values)));
        }
        return Ok(value);
    }

    if !is_pal_struct_type(&hint) {
        return Ok(value);
    }
    let Property::Array(ValueVec::Byte(ByteArray::Byte(bytes))) = &value else {
        return Ok(value);
    };
    // Empty payloads stay as byte arrays and round-trip unchanged
    if bytes.is_empty() {
        return Ok(value);
    }

    let len = bytes.len() as u64;
    let parsed = ar.with_nested(Cursor::new(bytes.clone()), |nested| {
        let struct_value = StructValue::read(nested, &hint)?;
        // Refuse partial parses: unconsumed bytes would be lost on rewrite
        let consumed = nested.stream_position()?;
        if consumed != len {
            return Err(crate::Error::Other(format!(
                "Palworld struct {hint:?} consumed only {consumed} of {len} bytes"
            )));
        }
        Ok(struct_value)
    });

    match parsed {
        Ok(struct_value) => {
            tag.data = PropertyTagDataPartial::Struct {
                struct_type: hint,
                id: Default::default(),
            };
            Ok(Property::Struct(struct_value))
        }
        Err(e) if ar.error_to_raw() => {
            if ar.log() {
                eprintln!(
                    "Warning: Failed to parse Palworld data at '{}', leaving as raw bytes: {}",
                    ar.scope.path(),
                    e
                );
            }
            Ok(value)
        }
        Err(e) => Err(e),
    }
}

/// Called for every property before it is written (with the property name on
/// the scope). Serializes typed Palworld struct values back into the byte
/// arrays they are embedded as, based on the schema recorded when reading.
pub(crate) fn process_property_for_write<W: Write + Seek>(
    ar: &mut SaveGameArchive<W>,
    _key: &PropertyKey,
    tag: &PropertyTagPartial,
    prop: &Property,
) -> Result<Option<(PropertyTagPartial, Property)>> {
    let PropertyTagDataPartial::Struct { struct_type, .. } = &tag.data else {
        return Ok(None);
    };
    if !is_pal_struct_type(struct_type) {
        return Ok(None);
    }

    let byte_array_tag = PropertyTagPartial {
        id: tag.id,
        data: PropertyTagDataPartial::Array(Box::new(PropertyTagDataPartial::Byte(None))),
    };

    match prop {
        // Empty/unparsed payloads were left as byte arrays on read
        Property::Array(ValueVec::Byte(ByteArray::Byte(_))) => {
            Ok(Some((byte_array_tag, prop.clone())))
        }
        Property::Struct(struct_value) => {
            let mut buf = Vec::new();
            ar.with_nested(Cursor::new(&mut buf), |nested| struct_value.write(nested))?;
            Ok(Some((
                byte_array_tag,
                Property::Array(ValueVec::Byte(ByteArray::Byte(buf))),
            )))
        }
        _ => Ok(None),
    }
}
