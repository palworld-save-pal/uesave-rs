use crate::{
    ArchiveReader, ArchiveType, ArchiveWriter, FGuid, Properties, Result, SaveGameArchiveType,
};
use byteorder::ReadBytesExt;
use serde::{Deserialize, Serialize};
use std::io::SeekFrom;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(bound(
    serialize = "T::ObjectRef: Serialize, T::SoftObjectPath: Serialize",
    deserialize = ""
))]
pub struct PalCharacterData<T: ArchiveType = SaveGameArchiveType> {
    pub object: Properties<T>,
    /// Present in saves written by older Palworld versions; absent (`None`)
    /// in newer ones, which write only [`group_id`](Self::group_id). Detected
    /// at read time from how many bytes remain in this entry's embedded byte
    /// range — see [`Self::read`].
    pub unknown_bytes: Option<[u8; 4]>,
    pub group_id: FGuid,
    /// Sibling of [`unknown_bytes`](Self::unknown_bytes): present together,
    /// absent together.
    pub trailing_bytes: Option<[u8; 4]>,
}

impl<T: ArchiveType> PalCharacterData<T> {
    pub fn read<A: ArchiveReader<ArchiveType = T>>(ar: &mut A) -> Result<Self> {
        let object = crate::read_properties_until_none(ar)?;

        // The trailer after the property list is 24 bytes (4 unknown + a
        // 16-byte group GUID + 4 more unknown) in saves from older Palworld
        // versions, and just the bare 16-byte GUID in newer ones — the two
        // 4-byte fields were dropped at some point. `PalCharacterData::read`
        // is always called on a byte range scoped to exactly this entry (the
        // caller hands it a fresh `Cursor` over the entry's own bytes), so
        // seeking to the end and back tells us which layout this entry uses
        // without consuming anything.
        let pos = ar.stream_position()?;
        let end = ar.seek(SeekFrom::End(0))?;
        ar.seek(SeekFrom::Start(pos))?;
        let has_legacy_padding = end.saturating_sub(pos) >= 24;

        let unknown_bytes = if has_legacy_padding {
            let mut bytes = [0; 4];
            ar.read_exact(&mut bytes)?;
            Some(bytes)
        } else {
            None
        };
        let group_id = FGuid::read(ar)?;
        let trailing_bytes = if has_legacy_padding {
            let mut bytes = [0; 4];
            ar.read_exact(&mut bytes)?;
            Some(bytes)
        } else {
            None
        };

        Ok(PalCharacterData {
            object,
            unknown_bytes,
            group_id,
            trailing_bytes,
        })
    }
    pub fn write<A: ArchiveWriter<ArchiveType = T>>(&self, ar: &mut A) -> Result<()> {
        crate::write_properties_none_terminated(ar, &self.object)?;
        if let Some(bytes) = &self.unknown_bytes {
            ar.write_all(bytes)?;
        }
        self.group_id.write(ar)?;
        if let Some(bytes) = &self.trailing_bytes {
            ar.write_all(bytes)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PalCharacterContainer {
    pub player_uid: FGuid,
    pub instance_id: FGuid,
    pub permission_tribe_id: u8,
    pub trailing_bytes: Option<Vec<u8>>,
}

impl PalCharacterContainer {
    pub fn read<A: ArchiveReader>(ar: &mut A) -> Result<Self> {
        let player_uid = FGuid::read(ar)?;
        let instance_id = FGuid::read(ar)?;
        let permission_tribe_id = ar.read_u8()?;
        let mut trailing_bytes = Vec::new();
        if ar.read_to_end(&mut trailing_bytes)? > 0 {
            Ok(PalCharacterContainer {
                player_uid,
                instance_id,
                permission_tribe_id,
                trailing_bytes: Some(trailing_bytes),
            })
        } else {
            Ok(PalCharacterContainer {
                player_uid,
                instance_id,
                permission_tribe_id,
                trailing_bytes: None,
            })
        }
    }
    pub fn write<A: ArchiveWriter>(&self, ar: &mut A) -> Result<()> {
        self.player_uid.write(ar)?;
        self.instance_id.write(ar)?;
        ar.write_all(&[self.permission_tribe_id])?;
        if let Some(trailing) = &self.trailing_bytes {
            ar.write_all(trailing)?;
        }
        Ok(())
    }
}
