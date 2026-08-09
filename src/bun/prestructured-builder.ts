import { FieldType } from "./types";

export interface FieldConfig { fieldType: FieldType; name: string; description: string; value: string }

export class PreStructuredPackageBuilder {
    private fields: FieldConfig[] = [];

    addField(config: FieldConfig): this { this.fields.push(config); return this; }

    build(): Uint8Array {
        let totalSize = 1;
        for (const field of this.fields) {
            const nameBytes = new TextEncoder().encode(field.name);
            const descBytes = new TextEncoder().encode(field.description);
            const valueBytes = new TextEncoder().encode(field.value);
            totalSize += 1 + 1 + nameBytes.length + 1 + descBytes.length + 2 + valueBytes.length;
        }
        const buffer = new Uint8Array(totalSize);
        let offset = 0;
        buffer[offset++] = this.fields.length;
        for (const field of this.fields) {
            const nameBytes = new TextEncoder().encode(field.name);
            const descBytes = new TextEncoder().encode(field.description);
            const valueBytes = new TextEncoder().encode(field.value);
            buffer[offset++] = field.fieldType;
            buffer[offset++] = nameBytes.length;
            buffer.set(nameBytes, offset); offset += nameBytes.length;
            buffer[offset++] = descBytes.length;
            buffer.set(descBytes, offset); offset += descBytes.length;
            const valLen = valueBytes.length;
            buffer[offset++] = valLen & 0xFF; buffer[offset++] = (valLen >> 8) & 0xFF;
            buffer.set(valueBytes, offset); offset += valueBytes.length;
        }
        return buffer;
    }
}

export function createPreStructuredPackage(): PreStructuredPackageBuilder { return new PreStructuredPackageBuilder(); }
