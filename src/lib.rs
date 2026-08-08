#![cfg_attr(not(test), no_std)]

#[cfg(not(test))]
use core::panic::PanicInfo;

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[repr(C)]
pub struct Header {
    pub magic: u32,
    pub version: u16,
    pub flags: u16,
    pub payload_len: u32,
}

#[repr(u8)]
pub enum ParseError {
    Ok = 0,
    BadMagic = 1,
    BadVersion = 2,
    TooShort = 3,
}

const MAGIC: u32 = 0x47414E41; // "PARAGON"
const VERSION: u16 = 1;
const HEADER_SIZE: usize = core::mem::size_of::<Header>();

#[no_mangle]
pub unsafe extern "C" fn proto_parse_header(
    data: *const u8,
    len: usize,
    out_header: *mut Header,
) -> u8 {
    if data.is_null() || out_header.is_null() {
        return ParseError::TooShort as u8;
    }
    if len < HEADER_SIZE {
        return ParseError::TooShort as u8;
    }

    let magic = u32::from_le_bytes([
        *data.add(0),
        *data.add(1),
        *data.add(2),
        *data.add(3),
    ]);
    if magic != MAGIC {
        return ParseError::BadMagic as u8;
    }

    let version = u16::from_le_bytes([*data.add(4), *data.add(5)]);
    if version != VERSION {
        return ParseError::BadVersion as u8;
    }

    let flags = u16::from_le_bytes([*data.add(6), *data.add(7)]);
    let payload_len = u32::from_le_bytes([
        *data.add(8),
        *data.add(9),
        *data.add(10),
        *data.add(11),
    ]);

    (*out_header) = Header {
        magic,
        version,
        flags,
        payload_len,
    };

    ParseError::Ok as u8
}

#[no_mangle]
pub extern "C" fn proto_version() -> u16 {
    VERSION
}
