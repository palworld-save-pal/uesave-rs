use crate::games::palworld::{
    self, PalCharacterData, PalDynamicItem, PalDynamicItemType, PalMapConcreteModel,
    PalMapConcreteModelVariant, PalMapObjectHatchingEggModel,
};
use crate::{
    FClothLODDataCommon, FGuid, FMeshToMeshVertData, FNiagaraVariable, FNiagaraVariableBase,
    FNiagaraVariableWithOffset, Properties, Property, PropertyKey, PropertySchemas,
    PropertyTagDataPartial, Root, Save, SoftObjectPath, StructType, StructValue, ValueVec,
};
use serde::{
    de::{DeserializeSeed, MapAccess, SeqAccess, Visitor},
    Deserialize, Deserializer,
};
use std::fmt;

/// Generates one or more `DeserializeSeed`s that thread `PropertySchemas` + a
/// path prefix through to any field marked `= "segment"`, which is seeded as a
/// nested `Properties` at `{parent_path}.{segment}`. Unmarked fields use plain
/// `Deserialize`.
macro_rules! properties_seeds {
    (
        $(
            $value:ident => $seed:ident {
                $( $field:ident : $ty:ty $( = $segment:literal )? ),+ $(,)?
            }
        )+
    ) => {
        $(
            struct $seed<'a> {
                path: &'a str,
                schemas: &'a PropertySchemas,
            }

            impl<'de, 'a> DeserializeSeed<'de> for $seed<'a> {
                type Value = $value;

                fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                where
                    D: Deserializer<'de>,
                {
                    deserializer.deserialize_map(self)
                }
            }

            impl<'de, 'a> Visitor<'de> for $seed<'a> {
                type Value = $value;

                fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    f.write_str(concat!(stringify!($value), " map"))
                }

                fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
                where
                    A: MapAccess<'de>,
                {
                    let __path = self.path;
                    let __schemas = self.schemas;
                    $( let mut $field: Option<$ty> = None; )+

                    while let Some(__key) = map.next_key::<String>()? {
                        match __key.as_str() {
                            $(
                                stringify!($field) => {
                                    $field = Some(properties_seeds!(
                                        @read __path, __schemas, map $(, $segment)?
                                    ));
                                }
                            )+
                            other => {
                                return Err(serde::de::Error::unknown_field(
                                    other,
                                    &[$( stringify!($field) ),+],
                                ))
                            }
                        }
                    }

                    Ok($value {
                        $(
                            $field: $field.ok_or_else(|| {
                                serde::de::Error::missing_field(stringify!($field))
                            })?,
                        )+
                    })
                }
            }
        )+
    };

    (@read $path:ident, $schemas:ident, $m:ident, $segment:literal) => {{
        let __sub_path = if $path.is_empty() {
            String::from($segment)
        } else if $segment.is_empty() {
            $path.to_string()
        } else {
            format!("{}.{}", $path, $segment)
        };
        $m.next_value_seed(PropertiesSeed {
            path: &__sub_path,
            schemas: $schemas,
        })?
    }};

    (@read $path:ident, $schemas:ident, $m:ident) => {{
        $m.next_value()?
    }};
}

properties_seeds! {
    Root => RootSeed {
        save_game_type: String,
        properties: Properties = "",
    }

    FClothLODDataCommon => ClothLODDataCommonSeed {
        properties: Properties = "properties",
        transition_up_skin_data: Vec<FMeshToMeshVertData>,
        transition_down_skin_data: Vec<FMeshToMeshVertData>,
    }

    FNiagaraVariableBase => NiagaraVariableBaseSeed {
        name: String,
        type_def: Properties = "type_def",
    }

    FNiagaraVariable => NiagaraVariableSeed {
        name: String,
        type_def: Properties = "type_def",
        var_data: Vec<u8>,
    }

    FNiagaraVariableWithOffset => NiagaraVariableWithOffsetSeed {
        name: String,
        type_def: Properties = "type_def",
        offset: i32,
    }

    PalCharacterData => PalCharacterDataSeed {
        object: Properties = "",
        unknown_bytes: [u8; 4],
        group_id: FGuid,
        trailing_bytes: [u8; 4],
    }

    PalMapObjectHatchingEggModel => PalMapObjectHatchingEggModelSeed {
        leading_bytes: [u8; 4],
        hatched_character_save_parameter: Properties = "",
        current_pal_egg_temp_diff: i32,
        hatched_character_guid: FGuid,
        trailing_bytes: [u8; 4],
    }
}

struct PropertySeed<'a> {
    tag: &'a PropertyTagDataPartial,
    path: &'a str,
    schemas: &'a PropertySchemas,
}

