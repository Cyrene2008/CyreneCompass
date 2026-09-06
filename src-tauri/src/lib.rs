use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::fs;
use serde::{Deserialize, Serialize};
use base64::Engine;
use std::sync::{Mutex, atomic::{AtomicBool, AtomicIsize, Ordering}};
use std::thread;
use std::time::Duration;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{Emitter, Manager, PhysicalPosition, PhysicalSize};

mod compass_mode;

const INSTANCE_PORT: u16 = 47682;
const INSTANCE_PING: &[u8] = b"CYRENE_COMPASS_SHOW\n";
const INSTANCE_ACK: &[u8] = b"CYRENE_COMPASS_ACK\n";
static READY: AtomicBool = AtomicBool::new(false);
static TOPMOST_HWND: AtomicIsize = AtomicIsize::new(0);
static BALL_ANCHOR: Mutex<Option<(i32, i32)>> = Mutex::new(None);
const UPDATE_URL_PREFIX: &str = "https://github.com/Cyrene2008/CyreneCompass/releases/download/";
const UPDATE_MIN_INSTALLER_SIZE: usize = 1024 * 1024;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct WorkArea {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct RestoredWindowPosition {
    x: i32,
    y: i32,
    reset: bool,
}

fn restore_target(x: i32, y: i32, width: u32, height: u32, candidate: Option<WorkArea>, primary: WorkArea) -> RestoredWindowPosition {
    let fits = candidate.is_some_and(|area| {
        let right = i64::from(x) + i64::from(width);
        let bottom = i64::from(y) + i64::from(height);
        i64::from(x) >= i64::from(area.left)
            && i64::from(y) >= i64::from(area.top)
            && right <= i64::from(area.right)
            && bottom <= i64::from(area.bottom)
    });
    if fits {
        return RestoredWindowPosition { x, y, reset: false };
    }

    let work_width = i64::from(primary.right) - i64::from(primary.left);
    let work_height = i64::from(primary.bottom) - i64::from(primary.top);
    let centered_x = i64::from(primary.left) + (work_width - i64::from(width)).max(0) / 2;
    let centered_y = i64::from(primary.top) + (work_height - i64::from(height)).max(0) / 2;
    RestoredWindowPosition {
        x: centered_x.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32,
        y: centered_y.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32,
        reset: true,
    }
}

fn clamp_target(x: i32, y: i32, width: u32, height: u32, area: WorkArea) -> RestoredWindowPosition {
    let max_x = (i64::from(area.right) - i64::from(width)).max(i64::from(area.left));
    let max_y = (i64::from(area.bottom) - i64::from(height)).max(i64::from(area.top));
    RestoredWindowPosition {
        x: i64::from(x).clamp(i64::from(area.left), max_x) as i32,
        y: i64::from(y).clamp(i64::from(area.top), max_y) as i32,
        reset: false,
    }
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
fn main_window_ready() -> Result<(), String> { READY.store(true, Ordering::Release); Ok(()) }

fn installation_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|dir| dir.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

fn config_dir() -> PathBuf { installation_dir().join("data") }

fn config_path() -> PathBuf { config_dir().join("cyrene-compass.json") }

#[tauri::command]
fn load_settings() -> Result<Option<serde_json::Value>, String> {
    let path = config_path();
    match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).map(Some).map_err(|error| format!("配置文件解析错误：{error}")),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!("无法读取配置文件：{error}")),
    }
}

#[tauri::command]
fn save_settings(value: serde_json::Value) -> Result<(), String> {
    let path = config_path();
    fs::create_dir_all(path.parent().ok_or("无效的配置目录")?).map_err(|error| format!("无法创建配置目录：{error}"))?;
    let content = serde_json::to_string_pretty(&value).map_err(|error| format!("配置文件序列化失败：{error}"))?;
    fs::write(&path, content).map_err(|error| format!("无法写入配置文件：{error}"))?;
    Ok(())
}

#[tauri::command]
fn set_ball_anchor(x: i32, y: i32) {
    if let Ok(mut anchor) = BALL_ANCHOR.lock() { *anchor = Some((x, y)); }
}

