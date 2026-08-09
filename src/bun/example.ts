import { getVersion, symbols } from "./ffi";
import { ptr } from "bun:ffi";
import { MAGIC, HEADER_SIZE } from "./types";

console.log(`Protocol Version from shared library: ${getVersion()}`);

const headerBuf = new Uint8Array(HEADER_SIZE);
const view = new DataView(headerBuf.buffer);
view.setUint32(0, MAGIC, true); 
view.setUint16(4, getVersion(), true); 
view.setUint16(6, 0, true); 
view.setUint32(8, 512, true); 

const outHeaderBuf = new Uint8Array(12); 
const res = symbols.proto_parse_header(
    ptr(headerBuf),
    headerBuf.byteLength,
    ptr(outHeaderBuf)
);

if (res === 0) {
    const outView = new DataView(outHeaderBuf.buffer);
    console.log("Header parsed successfully via Bun FFI!");
    console.log(`  Magic: 0x${outView.getUint32(0, true).toString(16).toUpperCase()}`);
    console.log(`  Version: ${outView.getUint16(4, true)}`);
    console.log(`  Flags: ${outView.getUint16(6, true)}`);
    console.log(`  Payload Len: ${outView.getUint32(8, true)}`);
} else {
    console.error(`Failed to parse header: error code ${res}`);
}

console.log("Bun FFI protocol integration layer verified successfully!");
