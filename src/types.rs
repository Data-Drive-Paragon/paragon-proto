use heapless::Vec;

pub const MIME_MAX_LEN: usize = 64;
pub const SIGN_MAX_LEN: usize = 64;
pub const DATA_MAX_LEN: usize = 4096;
pub const DESTINATION_MAX_LEN: usize = 128;
pub const CLASS_MAX_LEN: usize = 64;
pub const PACKAGES_MAX_COUNT: usize = 16;
pub const KEY_SIZE: usize = 32;
pub const NONCE_SIZE: usize = 12;

pub const FIELD_NAME_MAX_LEN: usize = 32;
pub const FIELD_DESC_MAX_LEN: usize = 64;
pub const FIELD_VALUE_MAX_LEN: usize = 128;
pub const MAX_PRE_STRUCTURED_FIELDS: usize = 16;
pub const PPSP_MIME_TYPE: &[u8] = b"application/vnd.paragon.ppsp";

pub const MAGIC: u32 = 0x47414E41;
pub const VERSION: u16 = 1;
pub const HEADER_SIZE: usize = core::mem::size_of::<Header>();

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum FieldType {
    String = 1,
    Int = 2,
    Float = 3,
    Bool = 4,
    Bytes = 5,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct PreStructuredField {
    pub name: Vec<u8, FIELD_NAME_MAX_LEN>,
    pub description: Vec<u8, FIELD_DESC_MAX_LEN>,
    pub field_type: FieldType,
    pub value: Vec<u8, FIELD_VALUE_MAX_LEN>,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct ParagonPreStructuredPackage {
    pub fields: Vec<PreStructuredField, MAX_PRE_STRUCTURED_FIELDS>,
}

#[repr(C)]
pub struct Header {
    pub magic: u32,
    pub version: u16,
    pub flags: u16,
    pub payload_len: u32,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct ParagonPackage {
    pub version: [u8; 4],
    pub mime_type: Vec<u8, MIME_MAX_LEN>,
    pub raw_data: Vec<u8, DATA_MAX_LEN>,
    pub sign: Vec<u8, SIGN_MAX_LEN>,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct ParagonPackageEncrypted {
    pub version: [u8; 4],
    pub mime_type: Vec<u8, MIME_MAX_LEN>,
    pub encrypted_data: Vec<u8, DATA_MAX_LEN>,
    pub sign: Vec<u8, SIGN_MAX_LEN>,
}

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum EncryptionType {
    None = 0,
    X25519ChaCha20Poly1305 = 1,
    X25519Aes256Gcm = 2,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct ParagonDataFrame {
    pub packages: Vec<ParagonPackageEncrypted, PACKAGES_MAX_COUNT>,
    pub class: Vec<u8, CLASS_MAX_LEN>,
    pub destination: Vec<u8, DESTINATION_MAX_LEN>,
    pub enc: EncryptionType,
}

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum ParseError {
    Ok = 0,
    BadMagic = 1,
    BadVersion = 2,
    TooShort = 3,
    InvalidDestination = 4,
    InvalidPackageMix = 5,
    StructuralMismatch = 6,
    InvalidPpsp = 7,
}

#[repr(C)]
pub struct KeyPair {
    pub public: [u8; 32],
    pub private: [u8; 32],
}
