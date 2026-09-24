# Byte Telemetry Engine V2

Phase 5 replaces the development snapshot with one production telemetry worker on Windows.

## Signals

- CPU: global usage from sysinfo.
- Memory: used, total, and available RAM from sysinfo. Swap is intentionally not sampled.
- Storage: total/free space for the volume containing Byte, falling back to the largest known disk.
- Network: aggregate receive/transmit deltas across known interfaces, normalized by actual elapsed time and exposed as Mbps.
- Battery: Windows GetSystemPowerStatus. Desktop/no-battery systems return no battery value.
- Thermal: best-effort sysinfo component temperatures. Missing thermal data is represented as unavailable, never fabricated.

## Cadence

One worker owns all sampling. Phase 23 makes its fast-signal cadence adaptive:

- NEEDS_ATTENTION / STRESSED: every 1.5 seconds.
- BUSY: every 2.5 seconds.
- CALM: every 5 seconds.
- FULLSCREEN_REDUCED: every 8 seconds.
- Monitoring disabled: a 30-second dormant wait that is interrupted immediately by preference/lifecycle changes.
- Battery: cached and refreshed no more often than every 5 seconds.
- Thermal: cached and refreshed no more often than every 10 seconds.
- Storage: cached and refreshed no more often than every 15 seconds.

The frontend still reads only the authoritative cache. The companion receives snapshots emitted by that worker rather than running a second 2-second polling loop.

CPU is primed using sysinfo's minimum update interval because CPU usage is delta-based.

## Smoothing

Volatile values use an EWMA with alpha 0.35:

- CPU
- memory percentage
- network throughput
- thermal value

The first real sample initializes the filter directly; it is not biased toward zero.

Battery and storage are not smoothed.

## Product boundary

Phase 5 reports facts only. It does not decide whether high CPU or memory means a problem.

The returned snapshot remains CALM/NORMAL unless data is unavailable, because semantic health classification belongs to Phase 6. This avoids mixing hardware collection with diagnosis.

## Lifecycle

The worker is owned by AppState and is joined during explicit shutdown.

Lock, display-sleep, and system-sleep states block on a condition variable instead of waking on a polling timer. When lifecycle or monitoring preferences change, the same coordinator wakes the telemetry worker immediately.

After a suspended state resumes, Byte rebuilds the telemetry source and diagnostic engine before sampling. This prevents stale CPU/network deltas, EWMA state, or sustained-condition timers from crossing a sleep/lock boundary.

## Privacy and network behavior

Telemetry is local only. No telemetry is uploaded.

Byte's own network throughput measurement reads OS counters and does not create network traffic.
