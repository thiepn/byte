import fs from "node:fs";

const policy = JSON.parse(fs.readFileSync("maintenance/release-policy.json", "utf8"));
const tauri = JSON.parse(fs.readFileSync("src-tauri/tauri.conf.json", "utf8"));
const changelog = fs.readFileSync("CHANGELOG.md", "utf8");
const configSource = fs.readFileSync("src-tauri/src/core/config.rs", "utf8");
const releaseWorkflow = fs.readFileSync(".github/workflows/release.yml", "utf8");
const packageWorkflow = fs.readFileSync(".github/workflows/package.yml", "utf8");


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

const distribution = policy.distribution;
if (!distribution ||
    distribution.public_release_requires_authenticode !== true ||
    distribution.installer_requires_authenticode !== true ||
    distribution.portable_binary_requires_authenticode !== true ||
    distribution.timestamp_required !== true ||
    distribution.same_signer_required !== true ||
    distribution.unsigned_release_candidates_allowed !== true) {
  throw new Error("Windows distribution trust policy is incomplete or has been weakened.");
}
if (tauri.bundle?.publisher !== distribution.publisher) {
  throw new Error("Tauri publisher metadata does not match the release trust policy.");
}
if (tauri.bundle?.homepage !== distribution.homepage) {
  throw new Error("Tauri homepage metadata does not match the release trust policy.");
}

const requiredReleaseTrustFragments = [
  "prepare-windows-signing.ps1 -RequireSigning",
  "build-windows-release.ps1 -RequireSigning",
  "stage-release.ps1 -RequireSigning",
  "verify-release-artifacts.ps1 -RequireCertification -RequireSigning",
  "test-windows-installer.ps1 -Installer $installer -RequireSigning",
  "Verify published release trust",
];
for (const fragment of requiredReleaseTrustFragments) {
  if (!releaseWorkflow.includes(fragment)) {
    throw new Error("Tagged release workflow is missing required trust gate: " + fragment);
  }
}
if (releaseWorkflow.includes("Prepare optional Windows code signing")) {
  throw new Error("Tagged public releases must not treat Authenticode signing as optional.");
}
if (packageWorkflow.includes("WINDOWS_CERTIFICATE")) {
  throw new Error("Ordinary PR/main packaging must not consume production code-signing secrets.");
}
if (!changelog.includes("## [Unreleased]")) {
  throw new Error("CHANGELOG.md must retain an [Unreleased] section.");
}

const escapeRegex = (value) => value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
const currentHeading = new RegExp(
  "^## \\[" + escapeRegex(tauri.version) + "\\](?: - \\d{4}-\\d{2}-\\d{2})?\\s*$",
  "m",
);
if (!currentHeading.test(changelog)) {
  throw new Error("CHANGELOG.md must contain release notes for current version " + tauri.version + ".");
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

console.log(
  "Maintenance policy verified: schema " + schemaMatch[1] +
    ", migration floor " + minMatch[1] +
    ", version " + tauri.version +
    ", signed public Windows releases required.",
);
