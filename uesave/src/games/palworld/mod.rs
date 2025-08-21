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

use map_object::parse_map_object_with_context;

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
    ByteArray, Context, Property, PropertyInner, PropertyKey, PropertyTagFull, Readable,
    StructType, StructValue, TResult, Types, ValueArray, ValueVec, VersionInfo, Writable,
};
use std::io::{Cursor, Read, Seek, Write};

pub fn palworld_types() -> Types {
    let mut types = Types::new();

    types.add(
        ".worldSaveData.CharacterContainerSaveData.Key".to_string(),
        StructType::Struct(None),
    );
    types.add(
        ".worldSaveData.CharacterSaveParameterMap.Key".to_string(),
        StructType::Struct(None),
    );
    types.add(
        ".worldSaveData.CharacterSaveParameterMap.Value".to_string(),
        StructType::Struct(None),
    );
    types.add(
        ".worldSaveData.FoliageGridSaveDataMap.Key".to_string(),
        StructType::Struct(None),
    );
    types.add(
        ".worldSaveData.FoliageGridSaveDataMap.Value".to_string(),
        StructType::Struct(None),
    );
    types.add(
        ".worldSaveData.FoliageGridSaveDataMap.ModelMap.Value".to_string(),
        StructType::Struct(None),
    );
    types.add(
        ".worldSaveData.FoliageGridSaveDataMap.ModelMap.InstanceDataMap.Key".to_string(),
        StructType::Struct(None),
    );
    types.add(
        ".worldSaveData.FoliageGridSaveDataMap.ModelMap.InstanceDataMap.Value".to_string(),
        StructType::Struct(None),
    );
    types.add(
        ".worldSaveData.ItemContainerSaveData.Key".to_string(),
        StructType::Struct(None),
    );
    types.add(
        ".worldSaveData.ItemContainerSaveData.Value".to_string(),
        StructType::Struct(None),
    );
    types.add(
        ".worldSaveData.MapObjectSaveData.ConcreteModel.ModuleMap.Value".to_string(),
        StructType::Struct(None),
    );
    types.add(
        ".worldSaveData.MapObjectSaveData.Model.EffectMap.Value".to_string(),
        StructType::Struct(None),
    );
    types.add(
        ".worldSaveData.MapObjectSpawnerInStageSaveData.Key".to_string(),
        StructType::Struct(None),
    );
    types.add(
        ".worldSaveData.MapObjectSpawnerInStageSaveData.Value".to_string(),
        StructType::Struct(None),
    );
    types.add(".worldSaveData.MapObjectSpawnerInStageSaveData.Value.SpawnerDataMapByLevelObjectInstanceId.Key".to_string(), StructType::Guid);
    types.add(".worldSaveData.MapObjectSpawnerInStageSaveData.Value.SpawnerDataMapByLevelObjectInstanceId.Value".to_string(), StructType::Struct(None));
    types.add(".worldSaveData.MapObjectSpawnerInStageSaveData.Value.SpawnerDataMapByLevelObjectInstanceId.Value.ItemMap.Value".to_string(), StructType::Struct(None));
    types.add(
        ".worldSaveData.WorkSaveData.WorkSaveData.WorkAssignMap.Value".to_string(),
        StructType::Struct(None),
    );
    types.add(
        ".worldSaveData.WorkSaveData.WorkAssignMap.Value".to_string(),
        StructType::Struct(None),
    );
    types.add(
        ".worldSaveData.BaseCampSaveData.Key".to_string(),
        StructType::Guid,
    );
    types.add(
        ".worldSaveData.BaseCampSaveData.Value".to_string(),
        StructType::Struct(None),
    );
    types.add(
        ".worldSaveData.BaseCampSaveData.ModuleMap.Value".to_string(),
        StructType::Struct(None),
    );
    types.add(
        ".worldSaveData.CharacterContainerSaveData.Value".to_string(),
        StructType::Struct(None),
    );
    types.add(
        ".worldSaveData.GroupSaveDataMap.Key".to_string(),
        StructType::Guid,
    );
    types.add(
        ".worldSaveData.GroupSaveDataMap.Value".to_string(),
        StructType::Struct(None),
    );
    types.add(
        ".worldSaveData.EnemyCampSaveData.EnemyCampStatusMap.Value".to_string(),
        StructType::Struct(None),
    );
    types.add(".worldSaveData.EnemyCampSaveData.EnemyCampStatusMap.Value.TreasureBoxInfoMapBySpawnerName.Value".to_string(), StructType::Struct(None));
    types.add(".worldSaveData.DungeonSaveData.DungeonSaveData.MapObjectSaveData.MapObjectSaveData.Model.EffectMap.Value".to_string(), StructType::Struct(None));
    types.add(".worldSaveData.DungeonSaveData.DungeonSaveData.MapObjectSaveData.MapObjectSaveData.ConcreteModel.ModuleMap.Value".to_string(), StructType::Struct(None));
    types.add(
        ".worldSaveData.DungeonSaveData.DungeonSaveData.RewardSaveDataMap.Key".to_string(),
        StructType::Guid,
    );
    types.add(
        ".worldSaveData.DungeonSaveData.DungeonSaveData.RewardSaveDataMap.Value".to_string(),
        StructType::Struct(None),
    );
    types.add(
        ".worldSaveData.InvaderSaveData.Key".to_string(),
        StructType::Guid,
    );
    types.add(
        ".worldSaveData.InvaderSaveData.Value".to_string(),
        StructType::Struct(None),
    );
    types.add(
        ".worldSaveData.OilrigSaveData.OilrigMap.Value".to_string(),
        StructType::Struct(None),
    );
    types.add(
        ".worldSaveData.SupplySaveData.SupplyInfos.Key".to_string(),
        StructType::Guid,
    );
    types.add(
        ".worldSaveData.SupplySaveData.SupplyInfos.Value".to_string(),
        StructType::Struct(None),
    );
    types.add(
        ".worldSaveData.GuildExtraSaveDataMap.Key".to_string(),
        StructType::Guid,
    );
    types.add(
        ".worldSaveData.GuildExtraSaveDataMap.Value".to_string(),
        StructType::Struct(None),
    );
    types.add(
        ".SaveData.Local_MaxFriendshipPalIds.Key".to_string(),
        StructType::Guid,
    );
    types.add(
        ".SaveData.Local_MaxFriendshipPalIds.Value".to_string(),
        StructType::Struct(None),
    );
    types.add(
        ".worldSaveData.MapObjectSpawnerInStageSaveData.SpawnerDataMapByLevelObjectInstanceId.Key"
            .to_string(),
        StructType::Guid,
    );
    types.add(".worldSaveData.MapObjectSpawnerInStageSaveData.SpawnerDataMapByLevelObjectInstanceId.Value".to_string(), StructType::Struct(None));
    types.add(".worldSaveData.MapObjectSpawnerInStageSaveData.SpawnerDataMapByLevelObjectInstanceId.ItemMap.Value".to_string(), StructType::Struct(None));
    types.add(
        ".worldSaveData.DungeonSaveData.RewardSaveDataMap.Key".to_string(),
        StructType::Guid,
    );
    types.add(
        ".worldSaveData.DungeonSaveData.RewardSaveDataMap.Value".to_string(),
        StructType::Struct(None),
    );
    types.add(
        ".worldSaveData.InLockerCharacterInstanceIDArray".to_string(),
        StructType::Struct(None),
    );
    types.add(
        ".worldSaveData.EnemyCampSaveData.EnemyCampStatusMap.TreasureBoxInfoMapBySpawnerName.Value"
            .to_string(),
        StructType::Struct(None),
    );
    types.add(
        ".worldSaveData.GroupSaveDataMap.RawData".to_string(),
        StructType::PalGroupData,
    );
    types.add(
        ".worldSaveData.CharacterSaveParameterMap.RawData".to_string(),
        StructType::PalCharacterData,
    );
    types.add(
        ".worldSaveData.ItemContainerSaveData.RawData".to_string(),
        StructType::PalItemContainer,
    );
    types.add(
        ".worldSaveData.ItemContainerSaveData.Slots.RawData".to_string(),
        StructType::PalItemContainerSlots,
    );
    types.add(
        ".worldSaveData.CharacterContainerSaveData.Slots.RawData".to_string(),
        StructType::PalCharacterContainer,
    );
    types.add(
        ".worldSaveData.DynamicItemSaveData.RawData".to_string(),
        StructType::PalDynamicItem,
    );
    types.add(
        ".worldSaveData.FoliageGridSaveDataMap.ModelMap.RawData".to_string(),
        StructType::Struct(None),
    );
    types.add(
        ".worldSaveData.FoliageGridSaveDataMap.ModelMap.InstanceDataMap.RawData".to_string(),
        StructType::Struct(None),
    );
    types.add(
        ".worldSaveData.BaseCampSaveData.RawData".to_string(),
        StructType::PalBaseCamp,
    );
    types.add(
        ".worldSaveData.BaseCampSaveData.WorkerDirector.RawData".to_string(),
        StructType::Struct(None),
    );
    types.add(
        ".worldSaveData.BaseCampSaveData.WorkCollection.RawData".to_string(),
        StructType::Struct(None),
    );
    types.add(
        ".worldSaveData.WorkSaveData".to_string(),
        StructType::PalWork,
    );
    types.add(
        ".worldSaveData.GuildExtraSaveDataMap.GuildItemStorage.RawData".to_string(),
        StructType::PalGuildItemStorage,
    );
    types.add(
        ".worldSaveData.GuildExtraSaveDataMap.Lab.RawData".to_string(),
        StructType::PalGuildLab,
    );

    types
}