#[tauri::command]
fn restore_ball_position(app: tauri::AppHandle, x: i32, y: i32) -> Result<RestoredWindowPosition, String> {
    let win = app.get_webview_window("main").ok_or("主窗口不存在")?;
    let size = win.outer_size().map_err(|e| e.to_string())?;
    let target = restore_target(x, y, size.width, size.height, monitor_work_area_for_rect(x, y, size.width, size.height), primary_monitor_work_area());
    win.set_position(PhysicalPosition::new(target.x, target.y)).map_err(|e| e.to_string())?;
    Ok(target)
}

#[tauri::command]
fn move_window_clamped(app: tauri::AppHandle, x: i32, y: i32) -> Result<RestoredWindowPosition, String> {
    let win = app.get_webview_window("main").ok_or("主窗口不存在")?;
    let size = win.outer_size().map_err(|e| e.to_string())?;
    let center_x = i64::from(x) + i64::from(size.width) / 2;
    let center_y = i64::from(y) + i64::from(size.height) / 2;
    let (left, top, right, bottom) = monitor_bounds(
        center_x.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32,
        center_y.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32,
    );
    let target = clamp_target(x, y, size.width, size.height, WorkArea { left, top, right, bottom });
    win.set_position(PhysicalPosition::new(target.x, target.y)).map_err(|e| e.to_string())?;
    Ok(target)
}

#[tauri::command]
async fn check_update() -> Result<serde_json::Value, String> {
    let urls = [
        "https://api.github.com/repos/Cyrene2008/CyreneCompass/releases/latest",
        "https://api.kkgithub.com/repos/Cyrene2008/CyreneCompass/releases/latest",
    ];
    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(8))
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|error| error.to_string())?;
    for url in urls {
        if let Ok(response) = client
            .get(url)
            .header("User-Agent", "CyreneCompass")
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await
        {
            if response.status().is_success() {
                if let Ok(release) = response.json::<serde_json::Value>().await { return Ok(release); }
            }
        }
    }
    Err("无法连接到更新服务器".into())
}

fn update_installer_path(file_name: &str) -> Result<PathBuf, String> {
    let safe_name = Path::new(file_name).file_name().and_then(|value| value.to_str()).ok_or("安装包文件名无效")?;
    let official_name = safe_name.starts_with("CyreneCompass_") && safe_name.ends_with("_x64-setup.exe");
    if safe_name != file_name || !official_name {
        return Err("安装包文件名不属于 CyreneCompass 官方格式".into());
    }
    let directory = std::env::temp_dir().join("CyreneCompass").join("updates");
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    Ok(directory.join(safe_name))
}

async fn download_update_bytes(app: &tauri::AppHandle, url: &str, expected_size: u64) -> Result<Vec<u8>, String> {
    if !url.starts_with(UPDATE_URL_PREFIX) { return Err("更新地址不属于 CyreneCompass 官方发布源".into()); }
    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(6))
        .timeout(Duration::from_secs(120))
        .build()
        .map_err(|error| error.to_string())?;
    let candidates = [
        url.to_string(),
        format!("https://v4.gh-proxy.com/{}", url),
        format!("https://gh.昔涟.cn/{}", url),
        format!("https://ghproxy.net/{}", url),
        format!("https://ghfast.top/{}", url),
    ];
    let display_url = |value: &str| value.replace("https://v4.gh-proxy.com/", "https://gh-proxy.com/");
    let mut failures = Vec::new();
    for candidate in candidates {
        match client.get(&candidate).header("User-Agent", "CyreneCompass").header("Accept", "application/octet-stream").send().await {
            Ok(mut response) if response.status().is_success() => {
                let content_type = response.headers().get(reqwest::header::CONTENT_TYPE).and_then(|value| value.to_str().ok()).unwrap_or("").to_ascii_lowercase();
                if content_type.contains("text/html") || content_type.contains("application/json") {
                    failures.push(format!("{} 返回了非安装程序内容", display_url(&candidate)));
                    continue;
                }
                let total = if expected_size > 0 { expected_size } else { response.content_length().unwrap_or(0) };
                let mut bytes = Vec::with_capacity(total.min(usize::MAX as u64) as usize);
                let mut last_progress = 0u8;
                let _ = app.emit("update-download-progress", 0u8);
                loop {
                    match response.chunk().await {
                        Ok(Some(chunk)) => {
                            bytes.extend_from_slice(&chunk);
                            if total > 0 {
                                let progress = ((bytes.len() as u64 * 99) / total).min(99) as u8;
                                if progress > last_progress { last_progress = progress; let _ = app.emit("update-download-progress", progress); }
                            }
                        }
                        Ok(None) => break,
                        Err(error) => { failures.push(format!("{} 下载中断：{}", display_url(&candidate), error)); bytes.clear(); break; }
                    }
                }
                if bytes.len() < UPDATE_MIN_INSTALLER_SIZE { failures.push(format!("{} 安装包体积异常", candidate)); continue; }
                if expected_size > 0 && bytes.len() as u64 != expected_size { failures.push(format!("{} 安装包大小校验失败", candidate)); continue; }
                if !bytes.starts_with(b"MZ") { failures.push(format!("{} 不是有效的 Windows 安装程序", candidate)); continue; }
                let _ = app.emit("update-download-progress", 100u8);
                return Ok(bytes);
            }
            Ok(response) => failures.push(format!("{} 返回 HTTP {}", candidate, response.status())),
            Err(error) => failures.push(format!("{}：{}", candidate, error)),
        }
    }
    Err(format!("更新下载失败：{}", failures.join("；")))
}

