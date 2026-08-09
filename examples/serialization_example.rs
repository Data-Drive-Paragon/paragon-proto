use paragon_proto::{
    proto_version, proto_parse_header, package_serialize, package_deserialize,
    dataframe_serialize, dataframe_deserialize, validate_destination,
    Header, ParagonPackage, ParagonDataFrame, EncryptionType, MAGIC, HEADER_SIZE
};

fn main() {
    println!("=== Paragon Proto Serialization Example ===");
    println!("Protocol Version: {}", proto_version());

    // 1. Header parsing example
    let mut header_buf = [0u8; HEADER_SIZE];
    header_buf[0..4].copy_from_slice(&MAGIC.to_le_bytes());
    header_buf[4..6].copy_from_slice(&proto_version().to_le_bytes());
    header_buf[6..8].copy_from_slice(&0u16.to_le_bytes()); // flags
    header_buf[8..12].copy_from_slice(&256u32.to_le_bytes()); // payload_len

    let mut header = Header {
        magic: 0,
        version: 0,
        flags: 0,
        payload_len: 0,
    };

    let res = unsafe { proto_parse_header(header_buf.as_ptr(), header_buf.len(), &mut header) };
    if res == 0 {
        println!("Header parsed successfully!");
        println!("  Magic: 0x{:08X}", header.magic);
        println!("  Version: {}", header.version);
        println!("  Payload Len: {}", header.payload_len);
    } else {
        println!("Failed to parse header: {}", res);
    }

    // 2. Destination validation example
    let valid_dest = b"service.node.v1";
    let invalid_dest = b"service..node";
    println!("Validating destination '{:?}': {}", core::str::from_utf8(valid_dest).unwrap(), validate_destination(valid_dest));
    println!("Validating destination '{:?}': {}", core::str::from_utf8(invalid_dest).unwrap(), validate_destination(invalid_dest));

    // 3. Package serialization/deserialization example
    let pkg = ParagonPackage {
        version: [1, 0, 0, 0],
        mime_type: heapless::Vec::from_slice(b"application/json").unwrap(),
        raw_data: heapless::Vec::from_slice(b"{\"message\": \"hello paragon\"}").unwrap(),
        sign: heapless::Vec::from_slice(b"fake_signature_bytes_64_bytes_long_placeholder_12345678").unwrap(),
    };

    let mut pkg_buf = [0u8; 1024];
    let mut pkg_out_len = 0;
    let ser_res = package_serialize(&pkg, pkg_buf.as_mut_ptr(), pkg_buf.len(), &mut pkg_out_len);
    if ser_res == 0 {
        println!("Package serialized successfully, bytes written: {}", pkg_out_len);
    } else {
        println!("Package serialization failed: {}", ser_res);
    }

    let mut deserialized_pkg = ParagonPackage {
        version: [0; 4],
        mime_type: heapless::Vec::new(),
        raw_data: heapless::Vec::new(),
        sign: heapless::Vec::new(),
    };
    let des_res = package_deserialize(pkg_buf.as_ptr(), pkg_out_len, &mut deserialized_pkg);
    if des_res == 0 {
        println!("Package deserialized successfully!");
        println!("  Mime type: {:?}", core::str::from_utf8(&deserialized_pkg.mime_type));
        println!("  Raw data: {:?}", core::str::from_utf8(&deserialized_pkg.raw_data));
    } else {
        println!("Package deserialization failed: {}", des_res);
    }

    // 4. DataFrame serialization/deserialization example
    let df = ParagonDataFrame {
        packages: heapless::Vec::new(),
        class: heapless::Vec::from_slice(b"TelemetryClass").unwrap(),
        destination: heapless::Vec::from_slice(b"collector.metrics").unwrap(),
        enc: EncryptionType::X25519ChaCha20Poly1305,
    };

    let mut df_buf = [0u8; 512];
    let mut df_out_len = 0;
    let df_ser_res = dataframe_serialize(&df, df_buf.as_mut_ptr(), df_buf.len(), &mut df_out_len);
    if df_ser_res == 0 {
        println!("DataFrame serialized successfully, bytes written: {}", df_out_len);
    } else {
        println!("DataFrame serialization failed: {}", df_ser_res);
    }

    let mut des_df = ParagonDataFrame {
        packages: heapless::Vec::new(),
        class: heapless::Vec::new(),
        destination: heapless::Vec::new(),
        enc: EncryptionType::None,
    };
    let df_des_res = dataframe_deserialize(df_buf.as_ptr(), df_out_len, &mut des_df);
    if df_des_res == 0 {
        println!("DataFrame deserialized successfully!");
        println!("  Class: {:?}", core::str::from_utf8(&des_df.class));
        println!("  Destination: {:?}", core::str::from_utf8(&des_df.destination));
    } else {
        println!("DataFrame deserialization failed: {}", df_des_res);
    }
}
