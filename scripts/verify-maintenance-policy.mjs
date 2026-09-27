import fs from "node:fs";

const policy = JSON.parse(fs.readFileSync("maintenance/release-policy.json", "utf8"));
const tauri = JSON.parse(fs.readFileSync("src-tauri/tauri.conf.json", "utf8"));
const changelog = fs.readFileSync("CHANGELOG.md", "utf8");
const configSource = fs.readFileSync("src-tauri/src/core/config.rs", "utf8");
const releaseWorkflow = fs.readFileSync(".github/workflows/release.yml", "utf8");
const packageWorkflow = fs.readFileSync(".github/workflows/package.yml", "utf8");
const ciWorkflow = fs.readFileSync(".github/workflows/ci.yml", "utf8");
const deviceCandidateWorkflow = fs.readFileSync(".github/workflows/device-signoff-candidate.yml", "utf8");


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

const productCertification = policy.product_certification;
const expectedModes = ["HABITAT", "PERCH", "MINI", "EDGE", "TRAY"];
const expectedScales = [100, 110, 125, 150, 175, 200];
if (!productCertification ||
    productCertification.automated_runtime_gate !== true ||
    productCertification.fresh_first_run !== true ||
    productCertification.onboarding_boundary !== true ||
    productCertification.runtime_schema_migration !== true ||
    productCertification.corrupt_config_recovery !== true ||
    productCertification.manual_device_signoff_required !== true ||
    productCertification.sustained_runtime_seconds < 30 ||
    JSON.stringify(productCertification.display_modes) !== JSON.stringify(expectedModes) ||
    JSON.stringify(productCertification.interface_scales) !== JSON.stringify(expectedScales)) {
  throw new Error("Real-world product certification policy is incomplete or has been weakened.");
}
if (!Array.isArray(productCertification.manual_device_areas) ||
    productCertification.manual_device_areas.length < 8) {
  throw new Error("Real-world product certification must retain the manual device signoff matrix.");
}

const physicalSignoff = productCertification.physical_device_signoff;
const expectedPhysicalStates = ["ACTIVE_CALM", "FULLSCREEN_REDUCED", "LOCKED_DISPLAY_OFF"];
const expectedPhysicalThresholds = {
  cpu_average_percent_max: 0.25,
  working_set_average_mb_max: 60,
  gpu_average_percent_max: 1.0,
  disk_write_average_bytes_per_sec_max: 4096,
  remote_tcp_connections_max: 0,
};
if (!physicalSignoff ||
    physicalSignoff.tooling_required !== true ||
    physicalSignoff.runner_script !== "scripts/run-physical-device-signoff.ps1" ||
    physicalSignoff.verifier_script !== "scripts/verify-physical-device-signoff.ps1" ||
    physicalSignoff.minimum_performance_minutes_per_state < 10 ||
    JSON.stringify(physicalSignoff.performance_states) !== JSON.stringify(expectedPhysicalStates) ||
    JSON.stringify(physicalSignoff.thresholds) !== JSON.stringify(expectedPhysicalThresholds) ||
    physicalSignoff.public_release_requires_signed_candidate !== true ||
    physicalSignoff.public_release_requires_dual_display_coverage !== true ||
    physicalSignoff.public_release_allows_capability_gaps !== false ||
    physicalSignoff.signed_candidate_workflow !== ".github/workflows/device-signoff-candidate.yml" ||
    physicalSignoff.signed_candidate_retention_days !== 14 ||
    physicalSignoff.signed_candidate_main_only !== true ||
    physicalSignoff.candidate_bound_verifier_harness !== "scripts/test-physical-device-signoff-verifier.ps1") {
  throw new Error("Physical Windows device signoff policy is incomplete or has been weakened.");
}
for (const script of [
  physicalSignoff.runner_script,
  physicalSignoff.verifier_script,
  "scripts/test-physical-device-signoff-tooling.ps1",
  physicalSignoff.candidate_bound_verifier_harness,
]) {
  if (!fs.existsSync(script)) {
    throw new Error("Physical Windows device signoff tooling is missing: " + script);
  }
}
if (!ciWorkflow.includes("test-physical-device-signoff-tooling.ps1") ||
    !packageWorkflow.includes("test-physical-device-signoff-tooling.ps1") ||
    !releaseWorkflow.includes("test-physical-device-signoff-tooling.ps1") ||
    !deviceCandidateWorkflow.includes("test-physical-device-signoff-tooling.ps1")) {
  throw new Error("CI, packaging, tagged release, and signed-device-candidate workflows must validate the physical-device signoff tooling.");
}
for (const [name, workflow] of [
  ["package", packageWorkflow],
  ["release", releaseWorkflow],
  ["signed-device-candidate", deviceCandidateWorkflow],
]) {
  if (!workflow.includes("test-physical-device-signoff-verifier.ps1")) {
    throw new Error(name + " workflow is missing the candidate-bound physical device verifier harness.");
  }
}

