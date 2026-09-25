import fs from "node:fs";

const policy = JSON.parse(fs.readFileSync("maintenance/release-policy.json", "utf8"));
const tauri = JSON.parse(fs.readFileSync("src-tauri/tauri.conf.json", "utf8"));
const changelog = fs.readFileSync("CHANGELOG.md", "utf8");
const configSource = fs.readFileSync("src-tauri/src/core/config.rs", "utf8");

if (policy.schema_version !== 1 || policy.product !== "Byte" || policy.repository !== "thiepn/byte") {
  throw new Error("Maintenance policy identity is invalid.");
}
if (policy.compatibility.target !== "x86_64-pc-windows-msvc") {
  throw new Error("Maintenance target must match the release target.");
}
if (policy.maintenance.hotfix_strategy !== "patch-forward") {
  throw new Error("Byte maintenance policy must remain patch-forward.");
}
if (policy.maintenance.forced_auto_update !== false) {
  throw new Error("Byte must not enable forced automatic updating.");
}
if (!changelog.includes("## [Unreleased]")) {
  throw new Error("CHANGELOG.md must retain an [Unreleased] section.");
}

const schemaMatch = configSource.match(/CURRENT_SCHEMA_VERSION:\s*u32\s*=\s*(\d+)/);
const minMatch = configSource.match(/MIN_MIGRATABLE_SCHEMA_VERSION:\s*u32\s*=\s*(\d+)/);
if (!schemaMatch || !minMatch) {
  throw new Error("Could not read configuration compatibility constants.");
}
if (Number(schemaMatch[1]) !== policy.configuration.current_schema) {
  throw new Error("Maintenance policy current_schema does not match Rust configuration.");
}
if (Number(minMatch[1]) !== policy.configuration.minimum_migratable_schema) {
  throw new Error("Maintenance policy minimum_migratable_schema does not match Rust configuration.");
}

const stable = new RegExp(policy.release_channels.stable.tag_pattern);
const beta = new RegExp(policy.release_channels.beta.tag_pattern);
const tag = "v" + tauri.version;
if (!stable.test(tag) && !beta.test(tag)) {
  throw new Error("Current Byte version " + tauri.version + " matches neither Stable nor Beta release policy.");
}

console.log("Maintenance policy verified: schema " + schemaMatch[1] + ", migration floor " + minMatch[1] + ", version " + tauri.version + ".");
