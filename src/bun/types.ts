export enum FieldType { String = 1, Int = 2, Float = 3, Bool = 4, Bytes = 5 }
export enum EncryptionType { None = 0, X25519ChaCha20Poly1305 = 1, X25519Aes256Gcm = 2 }
export enum ParseError { Ok = 0, BadMagic = 1, BadVersion = 2, TooShort = 3, InvalidDestination = 4, InvalidPackageMix = 5, StructuralMismatch = 6, InvalidPpsp = 7 }

export interface Header { magic: number; version: number; flags: number; payloadLen: number }
export interface PreStructuredField { fieldType: FieldType; name: string; description: string; value: string }
export interface ParagonPreStructuredPackage { fields: PreStructuredField[] }

export const MAGIC = 0x47414E41;
export const VERSION = 1;
export const HEADER_SIZE = 12;
export const PPSP_MIME_TYPE = "application/vnd.paragon.ppsp";