const requiredDeviceCandidateFragments = [
  "workflow_dispatch",
  "if: github.ref == 'refs/heads/main'",
  "prepare-windows-signing.ps1 -RequireSigning",
  "build-windows-release.ps1 -RequireSigning",
  "stage-release.ps1 -RequireSigning",
  "test-windows-installer.ps1 -Installer $installer -RequireSigning",
  "finalize-release-certification.ps1 -RequireSigning",
  "verify-release-artifacts.ps1 -RequireCertification -RequireSigning",
  "byte-device-signoff-candidate-${{ github.sha }}",
  "cleanup-windows-signing.ps1",
];
for (const fragment of requiredDeviceCandidateFragments) {
  if (!deviceCandidateWorkflow.includes(fragment)) {
    throw new Error("Signed device candidate workflow is missing required gate: " + fragment);
  }
}
if (deviceCandidateWorkflow.includes("gh release create") ||
    deviceCandidateWorkflow.includes("gh release upload") ||
    deviceCandidateWorkflow.includes("Publish GitHub Release")) {
  throw new Error("Signed device candidate workflow must never publish a GitHub Release.");
}

const releaseApproval = productCertification.release_approval;
if (!releaseApproval ||
    releaseApproval.tooling_required !== true ||
    releaseApproval.orchestrator_script !== "scripts/run-release-approval.ps1" ||
    releaseApproval.verifier_script !== "scripts/verify-release-approval.ps1" ||
    releaseApproval.verifier_harness !== "scripts/test-release-approval-verifier.ps1" ||
    releaseApproval.receipt_schema_version !== 1 ||
    releaseApproval.source_branch !== "main" ||
    releaseApproval.exact_remote_main_required !== true ||
    releaseApproval.clean_working_tree_required !== true ||
    releaseApproval.signed_candidate_required_for_tagging !== true ||
    releaseApproval.public_device_report_required_for_tagging !== true ||
    releaseApproval.github_attestation_verification_required_for_tagging !== true ||
    releaseApproval.automatic_tagging !== false ||
    releaseApproval.automatic_release_publish !== false) {
  throw new Error("Release approval policy is incomplete or has been weakened.");
}
for (const script of [
  releaseApproval.orchestrator_script,
  releaseApproval.verifier_script,
  releaseApproval.verifier_harness,
  "scripts/test-release-approval-tooling.ps1",
]) {
  if (!fs.existsSync(script)) {
    throw new Error("Release approval tooling is missing: " + script);
  }
}
for (const [name, workflow] of [
  ["CI", ciWorkflow],
  ["package", packageWorkflow],
  ["release", releaseWorkflow],
  ["signed-device-candidate", deviceCandidateWorkflow],
]) {
  if (!workflow.includes("test-release-approval-tooling.ps1")) {
    throw new Error(name + " workflow is missing the P5 release approval tooling harness.");
  }
}
for (const [name, workflow] of [
  ["package", packageWorkflow],
  ["release", releaseWorkflow],
  ["signed-device-candidate", deviceCandidateWorkflow],
]) {
  if (!workflow.includes("test-release-approval-verifier.ps1")) {
    throw new Error(name + " workflow is missing the candidate-bound release approval verifier harness.");
  }
}
const approvalRunner = fs.readFileSync(releaseApproval.orchestrator_script, "utf8");
if (!approvalRunner.includes("gh attestation verify") ||
    !approvalRunner.includes("github_attestation_verified")) {
  throw new Error("Release approval orchestrator must verify GitHub provenance for signed candidates.");
}
for (const forbidden of [
  "git tag",
  "git push --tags",
  "gh release create",
  "gh release upload",
  "Publish GitHub Release",
]) {
  if (approvalRunner.includes(forbidden)) {
    throw new Error("Release approval orchestrator must not auto-tag or auto-publish: " + forbidden);
  }
}

