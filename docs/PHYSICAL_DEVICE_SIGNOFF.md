# P4-M — Physical Windows Device Signoff

P4-M turns Byte's remaining real-device release checks into a repeatable local certification process.

It does **not** replace P4 automated CI certification. It adds the evidence that GitHub-hosted runners cannot provide:

- first-run usability
- companion feel over normal desktop use
- real fullscreen/presentation suppression
- lock/display-off/sleep recovery
- multi-monitor hot-unplug and mixed DPI
- Windows notification permission behavior
- human visual/accessibility review
- representative-device CPU/RAM/GPU/disk/network measurements
- signed installer / SmartScreen / trust UX

## Evidence model

P4-M produces a local:

`device-certification-vX.Y.Z.json`

By default the file is written to:

`Documents\Byte\device-certification-vX.Y.Z.json`

The report contains non-secret device information such as OS/build, computer manufacturer/model, CPU/GPU names, memory size, display count, candidate hashes, manual pass/fail results, and performance measurements.

Do not publish the report blindly. Review it first if you intend to attach it to an issue, release process, or archive.

## 1. Start from the exact source

Run P4-M from the same Byte source revision as the candidate.

For an ordinary exact-`main` candidate, use the commit that produced the candidate artifact.

For a future tagged release, check out the matching release source before verification.

The existing release verifier reads the local Tauri metadata, so mixing a candidate from one version with source from another version intentionally fails.

## 2. Choose the candidate

### Functional/device QA

The ordinary **Byte Packaging Certification** workflow produces:

`byte-release-candidate-<commit>`

That candidate is suitable for:

- first-run UX
- companion/lifecycle checks
- display/DPI checks
- notification checks
- accessibility/visual QA
- representative-device performance

Normal `main` candidates may be unsigned, so they cannot complete signed trust/SmartScreen certification.

### Full public-release signoff

Before creating a public tag, manually run the GitHub Actions workflow:

**Byte Device Signoff Candidate**

This workflow:

- can run only against `main`
- requires production Authenticode secrets
- signs and timestamps `Byte.exe` and the NSIS installer
- runs release artifact verification
- runs portable launch certification
- runs signed installer lifecycle certification
- runs P4 packaged real-world runtime certification
- produces release-certification schema v3
- creates GitHub provenance attestations
- uploads a 14-day artifact named:
  `byte-device-signoff-candidate-<commit>`
- does **not** publish a GitHub Release

Use this signed candidate for the final installer/publisher/SmartScreen/trust pass.

## 3. Extract the candidate

Extract the GitHub Actions artifact to a local directory containing:

- `Byte-vX.Y.Z-windows-x64-setup.exe`
- `Byte-vX.Y.Z-windows-x64-portable.zip`
- `release-manifest.json`
- `product-certification.json`
- `release-certification.json`
- `SHA256SUMS.txt`

P4-M verifies the candidate before asking for manual signoff.

## 4. Run the physical signoff

From the matching Byte repository checkout:

```powershell
pwsh -NoProfile -File .\scripts\run-physical-device-signoff.ps1 `
  -CandidateDir "C:\Path\To\Candidate"
```

The harness uses the certified portable `Byte.exe` for performance measurement by default.

If the exact candidate is installed and you want the performance test to use that installed executable instead:

```powershell
pwsh -NoProfile -File .\scripts\run-physical-device-signoff.ps1 `
  -CandidateDir "C:\Path\To\Candidate" `
  -Executable "C:\Path\To\Installed\Byte.exe"
```

The installed executable must be byte-for-byte identical to the candidate's portable `Byte.exe` or the harness stops.

For the final signed candidate:

```powershell
pwsh -NoProfile -File .\scripts\run-physical-device-signoff.ps1 `
  -CandidateDir "C:\Path\To\SignedCandidate" `
  -RequireSigning
