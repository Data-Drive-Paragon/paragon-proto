#!/usr/bin/env bun
import { $ } from "bun";

const platform = process.platform;
const arch = process.arch;

console.log(`Building native library for ${platform}-${arch}...`);

// Build Rust library with std feature
await $`cargo build --release --features std`;

// Determine library name and extension
const libName = "libparagon_proto";
let libExt: string;
let targetDir: string;

switch (platform) {
  case "linux":
    libExt = "so";
    targetDir = "target/release";
    break;
  case "darwin":
    libExt = "dylib";
    targetDir = "target/release";
    break;
  case "win32":
    libExt = "dll";
    targetDir = "target/release";
    break;
  default:
    throw new Error(`Unsupported platform: ${platform}`);
}

const sourceLib = `${targetDir}/${libName}.${libExt}`;
const destDir = "src/bun/native";
const destLib = `${destDir}/${libName}.${libExt}`;

// Create destination directory
await $`mkdir -p ${destDir}`;

// Copy library to src/bun/native
await $`cp ${sourceLib} ${destLib}`;

console.log(`✅ Native library built and copied to ${destLib}`);
console.log(`📦 Ready for Bun FFI usage`);
