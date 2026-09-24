# Security Policy

Byte is a local-first Windows desktop application.

## Reporting

For a security vulnerability, prefer GitHub's private vulnerability-reporting flow for this repository when it is available. Avoid posting exploit details, private user data, credentials, or weaponized proof-of-concept material in a public issue.

For ordinary bugs that do not expose a security boundary, use the normal issue tracker.

## Scope

High-value reports include issues involving:

- Tauri IPC capability bypass
- arbitrary command or URL execution
- unexpected filesystem access
- exposure of typed content or cursor/input history
- remote content obtaining native privileges
- unsafe handling of process names or persisted state
- privilege escalation from a lower-privilege Byte WebView
- secrets or personal data leaving the local machine unexpectedly

Byte does not treat already having arbitrary same-user code execution or administrator/kernel control as a Byte-specific security boundary.

See [docs/SECURITY_PRIVACY.md](docs/SECURITY_PRIVACY.md) for the implemented threat model.