#[tauri::command]
async fn download_and_launch_update(app: tauri::AppHandle, url: String, file_name: String, expected_size: u64) -> Result<serde_json::Value, String> {
    let path = update_installer_path(&file_name)?;
    let bytes = download_update_bytes(&app, &url, expected_size).await?;
    let partial = path.with_extension("exe.part");
    let _ = fs::remove_file(&partial);
    fs::write(&partial, &bytes).map_err(|error| error.to_string())?;
    let _ = fs::remove_file(&path);
    fs::rename(&partial, &path).map_err(|error| error.to_string())?;
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::UI::Shell::ShellExecuteW;
        use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
        let wide = |value: &std::ffi::OsStr| value.encode_wide().chain(Some(0)).collect::<Vec<u16>>();
        let verb = wide(std::ffi::OsStr::new("open"));
        let file = wide(path.as_os_str());
        let result = unsafe { ShellExecuteW(std::ptr::null_mut(), verb.as_ptr(), file.as_ptr(), std::ptr::null(), std::ptr::null(), SW_SHOWNORMAL) } as isize;
        if result <= 32 { return Err(format!("无法启动更新安装程序：{}", result)); }
    }
    let exit_handle = app.clone();
    thread::spawn(move || { thread::sleep(Duration::from_millis(1500)); exit_handle.exit(0); });
    Ok(serde_json::json!({ "success": true, "filePath": path.to_string_lossy(), "size": bytes.len() }))
}

#[tauri::command]
fn open_external(url: String) -> Result<(), String> {
    let repository = "https://github.com/Cyrene2008/CyreneCompass";
    let repository_url = url == repository || url.strip_prefix(repository).is_some_and(|suffix| suffix.starts_with('/'));
    let license_url = matches!(url.as_str(), "https://www.gnu.org/licenses/gpl-3.0.html" | "https://www.gnu.org/licenses/gpl-3.0.en.html");
    if !(repository_url || license_url) {
        return Err("不允许打开未经授权的外部地址".into());
    }
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::UI::Shell::ShellExecuteW;
        use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
        let wide = |value: &std::ffi::OsStr| value.encode_wide().chain(Some(0)).collect::<Vec<u16>>();
        let verb = wide(std::ffi::OsStr::new("open"));
        let file = wide(std::ffi::OsStr::new(&url));
        let result = unsafe { ShellExecuteW(std::ptr::null_mut(), verb.as_ptr(), file.as_ptr(), std::ptr::null(), std::ptr::null(), SW_SHOWNORMAL) } as isize;
        if result <= 32 { return Err(format!("无法打开外部链接：{}", result)); }
    }
    Ok(())
}

