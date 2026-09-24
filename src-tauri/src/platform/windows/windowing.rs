use crate::{
    core::{error::ByteError, state::AppState},
    models::{
        ByteConfig, CompanionPreferences, CompanionSize, DisplayMode, EdgeAnchor, SavedPlacement,
        WindowShellState,
    },
};
use tauri::{window::Monitor, AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    SetWindowDisplayAffinity, WDA_EXCLUDEFROMCAPTURE, WDA_NONE,
};

const QUICK_PANEL_LOGICAL_WIDTH: f64 = 340.0;
const QUICK_PANEL_LOGICAL_HEIGHT: f64 = 500.0;
const PANEL_GAP_LOGICAL: f64 = 10.0;
const DEFAULT_MARGIN_LOGICAL: f64 = 18.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TaskbarEdge {
    Top,
    Bottom,
    Left,
    Right,
}

pub fn initialize(app: &AppHandle) -> Result<(), ByteError> {
    let preferences = companion_preferences(app);
    apply_companion_layout(app, &preferences)
}

pub fn apply_capture_affinity(app: &AppHandle, exclude: bool) -> Result<(), ByteError> {
    let affinity = if exclude {
        WDA_EXCLUDEFROMCAPTURE
    } else {
        WDA_NONE
    };
    let mut last_error = None;

    for label in ["companion", "quick-panel", "main"] {
        let Some(window) = app.get_webview_window(label) else {
            continue;
        };
        let hwnd = match window.hwnd() {
            Ok(hwnd) => hwnd,
            Err(error) => {
                last_error = Some(ByteError::Window(error.to_string()));
                continue;
            }
        };
        let raw = hwnd.0 as windows_sys::Win32::Foundation::HWND;

        // SAFETY: the HWND belongs to this process and identifies a top-level
        // Tauri window. Affinity is limited to documented Windows values.
        if unsafe { SetWindowDisplayAffinity(raw, affinity) } == 0 {
            last_error = Some(ByteError::Window(format!(
                "Windows could not update capture exclusion for {label}"
            )));
        }
    }

    match last_error {
        Some(error) => Err(error),
        None => Ok(()),
    }
}

pub fn show_main_window(app: &AppHandle) -> Result<(), ByteError> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| ByteError::Window("main window is unavailable".into()))?;

    window
        .unminimize()
        .map_err(|error| ByteError::Window(error.to_string()))?;
    window
        .show()
        .map_err(|error| ByteError::Window(error.to_string()))?;
    window
        .set_focus()
        .map_err(|error| ByteError::Window(error.to_string()))
}

pub fn show_quick_panel(app: &AppHandle) -> Result<(), ByteError> {
    if app.state::<AppState>().is_visibility_suppressed() {
        return Ok(());
    }

    position_quick_panel(app)?;

    let panel = app
        .get_webview_window("quick-panel")
        .ok_or_else(|| ByteError::Window("quick panel is unavailable".into()))?;
    panel
        .show()
        .map_err(|error| ByteError::Window(error.to_string()))?;
    panel
        .set_focus()
        .map_err(|error| ByteError::Window(error.to_string()))?;
    let _ = app.emit_to("quick-panel", "byte://quick-panel-opened", true);
    Ok(())
}

pub fn hide_quick_panel(app: &AppHandle) -> Result<(), ByteError> {
    let panel = app
        .get_webview_window("quick-panel")
        .ok_or_else(|| ByteError::Window("quick panel is unavailable".into()))?;
    panel
        .hide()
        .map_err(|error| ByteError::Window(error.to_string()))?;
    let _ = app.emit_to("quick-panel", "byte://quick-panel-closed", true);
    Ok(())
}

pub fn show_companion(app: &AppHandle) -> Result<(), ByteError> {
    if app.state::<AppState>().is_visibility_suppressed() {
        return Ok(());
    }

    let preferences = companion_preferences(app);
    if preferences.display_mode == DisplayMode::Tray {
        return Ok(());
    }
    apply_companion_layout(app, &preferences)
}

