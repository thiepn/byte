use crate::{core::error::ByteError, models::now_epoch_ms};
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    ptr::{null, null_mut},
    sync::{
        mpsc::{self, Receiver, RecvTimeoutError, Sender},
        Mutex, OnceLock,
    },
    thread::{self, JoinHandle},
    time::Duration,
};
use tauri::{AppHandle, Emitter};
use windows_sys::Win32::{
    System::{LibraryLoader::GetModuleHandleW, Threading::GetCurrentThreadId},
    UI::WindowsAndMessaging::{
        CallNextHookEx, GetMessageW, PeekMessageW, PostThreadMessageW, SetWindowsHookExW,
        UnhookWindowsHookEx, MSG, PM_NOREMOVE, WH_KEYBOARD_LL, WH_MOUSE_LL, WM_KEYDOWN,
        WM_LBUTTONDOWN, WM_MOUSEHWHEEL, WM_MOUSEWHEEL, WM_QUIT, WM_RBUTTONDOWN, WM_SYSKEYDOWN,
    },
};

const TYPING_WINDOW_MS: u64 = 750;
const FAST_TYPING_MIN_KEYS: usize = 6;
const FAST_TYPING_STOP_MS: u64 = 500;
const IDLE_START_MS: u64 = 30_000;
const MOUSE_DEBOUNCE_MS: u64 = 45;
const SCROLL_DEBOUNCE_MS: u64 = 75;
const PROCESSOR_MAX_SLEEP_MS: u64 = 5_000;

static CALLBACK_SENDER: OnceLock<Mutex<Option<Sender<RawInputMessage>>>> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RawInputKind {
    Keyboard,
    MouseLeft,
    MouseRight,
    Scroll,
}

#[derive(Debug, Clone, Copy)]
struct RawInputActivity {
    kind: RawInputKind,
    timestamp_epoch_ms: u64,
}

