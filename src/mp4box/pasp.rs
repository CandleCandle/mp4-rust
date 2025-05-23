use serde::Serialize;
use std::io::{Read, Seek, Write};

use crate::mp4box::*;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct PaspBox {
    pub h_spacing: u32,
    pub v_spacing: u32,
}

impl PaspBox {
    pub(crate) fn new() -> PaspBox {
        Default::default()
    }

    pub fn get_type(&self) -> BoxType {
        BoxType::PaspBox
    }

    pub fn get_size(&self) -> u64 {
        HEADER_SIZE
            + size_of::<u32>() as u64
            + size_of::<u32>() as u64
    }
}

impl Mp4Box for PaspBox {
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

impl<R: Read + Seek> ReadBox<&mut R> for PaspBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let mut pasp = PaspBox::new();
        pasp.h_spacing = reader.read_u32::<BigEndian>()?;
        pasp.v_spacing = reader.read_u32::<BigEndian>()?;

        skip_bytes_to(reader, start + size)?;
        Ok(pasp)
    }
}

impl<W: Write> WriteBox<&mut W> for PaspBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        writer.write_u32::<BigEndian>(self.h_spacing).unwrap();
        writer.write_u32::<BigEndian>(self.v_spacing).unwrap();

        Ok(size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mp4box::BoxHeader;
    use std::io::Cursor;

    #[test]
    fn test_pasp() {
        let src_box = PaspBox {
            h_spacing: 0xFF_FF_FF_FF,
            v_spacing: 0x00_00_00_00,
        };
        let mut buf = Vec::new();
        src_box.write_box(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.box_size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = BoxHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, BoxType::PaspBox);
        assert_eq!(src_box.box_size(), header.size);

        let dst_box = PaspBox::read_box(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }
}
