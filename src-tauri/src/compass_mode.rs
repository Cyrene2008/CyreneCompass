// 桌面罗盘模式：任意界面长按 Tab，罗盘在屏幕中央展开。
// 始终以屏幕中心（罗盘所在显示器的中央）为原点观察鼠标位置，
// 鼠标朝某个方向移动即选中并高亮对应应用，松开 Tab 启动，移回中心松开则取消。
//
// 实现：WH_KEYBOARD_LL 低级键盘钩子检测 Tab 按下/松开（可全局感知，无需窗口焦点），
// 控制器线程轮询 GetCursorPos 计算方向（带死区与迟滞），并通过 Tauri 事件驱动前端 overlay 窗口。

use std::sync::atomic::{AtomicBool, AtomicI8, AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize};

const VK_TAB: u32 = 0x09;
const LLKHF_INJECTED: u32 = 0x10;
const LLKHF_UP: u32 = 0x80;

const OVERLAY_LABEL: &str = "compass-overlay";
const POLL_INTERVAL: Duration = Duration::from_millis(12);
const HIDE_DELAY: Duration = Duration::from_millis(170);
/// 死区：位移低于该距离视为未选择（物理像素，随 DPI 缩放）。
const BASE_DEAD_ZONE: f64 = 42.0;

static APP: OnceLock<AppHandle> = OnceLock::new();
static ENABLED: AtomicBool = AtomicBool::new(true);
static HOLD_MS: AtomicU64 = AtomicU64::new(1000);
/// 劫持 Tab：按下时吞掉不转发，松开时补发一次完整按键。
static HIJACK: AtomicBool = AtomicBool::new(false);
static TAB_DOWN: Mutex<Option<Instant>> = Mutex::new(None);
/// Tab 当前是否按住（由钩子根据真实按键事件维护，不依赖 GetAsyncKeyState——
/// 被吞掉的按键在部分输入链/驱动下会导致 GetAsyncKeyState 误报“未按住”）。
static TAB_HELD: AtomicBool = AtomicBool::new(false);
static ACTIVE: AtomicBool = AtomicBool::new(false);
static CLOSING_AT: Mutex<Option<Instant>> = Mutex::new(None);
/// 最近一次激活的时间（用于“激活后窗口不可见”的自愈判断，避免竞态误复位）。
static ACTIVATED_AT: Mutex<Option<Instant>> = Mutex::new(None);
/// 方向原点：罗盘所在显示器的屏幕中心（物理像素）。
static ORIGIN: Mutex<(i32, i32)> = Mutex::new((0, 0));
static SCALE: Mutex<f64> = Mutex::new(1.0);
static CURRENT_DIR: AtomicI8 = AtomicI8::new(-1);

/// 方向常量：0=上 1=右上 2=右 3=右下 4=下 5=左下 6=左 7=左上，-1=未选中。
/// (dx, dy) 为鼠标相对屏幕中心（原点）的位移。
pub fn compass_direction(dx: i32, dy: i32, current: i8, dead_zone_px: f64) -> i8 {
    let len = ((dx as f64).powi(2) + (dy as f64).powi(2)).sqrt();
    if len < dead_zone_px {
        return -1;
    }
    // 屏幕坐标 y 向下：atan2(dy, dx) 为 0 时指向右；+90° 后 0° 指向正上，顺时针递增。
    let angle = ((dy as f64).atan2(dx as f64).to_degrees() + 90.0).rem_euclid(360.0);
    let sector = (((angle + 22.5) / 45.0).floor() as i8).rem_euclid(8);
    if !(0..=7).contains(&current) {
        return sector;
    }
    // 迟滞：已选中的方向保持，直到角度明显越过边界（距新扇区中心过半）。
    let current_center = f64::from(current) * 45.0;
    if angular_distance(angle, current_center) <= 22.5 {
        current
    } else {
        sector
    }
}

fn angular_distance(a: f64, b: f64) -> f64 {
    let d = (a - b).rem_euclid(360.0);
    if d > 180.0 { 360.0 - d } else { d }
}

pub fn set_config(enabled: bool, hold_ms: u64, hijack: bool) {
    ENABLED.store(enabled, Ordering::Relaxed);
    HOLD_MS.store(hold_ms.clamp(300, 3000), Ordering::Relaxed);
    HIJACK.store(hijack, Ordering::Relaxed);
}

pub fn init(app: AppHandle) {
    if APP.set(app).is_err() {
        return;
    }
    #[cfg(target_os = "windows")]
    {
        install_keyboard_hook();
        spawn_controller();
    }
}