#[tauri::command]
async fn set_window_mode(app: tauri::AppHandle, mode: String, opacity: f64, width: Option<f64>, height: Option<f64>, animate: Option<bool>) -> Result<(), String> {
    let win = app.get_webview_window("main").ok_or("主窗口不存在")?;
    let (default_w, default_h) = match mode.as_str() { "compass" => (560.0, 630.0), "settings" => (1460.0, 760.0), _ => (88.0, 88.0) };
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
    // ponytail: 原生逐帧 resize 动画已移除（低资源机器上的 2~3s 展开延迟热点）。
    // 尺寸/位置一次到位，视觉过渡由前端 GSAP 对 .mode-surface 完成。
    let _ = animate;
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

#[cfg(target_os = "windows")]
pub(crate) fn monitor_bounds(x: i32, y: i32) -> (i32, i32, i32, i32) {
    use windows_sys::Win32::Foundation::POINT;
    use windows_sys::Win32::Graphics::Gdi::{GetMonitorInfoW, MonitorFromPoint, MONITORINFO, MONITOR_DEFAULTTONEAREST};
    unsafe {
        let monitor = MonitorFromPoint(POINT { x, y }, MONITOR_DEFAULTTONEAREST);
        let mut info = MONITORINFO { cbSize: std::mem::size_of::<MONITORINFO>() as u32, rcMonitor: std::mem::zeroed(), rcWork: std::mem::zeroed(), dwFlags: 0 };
        if monitor != std::ptr::null_mut() && GetMonitorInfoW(monitor, &mut info) != 0 {
            return (info.rcMonitor.left, info.rcMonitor.top, info.rcMonitor.right, info.rcMonitor.bottom);
        }
    }
    (0, 0, 1920, 1080)
}

#[cfg(target_os = "windows")]
fn monitor_work_area_for_rect(x: i32, y: i32, width: u32, height: u32) -> Option<WorkArea> {
    use windows_sys::Win32::Foundation::RECT;
    use windows_sys::Win32::Graphics::Gdi::{GetMonitorInfoW, MonitorFromRect, MONITORINFO, MONITOR_DEFAULTTONULL};
    let width = i32::try_from(width).unwrap_or(i32::MAX);
    let height = i32::try_from(height).unwrap_or(i32::MAX);
    let rect = RECT { left: x, top: y, right: x.saturating_add(width), bottom: y.saturating_add(height) };
    unsafe {
        let monitor = MonitorFromRect(&rect, MONITOR_DEFAULTTONULL);
        let mut info = MONITORINFO { cbSize: std::mem::size_of::<MONITORINFO>() as u32, rcMonitor: std::mem::zeroed(), rcWork: std::mem::zeroed(), dwFlags: 0 };
        if monitor != std::ptr::null_mut() && GetMonitorInfoW(monitor, &mut info) != 0 {
            return Some(WorkArea { left: info.rcWork.left, top: info.rcWork.top, right: info.rcWork.right, bottom: info.rcWork.bottom });
        }
    }
    None
}

#[cfg(target_os = "windows")]
fn primary_monitor_work_area() -> WorkArea {
    use windows_sys::Win32::Foundation::POINT;
    use windows_sys::Win32::Graphics::Gdi::{GetMonitorInfoW, MonitorFromPoint, MONITORINFO, MONITOR_DEFAULTTOPRIMARY};
    unsafe {
        let monitor = MonitorFromPoint(POINT { x: 0, y: 0 }, MONITOR_DEFAULTTOPRIMARY);
        let mut info = MONITORINFO { cbSize: std::mem::size_of::<MONITORINFO>() as u32, rcMonitor: std::mem::zeroed(), rcWork: std::mem::zeroed(), dwFlags: 0 };
        if monitor != std::ptr::null_mut() && GetMonitorInfoW(monitor, &mut info) != 0 {
            return WorkArea { left: info.rcWork.left, top: info.rcWork.top, right: info.rcWork.right, bottom: info.rcWork.bottom };
        }
    }
    WorkArea { left: 0, top: 0, right: 1920, bottom: 1080 }
}

#[cfg(not(target_os = "windows"))]
fn monitor_work_area(_x: i32, _y: i32) -> (i32, i32, i32, i32) { (0, 0, 1920, 1080) }

#[cfg(not(target_os = "windows"))]
fn monitor_bounds(_x: i32, _y: i32) -> (i32, i32, i32, i32) { (0, 0, 1920, 1080) }

#[cfg(not(target_os = "windows"))]
fn monitor_work_area_for_rect(_x: i32, _y: i32, _width: u32, _height: u32) -> Option<WorkArea> { Some(primary_monitor_work_area()) }

#[cfg(not(target_os = "windows"))]
fn primary_monitor_work_area() -> WorkArea { WorkArea { left: 0, top: 0, right: 1920, bottom: 1080 } }

#[cfg(test)]
mod position_tests {
    use super::{clamp_target, restore_target, WorkArea};

    const PRIMARY: WorkArea = WorkArea { left: 0, top: 0, right: 1920, bottom: 1040 };
    const DISPLAY: WorkArea = WorkArea { left: 0, top: 0, right: 1920, bottom: 1080 };

    #[test]
    fn keeps_a_valid_position() {
        let result = restore_target(100, 200, 88, 88, Some(PRIMARY), PRIMARY);
        assert_eq!((result.x, result.y, result.reset), (100, 200, false));
    }

    #[test]
    fn centers_a_position_outside_every_screen() {
        let result = restore_target(4000, 3000, 88, 88, None, PRIMARY);
        assert_eq!((result.x, result.y, result.reset), (916, 476, true));
    }

    #[test]
    fn centers_a_position_overlapping_the_taskbar() {
        let result = restore_target(100, 1000, 88, 88, Some(PRIMARY), PRIMARY);
        assert_eq!((result.x, result.y, result.reset), (916, 476, true));
    }

    #[test]
    fn accepts_a_valid_negative_coordinate_monitor() {
        let secondary = WorkArea { left: -1280, top: 0, right: 0, bottom: 984 };
        let result = restore_target(-1200, 100, 88, 88, Some(secondary), PRIMARY);
        assert_eq!((result.x, result.y, result.reset), (-1200, 100, false));
    }


    #[test]
    fn clamps_dragged_windows_inside_the_work_area() {
        let left = clamp_target(-40, 100, 88, 88, PRIMARY);
        let bottom = clamp_target(1900, 1030, 88, 88, PRIMARY);
        assert_eq!((left.x, left.y), (0, 100));
        assert_eq!((bottom.x, bottom.y), (1832, 952));
    }

    #[test]
    fn allows_dragging_over_the_taskbar_area() {
        let target = clamp_target(1900, 1060, 88, 88, DISPLAY);
        assert_eq!((target.x, target.y), (1832, 992));
    }

    #[test]
    fn extreme_coordinates_do_not_overflow() {
        let result = restore_target(i32::MAX, i32::MIN, u32::MAX, u32::MAX, None, PRIMARY);
        assert_eq!((result.x, result.y, result.reset), (0, 0, true));
    }
}

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
[Console]::OutputEncoding = [Text.UTF8Encoding]::new($false)
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

#[cfg(target_os = "windows")]
fn shell_open_file(target: &str) -> Result<(), i32> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::UI::Shell::ShellExecuteW;
    use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
    let wide = |value: &std::ffi::OsStr| value.encode_wide().chain(Some(0)).collect::<Vec<u16>>();
    let verb = wide(std::ffi::OsStr::new("open"));
    let file = wide(std::ffi::OsStr::new(target));
    let result = unsafe { ShellExecuteW(std::ptr::null_mut(), verb.as_ptr(), file.as_ptr(), std::ptr::null(), std::ptr::null(), SW_SHOWNORMAL) } as isize;
    if result <= 32 { Err(result as i32) } else { Ok(()) }
}

