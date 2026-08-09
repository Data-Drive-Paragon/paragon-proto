import { dlopen, FFIType, suffix } from "bun:ffi";
import { resolve } from "path";

const libPath = resolve(import.meta.dir, `./native/libparagon_proto.${suffix}`);

export const { symbols } = dlopen(libPath, {
    proto_version: { args: [], returns: FFIType.u16 },
    proto_parse_header: { args: [FFIType.ptr, FFIType.u64, FFIType.ptr], returns: FFIType.u8 },
    package_serialize: { args: [FFIType.ptr, FFIType.ptr, FFIType.u64, FFIType.ptr], returns: FFIType.u8 },
    package_deserialize: { args: [FFIType.ptr, FFIType.u64, FFIType.ptr], returns: FFIType.u8 },
    dataframe_serialize: { args: [FFIType.ptr, FFIType.ptr, FFIType.u64, FFIType.ptr], returns: FFIType.u8 },
    dataframe_deserialize: { args: [FFIType.ptr, FFIType.u64, FFIType.ptr], returns: FFIType.u8 },
    crypto_derive_shared_secret: { args: [FFIType.ptr, FFIType.ptr], returns: FFIType.ptr },
    crypto_encrypt_chacha20poly1305: { args: [FFIType.ptr, FFIType.ptr, FFIType.ptr, FFIType.u64, FFIType.ptr, FFIType.u64, FFIType.ptr], returns: FFIType.u8 },
    crypto_decrypt_chacha20poly1305: { args: [FFIType.ptr, FFIType.ptr, FFIType.ptr, FFIType.u64, FFIType.ptr, FFIType.ptr, FFIType.u64], returns: FFIType.u8 },
});

export function getVersion(): number { return symbols.proto_version(); }