impl<'de, 'a> DeserializeSeed<'de> for PropertySeed<'a> {
    type Value = Property;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        use crate::PropertyType;
        if self.tag.has_raw_struct() {
            return Ok(Property::Raw(Vec::<u8>::deserialize(deserializer)?));
        }
        match &self.tag {
            PropertyTagDataPartial::Other(pt) => match pt {
                PropertyType::BoolProperty => Ok(Property::Bool(bool::deserialize(deserializer)?)),
                PropertyType::Int8Property => Ok(Property::Int8(i8::deserialize(deserializer)?)),
                PropertyType::Int16Property => Ok(Property::Int16(i16::deserialize(deserializer)?)),
                PropertyType::IntProperty => Ok(Property::Int(i32::deserialize(deserializer)?)),
                PropertyType::Int64Property => Ok(Property::Int64(i64::deserialize(deserializer)?)),
                PropertyType::UInt8Property => Ok(Property::UInt8(u8::deserialize(deserializer)?)),
                PropertyType::UInt16Property => {
                    Ok(Property::UInt16(u16::deserialize(deserializer)?))
                }
                PropertyType::UInt32Property => {
                    Ok(Property::UInt32(u32::deserialize(deserializer)?))
                }
                PropertyType::UInt64Property => {
                    Ok(Property::UInt64(u64::deserialize(deserializer)?))
                }
                PropertyType::FloatProperty => {
                    Ok(Property::Float(f32::deserialize(deserializer)?.into()))
                }
                PropertyType::DoubleProperty => {
                    Ok(Property::Double(f64::deserialize(deserializer)?.into()))
                }
                PropertyType::NameProperty => {
                    Ok(Property::Name(String::deserialize(deserializer)?))
                }
                PropertyType::StrProperty => Ok(Property::Str(String::deserialize(deserializer)?)),
                PropertyType::ObjectProperty | PropertyType::InterfaceProperty => {
                    Ok(Property::Object(String::deserialize(deserializer)?))
                }
                PropertyType::TextProperty => {
                    Ok(Property::Text(crate::Text::deserialize(deserializer)?))
                }
                PropertyType::FieldPathProperty => Ok(Property::FieldPath(
                    crate::FieldPath::deserialize(deserializer)?,
                )),
                PropertyType::SoftObjectProperty => Ok(Property::SoftObject(
                    crate::SoftObjectPath::deserialize(deserializer)?,
                )),
                PropertyType::DelegateProperty => Ok(Property::Delegate(
                    crate::Delegate::deserialize(deserializer)?,
                )),
                PropertyType::MulticastDelegateProperty => Ok(Property::MulticastDelegate(
                    crate::MulticastDelegate::deserialize(deserializer)?,
                )),
                PropertyType::MulticastInlineDelegateProperty => {
                    Ok(Property::MulticastInlineDelegate(
                        crate::MulticastInlineDelegate::deserialize(deserializer)?,
                    ))
                }
                PropertyType::MulticastSparseDelegateProperty => {
                    Ok(Property::MulticastSparseDelegate(
                        crate::MulticastSparseDelegate::deserialize(deserializer)?,
                    ))
                }
                // These should never appear in Other - they have dedicated variants
                PropertyType::ByteProperty
                | PropertyType::EnumProperty
                | PropertyType::ArrayProperty
                | PropertyType::SetProperty
                | PropertyType::MapProperty
                | PropertyType::StructProperty => Err(serde::de::Error::custom(format!(
                    "Property type {pt:?} should not appear in Other variant"
                ))),
            },
            PropertyTagDataPartial::Byte(_enum_type) => {
                Ok(Property::Byte(crate::Byte::deserialize(deserializer)?))
            }
            PropertyTagDataPartial::Enum(_, _) => {
                Ok(Property::Enum(String::deserialize(deserializer)?))
            }
            PropertyTagDataPartial::Struct { struct_type, .. } => {
                // Palworld structs embedded as byte arrays: empty/unparsed
                // payloads stay byte arrays in JSON, so accept both shapes
                if palworld::is_pal_struct_type(struct_type) {
                    return PalPropertySeed {
                        struct_type,
                        path: self.path,
                        schemas: self.schemas,
                    }
                    .deserialize(deserializer);
                }
                let sv = StructValueSeed {
                    struct_type,
                    path: self.path,
                    schemas: self.schemas,
                }
                .deserialize(deserializer)?;
                Ok(Property::Struct(sv))
            }
            PropertyTagDataPartial::Array(inner_tag) => {
                let va = ValueVecSeed {
                    tag: inner_tag,
                    path: self.path,
                    schemas: self.schemas,
                }
                .deserialize(deserializer)?;
                Ok(Property::Array(va))
            }
            PropertyTagDataPartial::Set { key_type } => {
                let vs = ValueVecSeed {
                    tag: key_type,
                    path: self.path,
                    schemas: self.schemas,
                }
                .deserialize(deserializer)?;
                Ok(Property::Set(vs))
            }
            PropertyTagDataPartial::Map {
                key_type,
                value_type,
            } => {
                let entries = MapEntriesSeed {
                    key_type,
                    value_type,
                    path: self.path,
                    schemas: self.schemas,
                }
                .deserialize(deserializer)?;
                Ok(Property::Map(entries))
            }
        }
    }
}

struct StructValueSeed<'a> {
    struct_type: &'a StructType,
    path: &'a str,
    schemas: &'a PropertySchemas,
}

