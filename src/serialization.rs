use crate::types::{
    Header, ParagonPackage, ParagonPackageEncrypted, ParagonDataFrame,
    ParagonPreStructuredPackage, PreStructuredField, FieldType, ParseError,
    MAGIC, VERSION, HEADER_SIZE, PPSP_MIME_TYPE
};
use crate::validation::validate_destination;
use heapless::Vec;

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

#[no_mangle]
pub extern "C" fn package_serialize(
    package: *const ParagonPackage,
    buffer: *mut u8,
    buffer_len: usize,
    out_len: *mut usize,
) -> u8 {
    if package.is_null() || buffer.is_null() || out_len.is_null() {
        return 1;
    }
    
    let pkg = unsafe { &*package };
    let mut offset = 0;
    
    let total_len = 4 + 1 + pkg.mime_type.len() + 2 + pkg.raw_data.len() + 1 + pkg.sign.len();
    if buffer_len < total_len {
        return 1;
    }
    
    let buf = unsafe { core::slice::from_raw_parts_mut(buffer, buffer_len) };
    
    buf[offset..offset + 4].copy_from_slice(&pkg.version);
    offset += 4;
    
    buf[offset] = pkg.mime_type.len() as u8;
    offset += 1;
    buf[offset..offset + pkg.mime_type.len()].copy_from_slice(&pkg.mime_type);
    offset += pkg.mime_type.len();
    
    let data_len = pkg.raw_data.len() as u16;
    buf[offset..offset + 2].copy_from_slice(&data_len.to_le_bytes());
    offset += 2;
    buf[offset..offset + pkg.raw_data.len()].copy_from_slice(&pkg.raw_data);
    offset += pkg.raw_data.len();
    
    buf[offset] = pkg.sign.len() as u8;
    offset += 1;
    buf[offset..offset + pkg.sign.len()].copy_from_slice(&pkg.sign);
    offset += pkg.sign.len();
    
    unsafe { *out_len = offset };
    0
}

#[no_mangle]
pub extern "C" fn package_deserialize(
    buffer: *const u8,
    buffer_len: usize,
    package: *mut ParagonPackage,
) -> u8 {
    if buffer.is_null() || package.is_null() {
        return 1;
    }
    
    let buf = unsafe { core::slice::from_raw_parts(buffer, buffer_len) };
    let mut offset = 0;
    
    if buffer_len < 4 {
        return 1;
    }
    
    let pkg = unsafe { &mut *package };
    pkg.version.copy_from_slice(&buf[offset..offset + 4]);
    offset += 4;
    
    if offset >= buffer_len {
        return 1;
    }
    let mime_len = buf[offset] as usize;
    offset += 1;
    
    if offset + mime_len > buffer_len {
        return 1;
    }
    pkg.mime_type.clear();
    for i in 0..mime_len {
        if pkg.mime_type.push(buf[offset + i]).is_err() {
            return 1;
        }
    }
    offset += mime_len;
    
    if offset + 2 > buffer_len {
        return 1;
    }
    let data_len = u16::from_le_bytes([buf[offset], buf[offset + 1]]) as usize;
    offset += 2;
    
    if offset + data_len > buffer_len {
        return 1;
    }
    pkg.raw_data.clear();
    for i in 0..data_len {
        if pkg.raw_data.push(buf[offset + i]).is_err() {
            return 1;
        }
    }
    offset += data_len;
    
    if offset >= buffer_len {
        return 1;
    }
    let sign_len = buf[offset] as usize;
    offset += 1;
    
    if offset + sign_len > buffer_len {
        return 1;
    }
    pkg.sign.clear();
    for i in 0..sign_len {
        if pkg.sign.push(buf[offset + i]).is_err() {
            return 1;
        }
    }
    
    0
}