#[cfg(target_os = "windows")]
fn shell_runas(file: &str, params: &str) -> Result<(), i32> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::UI::Shell::ShellExecuteW;
    use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
    let wide = |value: &std::ffi::OsStr| value.encode_wide().chain(Some(0)).collect::<Vec<u16>>();
    let verb = wide(std::ffi::OsStr::new("runas"));
    let f = wide(std::ffi::OsStr::new(file));
    let p = wide(std::ffi::OsStr::new(params));
    let result = unsafe { ShellExecuteW(std::ptr::null_mut(), verb.as_ptr(), f.as_ptr(), p.as_ptr(), std::ptr::null(), SW_SHOWNORMAL) } as isize;
    if result <= 32 { Err(result as i32) } else { Ok(()) }
}

#[cfg(target_os = "windows")]
fn expand_env_vars(input: &str) -> String {
    let mut output = String::new();
    let mut rest = input;
    while let Some(start) = rest.find('%') {
        output.push_str(&rest[..start]);
        let tail = &rest[start + 1..];
        let Some((name, after)) = tail.split_once('%').filter(|(name, _)| !name.is_empty()) else {
            output.push('%');
            rest = tail;
            continue;
        };
        match std::env::var(name) {
            Ok(value) => output.push_str(&value),
            Err(_) => { output.push('%'); output.push_str(name); output.push('%'); }
        }
        rest = after;
    }
    output.push_str(rest);
    output
}