impl<'de, 'a> DeserializeSeed<'de> for StructValueSeed<'a> {
    type Value = StructValue;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        match self.struct_type {
            StructType::Guid => Ok(StructValue::Guid(crate::FGuid::deserialize(deserializer)?)),
            StructType::DateTime => Ok(StructValue::DateTime(u64::deserialize(deserializer)?)),
            StructType::Timespan => Ok(StructValue::Timespan(i64::deserialize(deserializer)?)),
            StructType::Vector2D => Ok(StructValue::Vector2D(crate::Vector2D::deserialize(
                deserializer,
            )?)),
            StructType::Vector => Ok(StructValue::Vector(crate::Vector::deserialize(
                deserializer,
            )?)),
            StructType::Vector4 => Ok(StructValue::Vector4(crate::Vector4::deserialize(
                deserializer,
            )?)),
            StructType::IntVector => Ok(StructValue::IntVector(crate::IntVector::deserialize(
                deserializer,
            )?)),
            StructType::Box => Ok(StructValue::Box(crate::Box::deserialize(deserializer)?)),
            StructType::Box2D => Ok(StructValue::Box2D(crate::Box2D::deserialize(deserializer)?)),
            StructType::IntPoint => Ok(StructValue::IntPoint(crate::IntPoint::deserialize(
                deserializer,
            )?)),
            StructType::Quat => Ok(StructValue::Quat(crate::Quat::deserialize(deserializer)?)),
            StructType::LinearColor => Ok(StructValue::LinearColor(
                crate::LinearColor::deserialize(deserializer)?,
            )),
            StructType::Color => Ok(StructValue::Color(crate::Color::deserialize(deserializer)?)),
            StructType::Rotator => Ok(StructValue::Rotator(crate::Rotator::deserialize(
                deserializer,
            )?)),
            StructType::SoftObjectPath => Ok(StructValue::SoftObjectPath(
                crate::SoftObjectPath::deserialize(deserializer)?,
            )),
            StructType::SoftClassPath => Ok(StructValue::SoftClassPath(
                crate::SoftObjectPath::deserialize(deserializer)?,
            )),
            StructType::GameplayTagContainer => Ok(StructValue::GameplayTagContainer(
                crate::GameplayTagContainer::deserialize(deserializer)?,
            )),
            StructType::UniqueNetIdRepl => Ok(StructValue::UniqueNetIdRepl(
                crate::UniqueNetIdRepl::deserialize(deserializer)?,
            )),
            StructType::KeyHandleMap => Ok(StructValue::KeyHandleMap(
                crate::FKeyHandleMap::deserialize(deserializer)?,
            )),
            StructType::RichCurveKey => Ok(StructValue::RichCurveKey(
                crate::FRichCurveKey::deserialize(deserializer)?,
            )),
            StructType::SkeletalMeshSamplingLODBuiltData => {
                Ok(StructValue::SkeletalMeshSamplingLODBuiltData(
                    crate::FSkeletalMeshSamplingLODBuiltData::deserialize(deserializer)?,
                ))
            }
            StructType::PerPlatformFloat => Ok(StructValue::PerPlatformFloat(
                crate::FPerPlatformFloat::deserialize(deserializer)?,
            )),
            StructType::MovieSceneFrameRange => Ok(StructValue::MovieSceneFrameRange(
                crate::FMovieSceneFrameRange::deserialize(deserializer)?,
            )),
            StructType::MovieSceneFloatChannel => Ok(StructValue::MovieSceneFloatChannel(
                crate::FMovieSceneFloatChannel::deserialize(deserializer)?,
            )),
            StructType::FrameNumber => Ok(StructValue::FrameNumber(
                crate::FFrameNumber::deserialize(deserializer)?,
            )),
            StructType::ExpressionInput => Ok(StructValue::ExpressionInput(
                crate::FExpressionInput::deserialize(deserializer)?,
            )),
            StructType::MaterialAttributesInput => Ok(StructValue::MaterialAttributesInput(
                crate::FExpressionInput::deserialize(deserializer)?,
            )),
            StructType::ColorMaterialInput => Ok(StructValue::ColorMaterialInput(
                crate::FColorMaterialInput::deserialize(deserializer)?,
            )),
            StructType::ScalarMaterialInput => Ok(StructValue::ScalarMaterialInput(
                crate::FScalarMaterialInput::deserialize(deserializer)?,
            )),
            StructType::ShadingModelMaterialInput => Ok(StructValue::ShadingModelMaterialInput(
                crate::FShadingModelMaterialInput::deserialize(deserializer)?,
            )),
            StructType::VectorMaterialInput => Ok(StructValue::VectorMaterialInput(
                crate::FVectorMaterialInput::deserialize(deserializer)?,
            )),
            StructType::Vector2MaterialInput => Ok(StructValue::Vector2MaterialInput(
                crate::FVector2MaterialInput::deserialize(deserializer)?,
            )),
            StructType::MovieSceneSequenceID => Ok(StructValue::MovieSceneSequenceID(
                crate::FMovieSceneSequenceID::deserialize(deserializer)?,
            )),
            StructType::MovieSceneTrackIdentifier => Ok(StructValue::MovieSceneTrackIdentifier(
                crate::FMovieSceneTrackIdentifier::deserialize(deserializer)?,
            )),
            StructType::MovieSceneEvaluationKey => Ok(StructValue::MovieSceneEvaluationKey(
                crate::FMovieSceneEvaluationKey::deserialize(deserializer)?,
            )),
            StructType::MovieSceneEvaluationFieldEntityTree => {
                Ok(StructValue::MovieSceneEvaluationFieldEntityTree(
                    crate::FMovieSceneEvaluationFieldEntityTree::deserialize(deserializer)?,
                ))
            }
            StructType::NiagaraDataInterfaceGeneratedFunction => {
                Ok(StructValue::NiagaraDataInterfaceGeneratedFunction(
                    crate::FNiagaraDataInterfaceGeneratedFunction::deserialize(deserializer)?,
                ))
            }
            StructType::NiagaraDataInterfaceGPUParamInfo => {
                Ok(StructValue::NiagaraDataInterfaceGPUParamInfo(
                    crate::FNiagaraDataInterfaceGPUParamInfo::deserialize(deserializer)?,
                ))
            }
            StructType::FontData => Ok(StructValue::FontData(crate::FFontData::deserialize(
                deserializer,
            )?)),
            StructType::ClothLODDataCommon => Ok(StructValue::ClothLODDataCommon(
                ClothLODDataCommonSeed {
                    path: self.path,
                    schemas: self.schemas,
                }
                .deserialize(deserializer)?,
            )),
            StructType::NiagaraVariable => Ok(StructValue::NiagaraVariable(
                NiagaraVariableSeed {
                    path: self.path,
                    schemas: self.schemas,
                }
                .deserialize(deserializer)?,
            )),
            StructType::NiagaraVariableBase => Ok(StructValue::NiagaraVariableBase(
                NiagaraVariableBaseSeed {
                    path: self.path,
                    schemas: self.schemas,
                }
                .deserialize(deserializer)?,
            )),
            StructType::NiagaraVariableWithOffset => Ok(StructValue::NiagaraVariableWithOffset(
                NiagaraVariableWithOffsetSeed {
                    path: self.path,
                    schemas: self.schemas,
                }
                .deserialize(deserializer)?,
            )),
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
            | StructType::PalWorkAssign
            | StructType::PalMapModel
            | StructType::PalMapConcreteModel
            | StructType::PalMapConcreteModelModule => PalStructValueSeed {
                struct_type: self.struct_type,
                path: self.path,
                schemas: self.schemas,
            }
            .deserialize(deserializer),
            StructType::Struct(_) => {
                let props = PropertiesSeed {
                    path: self.path,
                    schemas: self.schemas,
                }
                .deserialize(deserializer)?;
                Ok(StructValue::Struct(props))
            }
            StructType::Raw(_) => Ok(StructValue::Raw(Vec::<u8>::deserialize(deserializer)?)),
        }
    }
}