pub fn ppsp_serialize(
    ppsp: &ParagonPreStructuredPackage,
    buffer: &mut [u8],
) -> Result<usize, u8> {
    let mut offset = 0;
    if buffer.len() < 1 {
        return Err(1);
    }
    if ppsp.fields.len() > 255 {
        return Err(1);
    }
    buffer[offset] = ppsp.fields.len() as u8;
    offset += 1;

    for field in &ppsp.fields {
        if offset + 1 + 1 + field.name.len() + 1 + field.description.len() + 2 + field.value.len() > buffer.len() {
            return Err(1);
        }
        buffer[offset] = field.field_type as u8;
        offset += 1;

        buffer[offset] = field.name.len() as u8;
        offset += 1;
        buffer[offset..offset + field.name.len()].copy_from_slice(&field.name);
        offset += field.name.len();

        buffer[offset] = field.description.len() as u8;
        offset += 1;
        buffer[offset..offset + field.description.len()].copy_from_slice(&field.description);
        offset += field.description.len();

        let val_len = field.value.len() as u16;
        buffer[offset..offset + 2].copy_from_slice(&val_len.to_le_bytes());
        offset += 2;
        buffer[offset..offset + field.value.len()].copy_from_slice(&field.value);
        offset += field.value.len();
    }
    Ok(offset)
}

pub fn ppsp_deserialize(
    buffer: &[u8],
    ppsp: &mut ParagonPreStructuredPackage,
) -> Result<(), u8> {
    let mut offset = 0;
    if buffer.len() < 1 {
        return Err(1);
    }
    let fields_count = buffer[offset] as usize;
    offset += 1;

    ppsp.fields.clear();

    for _ in 0..fields_count {
        if offset >= buffer.len() {
            return Err(1);
        }
        let field_type_u8 = buffer[offset];
        offset += 1;
        let field_type = match field_type_u8 {
            1 => FieldType::String,
            2 => FieldType::Int,
            3 => FieldType::Float,
            4 => FieldType::Bool,
            5 => FieldType::Bytes,
            _ => return Err(1),
        };

        if offset >= buffer.len() {
            return Err(1);
        }
        let name_len = buffer[offset] as usize;
        offset += 1;
        if offset + name_len > buffer.len() {
            return Err(1);
        }
        let mut name = Vec::new();
        for i in 0..name_len {
            if name.push(buffer[offset + i]).is_err() {
                return Err(1);
            }
        }
        offset += name_len;

        if offset >= buffer.len() {
            return Err(1);
        }
        let desc_len = buffer[offset] as usize;
        offset += 1;
        if offset + desc_len > buffer.len() {
            return Err(1);
        }
        let mut description = Vec::new();
        for i in 0..desc_len {
            if description.push(buffer[offset + i]).is_err() {
                return Err(1);
            }
        }
        offset += desc_len;

        if offset + 2 > buffer.len() {
            return Err(1);
        }
        let val_len = u16::from_le_bytes([buffer[offset], buffer[offset + 1]]) as usize;
        offset += 2;
        if offset + val_len > buffer.len() {
            return Err(1);
        }
        let mut value = Vec::new();
        for i in 0..val_len {
            if value.push(buffer[offset + i]).is_err() {
                return Err(1);
            }
        }
        offset += val_len;

        let field = PreStructuredField {
            name,
            description,
            field_type,
            value,
        };
        if ppsp.fields.push(field).is_err() {
            return Err(1);
        }
    }
    Ok(())
}

pub fn package_encrypted_serialize(
    pkg: &ParagonPackageEncrypted,
    buffer: &mut [u8],
) -> Result<usize, u8> {
    let mut offset = 0;
    let total_len = 4 + 1 + pkg.mime_type.len() + 2 + pkg.encrypted_data.len() + 1 + pkg.sign.len();
    if buffer.len() < total_len {
        return Err(1);
    }

    buffer[offset..offset + 4].copy_from_slice(&pkg.version);
    offset += 4;

    buffer[offset] = pkg.mime_type.len() as u8;
    offset += 1;
    buffer[offset..offset + pkg.mime_type.len()].copy_from_slice(&pkg.mime_type);
    offset += pkg.mime_type.len();

    let data_len = pkg.encrypted_data.len() as u16;
    buffer[offset..offset + 2].copy_from_slice(&data_len.to_le_bytes());
    offset += 2;
    buffer[offset..offset + pkg.encrypted_data.len()].copy_from_slice(&pkg.encrypted_data);
    offset += pkg.encrypted_data.len();

    buffer[offset] = pkg.sign.len() as u8;
    offset += 1;
    buffer[offset..offset + pkg.sign.len()].copy_from_slice(&pkg.sign);
    offset += pkg.sign.len();

    Ok(offset)
}