```

## Manual groups

The harness records explicit PASS / FAIL results for:

1. First-run and everyday flow
2. Companion behavior
3. Fullscreen / presentation / lock / sleep lifecycle
4. Multi-monitor and DPI
5. Accessibility and visual QA
6. Windows notifications
7. Installer / update / trust UX
8. Performance observation

A group result applies to every listed item shown by the script. Do not mark the group PASS unless every item in the group has actually been checked.

### Capability-aware NA

P4-M only permits NA in two places:

- multi-monitor/DPI when fewer than two active displays are detected
- installer/trust when the candidate is unsigned

Those gaps are preserved in the report.

They do not prevent the machine from becoming `device_ready` for ordinary product QA, but they prevent `public_release_ready`.

A public-release-ready report therefore requires:

- signed and timestamped candidate
- same signer for application and installer
- installer/trust PASS
- at least two displays and multi-monitor/DPI PASS
- no recorded capability gaps

## 5. Performance measurement

The tool measures the **full Byte process tree**, including WebView2 child processes.

Each state runs for at least 10 minutes:

### ACTIVE_CALM

Use normal Habitat mode and leave Byte/system interaction calm.

### FULLSCREEN_REDUCED

Open a real fullscreen game, fullscreen video, or presentation that causes Byte's fullscreen/presentation suppression path.

### LOCKED_DISPLAY_OFF

After the tool gives the prompt, lock Windows or turn the display off and leave the machine untouched.

The PowerShell sampler continues while the session is locked.

## Measured data

For each state P4-M records:

- aggregate process-tree CPU
- aggregate working set
- aggregate private memory
- handle count
- thread count
- maximum process-tree size
- process-tree GPU engine utilization when Windows exposes the counter
- process-tree disk write rate when Windows exposes the counter
- non-loopback established TCP connections owned by Byte/WebView2 when Windows exposes ownership data

## Physical performance gates

The current release policy requires:

| Metric | Gate |
| --- | ---: |
| Average CPU | <= 0.25% |
| Average working set | <= 60 MB |
| Average GPU, when available | <= 1.0% |
| Average disk writes, when available | <= 4096 B/s |
| Non-loopback established TCP connections, when available | 0 |

The CPU and memory targets come directly from Byte's performance contract.

GPU/disk/network counters can be unavailable on some Windows configurations. P4-M records that as a capability gap instead of inventing a value.

Such a report may remain useful for product QA, but it is not public-release-ready.

## 6. Verify the report

After a successful run:

```powershell
pwsh -NoProfile -File .\scripts\verify-physical-device-signoff.ps1 `
  -CandidateDir "C:\Path\To\Candidate" `
  -Report "$HOME\Documents\Byte\device-certification-vX.Y.Z.json"
```

For final public-release evidence:

```powershell
pwsh -NoProfile -File .\scripts\verify-physical-device-signoff.ps1 `
  -CandidateDir "C:\Path\To\SignedCandidate" `
  -Report "$HOME\Documents\Byte\device-certification-vX.Y.Z.json" `
  -RequireSigning `
  -RequirePublicReleaseReady
```

The verifier rejects:

- wrong version
- wrong source/candidate hashes
- modified release/product certification evidence
- dry-run reports
- missing required manual PASS groups
- impossible multi-monitor/trust claims
- performance runs shorter than 10 minutes
- CPU/RAM/GPU/disk/network threshold failures
- public-release readiness with unsigned artifacts
- public-release readiness with fewer than two displays
- public-release readiness with any capability gap

## Development dry-run

To test the interactive harness without spending the full certification duration:

```powershell
pwsh -NoProfile -File .\scripts\run-physical-device-signoff.ps1 `
  -CandidateDir "C:\Path\To\Candidate" `
  -PerformanceMinutes 1 `
  -DevelopmentDryRun
```

A dry-run report is intentionally non-certifying and the strict verifier rejects it.

## Release approval rule

For a Stable public release, use P4-M as follows:

1. exact `main` CI is green
2. exact `main` Packaging Certification/P4 is green
3. create the signed **Byte Device Signoff Candidate**
4. perform P4-M on a physical Windows 11 machine
5. verify with `-RequireSigning -RequirePublicReleaseReady`
6. only then create the public release tag
7. tagged release workflow independently rebuilds and re-runs signing, packaging, runtime, provenance, and public-download verification

P4-M is therefore a pre-tag human/device approval layer, while the tagged workflow remains the final reproducible release-engineering gate.