struct StructVecSeed<'a> {
    struct_type: &'a StructType,
    path: &'a str,
    schemas: &'a PropertySchemas,
}

impl<'de, 'a> DeserializeSeed<'de> for StructVecSeed<'a> {
    type Value = Vec<StructValue>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(self)
    }
}

impl<'de, 'a> Visitor<'de> for StructVecSeed<'a> {
    type Value = Vec<StructValue>;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("array or set of structs")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut vec = Vec::new();
        while let Some(elem) = seq.next_element_seed(StructValueSeed {
            struct_type: self.struct_type,
            path: self.path,
            schemas: self.schemas,
        })? {
            vec.push(elem);
        }
        Ok(vec)
    }
}

struct ValueVecSeed<'a> {
    tag: &'a PropertyTagDataPartial,
    path: &'a str,
    schemas: &'a PropertySchemas,
}

impl<'de, 'a> DeserializeSeed<'de> for ValueVecSeed<'a> {
    type Value = ValueVec;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        match self.tag {
            PropertyTagDataPartial::Struct { struct_type, .. } => {
                let structs = StructVecSeed {
                    struct_type,
                    path: self.path,
                    schemas: self.schemas,
                }
                .deserialize(deserializer)?;
                Ok(ValueVec::Struct(structs))
            }
            PropertyTagDataPartial::Other(pt) => {
                use crate::PropertyType;
                match pt {
                    PropertyType::Int8Property => {
                        Ok(ValueVec::Int8(Vec::<i8>::deserialize(deserializer)?))
                    }
                    PropertyType::Int16Property => {
                        Ok(ValueVec::Int16(Vec::<i16>::deserialize(deserializer)?))
                    }
                    PropertyType::IntProperty => {
                        Ok(ValueVec::Int(Vec::<i32>::deserialize(deserializer)?))
                    }
                    PropertyType::Int64Property => {
                        Ok(ValueVec::Int64(Vec::<i64>::deserialize(deserializer)?))
                    }
                    PropertyType::UInt8Property => {
                        Ok(ValueVec::UInt8(Vec::<u8>::deserialize(deserializer)?))
                    }
                    PropertyType::UInt16Property => {
                        Ok(ValueVec::UInt16(Vec::<u16>::deserialize(deserializer)?))
                    }
                    PropertyType::UInt32Property => {
                        Ok(ValueVec::UInt32(Vec::<u32>::deserialize(deserializer)?))
                    }
                    PropertyType::UInt64Property => {
                        Ok(ValueVec::UInt64(Vec::<u64>::deserialize(deserializer)?))
                    }
                    PropertyType::FloatProperty => Ok(ValueVec::Float(
                        Vec::<crate::Float>::deserialize(deserializer)?,
                    )),
                    PropertyType::DoubleProperty => Ok(ValueVec::Double(
                        Vec::<crate::Double>::deserialize(deserializer)?,
                    )),
                    PropertyType::BoolProperty => {
                        Ok(ValueVec::Bool(Vec::<bool>::deserialize(deserializer)?))
                    }
                    PropertyType::StrProperty => {
                        Ok(ValueVec::Str(Vec::<String>::deserialize(deserializer)?))
                    }
                    PropertyType::NameProperty => {
                        Ok(ValueVec::Name(Vec::<String>::deserialize(deserializer)?))
                    }
                    PropertyType::ObjectProperty | PropertyType::InterfaceProperty => {
                        Ok(ValueVec::Object(Vec::<String>::deserialize(deserializer)?))
                    }
                    PropertyType::SoftObjectProperty => {
                        Ok(ValueVec::SoftObject(Vec::<SoftObjectPath>::deserialize(
                            deserializer,
                        )?))
                    }
                    PropertyType::TextProperty => Ok(ValueVec::Text(
                        Vec::<crate::Text>::deserialize(deserializer)?,
                    )),
                    PropertyType::ByteProperty
                    | PropertyType::EnumProperty
                    | PropertyType::ArrayProperty
                    | PropertyType::SetProperty
                    | PropertyType::MapProperty
                    | PropertyType::StructProperty
                    | PropertyType::FieldPathProperty
                    | PropertyType::DelegateProperty
                    | PropertyType::MulticastDelegateProperty
                    | PropertyType::MulticastInlineDelegateProperty
                    | PropertyType::MulticastSparseDelegateProperty => {
                        Err(serde::de::Error::custom(format!(
                            "Unexpected property type {pt:?} in array"
                        )))
                    }
                }
            }
            PropertyTagDataPartial::Byte(_) => {
                Ok(ValueVec::Byte(crate::ByteArray::deserialize(deserializer)?))
            }
            PropertyTagDataPartial::Enum(_, _) => {
                Ok(ValueVec::Enum(Vec::<String>::deserialize(deserializer)?))
            }
            PropertyTagDataPartial::Array(_)
            | PropertyTagDataPartial::Set { .. }
            | PropertyTagDataPartial::Map { .. } => Err(serde::de::Error::custom(
                "Nested array/set/map not supported",
            )),
        }
    }
}

