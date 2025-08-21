use super::{
    PalBuildProcess, PalConnector, PalMapConcreteModel, PalMapConcreteModelModule, PalMapModel,
};
use crate::{
    ByteArray, Context, Properties, PropertyInner, PropertyKey, Readable, StructValue, TResult,
    ValueArray, ValueVec,
};
use std::io::{Cursor, Read, Seek};

fn determine_object_id(properties: &Properties) -> TResult<String> {
    match &properties["MapObjectId"].inner {
        PropertyInner::Str(object_id) => Ok(object_id.clone()),
        PropertyInner::Name(object_id) => Ok(object_id.clone()),
        other => Err(crate::Error::Other(format!(
            "MapObjectId expected as string or name, but found: {:?}",
            other
        ))),
    }
}

pub(crate) fn parse_map_object_with_context<R: Read + Seek, V: crate::VersionInfo>(
    reader: &mut Context<R, V>,
    properties: &mut Properties,
) -> TResult<()> {
    let map_object_id_value = determine_object_id(properties)?;

    if let Some(model_prop) = properties.0.get_mut(&PropertyKey::from("Model")) {
        if let PropertyInner::Struct(StructValue::Struct(ref mut model_properties)) =
            &mut model_prop.inner
        {
            if let Some(raw_data_prop) = model_properties.0.get_mut(&PropertyKey::from("RawData")) {
                if let PropertyInner::Array(ValueArray::Base(ValueVec::Byte(ByteArray::Byte(
                    ref bytes,
                )))) = &raw_data_prop.inner
                {
                    let model =
                        reader.with_stream(&mut Cursor::new(bytes.clone()), PalMapModel::read)?;

                    raw_data_prop.inner = PropertyInner::Struct(StructValue::PalMapModel(model));
                }
            }

            if let Some(connector_prop) =
                model_properties.0.get_mut(&PropertyKey::from("Connector"))
            {
                if let PropertyInner::Struct(StructValue::Struct(ref mut connector_properties)) =
                    &mut connector_prop.inner
                {
                    if let Some(raw_data_prop) = connector_properties
                        .0
                        .get_mut(&PropertyKey::from("RawData"))
                    {
                        if let PropertyInner::Array(ValueArray::Base(ValueVec::Byte(
                            ByteArray::Byte(ref bytes),
                        ))) = &raw_data_prop.inner
                        {
                            if !bytes.is_empty() {
                                let connector = reader.with_stream(
                                    &mut Cursor::new(bytes.clone()),
                                    PalConnector::read,
                                )?;

                                raw_data_prop.inner =
                                    PropertyInner::Struct(StructValue::PalConnector(connector));
                            }
                        }
                    }
                }
            }

            if let Some(build_process_prop) = model_properties
                .0
                .get_mut(&PropertyKey::from("BuildProcess"))
            {
                if let PropertyInner::Struct(StructValue::Struct(
                    ref mut build_process_properties,
                )) = &mut build_process_prop.inner
                {
                    if let Some(raw_data_prop) = build_process_properties
                        .0
                        .get_mut(&PropertyKey::from("RawData"))
                    {
                        if let PropertyInner::Array(ValueArray::Base(ValueVec::Byte(
                            ByteArray::Byte(ref bytes),
                        ))) = &raw_data_prop.inner
                        {
                            if !bytes.is_empty() {
                                let build_process = reader.with_stream(
                                    &mut Cursor::new(bytes.clone()),
                                    PalBuildProcess::read,
                                )?;

                                raw_data_prop.inner = PropertyInner::Struct(
                                    StructValue::PalBuildProcess(build_process),
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    if let Some(concrete_model_prop) = properties.0.get_mut(&PropertyKey::from("ConcreteModel")) {
        if let PropertyInner::Struct(StructValue::Struct(ref mut concrete_model_properties)) =
            &mut concrete_model_prop.inner
        {
            if let Some(raw_data_prop) = concrete_model_properties
                .0
                .get_mut(&PropertyKey::from("RawData"))
            {
                if let PropertyInner::Array(ValueArray::Base(ValueVec::Byte(ByteArray::Byte(
                    ref bytes,
                )))) = &raw_data_prop.inner
                {
                    if !bytes.is_empty() {
                        let concrete_model =
                            reader.with_stream(&mut Cursor::new(bytes.clone()), |byte_reader| {
                                PalMapConcreteModel::read_with_object_id(
                                    byte_reader,
                                    &map_object_id_value,
                                )
                            })?;
                        raw_data_prop.inner =
                            PropertyInner::Struct(StructValue::PalMapConcreteModel(concrete_model));
                    }
                }
            }

            if let Some(module_map_prop) = concrete_model_properties
                .0
                .get_mut(&PropertyKey::from("ModuleMap"))
            {
                if let PropertyInner::Map(ref mut module_entries) = &mut module_map_prop.inner {
                    for entry in module_entries.iter_mut() {
                        let module_type = match &entry.key {
                            crate::PropertyValue::Enum(t) => t.clone(),
                            crate::PropertyValue::Str(t) => t.clone(),
                            crate::PropertyValue::Name(t) => t.clone(),
                            _ => continue,
                        };

                        if let crate::PropertyValue::Struct(ref mut value_struct) = &mut entry.value
                        {
                            if let StructValue::Struct(ref mut value_props) = value_struct {
                                let custom_version_data = if let Some(custom_version_prop) =
                                    value_props.0.get(&PropertyKey::from("CustomVersionData"))
                                {
                                    if let PropertyInner::Array(ValueArray::Base(ValueVec::Byte(
                                        ByteArray::Byte(ref custom_bytes),
                                    ))) = &custom_version_prop.inner
                                    {
                                        custom_bytes.clone()
                                    } else {
                                        Vec::new()
                                    }
                                } else {
                                    Vec::new()
                                };

                                if let Some(raw_data_prop) =
                                    value_props.0.get_mut(&PropertyKey::from("RawData"))
                                {
                                    if let PropertyInner::Array(ValueArray::Base(ValueVec::Byte(
                                        ByteArray::Byte(ref bytes),
                                    ))) = &raw_data_prop.inner
                                    {
                                        if !bytes.is_empty() {
                                            let module_bytes_clone = bytes.clone();
                                            let module = reader.with_stream(&mut Cursor::new(&bytes), |byte_reader| {
                                                PalMapConcreteModelModule::read_with_module_type(
                                                    byte_reader,
                                                    &module_type,
                                                    module_bytes_clone,
                                                    custom_version_data,
                                                )
                                            })?;
                                            raw_data_prop.inner = PropertyInner::Struct(
                                                StructValue::PalMapConcreteModelModule(module),
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