pub fn package_encrypted_deserialize(
    buffer: &[u8],
    pkg: &mut ParagonPackageEncrypted,
) -> Result<usize, u8> {
    let mut offset = 0;
    if buffer.len() < 4 {
        return Err(1);
    }

    pkg.version.copy_from_slice(&buffer[offset..offset + 4]);
    offset += 4;

    if offset >= buffer.len() {
        return Err(1);
    }
    let mime_len = buffer[offset] as usize;
    offset += 1;
    if offset + mime_len > buffer.len() {
        return Err(1);
    }
    pkg.mime_type.clear();
    for i in 0..mime_len {
        if pkg.mime_type.push(buffer[offset + i]).is_err() {
            return Err(1);
        }
    }
    offset += mime_len;

    if offset + 2 > buffer.len() {
        return Err(1);
    }
    let data_len = u16::from_le_bytes([buffer[offset], buffer[offset + 1]]) as usize;
    offset += 2;
    if offset + data_len > buffer.len() {
        return Err(1);
    }
    pkg.encrypted_data.clear();
    for i in 0..data_len {
        if pkg.encrypted_data.push(buffer[offset + i]).is_err() {
            return Err(1);
        }
    }
    offset += data_len;

    if offset >= buffer.len() {
        return Err(1);
    }
    let sign_len = buffer[offset] as usize;
    offset += 1;
    if offset + sign_len > buffer.len() {
        return Err(1);
    }
    pkg.sign.clear();
    for i in 0..sign_len {
        if pkg.sign.push(buffer[offset + i]).is_err() {
            return Err(1);
        }
    }
    offset += sign_len;

    Ok(offset)
}

