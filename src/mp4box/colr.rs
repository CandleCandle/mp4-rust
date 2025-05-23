use serde::Serialize;
use std::io::{Read, Seek, Write};

use crate::mp4box::*;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct ColrBox {
    pub nclx: Option<NclxConfig>,
    // pub ricc: Option<RiccConfig>,
    // pub prof: OPtion<ProfConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[allow(dead_code)]
pub enum ColourType {
    Nclx = 0x6e636c78, // nclx
    Ricc = 0x72494343, // rICC
    Prof = 0x70726f66, // prof
}
impl Default for ColourType {
    fn default() -> ColourType { ColourType::Nclx }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct NclxConfig {
    pub colour_primaries: u16,
    pub transfer_characteristics: u16,
    pub matrix_coefficients: u16,
    pub full_range_flag: bool,
}

impl ColrBox {
    pub(crate) fn new() -> ColrBox {
        Default::default()
    }

    pub fn get_type(&self) -> BoxType {
        BoxType::ColrBox
    }

    pub fn get_size(&self) -> u64 {
        let mut size = HEADER_SIZE;
        size += size_of::<u32>() as u64;
        if self.nclx.is_some() {
            size += 7
        }
        size
    }
}

impl Mp4Box for ColrBox {
    fn box_type(&self) -> BoxType {
        self.get_type()
    }

    fn box_size(&self) -> u64 {
        self.get_size()
    }

    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        let s = String::new();
        Ok(s)
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for ColrBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let mut colr = ColrBox::new();
        let colour_type = reader.read_u32::<BigEndian>()?;
        match colour_type {
            t if t == (ColourType::Nclx as u32) => {
                colr.nclx = Some(NclxConfig {
                    colour_primaries: reader.read_u16::<BigEndian>()?,
                    transfer_characteristics: reader.read_u16::<BigEndian>()?,
                    matrix_coefficients: reader.read_u16::<BigEndian>()?,
                    full_range_flag: (reader.read_u8()? & 0b1000_0000) > 0, // remaining 7 bits are reserved
                });
            }
            _ => {
                return Err(Error::InvalidData(
                    "colr box contains an unimplemented or unknown colour_type",
                ))
            }
        }

        skip_bytes_to(reader, start + size)?;
        Ok(colr)
    }
}

impl<W: Write> WriteBox<&mut W> for ColrBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        if let Some(ref nclx) = self.nclx {
            writer.write_u32::<BigEndian>(ColourType::Nclx as u32).unwrap();
            writer.write_u16::<BigEndian>(nclx.colour_primaries).unwrap();
            writer.write_u16::<BigEndian>(nclx.transfer_characteristics).unwrap();
            writer.write_u16::<BigEndian>(nclx.matrix_coefficients).unwrap();
            writer.write_u8((nclx.full_range_flag as u8) << 7).unwrap();
        };

        Ok(size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mp4box::BoxHeader;
    use std::io::Cursor;

    #[test]
    fn test_colr() {
        let src_box = ColrBox {
            nclx: Some(NclxConfig {
                colour_primaries: 1,
                transfer_characteristics: 0xFFFF,
                matrix_coefficients: 0x4444,
                full_range_flag: true,
            }),
        };
        let mut buf = Vec::new();
        src_box.write_box(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.box_size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = BoxHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, BoxType::ColrBox);
        assert_eq!(src_box.box_size(), header.size);

        let dst_box = ColrBox::read_box(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }
}