#[cfg(target_os = "windows")]
fn split_command_line(input: &str) -> (String, Vec<String>) {
    let expanded = expand_env_vars(input.trim());
    let mut program = String::new();
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_program = true;
    let mut quoted = false;
    for c in expanded.chars() {
        match c {
            '"' => quoted = !quoted,
            ' ' | '\t' if !quoted => {
                if in_program {
                    if !current.is_empty() { program = std::mem::take(&mut current); in_program = false; }
                } else if !current.is_empty() {
                    args.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(c),
        }
    }
    if in_program { program = current } else if !current.is_empty() { args.push(current); }
    (program, args)
}

#[cfg(target_os = "windows")]
fn resolve_program(program: &str) -> Option<PathBuf> {
    let candidate = PathBuf::from(program);
    if candidate.is_file() { return Some(candidate); }
    if program.contains('/') || program.contains('\\') { return None; }
    let mut dirs: Vec<PathBuf> = Vec::new();
    if let Ok(current) = std::env::current_dir() { dirs.push(current); }
    if let Ok(path) = std::env::var("PATH") {
        dirs.extend(path.split(';').filter(|entry| !entry.is_empty()).map(PathBuf::from));
    }
    if let Ok(root) = std::env::var("SystemRoot") {
        dirs.push(PathBuf::from(&root).join("System32"));
        dirs.push(PathBuf::from(&root));
    }
    for dir in dirs {
        let base = dir.join(program);
        if base.is_file() { return Some(base.clone()); }
        for ext in ["exe", "com"] {
            let with_ext = base.with_extension(ext);
            if with_ext.is_file() { return Some(with_ext); }
        }
    }
    None
}

#[cfg(target_os = "windows")]
fn run_command_line(target: &str) -> Result<(), String> {
    let expanded = expand_env_vars(target);
    let expanded = expanded.trim();
    if expanded.is_empty() { return Ok(()); }
    let (program, args) = split_command_line(expanded);
    if program.is_empty() { return Err("无法解析命令".into()); }
    let ext = Path::new(&program).extension().and_then(|value| value.to_str()).unwrap_or("").to_ascii_lowercase();
    if ext == "bat" || ext == "cmd" {
        hidden_command("cmd.exe").args(["/S", "/C", expanded]).spawn().map_err(|e| format!("无法运行命令行：{}", e))?;
        return Ok(());
    }
    if ext == "ps1" {
        let spawn_ps1 = |exe: &str| {
            let mut command = hidden_command(exe);
            command.args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-File", &program]);
            command.args(&args);
            command.spawn()
        };
        spawn_ps1("pwsh.exe").or_else(|_| spawn_ps1("powershell.exe")).map_err(|e| format!("无法运行 PowerShell 脚本：{}", e))?;
        return Ok(());
    }
    let resolved = resolve_program(&program);
    let spawned = match &resolved {
        Some(path) => hidden_command(&path.to_string_lossy()).args(&args).spawn(),
        None => hidden_command(&program).args(&args).spawn(),
    };
    spawned.map_err(|e| format!("无法运行程序 {program}：{e}"))?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn run_command_elevated(target: &str) -> Result<(), String> {
    let trimmed = target.trim();
    let unquoted = trimmed.trim_matches('"');
    let ext = Path::new(unquoted).extension().and_then(|value| value.to_str()).unwrap_or("").to_ascii_lowercase();
    if Path::new(unquoted).is_file() {
        if ext == "bat" || ext == "cmd" {
            let line = format!("/S /C \"{}\"", unquoted.replace('"', "\"\""));
            return shell_runas("cmd.exe", &line).map_err(|r| format!("管理员启动失败：{}", r));
        }
        if ext == "ps1" {
            let exe = if resolve_program("pwsh.exe").is_some() { "pwsh.exe" } else { "powershell.exe" };
            let line = format!("-NoProfile -ExecutionPolicy Bypass -File \"{}\"", unquoted);
            return shell_runas(exe, &line).map_err(|r| format!("管理员启动失败：{}", r));
        }
        return shell_runas(unquoted, "").map_err(|r| format!("管理员启动失败：{}", r));
    }
    let (program, args) = split_command_line(trimmed);
    if program.is_empty() { return Err("无法解析命令".into()); }
    let resolved = resolve_program(&program).ok_or_else(|| format!("找不到程序 {program}，管理员模式需要完整路径"))?;
    shell_runas(&resolved.to_string_lossy(), &args.join(" ")).map_err(|r| format!("管理员启动失败：{}", r))
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
            let run_script = |program: &str| {
                let mut command = hidden_command(program);
                command.args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &target]);
                command.spawn()
            };
            run_script("pwsh.exe").or_else(|_| run_script("powershell.exe")).map_err(|e| format!("无法运行 PowerShell 脚本：{}", e))?;
        } else if elevated {
            run_command_elevated(&target)?;
        } else if let Err(shell_error) = shell_open_file(&target) {
            run_command_line(&target).map_err(|command_error| format!("无法打开文件或 URI：{}；按命令行执行也失败：{}", shell_error, command_error))?;
        }
        return Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    { Command::new("sh").args(["-c", &target]).spawn().map_err(|e| e.to_string())?; Ok(()) }
}