pub fn hide_companion(app: &AppHandle) -> Result<(), ByteError> {
    let window = companion_window(app)?;
    window
        .hide()
        .map_err(|error| ByteError::Window(error.to_string()))?;
    emit_companion_visibility(app, false);
    Ok(())
}

pub fn is_companion_visible(app: &AppHandle) -> Result<bool, ByteError> {
    companion_window(app)?
        .is_visible()
        .map_err(|error| ByteError::Window(error.to_string()))
}

pub fn toggle_companion(app: &AppHandle) -> Result<(), ByteError> {
    let window = companion_window(app)?;
    let preferences = companion_preferences(app);

    if preferences.display_mode == DisplayMode::Tray {
        set_display_mode(app, DisplayMode::Habitat)?;
        return Ok(());
    }

    let visible = window
        .is_visible()
        .map_err(|error| ByteError::Window(error.to_string()))?;
    if visible {
        hide_companion(app)
    } else {
        show_companion(app)
    }
}

pub fn set_display_mode(app: &AppHandle, mode: DisplayMode) -> Result<ByteConfig, ByteError> {
    let config = {
        let state = app.state::<AppState>();
        let mut store = state
            .config
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        store.update_companion_with(|preferences| preferences.display_mode = mode)?
    };

    apply_companion_layout(app, &config.companion)?;
    let _ = app.emit_to("companion", "byte://display-mode-changed", mode);
    Ok(config)
}

pub fn set_companion_size(app: &AppHandle, size: CompanionSize) -> Result<ByteConfig, ByteError> {
    let config = {
        let state = app.state::<AppState>();
        let mut store = state
            .config
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        store.update_companion_with(|preferences| preferences.size = size)?
    };

    apply_companion_layout(app, &config.companion)?;
    let _ = app.emit_to("companion", "byte://companion-size-changed", size);
    Ok(config)
}

pub fn set_edge_anchor(app: &AppHandle, anchor: EdgeAnchor) -> Result<ByteConfig, ByteError> {
    let config = {
        let state = app.state::<AppState>();
        let mut store = state
            .config
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        store.update_companion_with(|preferences| preferences.edge_anchor = anchor)?
    };

    if config.companion.display_mode == DisplayMode::Edge {
        apply_companion_layout(app, &config.companion)?;
    }
    Ok(config)
}

pub fn begin_move_mode(app: &AppHandle) -> Result<WindowShellState, ByteError> {
    if app.state::<AppState>().is_visibility_suppressed() {
        return Err(ByteError::Window(
            "Byte cannot enter Move Mode while desktop awareness is hiding it".into(),
        ));
    }

    let preferences = companion_preferences(app);
    if preferences.display_mode == DisplayMode::Tray {
        return Err(ByteError::Window(
            "Byte cannot be moved while Tray mode is active".into(),
        ));
    }

    let window = companion_window(app)?;
    window
        .set_ignore_cursor_events(false)
        .map_err(|error| ByteError::Window(error.to_string()))?;
    window
        .show()
        .map_err(|error| ByteError::Window(error.to_string()))?;
    window
        .set_focus()
        .map_err(|error| ByteError::Window(error.to_string()))?;

    let shell = {
        let state = app.state::<AppState>();
        let mut shell = state
            .window_shell
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        shell.move_mode = true;
        shell.click_through = false;
        *shell
    };

    emit_move_mode(app, true);
    emit_click_through(app, false);
    Ok(shell)
}

pub fn drag_companion(app: &AppHandle) -> Result<WindowShellState, ByteError> {
    if !shell_state(app).move_mode {
        return Err(ByteError::Window("Move mode is not active".into()));
    }

    companion_window(app)?
        .start_dragging()
        .map_err(|error| ByteError::Window(error.to_string()))?;

    finish_move_mode(app)
}

