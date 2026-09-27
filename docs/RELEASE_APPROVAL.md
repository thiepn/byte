# P5 — One-Command Release Approval

P5 turns Byte's pre-tag approval process into one guided local command while preserving the P4/P4-M evidence boundary.

The command can:

- confirm the local checkout is the exact current `main`
- require a clean working tree
- authenticate with GitHub CLI
- create or reuse the exact signed device-signoff candidate
- watch the candidate workflow to completion
- download the exact workflow artifact
- verify all release/P4 certification evidence
- run the P4-M physical Windows signoff
- verify the resulting device report
- generate a candidate-bound release approval receipt
- verify that receipt immediately

P5 deliberately does **not**:

- create a Git tag
- push a tag
- create a GitHub Release
- upload release assets
- bypass physical signoff
- turn failed or missing device evidence into approval

The final tagging decision remains separate and explicit.

## Requirements

Run from a Windows checkout of `thiepn/byte`.

You need:

- PowerShell 7 (`pwsh`)
- Git
- GitHub CLI (`gh`)
- an authenticated GitHub CLI session with access to the repository
- a clean local `main` branch exactly matching remote `main`

For signed public-release approval, the repository must also have valid:

- `WINDOWS_CERTIFICATE`
- `WINDOWS_CERTIFICATE_PASSWORD`
- `WINDOWS_TIMESTAMP_URL`

The signed candidate workflow uses those secrets; they are never copied to the local machine by P5.

## Public-release approval

From current `main`:

```powershell
npm run release:approve -- -CandidateType signed
```

This performs the following sequence.

### 1. Exact-main preflight

P5 compares:

- local Git branch
- local HEAD
- remote GitHub `main`
- working-tree cleanliness

It refuses to proceed if the checkout is not exact current `main`.

### 2. Signed candidate

Unless `-ReuseExistingCandidate` is supplied, P5 dispatches:

**Byte Device Signoff Candidate**

on `main`.

It then identifies the newly dispatched run for the exact `main` SHA and watches it with `gh run watch --exit-status`.

If signing fails, the process stops before physical signoff.

### 3. Exact artifact download

P5 downloads:

`byte-device-signoff-candidate-<main-sha>`

to:

`Documents\Byte\ReleaseApproval\candidate-signed-<main-sha>`

by default.

It then runs normal certified artifact verification with required signing.

For signed candidates, P5 also runs GitHub artifact-attestation verification locally for both:

- the NSIS installer
- the portable ZIP

A signed candidate cannot reach tagging approval unless both provenance checks succeed.

### 4. Physical signoff

P5 calls the existing P4-M runner.

The human/device requirements remain unchanged, including:

- first-run and everyday UX
- companion behavior
- fullscreen/presentation
- lock/display-off/sleep
- multi-monitor/DPI
- accessibility/visual QA
- Windows notifications
- installer/update/trust UX
- 10-minute ACTIVE/CALM performance
- 10-minute FULLSCREEN_REDUCED performance
- 10-minute LOCKED/DISPLAY-OFF performance

P5 does not answer these prompts automatically.

### 5. Device evidence verification

For a signed candidate, P5 requires the physical report to verify with:

- required signing
- public-release readiness

This means the P4-M report must have:

- valid signed/timestamped same-signer candidate
- installer/trust PASS
- dual-display multi-monitor/DPI PASS
- no capability gaps
- all required human groups PASS
- all required performance states and thresholds PASS

### 6. Approval receipt

After the physical report verifies, P5 writes:

`Documents\Byte\release-approval-vX.Y.Z.json`

The receipt contains:

- Byte version
- exact source commit
- exact local HEAD
- candidate type
- candidate workflow run ID/URL when P5 downloaded it
- installer SHA-256
- portable SHA-256
- release-manifest SHA-256
- product-certification SHA-256
- release-certification SHA-256
- whether GitHub candidate provenance was verified
- physical-device-report SHA-256
- device/public readiness state
- whether the evidence is sufficient for tagging

For a passing signed public candidate:

`approved_for_tagging: true`

The receipt also explicitly records:

- `automatic_tag_created: false`
- `automatic_release_published: false`

## Reusing an existing signed candidate

If a successful signed candidate already exists for exact current `main`:

```powershell
npm run release:approve -- -CandidateType signed -ReuseExistingCandidate
```

P5 finds the newest successful exact-SHA device-signoff workflow run and downloads its artifact.

If the artifact has expired or cannot be downloaded, P5 fails rather than silently substituting a different candidate.

## Unsigned device QA

For ordinary product/device QA using the normal exact-main Packaging Certification artifact:

```powershell
npm run release:approve -- -CandidateType unsigned -ReuseExistingCandidate
```

This uses:

`byte-release-candidate-<main-sha>`

It can generate a verified device-QA approval receipt, but it cannot produce public tagging approval because the candidate is unsigned.

## Existing candidate directory

To avoid GitHub candidate acquisition entirely:

```powershell
npm run release:approve -- \
  -CandidateType signed \
  -CandidateDir "C:\Path\To\Candidate"
```

The local candidate must still match exact current remote `main`.

## Verify-only mode

If physical signoff was already completed:

```powershell
npm run release:approve -- \
  -CandidateType signed \
  -CandidateDir "C:\Path\To\Candidate" \
  -Report "$HOME\Documents\Byte\device-certification-vX.Y.Z.json" \
  -VerifyOnly
```

`-VerifyOnly` is intentionally side-effect-free.

It will not dispatch a workflow. If no `-CandidateDir` is supplied, use `-ReuseExistingCandidate` explicitly.

## Verify a receipt independently

```powershell
npm run release:approve:verify -- \
  -CandidateDir "C:\Path\To\SignedCandidate" \
  -DeviceReport "$HOME\Documents\Byte\device-certification-vX.Y.Z.json" \
  -Approval "$HOME\Documents\Byte\release-approval-vX.Y.Z.json" \
  -RequireTaggingApproval
```

The verifier rejects:

- wrong Byte version
- wrong repository/branch
- wrong source commit
- wrong local-head claim
- modified installer/portable hashes
- modified P4/release-certification hashes
- modified physical-report hash
- inconsistent device/public-ready state
- invalid physical-device evidence
- tagging approval without signed/timestamped same-signer artifacts
- tagging approval without verified GitHub provenance for the downloaded signed candidate
- tagging approval without a public-release-ready physical report
- any claim that P5 automatically created a tag
- any claim that P5 automatically published a release

## Failure modes

### Signing secrets missing

The signed candidate workflow fails and P5 stops.

Configure the production signing secrets before retrying.

### Local main is stale

P5 refuses to continue.

Update the checkout so local `main` equals remote `main`.

### Working tree is dirty

P5 refuses to continue.

Commit or stash local work first.

### Candidate workflow succeeds but artifact is missing/expired

P5 stops during artifact download.

Run a fresh signed device-signoff candidate.

### Physical signoff fails

No public tagging approval is emitted.

Fix/retest Byte and repeat the signoff.

### Receipt verification fails

Treat the approval as invalid.

Do not tag based on a receipt that does not independently verify.

## Release boundary

P5 ends at:

**verified evidence + `approved_for_tagging: true`**

It intentionally does not cross into release publication.

After reviewing the receipt, creating the actual release tag remains an explicit action. The tag then triggers the normal Byte Release workflow, which independently rebuilds, signs, certifies, attests, publishes, and re-download-verifies the public assets.
