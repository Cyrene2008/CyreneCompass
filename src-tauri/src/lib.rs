use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream};
use std::process::Command;
use serde::{Deserialize, Serialize};
use base64::Engine;
use std::sync::{Mutex, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::Duration;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{Emitter, Manager, PhysicalPosition, PhysicalSize};

const INSTANCE_PORT: u16 = 47682;
const INSTANCE_PING: &[u8] = b"CYRENE_COMPASS_SHOW\n";
const INSTANCE_ACK: &[u8] = b"CYRENE_COMPASS_ACK\n";
static READY: AtomicBool = AtomicBool::new(false);
static BALL_ANCHOR: Mutex<Option<(i32, i32)>> = Mutex::new(None);

#[cfg(target_os = "windows")]
fn hidden_command(program: &str) -> Command {
    use std::os::windows::process::CommandExt;
    let mut command = Command::new(program);
    command.creation_flags(windows_sys::Win32::System::Threading::CREATE_NO_WINDOW);
    command
}

#[derive(Debug, Deserialize, Serialize)]
struct PathMetadata {
    path: String,
    target: String,
    resolved_target: Option<String>,
    label: String,
    kind: String,
    icon_data_url: Option<String>,
}

fn addr() -> SocketAddrV4 { SocketAddrV4::new(Ipv4Addr::LOCALHOST, INSTANCE_PORT) }
fn ping_existing() -> bool {
    let Ok(mut stream) = TcpStream::connect_timeout(&addr().into(), Duration::from_millis(250)) else { return false };
    let _ = stream.set_read_timeout(Some(Duration::from_millis(350)));
    if stream.write_all(INSTANCE_PING).is_err() { return false }
    let mut buf = [0; INSTANCE_ACK.len()]; stream.read_exact(&mut buf).is_ok() && buf == INSTANCE_ACK
}
fn start_instance_listener(app: tauri::AppHandle, listener: TcpListener) {
    thread::spawn(move || for mut stream in listener.incoming().flatten() {
        let mut buf = [0u8; 64]; if stream.read(&mut buf).is_ok() && buf.starts_with(INSTANCE_PING) { let _ = stream.write_all(INSTANCE_ACK); let _ = app.emit("compass-show-requested", ()); }
    });
}

#[tauri::command]
async fn main_window_ready() -> Result<(), String> { READY.store(true, Ordering::Release); Ok(()) }

#[tauri::command]
fn set_ball_anchor(x: i32, y: i32) {
    if let Ok(mut anchor) = BALL_ANCHOR.lock() { *anchor = Some((x, y)); }
}

#[tauri::command]
fn restore_ball_position(app: tauri::AppHandle, x: i32, y: i32) -> Result<(), String> {
    let win = app.get_webview_window("main").ok_or("主窗口不存在")?;
    let size = win.outer_size().map_err(|e| e.to_string())?;
    let center_x = x + size.width as i32 / 2;
    let center_y = y + size.height as i32 / 2;
    let (left, top, right, bottom) = monitor_work_area(center_x, center_y);
    let target_x = x.clamp(left, (right - size.width as i32).max(left));
    let target_y = y.clamp(top, (bottom - size.height as i32).max(top));
    win.set_position(PhysicalPosition::new(target_x, target_y)).map_err(|e| e.to_string())
}

#[tauri::command]
async fn set_window_mode(app: tauri::AppHandle, mode: String, opacity: f64, width: Option<f64>, height: Option<f64>, animate: Option<bool>) -> Result<(), String> {
    let win = app.get_webview_window("main").ok_or("主窗口不存在")?;
    let (default_w, default_h) = match mode.as_str() { "compass" => (560.0, 630.0), "settings" => (1120.0, 760.0), _ => (88.0, 88.0) };
    let w = width.unwrap_or(default_w).max(44.0);
    let h = height.unwrap_or(default_h).max(44.0);
    let old_size = win.outer_size().map_err(|e| e.to_string())?;
    let old_position = win.outer_position().map_err(|e| e.to_string())?;
    let scale = win.scale_factor().map_err(|e| e.to_string())?;
    let center_x = old_position.x as f64 + old_size.width as f64 / 2.0;
    let center_y = old_position.y as f64 + old_size.height as f64 / 2.0;
    if mode != "ball" && old_size.width <= 200 && old_size.height <= 200 {
        if let Ok(mut anchor) = BALL_ANCHOR.lock() { *anchor = Some((center_x.round() as i32, center_y.round() as i32)); }
    }
    let anchor = if mode == "ball" { BALL_ANCHOR.lock().ok().and_then(|value| *value) } else { None };
    let target_center_x = anchor.map(|value| value.0 as f64).unwrap_or(center_x);
    let target_center_y = anchor.map(|value| value.1 as f64).unwrap_or(center_y);
    let requested_w = (w * scale).round().max(44.0) as i32;
    let requested_h = (h * scale).round().max(44.0) as i32;
    let (work_left, work_top, work_right, work_bottom) = monitor_work_area(target_center_x.round() as i32, target_center_y.round() as i32);
    let physical_w = requested_w.min((work_right - work_left).max(44));
    let physical_h = requested_h.min((work_bottom - work_top).max(44));
    let centered_x = (target_center_x - physical_w as f64 / 2.0).round() as i32;
    let centered_y = (target_center_y - physical_h as f64 / 2.0).round() as i32;
    let new_position = PhysicalPosition::new(
        centered_x.clamp(work_left, (work_right - physical_w).max(work_left)),
        centered_y.clamp(work_top, (work_bottom - physical_h).max(work_top)),
    );
    let should_animate = animate.unwrap_or(true);
    if should_animate {
        let start_w = old_size.width.max(44) as f64;
        let start_h = old_size.height.max(44) as f64;
        let target_w = physical_w.max(44) as f64;
        let target_h = physical_h.max(44) as f64;
        let start_x = old_position.x as f64;
        let start_y = old_position.y as f64;
        let target_x = new_position.x as f64;
        let target_y = new_position.y as f64;
        for frame in 1..=10 {
            let t = frame as f64 / 10.0;
            let eased = 1.0 - (1.0 - t).powi(3);
            let frame_w = (start_w + (target_w - start_w) * eased).round().max(44.0) as u32;
            let frame_h = (start_h + (target_h - start_h) * eased).round().max(44.0) as u32;
            let frame_x = (start_x + (target_x - start_x) * eased).round() as i32;
            let frame_y = (start_y + (target_y - start_y) * eased).round() as i32;
            win.set_size(PhysicalSize::new(frame_w, frame_h)).map_err(|e| e.to_string())?;
            win.set_position(PhysicalPosition::new(frame_x, frame_y)).map_err(|e| e.to_string())?;
            std::thread::sleep(Duration::from_millis(8));
        }
    }
    win.set_size(PhysicalSize::new(physical_w.max(44) as u32, physical_h.max(44) as u32)).map_err(|e| e.to_string())?;
    win.set_position(new_position).map_err(|e| e.to_string())?;
    if mode == "ball" { if let Ok(mut anchor) = BALL_ANCHOR.lock() { *anchor = None; } }
    win.set_always_on_top(true).map_err(|e| e.to_string())?;
    #[cfg(target_os = "windows")]
    set_no_activate(win.hwnd().map_err(|e| e.to_string())?.0, mode != "settings");
    let _ = opacity;
    win.show().map_err(|e| e.to_string())?;
    if mode == "settings" { win.set_focus().map_err(|e| e.to_string())?; }
    Ok(())
}

#[cfg(target_os = "windows")]
fn monitor_work_area(x: i32, y: i32) -> (i32, i32, i32, i32) {
    use windows_sys::Win32::Foundation::POINT;
    use windows_sys::Win32::Graphics::Gdi::{GetMonitorInfoW, MonitorFromPoint, MONITORINFO, MONITOR_DEFAULTTONEAREST};
    unsafe {
        let monitor = MonitorFromPoint(POINT { x, y }, MONITOR_DEFAULTTONEAREST);
        let mut info = MONITORINFO { cbSize: std::mem::size_of::<MONITORINFO>() as u32, rcMonitor: std::mem::zeroed(), rcWork: std::mem::zeroed(), dwFlags: 0 };
        if monitor != std::ptr::null_mut() && GetMonitorInfoW(monitor, &mut info) != 0 {
            return (info.rcWork.left, info.rcWork.top, info.rcWork.right, info.rcWork.bottom);
        }
    }
    (0, 0, 1920, 1080)
}

#[cfg(not(target_os = "windows"))]
fn monitor_work_area(_x: i32, _y: i32) -> (i32, i32, i32, i32) { (0, 0, 1920, 1080) }

#[tauri::command]
fn system_accent() -> String {
    #[cfg(target_os = "windows")]
    {
        for (key, value_name) in [
            (r"HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Accent", "AccentColorMenu"),
            (r"HKCU\Software\Microsoft\Windows\DWM", "AccentColor"),
        ] {
            if let Ok(output) = hidden_command("reg").args(["query", key, "/v", value_name]).output() {
                let text = String::from_utf8_lossy(&output.stdout);
                if let Some(raw) = text.split_whitespace().find(|part| part.starts_with("0x")) {
                    if let Ok(value) = u32::from_str_radix(raw.trim_start_matches("0x"), 16) {
                        return format!("#{:02x}{:02x}{:02x}", value & 0xff, (value >> 8) & 0xff, (value >> 16) & 0xff);
                    }
                }
            }
        }
    }
    "#0078d4".into()
}

#[tauri::command]
fn inspect_path(path: String) -> Result<PathMetadata, String> {
    let source = std::path::PathBuf::from(&path);
    if !source.exists() { return Err("文件不存在".into()); }
    #[cfg(target_os = "windows")]
    {
        let script = r#"
$p = [Environment]::GetEnvironmentVariable('CYRENE_INSPECT_PATH')
$item = Get-Item -LiteralPath $p -ErrorAction Stop
$target = $p
$resolvedTarget = $null
$kind = if ($item.PSIsContainer) { 'folder' } else { 'file' }
$label = if ($item.PSIsContainer) { $item.Name } else { $item.BaseName }
$shellApp = New-Object -ComObject Shell.Application
try {
  $parent = $shellApp.Namespace($item.DirectoryName)
  $shellItem = $parent.ParseName($item.Name)
  if ($null -ne $shellItem) {
    $displayName = $parent.GetDetailsOf($shellItem, 0)
    $typeName = $parent.GetDetailsOf($shellItem, 2)
    if (-not [string]::IsNullOrWhiteSpace($displayName)) { $label = $displayName }
    if (-not [string]::IsNullOrWhiteSpace($typeName)) { $kind = $typeName }
  }
} catch { }
if ($item.Extension -ieq '.lnk') {
  $shell = New-Object -ComObject WScript.Shell
  $shortcut = $shell.CreateShortcut($p)
  $resolvedTarget = [string]$shortcut.TargetPath
  $kind = 'shortcut'
}
$icon = $null
try {
  Add-Type -AssemblyName System.Drawing
  Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class CyreneShellIcon {
  [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)] public struct SHFILEINFO { public IntPtr hIcon; public int iIcon; public uint dwAttributes; [MarshalAs(UnmanagedType.ByValTStr, SizeConst=260)] public string szDisplayName; [MarshalAs(UnmanagedType.ByValTStr, SizeConst=80)] public string szTypeName; }
  [DllImport("shell32.dll", CharSet=CharSet.Unicode)] public static extern IntPtr SHGetFileInfo(string path, uint attrs, ref SHFILEINFO info, uint size, uint flags);
  [DllImport("user32.dll")] public static extern bool DestroyIcon(IntPtr handle);
}

'@
  $info = New-Object CyreneShellIcon+SHFILEINFO
  $flags = 0x000000100 -bor 0x000000000
  [void][CyreneShellIcon]::SHGetFileInfo($p, 0, [ref]$info, [Runtime.InteropServices.Marshal]::SizeOf($info), $flags)
  if ($info.hIcon -ne [IntPtr]::Zero) { $ico = [Drawing.Icon]::FromHandle($info.hIcon); $ms = New-Object IO.MemoryStream; $ico.ToBitmap().Save($ms, [Drawing.Imaging.ImageFormat]::Png); $icon = 'data:image/png;base64,' + [Convert]::ToBase64String($ms.ToArray()); $ms.Dispose(); [CyreneShellIcon]::DestroyIcon($info.hIcon) | Out-Null }
} catch { }
@{ path=$p; target=$target; resolved_target=$resolvedTarget; label=$label; kind=$kind; icon_data_url=$icon } | ConvertTo-Json -Compress
"#;
        let out = hidden_command("powershell.exe")
            .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", script])
            .env("CYRENE_INSPECT_PATH", &path)
            .output().map_err(|e| e.to_string())?;
        if !out.status.success() { return Err(String::from_utf8_lossy(&out.stderr).trim().to_string()); }
        return serde_json::from_slice(&out.stdout).map_err(|e| e.to_string());
    }
    #[cfg(not(target_os = "windows"))]
    {
        Ok(PathMetadata { path: path.clone(), target: path.clone(), resolved_target: None, label: source.file_stem().and_then(|x| x.to_str()).unwrap_or("文件").to_string(), kind: "file".into(), icon_data_url: None })
    }
}

