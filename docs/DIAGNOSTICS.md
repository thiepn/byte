# Byte Human-Friendly System Intelligence

Phase 6 converts factual telemetry into calm, understandable system states. The diagnostic layer is intentionally separate from telemetry collection.

## Product states

- CALM: no meaningful pressure.
- BUSY: legitimate workload is visible, but no issue is open.
- STRESSED: one or more sustained HIGH issues are active.
- NEEDS_ATTENTION: at least one CRITICAL issue is active.

A high CPU reading is not automatically a problem. CPU first becomes BUSY and must remain extremely high before Byte opens an issue.

## Conservative v1 rules

These thresholds are implementation defaults, not user-facing settings.

### CPU
- Busy/elevated: 70%+
- High candidate: 95%+ for 60 seconds
- Critical candidate: 99%+ for 5 minutes
- Recovery: 90% or below for 15 seconds

### Memory
Uses both used percentage and available physical memory.
- Elevated: 88% used, or 80%+ with 4 GB or less available
- High: 96% used, or 90%+ with 2 GB or less available, sustained for 15 seconds
- Critical: 99% used, or 96%+ with 1 GB or less available, sustained for 30 seconds
- Recovery: 88% or below, or 2.5 GB+ available, for 15 seconds

### Storage
- Elevated: 30 GB or less free, or 90%+ used
- High: 15 GB or less free, or 95%+ used
- Critical: 5 GB or less free, or 98%+ used
- Storage is slow-moving, so valid low-space conditions activate immediately.

### Battery
Only applies while not charging.
- Elevated: 20% or less
- High: 10% or less
- Critical: 5% or less
- Charging immediately resolves the low-battery condition; otherwise 12%+ clears it.

### Thermal
Temperature is best-effort and may come from different hardware sensors, so thresholds are deliberately conservative.
- Elevated: 90°C+
- High: 100°C+ for 60 seconds
- Critical: 110°C+ for 30 seconds
- Recovery: 95°C or below for 30 seconds

## Hysteresis and recovery

Each issue has separate activation and recovery timing. Once an issue opens, crossing just below its opening threshold does not instantly close it. This prevents Byte from repeatedly changing expression and status when a value sits near a boundary.

Resolved HIGH issues have a short reopen cooldown. CRITICAL conditions can bypass that cooldown.

## Process attribution

Process scanning is lazy.

Byte does not continuously scan every process. It begins refreshing process CPU/memory only when CPU or memory is already in a high candidate state or an issue is active, and throttles those scans to at most once every 3 seconds.

Related processes are grouped by process name. CPU usage is normalized by logical CPU count. Byte only names a culprit when one process group accounts for a meaningful share of measured process usage.

Attribution confidence:
- HIGH: one process clearly dominates.
- MEDIUM: one process is a meaningful contributor.
- LOW: no culprit is shown.

This is intentionally conservative: Byte prefers saying no single app stands out over blaming the wrong application.

## Issue priority

Severity always outranks category.

For equal severity, v1 priority is:
1. Thermal
2. Memory / Battery
3. Storage
4. CPU

Only one primary issue is shown in compact surfaces. Additional active issues are represented by secondary_issue_count.

## Recommendations

Phase 6 attaches narrow, safe recommendations:
- CPU / memory: Open Task Manager
- Storage: Open Storage Settings
- Battery: Open Battery Settings
- Thermal: View details

The diagnostic engine does not kill processes, delete files, change power settings, or alter the system automatically.

## Confidence

Issue confidence and culprit confidence are separate.

CPU, memory, storage, and battery conditions are based on direct measurements and normally carry HIGH issue confidence. Thermal issues use MEDIUM confidence because Windows hardware temperature availability and sensor identity vary by machine.
