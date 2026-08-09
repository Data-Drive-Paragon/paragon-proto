# paragon-proto

Binary protocol for structured data with encryption support for every Paragon products.

## Bun Usage

Build native library:
```bash
bun run build
```

Builder pattern API:
```typescript
import { createPackage, createFrame, createPreStructuredPackage, parseFrame, FieldType, EncryptionType } from "./src/bun/index";

const package = createPackage({
    mimeType: "application/json",
    rawData: new TextEncoder().encode('{"message": "hello"}')
}).build();

const psp = createPreStructuredPackage()
    .addField({
        fieldType: FieldType.String,
        name: "username",
        description: "User's login name",
        value: "john_doe"
    })
    .addField({
        fieldType: FieldType.Int,
        name: "age",
        description: "User's age in years",
        value: "30"
    })
    .build();

const frame = createFrame()
    .setClass("UserMessage")
    .setDestination("node.service")
    .setEncryption(EncryptionType.X25519ChaCha20Poly1305)
    .addPackage(package) // ONLY this
    .addPackage(psp) // or this. not both
    .build();

const parsed = parseFrame(frame);
```

## Rust Usage

Build with std feature:
```bash
cargo build --release --features std
```

FFI exports available for native integration.