pub(crate) fn process_property_for_read<R: Read + Seek, V: VersionInfo>(
    reader: &mut Context<R, V>,
    _tag: &PropertyTagFull,
    path: &str,
    inner: PropertyInner,
) -> TResult<PropertyInner> {
    if path == ".worldSaveData.MapObjectSaveData" {
        if let PropertyInner::Array(ValueArray::Struct {
            mut value,
            type_,
            struct_type,
            id,
        }) = inner
        {
            for (i, struct_value) in value.iter_mut().enumerate() {
                if let StructValue::Struct(ref mut properties) = struct_value {
                    parse_map_object_with_context(reader, properties)?;
                } else {
                    return Err(crate::Error::Other(format!(
                        "Expected struct value for MapObjectSaveData element {}, got: {:?}",
                        i + 1,
                        struct_value
                    )));
                }
            }

            return Ok(PropertyInner::Array(ValueArray::Struct {
                type_,
                struct_type,
                id,
                value,
            }));
        }
    }

    match path {
        ".worldSaveData.CharacterSaveParameterMap.RawData" => {
            if let PropertyInner::Array(ValueArray::Base(ValueVec::Byte(ByteArray::Byte(
                ref bytes,
            )))) = inner
            {
                let data =
                    reader.with_stream(&mut Cursor::new(bytes.clone()), PalCharacterData::read)?;
                return Ok(PropertyInner::Struct(StructValue::PalCharacterData(data)));
            }
        }
        ".worldSaveData.ItemContainerSaveData.RawData" => {
            if let PropertyInner::Array(ValueArray::Base(ValueVec::Byte(ByteArray::Byte(
                ref bytes,
            )))) = inner
            {
                let data =
                    reader.with_stream(&mut Cursor::new(bytes.clone()), PalItemContainer::read)?;
                return Ok(PropertyInner::Struct(StructValue::PalItemContainer(data)));
            }
        }
        ".worldSaveData.GroupSaveDataMap.RawData" => {
            if let PropertyInner::Array(ValueArray::Base(ValueVec::Byte(ByteArray::Byte(
                ref bytes,
            )))) = inner
            {
                let data =
                    reader.with_stream(&mut Cursor::new(bytes.clone()), PalGroupData::read)?;
                return Ok(PropertyInner::Struct(StructValue::PalGroupData(data)));
            }
        }
        ".worldSaveData.CharacterContainerSaveData.Slots.RawData" => {
            if let PropertyInner::Array(ValueArray::Base(ValueVec::Byte(ByteArray::Byte(
                ref bytes,
            )))) = inner
            {
                let data = reader
                    .with_stream(&mut Cursor::new(bytes.clone()), PalCharacterContainer::read)?;
                return Ok(PropertyInner::Struct(StructValue::PalCharacterContainer(
                    data,
                )));
            }
        }
        ".worldSaveData.DynamicItemSaveData.RawData" => {
            if let PropertyInner::Array(ValueArray::Base(ValueVec::Byte(ByteArray::Byte(
                ref bytes,
            )))) = inner
            {
                let data =
                    reader.with_stream(&mut Cursor::new(bytes.clone()), PalDynamicItem::read)?;
                return Ok(PropertyInner::Struct(StructValue::PalDynamicItem(data)));
            }
        }
        ".worldSaveData.BaseCampSaveData.RawData" => {
            if let PropertyInner::Array(ValueArray::Base(ValueVec::Byte(ByteArray::Byte(
                ref bytes,
            )))) = inner
            {
                let data =
                    reader.with_stream(&mut Cursor::new(bytes.clone()), PalBaseCamp::read)?;
                return Ok(PropertyInner::Struct(StructValue::PalBaseCamp(data)));
            }
        }
        ".worldSaveData.GuildExtraSaveDataMap.Lab.RawData" => {
            if let PropertyInner::Array(ValueArray::Base(ValueVec::Byte(ByteArray::Byte(
                ref bytes,
            )))) = inner
            {
                let data =
                    reader.with_stream(&mut Cursor::new(bytes.clone()), PalGuildLab::read)?;
                return Ok(PropertyInner::Struct(StructValue::PalGuildLab(data)));
            }
        }
        ".worldSaveData.GuildExtraSaveDataMap.GuildItemStorage.RawData" => {
            if let PropertyInner::Array(ValueArray::Base(ValueVec::Byte(ByteArray::Byte(
                ref bytes,
            )))) = inner
            {
                let data = reader
                    .with_stream(&mut Cursor::new(bytes.clone()), PalGuildItemStorage::read)?;
                return Ok(PropertyInner::Struct(StructValue::PalGuildItemStorage(
                    data,
                )));
            }
        }
        ".worldSaveData.ItemContainerSaveData.Slots.RawData" => {
            if let PropertyInner::Array(ValueArray::Base(ValueVec::Byte(ByteArray::Byte(
                ref bytes,
            )))) = inner
            {
                let data = reader
                    .with_stream(&mut Cursor::new(bytes.clone()), PalItemContainerSlot::read)?;
                return Ok(PropertyInner::Struct(StructValue::PalItemContainerSlots(
                    data,
                )));
            }
        }
        _ => {}
    }

    Ok(inner)
}

