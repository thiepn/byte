# P7 — Release Completion & Post-Publish Certification

P7 closes the release after P6 has created the exact approved tag and the tag-driven **Byte Release** workflow has published the release.

P7 has two layers:

1. **workflow-side public certification** — proves the actual files downloaded from the GitHub Release are valid and usable;
2. **local release closure** — binds that public certification back to the P6 initiation/P5/P4-M evidence and emits the final release-completion receipt.

P7 never changes the release tag.

The local completion command never creates, edits, uploads, or deletes GitHub Release assets.

## Historical release note

`v0.1.0` was published before P4-M/P5/P6/P7 existed.

It does not contain the P7 `public-release-certification.json` asset and should be treated as a historical release under the older certification model.

P7 applies to the next prepared release and every release after it.

Do not republish or mutate `v0.1.0` merely to retrofit P7.

## P7 public release asset set

Before P7 public certification is uploaded, the release contains six base assets:

- `Byte-vX.Y.Z-windows-x64-setup.exe`
- `Byte-vX.Y.Z-windows-x64-portable.zip`
- `release-manifest.json`
- `product-certification.json`
- `release-certification.json`
- `SHA256SUMS.txt`

After P7 succeeds, the completed release contains exactly seven assets:

- the six base assets above
- `public-release-certification.json`

The public certification intentionally does not hash itself.

It hashes and identifies the six release artifacts that existed before it was created.

## Workflow-side certification

After the existing `Publish GitHub Release` step, **Byte Release** performs the P7 chain.

### 1. Fresh public download

The workflow downloads the six base assets from the GitHub Release into a new temporary directory.

It does not certify the local `release-artifacts/` build directory as the public release.

### 2. Release artifact verification

The fresh download must pass:

`verify-release-artifacts.ps1 -RequireCertification -RequireSigning`

This rechecks:

- SHA-256 checksums
- manifest identity
- x64 executable format
- product/release certification
- valid Authenticode signatures
- timestamps
- same signer on installer and portable executable
- `public_distribution_ready: true`

### 3. Public portable launch

The freshly downloaded portable ZIP is launched through the existing portable-launch certification.

The public ZIP itself—not the pre-upload copy—must start successfully.

### 4. Public installer lifecycle

The freshly downloaded installer is run through the signed installer lifecycle certification.

This proves the actual public installer can be executed, installed, reinstalled, and uninstalled through Byte's release test harness.

### 5. Public GitHub provenance

GitHub attestation verification is rerun against the freshly downloaded:

- installer
- portable ZIP

Both must verify against `thiepn/byte`.

### 6. Release-page metadata

P7 queries GitHub's release API and requires:

- tag exactly matches `v<Byte version>`
- release title exactly `Byte <version>`
- release is not a draft
- Stable version -> not prerelease
- Beta version -> prerelease
- Stable version -> repository latest release
- `published_at` exists
- release notes exactly match the changelog-derived notes after newline normalization

### 7. P6 tag identity

The remote release tag must remain a lightweight direct commit ref.

It must point exactly to the tagged `Byte Release` source commit.

An annotated replacement tag or moved tag fails P7.

### 8. Base public asset identity

The freshly downloaded directory must contain exactly the six expected base assets.

GitHub's release metadata must also contain those base assets.

On a rerun of the same release workflow, one older `public-release-certification.json` is tolerated remotely so the workflow can safely replace it without deleting or moving the release tag.

### 9. Public certification

P7 writes:

`public-release-certification.json`

Schema version: `1`.

It records:

- Byte version
- release tag
- exact source commit
- Stable/Beta channel
- release database ID and URL
- publication timestamp
- release-note SHA-256
- Byte Release workflow run ID
- workflow source SHA/tag
- lightweight tag identity
- all six base asset names, sizes, and SHA-256 hashes
- remote asset digest when GitHub exposes one
- release/product/checksum hashes
- Authenticode signer thumbprints
- every public certification check

The file records:

`public_distribution_ready: true`

only after every preceding P7 check passes.

## Final seven-asset verification

The workflow uploads `public-release-certification.json` to the existing GitHub Release with `--clobber`.

It then downloads the release again into another clean directory.

The final verifier requires exactly seven assets and rechecks:

- base release artifact certification
- public certification structure
- every recorded asset hash
- release metadata
- Stable/Beta state
- lightweight tag target
- GitHub provenance
- referenced Byte Release workflow identity

The workflow also retains the public certification as a workflow artifact for 30 days.

## Release workflow completion

The public certification is generated while the Byte Release workflow is still running, so it records the workflow as observed during certification.

A release is **not closed** merely because `public-release-certification.json` exists.

Final closure requires the Byte Release run itself to reach:

- `status: completed`
- `conclusion: success`

That is checked by the local P7 completion command.

## Local P7 completion

After Byte Release finishes successfully, check out the exact released source commit/tag.

The working tree must be clean.

Then run:

~~~powershell
npm run release:complete -- `
  -Initiation "$HOME\Documents\Byte\release-initiation-vX.Y.Z.json"
~~~

When P5/P6 defaults were used, P7 derives the candidate, approval, and physical-device report paths automatically.

### Exact source checkout

P7 does not require the release commit to still be current `main`.

That would make legitimate historical verification impossible once development continues.

Instead it requires local HEAD to equal the exact commit stored in the P6 initiation receipt.

For example:

~~~powershell
git checkout v0.2.0
npm run release:complete -- `
  -Initiation "$HOME\Documents\Byte\release-initiation-v0.2.0.json"
~~~

The command may therefore run from a detached release-tag checkout.

## Local final verification

P7 first re-verifies the P6 initiation receipt with:

- remote tag verification
- referenced Byte Release workflow
- workflow success

It then downloads the entire final GitHub Release again.

The public certification must be present.

`verify-public-release-certification.ps1` is run with:

- remote-state verification
- workflow-success requirement
- runtime recheck

This means the local closure again exercises the actual public:

- portable package
- installer

It also reruns signature, checksum, release metadata, tag, and provenance verification.

## Final release-completion receipt

After all public checks pass, P7 writes:

`Documents\Byte\release-completion-vX.Y.Z.json`

Schema version: `1`.

The final receipt binds:

- release version/tag
- exact release commit
- P6 initiation receipt SHA-256
- P5 approval receipt SHA-256
- P4-M physical-device report SHA-256
- public-release-certification SHA-256
- GitHub Release database ID/URL/publication time
- successful Byte Release workflow run ID/URL
- exact seven public asset names, sizes, and SHA-256 hashes
- completion check states

A completed receipt requires all of these to be true:

~~~text
p6_initiation_verified
release_workflow_success
final_public_download_complete
public_release_certification_verified
public_asset_hashes_verified
authenticode_verified
github_provenance_verified
public_portable_launch_rechecked
public_installer_lifecycle_rechecked
release_metadata_verified
stable_beta_metadata_verified
exact_tag_target_verified
~~~

It also records:

~~~text
tag_mutation_performed: false
release_mutation_performed: false
asset_upload_performed: false
complete: true
receipt_self_hashed: false
~~~

The receipt is hash-bound to every earlier local evidence object and to every final public release asset without pretending to hash itself.

## Independent completion verification

Run:

~~~powershell
npm run release:complete:verify -- `
  -PublishedDir "$HOME\Documents\Byte\ReleaseCompletion\public-vX.Y.Z" `
  -PublicCertification "$HOME\Documents\Byte\ReleaseCompletion\public-vX.Y.Z\public-release-certification.json" `
  -Initiation "$HOME\Documents\Byte\release-initiation-vX.Y.Z.json" `
  -CandidateDir "$HOME\Documents\Byte\ReleaseApproval\candidate-signed-<commit>" `
  -DeviceReport "$HOME\Documents\Byte\device-certification-vX.Y.Z.json" `
  -Approval "$HOME\Documents\Byte\release-approval-vX.Y.Z.json" `
  -Completion "$HOME\Documents\Byte\release-completion-vX.Y.Z.json" `
  -RequireRemoteState
~~~

The verifier rechecks P6, the public certification, all receipt hashes, all final downloaded assets, and current remote state.

## Failure and recovery rules

### Byte Release failed before publication

Do not create another tag.

Fix the cause and rerun the existing GitHub Actions workflow for the same tag when appropriate.

If source changes are required, prepare a new higher version through the normal patch-forward process.

### Release exists but P7 public certification failed

The release is incomplete.

Do not call it release-complete.

Fix the release workflow/tooling problem and rerun the **same tag's** Byte Release workflow only when the source does not need to change.

The workflow safely replaces base assets and `public-release-certification.json` with `--clobber`.

It never requires moving the tag.

### Public certification uploaded but Byte Release later failed

The release is still incomplete.

`release:complete` requires the referenced Byte Release run to finish successfully and therefore refuses closure.

### Public release assets changed after certification

P7 verification fails because:

- asset hashes,
- remote digests,
- release metadata,
- or the final seven-asset set

no longer match the public certification.

Treat the release as altered and investigate.

Do not regenerate a completion receipt without understanding the mutation.

### Tag changed

P7 fails immediately.

Release tags must never be moved after P6.

Use a higher patch version instead.

### Main moved after publication

This is allowed.

P7 closes the exact release commit from P6 and does not require it to remain `main`.

The local checkout must simply be that exact release commit.

## Complete release lifecycle

The production path is now:

~~~text
release:prepare
      ↓
reviewed version PR
      ↓
CI + Packaging + Maintenance
      ↓
signed device candidate
      ↓
P4-M physical certification
      ↓
P5 release approval
      ↓
P6 exact-commit controlled tag
      ↓
Byte Release workflow
      ↓
signed build + certification + attestations
      ↓
GitHub Release publication
      ↓
P7 fresh public download certification
      ↓
public-release-certification.json
      ↓
Byte Release workflow success
      ↓
P7 local final public recheck
      ↓
release-completion-vX.Y.Z.json
      ↓
RELEASE CLOSED
~~~

P7 is the terminal certification layer.

After a P7 completion receipt verifies, any future correction belongs in a new patch-forward release rather than mutation of the completed release.