const releaseInitiation = productCertification.release_initiation;
if (!releaseInitiation ||
    releaseInitiation.tooling_required !== true ||
    releaseInitiation.orchestrator_script !== "scripts/run-release-initiation.ps1" ||
    releaseInitiation.verifier_script !== "scripts/verify-release-initiation.ps1" ||
    releaseInitiation.tooling_harness !== "scripts/test-release-initiation-tooling.ps1" ||
    releaseInitiation.receipt_schema_version !== 1 ||
    releaseInitiation.source_branch !== "main" ||
    releaseInitiation.exact_remote_main_required !== true ||
    releaseInitiation.clean_working_tree_required !== true ||
    releaseInitiation.pre_tag_main_reverification_required !== true ||
    releaseInitiation.p5_tagging_approval_required !== true ||
    releaseInitiation.current_main_ci_required !== true ||
    releaseInitiation.current_main_packaging_required !== true ||
    releaseInitiation.release_order_required !== true ||
    releaseInitiation.duplicate_tag_or_release_forbidden !== true ||
    releaseInitiation.explicit_tag_confirmation_required !== true ||
    releaseInitiation.tag_type !== "lightweight" ||
    releaseInitiation.exact_commit_tag_required !== true ||
    releaseInitiation.force_tag_update_allowed !== false ||
    releaseInitiation.resume_existing_tag_exact_commit_only !== true ||
    releaseInitiation.direct_release_publish_allowed !== false ||
    releaseInitiation.direct_asset_upload_allowed !== false ||
    releaseInitiation.release_workflow_file !== ".github/workflows/release.yml" ||
    releaseInitiation.release_workflow_name !== "Byte Release" ||
    releaseInitiation.release_workflow_detection_required !== true ||
    releaseInitiation.github_attestation_reverification_required !== true) {
  throw new Error("Controlled release initiation policy is incomplete or has been weakened.");
}

for (const script of [
  releaseInitiation.orchestrator_script,
  releaseInitiation.verifier_script,
  releaseInitiation.tooling_harness,
]) {
  if (!fs.existsSync(script)) {
    throw new Error("P6 controlled release initiation tooling is missing: " + script);
  }
}

for (const [name, workflow] of [
  ["CI", ciWorkflow],
  ["package", packageWorkflow],
  ["release", releaseWorkflow],
  ["signed-device-candidate", deviceCandidateWorkflow],
]) {
  if (!workflow.includes("test-release-initiation-tooling.ps1")) {
    throw new Error(name + " workflow is missing the P6 controlled-tagging tooling harness.");
  }
}

const initiationRunner = fs.readFileSync(releaseInitiation.orchestrator_script, "utf8");
for (const required of [
  'verify-release-approval.ps1',
  'RequireTaggingApproval',
  'Get-SuccessfulWorkflowRun "ci.yml"',
  'Get-SuccessfulWorkflowRun "package.yml"',
  'check-release-order.mjs',
  'git push origin',
  ':refs/tags/',
  'ConfirmTag',
  'ResumeExistingTag',
  'Wait-ForReleaseRun',
  'release.yml',
  'gh attestation verify',
  'github_attestation_reverified',
]) {
  if (!initiationRunner.includes(required)) {
    throw new Error("P6 release initiation runner is missing required gate: " + required);
  }
}
for (const forbidden of [
  "--force",
  "git push --force",
  "git tag -d",
  "git push origin --delete",
  "gh release create",
  "gh release upload",
]) {
  if (initiationRunner.includes(forbidden)) {
    throw new Error("P6 release initiation runner contains forbidden mutation: " + forbidden);
  }
}

const initiationVerifier = fs.readFileSync(releaseInitiation.verifier_script, "utf8");
if (!initiationVerifier.includes("gh attestation verify") ||
    !initiationVerifier.includes("github_attestation_reverified")) {
  throw new Error("P6 verifier must independently re-check candidate GitHub provenance.");
}
if (!releaseWorkflow.includes('tags:') || !releaseWorkflow.includes('- "v*"')) {
  throw new Error("Byte Release must remain tag-driven for P6 release initiation.");
}

const releaseCompletion = productCertification.release_completion;
if (!releaseCompletion ||
    releaseCompletion.tooling_required !== true ||
    releaseCompletion.public_certification_generator !== "scripts/finalize-public-release-certification.ps1" ||
    releaseCompletion.public_certification_verifier !== "scripts/verify-public-release-certification.ps1" ||
    releaseCompletion.completion_orchestrator !== "scripts/run-release-completion.ps1" ||
    releaseCompletion.completion_verifier !== "scripts/verify-release-completion.ps1" ||
    releaseCompletion.tooling_harness !== "scripts/test-release-completion-tooling.ps1" ||
    releaseCompletion.candidate_bound_public_verifier_harness !== "scripts/test-public-release-certification-verifier.ps1" ||
    releaseCompletion.public_certification_schema_version !== 1 ||
    releaseCompletion.completion_receipt_schema_version !== 1 ||
    releaseCompletion.public_certification_asset !== "public-release-certification.json" ||
    releaseCompletion.final_public_asset_count !== 7 ||
    releaseCompletion.fresh_public_download_required !== true ||
    releaseCompletion.public_authenticode_required !== true ||
    releaseCompletion.public_timestamp_required !== true ||
    releaseCompletion.public_same_signer_required !== true ||
    releaseCompletion.public_github_provenance_required !== true ||
    releaseCompletion.public_portable_launch_required !== true ||
    releaseCompletion.public_installer_lifecycle_required !== true ||
    releaseCompletion.release_notes_match_required !== true ||
    releaseCompletion.stable_beta_metadata_required !== true ||
    releaseCompletion.exact_lightweight_tag_required !== true ||
    releaseCompletion.release_workflow_success_required_for_completion !== true ||
    releaseCompletion.p6_initiation_binding_required !== true ||
    releaseCompletion.local_completion_tag_mutation_allowed !== false ||
    releaseCompletion.local_completion_release_mutation_allowed !== false ||
    releaseCompletion.local_completion_asset_upload_allowed !== false) {
  throw new Error("P7 release completion policy is incomplete or has been weakened.");
}