pub(crate) fn process_property_for_write<W: Write, V: VersionInfo>(
    writer: &mut Context<W, V>,
    key: &PropertyKey,
    prop: &Property,
) -> TResult<Option<Property>> {
    if key.1 != "RawData" {
        return Ok(None);
    }

    let bytes = match &prop.inner {
        PropertyInner::Struct(StructValue::PalMapModel(model)) => {
            let mut buf = Vec::new();
            writer.with_stream(&mut Cursor::new(&mut buf), |w| model.write(w))?;
            buf
        }
        PropertyInner::Struct(StructValue::PalMapConcreteModel(model)) => {
            let mut buf = Vec::new();
            writer.with_stream(&mut Cursor::new(&mut buf), |w| model.write(w))?;
            buf
        }
        PropertyInner::Struct(StructValue::PalMapConcreteModelModule(module)) => {
            let mut buf = Vec::new();
            writer.with_stream(&mut Cursor::new(&mut buf), |w| module.write(w))?;
            buf
        }
        PropertyInner::Struct(StructValue::PalConnector(connector)) => {
            let mut buf = Vec::new();
            writer.with_stream(&mut Cursor::new(&mut buf), |w| connector.write(w))?;
            buf
        }
        PropertyInner::Struct(StructValue::PalBuildProcess(build_process)) => {
            let mut buf = Vec::new();
            writer.with_stream(&mut Cursor::new(&mut buf), |w| build_process.write(w))?;
            buf
        }
        PropertyInner::Struct(StructValue::PalCharacterData(data)) => {
            let mut buf = Vec::new();
            writer.with_stream(&mut Cursor::new(&mut buf), |w| data.write(w))?;
            buf
        }
        PropertyInner::Struct(StructValue::PalItemContainer(data)) => {
            let mut buf = Vec::new();
            writer.with_stream(&mut Cursor::new(&mut buf), |w| data.write(w))?;
            buf
        }
        PropertyInner::Struct(StructValue::PalGroupData(data)) => {
            let mut buf = Vec::new();
            writer.with_stream(&mut Cursor::new(&mut buf), |w| data.write(w))?;
            buf
        }
        PropertyInner::Struct(StructValue::PalCharacterContainer(data)) => {
            let mut buf = Vec::new();
            writer.with_stream(&mut Cursor::new(&mut buf), |w| data.write(w))?;
            buf
        }
        PropertyInner::Struct(StructValue::PalDynamicItem(data)) => {
            let mut buf = Vec::new();
            writer.with_stream(&mut Cursor::new(&mut buf), |w| data.write(w))?;
            buf
        }
        PropertyInner::Struct(StructValue::PalBaseCamp(data)) => {
            let mut buf = Vec::new();
            writer.with_stream(&mut Cursor::new(&mut buf), |w| data.write(w))?;
            buf
        }
        PropertyInner::Struct(StructValue::PalGuildLab(data)) => {
            let mut buf = Vec::new();
            writer.with_stream(&mut Cursor::new(&mut buf), |w| data.write(w))?;
            buf
        }
        PropertyInner::Struct(StructValue::PalGuildItemStorage(data)) => {
            let mut buf = Vec::new();
            writer.with_stream(&mut Cursor::new(&mut buf), |w| data.write(w))?;
            buf
        }
        PropertyInner::Struct(StructValue::PalItemContainerSlots(data)) => {
            let mut buf = Vec::new();
            writer.with_stream(&mut Cursor::new(&mut buf), |w| data.write(w))?;
            buf
        }
        _ => return Ok(None),
    };

    Ok(Some(Property {
        tag: prop.tag.clone(),
        inner: PropertyInner::Array(ValueArray::Base(ValueVec::Byte(ByteArray::Byte(bytes)))),
    }))
}

pub(crate) fn post_process_property<R: Read + Seek, V: VersionInfo>(
    reader: &mut Context<R, V>,
    tag: &PropertyTagFull,
    path: &str,
    inner: PropertyInner,
) -> TResult<PropertyInner> {
    process_property_for_read(reader, tag, path, inner)
}
