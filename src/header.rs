use std::convert::TryFrom;

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MagicString {
    IWAD, // Internal WAD, contains main game data
    PWAD, // Patch WAD; contains custom levels, graphics, etc.
}

impl TryFrom<&[u8; 4]> for MagicString {
    type Error = Error;

    fn try_from(bytes: &[u8; 4]) -> std::result::Result<Self, Self::Error> {
        match bytes {
            b"IWAD" => Ok(MagicString::IWAD),
            b"PWAD" => Ok(MagicString::PWAD),
            _ => Err("Invalid WAD header identification".into()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Header {
    pub identification: MagicString,
    pub num_lumps: i32,
    pub info_table_offset: i32,
}

impl TryFrom<&[u8; 12]> for Header {
    type Error = Error;

    fn try_from(bytes: &[u8; 12]) -> Result<Self> {
        let identification = MagicString::try_from(&[bytes[0], bytes[1], bytes[2], bytes[3]])?;
        let num_lumps = i32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        let info_table_offset = i32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
        Ok(Header {
            identification,
            num_lumps,
            info_table_offset,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_id_can_convert_from_bytes() {
        let iwad_bytes = [b'I', b'W', b'A', b'D'];
        let pwad_bytes = [b'P', b'W', b'A', b'D'];

        let iwad_id = MagicString::try_from(&iwad_bytes).unwrap();
        let pwad_id = MagicString::try_from(&pwad_bytes).unwrap();

        assert_eq!(iwad_id, MagicString::IWAD);
        assert_eq!(pwad_id, MagicString::PWAD);
    }

    #[test]
    fn header_identifies_iwad_from_bytes() {
        let bytes: [u8; 12] = [
            b'I', b'W', b'A', b'D', 0x02, 0x00, 0x00, 0x00, 0x34, 0x12, 0x00, 0x00,
        ];
        let header = Header::try_from(&bytes).unwrap();
        assert_eq!(header.identification, MagicString::IWAD);
        assert_eq!(header.num_lumps, 2);
        assert_eq!(header.info_table_offset, 0x1234);
    }

    #[test]
    fn header_identifies_pwad_from_bytes() {
        let bytes: [u8; 12] = [
            b'P', b'W', b'A', b'D', 0x05, 0x00, 0x00, 0x00, 0x78, 0x56, 0x00, 0x00,
        ];
        let header = Header::try_from(&bytes).unwrap();
        assert_eq!(header.identification, MagicString::PWAD);
        assert_eq!(header.num_lumps, 5);
        assert_eq!(header.info_table_offset, 0x5678);
    }

    #[test]
    fn header_can_detect_invalid_header_id() {
        let bytes: [u8; 12] = [
            b'X', b'Y', b'Z', b'W', 0x02, 0x00, 0x00, 0x00, 0x34, 0x12, 0x00, 0x00,
        ];
        let result = Header::try_from(&bytes);
        assert!(result.is_err());
    }
}