enum RawInputMessage {
    Activity(RawInputActivity),
    Shutdown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InputReactionKind {
    TypingTapLeft,
    TypingTapRight,
    TypingFastStart,
    TypingFastStop,
    MouseLeft,
    MouseRight,
    Scroll,
    IdleStart,
    IdleEnd,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct InputReactionEvent {
    pub kind: InputReactionKind,
    pub timestamp_epoch_ms: u64,
    pub keys_per_second: Option<f32>,
}

pub struct InputRuntime {
    hook_thread_id: u32,
    control_tx: Sender<RawInputMessage>,
    hook_worker: JoinHandle<()>,
    processor_worker: JoinHandle<()>,
}

impl InputRuntime {
    pub fn start(app: AppHandle) -> Result<Self, ByteError> {
        let (raw_tx, raw_rx) = mpsc::channel::<RawInputMessage>();
        let (ready_tx, ready_rx) = mpsc::sync_channel::<Result<u32, String>>(1);

        let processor_app = app.clone();
        let processor_worker = thread::Builder::new()
            .name("byte-input-processor".into())
            .spawn(move || processor_loop(processor_app, raw_rx))
            .map_err(|error| ByteError::Io(error.to_string()))?;

        let hook_tx = raw_tx.clone();
        let hook_worker = match thread::Builder::new()
            .name("byte-input-hooks".into())
            .spawn(move || hook_loop(hook_tx, ready_tx))
        {
            Ok(worker) => worker,
            Err(error) => {
                let _ = raw_tx.send(RawInputMessage::Shutdown);
                let _ = processor_worker.join();
                return Err(ByteError::Io(error.to_string()));
            }
        };

        let hook_thread_id = match ready_rx.recv() {
            Ok(Ok(thread_id)) => thread_id,
            Ok(Err(message)) => {
                let _ = raw_tx.send(RawInputMessage::Shutdown);
                let _ = hook_worker.join();
                let _ = processor_worker.join();
                return Err(ByteError::Io(message));
            }
            Err(error) => {
                let _ = raw_tx.send(RawInputMessage::Shutdown);
                let _ = hook_worker.join();
                let _ = processor_worker.join();
                return Err(ByteError::Io(error.to_string()));
            }
        };

        Ok(Self {
            hook_thread_id,
            control_tx: raw_tx,
            hook_worker,
            processor_worker,
        })
    }

    pub fn stop(self) {
        let _ = self.control_tx.send(RawInputMessage::Shutdown);

        // SAFETY: the hook thread creates a message queue before reporting ready,
        // and this call only posts WM_QUIT to that known thread.
        unsafe {
            let _ = PostThreadMessageW(self.hook_thread_id, WM_QUIT, 0, 0);
        }

        let _ = self.hook_worker.join();
        let _ = self.processor_worker.join();
    }
}

fn callback_sender() -> &'static Mutex<Option<Sender<RawInputMessage>>> {
    CALLBACK_SENDER.get_or_init(|| Mutex::new(None))
}

fn install_callback_sender(sender: Sender<RawInputMessage>) {
    *callback_sender()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(sender);
}

fn clear_callback_sender() {
    *callback_sender()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = None;
}

fn enqueue_from_hook(kind: RawInputKind) {
    let sender = callback_sender()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();

    if let Some(sender) = sender {
        let _ = sender.send(RawInputMessage::Activity(RawInputActivity {
            kind,
            timestamp_epoch_ms: now_epoch_ms(),
        }));
    }
}

unsafe extern "system" fn keyboard_hook(code: i32, wparam: usize, lparam: isize) -> isize {
    if code >= 0 {
        let message = wparam as u32;
        if message == WM_KEYDOWN || message == WM_SYSKEYDOWN {
            enqueue_from_hook(RawInputKind::Keyboard);
        }
    }

    // SAFETY: the callback never blocks or consumes the event; it immediately
    // passes the original hook arguments to the next hook in the chain.
    unsafe { CallNextHookEx(null_mut(), code, wparam, lparam) }
}

unsafe extern "system" fn mouse_hook(code: i32, wparam: usize, lparam: isize) -> isize {
    if code >= 0 {
        match wparam as u32 {
            WM_LBUTTONDOWN => enqueue_from_hook(RawInputKind::MouseLeft),
            WM_RBUTTONDOWN => enqueue_from_hook(RawInputKind::MouseRight),
            WM_MOUSEWHEEL | WM_MOUSEHWHEEL => enqueue_from_hook(RawInputKind::Scroll),
            _ => {}
        }
    }

    // SAFETY: as above, Byte observes but never consumes the system input event.
    unsafe { CallNextHookEx(null_mut(), code, wparam, lparam) }
}

fn hook_loop(sender: Sender<RawInputMessage>, ready: mpsc::SyncSender<Result<u32, String>>) {
    install_callback_sender(sender);

    // SAFETY: the hook thread owns its Win32 message queue and hook handles for
    // the full duration of this function.
    unsafe {
        let thread_id = GetCurrentThreadId();
        let mut message: MSG = std::mem::zeroed();

        // Force creation of a message queue so PostThreadMessageW is reliable.
        let _ = PeekMessageW(&mut message, null_mut(), 0, 0, PM_NOREMOVE);

        let module = GetModuleHandleW(null());
        if module.is_null() {
            clear_callback_sender();
            let _ = ready.send(Err(std::io::Error::last_os_error().to_string()));
            return;
        }

        let keyboard = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook), module, 0);
        if keyboard.is_null() {
            clear_callback_sender();
            let _ = ready.send(Err(std::io::Error::last_os_error().to_string()));
            return;
        }

        let mouse = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_hook), module, 0);
        if mouse.is_null() {
            let _ = UnhookWindowsHookEx(keyboard);
            clear_callback_sender();
            let _ = ready.send(Err(std::io::Error::last_os_error().to_string()));
            return;
        }

        let _ = ready.send(Ok(thread_id));

        loop {
            let result = GetMessageW(&mut message, null_mut(), 0, 0);
            if result <= 0 {
                break;
            }
        }

        let _ = UnhookWindowsHookEx(mouse);
        let _ = UnhookWindowsHookEx(keyboard);
    }

    clear_callback_sender();
}

