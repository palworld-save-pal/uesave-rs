use crate::{Error, TResult};
use byteorder::{WriteBytesExt, LE};
use std::io::{Read, Write};

#[cfg(feature = "oodle")]
use ooz_rs::{compress, decompress, OodleCompressor, OodleLevel};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionFormat {
    None,
    Oodle,
    Zlib,
    Chunk,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MagicBytes {
    GVAS,
    PLM,
    PLZ,
    CNK,
}

impl MagicBytes {
    pub fn as_bytes(&self) -> &'static [u8] {
        match self {
            MagicBytes::GVAS => b"GVAS",
            MagicBytes::PLM => b"PlM",
            MagicBytes::PLZ => b"PlZ",
            MagicBytes::CNK => b"CNK",
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        match bytes {
            b"GVAS" => Some(MagicBytes::GVAS),
            b"PlM" => Some(MagicBytes::PLM),
            b"PlZ" => Some(MagicBytes::PLZ),
            b"CNK" => Some(MagicBytes::CNK),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct CompressionHeader {
    pub uncompressed_len: u32,
    pub compressed_len: u32,
    pub magic_bytes: MagicBytes,
    pub save_type: u8,
    pub data_offset: usize,
}

impl CompressionHeader {
    pub fn read<R: Read>(reader: &mut R) -> TResult<Option<Self>> {
        let mut header_buf = vec![0u8; 24];
        let bytes_read = reader.read(&mut header_buf)?;

        if bytes_read < 12 {
            return Ok(None);
        }

        if &header_buf[0..4] == b"GVAS" {
            return Ok(None);
        }

        let mut uncompressed_len =
            u32::from_le_bytes([header_buf[0], header_buf[1], header_buf[2], header_buf[3]]);
        let mut compressed_len =
            u32::from_le_bytes([header_buf[4], header_buf[5], header_buf[6], header_buf[7]]);
        let mut magic_offset = 8;
        let mut save_type_offset = 11;
        let mut data_offset = 12;

        if bytes_read >= 24 && &header_buf[20..23] == b"CNK" {
            uncompressed_len = u32::from_le_bytes([
                header_buf[12],
                header_buf[13],
                header_buf[14],
                header_buf[15],
            ]);
            compressed_len = u32::from_le_bytes([
                header_buf[16],
                header_buf[17],
                header_buf[18],
                header_buf[19],
            ]);
            magic_offset = 20;
            save_type_offset = 23;
            data_offset = 24;
        }

        let magic_bytes = MagicBytes::from_bytes(&header_buf[magic_offset..magic_offset + 3])
            .ok_or_else(|| {
                Error::Other(format!(
                    "Unknown magic bytes: {:?}",
                    &header_buf[magic_offset..magic_offset + 3]
                ))
            })?;
        let save_type = header_buf[save_type_offset];

        Ok(Some(CompressionHeader {
            uncompressed_len,
            compressed_len,
            magic_bytes,
            save_type,
            data_offset,
        }))
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> TResult<()> {
        if self.magic_bytes != MagicBytes::PLM {
            return Err(Error::Other("Only PLM format writing is supported".into()));
        }

        writer.write_u32::<LE>(self.uncompressed_len)?;
        writer.write_u32::<LE>(self.compressed_len)?;
        writer.write_all(self.magic_bytes.as_bytes())?;
        writer.write_u8(self.save_type)?;

        Ok(())
    }
}

pub fn decompress_save<R: Read>(reader: &mut R) -> TResult<Vec<u8>> {
    let mut data = Vec::new();
    reader.read_to_end(&mut data)?;

    if let Some(header) = CompressionHeader::read(&mut &data[..])? {
        match header.magic_bytes {
            #[cfg(feature = "oodle")]
            MagicBytes::PLM => {
                let compressed_payload =
                    &data[header.data_offset..header.data_offset + header.compressed_len as usize];

                let decompressed = decompress(compressed_payload, header.uncompressed_len as usize)
                    .map_err(|e| Error::Other(format!("Oodle decompression failed: {}", e)))?;

                Ok(decompressed)
            }
            #[cfg(not(feature = "oodle"))]
            MagicBytes::PLM => Err(Error::Other(
                "Oodle compression support not enabled. Rebuild with --features oodle".into(),
            )),
            MagicBytes::PLZ => Err(Error::Other("Zlib compression not yet supported".into())),
            MagicBytes::CNK => Err(Error::Other("Chunk format not yet supported".into())),
            _ => Ok(data),
        }
    } else {
        Ok(data)
    }
}

#[cfg(feature = "oodle")]
pub fn compress_save(data: &[u8], format: CompressionFormat) -> TResult<Vec<u8>> {
    match format {
        CompressionFormat::None => Ok(data.to_vec()),
        CompressionFormat::Oodle => {
            let compressed = compress(OodleCompressor::Mermaid, OodleLevel::Normal, data)
                .map_err(|e| Error::Other(format!("Oodle compression failed: {}", e)))?;

            let header = CompressionHeader {
                uncompressed_len: data.len() as u32,
                compressed_len: compressed.len() as u32,
                magic_bytes: MagicBytes::PLM,
                save_type: 0x31,
                data_offset: 12,
            };

            let mut result = Vec::new();
            header.write(&mut result)?;
            result.extend_from_slice(&compressed);

            Ok(result)
        }
        CompressionFormat::Zlib => Err(Error::Other("Zlib compression not yet supported".into())),
        CompressionFormat::Chunk => Err(Error::Other("Chunk format not yet supported".into())),
    }
}

#[cfg(not(feature = "oodle"))]
pub fn compress_save(data: &[u8], format: CompressionFormat) -> TResult<Vec<u8>> {
    match format {
        CompressionFormat::None => Ok(data.to_vec()),
        _ => Err(Error::Other(
            "Compression support not enabled. Rebuild with --features oodle".into(),
        )),
    }
}
