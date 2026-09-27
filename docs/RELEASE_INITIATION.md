# P6 — Controlled Tagging & Release Initiation

P6 is the boundary between a verified P5 approval receipt and the tag-driven public release workflow.

P5 proves:

- the exact candidate was signed and timestamped,
- GitHub provenance was verified,
- physical Windows signoff passed,
- the exact current `main` commit was approved for tagging.

P6 is responsible for exactly one additional irreversible action:

> create the approved version tag on the exact approved commit and confirm that the existing **Byte Release** workflow starts.

P6 does **not** build, sign, upload, or publish release assets itself.

## Important current-state note

Byte already has a published `v0.1.0` release.

P6 therefore intentionally refuses to tag the current `0.1.0` metadata again.

Use P6 after preparing and merging the next version, for example with:

```powershell
npm run release:prepare -- --version 0.2.0 --channel stable
```

The version-preparation changes must then pass the normal PR, P4/P4-M, and P5 approval path before P6 can create `v0.2.0`.

## One controlled action

The normal command is:

```powershell
npm run release:initiate -- \
  -Approval "$HOME\Documents\Byte\release-approval-vX.Y.Z.json"
```

When P5 used its default paths, P6 derives:

- the signed candidate directory from the approval commit,
- the physical-device report from the receipt,
- the initiation receipt output path.

Before changing Git state, P6 performs every preflight below.

## Preflight

### Exact repository state

P6 requires:

- repository origin is `thiepn/byte`,
- local branch is `main`,
- local working tree is clean,
- local HEAD equals current remote `main`,
- P5 approval commit equals current remote `main`.

A P5 approval becomes stale immediately if `main` moves.

### P5 evidence

P6 re-runs:

`verify-release-approval.ps1 -RequireTaggingApproval`

against:

- the signed candidate,
- the physical-device report,
- the P5 approval receipt.

The approval must still report:

- `approved_for_tagging: true`,
- valid signed/timestamped same-signer candidate,
- public-release-ready physical signoff,
- verified GitHub candidate provenance.

P6 then independently runs `gh attestation verify` again against both the signed installer and portable ZIP. Tagging does not rely only on the older P5 receipt's provenance flag.

### Version and release notes

P6 requires:

- all Byte version files agree,
- tag is exactly `v<current version>`,
- version satisfies Stable/Beta release policy,
- concrete changelog notes exist for that exact version.

### Exact-main automation

P6 requires successful **push** runs for the exact approved commit from:

- Byte CI
- Byte Packaging Certification

A successful PR run is not enough.

### Release ordering

The proposed tag is compared with every relevant published Byte release using the existing SemVer ordering policy.

P6 refuses non-monotonic releases.

For Stable releases, prereleases are ignored in the same way as the tagged release workflow.

For Beta releases, published Stable and Beta versions participate in ordering.

### Tag/release uniqueness

Before creating a new tag, P6 requires:

- no matching remote tag,
- no matching local tag,
- no GitHub Release or draft with the same tag.

P6 never overwrites an existing release identity.

## Dry run

Run every preflight without creating a tag:

```powershell
npm run release:initiate -- \
  -Approval "$HOME\Documents\Byte\release-approval-vX.Y.Z.json" \
  -DryRun
```

A dry run performs no Git mutation and does not start Byte Release.

Use this immediately before the real initiation if you want an additional operator check.

## Explicit confirmation

For a real initiation, P6 prints:

- version,
- tag,
- exact commit,
- approval receipt SHA-256,
- exact-main CI run,
- exact-main Packaging run.

It then requires the exact version tag to be typed, for example:

`v0.2.0`

Any mismatch aborts before tagging.

For an intentionally non-interactive shell, pass the exact tag explicitly:

```powershell
npm run release:initiate -- \
  -Approval "$HOME\Documents\Byte\release-approval-v0.2.0.json" \
  -ConfirmTag v0.2.0
```

There is no generic yes/force flag.

## Tag behavior

P6 creates one lightweight tag directly on the approved commit:

`<approved-main-sha>:refs/tags/vX.Y.Z`

This is intentional.

A lightweight exact-commit tag means the tag ref points directly to the same commit bound into the P5 evidence.

P6 never uses:

- `--force`,
- tag deletion,
- tag movement,
- replacement refs.

After pushing, P6 immediately queries the remote ref and requires it to point exactly to the approved commit.

## Release initiation

The repository's existing `Byte Release` workflow remains triggered by:

`push -> tags -> v*`

