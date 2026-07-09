use super::{
    PalBuildProcess, PalConnector, PalMapConcreteModel, PalMapConcreteModelModule, PalMapModel,
};
use crate::{
    ByteArray, Properties, Property, PropertyKey, PropertyTagDataPartial, PropertyTagPartial,
    Result, SaveGameArchive, StructType, StructValue, ValueVec,
};
use std::io::{Cursor, Read, Seek};

fn determine_object_id(properties: &Properties) -> Result<String> {
    let map_object_id = properties
        .0
        .get(&PropertyKey::from("MapObjectId"))
        .ok_or_else(|| crate::Error::Other("MapObjectSaveData element is missing MapObjectId".to_string()))?;
    match map_object_id {
        Property::Str(object_id) => Ok(object_id.clone()),
        Property::Name(object_id) => Ok(object_id.clone()),
        other => Err(crate::Error::Other(format!(
            "MapObjectId expected as string or name, but found: {:?}",
            other
        ))),
    }
}

/// If `prop` holds a non-empty byte array, parse it as an embedded Palworld
/// struct via `parse` and replace it with the parsed value. The scope is
/// extended by `scope_segments` for the duration of the parse so that nested
/// schemas are recorded at the right paths, and the schema of the property
/// itself is updated to the given `struct_type`.
fn convert_embedded<R: Read + Seek>(
    ar: &mut SaveGameArchive<R>,
    prop: &mut Property,
    scope_segments: &[&str],
    struct_type: StructType,
    parse: impl FnOnce(&mut SaveGameArchive<Cursor<Vec<u8>>>) -> Result<StructValue>,
) -> Result<()> {
    let Property::Array(ValueVec::Byte(ByteArray::Byte(bytes))) = &*prop else {
        return Ok(());
    };
    // Empty payloads stay as byte arrays and round-trip unchanged
    if bytes.is_empty() {
        return Ok(());
    }
    let bytes = bytes.clone();

    for segment in scope_segments {
        ar.scope.push(segment);
    }
    let result = (|ar: &mut SaveGameArchive<R>| -> Result<StructValue> {
        let parsed = ar.with_nested(Cursor::new(bytes), parse)?;
        ar.schemas.borrow_mut().record(
            ar.scope.path(),
            PropertyTagPartial {
                id: None,
                data: PropertyTagDataPartial::Struct {
                    struct_type,
                    id: Default::default(),
                },
            },
        );
        Ok(parsed)
    })(ar);
    for _ in scope_segments {
        ar.scope.pop();
    }

    *prop = Property::Struct(result?);
    Ok(())
}

pub(crate) fn parse_map_object_with_context<R: Read + Seek>(
    ar: &mut SaveGameArchive<R>,
    properties: &mut Properties,
) -> Result<()> {
    let map_object_id_value = determine_object_id(properties)?;

    if let Some(Property::Struct(StructValue::Struct(model_properties))) =
        properties.0.get_mut(&PropertyKey::from("Model"))
    {
        if let Some(raw_data_prop) = model_properties.0.get_mut(&PropertyKey::from("RawData")) {
            convert_embedded(
                ar,
                raw_data_prop,
                &["Model", "RawData"],
                StructType::PalMapModel,
                |nested| Ok(StructValue::PalMapModel(PalMapModel::read(nested)?)),
            )?;
        }

        if let Some(Property::Struct(StructValue::Struct(connector_properties))) =
            model_properties.0.get_mut(&PropertyKey::from("Connector"))
        {
            if let Some(raw_data_prop) =
                connector_properties.0.get_mut(&PropertyKey::from("RawData"))
            {
                convert_embedded(
                    ar,
                    raw_data_prop,
                    &["Model", "Connector", "RawData"],
                    StructType::PalConnector,
                    |nested| Ok(StructValue::PalConnector(PalConnector::read(nested)?)),
                )?;
            }
        }

        if let Some(Property::Struct(StructValue::Struct(build_process_properties))) =
            model_properties.0.get_mut(&PropertyKey::from("BuildProcess"))
        {
            if let Some(raw_data_prop) = build_process_properties
                .0
                .get_mut(&PropertyKey::from("RawData"))
            {
                convert_embedded(
                    ar,
                    raw_data_prop,
                    &["Model", "BuildProcess", "RawData"],
                    StructType::PalBuildProcess,
                    |nested| Ok(StructValue::PalBuildProcess(PalBuildProcess::read(nested)?)),
                )?;
            }
        }
    }

    if let Some(Property::Struct(StructValue::Struct(concrete_model_properties))) =
        properties.0.get_mut(&PropertyKey::from("ConcreteModel"))
    {
        if let Some(raw_data_prop) = concrete_model_properties
            .0
            .get_mut(&PropertyKey::from("RawData"))
        {
            let map_object_id = map_object_id_value.clone();
            convert_embedded(
                ar,
                raw_data_prop,
                &["ConcreteModel", "RawData"],
                StructType::PalMapConcreteModel,
                |nested| {
                    Ok(StructValue::PalMapConcreteModel(
                        PalMapConcreteModel::read_with_object_id(nested, &map_object_id)?,
                    ))
                },
            )?;
        }

        if let Some(Property::Map(module_entries)) = concrete_model_properties
            .0
            .get_mut(&PropertyKey::from("ModuleMap"))
        {
            for entry in module_entries.iter_mut() {
                let module_type = match &entry.key {
                    Property::Enum(t) => t.clone(),
                    Property::Str(t) => t.clone(),
                    Property::Name(t) => t.clone(),
                    _ => continue,
                };

                if let Property::Struct(StructValue::Struct(value_props)) = &mut entry.value {
                    let custom_version_data = match value_props
                        .0
                        .get(&PropertyKey::from("CustomVersionData"))
                    {
                        Some(Property::Array(ValueVec::Byte(ByteArray::Byte(custom_bytes)))) => {
                            custom_bytes.clone()
                        }
                        _ => Vec::new(),
                    };

                    if let Some(raw_data_prop) =
                        value_props.0.get_mut(&PropertyKey::from("RawData"))
                    {
                        let module_bytes = match &*raw_data_prop {
                            Property::Array(ValueVec::Byte(ByteArray::Byte(bytes))) => {
                                bytes.clone()
                            }
                            _ => continue,
                        };
                        convert_embedded(
                            ar,
                            raw_data_prop,
                            &["ConcreteModel", "ModuleMap", "Value", "RawData"],
                            StructType::PalMapConcreteModelModule,
                            move |nested| {
                                Ok(StructValue::PalMapConcreteModelModule(
                                    PalMapConcreteModelModule::read_with_module_type(
                                        nested,
                                        &module_type,
                                        module_bytes,
                                        custom_version_data,
                                    )?,
                                ))
                            },
                        )?;
                    }
                }
            }
        }
    }

    Ok(())
}