#[tauri::command]
fn read_visual_data_url(path: String) -> Result<String, String> {
    let source = std::path::PathBuf::from(&path);
    let metadata = std::fs::metadata(&source).map_err(|e| e.to_string())?;
    if !metadata.is_file() { return Err("请选择图片文件".into()); }
    if metadata.len() > 20 * 1024 * 1024 { return Err("图片最大为 20 MB".into()); }
    let extension = source.extension().and_then(|x| x.to_str()).unwrap_or("").to_ascii_lowercase();
    let mime = match extension.as_str() { "png" => "image/png", "jpg" | "jpeg" => "image/jpeg", "gif" => "image/gif", "webp" => "image/webp", "bmp" => "image/bmp", _ => return Err("不支持的图片格式".into()) };
    let bytes = std::fs::read(source).map_err(|e| e.to_string())?;
    Ok(format!("data:{};base64,{}", mime, base64::engine::general_purpose::STANDARD.encode(bytes)))
}

#[tauri::command]
async fn execute_action(target: String, elevated: bool, script: bool) -> Result<(), String> {
    if target.trim().is_empty() { return Ok(()) }
    #[cfg(target_os = "windows")]
    {
        if script && elevated {
            use std::os::windows::ffi::OsStrExt;
            use windows_sys::Win32::UI::Shell::ShellExecuteW;
            use windows_sys::Win32::UI::WindowsAndMessaging::SW_HIDE;
            let params = format!("-NoProfile -ExecutionPolicy Bypass -Command \"{}\"", target.replace('"', "\\\""));
            let wide = |value: &std::ffi::OsStr| value.encode_wide().chain(Some(0)).collect::<Vec<u16>>();
            let verb = wide(std::ffi::OsStr::new("runas")); let file = wide(std::ffi::OsStr::new("pwsh.exe")); let args = wide(std::ffi::OsStr::new(&params));
            let result = unsafe { ShellExecuteW(std::ptr::null_mut(), verb.as_ptr(), file.as_ptr(), args.as_ptr(), std::ptr::null(), SW_HIDE) } as isize;
            if result <= 32 { return Err(format!("管理员 PowerShell 启动失败：{}", result)); }
        } else if script {
            hidden_command("pwsh.exe").args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &target]).spawn().map_err(|e| e.to_string())?;
        } else if elevated {
            use std::os::windows::ffi::OsStrExt;
            use windows_sys::Win32::UI::Shell::ShellExecuteW;
            use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
            let file: Vec<u16> = std::ffi::OsStr::new(&target).encode_wide().chain(Some(0)).collect(); let verb: Vec<u16> = std::ffi::OsStr::new("runas").encode_wide().chain(Some(0)).collect();
            let result = unsafe { ShellExecuteW(std::ptr::null_mut(), verb.as_ptr(), file.as_ptr(), std::ptr::null(), std::ptr::null(), SW_SHOWNORMAL) } as isize; if result <= 32 { return Err(format!("管理员启动失败：{}", result)) }
        } else {
            use std::os::windows::ffi::OsStrExt;
            use windows_sys::Win32::UI::Shell::ShellExecuteW;
            use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
            let wide = |value: &std::ffi::OsStr| value.encode_wide().chain(Some(0)).collect::<Vec<u16>>();
            let verb = wide(std::ffi::OsStr::new("open"));
            let file = wide(std::ffi::OsStr::new(&target));
            let result = unsafe { ShellExecuteW(std::ptr::null_mut(), verb.as_ptr(), file.as_ptr(), std::ptr::null(), std::ptr::null(), SW_SHOWNORMAL) } as isize;
            if result <= 32 { return Err(format!("无法打开文件或 URI：{}", result)); }
        }
        return Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    { Command::new("sh").args(["-c", &target]).spawn().map_err(|e| e.to_string())?; Ok(()) }
}