After tag creation, P6 waits for the corresponding workflow run to appear.

It binds the initiation receipt to:

- workflow run ID,
- workflow URL,
- event,
- source SHA,
- observed status/conclusion.

P6 only initiates the workflow.

The workflow itself remains responsible for:

- release metadata verification,
- dependency/security audits,
- Authenticode signing,
- installer and portable builds,
- install/upgrade/uninstall certification,
- P4 packaged runtime certification,
- release certification,
- provenance attestations,
- GitHub Release publication,
- fresh public-download verification.

## Initiation receipt

A successful P6 run writes:

`Documents\Byte\release-initiation-vX.Y.Z.json`

It binds together:

- exact source commit,
- P5 approval SHA-256,
- physical-device report SHA-256,
- signed candidate release-manifest SHA-256,
- signed candidate release-certification SHA-256,
- fresh P6 GitHub-attestation re-verification state,
- exact-main CI run,
- exact-main Packaging run,
- release-order result,
- tag name/ref/type/target,
- whether the tag was created or resumed,
- Byte Release workflow run ID and URL,
- workflow state observed at initiation.

It also records:

```text
forced_update: false
direct_release_publish_performed: false
direct_asset_upload_performed: false
```

## Independent verification

Verify the initiation receipt and remote state with:

```powershell
npm run release:initiate:verify -- \
  -CandidateDir "C:\Path\To\SignedCandidate" \
  -DeviceReport "$HOME\Documents\Byte\device-certification-vX.Y.Z.json" \
  -Approval "$HOME\Documents\Byte\release-approval-vX.Y.Z.json" \
  -Initiation "$HOME\Documents\Byte\release-initiation-vX.Y.Z.json" \
  -RequireRemoteState
```

The verifier independently requires:

- P5 tagging approval still verifies,
- receipt hashes match candidate/device/approval evidence,
- tag name equals app version,
- tag type is lightweight,
- tag target equals approved commit,
- exact-main CI and Packaging evidence is successful,
- no force update was recorded,
- no direct publishing/upload was performed,
- remote GitHub tag is a direct commit ref to the approved commit,
- GitHub provenance verifies again for the signed installer and portable ZIP,
- referenced workflow is **Byte Release**,
- workflow event is `push`,
- workflow source SHA equals the approved commit.

After the release workflow finishes, additionally require success:

```powershell
npm run release:initiate:verify -- \
  -CandidateDir "C:\Path\To\SignedCandidate" \
  -DeviceReport "$HOME\Documents\Byte\device-certification-vX.Y.Z.json" \
  -Approval "$HOME\Documents\Byte\release-approval-vX.Y.Z.json" \
  -Initiation "$HOME\Documents\Byte\release-initiation-vX.Y.Z.json" \
  -RequireRemoteState \
  -RequireWorkflowSuccess
```

## Recovery: tag exists but local command was interrupted

Tag creation is intentionally treated as irreversible release initiation.

P6 never deletes or force-moves a tag automatically.

If the tag was successfully created but the local process stopped before recording the workflow, run:

```powershell
npm run release:initiate -- \
  -Approval "$HOME\Documents\Byte\release-approval-vX.Y.Z.json" \
  -ResumeExistingTag
```

Resume is permitted only when the remote tag already points to the **exact approved commit**.

If it points anywhere else, P6 stops.

Resume does not recreate, move, or delete the tag.

## Failure after tag creation

If P6 verifies the tag but does not detect Byte Release within its observation window:

1. do not rerun normal tag creation,
2. do not delete or force-move the tag,
3. inspect GitHub Actions for the existing tag,
4. use `-ResumeExistingTag` after resolving the workflow issue.

The initiation receipt records the partial state before P6 exits.

## What P6 deliberately cannot guarantee

P6 verifies that the release workflow was initiated against the correct commit.

It does not declare the release successful merely because the workflow started.

Final public-release success remains the job of the existing Byte Release workflow and its post-publication trust checks.

## Release sequence

The intended Stable/Beta path is now:

```text
release:prepare
      ↓
reviewed version PR
      ↓
CI + Packaging + Maintenance
      ↓
signed device candidate
      ↓
P4-M physical Windows certification
      ↓
P5 approved_for_tagging receipt
      ↓
P6 controlled exact-commit tag
      ↓
Byte Release workflow
      ↓
signed/certified GitHub Release
      ↓
fresh public-download trust verification
```

P6 therefore makes tag creation a controlled, evidence-gated release operation rather than a manual `git tag && git push` step.
