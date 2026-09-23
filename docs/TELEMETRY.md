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

One worker owns all sampling.

- CPU / memory / network: every 1.5 seconds.
- Battery: cached and refreshed every 5 seconds.
- Thermal: cached and refreshed every 10 seconds.
- Storage: cached and refreshed every 15 seconds.

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

Display/system sleep states block on a condition variable instead of waking on a polling timer. When lifecycle state changes, the same coordinator wakes the telemetry worker.

## Privacy and network behavior

Telemetry is local only. No telemetry is uploaded.

Byte's own network throughput measurement reads OS counters and does not create network traffic.