struct MapEntriesSeed<'a> {
    key_type: &'a PropertyTagDataPartial,
    value_type: &'a PropertyTagDataPartial,
    path: &'a str,
    schemas: &'a PropertySchemas,
}

impl<'de, 'a> DeserializeSeed<'de> for MapEntriesSeed<'a> {
    type Value = Vec<crate::MapEntry>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(self)
    }
}

impl<'de, 'a> Visitor<'de> for MapEntriesSeed<'a> {
    type Value = Vec<crate::MapEntry>;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("array of map entries")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut vec = Vec::new();
        while let Some(elem) = seq.next_element_seed(MapEntrySeed {
            key_type: self.key_type,
            value_type: self.value_type,
            path: self.path,
            schemas: self.schemas,
        })? {
            vec.push(elem);
        }
        Ok(vec)
    }
}

struct MapEntrySeed<'a> {
    key_type: &'a PropertyTagDataPartial,
    value_type: &'a PropertyTagDataPartial,
    path: &'a str,
    schemas: &'a PropertySchemas,
}

impl<'de, 'a> DeserializeSeed<'de> for MapEntrySeed<'a> {
    type Value = crate::MapEntry;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_struct("MapEntry", &["key", "value"], self)
    }
}

impl<'de, 'a> Visitor<'de> for MapEntrySeed<'a> {
    type Value = crate::MapEntry;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("map entry with key and value fields")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Key,
            Value,
        }

        let mut key = None;
        let mut value = None;

        while let Some(field) = map.next_key()? {
            match field {
                Field::Key => {
                    key = Some(map.next_value_seed(PropertySeed {
                        tag: self.key_type,
                        path: self.path,
                        schemas: self.schemas,
                    })?);
                }
                Field::Value => {
                    value = Some(map.next_value_seed(PropertySeed {
                        tag: self.value_type,
                        path: self.path,
                        schemas: self.schemas,
                    })?);
                }
            }
        }

        let key = key.ok_or_else(|| serde::de::Error::missing_field("key"))?;
        let value = value.ok_or_else(|| serde::de::Error::missing_field("value"))?;

        Ok(crate::MapEntry { key, value })
    }
}

struct PropertiesSeed<'a> {
    path: &'a str,
    schemas: &'a PropertySchemas,
}

impl<'de, 'a> DeserializeSeed<'de> for PropertiesSeed<'a> {
    type Value = Properties;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(self)
    }
}

impl<'de, 'a> Visitor<'de> for PropertiesSeed<'a> {
    type Value = Properties;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("properties map")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut properties = indexmap::IndexMap::new();

        while let Some(key) = map.next_key::<PropertyKey>()? {
            let prop_path = if self.path.is_empty() {
                key.1.to_string()
            } else {
                format!("{}.{}", self.path, key.1)
            };

            let tag = self.schemas.schemas().get(&prop_path).ok_or_else(|| {
                serde::de::Error::custom(format!("No schema for property: {prop_path}"))
            })?;

            let prop = map.next_value_seed(PropertySeed {
                tag: &tag.data,
                path: &prop_path,
                schemas: self.schemas,
            })?;

            properties.insert(key, prop);
        }

        Ok(Properties(properties))
    }
}

// Deserialize implementation for Save
impl<'de> Deserialize<'de> for Save {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            Header,
            Schemas,
            Root,
            Extra,
        }

        struct SaveVisitor;

        impl<'de> Visitor<'de> for SaveVisitor {
            type Value = Save;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("Save struct")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut header = None;
                let mut schemas = None;
                let mut root = None;
                let mut extra = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Header => {
                            header = Some(map.next_value()?);
                        }
                        Field::Schemas => {
                            schemas = Some(map.next_value()?);
                        }
                        Field::Root => {
                            // Schemas must have been parsed before root
                            let schemas_ref = schemas.as_ref().ok_or_else(|| {
                                serde::de::Error::custom("schemas must appear before root in JSON")
                            })?;

                            root = Some(map.next_value_seed(RootSeed {
                                path: "",
                                schemas: schemas_ref,
                            })?);
                        }
                        Field::Extra => {
                            extra = Some(map.next_value()?);
                        }
                    }
                }

                let header = header.ok_or_else(|| serde::de::Error::missing_field("header"))?;
                let schemas = schemas.ok_or_else(|| serde::de::Error::missing_field("schemas"))?;
                let root = root.ok_or_else(|| serde::de::Error::missing_field("root"))?;
                let extra = extra.ok_or_else(|| serde::de::Error::missing_field("extra"))?;

                Ok(Save {
                    header,
                    schemas,
                    root,
                    extra,
                })
            }
        }

        deserializer.deserialize_struct(
            "Save",
            &["header", "schemas", "root", "extra"],
            SaveVisitor,
        )
    }
}