#[cfg(all(test, target_os = "windows"))]
mod command_tests {
    use super::{expand_env_vars, resolve_program, split_command_line};

    #[test]
    fn splits_bare_command() {
        let (program, args) = split_command_line("shutdown -s -t 0");
        assert_eq!(program, "shutdown");
        assert_eq!(args, vec!["-s", "-t", "0"]);
    }

    #[test]
    fn splits_quoted_program_with_spaces() {
        let (program, args) = split_command_line(r#""C:\Program Files\App\run.exe" --fast"#);
        assert_eq!(program, r"C:\Program Files\App\run.exe");
        assert_eq!(args, vec!["--fast"]);
    }

    #[test]
    fn expands_percent_env_vars() {
        assert_eq!(expand_env_vars("%SystemRoot%\\System32\\shutdown.exe"), r"C:\Windows\System32\shutdown.exe");
    }

    #[test]
    fn resolves_known_system_programs() {
        assert!(resolve_program("shutdown").is_some());
        assert!(resolve_program("notepad.exe").is_some());
        assert!(resolve_program("definitely-not-exists-cyrene-xyz").is_none());
    }
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
fn set_compass_mode(enabled: bool, hold_ms: u64, hijack: bool) {
    compass_mode::set_config(enabled, hold_ms, hijack);
}

#[tauri::command]
fn open_submenu_compass(app: tauri::AppHandle, item_id: String) {
    let _ = app.emit("compass-show-requested", serde_json::json!({ "itemId": item_id }));
}

#[tauri::command]
fn quit_app(app: tauri::AppHandle) { app.exit(0); }

#[cfg(target_os = "windows")]
pub(crate) fn set_no_activate(hwnd: windows_sys::Win32::Foundation::HWND, enabled: bool) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{GetWindowLongW, SetWindowLongW, SetWindowPos, GWL_EXSTYLE, HWND_TOPMOST, SWP_FRAMECHANGED, SWP_NOMOVE, SWP_NOACTIVATE, SWP_NOSIZE, SWP_SHOWWINDOW, WS_EX_APPWINDOW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW};
    unsafe {
        let style = GetWindowLongW(hwnd, GWL_EXSTYLE);
        let next = if enabled { style | WS_EX_NOACTIVATE as i32 | WS_EX_TOOLWINDOW as i32 } else { (style & !(WS_EX_NOACTIVATE as i32)) | WS_EX_TOOLWINDOW as i32 };
        let next = next & !(WS_EX_APPWINDOW as i32);
        SetWindowLongW(hwnd, GWL_EXSTYLE, next);
        let flags = SWP_NOMOVE | SWP_NOSIZE | SWP_FRAMECHANGED | SWP_SHOWWINDOW | if enabled { SWP_NOACTIVATE } else { 0 };
        SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, flags);
    }
}

