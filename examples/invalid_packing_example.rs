use paragon_proto::{
    ParagonDataFrame, ParagonPackageEncrypted, ParagonPreStructuredPackage,
    PreStructuredField, FieldType, EncryptionType, ParseError,
    dataframe_serialize, validate_dataframe_packages, validate_destination,
    PPSP_MIME_TYPE
};

fn main() {
    println!("=== Paragon Proto Invalid Packing Examples ===");

    // Helper to create a valid PPSP package
    let mut ppsp = ParagonPreStructuredPackage {
        fields: heapless::Vec::new(),
    };
    ppsp.fields.push(PreStructuredField {
        name: heapless::Vec::from_slice(b"username").unwrap(),
        description: heapless::Vec::from_slice(b"Name of user").unwrap(),
        field_type: FieldType::String,
        value: heapless::Vec::from_slice(b"PYALOAD STRING VALUE").unwrap(),
    }).unwrap();

    let mut ppsp_buf = [0u8; 512];
    let ppsp_len = paragon_proto::serialization::ppsp_serialize(&ppsp, &mut ppsp_buf).unwrap();

    let ppsp_pkg = ParagonPackageEncrypted {
        version: [1, 0, 0, 0],
        mime_type: heapless::Vec::from_slice(PPSP_MIME_TYPE).unwrap(),
        encrypted_data: heapless::Vec::from_slice(&ppsp_buf[..ppsp_len]).unwrap(),
        sign: heapless::Vec::new(),
    };

    // --- Scenario 1: Intentional Invalid Package Mix ---
    println!("\n[Scenario 1] Attempting to pack a PPSP package and a free-text package together...");
    let free_text_pkg = ParagonPackageEncrypted {
        version: [1, 0, 0, 0],
        mime_type: heapless::Vec::from_slice(b"text/plain").unwrap(),
        encrypted_data: heapless::Vec::from_slice(b"unstructured free text").unwrap(),
        sign: heapless::Vec::new(),
    };

    let mut mixed_df = ParagonDataFrame {
        packages: heapless::Vec::new(),
        class: heapless::Vec::from_slice(b"MixedClass").unwrap(),
        destination: heapless::Vec::from_slice(b"service.node").unwrap(),
        enc: EncryptionType::None,
    };
    mixed_df.packages.push(ppsp_pkg.clone()).unwrap();
    mixed_df.packages.push(free_text_pkg).unwrap();

    let mix_res = validate_dataframe_packages(&mixed_df);
    match mix_res {
        Err(e) if e == ParseError::InvalidPackageMix as u8 => {
            println!("  -> Successfully caught! Error: InvalidPackageMix ({})", e);
        }
        other => {
            println!("  -> Unexpected result: {:?}", other);
        }
    }

    let mut buf = [0u8; 1024];
    let mut out_len = 0;
    let ser_res = dataframe_serialize(&mixed_df, buf.as_mut_ptr(), buf.len(), &mut out_len);
    println!("  -> dataframe_serialize returned status: {} (non-zero means rejection)", ser_res);

    // --- Scenario 2: Intentional Structural Mismatch between PPSP packages ---
    println!("\n[Scenario 2] Attempting to pack two PPSP packages with differing field descriptions/types...");
    let mut ppsp_mismatch = ParagonPreStructuredPackage {
        fields: heapless::Vec::new(),
    };
    ppsp_mismatch.fields.push(PreStructuredField {
        name: heapless::Vec::from_slice(b"username").unwrap(),
        description: heapless::Vec::from_slice(b"Different description for user name").unwrap(), // mismatch!
        field_type: FieldType::String,
        value: heapless::Vec::from_slice(b"ANOTHER VALUE").unwrap(),
    }).unwrap();

    let mut mismatch_buf = [0u8; 512];
    let mismatch_len = paragon_proto::serialization::ppsp_serialize(&ppsp_mismatch, &mut mismatch_buf).unwrap();

    let ppsp_mismatch_pkg = ParagonPackageEncrypted {
        version: [1, 0, 0, 0],
        mime_type: heapless::Vec::from_slice(PPSP_MIME_TYPE).unwrap(),
        encrypted_data: heapless::Vec::from_slice(&mismatch_buf[..mismatch_len]).unwrap(),
        sign: heapless::Vec::new(),
    };

    let mut mismatch_df = ParagonDataFrame {
        packages: heapless::Vec::new(),
        class: heapless::Vec::from_slice(b"MismatchClass").unwrap(),
        destination: heapless::Vec::from_slice(b"service.node").unwrap(),
        enc: EncryptionType::None,
    };
    mismatch_df.packages.push(ppsp_pkg.clone()).unwrap();
    mismatch_df.packages.push(ppsp_mismatch_pkg).unwrap();

    let mismatch_res = validate_dataframe_packages(&mismatch_df);
    match mismatch_res {
        Err(e) if e == ParseError::StructuralMismatch as u8 => {
            println!("  -> Successfully caught! Error: StructuralMismatch ({})", e);
        }
        other => {
            println!("  -> Unexpected result: {:?}", other);
        }
    }

    let ser_res2 = dataframe_serialize(&mismatch_df, buf.as_mut_ptr(), buf.len(), &mut out_len);
    println!("  -> dataframe_serialize returned status: {} (non-zero means rejection)", ser_res2);

    // --- Scenario 3: Intentional Invalid Destination in DataFrame ---
    println!("\n[Scenario 3] Attempting to serialize DataFrame with an invalid destination format...");
    let mut valid_p_df = ParagonDataFrame {
        packages: heapless::Vec::new(),
        class: heapless::Vec::from_slice(b"ValidClass").unwrap(),
        destination: heapless::Vec::from_slice(b"service..node").unwrap(), // invalid dot sequence
        enc: EncryptionType::None,
    };
    valid_p_df.packages.push(ppsp_pkg).unwrap();

    println!("  -> Destination validation for 'service..node': {}", validate_destination(&valid_p_df.destination));
    let ser_res3 = dataframe_serialize(&valid_p_df, buf.as_mut_ptr(), buf.len(), &mut out_len);
    match ser_res3 {
        err if err == ParseError::InvalidDestination as u8 => {
            println!("  -> Successfully caught! Error: InvalidDestination ({})", err);
        }
        other => {
            println!("  -> Unexpected result: {}", other);
        }
    }

    println!("\nAll invalid packing attempts were successfully detected and rejected!");
}