/// Deserializes a property whose schema names a Palworld struct type. The JSON
/// value is either the typed struct (a map) or a plain byte array (empty or
/// unparsed payloads, which are kept in their embedded form).
struct PalPropertySeed<'a> {
    struct_type: &'a StructType,
    path: &'a str,
    schemas: &'a PropertySchemas,
}

impl<'de> DeserializeSeed<'de> for PalPropertySeed<'_> {
    type Value = Property;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}

impl<'de> Visitor<'de> for PalPropertySeed<'_> {
    type Value = Property;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a {:?} struct or a byte array", self.struct_type)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut bytes = Vec::new();
        while let Some(byte) = seq.next_element::<u8>()? {
            bytes.push(byte);
        }
        Ok(Property::Array(ValueVec::Byte(crate::ByteArray::Byte(
            bytes,
        ))))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        // Byte arrays serialize externally tagged ({"Byte": [...]}), so peek
        // at the first key to tell them apart from Pal struct maps
        let Some(first_key) = map.next_key::<String>()? else {
            return Err(serde::de::Error::custom(format!(
                "empty map for {:?} property",
                self.struct_type
            )));
        };
        match first_key.as_str() {
            "Byte" => Ok(Property::Array(ValueVec::Byte(crate::ByteArray::Byte(
                map.next_value()?,
            )))),
            "Label" => Ok(Property::Array(ValueVec::Byte(crate::ByteArray::Label(
                map.next_value()?,
            )))),
            _ => {
                let sv = PalStructValueSeed {
                    struct_type: self.struct_type,
                    path: self.path,
                    schemas: self.schemas,
                }
                .deserialize(serde::de::value::MapAccessDeserializer::new(
                    PrependedMapAccess {
                        first_key: Some(first_key),
                        inner: map,
                    },
                ))?;
                Ok(Property::Struct(sv))
            }
        }
    }
}

/// A [`MapAccess`] adapter that re-injects an already-consumed first key.
struct PrependedMapAccess<M> {
    first_key: Option<String>,
    inner: M,
}

impl<'de, M: MapAccess<'de>> MapAccess<'de> for PrependedMapAccess<M> {
    type Error = M::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        use serde::de::IntoDeserializer;
        if let Some(key) = self.first_key.take() {
            return seed.deserialize(key.into_deserializer()).map(Some);
        }
        self.inner.next_key_seed(seed)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        self.inner.next_value_seed(seed)
    }
}

struct PalStructValueSeed<'a> {
    struct_type: &'a StructType,
    path: &'a str,
    schemas: &'a PropertySchemas,
}

impl<'de> DeserializeSeed<'de> for PalStructValueSeed<'_> {
    type Value = StructValue;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        match self.struct_type {
            StructType::PalCharacterData => Ok(StructValue::PalCharacterData(
                PalCharacterDataSeed {
                    path: self.path,
                    schemas: self.schemas,
                }
                .deserialize(deserializer)?,
            )),
            StructType::PalItemContainer => Ok(StructValue::PalItemContainer(
                palworld::PalItemContainer::deserialize(deserializer)?,
            )),
            StructType::PalGroupData => Ok(StructValue::PalGroupData(
                palworld::PalGroupData::deserialize(deserializer)?,
            )),
            StructType::PalDynamicItem => Ok(StructValue::PalDynamicItem(
                PalDynamicItemSeed {
                    path: self.path,
                    schemas: self.schemas,
                }
                .deserialize(deserializer)?
                .into(),
            )),
            StructType::PalBuildProcess => Ok(StructValue::PalBuildProcess(
                palworld::PalBuildProcess::deserialize(deserializer)?,
            )),
            StructType::PalGuildItemStorage => Ok(StructValue::PalGuildItemStorage(
                palworld::PalGuildItemStorage::deserialize(deserializer)?,
            )),
            StructType::PalGuildLab => Ok(StructValue::PalGuildLab(
                palworld::PalGuildLab::deserialize(deserializer)?,
            )),
            StructType::PalItemContainerSlots => Ok(StructValue::PalItemContainerSlots(
                palworld::PalItemContainerSlot::deserialize(deserializer)?,
            )),
            StructType::PalCharacterContainer => Ok(StructValue::PalCharacterContainer(
                palworld::PalCharacterContainer::deserialize(deserializer)?,
            )),
            StructType::PalConnector => Ok(StructValue::PalConnector(
                palworld::PalConnector::deserialize(deserializer)?,
            )),
            StructType::PalBaseCamp => Ok(StructValue::PalBaseCamp(
                palworld::PalBaseCamp::deserialize(deserializer)?.into(),
            )),
            StructType::PalWork => Ok(StructValue::PalWork(
                palworld::PalWork::deserialize(deserializer)?.into(),
            )),
            StructType::PalWorkAssign => Ok(StructValue::PalWorkAssign(
                palworld::PalWorkAssign::deserialize(deserializer)?,
            )),
            StructType::PalMapModel => Ok(StructValue::PalMapModel(
                palworld::PalMapModel::deserialize(deserializer)?.into(),
            )),
            StructType::PalMapConcreteModel => Ok(StructValue::PalMapConcreteModel(
                PalMapConcreteModelSeed {
                    path: self.path,
                    schemas: self.schemas,
                }
                .deserialize(deserializer)?
                .into(),
            )),
            StructType::PalMapConcreteModelModule => Ok(StructValue::PalMapConcreteModelModule(
                palworld::PalMapConcreteModelModule::deserialize(deserializer)?,
            )),
            other => Err(serde::de::Error::custom(format!(
                "not a Palworld struct type: {other:?}"
            ))),
        }
    }
}

