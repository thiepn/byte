# Byte Apps & Diagnostics — Phase 17

Phase 17 turns the Apps destination into a focused diagnostic investigation surface without turning Byte into Task Manager.

## Product question

The page should answer:

> Is one application meaningfully contributing to what my computer is doing right now?

It should not answer every process-management question Windows already handles better.

## On-demand inspection

Byte does not keep an Apps leaderboard running in the background.

A process scan occurs only when:

- the user opens Apps for the first time in the current main-window session
- the user presses Refresh scan

The full application's ordinary two-second snapshot refresh does not refresh process diagnostics.

Rapid repeat requests within 750 ms reuse the last point-in-time result.

## First CPU sample

Per-process CPU requires two refresh observations to become useful.

On the first deliberate Apps inspection, Byte:

1. refreshes process CPU and memory
2. waits 250 ms
3. refreshes once more
4. builds the result

The delay exists only for that explicit first scan. It does not create a worker or recurring timer.

## Aggregation

Related processes are grouped by friendly process name.

For each group Byte exposes:

- name
- number of processes in the group
- normalized CPU percent
- memory usage in MB
- share of observed process CPU
- share of observed process memory

Only the 12 most relevant groups are returned. Relevance is the greater of CPU share and memory share.

The UI can re-sort those returned rows by relevance, CPU, or memory without performing another scan.

## Attribution

Context rows and attribution are deliberately different concepts.

An app can appear in the table because it is using resources without Byte claiming that the app is responsible for a system problem.

CPU stands out only when:

- normalized CPU is at least 15%, and
- observed CPU share is at least 25% for MEDIUM confidence or 50% for HIGH confidence

Memory stands out only when:

- memory is at least 256 MB, and
- observed memory share is at least 18% for MEDIUM confidence or 35% for HIGH confidence

These match the diagnostic engine's existing culprit thresholds.

If no app satisfies those rules, the UI says no app clearly stands out.

## Diagnostic context

The page continues to show the current authoritative diagnostic issue, if any.

This keeps two different facts distinct:

- current system diagnosis comes from the sustained DiagnosticEngine
- Apps inspection is a point-in-time process snapshot

An on-demand Apps scan cannot open, close, or reprioritize a diagnostic issue.

## Privacy

The Apps IPC payload contains no:

- command-line arguments
- executable paths
- file paths
- window titles
- open files
- typed text
- mouse coordinates
- application usage history
- uploaded process names

Everything remains local.

## Process management boundary

Byte intentionally provides no:

- End task
- Kill process
- Suspend
- Change priority
- Startup disable
- service management

The only management handoff is the existing fixed **Open Task Manager** action.

## UI

The production Apps surface includes:

- an explicit On demand only privacy note
- current diagnostic context
- CPU standout card
- memory standout card
- scan freshness
- relevance / CPU / memory sorting
- aggregated app table
- per-row Context only or standout label
- Task Manager handoff
- explicit explanation for why Byte has no End task button

Next: **Phase 18 — Customization Studio**.
