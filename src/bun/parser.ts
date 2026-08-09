import { EncryptionType } from "./types";

export function parseFrame(buffer: Uint8Array): { packages: Uint8Array[]; class: string; destination: string; encryption: EncryptionType } | null {
    let offset = 0;
    if (buffer.length < 1) return null;
    const packageCount = buffer[offset++];
    if (packageCount === undefined) return null;
    const packages: Uint8Array[] = [];
    for (let i = 0; i < packageCount; i++) {
        if (offset + 2 > buffer.length) return null;
        const pkgLen = buffer[offset]! | (buffer[offset + 1]! << 8);
        offset += 2;
        if (offset + pkgLen > buffer.length) return null;
        packages.push(buffer.slice(offset, offset + pkgLen));
        offset += pkgLen;
    }
    if (offset >= buffer.length) return null;
    const encryption = buffer[offset++] as EncryptionType;
    if (offset >= buffer.length) return null;
    const classLen = buffer[offset++];
    if (classLen === undefined || offset + classLen > buffer.length) return null;
    const className = new TextDecoder().decode(buffer.slice(offset, offset + classLen));
    offset += classLen;
    if (offset >= buffer.length) return null;
    const destLen = buffer[offset++];
    if (destLen === undefined || offset + destLen > buffer.length) return null;
    const destination = new TextDecoder().decode(buffer.slice(offset, offset + destLen));
    return { packages, class: className, destination, encryption };
}
