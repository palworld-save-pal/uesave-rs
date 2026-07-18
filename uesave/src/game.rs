use std::convert::Infallible;
use std::io::{Read, Write};
use std::marker::PhantomData;

use serde::{Deserialize, Deserializer, Serialize};

use crate::archive::{ArchiveType, ArchiveWriter, SaveGameArchiveType};
use crate::{Error, Property, PropertyKey, PropertyTagPartial, Result, SaveGameArchive};

/// A game struct value: read/write against an archive plus native serde.
pub trait GameStruct<T: ArchiveType>:
    Clone + PartialEq + std::fmt::Debug + Serialize + for<'de> Deserialize<'de>
{
    fn write<A: ArchiveWriter<ArchiveType = T>>(&self, ar: &mut A) -> Result<()>;
}

/// The single extension point: one impl per game. Every method has a default,
/// so a game implements only what it needs.
pub trait Game: Clone + PartialEq + std::fmt::Debug + Default + Serialize + 'static {
    type Struct<T: ArchiveType>: GameStruct<T>;

    /// Does this fully-qualified type path name a game struct parsed from bytes?
    fn is_game_struct_type(_full_path: &str) -> bool {
        false
    }

    /// Read a game struct from the archive given its resolved type name.
    fn read_struct<R: Read + std::io::Seek>(
        ar: &mut SaveGameArchive<R>,
        name: &str,
    ) -> Result<Self::Struct<SaveGameArchiveType<Self>>>;

    /// Deserialize a game struct from serde given its type name.
    fn deserialize_struct<'de, D, T: ArchiveType>(
        name: &str,
        d: D,
    ) -> std::result::Result<Self::Struct<T>, D::Error>
    where
        D: Deserializer<'de>;

    /// Post-process a freshly read property (embedded-bytes → typed value).
    fn process_property_for_read<R: Read + std::io::Seek>(
        _ar: &mut SaveGameArchive<R>,
        _tag: &mut PropertyTagPartial,
        value: Property<SaveGameArchiveType<Self>>,
    ) -> Result<Property<SaveGameArchiveType<Self>>> {
        Ok(value)
    }

    /// Pre-process a property about to be written (typed value → embedded bytes).
    #[allow(clippy::type_complexity)]
    fn process_property_for_write<W: Write + std::io::Seek>(
        _ar: &mut SaveGameArchive<W>,
        _key: &PropertyKey,
        _tag: &PropertyTagPartial,
        _prop: &Property<SaveGameArchiveType<Self>>,
    ) -> Result<Option<(PropertyTagPartial, Property<SaveGameArchiveType<Self>>)>> {
        Ok(None)
    }

    /// Decode a wrapped/compressed save into plain GVAS bytes. Default: passthrough.
    fn decompress_save<R: Read>(reader: &mut R) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        Ok(buf)
    }

    /// Encode plain GVAS bytes into the game's container format. Default: identity.
    fn compress_save(data: &[u8]) -> Result<Vec<u8>> {
        Ok(data.to_vec())
    }
}

/// Uninhabited game-struct type used by `NoGame`; can never be constructed.
pub enum Never<T: ArchiveType> {
    #[doc(hidden)]
    _Never(Infallible, PhantomData<T>),
}
impl<T: ArchiveType> Clone for Never<T> {
    fn clone(&self) -> Self {
        match *self {
            Never::_Never(inf, _) => match inf {},
        }
    }
}
impl<T: ArchiveType> PartialEq for Never<T> {
    fn eq(&self, _: &Self) -> bool {
        match *self {
            Never::_Never(inf, _) => match inf {},
        }
    }
}
impl<T: ArchiveType> std::fmt::Debug for Never<T> {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Never::_Never(inf, _) => match inf {},
        }
    }
}
impl<T: ArchiveType> Serialize for Never<T> {
    fn serialize<S: serde::Serializer>(&self, _: S) -> std::result::Result<S::Ok, S::Error> {
        match *self {
            Never::_Never(inf, _) => match inf {},
        }
    }
}
impl<'de, T: ArchiveType> Deserialize<'de> for Never<T> {
    fn deserialize<D: Deserializer<'de>>(_: D) -> std::result::Result<Self, D::Error> {
        Err(serde::de::Error::custom("NoGame has no game struct types"))
    }
}
impl<T: ArchiveType> GameStruct<T> for Never<T> {
    fn write<A: ArchiveWriter<ArchiveType = T>>(&self, _: &mut A) -> Result<()> {
        match *self {
            Never::_Never(inf, _) => match inf {},
        }
    }
}

/// Core's default game — zero behavior for plain GVAS saves.
#[derive(Default, Clone, PartialEq, Debug, Serialize)]
pub struct NoGame;
impl Game for NoGame {
    type Struct<T: ArchiveType> = Never<T>;
    fn read_struct<R: Read + std::io::Seek>(
        _ar: &mut SaveGameArchive<R>,
        name: &str,
    ) -> Result<Self::Struct<SaveGameArchiveType<Self>>> {
        Err(Error::Other(format!(
            "no game loaded for struct type {name:?}"
        )))
    }
    fn deserialize_struct<'de, D, T: ArchiveType>(
        name: &str,
        _d: D,
    ) -> std::result::Result<Self::Struct<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Err(serde::de::Error::custom(format!(
            "no game loaded for struct type {name:?}"
        )))
    }
}
