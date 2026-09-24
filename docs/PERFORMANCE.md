# Byte Power & Performance Contract

Phase 23 defines the runtime rule that Byte must not materially affect the system it monitors.

## Targets

These are release-certification targets, not claims inferred from unit tests:

- idle CPU: preferably below 0.25% on a representative Windows 11 machine
- resident memory: target below 60 MB in the normal companion configuration
- GPU: negligible while the companion is visually idle
- network: zero application-generated telemetry traffic
- disk writes: event/config persistence only; never continuous telemetry logging

Phase 23 establishes the architecture needed to meet those targets. Final measured certification belongs to the later release/audit phases.

## Work budget by state

| State | Telemetry | Global input | Companion animation | Desktop awareness |
| --- | --- | --- | --- | --- |
| ACTIVE / calm | 5 s | event-driven | 12 FPS paced | 500 ms |
| ACTIVE / busy | 2.5 s | event-driven | 12 FPS paced | 500 ms |
| ACTIVE / stressed | 1.5 s | event-driven | 12 FPS paced | 500 ms |
| FULLSCREEN_REDUCED | 8 s | suspended | suspended | 1 s |
| LOCKED | blocked | suspended | suspended | 2 s sentinel |
| DISPLAY_SLEEP | blocked | suspended | suspended | 2 s sentinel |
| SYSTEM_SLEEP | blocked | suspended | suspended | 2 s sentinel |
| monitoring disabled | 30 s dormant wait | unchanged by setting | unchanged by setting | state-dependent |

The 30-second monitoring-disabled wait is interrupted immediately when preferences or lifecycle state change.

## Rules

1. **One hardware sampler.** Frontend surfaces may only read the cached SystemSnapshot.
2. **No hidden render loop.** A companion hidden by desktop awareness, Tray mode, or the shell unsubscribes from the animation scheduler instead of waking and returning early.
3. **No duplicate telemetry poller.** Companion updates come from byte://snapshot-updated emitted by the authoritative Rust worker.
4. **No unnecessary global-input processing.** Hook callbacks exit before timestamp/channel work while Byte is intentionally suppressed; the interpreter blocks until a lifecycle control message arrives.
5. **Expensive process scans are exceptional.** Diagnostic attribution starts only for sustained CPU/memory issues and is rate-limited. Apps diagnostics remain user-triggered.
6. **Slow signals stay cached.** Storage, battery, and thermal refresh independently from the fast telemetry cadence.
7. **No telemetry database.** Session trend points remain bounded/in-memory; only meaningful Activity events are persisted.
8. **No network analytics.** Network telemetry reads local OS byte counters; Byte does not upload telemetry.
9. **No fake optimization controls.** Byte does not trim RAM, kill arbitrary processes, control fans, run synthetic benchmarks, or claim to speed up Windows.

## Resume correctness

Lock/display/system sleep are discontinuities, not long sample gaps. On resume Byte:

1. wakes through LifecycleCoordinator,
2. reconstructs the Windows telemetry source,
3. resets diagnostic sustained-condition state,
4. marks the previous snapshot unavailable,
5. takes a fresh sample,
6. emits the fresh snapshot to the companion.

This prevents pre-sleep EWMA values, network deltas, or issue timers from being treated as continuous post-resume evidence. A scheduling gap of 12 seconds or more is also treated as a resume boundary, covering OS sleep cases where no intermediate lifecycle observation could run while Windows itself had suspended the process.

## Verification

Automated tests cover lifecycle suspension predicates, adaptive telemetry cadence, fullscreen/sleep awareness cadence, input capture suspension, and companion visual suspension.

Release certification should additionally measure an optimized packaged build on Windows 11 over at least:

- 10 minutes ACTIVE/CALM
- 10 minutes FULLSCREEN_REDUCED
- 10 minutes display-off/locked
- repeated lock/unlock and display sleep/wake cycles
- a sustained CPU/memory pressure scenario that activates process attribution

Record CPU, private working set, GPU engine utilization, wakeups where available, disk writes, and outbound network activity. Treat the targets above as acceptance gates rather than optimizing against debug-build measurements.