pub fn validate_dataframe_packages(df: &ParagonDataFrame) -> Result<(), u8> {
    if df.packages.is_empty() {
        return Ok(());
    }

    let mut ppsp_count = 0;
    for pkg in &df.packages {
        if pkg.mime_type.as_slice() == PPSP_MIME_TYPE {
            ppsp_count += 1;
        }
    }

    if ppsp_count > 0 && ppsp_count < df.packages.len() {
        return Err(ParseError::InvalidPackageMix as u8);
    }

    if ppsp_count > 0 {
        let mut reference_ppsp: Option<ParagonPreStructuredPackage> = None;

        for pkg in &df.packages {
            let mut ppsp = ParagonPreStructuredPackage {
                fields: Vec::new(),
            };
            if ppsp_deserialize(&pkg.encrypted_data, &mut ppsp).is_err() {
                return Err(ParseError::InvalidPpsp as u8);
            }

            match reference_ppsp {
                None => {
                    reference_ppsp = Some(ppsp);
                }
                Some(ref r_ppsp) => {
                    if ppsp.fields.len() != r_ppsp.fields.len() {
                        return Err(ParseError::StructuralMismatch as u8);
                    }
                    for (f, rf) in ppsp.fields.iter().zip(r_ppsp.fields.iter()) {
                        if f.field_type != rf.field_type
                            || f.name != rf.name
                            || f.description != rf.description
                        {
                            return Err(ParseError::StructuralMismatch as u8);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

#[no_mangle]
pub extern "C" fn dataframe_serialize(
    dataframe: *const ParagonDataFrame,
    buffer: *mut u8,
    buffer_len: usize,
    out_len: *mut usize,
) -> u8 {
    if dataframe.is_null() || buffer.is_null() || out_len.is_null() {
        return 1;
    }
    
    let df = unsafe { &*dataframe };
    
    if !validate_destination(&df.destination) {
        return ParseError::InvalidDestination as u8;
    }
    
    if let Err(e) = validate_dataframe_packages(df) {
        return e;
    }
    
    let mut offset = 0;
    if buffer_len < 1 {
        return 1;
    }
    let buf = unsafe { core::slice::from_raw_parts_mut(buffer, buffer_len) };
    
    buf[offset] = df.packages.len() as u8;
    offset += 1;
    
    for pkg in &df.packages {
        let mut pkg_buf = [0u8; 4096];
        match package_encrypted_serialize(pkg, &mut pkg_buf) {
            Ok(n) => {
                if offset + 2 + n > buffer_len {
                    return 1;
                }
                let len_u16 = n as u16;
                buf[offset..offset + 2].copy_from_slice(&len_u16.to_le_bytes());
                offset += 2;
                buf[offset..offset + n].copy_from_slice(&pkg_buf[..n]);
                offset += n;
            }
            Err(_) => return 1,
        }
    }
    
    if offset + 1 + 1 + df.class.len() + 1 + df.destination.len() > buffer_len {
        return 1;
    }
    
    buf[offset] = df.enc as u8;
    offset += 1;
    
    buf[offset] = df.class.len() as u8;
    offset += 1;
    buf[offset..offset + df.class.len()].copy_from_slice(&df.class);
    offset += df.class.len();
    
    buf[offset] = df.destination.len() as u8;
    offset += 1;
    buf[offset..offset + df.destination.len()].copy_from_slice(&df.destination);
    offset += df.destination.len();
    
    unsafe { *out_len = offset };
    0
}

#[no_mangle]
pub extern "C" fn dataframe_deserialize(
    buffer: *const u8,
    buffer_len: usize,
    dataframe: *mut ParagonDataFrame,
) -> u8 {
    if buffer.is_null() || dataframe.is_null() {
        return 1;
    }
    
    let buf = unsafe { core::slice::from_raw_parts(buffer, buffer_len) };
    let mut offset = 0;
    
    if buffer_len < 1 {
        return 1;
    }
    
    let df = unsafe { &mut *dataframe };
    df.packages.clear();
    
    let packages_count = buf[offset] as usize;
    offset += 1;
    
    for _ in 0..packages_count {
        if offset + 2 > buffer_len {
            return 1;
        }
        let pkg_len = u16::from_le_bytes([buf[offset], buf[offset + 1]]) as usize;
        offset += 2;
        
        if offset + pkg_len > buffer_len {
            return 1;
        }
        let mut pkg = ParagonPackageEncrypted {
            version: [0; 4],
            mime_type: Vec::new(),
            encrypted_data: Vec::new(),
            sign: Vec::new(),
        };
        match package_encrypted_deserialize(&buf[offset..offset + pkg_len], &mut pkg) {
            Ok(_) => {
                if df.packages.push(pkg).is_err() {
                    return 1;
                }
            }
            Err(_) => return 1,
        }
        offset += pkg_len;
    }
    
    if offset >= buffer_len {
        return 1;
    }
    let enc_type = buf[offset];
    offset += 1;
    df.enc = match enc_type {
        0 => crate::types::EncryptionType::None,
        1 => crate::types::EncryptionType::X25519ChaCha20Poly1305,
        2 => crate::types::EncryptionType::X25519Aes256Gcm,
        _ => return 1,
    };
    
    if offset >= buffer_len {
        return 1;
    }
    let class_len = buf[offset] as usize;
    offset += 1;
    
    if offset + class_len > buffer_len {
        return 1;
    }
    df.class.clear();
    for i in 0..class_len {
        if df.class.push(buf[offset + i]).is_err() {
            return 1;
        }
    }
    offset += class_len;
    
    if offset >= buffer_len {
        return 1;
    }
    let dest_len = buf[offset] as usize;
    offset += 1;
    
    if offset + dest_len > buffer_len {
        return 1;
    }
    df.destination.clear();
    for i in 0..dest_len {
        if df.destination.push(buf[offset + i]).is_err() {
            return 1;
        }
    }
    
    if !validate_destination(&df.destination) {
        return ParseError::InvalidDestination as u8;
    }
    
    if let Err(e) = validate_dataframe_packages(df) {
        return e;
    }
    
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ParagonPackage, ParagonDataFrame, EncryptionType};

    #[test]
    fn test_proto_parse_header() {
        let mut buf = [0u8; HEADER_SIZE];
        buf[0..4].copy_from_slice(&MAGIC.to_le_bytes());
        buf[4..6].copy_from_slice(&VERSION.to_le_bytes());
        buf[6..8].copy_from_slice(&0u16.to_le_bytes());
        buf[8..12].copy_from_slice(&100u32.to_le_bytes());

        let mut header = Header {
            magic: 0,
            version: 0,
            flags: 0,
            payload_len: 0,
        };

        let res = unsafe { proto_parse_header(buf.as_ptr(), buf.len(), &mut header) };
        assert_eq!(res, ParseError::Ok as u8);
        assert_eq!(header.magic, MAGIC);
        assert_eq!(header.version, VERSION);
        assert_eq!(header.payload_len, 100);

        // Test bad magic
        buf[0] = 0;
        let res = unsafe { proto_parse_header(buf.as_ptr(), buf.len(), &mut header) };
        assert_eq!(res, ParseError::BadMagic as u8);
    }

    #[test]
    fn test_package_serialization() {
        let pkg = ParagonPackage {
            version: [1, 0, 0, 0],
            mime_type: heapless::Vec::from_slice(b"application/json").unwrap(),
            raw_data: heapless::Vec::from_slice(b"test data").unwrap(),
            sign: heapless::Vec::from_slice(b"signature").unwrap(),
        };

        let mut buffer = [0u8; 512];
        let mut out_len = 0;

        let res = package_serialize(&pkg, buffer.as_mut_ptr(), buffer.len(), &mut out_len);
        assert_eq!(res, 0);
        assert!(out_len > 0);

        let mut deserialized_pkg = ParagonPackage {
            version: [0; 4],
            mime_type: heapless::Vec::new(),
            raw_data: heapless::Vec::new(),
            sign: heapless::Vec::new(),
        };

        let des_res = package_deserialize(buffer.as_ptr(), out_len, &mut deserialized_pkg);
        assert_eq!(des_res, 0);
        assert_eq!(deserialized_pkg.version, pkg.version);
        assert_eq!(deserialized_pkg.mime_type, pkg.mime_type);
        assert_eq!(deserialized_pkg.raw_data, pkg.raw_data);
        assert_eq!(deserialized_pkg.sign, pkg.sign);
    }

    #[test]
    fn test_dataframe_serialization() {
        let df = ParagonDataFrame {
            packages: heapless::Vec::new(),
            class: heapless::Vec::from_slice(b"TestClass").unwrap(),
            destination: heapless::Vec::from_slice(b"node.service").unwrap(),
            enc: EncryptionType::X25519ChaCha20Poly1305,
        };

        let mut buffer = [0u8; 256];
        let mut out_len = 0;

        let res = dataframe_serialize(&df, buffer.as_mut_ptr(), buffer.len(), &mut out_len);
        assert_eq!(res, 0);
        assert!(out_len > 0);

        let mut des_df = ParagonDataFrame {
            packages: heapless::Vec::new(),
            class: heapless::Vec::new(),
            destination: heapless::Vec::new(),
            enc: EncryptionType::None,
        };

        let des_res = dataframe_deserialize(buffer.as_ptr(), out_len, &mut des_df);
        assert_eq!(des_res, 0);
        assert_eq!(des_df.class, df.class);
        assert_eq!(des_df.destination, df.destination);
        assert!(matches!(des_df.enc, EncryptionType::X25519ChaCha20Poly1305));
    }

    #[test]
    fn test_ppsp_serialization_and_validation() {
        let mut ppsp1 = ParagonPreStructuredPackage {
            fields: heapless::Vec::new(),
        };
        ppsp1.fields.push(PreStructuredField {
            name: heapless::Vec::from_slice(b"age").unwrap(),
            description: heapless::Vec::from_slice(b"user age").unwrap(),
            field_type: FieldType::Int,
            value: heapless::Vec::from_slice(b"30").unwrap(),
        }).unwrap();

        let mut ppsp_buf = [0u8; 512];
        let ppsp_len = ppsp_serialize(&ppsp1, &mut ppsp_buf).unwrap();

        let mut ppsp_des = ParagonPreStructuredPackage {
            fields: heapless::Vec::new(),
        };
        ppsp_deserialize(&ppsp_buf[..ppsp_len], &mut ppsp_des).unwrap();
        assert_eq!(ppsp_des.fields.len(), 1);
        assert_eq!(ppsp_des.fields[0].name, ppsp1.fields[0].name);
        assert_eq!(ppsp_des.fields[0].field_type, FieldType::Int);

        let mut ppsp2 = ParagonPreStructuredPackage {
            fields: heapless::Vec::new(),
        };
        ppsp2.fields.push(PreStructuredField {
            name: heapless::Vec::from_slice(b"age").unwrap(),
            description: heapless::Vec::from_slice(b"user age").unwrap(),
            field_type: FieldType::Int,
            value: heapless::Vec::from_slice(b"25").unwrap(),
        }).unwrap();
        let mut ppsp_buf2 = [0u8; 512];
        let ppsp_len2 = ppsp_serialize(&ppsp2, &mut ppsp_buf2).unwrap();

        let pkg1 = ParagonPackageEncrypted {
            version: [1, 0, 0, 0],
            mime_type: heapless::Vec::from_slice(PPSP_MIME_TYPE).unwrap(),
            encrypted_data: heapless::Vec::from_slice(&ppsp_buf[..ppsp_len]).unwrap(),
            sign: heapless::Vec::new(),
        };

        let pkg2 = ParagonPackageEncrypted {
            version: [1, 0, 0, 0],
            mime_type: heapless::Vec::from_slice(PPSP_MIME_TYPE).unwrap(),
            encrypted_data: heapless::Vec::from_slice(&ppsp_buf2[..ppsp_len2]).unwrap(),
            sign: heapless::Vec::new(),
        };

        let mut df = ParagonDataFrame {
            packages: heapless::Vec::new(),
            class: heapless::Vec::from_slice(b"ClassPPSP").unwrap(),
            destination: heapless::Vec::from_slice(b"node.ppsp").unwrap(),
            enc: EncryptionType::None,
        };
        df.packages.push(pkg1.clone()).unwrap();
        df.packages.push(pkg2.clone()).unwrap();

        assert!(validate_dataframe_packages(&df).is_ok());

        let mut df_buf = [0u8; 1024];
        let mut df_out_len = 0;
        let ser_res = dataframe_serialize(&df, df_buf.as_mut_ptr(), df_buf.len(), &mut df_out_len);
        assert_eq!(ser_res, 0);

        let mut des_df = ParagonDataFrame {
            packages: heapless::Vec::new(),
            class: heapless::Vec::new(),
            destination: heapless::Vec::new(),
            enc: EncryptionType::None,
        };
        let des_res = dataframe_deserialize(df_buf.as_ptr(), df_out_len, &mut des_df);
        assert_eq!(des_res, 0);
        assert_eq!(des_df.packages.len(), 2);

        // Test Invalid DataFrame: mixing PPSP package and free text package
        let free_text_pkg = ParagonPackageEncrypted {
            version: [1, 0, 0, 0],
            mime_type: heapless::Vec::from_slice(b"text/plain").unwrap(),
            encrypted_data: heapless::Vec::from_slice(b"free text").unwrap(),
            sign: heapless::Vec::new(),
        };
        let mut mixed_df = ParagonDataFrame {
            packages: heapless::Vec::new(),
            class: heapless::Vec::from_slice(b"ClassMixed").unwrap(),
            destination: heapless::Vec::from_slice(b"node.mix").unwrap(),
            enc: EncryptionType::None,
        };
        mixed_df.packages.push(pkg1.clone()).unwrap();
        mixed_df.packages.push(free_text_pkg).unwrap();

        let mix_res = validate_dataframe_packages(&mixed_df);
        assert_eq!(mix_res, Err(ParseError::InvalidPackageMix as u8));

        // Test Invalid DataFrame: structural mismatch
        let mut ppsp_mismatch = ParagonPreStructuredPackage {
            fields: heapless::Vec::new(),
        };
        ppsp_mismatch.fields.push(PreStructuredField {
            name: heapless::Vec::from_slice(b"age").unwrap(),
            description: heapless::Vec::from_slice(b"user age different description").unwrap(),
            field_type: FieldType::Int,
            value: heapless::Vec::from_slice(b"30").unwrap(),
        }).unwrap();
        let mut mismatch_buf = [0u8; 512];
        let mismatch_len = ppsp_serialize(&ppsp_mismatch, &mut mismatch_buf).unwrap();

        let pkg_mismatch = ParagonPackageEncrypted {
            version: [1, 0, 0, 0],
            mime_type: heapless::Vec::from_slice(PPSP_MIME_TYPE).unwrap(),
            encrypted_data: heapless::Vec::from_slice(&mismatch_buf[..mismatch_len]).unwrap(),
            sign: heapless::Vec::new(),
        };

        let mut mismatch_df = ParagonDataFrame {
            packages: heapless::Vec::new(),
            class: heapless::Vec::from_slice(b"ClassMismatch").unwrap(),
            destination: heapless::Vec::from_slice(b"node.mismatch").unwrap(),
            enc: EncryptionType::None,
        };
        mismatch_df.packages.push(pkg1.clone()).unwrap();
        mismatch_df.packages.push(pkg_mismatch).unwrap();

        let mismatch_res = validate_dataframe_packages(&mismatch_df);
        assert_eq!(mismatch_res, Err(ParseError::StructuralMismatch as u8));
    }
}