#[cfg(target_os = "windows")]
pub(crate) fn raise_topmost(hwnd: windows_sys::Win32::Foundation::HWND) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{SetWindowPos, HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE};
    unsafe { SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE); }
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn foreground_changed(
    _hook: windows_sys::Win32::UI::Accessibility::HWINEVENTHOOK,
    _event: u32,
    _foreground: windows_sys::Win32::Foundation::HWND,
    _object_id: i32,
    _child_id: i32,
    _thread_id: u32,
    _event_time: u32,
) {
    let hwnd = TOPMOST_HWND.load(Ordering::Relaxed) as windows_sys::Win32::Foundation::HWND;
    if !hwnd.is_null() { raise_topmost(hwnd); }
}

#[cfg(target_os = "windows")]
fn watch_foreground_changes(hwnd: windows_sys::Win32::Foundation::HWND) {
    use windows_sys::Win32::UI::Accessibility::SetWinEventHook;
    use windows_sys::Win32::UI::WindowsAndMessaging::{EVENT_SYSTEM_FOREGROUND, WINEVENT_OUTOFCONTEXT};
    TOPMOST_HWND.store(hwnd as isize, Ordering::Relaxed);
    unsafe {
        SetWinEventHook(
            EVENT_SYSTEM_FOREGROUND,
            EVENT_SYSTEM_FOREGROUND,
            std::ptr::null_mut(),
            Some(foreground_changed),
            0,
            0,
            WINEVENT_OUTOFCONTEXT,
        );
    }
}

#[cfg(target_os = "windows")]
fn is_window_visible_iconic(hwnd: windows_sys::Win32::Foundation::HWND) -> (bool, bool) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{IsIconic, IsWindowVisible};
    unsafe {
        let visible = IsWindowVisible(hwnd) != 0;
        let iconic = IsIconic(hwnd) != 0;
        (visible, iconic)
    }
}

#[cfg(target_os = "windows")]
fn keep_window_visible(hwnd: windows_sys::Win32::Foundation::HWND) {
    // Win+D / 显示桌面 会最小化或隐藏普通置顶窗口；此处将其按非激活方式恢复。
    use windows_sys::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_SHOWNA};
    unsafe { ShowWindow(hwnd, SW_SHOWNA); }
}

#[cfg(target_os = "windows")]
fn watch_window_visibility(hwnd: windows_sys::Win32::Foundation::HWND) {
    let handle = hwnd as isize;
    thread::spawn(move || loop {
        let hwnd = handle as windows_sys::Win32::Foundation::HWND;
        if hwnd.is_null() {
            break;
        }
        let (visible, iconic) = is_window_visible_iconic(hwnd);
        if iconic || !visible {
            keep_window_visible(hwnd);
            raise_topmost(hwnd);
        }
        thread::sleep(Duration::from_millis(500));
    });
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
        compass_mode::init(app.handle().clone());
        #[cfg(target_os = "windows")]
        {
            use windows_sys::Win32::System::Threading::{GetCurrentProcess, GetCurrentThread, SetPriorityClass, SetThreadPriority, ABOVE_NORMAL_PRIORITY_CLASS, THREAD_PRIORITY_ABOVE_NORMAL};
            unsafe {
                SetPriorityClass(GetCurrentProcess(), ABOVE_NORMAL_PRIORITY_CLASS);
                SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_ABOVE_NORMAL);
            }
        }
        #[cfg(target_os = "windows")]
        if let Some(window) = app.get_webview_window("main") {
            if let Ok(hwnd) = window.hwnd() {
                set_no_activate(hwnd.0, true);
                watch_foreground_changes(hwnd.0);
                watch_window_visibility(hwnd.0);
                let handle = hwnd.0 as isize;
                thread::spawn(move || {
                    let hwnd_handle = handle as windows_sys::Win32::Foundation::HWND;
                    for _ in 0..24 {
                        if READY.load(Ordering::Acquire) { break; }
                        unsafe {
                            use windows_sys::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_SHOWNA};
                            ShowWindow(hwnd_handle, SW_SHOWNA);
                        }
                        raise_topmost(hwnd_handle);
                        thread::sleep(Duration::from_millis(400));
                    }
                });
            }
        }
        Ok(())
    }).invoke_handler(tauri::generate_handler![main_window_ready, set_ball_anchor, restore_ball_position, move_window_clamped, set_window_mode, system_accent, inspect_path, read_visual_data_url, execute_action, configure_startup, check_update, download_and_launch_update, open_external, set_compass_mode, open_submenu_compass, quit_app, load_settings, save_settings]).run(tauri::generate_context!()).expect("error while running Cyrene Compass");
}
