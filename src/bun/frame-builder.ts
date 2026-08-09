import { EncryptionType } from "./types";

export interface FrameConfig { class?: string; destination?: string; encryption?: EncryptionType }

export class FrameBuilder {
    private packages: Uint8Array[] = [];
    private className: string = "";
    private destination: string = "";
    private encryption: EncryptionType = EncryptionType.None;

    constructor(config?: FrameConfig) {
        if (config) {
            if (config.class) this.className = config.class;
            if (config.destination) this.destination = config.destination;
            if (config.encryption) this.encryption = config.encryption;
        }
    }

    addPackage(packageData: Uint8Array): this { this.packages.push(packageData); return this; }
    setClass(className: string): this { this.className = className; return this; }
    setDestination(destination: string): this { this.destination = destination; return this; }
    setEncryption(encryption: EncryptionType): this { this.encryption = encryption; return this; }

    build(): Uint8Array {
        let totalSize = 1;
        for (const pkg of this.packages) totalSize += 2 + pkg.length;
        totalSize += 1 + 1 + this.className.length + 1 + this.destination.length;
        const buffer = new Uint8Array(totalSize);
        let offset = 0;
        buffer[offset++] = this.packages.length;
        for (const pkg of this.packages) {
            const pkgLen = pkg.length;
            buffer[offset++] = pkgLen & 0xFF; buffer[offset++] = (pkgLen >> 8) & 0xFF;
            buffer.set(pkg, offset); offset += pkg.length;
        }
        buffer[offset++] = this.encryption;
        const classBytes = new TextEncoder().encode(this.className);
        buffer[offset++] = classBytes.length;
        buffer.set(classBytes, offset); offset += classBytes.length;
        const destBytes = new TextEncoder().encode(this.destination);
        buffer[offset++] = destBytes.length;
        buffer.set(destBytes, offset);
        return buffer;
    }
}

export function createFrame(config?: FrameConfig): FrameBuilder { return new FrameBuilder(config); }
