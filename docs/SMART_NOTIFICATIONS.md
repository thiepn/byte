# Byte Smart Notifications — Phase 21

Phase 21 makes Byte's native Windows notifications selective enough to remain useful over long-term daily use.

The design goal is **essentially zero annoying alerts during normal use**.

## Eligible conditions

Byte can send a Smart Notification for only five classes of condition.

### Critical memory

Requires the sustained DiagnosticEngine memory issue to reach CRITICAL.

A merely tight/high-memory condition remains visible inside Byte without creating an OS notification.

Cooldown: **4 hours**.

### Serious thermal pressure

Requires a sustained HIGH or CRITICAL thermal issue.

Thermal gets the highest simultaneous-alert priority because hardware heat can merit timely attention even before the diagnostic reaches its final critical tier.

Cooldown: **30 minutes**.

### Critically low storage

Requires the storage issue to reach CRITICAL.

Low-but-not-critical storage remains an in-app diagnostic only.

Cooldown: **24 hours**.

### Critically low battery

Requires CRITICAL battery state. The existing diagnostic engine already suppresses battery pressure while charging.

Cooldown: **1 hour**.

### Prolonged runaway process

This is intentionally much stricter than ordinary high CPU.

It requires all of the following:

- CPU diagnostic severity is CRITICAL
- critical/sustained condition has existed for roughly **10 minutes**
- Byte has a named culprit process
- culprit confidence is MEDIUM or HIGH
- Runaway-process notifications are enabled

Cooldown: **4 hours**.

Byte does not alert merely because the CPU is busy.

## App-hang detection

The original roadmap made app-hang detection optional.

Phase 21 does not implement it.

Byte does not currently have a reliable privacy-preserving signal that separates a genuinely hung GUI application from a deliberately waiting/background application with sufficiently low false-positive risk. Guessing would conflict with the notification design goal.

## All-active-issue routing

The compact UI still has one primary issue and a secondary count.

Smart Notifications are not limited to that primary issue. DiagnosticEngine retains its complete current active-issue list internally, and the notification router evaluates every active sustained condition.

This prevents a critical condition from being missed simply because another category has higher UI priority.

## Simultaneous incidents

Byte sends at most **one** new OS notification when several conditions become eligible together.

Selection priority:

1. thermal
2. memory
3. battery
4. storage
5. runaway process

Lower-priority simultaneous candidates are treated as part of the same incident and remain visible inside Byte rather than producing a burst of OS notifications.

## Duplicate suppression

A notification fingerprint combines:

- notification category
- diagnostic issue ID
- sustained issue start timestamp

Once Byte successfully notifies for that active fingerprint, it does not repeat the notification while the same incident remains active.

Recovery removes the active fingerprint.

## Persisted cooldowns

Successful category sends are written locally to:

`notifications.json`

Only category → last-send timestamp is persisted.

This means restarting Byte does not reset cooldowns or create easy duplicate alerts.

Malformed notification state safely recovers to a fresh local state file.

## Quiet mode

Notification Quiet mode suppresses all Byte OS health notifications indefinitely until switched off.

Quiet mode does **not**:

- disable monitoring
- suppress in-app diagnostics
- change companion system-health reactions
- delete notification preferences

## Snooze

Settings provides:

- 1 hour
- 4 hours
- until 08:00 tomorrow
- Resume now

Snooze suppresses OS notifications only.

The backend rejects snooze timestamps more than seven days in the future so a malformed IPC payload cannot silently create an accidental indefinite snooze. Indefinite suppression belongs to Quiet mode.

## Per-category controls

Settings exposes independent switches for:

- Critical memory
- Serious thermal pressure
- Critically low storage
- Critically low battery
- Prolonged runaway app

The master Windows Notifications switch remains above them.

Turning a category off does not disable its underlying diagnostic.

## Windows permission

Settings shows Byte's current OS notification permission as:

- Allowed by Windows
- Blocked by Windows
- Permission not decided

When permission is not granted, the user can explicitly request it.

Byte does not claim a notification was delivered if the OS permission blocks delivery.

The notification plugin capability remains scoped to Byte's trusted companion, Quick Panel, and main windows.

## Notification content

Notifications contain:

- a short Byte-specific title
- plain-language cause/context
- the existing recommended next-step label when available

Examples of next-step wording include:

- Open Task Manager
- Open Storage Settings
- Open Battery Settings
- View details

Phase 21 deliberately does not introduce arbitrary commands or destructive actions from notifications.

## Sound

Native notification sound follows Byte's existing **Sound cues** master switch.

With Sound cues off, Smart Notifications are sent silently.

Diagnostic meaning never depends on audio.

## Privacy

Smart Notifications introduce no new observation source.

They consume:

- existing sustained diagnostic issues
- existing confidence-aware process attribution
- local AppPreferences
- local cooldown timestamps

No notification data is uploaded.

## Phase boundary

Phase 21 does not add:

- notifications for normal workload
- network-activity alerts
- collection/unlock notifications
- daily summaries
- notification engagement analytics
- app-hang guesses
- process killing
- auto-remediation

Phase 22 continues with **Fullscreen, Gaming & Presentation Awareness**, focusing on lifecycle/overlay hardening beyond the basic fullscreen hide/restore primitive already introduced in Phase 20.
