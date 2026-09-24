# Byte Global Input Reaction Engine

Phase 10 makes the desktop companion respond to real keyboard and mouse activity while preserving the privacy boundary established in Phase 4.

## Privacy contract

Byte never forwards, stores, logs, or serializes:

- key identity
- virtual-key codes
- scan codes
- typed characters
- focused application text
- clipboard contents
- mouse coordinates

The native public event contains only:

- semantic reaction kind
- event timestamp
- optional anonymous typing-rate estimate

The low-level keyboard callback does not dereference the keyboard hook structure at all.

## Windows hook architecture

Byte installs:

- WH_KEYBOARD_LL
- WH_MOUSE_LL

on one dedicated Windows message-loop thread.

The hook callbacks only classify the event at the broadest possible level and enqueue:

- keyboard activity
- left click
- right click
- scroll

They immediately call CallNextHookEx and do no animation, telemetry, file I/O, IPC, or diagnostic work.

A second worker thread consumes those anonymous events and performs all timing/debounce/typing-speed interpretation before emitting semantic Tauri events to the companion WebView.

This deliberately follows the Windows low-level hook requirement that callbacks remain fast and that heavier work be handed off.

## Keyboard reactions

Normal typing alternates semantic taps:

- TYPING_TAP_LEFT
- TYPING_TAP_RIGHT

The character manifest decides what these mean visually:

- Byte: tiny console/keyboard taps
- Mochi: alternating paws
- Pip: left/right squash
- Kiwi: alternating pecks

No key identity is required to create this effect.

## Fast typing

Byte maintains a short anonymous rolling window of key activity.

Current v1 thresholds:

- 750 ms observation window
- 6 key-down activities within that window enters fast typing
- 500 ms without another key leaves fast typing

Fast typing emits a semantic loop start/stop instead of restarting an animation for every key.

Mouse/scroll reactions do not interrupt an active fast-typing loop.

## Mouse reactions

The engine observes only:

- left button down
- right button down
- vertical/horizontal wheel activity

It intentionally ignores mouse movement.

Debounce:

- mouse button reaction: 45 ms
- scroll reaction: 75 ms

Debouncing affects character reactions only. Any received input still resets the idle timer.

## Idle detection

The input interpreter tracks the time of the most recent anonymous keyboard/click/scroll activity.

After 30 seconds with no such activity:

- IDLE_START is emitted
- the companion may enter its semantic sleep behavior

The first subsequent activity emits:

- IDLE_END
- then the actual input reaction

The animation layer releases the idle/sleep behavior and requests wake.

This is not a productivity tracker. Idle duration is neither persisted nor exposed as analytics.

## Runtime lifecycle

InputRuntime owns:

- the Windows hook message-loop thread
- the semantic processing worker
- the control channel
- the Win32 hook thread identifier

Explicit Byte shutdown:

1. sends processor shutdown
2. posts WM_QUIT to the known hook thread
3. unhooks keyboard and mouse hooks
4. joins both workers
5. stops telemetry
6. exits

If hook installation fails, Byte continues without global input reactions rather than failing the application.

## Event contract

Rust → companion event:

byte://input-reaction

Kinds:

- TYPING_TAP_LEFT
- TYPING_TAP_RIGHT
- TYPING_FAST_START
- TYPING_FAST_STOP
- MOUSE_LEFT
- MOUSE_RIGHT
- SCROLL
- IDLE_START
- IDLE_END

The Svelte companion converts those events into the semantic animation requests already defined in Phase 8.

## Priority behavior

Input activity uses the animation runtime's input priority.

That means:

- diagnostic/critical system states still outrank typing reactions
- normal idle behavior yields to typing/clicking
- direct companion interaction remains above global passive input
- fast typing stays active until its explicit stop event

## Performance

No mouse-move hook reactions are generated.

The hook callbacks perform only a small channel send and return immediately.

The processor is deadline-driven rather than polling at a fixed high frequency: it sleeps until either input arrives or the next typing-stop/idle deadline approaches.

Phase 23 also gates the hook callback before timestamp/channel work and blocks the processor completely while Byte is fullscreen-reduced, locked, display-sleeping, system-sleeping, or shutting down. A lifecycle control message resets the interpreter on resume so stale typing/idle state cannot burst into the companion.

No input history database exists.

## Testing

Deterministic Rust tests cover:

- alternating left/right typing taps
- fast-typing entry/exit
- mouse and scroll debounce
- idle start/end
- public payload privacy

Frontend tests cover:

- reaction → behavior mapping
- fast typing start/stop
- sleep/wake mapping
- fast typing not being interrupted by mouse input
- animation source release back to system base state


## Phase 19 collection counting

The lightweight collection system may increment one anonymous typing counter when the existing keyboard hook reports keyboard activity.

This does not widen the input payload. Byte still never stores or exposes:

- key identity
- scan code
- virtual-key code
- typed text
- focused application
- cursor position

The counter exists only for two finite local cosmetic milestones, is checkpointed in `collection.json`, and stops being relevant once those milestones are unlocked.