fn processor_loop(app: AppHandle, receiver: Receiver<RawInputMessage>) {
    let mut interpreter = InputInterpreter::new(now_epoch_ms());

    loop {
        let now = now_epoch_ms();
        let timeout = interpreter.next_deadline_ms(now);
        let wait = Duration::from_millis(timeout.min(PROCESSOR_MAX_SLEEP_MS).max(1));

        match receiver.recv_timeout(wait) {
            Ok(RawInputMessage::Activity(activity)) => {
                emit_reactions(&app, interpreter.handle(activity));
            }
            Ok(RawInputMessage::Shutdown) => break,
            Err(RecvTimeoutError::Timeout) => {
                emit_reactions(&app, interpreter.tick(now_epoch_ms()));
            }
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }
}

fn emit_reactions(app: &AppHandle, reactions: Vec<InputReactionEvent>) {
    for reaction in reactions {
        let _ = app.emit_to("companion", "byte://input-reaction", reaction);
    }
}

struct InputInterpreter {
    key_times: VecDeque<u64>,
    next_left_tap: bool,
    fast_typing: bool,
    idle: bool,
    last_key_ms: Option<u64>,
    last_activity_ms: u64,
    last_left_click_ms: Option<u64>,
    last_right_click_ms: Option<u64>,
    last_scroll_ms: Option<u64>,
}

impl InputInterpreter {
    fn new(now: u64) -> Self {
        Self {
            key_times: VecDeque::new(),
            next_left_tap: true,
            fast_typing: false,
            idle: false,
            last_key_ms: None,
            last_activity_ms: now,
            last_left_click_ms: None,
            last_right_click_ms: None,
            last_scroll_ms: None,
        }
    }

    fn handle(&mut self, activity: RawInputActivity) -> Vec<InputReactionEvent> {
        let mut reactions = Vec::with_capacity(2);

        if self.idle {
            self.idle = false;
            reactions.push(reaction(
                InputReactionKind::IdleEnd,
                activity.timestamp_epoch_ms,
                None,
            ));
        }
        self.last_activity_ms = activity.timestamp_epoch_ms;

        match activity.kind {
            RawInputKind::Keyboard => {
                self.handle_keyboard(activity.timestamp_epoch_ms, &mut reactions)
            }
            RawInputKind::MouseLeft => {
                if debounce(
                    &mut self.last_left_click_ms,
                    activity.timestamp_epoch_ms,
                    MOUSE_DEBOUNCE_MS,
                ) {
                    reactions.push(reaction(
                        InputReactionKind::MouseLeft,
                        activity.timestamp_epoch_ms,
                        None,
                    ));
                }
            }
            RawInputKind::MouseRight => {
                if debounce(
                    &mut self.last_right_click_ms,
                    activity.timestamp_epoch_ms,
                    MOUSE_DEBOUNCE_MS,
                ) {
                    reactions.push(reaction(
                        InputReactionKind::MouseRight,
                        activity.timestamp_epoch_ms,
                        None,
                    ));
                }
            }
            RawInputKind::Scroll => {
                if debounce(
                    &mut self.last_scroll_ms,
                    activity.timestamp_epoch_ms,
                    SCROLL_DEBOUNCE_MS,
                ) {
                    reactions.push(reaction(
                        InputReactionKind::Scroll,
                        activity.timestamp_epoch_ms,
                        None,
                    ));
                }
            }
        }

        reactions
    }

    fn handle_keyboard(&mut self, now: u64, reactions: &mut Vec<InputReactionEvent>) {
        while self
            .key_times
            .front()
            .map(|timestamp| now.saturating_sub(*timestamp) > TYPING_WINDOW_MS)
            .unwrap_or(false)
        {
            self.key_times.pop_front();
        }

        self.key_times.push_back(now);
        self.last_key_ms = Some(now);

        let keys_per_second = self.key_times.len() as f32 * 1000.0 / TYPING_WINDOW_MS as f32;

        if self.key_times.len() >= FAST_TYPING_MIN_KEYS {
            if !self.fast_typing {
                self.fast_typing = true;
                reactions.push(reaction(
                    InputReactionKind::TypingFastStart,
                    now,
                    Some(keys_per_second),
                ));
            }
            return;
        }

        if !self.fast_typing {
            let kind = if self.next_left_tap {
                InputReactionKind::TypingTapLeft
            } else {
                InputReactionKind::TypingTapRight
            };
            self.next_left_tap = !self.next_left_tap;
            reactions.push(reaction(kind, now, Some(keys_per_second)));
        }
    }

    fn tick(&mut self, now: u64) -> Vec<InputReactionEvent> {
        let mut reactions = Vec::with_capacity(2);

        if self.fast_typing
            && self
                .last_key_ms
                .map(|last| now.saturating_sub(last) >= FAST_TYPING_STOP_MS)
                .unwrap_or(false)
        {
            self.fast_typing = false;
            self.key_times.clear();
            reactions.push(reaction(InputReactionKind::TypingFastStop, now, None));
        }

        if !self.idle && now.saturating_sub(self.last_activity_ms) >= IDLE_START_MS {
            self.idle = true;
            reactions.push(reaction(InputReactionKind::IdleStart, now, None));
        }

        reactions
    }

    fn next_deadline_ms(&self, now: u64) -> u64 {
        let idle_remaining =
            IDLE_START_MS.saturating_sub(now.saturating_sub(self.last_activity_ms));
        let typing_remaining = if self.fast_typing {
            self.last_key_ms
                .map(|last| FAST_TYPING_STOP_MS.saturating_sub(now.saturating_sub(last)))
                .unwrap_or(PROCESSOR_MAX_SLEEP_MS)
        } else {
            PROCESSOR_MAX_SLEEP_MS
        };

        idle_remaining.min(typing_remaining).max(1)
    }
}

fn debounce(last: &mut Option<u64>, now: u64, threshold_ms: u64) -> bool {
    let should_emit = last
        .map(|previous| now.saturating_sub(previous) >= threshold_ms)
        .unwrap_or(true);
    if should_emit {
        *last = Some(now);
    }
    should_emit
}

fn reaction(
    kind: InputReactionKind,
    timestamp_epoch_ms: u64,
    keys_per_second: Option<f32>,
) -> InputReactionEvent {
    InputReactionEvent {
        kind,
        timestamp_epoch_ms,
        keys_per_second,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raw(kind: RawInputKind, timestamp_epoch_ms: u64) -> RawInputActivity {
        RawInputActivity {
            kind,
            timestamp_epoch_ms,
        }
    }

    #[test]
    fn keyboard_activity_alternates_left_and_right_taps() {
        let mut interpreter = InputInterpreter::new(0);

        let first = interpreter.handle(raw(RawInputKind::Keyboard, 100));
        let second = interpreter.handle(raw(RawInputKind::Keyboard, 200));

        assert_eq!(first[0].kind, InputReactionKind::TypingTapLeft);
        assert_eq!(second[0].kind, InputReactionKind::TypingTapRight);
    }

    #[test]
    fn fast_typing_enters_once_and_stops_after_quiet_period() {
        let mut interpreter = InputInterpreter::new(0);

        let mut start_events = Vec::new();
        for index in 0..FAST_TYPING_MIN_KEYS {
            start_events
                .extend(interpreter.handle(raw(RawInputKind::Keyboard, 100 + index as u64 * 80)));
        }

        assert_eq!(
            start_events
                .iter()
                .filter(|event| event.kind == InputReactionKind::TypingFastStart)
                .count(),
            1
        );

        let stop = interpreter.tick(1_100);
        assert!(stop
            .iter()
            .any(|event| event.kind == InputReactionKind::TypingFastStop));
    }

    #[test]
    fn clicks_and_scroll_are_debounced_without_losing_activity_time() {
        let mut interpreter = InputInterpreter::new(0);

        assert_eq!(
            interpreter.handle(raw(RawInputKind::MouseLeft, 100)).len(),
            1
        );
        assert!(interpreter
            .handle(raw(RawInputKind::MouseLeft, 120))
            .is_empty());
        assert_eq!(
            interpreter.handle(raw(RawInputKind::MouseLeft, 160)).len(),
            1
        );

        assert_eq!(interpreter.handle(raw(RawInputKind::Scroll, 200)).len(), 1);
        assert!(interpreter
            .handle(raw(RawInputKind::Scroll, 240))
            .is_empty());
    }

    #[test]
    fn idle_starts_after_thirty_seconds_and_ends_on_any_input() {
        let mut interpreter = InputInterpreter::new(1_000);

        let idle = interpreter.tick(31_000);
        assert_eq!(idle[0].kind, InputReactionKind::IdleStart);

        let resumed = interpreter.handle(raw(RawInputKind::MouseRight, 31_100));
        assert_eq!(resumed[0].kind, InputReactionKind::IdleEnd);
        assert_eq!(resumed[1].kind, InputReactionKind::MouseRight);
    }

    #[test]
    fn public_reaction_payload_has_no_key_identity_fields() {
        let event = reaction(InputReactionKind::TypingTapLeft, 42, Some(2.0));
        let json = serde_json::to_value(event).expect("serialize");

        assert!(json.get("kind").is_some());
        assert!(json.get("timestamp_epoch_ms").is_some());
        assert!(json.get("keys_per_second").is_some());
        assert!(json.get("key").is_none());
        assert!(json.get("vk_code").is_none());
        assert!(json.get("scan_code").is_none());
    }
}