#[tauri::command]
fn configure_startup_direct(enabled: bool, mode: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let exe = std::env::current_exe().map_err(|e| e.to_string())?; let action = format!("\"{}\" --cyrene-autostart", exe.to_string_lossy());
        let output = if mode == "scheduled" {
            if enabled { hidden_command("schtasks").args(["/Create", "/TN", "CyreneCompassAutoStart", "/TR", &action, "/SC", "ONLOGON", "/RL", "HIGHEST", "/F"]).output() }
            else { hidden_command("schtasks").args(["/Delete", "/TN", "CyreneCompassAutoStart", "/F"]).output() }
        } else if enabled { hidden_command("reg").args(["add", r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run", "/v", "CyreneCompass", "/t", "REG_SZ", "/d", &action, "/f"]).output() }
        else { hidden_command("reg").args(["delete", r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run", "/v", "CyreneCompass", "/f"]).output() };
        let output = output.map_err(|e| e.to_string())?;
        if !output.status.success() { return Err(String::from_utf8_lossy(if output.stderr.is_empty() { &output.stdout } else { &output.stderr }).trim().to_string()); }
        return Ok(())
    }
    #[cfg(not(target_os = "windows"))] { let _ = (enabled, mode); Err("仅支持 Windows".into()) }
}

#[tauri::command]
fn configure_startup(enabled: bool, mode: String) -> Result<serde_json::Value, String> {
    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::UI::Shell::IsUserAnAdmin;
        if mode == "scheduled" && unsafe { IsUserAnAdmin() } == 0 {
            use std::os::windows::ffi::OsStrExt;
            use windows_sys::Win32::UI::Shell::ShellExecuteW;
            use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
            let exe = std::env::current_exe().map_err(|e| e.to_string())?;
            let params = format!("--cyrene-configure-startup={} --cyrene-startup-mode={}", if enabled { "enable" } else { "disable" }, mode);
            let wide = |value: &std::ffi::OsStr| value.encode_wide().chain(Some(0)).collect::<Vec<u16>>();
            let verb = wide(std::ffi::OsStr::new("runas")); let file = wide(exe.as_os_str()); let args = wide(std::ffi::OsStr::new(&params));
            let result = unsafe { ShellExecuteW(std::ptr::null_mut(), verb.as_ptr(), file.as_ptr(), args.as_ptr(), std::ptr::null(), SW_SHOWNORMAL) } as isize;
            if result <= 32 { return Err(format!("UAC 启动失败：{}", result)); }
            return Ok(serde_json::json!({ "success": true, "elevatedHelper": true }));
        }
    }
    configure_startup_direct(enabled, &mode)?;
    Ok(serde_json::json!({ "success": true, "elevatedHelper": false }))
}

#[tauri::command]
fn quit_app(app: tauri::AppHandle) { app.exit(0); }

#[cfg(target_os = "windows")]
fn set_no_activate(hwnd: windows_sys::Win32::Foundation::HWND, enabled: bool) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{GetWindowLongW, SetWindowLongW, SetWindowPos, GWL_EXSTYLE, HWND_TOPMOST, SWP_FRAMECHANGED, SWP_NOMOVE, SWP_NOACTIVATE, SWP_NOSIZE, SWP_SHOWWINDOW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW};
    unsafe {
        let style = GetWindowLongW(hwnd, GWL_EXSTYLE);
        let next = if enabled { style | WS_EX_NOACTIVATE as i32 | WS_EX_TOOLWINDOW as i32 } else { (style & !(WS_EX_NOACTIVATE as i32)) | WS_EX_TOOLWINDOW as i32 };
        SetWindowLongW(hwnd, GWL_EXSTYLE, next);
        let flags = SWP_NOMOVE | SWP_NOSIZE | SWP_FRAMECHANGED | SWP_SHOWWINDOW | if enabled { SWP_NOACTIVATE } else { 0 };
        SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, flags);
    }
}

fn create_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let settings = MenuItem::with_id(app, "settings", "打开设置", true, None::<&str>)?; let menu = Menu::with_items(app, &[&settings])?;
    TrayIconBuilder::with_id("compass-tray").icon(app.default_window_icon().cloned().unwrap()).tooltip("Cyreneの罗盘").menu(&menu).show_menu_on_left_click(false).on_tray_icon_event(|tray, event| { if let tauri::tray::TrayIconEvent::Click { button: tauri::tray::MouseButton::Left, button_state: tauri::tray::MouseButtonState::Up, .. } = event { let _ = tray.app_handle().emit("compass-show-requested", ()); } }).on_menu_event(|app, event| { if event.id.as_ref() == "settings" { let _ = app.emit("compass-settings-requested", ()); } }).build(app)?; Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let args = std::env::args().collect::<Vec<_>>();
    if let Some(value) = args.iter().find_map(|x| x.strip_prefix("--cyrene-configure-startup=")) {
        let mode = args.iter().find_map(|x| x.strip_prefix("--cyrene-startup-mode=")).unwrap_or("scheduled");
        let _ = configure_startup_direct(value == "enable", mode);
        return;
    }
    if ping_existing() { std::process::exit(0); }
    let listener = TcpListener::bind(addr()).expect("无法创建单实例 IPC");
    tauri::Builder::default().plugin(tauri_plugin_dialog::init()).setup(move |app| {
        start_instance_listener(app.handle().clone(), listener);
        create_tray(app)?;
        #[cfg(target_os = "windows")]
        if let Some(window) = app.get_webview_window("main") {
            if let Ok(hwnd) = window.hwnd() { set_no_activate(hwnd.0, true); }
        }
        Ok(())
    }).invoke_handler(tauri::generate_handler![main_window_ready, set_ball_anchor, restore_ball_position, set_window_mode, system_accent, inspect_path, read_visual_data_url, execute_action, configure_startup, quit_app]).run(tauri::generate_context!()).expect("error while running Cyrene Compass");
}