pub fn finish_move_mode(app: &AppHandle) -> Result<WindowShellState, ByteError> {
    // Clear the interaction state first so a monitor disappearing mid-drag
    // can never strand Byte in Move Mode.
    let shell = {
        let state = app.state::<AppState>();
        let mut shell = state
            .window_shell
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        shell.move_mode = false;
        *shell
    };
    emit_move_mode(app, false);

    let save_result = save_current_placement(app);
    let preferences = companion_preferences(app);
    let layout_result = apply_companion_layout(app, &preferences);

    save_result.and(layout_result)?;
    Ok(shell)
}

pub fn set_click_through(app: &AppHandle, enabled: bool) -> Result<WindowShellState, ByteError> {
    let window = companion_window(app)?;

    {
        let state = app.state::<AppState>();
        let mut shell = state
            .window_shell
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        shell.click_through = enabled;
        if enabled {
            shell.move_mode = false;
        }
    }

    window
        .set_ignore_cursor_events(enabled)
        .map_err(|error| ByteError::Window(error.to_string()))?;

    if enabled {
        emit_move_mode(app, false);
    }
    emit_click_through(app, enabled);

    Ok(shell_state(app))
}

pub fn toggle_click_through(app: &AppHandle) -> Result<WindowShellState, ByteError> {
    let enabled = !shell_state(app).click_through;
    set_click_through(app, enabled)
}