#[cfg(target_os = "windows")]
fn install_keyboard_hook() {
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows_sys::Win32::UI::WindowsAndMessaging::{DispatchMessageW, GetMessageW, SetWindowsHookExW, TranslateMessage, UnhookWindowsHookEx, WH_KEYBOARD_LL, MSG};
    thread::spawn(|| unsafe {
        let module = GetModuleHandleW(std::ptr::null());
        let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook_proc), module, 0);
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        if !hook.is_null() {
            UnhookWindowsHookEx(hook);
        }
    });
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn keyboard_hook_proc(code: i32, wparam: usize, lparam: isize) -> isize {
    use windows_sys::Win32::UI::WindowsAndMessaging::CallNextHookEx;
    // 回调绝不允许 panic 越过 FFI 边界（会杀死钩子线程并导致长按检测失效）
    let swallow = std::panic::catch_unwind(|| handle_tab_hook(code, wparam, lparam)).unwrap_or(false);
    if swallow {
        1
    } else {
        unsafe { CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam) }
    }
}

/// 返回 true 表示吞掉本次事件。低级键盘钩子只会收到 4 种按键消息，
/// 按下/松开以 KBDLLHOOKSTRUCT.flags 的 LLKHF_UP 为准，不依赖 wparam 的精确值。
#[cfg(target_os = "windows")]
fn handle_tab_hook(code: i32, wparam: usize, lparam: isize) -> bool {
    use windows_sys::Win32::UI::WindowsAndMessaging::KBDLLHOOKSTRUCT;
    if code < 0 {
        return false;
    }
    let kbd = unsafe { &*(lparam as *const KBDLLHOOKSTRUCT) };
    if kbd.vkCode != VK_TAB {
        return false;
    }
    let is_up = kbd.flags & LLKHF_UP != 0;
    if ACTIVE.load(Ordering::Relaxed) {
        // 罗盘展开期间吞掉 Tab 的重复按下与松开，避免下方应用焦点循环
        if is_up {
            TAB_HELD.store(false, Ordering::Relaxed);
            finalize_release();
        }
        return true;
    }
    let injected = kbd.flags & LLKHF_INJECTED != 0;
    // 劫持模式：按下吞掉不转发，松开时补发一次完整按键（Alt/Win 组合放行）
    if should_hijack(HIJACK.load(Ordering::Relaxed), injected, system_tab_blocked()) {
        if !is_up {
            TAB_HELD.store(true, Ordering::Relaxed);
            let mut guard = TAB_DOWN.lock().unwrap_or_else(|poison| poison.into_inner());
            if guard.is_none() {
                *guard = Some(Instant::now());
            }
            return true;
        }
        TAB_HELD.store(false, Ordering::Relaxed);
        // 仅当本次按下确实被吞掉（TAB_DOWN 有记录）才补发，
        // 防止残留 keyup 或状态竞态导致的多余 Tab
        let hijacked = TAB_DOWN.lock().unwrap_or_else(|poison| poison.into_inner()).take().is_some();
        if hijacked {
            relay_tab_press();
        }
        return true;
    }
    let _ = wparam;
    if !injected {
        if !is_up {
            TAB_HELD.store(true, Ordering::Relaxed);
            let mut guard = TAB_DOWN.lock().unwrap_or_else(|poison| poison.into_inner());
            if guard.is_none() {
                *guard = Some(Instant::now());
            }
        } else {
            TAB_HELD.store(false, Ordering::Relaxed);
            *TAB_DOWN.lock().unwrap_or_else(|poison| poison.into_inner()) = None;
        }
    }
    false
}

/// 是否对本次 Tab 事件启用劫持（Alt+Tab / Win+Tab 等系统组合一律放行）。
fn should_hijack(hijack_enabled: bool, injected: bool, system_tab_blocked: bool) -> bool {
    hijack_enabled && !injected && !system_tab_blocked
}

#[cfg(target_os = "windows")]
fn system_tab_blocked() -> bool {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{VK_LWIN, VK_MENU, VK_RWIN};
    key_down(VK_MENU) || key_down(VK_LWIN) || key_down(VK_RWIN)
}

