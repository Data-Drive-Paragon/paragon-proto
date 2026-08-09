#!/usr/bin/env bun
import { $ } from "bun";

const platform = process.platform;
const arch = process.arch;

console.log(`Building native library for ${platform}-${arch}...`);

// Build Rust library with std feature
await $`cargo build --release --features std`;

// Determine library extension and possible names
let libExt: string;
const targetDir = "target/release";

switch (platform) {
  case "linux":
    libExt = "so";
    break;
  case "darwin":
    libExt = "dylib";
    break;
  case "win32":
    libExt = "dll";
    break;
  default:
    throw new Error(`Unsupported platform: ${platform}`);
}

const possibleNames = platform === "win32" ? ["paragon_proto", "libparagon_proto"] : ["libparagon_proto"];
let sourceLib = "";
let foundName = "";

for (const name of possibleNames) {
  const candidate = `${targetDir}/${name}.${libExt}`;
  if (await Bun.file(candidate).exists()) {
    sourceLib = candidate;
    foundName = name;
    break;
  }
}

if (!sourceLib) {
  foundName = possibleNames[0];
  sourceLib = `${targetDir}/${foundName}.${libExt}`;
}

const destDir = "src/bun/native";
// Always copy to libparagon_proto.<ext> to match FFI expectation
const destLib = `${destDir}/libparagon_proto.${libExt}`;

// Create destination directory
await $`mkdir -p ${destDir}`;

// Copy library to src/bun/native
await $`cp ${sourceLib} ${destLib}`;

console.log(`✅ Native library built and copied to ${destLib}`);
console.log(`📦 Ready for Bun FFI usage`);