struct PalDynamicItemSeed<'a> {
    path: &'a str,
    schemas: &'a PropertySchemas,
}

impl<'de> DeserializeSeed<'de> for PalDynamicItemSeed<'_> {
    type Value = PalDynamicItem;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(self)
    }
}

impl<'de> Visitor<'de> for PalDynamicItemSeed<'_> {
    type Value = PalDynamicItem;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("PalDynamicItem map")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut id = None;
        let mut static_id = None;
        let mut item_type = None;

        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "id" => id = Some(map.next_value()?),
                "static_id" => static_id = Some(map.next_value()?),
                "item_type" => {
                    item_type = Some(map.next_value_seed(PalDynamicItemTypeSeed {
                        path: self.path,
                        schemas: self.schemas,
                    })?)
                }
                other => {
                    return Err(serde::de::Error::unknown_field(
                        other,
                        &["id", "static_id", "item_type"],
                    ))
                }
            }
        }

        Ok(PalDynamicItem {
            id: id.ok_or_else(|| serde::de::Error::missing_field("id"))?,
            static_id: static_id.ok_or_else(|| serde::de::Error::missing_field("static_id"))?,
            item_type: item_type.ok_or_else(|| serde::de::Error::missing_field("item_type"))?,
        })
    }
}

struct PalDynamicItemTypeSeed<'a> {
    path: &'a str,
    schemas: &'a PropertySchemas,
}

impl<'de> DeserializeSeed<'de> for PalDynamicItemTypeSeed<'_> {
    type Value = PalDynamicItemType;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(self)
    }
}

impl<'de> Visitor<'de> for PalDynamicItemTypeSeed<'_> {
    type Value = PalDynamicItemType;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("externally tagged PalDynamicItemType variant")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        #[derive(Deserialize)]
        struct UnknownContent {
            trailer: Vec<u8>,
        }
        #[derive(Deserialize)]
        struct ArmorContent {
            leading_bytes: [u8; 4],
            durability: f32,
            trailing_bytes: [u8; 4],
        }
        #[derive(Deserialize)]
        struct WeaponContent {
            leading_bytes: [u8; 4],
            durability: f32,
            remaining_bullets: i32,
            passive_skill_list: Vec<String>,
            unknown_str: Option<String>,
            trailing_bytes: [u8; 4],
        }

        let variant = map
            .next_key::<String>()?
            .ok_or_else(|| serde::de::Error::custom("expected PalDynamicItemType variant"))?;
        Ok(match variant.as_str() {
            "Unknown" => {
                let content: UnknownContent = map.next_value()?;
                PalDynamicItemType::Unknown {
                    trailer: content.trailer,
                }
            }
            "Egg" => map.next_value_seed(PalDynamicItemEggSeed {
                path: self.path,
                schemas: self.schemas,
            })?,
            "Armor" => {
                let content: ArmorContent = map.next_value()?;
                PalDynamicItemType::Armor {
                    leading_bytes: content.leading_bytes,
                    durability: content.durability,
                    trailing_bytes: content.trailing_bytes,
                }
            }
            "Weapon" => {
                let content: WeaponContent = map.next_value()?;
                PalDynamicItemType::Weapon {
                    leading_bytes: content.leading_bytes,
                    durability: content.durability,
                    remaining_bullets: content.remaining_bullets,
                    passive_skill_list: content.passive_skill_list,
                    unknown_str: content.unknown_str,
                    trailing_bytes: content.trailing_bytes,
                }
            }
            other => {
                return Err(serde::de::Error::unknown_variant(
                    other,
                    &["Unknown", "Egg", "Armor", "Weapon"],
                ))
            }
        })
    }
}

struct PalDynamicItemEggSeed<'a> {
    path: &'a str,
    schemas: &'a PropertySchemas,
}

impl<'de> DeserializeSeed<'de> for PalDynamicItemEggSeed<'_> {
    type Value = PalDynamicItemType;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(self)
    }
}

impl<'de> Visitor<'de> for PalDynamicItemEggSeed<'_> {
    type Value = PalDynamicItemType;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("PalDynamicItemType::Egg map")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut leading_bytes = None;
        let mut character_id = None;
        let mut object = None;
        let mut trailing_bytes = None;

        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "leading_bytes" => leading_bytes = Some(map.next_value()?),
                "character_id" => character_id = Some(map.next_value()?),
                "object" => {
                    object = Some(map.next_value_seed(PropertiesSeed {
                        path: self.path,
                        schemas: self.schemas,
                    })?)
                }
                "trailing_bytes" => trailing_bytes = Some(map.next_value()?),
                other => {
                    return Err(serde::de::Error::unknown_field(
                        other,
                        &["leading_bytes", "character_id", "object", "trailing_bytes"],
                    ))
                }
            }
        }

        Ok(PalDynamicItemType::Egg {
            leading_bytes: leading_bytes
                .ok_or_else(|| serde::de::Error::missing_field("leading_bytes"))?,
            character_id: character_id
                .ok_or_else(|| serde::de::Error::missing_field("character_id"))?,
            object: object.ok_or_else(|| serde::de::Error::missing_field("object"))?,
            trailing_bytes: trailing_bytes
                .ok_or_else(|| serde::de::Error::missing_field("trailing_bytes"))?,
        })
    }
}