/// 向系统补发一次 Tab 按下+松开（模拟用户短按了一次 Tab）。
#[cfg(target_os = "windows")]
fn relay_tab_press() {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{SendInput, INPUT, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VK_TAB};
    unsafe {
        let mut down = INPUT { r#type: INPUT_KEYBOARD, Anonymous: std::mem::zeroed() };
        down.Anonymous.ki = KEYBDINPUT { wVk: VK_TAB, wScan: 0, dwFlags: 0, time: 0, dwExtraInfo: 0 };
        let mut up = INPUT { r#type: INPUT_KEYBOARD, Anonymous: std::mem::zeroed() };
        up.Anonymous.ki = KEYBDINPUT { wVk: VK_TAB, wScan: 0, dwFlags: KEYEVENTF_KEYUP, time: 0, dwExtraInfo: 0 };
        let _ = SendInput(2, [down, up].as_ptr(), std::mem::size_of::<INPUT>() as i32);
    }
}

#[cfg(target_os = "windows")]
fn spawn_controller() {
    let Some(app) = APP.get().cloned() else { return };
    thread::spawn(move || loop {
        controller_tick(&app);
        thread::sleep(POLL_INTERVAL);
    });
}

#[cfg(target_os = "windows")]
fn controller_tick(app: &AppHandle) {
    if !ENABLED.load(Ordering::Relaxed) {
        return;
    }
    // 松开后的延迟隐藏（给前端留出收尾动画时间）
    let closing_due = CLOSING_AT
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .is_some_and(|at| Instant::now() >= at);
    if closing_due {
        *CLOSING_AT.lock().unwrap_or_else(|poison| poison.into_inner()) = None;
        if let Some(window) = app.get_webview_window(OVERLAY_LABEL) {
            let _ = window.hide();
        }
        return;
    }
    if !ACTIVE.load(Ordering::Relaxed) {
        let started = *TAB_DOWN.lock().unwrap_or_else(|poison| poison.into_inner());
        if let Some(started) = started {
            if TAB_HELD.load(Ordering::Relaxed) {
                if started.elapsed() >= Duration::from_millis(HOLD_MS.load(Ordering::Relaxed)) {
                    activate(app);
                }
            } else {
                // 钩子可能漏掉 keyup（如 Alt+Tab 被系统接管），此处自愈
                *TAB_DOWN.lock().unwrap_or_else(|poison| poison.into_inner()) = None;
            }
        }
        return;
    }
    // 自愈：展开期间若 Tab 已被松开（钩子状态），主动结束
    if !TAB_HELD.load(Ordering::Relaxed) {
        finalize_release();
        return;
    }
    // 自愈：ACTIVE 已置位但 overlay 窗口并不可见（activate 中途失败等）。
    // 激活后 500ms 内不检查，给窗口操作留足时间，避免时序竞态误复位。
    let activated_for = ACTIVATED_AT
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .map_or(Duration::ZERO, |at| at.elapsed());
    let overlay_visible = app
        .get_webview_window(OVERLAY_LABEL)
        .and_then(|window| window.is_visible().ok())
        .unwrap_or(false);
    if !overlay_visible && activated_for > Duration::from_millis(500) {
        ACTIVE.store(false, Ordering::Relaxed);
        CURRENT_DIR.store(-1, Ordering::Relaxed);
        *ACTIVATED_AT.lock().unwrap_or_else(|poison| poison.into_inner()) = None;
        *TAB_DOWN.lock().unwrap_or_else(|poison| poison.into_inner()) = None;
        return;
    }
    track_cursor(app);
}

#[cfg(target_os = "windows")]
fn activate(app: &AppHandle) {
    let Some(window) = app.get_webview_window(OVERLAY_LABEL) else { return };
    let Ok(scale) = window.scale_factor() else { return };
    let Some((x, y)) = cursor_pos() else { return };
    let (left, top, right, bottom) = crate::monitor_bounds(x, y);
    let width = (right - left).max(1) as u32;
    let height = (bottom - top).max(1) as u32;
    // 原点固定为该显示器的屏幕中心（罗盘就显示在那里），鼠标位置只用于决定覆盖哪个显示器
    let center_x = (left + right) / 2;
    let center_y = (top + bottom) / 2;
    // 先置状态，再操作窗口：若期间用户已松开，keyup 也会走正常的结束流程
    ACTIVE.store(true, Ordering::Relaxed);
    CURRENT_DIR.store(-1, Ordering::Relaxed);
    *ACTIVATED_AT.lock().unwrap_or_else(|poison| poison.into_inner()) = Some(Instant::now());
    *ORIGIN.lock().unwrap_or_else(|poison| poison.into_inner()) = (center_x, center_y);
    *SCALE.lock().unwrap_or_else(|poison| poison.into_inner()) = scale;
    let _ = window.set_position(PhysicalPosition::new(left, top));
    let _ = window.set_size(PhysicalSize::new(width, height));
    let _ = window.set_ignore_cursor_events(true);
    if let Ok(hwnd) = window.hwnd() {
        crate::set_no_activate(hwnd.0, true);
        crate::raise_topmost(hwnd.0);
    }
    let _ = window.show();
    let _ = app.emit("compass-mode-open", ());
}

#[cfg(target_os = "windows")]
fn track_cursor(app: &AppHandle) {
    let Some((x, y)) = cursor_pos() else { return };
    let (ox, oy) = *ORIGIN.lock().unwrap_or_else(|poison| poison.into_inner());
    let scale = *SCALE.lock().unwrap_or_else(|poison| poison.into_inner());
    let dead_zone = BASE_DEAD_ZONE * scale;
    let next = compass_direction(x - ox, y - oy, CURRENT_DIR.load(Ordering::Relaxed), dead_zone);
    if next != CURRENT_DIR.load(Ordering::Relaxed) {
        CURRENT_DIR.store(next, Ordering::Relaxed);
        let _ = app.emit("compass-mode-dir", next);
    }
}

fn finalize_release() {
    let dir = CURRENT_DIR.load(Ordering::Relaxed);
    ACTIVE.store(false, Ordering::Relaxed);
    CURRENT_DIR.store(-1, Ordering::Relaxed);
    TAB_HELD.store(false, Ordering::Relaxed);
    *ACTIVATED_AT.lock().unwrap_or_else(|poison| poison.into_inner()) = None;
    *TAB_DOWN.lock().unwrap_or_else(|poison| poison.into_inner()) = None;
    *CLOSING_AT.lock().unwrap_or_else(|poison| poison.into_inner()) = Some(Instant::now() + HIDE_DELAY);
    if let Some(app) = APP.get() {
        let _ = app.emit("compass-mode-release", dir);
    }
}

#[cfg(target_os = "windows")]
fn cursor_pos() -> Option<(i32, i32)> {
    use windows_sys::Win32::Foundation::POINT;
    use windows_sys::Win32::UI::WindowsAndMessaging::GetCursorPos;
    let mut point = POINT { x: 0, y: 0 };
    unsafe { (GetCursorPos(&mut point) != 0).then_some((point.x, point.y)) }
}

#[cfg(target_os = "windows")]
fn key_down(vk: u16) -> bool {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;
    unsafe { (GetAsyncKeyState(vk as i32) as u16) & 0x8000 != 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEAD: f64 = 42.0;

    #[test]
    fn dead_zone_returns_none() {
        assert_eq!(compass_direction(10, 10, -1, DEAD), -1);
        assert_eq!(compass_direction(41, 0, -1, DEAD), -1);
        assert_eq!(compass_direction(-25, -25, -1, DEAD), -1);
    }

    #[test]
    fn all_eight_directions() {
        assert_eq!(compass_direction(0, -60, -1, DEAD), 0); // 上
        assert_eq!(compass_direction(45, -45, -1, DEAD), 1); // 右上
        assert_eq!(compass_direction(60, 0, -1, DEAD), 2); // 右
        assert_eq!(compass_direction(45, 45, -1, DEAD), 3); // 右下
        assert_eq!(compass_direction(0, 60, -1, DEAD), 4); // 下
        assert_eq!(compass_direction(-45, 45, -1, DEAD), 5); // 左下
        assert_eq!(compass_direction(-60, 0, -1, DEAD), 6); // 左
        assert_eq!(compass_direction(-45, -45, -1, DEAD), 7); // 左上
    }

    #[test]
    fn keeps_current_until_clearly_in_neighbor() {
        // 当前上（中心 0°），角度 20°（右上侧但未过半）→ 保持上
        assert_eq!(compass_direction(34, -93, 0, DEAD), 0);
        // 角度 28°（过半进入右上）→ 切换右上
        assert_eq!(compass_direction(48, -90, 0, DEAD), 1);
    }

    #[test]
    fn hysteresis_wraps_across_zero() {
        // 当前左上（中心 315°），角度 340° → 距离 25° > 22.5° → 切换上
        assert_eq!(compass_direction(-30, -84, 7, DEAD), 0);
        // 当前左上，角度 330° → 距离 15° ≤ 22.5° → 保持左上
        assert_eq!(compass_direction(-50, -87, 7, DEAD), 7);
    }

    #[test]
    fn returning_to_origin_cancels() {
        assert_eq!(compass_direction(60, 60, 3, DEAD), 3);
        assert_eq!(compass_direction(5, 5, 3, DEAD), -1);
    }

    #[test]
    fn hijack_applies_only_to_real_tab_without_system_modifiers() {
        assert!(should_hijack(true, false, false));
        assert!(!should_hijack(false, false, false)); // 选项关闭
        assert!(!should_hijack(true, true, false)); // 注入事件（避免自转发死循环）
        assert!(!should_hijack(true, false, true)); // Alt+Tab / Win+Tab 放行
    }
}