for (const script of [
  releaseCompletion.public_certification_generator,
  releaseCompletion.public_certification_verifier,
  releaseCompletion.completion_orchestrator,
  releaseCompletion.completion_verifier,
  releaseCompletion.tooling_harness,
  releaseCompletion.candidate_bound_public_verifier_harness,
]) {
  if (!fs.existsSync(script)) {
    throw new Error("P7 release completion tooling is missing: " + script);
  }
}

for (const [name, workflow] of [
  ["CI", ciWorkflow],
  ["package", packageWorkflow],
  ["release", releaseWorkflow],
  ["signed-device-candidate", deviceCandidateWorkflow],
]) {
  if (!workflow.includes("test-release-completion-tooling.ps1")) {
    throw new Error(name + " workflow is missing the P7 release-completion tooling harness.");
  }
}

for (const [name, workflow] of [
  ["package", packageWorkflow],
  ["release", releaseWorkflow],
  ["signed-device-candidate", deviceCandidateWorkflow],
]) {
  if (!workflow.includes("test-public-release-certification-verifier.ps1")) {
    throw new Error(name + " workflow is missing the P7 candidate-bound public verifier harness.");
  }
}
if (!releaseWorkflow.includes("test-public-release-certification-verifier.ps1 -CandidateDir release-artifacts -RequireSigning") ||
    !deviceCandidateWorkflow.includes("test-public-release-certification-verifier.ps1 -CandidateDir release-artifacts -RequireSigning")) {
  throw new Error("Signed P7 candidate paths must exercise the positive public verifier harness.");
}
if (!packageWorkflow.includes("test-public-release-certification-verifier.ps1 -CandidateDir release-artifacts") ||
    packageWorkflow.includes("test-public-release-certification-verifier.ps1 -CandidateDir release-artifacts -RequireSigning")) {
  throw new Error("Ordinary packaging must exercise P7's unsigned public-release rejection path.");
}

const requiredP7ReleaseFragments = [
  "Certify freshly published public release",
  "finalize-public-release-certification.ps1",
  "Publish public release certification",
  "public-release-certification.json",
  "Verify completed public release",
  "verify-public-release-certification.ps1",
  "Retain public release certification",
];
for (const fragment of requiredP7ReleaseFragments) {
  if (!releaseWorkflow.includes(fragment)) {
    throw new Error("Byte Release workflow is missing required P7 post-publish gate: " + fragment);
  }
}

const publishIndex = releaseWorkflow.indexOf("Publish GitHub Release");
const certifyIndex = releaseWorkflow.indexOf("Certify freshly published public release");
const publicCertUploadIndex = releaseWorkflow.indexOf("Publish public release certification");
const finalVerifyIndex = releaseWorkflow.indexOf("Verify completed public release");
const retainPublicCertIndex = releaseWorkflow.indexOf("Retain public release certification");
if (publishIndex < 0 ||
    certifyIndex <= publishIndex ||
    publicCertUploadIndex <= certifyIndex ||
    finalVerifyIndex <= publicCertUploadIndex ||
    retainPublicCertIndex <= finalVerifyIndex) {
  throw new Error("P7 post-publish certification gates are missing or out of order.");
}

const completionRunner = fs.readFileSync(releaseCompletion.completion_orchestrator, "utf8");
for (const forbidden of [
  "gh release create",
  "gh release upload",
  "gh release edit",
  "gh release delete",
  "git push",
  "git tag",
]) {
  if (completionRunner.includes(forbidden)) {
    throw new Error("P7 local completion runner contains forbidden remote mutation: " + forbidden);
  }
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
  "Certify freshly published public release",
  "Verify completed public release",
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

for (const [name, workflow] of [
  ["release", releaseWorkflow],
  ["package", packageWorkflow],
]) {
  if (!workflow.includes("test-real-world-product.ps1")) {
    throw new Error(name + " workflow is missing the P4 real-world product runtime gate.");
  }
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
    ", signed public Windows releases required, real-world product gate enabled, physical-device signoff tooling locked, release approval receipt policy locked, controlled tag initiation locked, post-publish release completion locked.",
);
