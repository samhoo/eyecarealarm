// Single source of truth for the app version: src-tauri/tauri.conf.json.
// Syncs it into Cargo.toml and package.json. Runs before dev/build.

import { readFileSync, writeFileSync } from "node:fs";

const conf = JSON.parse(readFileSync("src-tauri/tauri.conf.json", "utf8"));
const v = conf.version;
if (!/^\d+\.\d+\.\d+$/.test(v)) {
  console.error(`tauri.conf.json version "${v}" is not semver x.y.z`);
  process.exit(1);
}

const cargoPath = "src-tauri/Cargo.toml";
let cargo = readFileSync(cargoPath, "utf8");
const next = cargo.replace(/(\[package\][\s\S]*?\nversion = ")[^"]*(")/, `$1${v}$2`);
if (next === cargo && !next.includes(`\nversion = "${v}"`)) {
  console.error("Cargo.toml [package] version line not found");
  process.exit(1);
}
writeFileSync(cargoPath, next);

const pkgPath = "package.json";
const pkg = JSON.parse(readFileSync(pkgPath, "utf8"));
pkg.version = v;
writeFileSync(pkgPath, JSON.stringify(pkg, null, 2) + "\n");

console.log(`synced version ${v} -> Cargo.toml, package.json`);