struct PalMapConcreteModelSeed<'a> {
    path: &'a str,
    schemas: &'a PropertySchemas,
}

impl<'de> DeserializeSeed<'de> for PalMapConcreteModelSeed<'_> {
    type Value = PalMapConcreteModel;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(self)
    }
}

impl<'de> Visitor<'de> for PalMapConcreteModelSeed<'_> {
    type Value = PalMapConcreteModel;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("PalMapConcreteModel map")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut instance_id = None;
        let mut model_instance_id = None;
        let mut concrete_model_type = None;
        let mut model_data = None;

        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "instance_id" => instance_id = Some(map.next_value()?),
                "model_instance_id" => model_instance_id = Some(map.next_value()?),
                "concrete_model_type" => concrete_model_type = Some(map.next_value()?),
                "model_data" => {
                    model_data = Some(map.next_value_seed(PalMapConcreteModelVariantSeed {
                        path: self.path,
                        schemas: self.schemas,
                    })?)
                }
                other => {
                    return Err(serde::de::Error::unknown_field(
                        other,
                        &[
                            "instance_id",
                            "model_instance_id",
                            "concrete_model_type",
                            "model_data",
                        ],
                    ))
                }
            }
        }

        Ok(PalMapConcreteModel {
            instance_id: instance_id
                .ok_or_else(|| serde::de::Error::missing_field("instance_id"))?,
            model_instance_id: model_instance_id
                .ok_or_else(|| serde::de::Error::missing_field("model_instance_id"))?,
            concrete_model_type: concrete_model_type
                .ok_or_else(|| serde::de::Error::missing_field("concrete_model_type"))?,
            model_data: model_data.ok_or_else(|| serde::de::Error::missing_field("model_data"))?,
        })
    }
}

struct PalMapConcreteModelVariantSeed<'a> {
    path: &'a str,
    schemas: &'a PropertySchemas,
}

impl<'de> DeserializeSeed<'de> for PalMapConcreteModelVariantSeed<'_> {
    type Value = PalMapConcreteModelVariant;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(self)
    }
}

impl<'de> Visitor<'de> for PalMapConcreteModelVariantSeed<'_> {
    type Value = PalMapConcreteModelVariant;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("externally tagged PalMapConcreteModelVariant")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        use PalMapConcreteModelVariant as V;

        let variant = map.next_key::<String>()?.ok_or_else(|| {
            serde::de::Error::custom("expected PalMapConcreteModelVariant variant")
        })?;
        Ok(match variant.as_str() {
            "CharacterTeamMission" => V::CharacterTeamMission(map.next_value()?),
            "FarmSkillFruits" => V::FarmSkillFruits(map.next_value()?),
            "SupplyStorage" => V::SupplyStorage(map.next_value()?),
            "ItemBooth" => V::ItemBooth(map.next_value()?),
            "EnergyStorage" => V::EnergyStorage(map.next_value()?),
            "DeathDroppedCharacter" => V::DeathDroppedCharacter(map.next_value()?),
            "ConvertItem" => V::ConvertItem(map.next_value()?),
            "PickupItemOnLevel" => V::PickupItemOnLevel(map.next_value()?),
            "DropItem" => V::DropItem(map.next_value()?),
            "ItemDropOnDamag" => V::ItemDropOnDamag(map.next_value()?),
            "DeathPenaltyStorage" => V::DeathPenaltyStorage(map.next_value()?),
            "DefenseBulletLauncher" => V::DefenseBulletLauncher(map.next_value()?),
            "GenerateEnergy" => V::GenerateEnergy(map.next_value()?),
            "FarmBlockV2" => V::FarmBlockV2(map.next_value()?),
            "FastTravelPoint" => V::FastTravelPoint(map.next_value()?),
            "ShippingItem" => V::ShippingItem(map.next_value()?),
            "ProductItem" => V::ProductItem(map.next_value()?),
            "RecoverOtomo" => V::RecoverOtomo(map.next_value()?),
            "HatchingEgg" => {
                V::HatchingEgg(map.next_value_seed(PalMapObjectHatchingEggModelSeed {
                    path: self.path,
                    schemas: self.schemas,
                })?)
            }
            "TreasureBox" => V::TreasureBox(map.next_value()?),
            "BreedFarm" => V::BreedFarm(map.next_value()?),
            "Signboard" => V::Signboard(map.next_value()?),
            "Lamp" => V::Lamp(map.next_value()?),
            "Torch" => V::Torch(map.next_value()?),
            "PalEgg" => V::PalEgg(map.next_value()?),
            "BaseCampPoint" => V::BaseCampPoint(map.next_value()?),
            "ItemChest" => V::ItemChest(map.next_value()?),
            "ItemChestAffectCorruption" => V::ItemChestAffectCorruption(map.next_value()?),
            "Unknown" => V::Unknown(map.next_value()?),
            other => {
                return Err(serde::de::Error::custom(format!(
                    "unknown PalMapConcreteModelVariant: {other}"
                )))
            }
        })
    }
}
