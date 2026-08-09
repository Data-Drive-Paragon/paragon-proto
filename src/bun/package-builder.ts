export interface PackageConfig { version?: [number, number, number, number]; mimeType?: string; rawData?: Uint8Array; sign?: Uint8Array }

export class PackageBuilder {
    private version: [number, number, number, number] = [1, 0, 0, 0];
    private mimeType: string = "application/octet-stream";
    private rawData: Uint8Array = new Uint8Array(0);
    private sign: Uint8Array = new Uint8Array(0);

    constructor(config?: PackageConfig) {
        if (config) {
            if (config.version) this.version = config.version;
            if (config.mimeType) this.mimeType = config.mimeType;
            if (config.rawData) this.rawData = config.rawData;
            if (config.sign) this.sign = config.sign;
        }
    }

    setVersion(version: [number, number, number, number]): this { this.version = version; return this; }
    setMimeType(mimeType: string): this { this.mimeType = mimeType; return this; }
    setRawData(data: Uint8Array): this { this.rawData = data; return this; }
    setSign(sign: Uint8Array): this { this.sign = sign; return this; }

    build(): Uint8Array {
        const mimeBytes = new TextEncoder().encode(this.mimeType);
        const totalSize = 4 + 1 + mimeBytes.length + 2 + this.rawData.length + 1 + this.sign.length;
        const buffer = new Uint8Array(totalSize);
        let offset = 0;
        buffer[offset++] = this.version[0]; buffer[offset++] = this.version[1]; buffer[offset++] = this.version[2]; buffer[offset++] = this.version[3];
        buffer[offset++] = mimeBytes.length;
        buffer.set(mimeBytes, offset); offset += mimeBytes.length;
        const dataLen = this.rawData.length;
        buffer[offset++] = dataLen & 0xFF; buffer[offset++] = (dataLen >> 8) & 0xFF;
        buffer.set(this.rawData, offset); offset += this.rawData.length;
        buffer[offset++] = this.sign.length;
        buffer.set(this.sign, offset);
        return buffer;
    }
}

export function createPackage(config?: PackageConfig): PackageBuilder { return new PackageBuilder(config); }