pub fn shell_state(app: &AppHandle) -> WindowShellState {
    *app.state::<AppState>()
        .window_shell
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub fn apply_companion_layout(
    app: &AppHandle,
    preferences: &CompanionPreferences,
) -> Result<(), ByteError> {
    let window = companion_window(app)?;

    if preferences.display_mode == DisplayMode::Tray {
        window
            .hide()
            .map_err(|error| ByteError::Window(error.to_string()))?;
        emit_companion_visibility(app, false);
        return Ok(());
    }

    let saved = preferences
        .placements
        .get(preferences.display_mode)
        .cloned();
    let monitor = choose_monitor(&window, saved.as_ref())?;
    let scale = monitor.scale_factor();
    let (logical_width, logical_height) = logical_size(preferences.display_mode, preferences.size);
    let physical_size = PhysicalSize::new(
        physical_pixels(logical_width, scale),
        physical_pixels(logical_height, scale),
    );

    window
        .set_size(physical_size)
        .map_err(|error| ByteError::Window(error.to_string()))?;

    let position = saved
        .as_ref()
        .map(|placement| {
            position_from_saved(
                &monitor,
                physical_size,
                placement,
                preferences.display_mode,
                preferences.edge_anchor,
            )
        })
        .unwrap_or_else(|| {
            default_position(
                &monitor,
                physical_size,
                preferences.display_mode,
                preferences.edge_anchor,
            )
        });

    window
        .set_position(position)
        .map_err(|error| ByteError::Window(error.to_string()))?;
    window
        .set_always_on_top(true)
        .map_err(|error| ByteError::Window(error.to_string()))?;
    window
        .set_skip_taskbar(true)
        .map_err(|error| ByteError::Window(error.to_string()))?;

    if app.state::<AppState>().is_visibility_suppressed() {
        window
            .hide()
            .map_err(|error| ByteError::Window(error.to_string()))?;
        emit_companion_visibility(app, false);
        return Ok(());
    }

    window
        .show()
        .map_err(|error| ByteError::Window(error.to_string()))?;

    let click_through = shell_state(app).click_through;
    window
        .set_ignore_cursor_events(click_through)
        .map_err(|error| ByteError::Window(error.to_string()))?;
    emit_companion_visibility(app, true);
    Ok(())
}

fn save_current_placement(app: &AppHandle) -> Result<(), ByteError> {
    let window = companion_window(app)?;
    let preferences = companion_preferences(app);

    if preferences.display_mode == DisplayMode::Tray {
        return Ok(());
    }

    let monitor = choose_monitor(&window, None)?;

    let position = window
        .outer_position()
        .map_err(|error| ByteError::Window(error.to_string()))?;
    let size = window
        .outer_size()
        .map_err(|error| ByteError::Window(error.to_string()))?;
    let work = monitor.work_area();

    let max_x = work.size.width.saturating_sub(size.width).max(1);
    let max_y = work.size.height.saturating_sub(size.height).max(1);
    let relative_x = (position.x - work.position.x).clamp(0, max_x as i32);
    let relative_y = (position.y - work.position.y).clamp(0, max_y as i32);

    let placement = SavedPlacement {
        monitor_name: monitor.name().cloned(),
        x: (relative_x as f32 / max_x as f32).clamp(0.0, 1.0),
        y: (relative_y as f32 / max_y as f32).clamp(0.0, 1.0),
    };

    let state = app.state::<AppState>();
    let mut store = state
        .config
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    store.update_companion_with(|preferences| {
        preferences
            .placements
            .set(preferences.display_mode, placement);
    })?;

    Ok(())
}

fn position_quick_panel(app: &AppHandle) -> Result<(), ByteError> {
    let panel = app
        .get_webview_window("quick-panel")
        .ok_or_else(|| ByteError::Window("quick panel is unavailable".into()))?;
    let companion = companion_window(app)?;
    let preferences = companion_preferences(app);

    let companion_visible = companion
        .is_visible()
        .map_err(|error| ByteError::Window(error.to_string()))?
        && preferences.display_mode != DisplayMode::Tray;

    if companion_visible {
        let monitor = choose_monitor(&companion, None)?;
        let scale = monitor.scale_factor();
        let panel_size = PhysicalSize::new(
            physical_pixels(QUICK_PANEL_LOGICAL_WIDTH, scale),
            physical_pixels(QUICK_PANEL_LOGICAL_HEIGHT, scale),
        );
        panel
            .set_size(panel_size)
            .map_err(|error| ByteError::Window(error.to_string()))?;

        let companion_position = companion
            .outer_position()
            .map_err(|error| ByteError::Window(error.to_string()))?;
        let companion_size = companion
            .outer_size()
            .map_err(|error| ByteError::Window(error.to_string()))?;
        let work = monitor.work_area();
        let gap = physical_pixels(PANEL_GAP_LOGICAL, scale) as i32;

        let work_right = work.position.x + work.size.width as i32;
        let work_bottom = work.position.y + work.size.height as i32;

        let right_candidate = companion_position.x + companion_size.width as i32 + gap;
        let left_candidate = companion_position.x - panel_size.width as i32 - gap;
        let x = if right_candidate + panel_size.width as i32 <= work_right {
            right_candidate
        } else {
            left_candidate.max(work.position.x)
        };

        let max_panel_y = (work_bottom - panel_size.height as i32).max(work.position.y);
        let y = companion_position.y.clamp(work.position.y, max_panel_y);

        panel
            .set_position(PhysicalPosition::new(x, y))
            .map_err(|error| ByteError::Window(error.to_string()))?;
        return Ok(());
    }

    let monitor = panel
        .primary_monitor()
        .map_err(|error| ByteError::Window(error.to_string()))?
        .or_else(|| panel.available_monitors().ok()?.into_iter().next())
        .ok_or_else(|| ByteError::Window("No monitor is available".into()))?;
    let scale = monitor.scale_factor();
    let panel_size = PhysicalSize::new(
        physical_pixels(QUICK_PANEL_LOGICAL_WIDTH, scale),
        physical_pixels(QUICK_PANEL_LOGICAL_HEIGHT, scale),
    );
    panel
        .set_size(panel_size)
        .map_err(|error| ByteError::Window(error.to_string()))?;

    let work = monitor.work_area();
    let margin = physical_pixels(DEFAULT_MARGIN_LOGICAL, scale) as i32;
    let x = work.position.x + work.size.width as i32 - panel_size.width as i32 - margin;
    let y = work.position.y + work.size.height as i32 - panel_size.height as i32 - margin;

    panel
        .set_position(PhysicalPosition::new(
            x.max(work.position.x),
            y.max(work.position.y),
        ))
        .map_err(|error| ByteError::Window(error.to_string()))
}

fn choose_monitor(
    window: &tauri::WebviewWindow,
    saved: Option<&SavedPlacement>,
) -> Result<Monitor, ByteError> {
    let available = window
        .available_monitors()
        .map_err(|error| ByteError::Window(error.to_string()))?;

    if let Some(saved_name) = saved.and_then(|placement| placement.monitor_name.as_deref()) {
        if let Some(monitor) = available
            .iter()
            .find(|monitor| monitor.name().map(String::as_str) == Some(saved_name))
        {
            return Ok(monitor.clone());
        }
    }

    if let Some(monitor) = window
        .current_monitor()
        .map_err(|error| ByteError::Window(error.to_string()))?
    {
        return Ok(monitor);
    }

    if let Some(monitor) = window
        .primary_monitor()
        .map_err(|error| ByteError::Window(error.to_string()))?
    {
        return Ok(monitor);
    }

    available
        .into_iter()
        .next()
        .ok_or_else(|| ByteError::Window("No monitor is available".into()))
}

fn position_from_saved(
    monitor: &Monitor,
    size: PhysicalSize<u32>,
    placement: &SavedPlacement,
    mode: DisplayMode,
    edge_anchor: EdgeAnchor,
) -> PhysicalPosition<i32> {
    let work = monitor.work_area();
    let max_x = work.size.width.saturating_sub(size.width) as i32;
    let max_y = work.size.height.saturating_sub(size.height) as i32;
    let saved_x = (placement.x.clamp(0.0, 1.0) * max_x as f32).round() as i32;
    let saved_y = (placement.y.clamp(0.0, 1.0) * max_y as f32).round() as i32;

    let (relative_x, relative_y) = match mode {
        DisplayMode::Perch => match infer_taskbar_edge(monitor) {
            TaskbarEdge::Bottom => (saved_x, max_y),
            TaskbarEdge::Top => (saved_x, 0),
            TaskbarEdge::Left => (0, saved_y),
            TaskbarEdge::Right => (max_x, saved_y),
        },
        DisplayMode::Edge => match edge_anchor {
            EdgeAnchor::Left => (0, saved_y),
            EdgeAnchor::Right => (max_x, saved_y),
        },
        _ => (saved_x, saved_y),
    };

    PhysicalPosition::new(
        work.position.x + relative_x.max(0),
        work.position.y + relative_y.max(0),
    )
}

fn default_position(
    monitor: &Monitor,
    size: PhysicalSize<u32>,
    mode: DisplayMode,
    edge_anchor: EdgeAnchor,
) -> PhysicalPosition<i32> {
    let work = monitor.work_area();
    let scale = monitor.scale_factor();
    let margin = physical_pixels(DEFAULT_MARGIN_LOGICAL, scale) as i32;
    let max_x = work.size.width.saturating_sub(size.width) as i32;
    let max_y = work.size.height.saturating_sub(size.height) as i32;

    let relative = match mode {
        DisplayMode::Habitat | DisplayMode::Mini => (max_x - margin, max_y - margin),
        DisplayMode::Perch => perch_relative_position(monitor, max_x, max_y, margin),
        DisplayMode::Edge => {
            let x = match edge_anchor {
                EdgeAnchor::Left => 0,
                EdgeAnchor::Right => max_x,
            };
            (x, max_y / 2)
        }
        DisplayMode::Tray => (max_x, max_y),
    };

    PhysicalPosition::new(
        work.position.x + relative.0.max(0),
        work.position.y + relative.1.max(0),
    )
}

fn perch_relative_position(monitor: &Monitor, max_x: i32, max_y: i32, margin: i32) -> (i32, i32) {
    match infer_taskbar_edge(monitor) {
        TaskbarEdge::Bottom => (max_x - margin, max_y),
        TaskbarEdge::Top => (max_x - margin, 0),
        TaskbarEdge::Left => (0, max_y - margin),
        TaskbarEdge::Right => (max_x, max_y - margin),
    }
}

fn infer_taskbar_edge(monitor: &Monitor) -> TaskbarEdge {
    let work = monitor.work_area();
    infer_taskbar_edge_from_rects(
        *monitor.position(),
        *monitor.size(),
        work.position,
        work.size,
    )
}

fn infer_taskbar_edge_from_rects(
    full_position: PhysicalPosition<i32>,
    full_size: PhysicalSize<u32>,
    work_position: PhysicalPosition<i32>,
    work_size: PhysicalSize<u32>,
) -> TaskbarEdge {
    let full_right = full_position.x + full_size.width as i32;
    let work_right = work_position.x + work_size.width as i32;

    if work_position.y > full_position.y {
        TaskbarEdge::Top
    } else if work_position.x > full_position.x {
        TaskbarEdge::Left
    } else if work_right < full_right {
        TaskbarEdge::Right
    } else {
        TaskbarEdge::Bottom
    }
}

fn logical_size(mode: DisplayMode, size: CompanionSize) -> (f64, f64) {
    let scale = match size {
        CompanionSize::Small => 0.80,
        CompanionSize::Medium => 1.0,
        CompanionSize::Large => 1.20,
    };

    let base = match mode {
        DisplayMode::Habitat => (240.0, 240.0),
        DisplayMode::Perch => (170.0, 126.0),
        DisplayMode::Mini => (116.0, 116.0),
        DisplayMode::Edge => (132.0, 176.0),
        DisplayMode::Tray => (1.0, 1.0),
    };

    (base.0 * scale, base.1 * scale)
}

fn physical_pixels(logical: f64, scale_factor: f64) -> u32 {
    (logical * scale_factor).round().max(1.0) as u32
}

fn companion_preferences(app: &AppHandle) -> CompanionPreferences {
    app.state::<AppState>()
        .config
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .snapshot()
        .companion
}

fn companion_window(app: &AppHandle) -> Result<tauri::WebviewWindow, ByteError> {
    app.get_webview_window("companion")
        .ok_or_else(|| ByteError::Window("companion window is unavailable".into()))
}

fn emit_companion_visibility(app: &AppHandle, visible: bool) {
    let _ = app.emit_to("companion", "byte://companion-visibility-changed", visible);
}

fn emit_move_mode(app: &AppHandle, enabled: bool) {
    let _ = app.emit_to("companion", "byte://move-mode-changed", enabled);
}

fn emit_click_through(app: &AppHandle, enabled: bool) {
    let _ = app.emit_to("companion", "byte://click-through-changed", enabled);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logical_sizes_follow_mode_and_size_category() {
        assert_eq!(
            logical_size(DisplayMode::Habitat, CompanionSize::Medium),
            (240.0, 240.0)
        );
        assert_eq!(
            logical_size(DisplayMode::Mini, CompanionSize::Large),
            (139.2, 139.2)
        );
    }

    #[test]
    fn physical_pixel_conversion_is_integer_and_dpi_aware() {
        assert_eq!(physical_pixels(240.0, 1.25), 300);
        assert_eq!(physical_pixels(116.0, 1.5), 174);
    }

    #[test]
    fn taskbar_edge_is_inferred_from_work_area() {
        let full_position = PhysicalPosition::new(0, 0);
        let full_size = PhysicalSize::new(1920, 1080);

        assert_eq!(
            infer_taskbar_edge_from_rects(
                full_position,
                full_size,
                PhysicalPosition::new(0, 0),
                PhysicalSize::new(1920, 1040),
            ),
            TaskbarEdge::Bottom
        );
        assert_eq!(
            infer_taskbar_edge_from_rects(
                full_position,
                full_size,
                PhysicalPosition::new(0, 40),
                PhysicalSize::new(1920, 1040),
            ),
            TaskbarEdge::Top
        );
        assert_eq!(
            infer_taskbar_edge_from_rects(
                full_position,
                full_size,
                PhysicalPosition::new(40, 0),
                PhysicalSize::new(1880, 1080),
            ),
            TaskbarEdge::Left
        );
        assert_eq!(
            infer_taskbar_edge_from_rects(
                full_position,
                full_size,
                PhysicalPosition::new(0, 0),
                PhysicalSize::new(1880, 1080),
            ),
            TaskbarEdge::Right
        );
    }
}
