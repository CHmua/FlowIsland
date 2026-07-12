use std::env;
use std::net::{SocketAddr, TcpStream};
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use sysinfo::{Networks, System};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, TrayIconBuilder, TrayIconEvent};
use tauri::{Emitter, Manager, State};
use tauri::{WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use tauri_plugin_autostart::MacosLauncher;

// --- WinAPI Imports ---
use std::os::windows::ffi::OsStrExt;
use winapi::shared::minwindef::{BOOL, FALSE, LPARAM, TRUE};
use winapi::shared::windef::{HWND, RECT};
use winapi::um::shellapi::ShellExecuteW;
use winapi::um::winuser::{
    keybd_event, EnumWindows, GetClassNameW, GetForegroundWindow, GetMonitorInfoW, GetWindowRect,
    GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible, MonitorFromWindow,
    SetForegroundWindow, ShowWindow, KEYEVENTF_KEYUP, MONITORINFO, MONITOR_DEFAULTTONEAREST,
    SW_HIDE, SW_SHOWNORMAL, VK_MEDIA_NEXT_TRACK, VK_MEDIA_PLAY_PAUSE, VK_MEDIA_PREV_TRACK, VK_MENU,
};

// --- Windows Crate Imports ---
use base64::{engine::general_purpose, Engine as _};
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;
use windows::core::Interface;
use windows::Foundation::TimeSpan;
use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSession, GlobalSystemMediaTransportControlsSessionManager,
    GlobalSystemMediaTransportControlsSessionMediaProperties,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus,
};
use windows::Storage::Streams::{Buffer, DataReader, InputStreamOptions};
use windows::Win32::Media::Audio::Endpoints::{IAudioEndpointVolume, IAudioMeterInformation};
use windows::Win32::Media::Audio::{
    eMultimedia, eRender, AudioSessionStateActive, IAudioSessionControl2, IAudioSessionManager2,
    IMMDeviceEnumerator, MMDeviceEnumerator,
};
use windows::Win32::System::Com::{CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED};
use windows_sys::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};

static LAST_NOTIFICATION_ID: AtomicU32 = AtomicU32::new(0);
static IS_NOTIF_INIT: AtomicBool = AtomicBool::new(false);
static TASKBAR_NOTIFY_STATE: OnceLock<Mutex<TaskbarNotifyState>> = OnceLock::new();
// 👈 新增：记录应用最后一次发出声音的绝对时间
static LAST_AUDIO_TIME: Mutex<Option<Instant>> = Mutex::new(None);

#[cfg(target_os = "windows")]
fn cleanup_legacy_shortcuts() {
    let legacy_shortcuts = [
        "NetSpeed Dynamic Pro.lnk",
        "NetSpeed Dynamic.lnk",
        "NSD.lnk",
    ];
    let legacy_folders = ["NetSpeed Dynamic Pro", "NetSpeed Dynamic", "NSD"];

    let mut roots = Vec::new();
    if let Some(appdata) = env::var_os("APPDATA") {
        roots.push(PathBuf::from(appdata).join("Microsoft\\Windows\\Start Menu\\Programs"));
    }
    if let Some(program_data) = env::var_os("ProgramData") {
        roots.push(PathBuf::from(program_data).join("Microsoft\\Windows\\Start Menu\\Programs"));
    }
    if let Some(profile) = env::var_os("USERPROFILE") {
        roots.push(PathBuf::from(profile).join("Desktop"));
    }
    if let Some(public) = env::var_os("PUBLIC") {
        roots.push(PathBuf::from(public).join("Desktop"));
    }

    for root in roots {
        for shortcut in legacy_shortcuts {
            let _ = std::fs::remove_file(root.join(shortcut));
        }
        for folder in legacy_folders {
            let _ = std::fs::remove_dir_all(root.join(folder));
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn cleanup_legacy_shortcuts() {}

#[derive(serde::Serialize, Clone)]
pub struct ToastData {
    pub app_name: String,
    pub title: String,
    pub body: String,
    pub aumid: String,
}

#[derive(Clone)]
struct TaskbarWindowSnapshot {
    hwnd_key: usize,
    app_name: String,
    title: String,
}

#[derive(Default)]
struct TaskbarNotifyState {
    initialized: bool,
    windows: Vec<TaskbarWindowSnapshot>,
    last_emit_key: String,
    last_emit_at: Option<Instant>,
}

struct TaskbarEnumContext {
    foreground_key: usize,
    windows: Vec<TaskbarWindowSnapshot>,
}

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MusicStatus {
    pub title: String,
    pub artist: String,
    pub source_app_id: String,
    pub thumbnail_data_uri: Option<String>,
    pub media_kind: String,
    pub is_playing: bool,
    pub position_ms: Option<u64>,
    pub duration_ms: Option<u64>,
}

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SyncedLyricLine {
    pub time_ms: u64,
    pub text: String,
}

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SyncedLyrics {
    pub source: String,
    pub lines: Vec<SyncedLyricLine>,
}

#[derive(serde::Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct HardwareStats {
    pub cpu_usage: f32,
    pub cpu_temperature: Option<f32>,
    pub cpu_fan_rpm: Option<u64>,
    pub used_memory: u64,
    pub total_memory: u64,
    pub gpu_usage: Option<f32>,
    pub gpu_fan_rpm: Option<u64>,
    pub gpu_fan_speed_percent: Option<f32>,
    pub gpu_memory_usage: Option<f32>,
    pub gpu_memory_used_mb: Option<u64>,
    pub gpu_memory_total_mb: Option<u64>,
    pub gpu_temperature: Option<f32>,
}

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseInfo {
    pub tag_name: String,
    pub name: String,
    pub html_url: String,
    pub download_url: Option<String>,
    pub asset_digest: Option<String>,
    pub asset_size: Option<u64>,
    pub source: String,
}

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDownloadProgress {
    pub status: String,
    pub downloaded: u64,
    pub total: Option<u64>,
    pub percent: u8,
}

struct ReleaseAsset {
    download_url: String,
    digest: Option<String>,
    size: Option<u64>,
}

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SystemStatusData {
    pub has_battery: bool,
    pub is_charging: bool,
    pub is_on_battery: bool,
    pub battery_percent: Option<u8>,
    pub volume_percent: Option<u8>,
}

#[derive(Clone, Copy, Default)]
struct HardwareAuxStats {
    cpu_temperature: Option<f32>,
    cpu_fan_rpm: Option<u64>,
    gpu_usage: Option<f32>,
    gpu_fan_rpm: Option<u64>,
    gpu_fan_speed_percent: Option<f32>,
    gpu_memory_usage: Option<f32>,
    gpu_memory_used_mb: Option<u64>,
    gpu_memory_total_mb: Option<u64>,
    gpu_temperature: Option<f32>,
}

#[derive(Default)]
struct HardwareSensorCache {
    value: HardwareAuxStats,
    checked_at: Option<Instant>,
    refreshing: bool,
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct TemperatureSensor {
    name: String,
    value: Option<f32>,
    #[serde(default)]
    sensor_type: Option<String>,
    #[serde(default)]
    hardware_type: Option<String>,
    #[serde(default)]
    identifier: Option<String>,
    #[serde(default)]
    parent: Option<String>,
}

struct MediaSessionSnapshot {
    title: String,
    artist: String,
    source_app_id: String,
    thumbnail_data_uri: Option<String>,
    is_playing: bool,
    position_ms: Option<u64>,
    duration_ms: Option<u64>,
}

struct MusicInfo {
    title: String,
    pid: u32,
}

struct WindowSearch {
    hwnd: HWND,
}

unsafe extern "system" fn enum_hardware_monitor_window_proc(hwnd: HWND, _lparam: LPARAM) -> BOOL {
    if IsWindowVisible(hwnd) == 0 {
        return TRUE;
    }

    let mut title = [0u16; 256];
    let len = GetWindowTextW(hwnd, title.as_mut_ptr(), title.len() as i32);
    if len <= 0 {
        return TRUE;
    }

    let title = String::from_utf16_lossy(&title[..len as usize]).to_lowercase();
    if title.contains("open hardware monitor")
        || title.contains("openhardwaremonitor")
        || title.contains("libre hardware monitor")
        || title.contains("librehardwaremonitor")
    {
        ShowWindow(hwnd, SW_HIDE);
    }

    TRUE
}

fn hide_hardware_monitor_windows() {
    unsafe {
        EnumWindows(Some(enum_hardware_monitor_window_proc), 0);
    }
}

fn ensure_widget_window(app: &tauri::App) -> tauri::Result<WebviewWindow> {
    if let Some(window) = app.get_webview_window("widget") {
        return Ok(window);
    }

    WebviewWindowBuilder::new(app, "widget", WebviewUrl::App("/widget".into()))
        .title("FlowIsland Widget")
        .inner_size(210.0, 36.0)
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .visible(false)
        .skip_taskbar(true)
        .shadow(false)
        .build()
}

fn apply_widget_window_style(widget_window: &WebviewWindow) {
    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::Foundation::HWND;
        use windows_sys::Win32::Graphics::Dwm::{
            DwmSetWindowAttribute, DWMWA_BORDER_COLOR, DWMWA_WINDOW_CORNER_PREFERENCE,
            DWMWCP_DONOTROUND,
        };
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            SetWindowLongPtrW, GWL_STYLE, WS_CAPTION,
        };

        if let Ok(hwnd) = widget_window.hwnd() {
            let hwnd_raw = hwnd.0 as HWND;
            unsafe {
                let current_style = windows_sys::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(
                    hwnd_raw, GWL_STYLE,
                );
                SetWindowLongPtrW(hwnd_raw, GWL_STYLE, current_style & !(WS_CAPTION as isize));

                let border_color: u32 = 0xFFFFFFFE;
                let _ = DwmSetWindowAttribute(
                    hwnd_raw,
                    DWMWA_BORDER_COLOR as u32,
                    &border_color as *const _ as *const _,
                    4,
                );

                let corner_preference = DWMWCP_DONOTROUND;
                let _ = DwmSetWindowAttribute(
                    hwnd_raw,
                    DWMWA_WINDOW_CORNER_PREFERENCE as u32,
                    &corner_preference as *const _ as *const _,
                    4,
                );
            }
        }
    }
}

fn collect_monitor_library_candidates(extra_roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Some(path) = env::var_os("NSD_OPEN_HARDWARE_MONITOR_LIB").map(PathBuf::from) {
        candidates.push(path);
    }
    if let Some(path) = env::var_os("NSD_HARDWARE_MONITOR_LIB").map(PathBuf::from) {
        candidates.push(path);
    }

    let mut roots = Vec::new();
    roots.extend_from_slice(extra_roots);

    if let Ok(current_exe) = env::current_exe() {
        if let Some(app_dir) = current_exe.parent() {
            roots.push(app_dir.to_path_buf());
        }
    }

    if let Ok(current_dir) = env::current_dir() {
        roots.push(current_dir.clone());
        roots.push(current_dir.join("resources"));
        roots.push(current_dir.join("src-tauri").join("resources"));
    }

    for var in [
        "ProgramFiles",
        "ProgramFiles(x86)",
        "LocalAppData",
        "ProgramData",
    ] {
        if let Some(base) = env::var_os(var).map(PathBuf::from) {
            roots.push(base);
        }
    }

    for root in roots {
        candidates.extend([
            root.join("tools")
                .join("OpenHardwareMonitor")
                .join("OpenHardwareMonitorLib.dll"),
            root.join("OpenHardwareMonitor")
                .join("OpenHardwareMonitorLib.dll"),
            root.join("Programs")
                .join("OpenHardwareMonitor")
                .join("OpenHardwareMonitorLib.dll"),
            root.join("tools")
                .join("LibreHardwareMonitor")
                .join("LibreHardwareMonitorLib.dll"),
            root.join("LibreHardwareMonitor")
                .join("LibreHardwareMonitorLib.dll"),
            root.join("Programs")
                .join("LibreHardwareMonitor")
                .join("LibreHardwareMonitorLib.dll"),
        ]);
    }

    candidates
        .into_iter()
        .filter(|path| path.is_file())
        .collect()
}

fn parse_optional_f32(value: &str) -> Option<f32> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("[not supported]") || trimmed == "N/A" {
        return None;
    }
    trimmed.parse::<f32>().ok()
}

fn parse_optional_u64(value: &str) -> Option<u64> {
    parse_optional_f32(value).map(|value| value.round() as u64)
}

fn command_output_with_timeout(mut command: Command, timeout: Duration) -> Option<Output> {
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .ok()?;
    let started_at = Instant::now();

    loop {
        match child.try_wait() {
            Ok(Some(_)) => return child.wait_with_output().ok(),
            Ok(None) => {
                if started_at.elapsed() >= timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    return None;
                }
                std::thread::sleep(Duration::from_millis(40));
            }
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
        }
    }
}

fn query_nvidia_gpu_stats() -> Option<HardwareStats> {
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let mut command = Command::new("nvidia-smi");
    command.args([
        "--query-gpu=temperature.gpu,utilization.gpu,utilization.memory,memory.used,memory.total,fan.speed",
        "--format=csv,noheader,nounits",
    ]);

    #[cfg(target_os = "windows")]
    command.creation_flags(CREATE_NO_WINDOW);

    let output = command_output_with_timeout(command, Duration::from_secs(2))?;
    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let line = text.lines().find(|line| !line.trim().is_empty())?;
    let parts: Vec<&str> = line.split(',').collect();
    if parts.len() < 5 {
        return None;
    }

    let gpu_temperature = parse_optional_f32(parts[0]);
    let gpu_usage = parse_optional_f32(parts[1]);
    let gpu_memory_usage = parse_optional_f32(parts[2]);
    let gpu_memory_used_mb = parse_optional_u64(parts[3]);
    let gpu_memory_total_mb = parse_optional_u64(parts[4]);
    let gpu_fan_speed_percent = parts.get(5).and_then(|value| parse_optional_f32(value));

    Some(HardwareStats {
        gpu_usage,
        gpu_fan_speed_percent,
        gpu_memory_usage,
        gpu_memory_used_mb,
        gpu_memory_total_mb,
        gpu_temperature,
        ..HardwareStats::default()
    })
}

fn merge_aux_stats(target: &mut HardwareAuxStats, source: HardwareAuxStats) {
    if target.cpu_temperature.is_none() {
        target.cpu_temperature = source.cpu_temperature;
    }
    if target.cpu_fan_rpm.is_none() {
        target.cpu_fan_rpm = source.cpu_fan_rpm;
    }
    if target.gpu_usage.is_none() {
        target.gpu_usage = source.gpu_usage;
    }
    if target.gpu_fan_rpm.is_none() {
        target.gpu_fan_rpm = source.gpu_fan_rpm;
    }
    if target.gpu_fan_speed_percent.is_none() {
        target.gpu_fan_speed_percent = source.gpu_fan_speed_percent;
    }
    if target.gpu_memory_usage.is_none() {
        target.gpu_memory_usage = source.gpu_memory_usage;
    }
    if target.gpu_memory_used_mb.is_none() {
        target.gpu_memory_used_mb = source.gpu_memory_used_mb;
    }
    if target.gpu_memory_total_mb.is_none() {
        target.gpu_memory_total_mb = source.gpu_memory_total_mb;
    }
    if target.gpu_temperature.is_none() {
        target.gpu_temperature = source.gpu_temperature;
    }
}

fn apply_aux_gpu_fallbacks(stats: &mut HardwareStats, aux_stats: HardwareAuxStats) {
    if stats.gpu_usage.is_none() {
        stats.gpu_usage = aux_stats.gpu_usage;
    }
    if stats.gpu_temperature.is_none() {
        stats.gpu_temperature = aux_stats.gpu_temperature;
    }
    if stats.gpu_memory_usage.is_none() {
        stats.gpu_memory_usage = aux_stats.gpu_memory_usage;
    }
    if stats.gpu_memory_used_mb.is_none() {
        stats.gpu_memory_used_mb = aux_stats.gpu_memory_used_mb;
    }
    if stats.gpu_memory_total_mb.is_none() {
        stats.gpu_memory_total_mb = aux_stats.gpu_memory_total_mb;
    }
    if stats.gpu_fan_speed_percent.is_none() {
        stats.gpu_fan_speed_percent = aux_stats.gpu_fan_speed_percent;
    }
}

fn is_reasonable_temperature(value: f32) -> bool {
    value.is_finite() && (5.0..=125.0).contains(&value)
}

fn cpu_sensor_score(sensor: &TemperatureSensor) -> Option<(i32, f32)> {
    let value = sensor.value?;
    if !is_reasonable_temperature(value) {
        return None;
    }

    let text = sensor_text(sensor);

    if text.contains("distance") {
        return None;
    }

    let is_non_cpu = [
        "gpu",
        "nvidia",
        "radeon",
        "nvme",
        "ssd",
        "hdd",
        "storage",
        "mainboard",
        "motherboard",
        "chipset",
        "pch",
        "battery",
    ]
    .iter()
    .any(|needle| text.contains(needle));

    if is_non_cpu {
        return None;
    }

    let has_cpu_hint = [
        "cpu", "intelcpu", "amdcpu", "package", "tctl", "tdie", "ccd", "core",
    ]
    .iter()
    .any(|needle| text.contains(needle));

    if !has_cpu_hint {
        return None;
    }

    let mut score = 10;
    if text.contains("package") || text.contains("tctl") || text.contains("tdie") {
        score += 100;
    }
    if text.contains("ccd") || text.contains("die") {
        score += 80;
    }
    if text.contains("max") {
        score += 55;
    }
    if text.contains("core") {
        score += 35;
    }
    if text.contains("cpu") || text.contains("intelcpu") || text.contains("amdcpu") {
        score += 25;
    }

    Some((score, value))
}

fn select_cpu_temperature(sensors: &[TemperatureSensor]) -> Option<f32> {
    sensors
        .iter()
        .filter_map(cpu_sensor_score)
        .max_by(|(left_score, left_value), (right_score, right_value)| {
            left_score.cmp(right_score).then_with(|| {
                left_value
                    .partial_cmp(right_value)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        })
        .map(|(_, value)| value)
}

fn is_reasonable_fan_rpm(value: f32) -> bool {
    value.is_finite() && (80.0..=10000.0).contains(&value)
}

fn sensor_text(sensor: &TemperatureSensor) -> String {
    format!(
        "{} {} {} {}",
        sensor.name,
        sensor.hardware_type.as_deref().unwrap_or_default(),
        sensor.identifier.as_deref().unwrap_or_default(),
        sensor.parent.as_deref().unwrap_or_default()
    )
    .to_lowercase()
}

fn sensor_type_is(sensor: &TemperatureSensor, expected: &str) -> bool {
    sensor
        .sensor_type
        .as_deref()
        .map(|value| value.eq_ignore_ascii_case(expected))
        .unwrap_or(false)
}

fn contains_any(text: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| text.contains(needle))
}

fn is_gpu_sensor(sensor: &TemperatureSensor) -> bool {
    let text = sensor_text(sensor);
    contains_any(
        &text,
        &[
            "gpu",
            "gpunvidia",
            "gpuamd",
            "gpuintel",
            "nvidia",
            "geforce",
            "rtx",
            "gtx",
            "radeon",
            "rx ",
            "arc",
            "intel graphics",
            "iris",
            "vega",
        ],
    )
}

fn percent_value(value: f32) -> Option<f32> {
    if value.is_finite() && (0.0..=100.0).contains(&value) {
        Some(value)
    } else {
        None
    }
}

fn select_gpu_temperature(sensors: &[TemperatureSensor]) -> Option<f32> {
    sensors
        .iter()
        .filter_map(|sensor| {
            let value = sensor.value?;
            if !sensor_type_is(sensor, "Temperature")
                || !is_reasonable_temperature(value)
                || !is_gpu_sensor(sensor)
            {
                return None;
            }

            let text = sensor_text(sensor);
            let mut score = 50;
            if contains_any(&text, &["core", "gpu temperature", "gpu core"]) {
                score += 90;
            }
            if contains_any(&text, &["hot spot", "hotspot", "junction"]) {
                score += 65;
            }
            if text.contains("memory") {
                score -= 30;
            }
            Some((score, value))
        })
        .max_by(|left, right| {
            left.0.cmp(&right.0).then_with(|| {
                left.1
                    .partial_cmp(&right.1)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        })
        .map(|(_, value)| value)
}

fn select_gpu_usage(sensors: &[TemperatureSensor]) -> Option<f32> {
    sensors
        .iter()
        .filter_map(|sensor| {
            let value = percent_value(sensor.value?)?;
            if !sensor_type_is(sensor, "Load") || !is_gpu_sensor(sensor) {
                return None;
            }

            let text = sensor_text(sensor);
            let mut score = 30;
            if contains_any(&text, &["gpu core", "gpu load", "core", "3d", "d3d"]) {
                score += 90;
            }
            if contains_any(&text, &["memory", "bus", "video", "decode", "encode"]) {
                score -= 35;
            }
            Some((score, value))
        })
        .max_by(|left, right| {
            left.0.cmp(&right.0).then_with(|| {
                left.1
                    .partial_cmp(&right.1)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        })
        .map(|(_, value)| value)
}

fn select_gpu_memory_usage(sensors: &[TemperatureSensor]) -> Option<f32> {
    sensors
        .iter()
        .filter_map(|sensor| {
            let value = percent_value(sensor.value?)?;
            if !sensor_type_is(sensor, "Load") || !is_gpu_sensor(sensor) {
                return None;
            }

            let text = sensor_text(sensor);
            if !contains_any(&text, &["memory", "vram", "dedicated"]) {
                return None;
            }

            let mut score = 50;
            if contains_any(&text, &["memory controller", "bus"]) {
                score -= 40;
            }
            if contains_any(&text, &["memory used", "memory load", "vram"]) {
                score += 40;
            }
            Some((score, value))
        })
        .max_by(|left, right| {
            left.0.cmp(&right.0).then_with(|| {
                left.1
                    .partial_cmp(&right.1)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        })
        .map(|(_, value)| value)
}

fn normalize_gpu_memory_mb(sensor: &TemperatureSensor) -> Option<u64> {
    let value = sensor.value?;
    if !value.is_finite() || value <= 0.0 {
        return None;
    }

    let text = sensor_text(sensor);
    let mb = if value <= 128.0 && contains_any(&text, &["memory", "vram"]) {
        value * 1024.0
    } else if value > 1024.0 * 1024.0 {
        value / 1024.0 / 1024.0
    } else if value > 262_144.0 {
        value / 1024.0
    } else {
        value
    };

    if mb.is_finite() && (1.0..=262_144.0).contains(&mb) {
        Some(mb.round() as u64)
    } else {
        None
    }
}

fn select_gpu_memory_mb_stats(sensors: &[TemperatureSensor]) -> (Option<u64>, Option<u64>) {
    let mut used_candidates: Vec<(i32, u64)> = Vec::new();
    let mut total_candidates: Vec<(i32, u64)> = Vec::new();

    for sensor in sensors {
        if !is_gpu_sensor(sensor)
            || !(sensor_type_is(sensor, "Data") || sensor_type_is(sensor, "SmallData"))
        {
            continue;
        }

        let text = sensor_text(sensor);
        if !contains_any(&text, &["memory", "vram", "dedicated"]) {
            continue;
        }

        let Some(value) = normalize_gpu_memory_mb(sensor) else {
            continue;
        };

        if contains_any(&text, &["total", "limit", "available"]) {
            let score = if text.contains("dedicated") { 90 } else { 60 };
            total_candidates.push((score, value));
        } else if contains_any(&text, &["used", "usage", "allocated", "dedicated"]) {
            let score = if text.contains("dedicated") { 90 } else { 60 };
            used_candidates.push((score, value));
        }
    }

    let used = used_candidates
        .into_iter()
        .max_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)))
        .map(|(_, value)| value);
    let total = total_candidates
        .into_iter()
        .max_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)))
        .map(|(_, value)| value);

    (used, total)
}

fn select_fan_stats(sensors: &[TemperatureSensor]) -> (Option<u64>, Option<u64>, Option<f32>) {
    let mut cpu_candidates: Vec<(i32, f32)> = Vec::new();
    let mut gpu_candidates: Vec<(i32, f32)> = Vec::new();
    let mut gpu_percent_candidates: Vec<(i32, f32)> = Vec::new();
    let mut fallback_candidates: Vec<f32> = Vec::new();

    for sensor in sensors {
        let Some(value) = sensor.value else {
            continue;
        };

        let sensor_type = sensor
            .sensor_type
            .as_deref()
            .unwrap_or_default()
            .to_lowercase();
        let text = sensor_text(sensor);
        let looks_like_fan = sensor_type == "fan"
            || sensor_type == "control"
            || text.contains("/fan/")
            || text.contains("fan");
        if !looks_like_fan {
            continue;
        }

        let is_gpu = is_gpu_sensor(sensor);
        let is_cpu = ["cpu", "processor"]
            .iter()
            .any(|needle| text.contains(needle));

        if sensor_type == "control" {
            if let Some(percent) = percent_value(value) {
                if is_gpu {
                    gpu_percent_candidates.push((80, percent));
                }
            }
            continue;
        }

        if !is_reasonable_fan_rpm(value) {
            continue;
        }

        if is_gpu {
            let score = if text.contains("gpu") { 80 } else { 60 };
            gpu_candidates.push((score, value));
        } else if is_cpu {
            let score = if text.contains("cpu") { 80 } else { 50 };
            cpu_candidates.push((score, value));
        } else {
            fallback_candidates.push(value);
        }
    }

    let cpu_fan_rpm = cpu_candidates
        .into_iter()
        .max_by(|left, right| {
            left.0.cmp(&right.0).then_with(|| {
                left.1
                    .partial_cmp(&right.1)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        })
        .map(|(_, value)| value.round() as u64)
        .or_else(|| {
            fallback_candidates
                .into_iter()
                .max_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal))
                .map(|value| value.round() as u64)
        });

    let gpu_fan_rpm = gpu_candidates
        .into_iter()
        .max_by(|left, right| {
            left.0.cmp(&right.0).then_with(|| {
                left.1
                    .partial_cmp(&right.1)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        })
        .map(|(_, value)| value.round() as u64);

    let gpu_fan_speed_percent = gpu_percent_candidates
        .into_iter()
        .max_by(|left, right| {
            left.0.cmp(&right.0).then_with(|| {
                left.1
                    .partial_cmp(&right.1)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        })
        .map(|(_, value)| value);

    (cpu_fan_rpm, gpu_fan_rpm, gpu_fan_speed_percent)
}

fn query_hardware_aux_once(extra_roots: &[PathBuf]) -> HardwareAuxStats {
    let mut stats = query_nvidia_gpu_stats()
        .map(|gpu_stats| HardwareAuxStats {
            gpu_usage: gpu_stats.gpu_usage,
            gpu_fan_speed_percent: gpu_stats.gpu_fan_speed_percent,
            gpu_memory_usage: gpu_stats.gpu_memory_usage,
            gpu_memory_used_mb: gpu_stats.gpu_memory_used_mb,
            gpu_memory_total_mb: gpu_stats.gpu_memory_total_mb,
            gpu_temperature: gpu_stats.gpu_temperature,
            ..HardwareAuxStats::default()
        })
        .unwrap_or_default();

    if env::var_os("NSD_DISABLE_LOW_LEVEL_SENSORS").is_none() {
        merge_aux_stats(
            &mut stats,
            query_hardware_aux_direct_lib(extra_roots).unwrap_or_default(),
        );
    }

    merge_aux_stats(&mut stats, query_hardware_aux_wmi().unwrap_or_default());
    stats
}

fn aux_stats_from_sensors(sensors: &[TemperatureSensor]) -> HardwareAuxStats {
    let cpu_temperature = select_cpu_temperature(sensors);
    let (cpu_fan_rpm, gpu_fan_rpm, gpu_fan_speed_percent) = select_fan_stats(sensors);
    let gpu_usage = select_gpu_usage(sensors);
    let gpu_temperature = select_gpu_temperature(sensors);
    let (gpu_memory_used_mb, gpu_memory_total_mb) = select_gpu_memory_mb_stats(sensors);
    let mut gpu_memory_usage = select_gpu_memory_usage(sensors);

    if gpu_memory_usage.is_none() {
        if let (Some(used), Some(total)) = (gpu_memory_used_mb, gpu_memory_total_mb) {
            if total > 0 {
                gpu_memory_usage = Some((used as f32 / total as f32) * 100.0);
            }
        }
    }

    HardwareAuxStats {
        cpu_temperature,
        cpu_fan_rpm,
        gpu_usage,
        gpu_fan_rpm,
        gpu_fan_speed_percent,
        gpu_memory_usage,
        gpu_memory_used_mb,
        gpu_memory_total_mb,
        gpu_temperature,
    }
}

fn query_hardware_aux_wmi() -> Option<HardwareAuxStats> {
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    let script = r#"
$ErrorActionPreference = 'SilentlyContinue'
$namespaces = @('root\LibreHardwareMonitor', 'root\OpenHardwareMonitor')
foreach ($namespace in $namespaces) {
  $sensors = @(Get-CimInstance -Namespace $namespace -ClassName Sensor | Where-Object {
    $_.SensorType -in @('Temperature','Fan','Load','Data','SmallData','Control')
  } | Select-Object Name, Value, SensorType, HardwareType, Identifier, Parent)
  if ($sensors.Count -gt 0) {
    ConvertTo-Json -InputObject $sensors -Compress -Depth 3
    exit 0
  }
}
exit 0
"#;

    let mut command = Command::new("powershell.exe");
    command.args([
        "-NoLogo",
        "-NoProfile",
        "-NonInteractive",
        "-ExecutionPolicy",
        "Bypass",
        "-Command",
        script,
    ]);

    #[cfg(target_os = "windows")]
    command.creation_flags(CREATE_NO_WINDOW);

    let output = command_output_with_timeout(command, Duration::from_secs(3))?;
    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }

    let sensors: Vec<TemperatureSensor> = if trimmed.starts_with('[') {
        serde_json::from_str(trimmed).ok()?
    } else {
        vec![serde_json::from_str(trimmed).ok()?]
    };

    Some(aux_stats_from_sensors(&sensors))
}

fn query_hardware_aux_direct_lib(extra_roots: &[PathBuf]) -> Option<HardwareAuxStats> {
    collect_monitor_library_candidates(extra_roots)
        .into_iter()
        .find_map(|lib_path| query_hardware_aux_from_library(&lib_path))
}

fn query_hardware_aux_from_library(lib_path: &Path) -> Option<HardwareAuxStats> {
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    let lib_path = lib_path.to_string_lossy().replace('\'', "''");
    let is_open_hardware_monitor = lib_path.to_lowercase().contains("openhardwaremonitorlib");
    let computer_type = if is_open_hardware_monitor {
        "OpenHardwareMonitor.Hardware.Computer"
    } else {
        "LibreHardwareMonitor.Hardware.Computer"
    };
    let script = format!(
        r#"
$ErrorActionPreference = 'Stop'
Add-Type -Path '{lib_path}'
$computer = [{computer_type}]::new()
$enabledProperties = @(
  'CPUEnabled','GPUEnabled','MainboardEnabled','RAMEnabled','HDDEnabled','FanControllerEnabled',
  'IsCpuEnabled','IsGpuEnabled','IsMotherboardEnabled','IsMemoryEnabled','IsStorageEnabled','IsControllerEnabled'
)
foreach ($propertyName in $enabledProperties) {{
  $property = $computer.GetType().GetProperty($propertyName)
  if ($property -and $property.CanWrite) {{
    $property.SetValue($computer, $true, $null)
  }}
}}
$computer.Open()
try {{
  for ($i = 0; $i -lt 2; $i++) {{
    Start-Sleep -Milliseconds 150
    foreach ($hardware in $computer.Hardware) {{
      $hardware.Update()
      foreach ($subHardware in $hardware.SubHardware) {{
        $subHardware.Update()
      }}
    }}
  }}

  $items = New-Object System.Collections.Generic.List[object]
  $allowedTypes = @('Temperature','Fan','Load','Data','SmallData','Control')
  foreach ($hardware in $computer.Hardware) {{
    foreach ($sensor in $hardware.Sensors) {{
      if ($allowedTypes -contains $sensor.SensorType.ToString()) {{
        $items.Add([pscustomobject]@{{
          Name = $sensor.Name
          Value = $sensor.Value
          SensorType = $sensor.SensorType.ToString()
          HardwareType = $hardware.HardwareType.ToString()
          Identifier = $sensor.Identifier.ToString()
          Parent = $hardware.Name
        }})
      }}
    }}
    foreach ($subHardware in $hardware.SubHardware) {{
      foreach ($sensor in $subHardware.Sensors) {{
        if ($allowedTypes -contains $sensor.SensorType.ToString()) {{
          $items.Add([pscustomobject]@{{
            Name = $sensor.Name
            Value = $sensor.Value
            SensorType = $sensor.SensorType.ToString()
            HardwareType = $subHardware.HardwareType.ToString()
            Identifier = $sensor.Identifier.ToString()
            Parent = $subHardware.Name
          }})
        }}
      }}
    }}
  }}

  if ($items.Count -gt 0) {{
    ConvertTo-Json -InputObject $items -Compress -Depth 3
  }}
}} finally {{
  $computer.Close()
}}
"#
    );

    let mut command = Command::new("powershell.exe");
    command.args([
        "-NoLogo",
        "-NoProfile",
        "-NonInteractive",
        "-ExecutionPolicy",
        "Bypass",
        "-Command",
        &script,
    ]);

    #[cfg(target_os = "windows")]
    command.creation_flags(CREATE_NO_WINDOW);

    let output = command_output_with_timeout(command, Duration::from_secs(3))?;
    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }

    let sensors: Vec<TemperatureSensor> = if trimmed.starts_with('[') {
        serde_json::from_str(trimmed).ok()?
    } else {
        vec![serde_json::from_str(trimmed).ok()?]
    };

    Some(aux_stats_from_sensors(&sensors))
}

fn query_hardware_aux_cached(
    cache: Arc<Mutex<HardwareSensorCache>>,
    extra_roots: Vec<PathBuf>,
) -> HardwareAuxStats {
    const HARDWARE_SENSOR_CACHE_TTL: Duration = Duration::from_secs(30);
    let now = Instant::now();
    let mut should_refresh = false;

    let cached_value = {
        let mut cached = cache.lock().unwrap();
        let is_fresh = cached
            .checked_at
            .map(|checked_at| now.duration_since(checked_at) < HARDWARE_SENSOR_CACHE_TTL)
            .unwrap_or(false);

        if !is_fresh && !cached.refreshing {
            cached.refreshing = true;
            should_refresh = true;
        }

        cached.value
    };

    if should_refresh {
        let cache_for_thread = Arc::clone(&cache);
        std::thread::spawn(move || {
            let value = std::panic::catch_unwind(|| query_hardware_aux_once(&extra_roots))
                .unwrap_or_default();
            let mut cached = cache_for_thread.lock().unwrap();
            cached.value = value;
            cached.checked_at = Some(Instant::now());
            cached.refreshing = false;
        });
    }

    cached_value
}

fn timespan_to_ms(value: TimeSpan) -> Option<u64> {
    if value.Duration < 0 {
        return None;
    }
    Some((value.Duration / 10_000) as u64)
}

fn media_thumbnail_content_type(bytes: &[u8]) -> &'static str {
    if bytes.starts_with(&[0x89, b'P', b'N', b'G']) {
        "image/png"
    } else if bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WEBP") {
        "image/webp"
    } else if bytes.starts_with(b"GIF") {
        "image/gif"
    } else {
        "image/jpeg"
    }
}

fn read_media_thumbnail_data_uri(
    props: &GlobalSystemMediaTransportControlsSessionMediaProperties,
) -> Option<String> {
    let thumbnail = props.Thumbnail().ok()?;
    let stream = thumbnail.OpenReadAsync().ok()?.get().ok()?;
    let size = stream.Size().ok()?;
    if size == 0 || size > 3 * 1024 * 1024 {
        return None;
    }

    let buffer = Buffer::Create(size as u32).ok()?;
    let read_buffer = stream
        .ReadAsync(&buffer, size as u32, InputStreamOptions::None)
        .ok()?
        .get()
        .ok()?;
    let reader = DataReader::FromBuffer(&read_buffer).ok()?;
    let len = reader.UnconsumedBufferLength().ok()? as usize;
    if len == 0 {
        return None;
    }

    let mut bytes = vec![0_u8; len];
    reader.ReadBytes(&mut bytes).ok()?;
    let content_type = media_thumbnail_content_type(&bytes);
    Some(format!(
        "data:{};base64,{}",
        content_type,
        general_purpose::STANDARD.encode(bytes)
    ))
}

fn windows_now_100ns() -> Option<i64> {
    const WINDOWS_UNIX_EPOCH_OFFSET_100NS: i128 = 116_444_736_000_000_000;
    let since_unix = SystemTime::now().duration_since(UNIX_EPOCH).ok()?;
    let now_100ns = (since_unix.as_nanos() / 100) as i128 + WINDOWS_UNIX_EPOCH_OFFSET_100NS;
    i64::try_from(now_100ns).ok()
}

fn elapsed_since_windows_time_ms(updated_time_100ns: i64) -> Option<u64> {
    let now = windows_now_100ns()?;
    if now <= updated_time_100ns {
        return Some(0);
    }
    Some(((now - updated_time_100ns) / 10_000) as u64)
}

fn normalize_media_text(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_alphanumeric())
        .flat_map(|ch| ch.to_lowercase())
        .collect()
}

fn text_matches(a: &str, b: &str) -> bool {
    let left = normalize_media_text(a);
    let right = normalize_media_text(b);
    if left.is_empty() || right.is_empty() {
        return false;
    }
    left.contains(&right) || right.contains(&left)
}

fn text_match_quality(a: &str, b: &str) -> i64 {
    let left = normalize_media_text(a);
    let right = normalize_media_text(b);
    if left.is_empty() || right.is_empty() {
        return 0;
    }
    if left == right {
        return 100;
    }
    if left.contains(&right) || right.contains(&left) {
        let min_len = left.chars().count().min(right.chars().count()) as f64;
        let max_len = left.chars().count().max(right.chars().count()) as f64;
        let ratio = if max_len > 0.0 {
            min_len / max_len
        } else {
            0.0
        };
        return if ratio >= 0.78 {
            88
        } else if ratio >= 0.58 {
            70
        } else {
            42
        };
    }
    0
}

fn is_unknown_artist_name(value: &str) -> bool {
    let normalized = normalize_media_text(value);
    normalized.is_empty()
        || normalized == normalize_media_text("未知歌手")
        || normalized == "unknownartist"
        || normalized == "unknown"
}

fn collapse_query_spaces(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn strip_lrclib_noise_segments(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut depth = 0_i32;
    let mut segment = String::new();

    for ch in value.chars() {
        let is_open = matches!(ch, '(' | '[' | '（' | '【');
        let is_close = matches!(ch, ')' | ']' | '）' | '】');

        if is_open {
            if depth == 0 {
                segment.clear();
            } else {
                segment.push(ch);
            }
            depth += 1;
            continue;
        }

        if is_close && depth > 0 {
            depth -= 1;
            if depth == 0 {
                let normalized = segment.to_lowercase();
                let is_noise = [
                    "feat",
                    "featuring",
                    "ft.",
                    "live",
                    "remaster",
                    "remastered",
                    "version",
                    "edit",
                    "mix",
                    "acoustic",
                    "demo",
                    "radio",
                    "single",
                    "explicit",
                    "sped",
                    "slowed",
                    "nightcore",
                    "karaoke",
                    "instrumental",
                ]
                .iter()
                .any(|keyword| normalized.contains(keyword));
                if !is_noise {
                    output.push('(');
                    output.push_str(segment.trim());
                    output.push(')');
                }
                segment.clear();
            } else {
                segment.push(ch);
            }
            continue;
        }

        if depth > 0 {
            segment.push(ch);
        } else {
            output.push(ch);
        }
    }

    collapse_query_spaces(&output)
}

fn clean_lrclib_track_name(value: &str) -> String {
    let mut cleaned = strip_lrclib_noise_segments(value)
        .replace("–", "-")
        .replace("—", "-")
        .replace("：", ":");

    for marker in [
        " feat. ",
        " ft. ",
        " featuring ",
        " Feat. ",
        " Ft. ",
        " Featuring ",
    ] {
        if let Some(index) = cleaned.find(marker) {
            cleaned.truncate(index);
        }
    }

    for marker in [
        " - Live",
        " - live",
        " - Remaster",
        " - remaster",
        " - Radio Edit",
        " - radio edit",
    ] {
        if let Some(index) = cleaned.find(marker) {
            cleaned.truncate(index);
        }
    }

    collapse_query_spaces(cleaned.trim_matches(|ch: char| matches!(ch, '-' | ':' | ' ' | '\t')))
}

fn clean_lrclib_artist_name(value: &str) -> String {
    let mut cleaned = value
        .replace("、", ",")
        .replace("，", ",")
        .replace(" & ", ",")
        .replace(" and ", ",");

    if let Some(index) = cleaned.find(',') {
        cleaned.truncate(index);
    }
    for marker in [
        " feat. ",
        " ft. ",
        " featuring ",
        " Feat. ",
        " Ft. ",
        " Featuring ",
    ] {
        if let Some(index) = cleaned.find(marker) {
            cleaned.truncate(index);
        }
    }

    collapse_query_spaces(cleaned.trim())
}

fn push_lrclib_query_variant(variants: &mut Vec<(String, String)>, title: String, artist: String) {
    let title = collapse_query_spaces(title.trim());
    let artist = collapse_query_spaces(artist.trim());
    if title.is_empty() {
        return;
    }
    if variants.iter().any(|(existing_title, existing_artist)| {
        normalize_media_text(existing_title) == normalize_media_text(&title)
            && normalize_media_text(existing_artist) == normalize_media_text(&artist)
    }) {
        return;
    }
    variants.push((title.to_string(), artist.to_string()));
}

fn lrclib_query_variants(song_name: &str, artist_name: &str) -> Vec<(String, String)> {
    let clean_title = clean_lrclib_track_name(song_name);
    let clean_artist = clean_lrclib_artist_name(artist_name);
    let mut variants = Vec::new();

    push_lrclib_query_variant(
        &mut variants,
        song_name.to_string(),
        artist_name.to_string(),
    );
    push_lrclib_query_variant(&mut variants, clean_title.clone(), artist_name.to_string());
    push_lrclib_query_variant(&mut variants, song_name.to_string(), clean_artist.clone());
    push_lrclib_query_variant(&mut variants, clean_title.clone(), clean_artist);
    push_lrclib_query_variant(&mut variants, clean_title, String::new());

    variants
}

#[allow(dead_code)]
fn duration_diff_ms(candidate_ms: Option<u64>, target_ms: Option<u64>) -> Option<u64> {
    Some(candidate_ms?.abs_diff(target_ms?))
}

fn session_matches_track(
    session_title: &str,
    _session_artist: &str,
    song_name: &str,
    _artist_name: &str,
) -> bool {
    text_matches(session_title, song_name)
}

fn read_media_session_snapshot(
    session: &GlobalSystemMediaTransportControlsSession,
) -> Option<MediaSessionSnapshot> {
    let props = session.TryGetMediaPropertiesAsync().ok()?.get().ok()?;
    let title = props.Title().ok()?.to_string().trim().to_string();
    let artist = props
        .Artist()
        .map(|value| value.to_string())
        .unwrap_or_default()
        .trim()
        .to_string();
    let source_app_id = session
        .SourceAppUserModelId()
        .map(|value| value.to_string())
        .unwrap_or_default();
    let thumbnail_data_uri = read_media_thumbnail_data_uri(&props);

    if title.is_empty() {
        return None;
    }

    let is_playing = session
        .GetPlaybackInfo()
        .ok()
        .and_then(|info| info.PlaybackStatus().ok())
        .map(|status| status == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing)
        .unwrap_or(false);

    let timeline = session.GetTimelineProperties().ok();
    let mut position_ms = timeline
        .as_ref()
        .and_then(|value| value.Position().ok())
        .and_then(timespan_to_ms);
    let duration_ms = timeline
        .as_ref()
        .and_then(|value| value.EndTime().ok())
        .and_then(timespan_to_ms);

    if is_playing {
        if let (Some(position), Some(elapsed)) = (
            position_ms,
            timeline
                .as_ref()
                .and_then(|value| value.LastUpdatedTime().ok())
                .and_then(|value| elapsed_since_windows_time_ms(value.UniversalTime)),
        ) {
            position_ms = Some(
                duration_ms
                    .map(|duration| (position + elapsed).min(duration))
                    .unwrap_or(position + elapsed),
            );
        }
    }

    Some(MediaSessionSnapshot {
        title,
        artist,
        source_app_id,
        thumbnail_data_uri,
        is_playing,
        position_ms,
        duration_ms,
    })
}

fn snapshot_from_session(
    session: &GlobalSystemMediaTransportControlsSession,
    song_name: &str,
    artist_name: &str,
) -> Option<MediaSessionSnapshot> {
    let snapshot = read_media_session_snapshot(session)?;

    if !session_matches_track(&snapshot.title, &snapshot.artist, song_name, artist_name) {
        return None;
    }

    Some(snapshot)
}

fn fetch_media_session_snapshot(
    song_name: &str,
    artist_name: &str,
) -> Option<MediaSessionSnapshot> {
    let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
        .ok()?
        .get()
        .ok()?;

    if let Ok(session) = manager.GetCurrentSession() {
        if let Some(snapshot) = snapshot_from_session(&session, song_name, artist_name) {
            return Some(snapshot);
        }
    }

    if let Ok(sessions) = manager.GetSessions() {
        let size = sessions.Size().ok().unwrap_or(0);
        for index in 0..size {
            if let Ok(session) = sessions.GetAt(index) {
                if let Some(snapshot) = snapshot_from_session(&session, song_name, artist_name) {
                    return Some(snapshot);
                }
            }
        }
    }

    None
}

fn is_known_music_source(source: &str) -> bool {
    let source = source.to_lowercase();
    [
        "spotify",
        "qqmusic",
        "qq音乐",
        "kugou",
        "酷狗",
        "kuwo",
        "酷我",
        "cloudmusic",
        "orpheus",
        "netease",
        "applemusic",
        "apple music",
        "itunes",
        "music.ui",
        "zune",
        "groove",
        "foobar",
        "musicbee",
        "aimp",
        "tidal",
        "deezer",
        "amazonmusic",
        "amazon music",
        "qobuz",
        "yandexmusic",
        "listen1",
        "lx-music",
        "lx_music",
        "yesplaymusic",
    ]
    .iter()
    .any(|keyword| source.contains(keyword))
}

fn is_netease_music_source(source: &str) -> bool {
    let source = source.to_lowercase();
    ["cloudmusic", "orpheus", "netease", "网易云", "網易雲"]
        .iter()
        .any(|keyword| source.contains(keyword))
}

fn is_browser_source(source: &str) -> bool {
    let source = source.to_lowercase();
    [
        "chrome",
        "google chrome",
        "msedge",
        "edge",
        "microsoft edge",
        "firefox",
        "brave",
        "opera",
        "opera gx",
        "vivaldi",
        "browser",
    ]
    .iter()
    .any(|keyword| source.contains(keyword))
}

fn media_kind_for_source(source: &str) -> &'static str {
    if is_browser_source(source) && !is_known_music_source(source) {
        "browser"
    } else {
        "music"
    }
}

fn is_bad_media_title(title: &str) -> bool {
    let title = title.trim().to_lowercase();
    title.is_empty()
        || title == "desktoplyric"
        || title == "网易云音乐"
        || title == "qq音乐"
        || title == "spotify"
        || title == "apple music"
        || title == "music"
        || title.contains("桌面歌词")
}

fn normalize_session_track(mut snapshot: MediaSessionSnapshot) -> Option<MediaSessionSnapshot> {
    if is_bad_media_title(&snapshot.title) {
        return None;
    }

    if snapshot.artist.trim().is_empty() && !is_browser_source(&snapshot.source_app_id) {
        let raw_title = snapshot.title.clone();
        for separator in [" - ", " — ", " – "] {
            if let Some((title, artist)) = raw_title.split_once(separator) {
                let title = title.trim();
                let artist = artist.trim();
                if !title.is_empty() && !artist.is_empty() {
                    snapshot.title = title.to_string();
                    snapshot.artist = artist.to_string();
                    break;
                }
            }
        }
    }

    if snapshot.artist.trim().is_empty() && !is_browser_source(&snapshot.source_app_id) {
        snapshot.artist = "未知歌手".to_string();
    }

    Some(snapshot)
}

fn media_session_score(snapshot: &MediaSessionSnapshot, is_current: bool) -> i32 {
    let known_music = is_known_music_source(&snapshot.source_app_id);
    let browser = is_browser_source(&snapshot.source_app_id);
    let has_artist = !snapshot.artist.trim().is_empty() && snapshot.artist != "未知歌手";

    let mut score = 0;
    if is_current {
        score += 30;
    }
    if snapshot.is_playing {
        score += 60;
    }
    if known_music {
        score += if snapshot.is_playing { 75 } else { 45 };
    }
    if browser {
        score += if snapshot.thumbnail_data_uri.is_some() {
            24
        } else {
            12
        };
    }
    if has_artist {
        score += 18;
    }
    if snapshot.duration_ms.unwrap_or(0) > 0 {
        score += 8;
    }
    score
}

fn accept_media_session(snapshot: &MediaSessionSnapshot) -> bool {
    if is_browser_source(&snapshot.source_app_id) && !is_known_music_source(&snapshot.source_app_id)
    {
        return false;
    }

    if is_known_music_source(&snapshot.source_app_id) {
        return true;
    }

    let has_artist = !snapshot.artist.trim().is_empty() && snapshot.artist != "未知歌手";
    snapshot.is_playing && has_artist
}

fn fetch_system_music_session() -> Option<MediaSessionSnapshot> {
    let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
        .ok()?
        .get()
        .ok()?;

    let mut candidates: Vec<(i32, MediaSessionSnapshot)> = Vec::new();

    if let Ok(session) = manager.GetCurrentSession() {
        if let Some(snapshot) = read_media_session_snapshot(&session)
            .and_then(normalize_session_track)
            .filter(accept_media_session)
        {
            candidates.push((media_session_score(&snapshot, true), snapshot));
        }
    }

    if let Ok(sessions) = manager.GetSessions() {
        let size = sessions.Size().ok().unwrap_or(0);
        for index in 0..size {
            if let Ok(session) = sessions.GetAt(index) {
                if let Some(snapshot) = read_media_session_snapshot(&session)
                    .and_then(normalize_session_track)
                    .filter(accept_media_session)
                {
                    candidates.push((media_session_score(&snapshot, false), snapshot));
                }
            }
        }
    }

    candidates
        .into_iter()
        .max_by_key(|(score, _)| *score)
        .map(|(_, snapshot)| snapshot)
}

unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let mut class_name = [0u16; 256];
    let len = GetClassNameW(hwnd, class_name.as_mut_ptr(), class_name.len() as i32);
    let class_str = String::from_utf16_lossy(&class_name[..len as usize]);

    if class_str.contains("Orpheus") || class_str.contains("CloudMusic") {
        let mut title = [0u16; 512];
        let len = GetWindowTextW(hwnd, title.as_mut_ptr(), title.len() as i32);
        let title_str = String::from_utf16_lossy(&title[..len as usize]);
        let clean_title = title_str.trim_matches('\0').trim().to_string();

        if !clean_title.is_empty() && clean_title != "网易云音乐" && clean_title != "DesktopLyric"
        {
            let mut pid = 0;
            GetWindowThreadProcessId(hwnd, &mut pid);

            let info = &mut *(lparam as *mut MusicInfo);
            info.title = clean_title;
            info.pid = pid;
            return FALSE;
        }
    }
    TRUE
}

unsafe extern "system" fn enum_netease_window_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    if IsWindowVisible(hwnd) == FALSE {
        return TRUE;
    }

    let mut class_name = [0u16; 256];
    let len = GetClassNameW(hwnd, class_name.as_mut_ptr(), class_name.len() as i32);
    let class_str = String::from_utf16_lossy(&class_name[..len as usize]);

    if !(class_str.contains("Orpheus") || class_str.contains("CloudMusic")) {
        return TRUE;
    }

    let mut title = [0u16; 512];
    let len = GetWindowTextW(hwnd, title.as_mut_ptr(), title.len() as i32);
    let title_str = String::from_utf16_lossy(&title[..len as usize]);
    let clean_title = title_str.trim_matches('\0').trim();

    if clean_title == "DesktopLyric" || clean_title.contains("桌面歌词") {
        return TRUE;
    }

    let search = &mut *(lparam as *mut WindowSearch);
    search.hwnd = hwnd;
    FALSE
}

// 核心重构：不仅查激活状态，更查“音轨分贝”！
unsafe fn is_process_playing_audio(target_pid: u32) -> bool {
    let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

    let enumerator: IMMDeviceEnumerator = match windows::Win32::System::Com::CoCreateInstance(
        &MMDeviceEnumerator,
        None,
        CLSCTX_ALL,
    ) {
        Ok(e) => e,
        Err(_) => return false,
    };

    let device = match enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia) {
        Ok(d) => d,
        Err(_) => return false,
    };

    let manager: IAudioSessionManager2 = match device.Activate(CLSCTX_ALL, None) {
        Ok(m) => m,
        Err(_) => return false,
    };

    let session_enum = match manager.GetSessionEnumerator() {
        Ok(s) => s,
        Err(_) => return false,
    };

    let count = session_enum.GetCount().unwrap_or(0);
    let mut is_making_sound = false;

    for i in 0..count {
        if let Ok(session) = session_enum.GetSession(i) {
            if let Ok(session2) = session.cast::<IAudioSessionControl2>() {
                if let Ok(pid) = session2.GetProcessId() {
                    if pid == target_pid {
                        if let Ok(state) = session2.GetState() {
                            if state == AudioSessionStateActive {
                                // 终极杀招：直接把音频会话强制转换为“声卡分贝仪”
                                if let Ok(meter) = session.cast::<IAudioMeterInformation>() {
                                    // 修复：完美适配 windows 0.58.0 版本的返回值签名
                                    if let Ok(peak) = meter.GetPeakValue() {
                                        // 哪怕只有极其微弱的声音 (振幅 > 0.0001)，也说明在发声
                                        if peak > 0.0001 {
                                            is_making_sound = true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if is_making_sound {
        // 更新最后一次发声时间
        if let Ok(mut last_time) = LAST_AUDIO_TIME.lock() {
            *last_time = Some(Instant::now());
        }
        return true;
    } else {
        // 如果此刻振幅跌至 0，给 2 秒的“静音宽限期”（防止歌曲本身的安静片段导致图标跳跃）
        if let Ok(last_time) = LAST_AUDIO_TIME.lock() {
            if let Some(time) = *last_time {
                if time.elapsed().as_secs() < 2 {
                    return true;
                }
            }
        }
    }
    false
}

#[tauri::command]
async fn fetch_netease_music_info() -> Result<Option<MusicStatus>, String> {
    let mut info = MusicInfo {
        title: String::new(),
        pid: 0,
    };

    unsafe {
        EnumWindows(Some(enum_windows_proc), &mut info as *mut _ as LPARAM);
    }

    if info.title.is_empty() {
        return Ok(None);
    }

    // 调用终极测音轨函数
    let is_playing = unsafe { is_process_playing_audio(info.pid) };

    let parts: Vec<&str> = info.title.splitn(2, " - ").collect();
    let song_name = parts[0].to_string();
    let artist_name = if parts.len() > 1 {
        parts[1].to_string()
    } else {
        "未知歌手".to_string()
    };

    let media_snapshot = fetch_media_session_snapshot(&song_name, &artist_name);

    Ok(Some(MusicStatus {
        title: song_name,
        artist: artist_name,
        source_app_id: "netease-cloudmusic".to_string(),
        thumbnail_data_uri: media_snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.thumbnail_data_uri.clone()),
        media_kind: "music".to_string(),
        is_playing: media_snapshot
            .as_ref()
            .map(|snapshot| snapshot.is_playing || is_playing)
            .unwrap_or(is_playing),
        position_ms: media_snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.position_ms),
        duration_ms: media_snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.duration_ms),
    }))
}

// ============== 下面的代码跟你原来一模一样，原封不动 ==============

#[tauri::command]
async fn fetch_music_info() -> Result<Option<MusicStatus>, String> {
    if let Some(snapshot) = fetch_system_music_session() {
        let netease_status = if is_netease_music_source(&snapshot.source_app_id) {
            fetch_netease_music_info().await.ok().flatten()
        } else {
            None
        };
        let media_kind = media_kind_for_source(&snapshot.source_app_id).to_string();

        return Ok(Some(MusicStatus {
            title: snapshot.title,
            artist: snapshot.artist,
            source_app_id: snapshot.source_app_id,
            thumbnail_data_uri: snapshot.thumbnail_data_uri.or_else(|| {
                netease_status
                    .as_ref()
                    .and_then(|status| status.thumbnail_data_uri.clone())
            }),
            media_kind,
            is_playing: snapshot.is_playing
                || netease_status
                    .as_ref()
                    .map(|status| status.is_playing)
                    .unwrap_or(false),
            position_ms: snapshot.position_ms.or_else(|| {
                netease_status
                    .as_ref()
                    .and_then(|status| status.position_ms)
            }),
            duration_ms: snapshot.duration_ms.or_else(|| {
                netease_status
                    .as_ref()
                    .and_then(|status| status.duration_ms)
            }),
        }));
    }

    fetch_netease_music_info().await
}

#[tauri::command]
async fn control_system_media(action: String) -> Result<(), String> {
    unsafe {
        let vk = match action.as_str() {
            "play_pause" => VK_MEDIA_PLAY_PAUSE,
            "next" => VK_MEDIA_NEXT_TRACK,
            "prev" => VK_MEDIA_PREV_TRACK,
            _ => return Ok(()),
        };
        keybd_event(vk as u8, 0, 0, 0);
        keybd_event(vk as u8, 0, KEYEVENTF_KEYUP, 0);
    }
    Ok(())
}

fn parse_lrc_timestamp(tag: &str) -> Option<u64> {
    let parts: Vec<&str> = tag.split(':').collect();
    let (hours, minutes, seconds) = match parts.as_slice() {
        [minutes, seconds] => (
            0.0,
            minutes.parse::<f64>().ok()?,
            seconds.parse::<f64>().ok()?,
        ),
        [hours, minutes, seconds] => (
            hours.parse::<f64>().ok()?,
            minutes.parse::<f64>().ok()?,
            seconds.parse::<f64>().ok()?,
        ),
        _ => return None,
    };

    let total = hours * 3_600.0 + minutes * 60.0 + seconds;
    if total.is_finite() && total >= 0.0 {
        Some((total * 1000.0).round() as u64)
    } else {
        None
    }
}

#[allow(dead_code)]
fn is_lyric_credit_line(text: &str) -> bool {
    let normalized = text
        .trim()
        .trim_matches(|ch| matches!(ch, '：' | ':' | ' ' | '\t'))
        .to_lowercase();

    if normalized.is_empty() {
        return true;
    }

    let compact = normalized
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>();

    let cjk_prefixes = [
        "作词",
        "作詞",
        "词",
        "詞",
        "作曲",
        "曲",
        "编曲",
        "編曲",
        "制作人",
        "制作",
        "监制",
        "監製",
        "出品",
        "发行",
        "發行",
        "录音",
        "錄音",
        "混音",
        "母带",
        "母帶",
        "和声",
        "和聲",
        "配唱",
        "统筹",
        "統籌",
        "企划",
        "企劃",
        "吉他",
        "贝斯",
        "貝斯",
        "鼓",
        "键盘",
        "鍵盤",
        "弦乐",
        "弦樂",
        "人声",
        "人聲",
    ];
    if cjk_prefixes.iter().any(|prefix| {
        compact.starts_with(prefix)
            && compact
                .chars()
                .nth(prefix.chars().count())
                .map(|ch| matches!(ch, ':' | '：' | '/' | '-' | '_' | '丨' | '|'))
                .unwrap_or(true)
    }) {
        return true;
    }

    let latin_prefixes = [
        "lyrics by",
        "lyricist",
        "written by",
        "composer",
        "composed by",
        "arranger",
        "arranged by",
        "producer",
        "produced by",
        "mixing",
        "mixed by",
        "mastering",
        "mastered by",
        "vocal",
        "guitar",
        "bass",
        "drums",
        "keyboard",
        "op:",
        "sp:",
    ];

    latin_prefixes
        .iter()
        .any(|prefix| normalized.starts_with(prefix))
}

fn parse_lrc_lines(raw: &str) -> Vec<SyncedLyricLine> {
    let mut lines = Vec::new();

    for source_line in raw.lines() {
        let mut rest = source_line.trim();
        let mut timestamps = Vec::new();

        while rest.starts_with('[') {
            let Some(end) = rest.find(']') else {
                break;
            };
            let tag = &rest[1..end];
            if let Some(time_ms) = parse_lrc_timestamp(tag) {
                timestamps.push(time_ms);
                rest = rest[end + 1..].trim_start();
            } else {
                break;
            }
        }

        let text = rest.trim();
        if text.is_empty() {
            continue;
        }

        for time_ms in timestamps {
            lines.push(SyncedLyricLine {
                time_ms,
                text: text.to_string(),
            });
        }
    }

    lines.sort_by_key(|line| line.time_ms);
    lines.dedup_by(|a, b| a.time_ms == b.time_ms && a.text == b.text);
    lines
}

fn lrclib_candidate_score(
    item: &serde_json::Value,
    song_name: &str,
    artist_name: &str,
    duration_ms: Option<u64>,
) -> i64 {
    let title = item
        .get("trackName")
        .or_else(|| item.get("name"))
        .and_then(|value| value.as_str())
        .unwrap_or_default();
    let artist = item
        .get("artistName")
        .and_then(|value| value.as_str())
        .unwrap_or_default();

    let title_quality = text_match_quality(title, song_name);
    if title_quality < 70 {
        return -10_000;
    }

    let mut duration_bonus = 0_i64;
    let mut duration_close = false;

    if let (Some(target_ms), Some(candidate_seconds)) = (
        duration_ms.filter(|value| *value > 0),
        item.get("duration").and_then(|value| value.as_f64()),
    ) {
        let target_seconds = target_ms as f64 / 1000.0;
        let diff = (candidate_seconds - target_seconds).abs();
        if diff <= 3.0 {
            duration_bonus = 90;
            duration_close = true;
        } else if diff <= 8.0 {
            duration_bonus = 55;
            duration_close = true;
        } else if diff <= 18.0 {
            duration_bonus = 20;
        } else {
            return -10_000;
        }
    }

    let artist_unknown = is_unknown_artist_name(artist_name);
    let mut artist_quality = if artist_unknown {
        65
    } else {
        text_match_quality(artist, artist_name)
    };
    if !artist_unknown && artist_quality < 45 {
        if title_quality >= 88 && duration_close {
            artist_quality = 35;
        } else {
            return -10_000;
        }
    }

    title_quality * 2 + artist_quality + duration_bonus
}

#[allow(unreachable_code)]
fn netease_song_score(
    item: &serde_json::Value,
    song_name: &str,
    artist_name: &str,
    duration_ms: Option<u64>,
) -> i64 {
    let title = item
        .get("name")
        .and_then(|value| value.as_str())
        .unwrap_or_default();
    let artists = item
        .get("artists")
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|artist| artist.get("name").and_then(|value| value.as_str()))
                .collect::<Vec<_>>()
                .join(" ")
        })
        .unwrap_or_default();

    let title_quality = text_match_quality(title, song_name);
    if title_quality < 72 {
        return -10_000;
    }

    let artist_unknown = is_unknown_artist_name(artist_name);
    let artist_quality = if artist_unknown {
        65
    } else {
        text_match_quality(&artists, artist_name)
    };
    if !artist_unknown && artist_quality < 45 {
        return -10_000;
    }

    let mut strict_score = title_quality * 2 + artist_quality;
    if let (Some(target_ms), Some(candidate_ms)) = (
        duration_ms.filter(|value| *value > 0),
        item.get("duration").and_then(|value| value.as_u64()),
    ) {
        let diff = candidate_ms.abs_diff(target_ms);
        if diff <= 3_000 {
            strict_score += 90;
        } else if diff <= 8_000 {
            strict_score += 55;
        } else if diff <= 18_000 {
            strict_score += 20;
        } else {
            return -10_000;
        }
    }
    return strict_score;

    let mut score = 0_i64;
    if text_matches(title, song_name) {
        score += 170;
    }
    if artist_name.trim().is_empty()
        || artist_name == "未知歌手"
        || text_matches(&artists, artist_name)
    {
        score += 120;
    }

    if let (Some(target_ms), Some(candidate_ms)) = (
        duration_ms.filter(|value| *value > 0),
        item.get("duration").and_then(|value| value.as_u64()),
    ) {
        let diff = candidate_ms.abs_diff(target_ms);
        if diff <= 3_000 {
            score += 90;
        } else if diff <= 8_000 {
            score += 55;
        } else if diff <= 18_000 {
            score += 20;
        } else {
            score -= (diff / 1000).min(60) as i64;
        }
    }

    score
}

fn contains_cjk_text(value: &str) -> bool {
    value.chars().any(|ch| {
        ('\u{4E00}'..='\u{9FFF}').contains(&ch)
            || ('\u{3400}'..='\u{4DBF}').contains(&ch)
            || ('\u{3040}'..='\u{30FF}').contains(&ch)
            || ('\u{AC00}'..='\u{D7AF}').contains(&ch)
    })
}

async fn fetch_netease_synced_lyrics(
    client: &reqwest::Client,
    song_name: &str,
    artist_name: &str,
    duration_ms: Option<u64>,
) -> Option<Vec<SyncedLyricLine>> {
    let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36";
    let query = if artist_name.trim().is_empty() || artist_name == "未知歌手" {
        song_name.to_string()
    } else {
        format!("{} {}", song_name, artist_name)
    };

    let search_json = client
        .post("https://music.163.com/api/search/get/web")
        .header("Referer", "https://music.163.com")
        .header("User-Agent", ua)
        .form(&[
            ("s", query.as_str()),
            ("type", "1"),
            ("limit", "8"),
            ("offset", "0"),
        ])
        .send()
        .await
        .ok()?
        .json::<serde_json::Value>()
        .await
        .ok()?;

    let songs = search_json
        .pointer("/result/songs")
        .and_then(|value| value.as_array())?;

    let best_song = songs
        .iter()
        .filter_map(|song| {
            let id = song.get("id").and_then(|value| value.as_i64())?;
            Some((
                netease_song_score(song, song_name, artist_name, duration_ms),
                id,
            ))
        })
        .max_by_key(|(score, _)| *score)?;

    if best_song.0 < 230 {
        return None;
    }

    let lyric_url = format!(
        "https://music.163.com/api/song/lyric?id={}&lv=1&kv=1&tv=-1",
        best_song.1
    );
    let lyric_json = client
        .get(&lyric_url)
        .header("Referer", "https://music.163.com")
        .header("User-Agent", ua)
        .send()
        .await
        .ok()?
        .json::<serde_json::Value>()
        .await
        .ok()?;

    let raw_lyrics = lyric_json
        .pointer("/lrc/lyric")
        .and_then(|value| value.as_str())?;
    let lines = parse_lrc_lines(raw_lyrics);
    if lines.len() >= 3 {
        Some(lines)
    } else {
        None
    }
}

async fn fetch_lrclib_exact_synced_lyrics(
    client: &reqwest::Client,
    song_name: &str,
    artist_name: &str,
    duration_ms: Option<u64>,
) -> Option<Vec<SyncedLyricLine>> {
    let mut url = reqwest::Url::parse("https://lrclib.net/api/get").ok()?;
    {
        let mut query = url.query_pairs_mut();
        query.append_pair("track_name", song_name);
        if !is_unknown_artist_name(artist_name) {
            query.append_pair("artist_name", artist_name);
        }
        if let Some(duration_ms) = duration_ms.filter(|value| *value > 0) {
            query.append_pair(
                "duration",
                &((duration_ms as f64 / 1000.0).round() as u64).to_string(),
            );
        }
    }

    let item = client
        .get(url)
        .send()
        .await
        .ok()
        .filter(|response| response.status().is_success())?
        .json::<serde_json::Value>()
        .await
        .ok()?;

    if lrclib_candidate_score(&item, song_name, artist_name, duration_ms) < 230 {
        return None;
    }

    let raw_lyrics = item.get("syncedLyrics").and_then(|value| value.as_str())?;
    let lines = parse_lrc_lines(raw_lyrics);
    if lines.len() >= 3 {
        Some(lines)
    } else {
        None
    }
}

async fn fetch_lrclib_search_synced_lyrics(
    client: &reqwest::Client,
    song_name: &str,
    artist_name: &str,
    duration_ms: Option<u64>,
) -> Option<Vec<SyncedLyricLine>> {
    let mut best: Option<(i64, Vec<SyncedLyricLine>)> = None;

    for (query_title, query_artist) in lrclib_query_variants(song_name, artist_name)
        .into_iter()
        .take(4)
    {
        let mut url = reqwest::Url::parse("https://lrclib.net/api/search").ok()?;
        {
            let mut query = url.query_pairs_mut();
            query.append_pair("track_name", &query_title);
            if !is_unknown_artist_name(&query_artist) {
                query.append_pair("artist_name", &query_artist);
            }
            if let Some(duration_ms) = duration_ms.filter(|value| *value > 0) {
                query.append_pair(
                    "duration",
                    &((duration_ms as f64 / 1000.0).round() as u64).to_string(),
                );
            }
        }

        let Ok(response) = client.get(url).send().await else {
            continue;
        };
        if !response.status().is_success() {
            continue;
        }

        let Ok(items) = response.json::<Vec<serde_json::Value>>().await else {
            continue;
        };

        for item in items {
            let Some(raw_lyrics) = item.get("syncedLyrics").and_then(|value| value.as_str()) else {
                continue;
            };
            if raw_lyrics.trim().is_empty() {
                continue;
            }

            let lines = parse_lrc_lines(raw_lyrics);
            if lines.len() < 3 {
                continue;
            }

            let score = lrclib_candidate_score(&item, song_name, artist_name, duration_ms);
            if best
                .as_ref()
                .map(|(best_score, _)| score > *best_score)
                .unwrap_or(true)
            {
                best = Some((score, lines));
            }
        }

        if best
            .as_ref()
            .map(|(score, _)| *score >= 285)
            .unwrap_or(false)
        {
            break;
        }
    }

    best.and_then(|(score, lines)| if score >= 225 { Some(lines) } else { None })
}

#[derive(Clone, Debug)]
struct WagequCandidate {
    url: String,
    title: String,
    artist: String,
}

fn decode_html_entity(entity: &str) -> Option<String> {
    match entity {
        "amp" => Some("&".to_string()),
        "lt" => Some("<".to_string()),
        "gt" => Some(">".to_string()),
        "quot" => Some("\"".to_string()),
        "apos" | "#39" => Some("'".to_string()),
        "nbsp" => Some(" ".to_string()),
        _ => {
            let number = entity
                .strip_prefix("#x")
                .or_else(|| entity.strip_prefix("#X"))
                .and_then(|value| u32::from_str_radix(value, 16).ok())
                .or_else(|| {
                    entity
                        .strip_prefix('#')
                        .and_then(|value| value.parse::<u32>().ok())
                })?;
            char::from_u32(number).map(|ch| ch.to_string())
        }
    }
}

fn decode_html_entities(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut rest = input;

    while let Some(pos) = rest.find('&') {
        output.push_str(&rest[..pos]);
        let after_amp = &rest[pos + 1..];
        if let Some(end) = after_amp.find(';').filter(|end| *end <= 12) {
            if let Some(decoded) = decode_html_entity(&after_amp[..end]) {
                output.push_str(&decoded);
                rest = &after_amp[end + 1..];
                continue;
            }
        }
        output.push('&');
        rest = after_amp;
    }

    output.push_str(rest);
    output
}

fn collapse_spaces(input: &str) -> String {
    input.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn byte_limit_at_char_boundary(input: &str, max_bytes: usize) -> usize {
    if input.len() <= max_bytes {
        return input.len();
    }

    let mut end = max_bytes;
    while end > 0 && !input.is_char_boundary(end) {
        end -= 1;
    }
    end
}

fn html_fragment_to_text(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut in_tag = false;

    for ch in input.chars() {
        match ch {
            '<' => {
                in_tag = true;
                output.push(' ');
            }
            '>' => in_tag = false,
            _ if !in_tag => output.push(ch),
            _ => {}
        }
    }

    collapse_spaces(&decode_html_entities(&output))
}

fn html_lyric_fragment_to_text(input: &str) -> String {
    let normalized = input
        .replace("<br>", "\n")
        .replace("<br/>", "\n")
        .replace("<br />", "\n")
        .replace("<BR>", "\n")
        .replace("<BR/>", "\n")
        .replace("<BR />", "\n");

    let mut output = String::with_capacity(normalized.len());
    let mut in_tag = false;

    for ch in normalized.chars() {
        match ch {
            '<' => {
                in_tag = true;
                output.push('\n');
            }
            '>' => in_tag = false,
            _ if !in_tag => output.push(ch),
            _ => {}
        }
    }

    decode_html_entities(&output)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn wagequ_absolute_lyric_url(href: &str) -> Option<String> {
    let href = href.trim();
    let path = if let Some(path) = href.strip_prefix("https://www.wagequ.com") {
        path
    } else if let Some(path) = href.strip_prefix("http://www.wagequ.com") {
        path
    } else {
        href
    };

    if !path.starts_with("/lyric/") || !path.ends_with(".html") {
        return None;
    }

    Some(format!("https://www.wagequ.com{}", path))
}

fn extract_first_attr(input: &str, attr: &str) -> Option<String> {
    let marker = format!("{}=\"", attr);
    let start = input.find(&marker)? + marker.len();
    let rest = &input[start..];
    let end = rest.find('"')?;
    Some(decode_html_entities(&rest[..end]))
}

fn extract_first_anchor_candidate(block: &str) -> Option<WagequCandidate> {
    let href_marker = "href=\"";
    let href_start = block.find(href_marker)? + href_marker.len();
    let href_rest = &block[href_start..];
    let href_end = href_rest.find('"')?;
    let href = &href_rest[..href_end];
    let url = wagequ_absolute_lyric_url(href)?;

    let anchor_tail = &href_rest[href_end + 1..];
    let anchor_text = anchor_tail
        .find('>')
        .and_then(|gt| {
            let text_start = gt + 1;
            let text_rest = &anchor_tail[text_start..];
            let text_end = text_rest
                .find("</a>")
                .unwrap_or_else(|| byte_limit_at_char_boundary(text_rest, 180));
            Some(html_fragment_to_text(&text_rest[..text_end]))
        })
        .unwrap_or_default();

    let mut title = anchor_text;
    if title.is_empty() {
        title = extract_first_attr(block, "title").unwrap_or_default();
    }
    if let Some(stripped) = title.strip_suffix("歌词") {
        title = stripped.trim().to_string();
    }

    let artist = block
        .find("class=\"singer\"")
        .and_then(|idx| {
            let singer_block = &block[idx..];
            let span_idx = singer_block.find("<span")?;
            let span_block = &singer_block[span_idx..];
            let gt = span_block.find('>')?;
            let text_start = gt + 1;
            let text_rest = &span_block[text_start..];
            let text_end = text_rest
                .find("</span>")
                .unwrap_or_else(|| byte_limit_at_char_boundary(text_rest, 120));
            Some(html_fragment_to_text(&text_rest[..text_end]))
        })
        .unwrap_or_default();

    Some(WagequCandidate { url, title, artist })
}

fn collect_wagequ_candidates_from_html(html: &str) -> Vec<WagequCandidate> {
    let mut candidates = Vec::new();

    for block in html.split("<li") {
        if !block.contains("/lyric/") {
            continue;
        }
        let Some(candidate) = extract_first_anchor_candidate(block) else {
            continue;
        };
        if candidates
            .iter()
            .any(|item: &WagequCandidate| item.url == candidate.url)
        {
            continue;
        }
        candidates.push(candidate);
    }

    candidates
}

fn extract_wagequ_detail_text(html: &str, marker: &str) -> Option<String> {
    let start = html.find(marker)?;
    let rest = &html[start..];
    let gt = rest.find('>')? + 1;
    let after_gt = &rest[gt..];
    let end = after_gt
        .find("</")
        .unwrap_or_else(|| byte_limit_at_char_boundary(after_gt, 240));
    Some(html_fragment_to_text(&after_gt[..end]))
}

fn extract_wagequ_detail_artist(html: &str) -> String {
    html.find("class=\"singer\"")
        .and_then(|idx| {
            let block = &html[idx..];
            let span_idx = block.find("<span")?;
            let span_block = &block[span_idx..];
            let gt = span_block.find('>')? + 1;
            let after_gt = &span_block[gt..];
            let end = after_gt
                .find("</span>")
                .unwrap_or_else(|| byte_limit_at_char_boundary(after_gt, 180));
            Some(html_fragment_to_text(&after_gt[..end]))
        })
        .unwrap_or_default()
}

fn extract_wagequ_lrc(html: &str) -> Option<String> {
    let mut lrc_start = None;
    for (idx, _) in html.match_indices('[') {
        let rest = &html[idx + 1..];
        let Some(end) = rest.find(']') else {
            continue;
        };
        if parse_lrc_timestamp(&rest[..end]).is_some() {
            lrc_start = Some(idx);
            break;
        }
    }

    let start = lrc_start?;
    let tail = &html[start..];
    let end = tail
        .find("<div class=\"download\"")
        .or_else(|| tail.find("<div class='download'"))
        .unwrap_or_else(|| byte_limit_at_char_boundary(tail, 40_000));

    Some(html_lyric_fragment_to_text(&tail[..end]))
}

fn wagequ_candidate_score(
    title: &str,
    artist: &str,
    song_name: &str,
    artist_name: &str,
    duration_ms: Option<u64>,
    lines: Option<&[SyncedLyricLine]>,
) -> i64 {
    let title_quality = text_match_quality(title, song_name);
    if title_quality < 72 {
        return -10_000;
    }

    let artist_unknown = is_unknown_artist_name(artist_name);
    let artist_quality = if artist_unknown {
        65
    } else if artist.trim().is_empty() {
        45
    } else {
        text_match_quality(artist, artist_name)
    };
    if !artist_unknown && artist_quality < 45 {
        return -10_000;
    }

    let mut score = title_quality * 2 + artist_quality;
    if let (Some(target_ms), Some(lines)) = (duration_ms.filter(|value| *value > 0), lines) {
        if let Some(last_ms) = lines.last().map(|line| line.time_ms) {
            let diff = last_ms.abs_diff(target_ms);
            if diff <= 8_000 {
                score += 70;
            } else if diff <= 25_000 {
                score += 35;
            } else if diff <= 55_000 {
                score += 10;
            } else if last_ms + 30_000 < target_ms {
                score -= 45;
            }
        }
    }

    score
}

async fn fetch_wagequ_page(client: &reqwest::Client, url: &str) -> Option<String> {
    client
        .get(url)
        .header("Referer", "https://www.wagequ.com/")
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36",
        )
        .send()
        .await
        .ok()
        .filter(|response| response.status().is_success())?
        .text()
        .await
        .ok()
}

async fn fetch_wagequ_detail(
    client: &reqwest::Client,
    url: &str,
) -> Option<(String, String, Vec<SyncedLyricLine>)> {
    let html = fetch_wagequ_page(client, url).await?;
    let title = extract_wagequ_detail_text(&html, "class=\"song-detail-name\"").unwrap_or_default();
    let artist = extract_wagequ_detail_artist(&html);
    let raw_lrc = extract_wagequ_lrc(&html)?;
    let lines = parse_lrc_lines(&raw_lrc);
    if lines.len() >= 3 {
        Some((title, artist, lines))
    } else {
        None
    }
}

async fn fetch_wagequ_synced_lyrics_inner(
    client: &reqwest::Client,
    song_name: &str,
    artist_name: &str,
    duration_ms: Option<u64>,
) -> Option<Vec<SyncedLyricLine>> {
    let query = if is_unknown_artist_name(artist_name) {
        song_name.to_string()
    } else {
        format!("{} {}", song_name, artist_name)
    };
    let encoded_query = urlencoding::encode(&query);
    let discovery_urls = [
        format!(
            "https://zhannei.baidu.com/cse/site?q={}&cc=www.wagequ.com",
            encoded_query
        ),
        "https://www.wagequ.com/lyric/".to_string(),
        "https://www.wagequ.com/lyric/?page=2".to_string(),
        "https://www.wagequ.com/lyric/?page=3".to_string(),
    ];

    let mut candidates: Vec<WagequCandidate> = Vec::new();
    for url in discovery_urls {
        let Some(html) = fetch_wagequ_page(client, &url).await else {
            continue;
        };
        for candidate in collect_wagequ_candidates_from_html(&html) {
            if candidates.iter().any(|item| item.url == candidate.url) {
                continue;
            }
            let score = wagequ_candidate_score(
                &candidate.title,
                &candidate.artist,
                song_name,
                artist_name,
                duration_ms,
                None,
            );
            if score >= 185 || candidate.title.is_empty() {
                candidates.push(candidate);
            }
        }
        if candidates.len() >= 8 {
            break;
        }
    }

    let mut best: Option<(i64, Vec<SyncedLyricLine>)> = None;
    for candidate in candidates.into_iter().take(8) {
        let Some((detail_title, detail_artist, lines)) =
            fetch_wagequ_detail(client, &candidate.url).await
        else {
            continue;
        };
        let title = if detail_title.is_empty() {
            candidate.title.as_str()
        } else {
            detail_title.as_str()
        };
        let artist = if detail_artist.is_empty() {
            candidate.artist.as_str()
        } else {
            detail_artist.as_str()
        };
        let score = wagequ_candidate_score(
            title,
            artist,
            song_name,
            artist_name,
            duration_ms,
            Some(&lines),
        );
        if best
            .as_ref()
            .map(|(best_score, _)| score > *best_score)
            .unwrap_or(true)
        {
            best = Some((score, lines));
        }
    }

    best.and_then(|(score, lines)| if score >= 220 { Some(lines) } else { None })
}

async fn fetch_wagequ_synced_lyrics(
    client: &reqwest::Client,
    song_name: &str,
    artist_name: &str,
    duration_ms: Option<u64>,
) -> Option<Vec<SyncedLyricLine>> {
    tokio::time::timeout(
        Duration::from_secs(5),
        fetch_wagequ_synced_lyrics_inner(client, song_name, artist_name, duration_ms),
    )
    .await
    .ok()
    .flatten()
}

#[tauri::command]
async fn fetch_synced_lyrics(
    song_name: String,
    artist_name: String,
    duration_ms: Option<u64>,
) -> Result<Option<SyncedLyrics>, String> {
    let title = song_name.trim();
    let artist = artist_name.trim();
    if title.is_empty() || title == "未在播放歌曲" {
        return Ok(None);
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(4))
        .user_agent("FlowIsland/2.3.5")
        .build()
        .map_err(|e| e.to_string())?;

    if false && (contains_cjk_text(title) || contains_cjk_text(artist)) {
        if let Some(lines) = fetch_netease_synced_lyrics(&client, title, artist, duration_ms).await
        {
            return Ok(Some(SyncedLyrics {
                source: "网易云音乐".to_string(),
                lines,
            }));
        }
    }

    if let Some(lines) = fetch_lrclib_exact_synced_lyrics(&client, title, artist, duration_ms).await
    {
        return Ok(Some(SyncedLyrics {
            source: "LRCLIB".to_string(),
            lines,
        }));
    }

    if let Some(lines) =
        fetch_lrclib_search_synced_lyrics(&client, title, artist, duration_ms).await
    {
        return Ok(Some(SyncedLyrics {
            source: "LRCLIB".to_string(),
            lines,
        }));
    }

    let mut url =
        reqwest::Url::parse("https://lrclib.net/api/search").map_err(|e| e.to_string())?;
    {
        let mut query = url.query_pairs_mut();
        query.append_pair("track_name", title);
        if !artist.is_empty() && artist != "未知歌手" {
            query.append_pair("artist_name", artist);
        }
        if let Some(duration_ms) = duration_ms.filter(|value| *value > 0) {
            query.append_pair(
                "duration",
                &((duration_ms as f64 / 1000.0).round() as u64).to_string(),
            );
        }
    }

    let response = client.get(url).send().await.map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        if let Some(lines) = fetch_netease_synced_lyrics(&client, title, artist, duration_ms).await
        {
            return Ok(Some(SyncedLyrics {
                source: "NetEase".to_string(),
                lines,
            }));
        }
        if let Some(lines) = fetch_wagequ_synced_lyrics(&client, title, artist, duration_ms).await {
            return Ok(Some(SyncedLyrics {
                source: "Wagequ".to_string(),
                lines,
            }));
        }
        return Ok(None);
    }

    let items = response
        .json::<Vec<serde_json::Value>>()
        .await
        .map_err(|e| e.to_string())?;

    let mut best: Option<(i64, Vec<SyncedLyricLine>)> = None;
    for item in items {
        let Some(raw_lyrics) = item.get("syncedLyrics").and_then(|value| value.as_str()) else {
            continue;
        };
        if raw_lyrics.trim().is_empty() {
            continue;
        }

        let lines = parse_lrc_lines(raw_lyrics);
        if lines.len() < 3 {
            continue;
        }

        let score = lrclib_candidate_score(&item, title, artist, duration_ms);
        if best
            .as_ref()
            .map(|(best_score, _)| score > *best_score)
            .unwrap_or(true)
        {
            best = Some((score, lines));
        }
    }

    if let Some((score, lines)) = best {
        if score >= 230 {
            return Ok(Some(SyncedLyrics {
                source: "LRCLIB".to_string(),
                lines,
            }));
        }
    }

    if let Some(lines) = fetch_netease_synced_lyrics(&client, title, artist, duration_ms).await {
        return Ok(Some(SyncedLyrics {
            source: "NetEase".to_string(),
            lines,
        }));
    }

    if let Some(lines) = fetch_wagequ_synced_lyrics(&client, title, artist, duration_ms).await {
        return Ok(Some(SyncedLyrics {
            source: "Wagequ".to_string(),
            lines,
        }));
    }

    Ok(
        fetch_netease_synced_lyrics(&client, title, artist, duration_ms)
            .await
            .map(|lines| SyncedLyrics {
                source: "网易云音乐".to_string(),
                lines,
            }),
    )
}

async fn cover_url_to_data_uri(client: &reqwest::Client, url: String) -> String {
    if url.starts_with("data:") {
        return url;
    }

    let response = match client.get(&url).send().await {
        Ok(resp) if resp.status().is_success() => resp,
        _ => return url,
    };

    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .filter(|value| value.starts_with("image/"))
        .map(|value| value.split(';').next().unwrap_or(value).to_string())
        .unwrap_or_else(|| {
            let lowered = url.to_lowercase();
            if lowered.contains(".png") {
                "image/png".to_string()
            } else if lowered.contains(".webp") {
                "image/webp".to_string()
            } else {
                "image/jpeg".to_string()
            }
        });

    let bytes = match response.bytes().await {
        Ok(bytes) if bytes.len() <= 3 * 1024 * 1024 => bytes,
        _ => return url,
    };

    format!(
        "data:{};base64,{}",
        content_type,
        general_purpose::STANDARD.encode(bytes)
    )
}

fn is_placeholder_cover_url(url: &str) -> bool {
    let normalized = url.to_lowercase();
    normalized.is_empty()
        || normalized.contains("3135032972947607")
        || normalized.contains("5639395138885805")
        || normalized.contains("109951163271499000")
}

async fn fetch_netease_cover_url(
    client: &reqwest::Client,
    song_name: &str,
    artist_name: &str,
) -> Option<String> {
    let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36";
    let query = if is_unknown_artist_name(artist_name) {
        song_name.to_string()
    } else {
        format!("{} {}", song_name, artist_name)
    };

    let search_json = client
        .post("https://music.163.com/api/search/get/web")
        .header("Referer", "https://music.163.com")
        .header("User-Agent", ua)
        .form(&[
            ("s", query.as_str()),
            ("type", "1"),
            ("limit", "8"),
            ("offset", "0"),
        ])
        .send()
        .await
        .ok()?
        .json::<serde_json::Value>()
        .await
        .ok()?;

    let songs = search_json
        .pointer("/result/songs")
        .and_then(|value| value.as_array())?;

    let (score, id, search_pic) = songs
        .iter()
        .filter_map(|song| {
            let id = song.get("id").and_then(|value| value.as_i64())?;
            let pic = song
                .pointer("/al/picUrl")
                .or_else(|| song.pointer("/album/picUrl"))
                .and_then(|value| value.as_str())
                .unwrap_or_default()
                .to_string();
            Some((
                netease_song_score(song, song_name, artist_name, None),
                id,
                pic,
            ))
        })
        .max_by_key(|(score, _, _)| *score)?;

    if score < 210 {
        return None;
    }

    let detail_url = format!(
        "https://music.163.com/api/v3/song/detail?c=%5B%7B%22id%22%3A{}%7D%5D",
        id
    );
    let detail_json = client
        .get(&detail_url)
        .header("Referer", "https://music.163.com")
        .header("User-Agent", ua)
        .send()
        .await
        .ok()
        .filter(|response| response.status().is_success())?
        .json::<serde_json::Value>()
        .await
        .ok()?;

    let detail_pic = detail_json
        .pointer("/songs/0/al/picUrl")
        .or_else(|| detail_json.pointer("/songs/0/album/picUrl"))
        .and_then(|value| value.as_str())
        .unwrap_or_default();

    let pic = if !is_placeholder_cover_url(detail_pic) {
        detail_pic
    } else if !is_placeholder_cover_url(&search_pic) {
        &search_pic
    } else {
        ""
    };

    if pic.is_empty() {
        None
    } else {
        Some(format!(
            "{}?param=300y300",
            pic.replace("http://", "https://")
        ))
    }
}

fn is_itunes_cover_source(source: &str) -> bool {
    let source = source.to_lowercase();
    ["applemusic", "apple music", "itunes"]
        .iter()
        .any(|keyword| source.contains(keyword))
}

fn is_deezer_cover_source(source: &str) -> bool {
    source.to_lowercase().contains("deezer")
}

async fn fetch_itunes_cover_url(
    client: &reqwest::Client,
    song_name: &str,
    artist_name: &str,
) -> Option<String> {
    let query = if is_unknown_artist_name(artist_name) {
        song_name.to_string()
    } else {
        format!("{} {}", song_name, artist_name)
    };
    let encoded_query = urlencoding::encode(&query).into_owned();
    let itunes_url = format!(
        "https://itunes.apple.com/search?term={}&media=music&limit=1",
        encoded_query
    );
    let json = client
        .get(&itunes_url)
        .send()
        .await
        .ok()?
        .json::<serde_json::Value>()
        .await
        .ok()?;
    let artwork = json
        .pointer("/results/0/artworkUrl100")
        .and_then(|value| value.as_str())?;
    if artwork.is_empty() {
        None
    } else {
        Some(artwork.replace("100x100bb", "300x300bb"))
    }
}

async fn fetch_deezer_cover_url(
    client: &reqwest::Client,
    song_name: &str,
    artist_name: &str,
) -> Option<String> {
    let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36";
    let deezer_url = format!(
        "https://api.deezer.com/search?q=track:\"{}\" artist:\"{}\"&limit=1",
        urlencoding::encode(song_name).into_owned(),
        urlencoding::encode(artist_name).into_owned()
    );
    let json = client
        .get(&deezer_url)
        .header("User-Agent", ua)
        .send()
        .await
        .ok()?
        .json::<serde_json::Value>()
        .await
        .ok()?;

    json.pointer("/data/0/album/cover_big")
        .or_else(|| json.pointer("/data/0/album/cover_medium"))
        .and_then(|value| value.as_str())
        .filter(|cover| !cover.is_empty())
        .map(|cover| cover.to_string())
}

/*
#[tauri::command]
async fn get_random_cover_url(
    song_name: String,
    artist_name: String,
    source_app_id: Option<String>,
) -> Result<String, String> {
    // 设置全局最大超时时间为 3 秒
    let client = reqwest::Client::builder()
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .map_err(|e| e.to_string())?;

    let source_app_id = source_app_id.unwrap_or_default();
    if is_netease_music_source(&source_app_id) {
        if let Some(url) = fetch_netease_cover_url(&client, &song_name, &artist_name).await {
            return Ok(cover_url_to_data_uri(&client, url).await);
        }
    } else if is_itunes_cover_source(&source_app_id) {
        if let Some(url) = fetch_itunes_cover_url(&client, &song_name, &artist_name).await {
            return Ok(cover_url_to_data_uri(&client, url).await);
        }
    } else if is_deezer_cover_source(&source_app_id) {
        if let Some(url) = fetch_deezer_cover_url(&client, &song_name, &artist_name).await {
            return Ok(cover_url_to_data_uri(&client, url).await);
        }
    }

    // 创建一个通道，用于接收最快返回的封面 URL (并发竞速)
    let (tx, mut rx) = tokio::sync::mpsc::channel(3);

    // 1号赛道：Apple Music (iTunes) - 通常速度最快，且欧美和华语覆盖率极高
    let tx_itunes = tx.clone();
    let tx_itunes = tx.clone();
    let client_itunes = client.clone();
    let query_itunes = format!("{} {}", song_name, artist_name);
    tokio::spawn(async move {
        let encoded_query = urlencoding::encode(&query_itunes).into_owned();
        let itunes_url = format!("https://itunes.apple.com/search?term={}&media=music&limit=1", encoded_query);
        if let Ok(resp) = client_itunes.get(&itunes_url).send().await {
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                if let Some(artwork) = json.pointer("/results/0/artworkUrl100").and_then(|v| v.as_str()) {
                    let _ = tx_itunes.send(artwork.replace("100x100bb", "300x300bb")).await;
                    // iTunes 默认给 100x100，替换为 300x300 保证清晰度
                    let _ = tx_itunes.send(artwork.replace("100x100bb", "300x300bb")).await;
                }
            }
        }
    });

    // 2号赛道：网易云音乐 API (华语原配)
    let tx_netease = tx.clone();
    let client_netease = client.clone();
    let song_netease = song_name.clone();
    let artist_netease = artist_name.clone();
    tokio::spawn(async move {
        if let Some(url) = fetch_netease_cover_url(&client_netease, &song_netease, &artist_netease).await {
            let _ = tx_netease.send(url).await;
            return;
        }
        let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36";
        let query = format!("{} {}", song_netease, artist_netease);
        if let Ok(resp) = client_netease.post("https://music.163.com/api/search/get/web")
            .header("Referer", "https://music.163.com")
            .header("User-Agent", ua)
            .form(&[("s", query.as_str()), ("type", "1"), ("limit", "1"), ("offset", "0")])
            .send().await
        {
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                if let Some(pic) = json.pointer("/result/songs/0/al/picUrl").and_then(|v| v.as_str()) {
                    if !pic.is_empty() && pic != "http://p4.music.126.net/UeTuwE7pvjBpypWLudqukQ==/3135032972947607.jpg" {
                        let _ = tx_netease.send(pic.replace("http://", "https://") + "?param=300y300").await;
                    }
                }
            }
        }
    });

    // 3号赛道：Deezer API (备用补充)
    let tx_deezer = tx.clone();
    let client_deezer = client.clone();
    let song_deezer = song_name.clone();
    let artist_deezer = artist_name.clone();
    tokio::spawn(async move {
        let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36";
        let deezer_url = format!(
            "https://api.deezer.com/search?q=track:\"{}\" artist:\"{}\"&limit=1",
            urlencoding::encode(&song_deezer).into_owned(),
            urlencoding::encode(&artist_deezer).into_owned()
        );
        if let Ok(resp) = client_deezer.get(&deezer_url).header("User-Agent", ua).send().await {
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                if let Some(cover) = json.pointer("/data/0/album/cover_medium").and_then(|v| v.as_str()) {
                    if !cover.is_empty() { let _ = tx_deezer.send(cover.to_string()).await; }
                } else if let Some(cover) = json.pointer("/data/0/album/cover_big").and_then(|v| v.as_str()) {
                    if !cover.is_empty() { let _ = tx_deezer.send(cover.to_string()).await; }
                }
            }
        }
    });

    // 终点线判定：等待第一个成功的请求，谁先返回就用谁的，最多等 3 秒
    match tokio::time::timeout(std::time::Duration::from_secs(3), rx.recv()).await {
        Ok(Some(url)) => Ok(cover_url_to_data_uri(&client, url).await), // 拿到最快返回的那个链接，并转为可取色的内嵌图片
        _ => Ok("data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMTUwIiBoZWlnaHQ9IjE1MCIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj48ZGVmcz48bGluZWFyR3JhZGllbnQgaWQ9ImciIHgxPSIwJSIgeTE9IjAlIiB4Mj0iMTAwJSIgeTI9IjEwMCUiPjxzdG9wIG9mZnNldD0iMCUiIHN0b3AtY29sb3I9IiNhOGVkZWEiLz48c3RvcCBvZmZzZXQ9IjEwMCUiIHN0b3AtY29sb3I9IiNmZWQ2ZTMiLz48L2xpbmVhckdyYWRpZW50PjwvZGVmcz48cmVjdCB3aWR0aD0iMTUwIiBoZWlnaHQ9IjE1MCIgcng9Ijc1IiBmaWxsPSJ1cmwoI2cpIi8+PC9zdmc+".to_string()), // 全都失败则返回默认渐变色图
    }
}

*/

#[tauri::command]
async fn get_random_cover_url(
    song_name: String,
    artist_name: String,
    source_app_id: Option<String>,
) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .map_err(|e| e.to_string())?;

    let source_app_id = source_app_id.unwrap_or_default();
    if is_netease_music_source(&source_app_id) {
        if let Some(url) = fetch_netease_cover_url(&client, &song_name, &artist_name).await {
            return Ok(cover_url_to_data_uri(&client, url).await);
        }
    } else if is_itunes_cover_source(&source_app_id) {
        if let Some(url) = fetch_itunes_cover_url(&client, &song_name, &artist_name).await {
            return Ok(cover_url_to_data_uri(&client, url).await);
        }
    } else if is_deezer_cover_source(&source_app_id) {
        if let Some(url) = fetch_deezer_cover_url(&client, &song_name, &artist_name).await {
            return Ok(cover_url_to_data_uri(&client, url).await);
        }
    }

    let (tx, mut rx) = tokio::sync::mpsc::channel(3);

    let tx_itunes = tx.clone();
    let client_itunes = client.clone();
    let song_itunes = song_name.clone();
    let artist_itunes = artist_name.clone();
    tokio::spawn(async move {
        if let Some(url) =
            fetch_itunes_cover_url(&client_itunes, &song_itunes, &artist_itunes).await
        {
            let _ = tx_itunes.send(url).await;
        }
    });

    let tx_netease = tx.clone();
    let client_netease = client.clone();
    let song_netease = song_name.clone();
    let artist_netease = artist_name.clone();
    tokio::spawn(async move {
        if let Some(url) =
            fetch_netease_cover_url(&client_netease, &song_netease, &artist_netease).await
        {
            let _ = tx_netease.send(url).await;
        }
    });

    let tx_deezer = tx.clone();
    let client_deezer = client.clone();
    let song_deezer = song_name.clone();
    let artist_deezer = artist_name.clone();
    tokio::spawn(async move {
        if let Some(url) =
            fetch_deezer_cover_url(&client_deezer, &song_deezer, &artist_deezer).await
        {
            let _ = tx_deezer.send(url).await;
        }
    });

    match tokio::time::timeout(std::time::Duration::from_secs(3), rx.recv()).await {
        Ok(Some(url)) => Ok(cover_url_to_data_uri(&client, url).await),
        _ => Ok("data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMTUwIiBoZWlnaHQ9IjE1MCIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj48ZGVmcz48bGluZWFyR3JhZGllbnQgaWQ9ImciIHgxPSIwJSIgeTE9IjAlIiB4Mj0iMTAwJSIgeTI9IjEwMCUiPjxzdG9wIG9mZnNldD0iMCUiIHN0b3AtY29sb3I9IiNhOGVkZWEiLz48c3RvcCBvZmZzZXQ9IjEwMCUiIHN0b3AtY29sb3I9IiNmZWQ2ZTMiLz48L2xpbmVhckdyYWRpZW50PjwvZGVmcz48cmVjdCB3aWR0aD0iMTUwIiBoZWlnaHQ9IjE1MCIgcng9Ijc1IiBmaWxsPSJ1cmwoI2cpIi8+PC9zdmc+".to_string()),
    }
}

fn read_window_class(hwnd: HWND) -> String {
    let mut class_name = [0u16; 256];
    let len = unsafe { GetClassNameW(hwnd, class_name.as_mut_ptr(), class_name.len() as i32) };
    String::from_utf16_lossy(&class_name[..len as usize])
        .trim_matches('\0')
        .trim()
        .to_string()
}

fn read_window_title(hwnd: HWND) -> String {
    let mut title = [0u16; 512];
    let len = unsafe { GetWindowTextW(hwnd, title.as_mut_ptr(), title.len() as i32) };
    String::from_utf16_lossy(&title[..len as usize])
        .trim_matches('\0')
        .trim()
        .to_string()
}

fn classify_taskbar_app(class_name: &str, title: &str) -> Option<&'static str> {
    let class_lower = class_name.to_lowercase();
    let title_lower = title.to_lowercase();
    let text = format!("{} {}", class_lower, title_lower);

    if text.contains("flowisland")
        || text.contains("flow island")
        || text.contains("netspeed")
        || text.contains("nsd 控制台")
        || text.contains("nsd widget")
        || text.contains("flowisland 控制台")
        || text.contains("flowisland widget")
    {
        return None;
    }
    if class_lower == "progman" || class_lower == "workerw" || class_lower == "shell_traywnd" {
        return None;
    }

    if text.contains("wechat") || text.contains("微信") || class_lower.contains("weui") {
        return Some("微信");
    }
    if text.contains("wxwork") || text.contains("企业微信") {
        return Some("企业微信");
    }
    if text.contains("qq") || class_lower.contains("txguifoundation") {
        return Some("QQ");
    }
    if text.contains("dingtalk") || text.contains("钉钉") {
        return Some("钉钉");
    }
    if text.contains("feishu") || text.contains("lark") || text.contains("飞书") {
        return Some("飞书");
    }
    if text.contains("telegram") {
        return Some("Telegram");
    }
    if text.contains("discord") {
        return Some("Discord");
    }
    if text.contains("slack") {
        return Some("Slack");
    }
    if text.contains("teams") || text.contains("msteams") {
        return Some("Teams");
    }

    None
}

fn is_generic_taskbar_title(app_name: &str, title: &str) -> bool {
    let title = title.trim();
    if title.is_empty() {
        return true;
    }

    let title_lower = title.to_lowercase();
    let app_lower = app_name.to_lowercase();
    title_lower == app_lower
        || title_lower == "wechat"
        || title == "微信"
        || title_lower == "qq"
        || title_lower == "dingtalk"
        || title == "钉钉"
        || title_lower == "telegram"
        || title_lower == "discord"
        || title_lower == "slack"
        || title_lower == "microsoft teams"
}

fn title_has_attention_signal(title: &str) -> bool {
    let title = title.trim();
    let lower = title.to_lowercase();
    if title.is_empty() {
        return false;
    }

    lower.contains("新消息")
        || lower.contains("未读")
        || lower.contains("条消息")
        || lower.contains("new message")
        || lower.contains("unread")
        || (title.starts_with('[')
            && title.chars().take_while(|ch| *ch != ']').count() <= 5
            && title.contains(']'))
        || (title.starts_with('(')
            && title.chars().take_while(|ch| *ch != ')').count() <= 5
            && title.contains(')'))
}

unsafe extern "system" fn enum_taskbar_notify_window_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    if IsWindowVisible(hwnd) == FALSE {
        return TRUE;
    }

    let class_name = read_window_class(hwnd);
    let title = read_window_title(hwnd);
    let Some(app_name) = classify_taskbar_app(&class_name, &title) else {
        return TRUE;
    };

    let context = &mut *(lparam as *mut TaskbarEnumContext);
    if context.foreground_key == hwnd as usize {
        return TRUE;
    }

    context.windows.push(TaskbarWindowSnapshot {
        hwnd_key: hwnd as usize,
        app_name: app_name.to_string(),
        title,
    });

    TRUE
}

fn fetch_taskbar_notification_fallback() -> Option<ToastData> {
    let foreground_key = unsafe { GetForegroundWindow() as usize };
    let mut context = TaskbarEnumContext {
        foreground_key,
        windows: Vec::new(),
    };

    unsafe {
        EnumWindows(
            Some(enum_taskbar_notify_window_proc),
            &mut context as *mut _ as LPARAM,
        );
    }

    let state_lock = TASKBAR_NOTIFY_STATE.get_or_init(|| Mutex::new(TaskbarNotifyState::default()));
    let mut state = state_lock.lock().ok()?;

    if !state.initialized {
        state.windows = context.windows;
        state.initialized = true;
        return None;
    }

    let mut candidate: Option<TaskbarWindowSnapshot> = None;

    for current in &context.windows {
        let previous = state
            .windows
            .iter()
            .find(|item| item.hwnd_key == current.hwnd_key);
        let title_changed = previous
            .map(|item| item.title != current.title)
            .unwrap_or(false);
        let attention_title = title_has_attention_signal(&current.title);
        let useful_title = !is_generic_taskbar_title(&current.app_name, &current.title);

        if attention_title || (title_changed && useful_title) {
            candidate = Some(current.clone());
            break;
        }
    }

    state.windows = context.windows;

    if let Some(candidate) = candidate {
        let emit_key = format!("{}:{}", candidate.app_name, candidate.title);
        let now = Instant::now();
        let recently_emitted = state.last_emit_key == emit_key
            && state
                .last_emit_at
                .map(|instant| now.duration_since(instant) < Duration::from_secs(12))
                .unwrap_or(false);

        if recently_emitted {
            return None;
        }

        state.last_emit_key = emit_key;
        state.last_emit_at = Some(now);

        return Some(ToastData {
            app_name: candidate.app_name.clone(),
            title: format!("{} 有新提醒", candidate.app_name),
            body: "请打开应用查看详情".to_string(),
            aumid: String::new(),
        });
    }

    None
}

#[tauri::command]
async fn fetch_latest_notification() -> Result<Option<ToastData>, String> {
    use windows::UI::Notifications::Management::UserNotificationListener;
    use windows::UI::Notifications::NotificationKinds;

    let listener = match UserNotificationListener::Current() {
        Ok(l) => l,
        Err(_) => return Ok(fetch_taskbar_notification_fallback()),
    };

    let _ = listener.RequestAccessAsync();

    let notifications = match listener.GetNotificationsAsync(NotificationKinds::Toast) {
        Ok(op) => match op.get() {
            Ok(ns) => ns,
            Err(_) => return Ok(fetch_taskbar_notification_fallback()),
        },
        Err(_) => return Ok(fetch_taskbar_notification_fallback()),
    };

    let mut latest_notif = None;
    let mut max_id = 0u32;

    for notif in notifications {
        if let Ok(id) = notif.Id() {
            if id > max_id {
                max_id = id;
                latest_notif = Some(notif);
            }
        }
    }

    if max_id == 0 {
        return Ok(fetch_taskbar_notification_fallback());
    }

    let last_processed_id = LAST_NOTIFICATION_ID.load(Ordering::SeqCst);

    if !IS_NOTIF_INIT.load(Ordering::SeqCst) {
        LAST_NOTIFICATION_ID.store(max_id, Ordering::SeqCst);
        IS_NOTIF_INIT.store(true, Ordering::SeqCst);
        return Ok(None);
    }

    if max_id > last_processed_id {
        LAST_NOTIFICATION_ID.store(max_id, Ordering::SeqCst);

        if let Some(notif) = latest_notif {
            let app_name = notif
                .AppInfo()
                .and_then(|info| info.DisplayInfo())
                .and_then(|dinfo| dinfo.DisplayName())
                .map(|name| name.to_string())
                .unwrap_or_else(|_| "系统通知".to_string());

            let aumid = notif
                .AppInfo()
                .and_then(|info| info.AppUserModelId())
                .map(|id| id.to_string())
                .unwrap_or_default();

            if let Ok(toast_binding) = notif
                .Notification()
                .and_then(|n| n.Visual())
                .and_then(|v| v.GetBinding(&windows::core::HSTRING::from("ToastGeneric")))
            {
                if let Ok(text_elements) = toast_binding.GetTextElements() {
                    let mut text_list = Vec::new();
                    for elem in text_elements {
                        if let Ok(text) = elem.Text() {
                            text_list.push(text.to_string());
                        }
                    }

                    if !text_list.is_empty() {
                        let title = text_list.first().cloned().unwrap_or_default();
                        let body = if text_list.len() > 1 {
                            text_list[1..].join(" ")
                        } else {
                            String::new()
                        };

                        return Ok(Some(ToastData {
                            app_name,
                            title,
                            body,
                            aumid,
                        }));
                    }
                }
            }
        }
    }

    Ok(fetch_taskbar_notification_fallback())
}

#[tauri::command]
fn open_app_by_aumid(aumid: String, app_name: String) {
    let app_lower = app_name.to_lowercase();

    unsafe {
        keybd_event(VK_MENU as u8, 0, 0, 0);
        keybd_event(VK_MENU as u8, 0, KEYEVENTF_KEYUP, 0);
    }

    let execute_protocol = |protocol: &str| unsafe {
        let op = std::ffi::OsStr::new("open")
            .encode_wide()
            .chain(Some(0))
            .collect::<Vec<u16>>();
        let file = std::ffi::OsStr::new(protocol)
            .encode_wide()
            .chain(Some(0))
            .collect::<Vec<u16>>();
        ShellExecuteW(
            std::ptr::null_mut(),
            op.as_ptr(),
            file.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            SW_SHOWNORMAL,
        );
    };

    if app_lower.contains("qq") {
        execute_protocol("tencent://message/");
    } else if app_lower.contains("微信") || app_lower.contains("wechat") {
        execute_protocol("weixin://");
    } else if app_lower.contains("钉钉") || app_lower.contains("dingtalk") {
        execute_protocol("dingtalk://");
    } else if !aumid.is_empty() {
        unsafe {
            let op = std::ffi::OsStr::new("open")
                .encode_wide()
                .chain(Some(0))
                .collect::<Vec<u16>>();
            let file = std::ffi::OsStr::new("explorer.exe")
                .encode_wide()
                .chain(Some(0))
                .collect::<Vec<u16>>();
            let params = std::ffi::OsStr::new(&format!("shell:AppsFolder\\{}", aumid))
                .encode_wide()
                .chain(Some(0))
                .collect::<Vec<u16>>();
            ShellExecuteW(
                std::ptr::null_mut(),
                op.as_ptr(),
                file.as_ptr(),
                params.as_ptr(),
                std::ptr::null(),
                SW_SHOWNORMAL,
            );
        }
    }
}

fn shell_execute_open(target: &str) {
    unsafe {
        let op = std::ffi::OsStr::new("open")
            .encode_wide()
            .chain(Some(0))
            .collect::<Vec<u16>>();
        let file = std::ffi::OsStr::new(target)
            .encode_wide()
            .chain(Some(0))
            .collect::<Vec<u16>>();
        ShellExecuteW(
            std::ptr::null_mut(),
            op.as_ptr(),
            file.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            SW_SHOWNORMAL,
        );
    }
}

#[tauri::command]
fn open_music_app(source_app_id: String) -> Result<(), String> {
    let source = source_app_id.to_lowercase();

    if source.contains("cloudmusic") || source.contains("orpheus") || source.contains("netease") {
        return open_netease_music();
    }

    if source.contains("msedge") || source.contains("microsoft edge") || source.contains("edge") {
        shell_execute_open("microsoft-edge:");
        return Ok(());
    }
    if source.contains("chrome") {
        shell_execute_open("chrome.exe");
        return Ok(());
    }
    if source.contains("firefox") {
        shell_execute_open("firefox.exe");
        return Ok(());
    }
    if source.contains("brave") {
        shell_execute_open("brave.exe");
        return Ok(());
    }
    if source.contains("opera") {
        shell_execute_open("opera.exe");
        return Ok(());
    }
    if source.contains("vivaldi") {
        shell_execute_open("vivaldi.exe");
        return Ok(());
    }

    let protocol = if source.contains("spotify") {
        Some("spotify:")
    } else if source.contains("qqmusic") || source.contains("qq音乐") {
        Some("qqmusic://")
    } else if source.contains("kugou") || source.contains("酷狗") {
        Some("kugou://")
    } else if source.contains("kuwo") || source.contains("酷我") {
        Some("kuwo://")
    } else if source.contains("applemusic")
        || source.contains("apple music")
        || source.contains("itunes")
        || source.contains("music.ui")
    {
        Some("music:")
    } else if source.contains("deezer") {
        Some("deezer:")
    } else {
        None
    };

    if let Some(protocol) = protocol {
        shell_execute_open(protocol);
        return Ok(());
    }

    if !source_app_id.trim().is_empty() {
        unsafe {
            let op = std::ffi::OsStr::new("open")
                .encode_wide()
                .chain(Some(0))
                .collect::<Vec<u16>>();
            let file = std::ffi::OsStr::new("explorer.exe")
                .encode_wide()
                .chain(Some(0))
                .collect::<Vec<u16>>();
            let params = std::ffi::OsStr::new(&format!("shell:AppsFolder\\{}", source_app_id))
                .encode_wide()
                .chain(Some(0))
                .collect::<Vec<u16>>();
            ShellExecuteW(
                std::ptr::null_mut(),
                op.as_ptr(),
                file.as_ptr(),
                params.as_ptr(),
                std::ptr::null(),
                SW_SHOWNORMAL,
            );
        }
        return Ok(());
    }

    Ok(())
}

#[tauri::command]
fn open_netease_music() -> Result<(), String> {
    unsafe {
        let mut search = WindowSearch {
            hwnd: std::ptr::null_mut(),
        };
        EnumWindows(
            Some(enum_netease_window_proc),
            &mut search as *mut _ as LPARAM,
        );

        if !search.hwnd.is_null() {
            ShowWindow(search.hwnd, SW_SHOWNORMAL);
            keybd_event(VK_MENU as u8, 0, 0, 0);
            keybd_event(VK_MENU as u8, 0, KEYEVENTF_KEYUP, 0);
            SetForegroundWindow(search.hwnd);
            return Ok(());
        }
    }

    let candidates = [
        "C:\\Program Files\\Netease\\CloudMusic\\cloudmusic.exe",
        "C:\\Program Files (x86)\\Netease\\CloudMusic\\cloudmusic.exe",
    ];

    if let Some(path) = candidates
        .iter()
        .find(|path| std::path::Path::new(path).exists())
    {
        shell_execute_open(path);
    } else {
        shell_execute_open("orpheus://");
    }

    Ok(())
}

#[tauri::command]
fn is_foreground_fullscreen() -> Result<bool, String> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_null() {
            return Ok(false);
        }

        let mut class_name = [0u16; 256];
        let len = GetClassNameW(hwnd, class_name.as_mut_ptr(), class_name.len() as i32);
        let class_str = String::from_utf16_lossy(&class_name[..len as usize]);

        if class_str == "Progman" || class_str == "WorkerW" || class_str == "Shell_TrayWnd" {
            return Ok(false);
        }

        let mut rect: RECT = std::mem::zeroed();
        if GetWindowRect(hwnd, &mut rect) == 0 {
            return Ok(false);
        }

        let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        if monitor.is_null() {
            return Ok(false);
        }

        let mut mi: MONITORINFO = std::mem::zeroed();
        mi.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
        if GetMonitorInfoW(monitor, &mut mi) == 0 {
            return Ok(false);
        }

        let tolerance = 4;
        Ok(rect.left <= mi.rcMonitor.left + tolerance
            && rect.top <= mi.rcMonitor.top + tolerance
            && rect.right >= mi.rcMonitor.right - tolerance
            && rect.bottom >= mi.rcMonitor.bottom - tolerance)
    }
}

#[tauri::command]
fn force_window_topmost(app: tauri::AppHandle) {
    #[cfg(target_os = "windows")]
    {
        fn is_shell_surface_class(class_str: &str) -> bool {
            matches!(
                class_str,
                "Progman"
                    | "WorkerW"
                    | "Shell_TrayWnd"
                    | "Shell_SecondaryTrayWnd"
                    | "Windows.UI.Core.CoreWindow"
                    | "ApplicationFrameWindow"
                    | "XamlExplorerHostIslandWindow"
                    | "Windows.UI.Composition.DesktopWindowContentBridge"
                    | "#32768"
            ) || class_str.contains("Start")
                || class_str.contains("Shell")
        }

        unsafe {
            let fg_hwnd = winapi::um::winuser::GetForegroundWindow();
            if !fg_hwnd.is_null() {
                let mut class_name = [0u16; 256];
                let len = winapi::um::winuser::GetClassNameW(
                    fg_hwnd,
                    class_name.as_mut_ptr(),
                    class_name.len() as i32,
                );
                let class_str = String::from_utf16_lossy(&class_name[..len as usize]);

                let mut rect: RECT = std::mem::zeroed();
                winapi::um::winuser::GetWindowRect(fg_hwnd, &mut rect);

                let monitor = winapi::um::winuser::MonitorFromWindow(
                    fg_hwnd,
                    winapi::um::winuser::MONITOR_DEFAULTTONEAREST,
                );
                let mut mi: winapi::um::winuser::MONITORINFO = std::mem::zeroed();
                mi.cbSize = std::mem::size_of::<winapi::um::winuser::MONITORINFO>() as u32;
                winapi::um::winuser::GetMonitorInfoW(monitor, &mut mi);

                if rect.left == mi.rcMonitor.left
                    && rect.top == mi.rcMonitor.top
                    && rect.right == mi.rcMonitor.right
                    && rect.bottom == mi.rcMonitor.bottom
                {
                    if !is_shell_surface_class(&class_str) {
                        return;
                    }
                }
            }

            if let Some(win) = app.get_webview_window("widget") {
                if let Ok(hwnd) = win.hwnd() {
                    // SWP_NOSIZE | SWP_NOMOVE | SWP_NOACTIVATE | SWP_SHOWWINDOW
                    winapi::um::winuser::SetWindowPos(
                        hwnd.0 as _,
                        -1isize as _,
                        0,
                        0,
                        0,
                        0,
                        0x0053,
                    );
                }
            }
        }
    }
}

// 新增：底层原子化窗口调整指令，彻底消除位移闪烁
#[tauri::command]
fn set_window_bounds(app: tauri::AppHandle, x: i32, y: i32, width: i32, height: i32) {
    #[cfg(target_os = "windows")]
    {
        if let Some(win) = app.get_webview_window("widget") {
            if let Ok(hwnd) = win.hwnd() {
                unsafe {
                    // 0x0014 = SWP_NOACTIVATE (0x0010) | SWP_NOZORDER (0x0004)
                    // 确保同时修改坐标和尺寸时，不抢占用户焦点，不打乱窗口层级
                    winapi::um::winuser::SetWindowPos(
                        hwnd.0 as _,
                        std::ptr::null_mut(),
                        x,
                        y,
                        width,
                        height,
                        0x0014,
                    );
                }
            }
        }
    }
}

struct AppState {
    networks: Mutex<Networks>,
    system: Mutex<System>,
    hardware_sensor_cache: Arc<Mutex<HardwareSensorCache>>,
    hardware_monitor_roots: Mutex<Vec<PathBuf>>,
}

#[tauri::command]
fn get_hardware_stats(state: State<'_, AppState>) -> HardwareStats {
    let mut sys = state.system.lock().unwrap();
    sys.refresh_cpu_usage();
    sys.refresh_memory();

    let mut stats = HardwareStats::default();
    stats.cpu_usage = sys.global_cpu_info().cpu_usage();
    let hardware_monitor_roots = state.hardware_monitor_roots.lock().unwrap().clone();
    let aux_stats = query_hardware_aux_cached(
        Arc::clone(&state.hardware_sensor_cache),
        hardware_monitor_roots,
    );
    stats.cpu_temperature = aux_stats.cpu_temperature;
    stats.cpu_fan_rpm = aux_stats.cpu_fan_rpm;
    stats.gpu_fan_rpm = aux_stats.gpu_fan_rpm;
    apply_aux_gpu_fallbacks(&mut stats, aux_stats);
    stats.used_memory = sys.used_memory();
    stats.total_memory = sys.total_memory();
    stats
}

#[tauri::command]
fn get_network_stats(state: State<'_, AppState>) -> (u64, u64) {
    let mut networks = state.networks.lock().unwrap();
    networks.refresh_list();

    let mut total_rx = 0;
    let mut total_tx = 0;

    for (_interface_name, data) in networks.iter() {
        total_rx += data.total_received();
        total_tx += data.total_transmitted();
    }

    (total_rx, total_tx)
}

#[tauri::command]
fn get_network_latency() -> Result<u128, String> {
    let addr: SocketAddr = "223.5.5.5:53".parse().unwrap();
    let timeout = Duration::from_millis(1500);

    let start = Instant::now();
    match TcpStream::connect_timeout(&addr, timeout) {
        Ok(_) => Ok(start.elapsed().as_millis()),
        Err(_) => Err("Timeout".to_string()),
    }
}

fn read_power_status() -> Option<(bool, bool, bool, Option<u8>)> {
    let mut status: SYSTEM_POWER_STATUS = unsafe { std::mem::zeroed() };
    let ok = unsafe { GetSystemPowerStatus(&mut status as *mut SYSTEM_POWER_STATUS) };
    if ok == 0 {
        return None;
    }

    let has_battery = status.BatteryFlag != 128 && status.BatteryFlag != 255;
    let is_on_battery = status.ACLineStatus == 0;
    let is_charging = has_battery && status.ACLineStatus == 1;
    let battery_percent = if has_battery && status.BatteryLifePercent <= 100 {
        Some(status.BatteryLifePercent)
    } else {
        None
    };

    Some((has_battery, is_charging, is_on_battery, battery_percent))
}

unsafe fn read_master_volume_percent() -> Option<u8> {
    let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

    let enumerator: IMMDeviceEnumerator =
        windows::Win32::System::Com::CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
            .ok()?;
    let device = enumerator
        .GetDefaultAudioEndpoint(eRender, eMultimedia)
        .ok()?;
    let endpoint: IAudioEndpointVolume = device.Activate(CLSCTX_ALL, None).ok()?;
    let scalar = endpoint.GetMasterVolumeLevelScalar().ok()?;
    let percent = (scalar.clamp(0.0, 1.0) * 100.0).round() as u8;
    Some(percent)
}

#[tauri::command]
fn fetch_system_status() -> Result<SystemStatusData, String> {
    let (has_battery, is_charging, is_on_battery, battery_percent) =
        read_power_status().unwrap_or((false, false, false, None));
    let volume_percent = unsafe { read_master_volume_percent() };

    Ok(SystemStatusData {
        has_battery,
        is_charging,
        is_on_battery,
        battery_percent,
        volume_percent,
    })
}

fn extract_release_tag_from_url(url: &str) -> Option<String> {
    let marker = "/releases/tag/";
    let tag = url.split(marker).nth(1)?;
    let tag = tag.split(&['?', '#'][..]).next().unwrap_or(tag).trim();
    if tag.is_empty() {
        None
    } else {
        Some(tag.to_string())
    }
}

fn release_download_url_from_tag(tag_name: &str) -> Option<String> {
    let version = tag_name.trim_start_matches('v');
    if version.is_empty() {
        return None;
    }
    Some(format!(
        "https://github.com/CHmua/FlowIsland/releases/download/{}/FlowIsland_{}_x64-setup.exe",
        tag_name, version
    ))
}

fn release_asset_from_json(asset: &serde_json::Value) -> Option<ReleaseAsset> {
    Some(ReleaseAsset {
        download_url: asset.get("browser_download_url")?.as_str()?.to_string(),
        digest: asset
            .get("digest")
            .and_then(|value| value.as_str())
            .map(str::to_string),
        size: asset.get("size").and_then(|value| value.as_u64()),
    })
}

fn pick_installer_asset(data: &serde_json::Value) -> Option<ReleaseAsset> {
    let assets = data.get("assets")?.as_array()?;
    assets.iter().find_map(|asset| {
        let name = asset.get("name")?.as_str()?.to_lowercase();
        if name.ends_with(".exe") && name.contains("setup") {
            release_asset_from_json(asset)
        } else {
            None
        }
    })
}

fn github_rate_limit_reset_text(headers: &reqwest::header::HeaderMap) -> Option<String> {
    let reset = headers
        .get("x-ratelimit-reset")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())?;
    let now = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();
    let wait_minutes = reset.saturating_sub(now).div_ceil(60);
    if wait_minutes == 0 {
        Some("稍后".to_string())
    } else {
        Some(format!("约 {} 分钟后", wait_minutes))
    }
}

async fn fetch_release_from_github_latest_page(
    client: &reqwest::Client,
) -> Result<ReleaseInfo, String> {
    let response = client
        .get("https://github.com/CHmua/FlowIsland/releases/latest")
        .send()
        .await
        .map_err(|err| format!("无法访问 GitHub Releases 页面：{}", err))?;

    let final_url = response.url().to_string();
    if !response.status().is_success() {
        return Err(format!(
            "GitHub Releases 页面返回 HTTP {}",
            response.status().as_u16()
        ));
    }

    let tag_name = extract_release_tag_from_url(&final_url)
        .ok_or_else(|| "没有从 GitHub Releases 页面解析到最新版本号".to_string())?;

    Ok(ReleaseInfo {
        name: tag_name.trim_start_matches('v').to_string(),
        html_url: final_url,
        download_url: release_download_url_from_tag(&tag_name),
        asset_digest: None,
        asset_size: None,
        tag_name,
        source: "github-page".to_string(),
    })
}

#[tauri::command]
async fn fetch_latest_release_info() -> Result<ReleaseInfo, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(8))
        .user_agent(format!("FlowIsland/{}", env!("CARGO_PKG_VERSION")))
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .map_err(|err| format!("创建更新检查客户端失败：{}", err))?;

    let api_response = client
        .get("https://api.github.com/repos/CHmua/FlowIsland/releases/latest")
        .header("Accept", "application/vnd.github+json")
        .send()
        .await;

    match api_response {
        Ok(response) if response.status().is_success() => {
            let data = response
                .json::<serde_json::Value>()
                .await
                .map_err(|err| format!("解析更新信息失败：{}", err))?;
            let tag_name = data
                .get("tag_name")
                .and_then(|value| value.as_str())
                .unwrap_or_default()
                .to_string();
            if tag_name.trim().is_empty() {
                return Err("GitHub Release 没有版本号。".to_string());
            }
            let installer_asset = pick_installer_asset(&data);
            Ok(ReleaseInfo {
                name: data
                    .get("name")
                    .and_then(|value| value.as_str())
                    .unwrap_or(&tag_name)
                    .to_string(),
                html_url: data
                    .get("html_url")
                    .and_then(|value| value.as_str())
                    .unwrap_or("https://github.com/CHmua/FlowIsland/releases/latest")
                    .to_string(),
                download_url: installer_asset
                    .as_ref()
                    .map(|asset| asset.download_url.clone())
                    .or_else(|| release_download_url_from_tag(&tag_name)),
                asset_digest: installer_asset
                    .as_ref()
                    .and_then(|asset| asset.digest.clone()),
                asset_size: installer_asset.and_then(|asset| asset.size),
                tag_name,
                source: "github-api".to_string(),
            })
        }
        Ok(response) => {
            let status = response.status().as_u16();
            let reset_text = github_rate_limit_reset_text(response.headers());
            let detail = response.text().await.unwrap_or_default();

            if let Ok(release) = fetch_release_from_github_latest_page(&client).await {
                return Ok(release);
            }

            if status == 403 && detail.to_lowercase().contains("rate limit") {
                let retry_text = reset_text.unwrap_or_else(|| "稍后".to_string());
                return Err(format!(
                    "GitHub 临时限制了未登录检查更新的访问频率，请{}再试。",
                    retry_text
                ));
            }
            if status == 404 {
                return Err(
                    "没有找到公开的更新源。请确认 GitHub 仓库已设为 Public，并且已经发布 Release。"
                        .to_string(),
                );
            }
            Err(format!("更新源返回异常状态：HTTP {}。", status))
        }
        Err(err) => match fetch_release_from_github_latest_page(&client).await {
            Ok(release) => Ok(release),
            Err(fallback_err) => Err(format!(
                "无法连接更新源：{}；兜底检查也失败：{}",
                err, fallback_err
            )),
        },
    }
}

fn validate_update_download_url(download_url: &str) -> Result<reqwest::Url, String> {
    let url = reqwest::Url::parse(download_url).map_err(|_| "更新包下载地址无效。".to_string())?;
    if url.scheme() != "https" || url.host_str() != Some("github.com") {
        return Err("为保证安全，只允许从 FlowIsland 官方 GitHub Release 下载更新。".to_string());
    }

    let segments = url
        .path_segments()
        .map(|segments| segments.collect::<Vec<_>>())
        .unwrap_or_default();
    let is_official_release = segments.len() == 6
        && segments[0].eq_ignore_ascii_case("CHmua")
        && segments[1].eq_ignore_ascii_case("FlowIsland")
        && segments[2] == "releases"
        && segments[3] == "download"
        && !segments[4].is_empty()
        && !segments[4].contains("..")
        && segments[5].to_ascii_lowercase().starts_with("flowisland_")
        && segments[5].to_ascii_lowercase().ends_with("_x64-setup.exe");
    if !is_official_release {
        return Err("更新包不是 FlowIsland 官方 x64 安装程序。".to_string());
    }
    Ok(url)
}

fn normalize_expected_sha256(expected_digest: Option<String>) -> Result<Option<String>, String> {
    let Some(raw_digest) = expected_digest else {
        return Ok(None);
    };
    let digest = raw_digest
        .trim()
        .strip_prefix("sha256:")
        .unwrap_or(raw_digest.trim())
        .to_ascii_lowercase();
    if digest.len() != 64
        || !digest
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        return Err("GitHub 返回的更新包校验值无效。".to_string());
    }
    Ok(Some(digest))
}

#[cfg(test)]
mod update_validation_tests {
    use super::{normalize_expected_sha256, validate_update_download_url};

    #[test]
    fn accepts_official_flowisland_installer() {
        let url = "https://github.com/CHmua/FlowIsland/releases/download/v2.3.11/FlowIsland_2.3.11_x64-setup.exe";
        assert!(validate_update_download_url(url).is_ok());
    }

    #[test]
    fn rejects_other_repositories_and_files() {
        assert!(validate_update_download_url(
            "https://github.com/someone/FlowIsland/releases/download/v9/FlowIsland_9_x64-setup.exe"
        )
        .is_err());
        assert!(validate_update_download_url(
            "https://github.com/CHmua/FlowIsland/releases/download/v9/notes.txt"
        )
        .is_err());
    }

    #[test]
    fn normalizes_github_sha256_digest() {
        let digest = format!("sha256:{}", "a".repeat(64));
        assert_eq!(
            normalize_expected_sha256(Some(digest)).unwrap(),
            Some("a".repeat(64))
        );
        assert!(normalize_expected_sha256(Some("sha256:invalid".to_string())).is_err());
    }
}

fn emit_update_progress(
    app: &tauri::AppHandle,
    status: &str,
    downloaded: u64,
    total: Option<u64>,
    percent: u8,
) {
    let _ = app.emit(
        "update-download-progress",
        UpdateDownloadProgress {
            status: status.to_string(),
            downloaded,
            total,
            percent,
        },
    );
}

#[tauri::command]
async fn download_and_install_update(
    app: tauri::AppHandle,
    download_url: String,
    version: String,
    expected_digest: Option<String>,
    expected_size: Option<u64>,
) -> Result<(), String> {
    const MIN_INSTALLER_SIZE: u64 = 256 * 1024;
    const MAX_INSTALLER_SIZE: u64 = 300 * 1024 * 1024;

    let url = validate_update_download_url(&download_url)?;
    let expected_sha256 = normalize_expected_sha256(expected_digest)?;
    let safe_version = version
        .trim_start_matches('v')
        .chars()
        .filter(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_')
        })
        .collect::<String>();
    if safe_version.is_empty() {
        return Err("更新版本号无效。".to_string());
    }

    let temp_dir = env::temp_dir().join("FlowIslandUpdates");
    tokio::fs::create_dir_all(&temp_dir)
        .await
        .map_err(|err| format!("无法创建更新缓存目录：{}", err))?;
    let installer_path = temp_dir.join(format!("FlowIsland_{}_x64-setup.exe", safe_version));
    let partial_path = installer_path.with_extension("exe.part");
    let _ = tokio::fs::remove_file(&partial_path).await;

    emit_update_progress(&app, "正在连接 GitHub…", 0, expected_size, 0);
    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(15))
        .timeout(Duration::from_secs(15 * 60))
        .user_agent(format!("FlowIsland/{} updater", env!("CARGO_PKG_VERSION")))
        .redirect(reqwest::redirect::Policy::limited(8))
        .build()
        .map_err(|err| format!("无法创建更新下载器：{}", err))?;
    let mut response = client
        .get(url)
        .header("Accept", "application/octet-stream")
        .send()
        .await
        .map_err(|err| format!("连接更新服务器失败：{}", err))?;
    if !response.status().is_success() {
        return Err(format!(
            "下载更新包失败：HTTP {}",
            response.status().as_u16()
        ));
    }
    let final_host = response.url().host_str().unwrap_or_default();
    let is_trusted_download_host = final_host == "github.com"
        || final_host == "release-assets.githubusercontent.com"
        || final_host.ends_with(".githubusercontent.com");
    if response.url().scheme() != "https" || !is_trusted_download_host {
        return Err("更新下载被重定向到非 GitHub 地址，已停止安装。".to_string());
    }

    let response_size = response.content_length();
    if let (Some(server_size), Some(asset_size)) = (response_size, expected_size) {
        if server_size != asset_size {
            return Err("更新包大小与 GitHub Release 信息不一致。".to_string());
        }
    }
    let total = response_size.or(expected_size);
    if total.is_some_and(|size| size < MIN_INSTALLER_SIZE || size > MAX_INSTALLER_SIZE) {
        return Err("更新包大小异常，已停止安装。".to_string());
    }

    let mut file = tokio::fs::File::create(&partial_path)
        .await
        .map_err(|err| format!("无法创建更新文件：{}", err))?;
    let mut hasher = Sha256::new();
    let mut downloaded = 0u64;
    let mut last_percent = 0u8;
    let mut executable_header = Vec::with_capacity(2);

    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|err| format!("下载更新包时连接中断：{}", err))?
    {
        if executable_header.len() < 2 {
            let needed = 2 - executable_header.len();
            executable_header.extend_from_slice(&chunk[..chunk.len().min(needed)]);
        }
        downloaded = downloaded.saturating_add(chunk.len() as u64);
        if downloaded > MAX_INSTALLER_SIZE {
            let _ = tokio::fs::remove_file(&partial_path).await;
            return Err("更新包超过允许的最大大小，已停止安装。".to_string());
        }
        hasher.update(&chunk);
        file.write_all(&chunk)
            .await
            .map_err(|err| format!("保存更新包失败：{}", err))?;

        let percent = total
            .filter(|size| *size > 0)
            .map(|size| ((downloaded.saturating_mul(100) / size).min(99)) as u8)
            .unwrap_or(0);
        if percent != last_percent {
            last_percent = percent;
            emit_update_progress(&app, "正在下载更新…", downloaded, total, percent);
        }
    }
    file.flush()
        .await
        .map_err(|err| format!("写入更新包失败：{}", err))?;
    drop(file);

    if downloaded < MIN_INSTALLER_SIZE || executable_header.as_slice() != b"MZ" {
        let _ = tokio::fs::remove_file(&partial_path).await;
        return Err("下载内容不是有效的 Windows 安装程序。".to_string());
    }
    if expected_size.is_some_and(|size| size != downloaded) {
        let _ = tokio::fs::remove_file(&partial_path).await;
        return Err("更新包下载不完整，已停止安装。".to_string());
    }

    emit_update_progress(&app, "正在校验更新包…", downloaded, total, 99);
    let actual_sha256 = format!("{:x}", hasher.finalize());
    if expected_sha256.is_some_and(|expected| expected != actual_sha256) {
        let _ = tokio::fs::remove_file(&partial_path).await;
        return Err("更新包 SHA-256 校验失败，已阻止安装。".to_string());
    }

    let _ = tokio::fs::remove_file(&installer_path).await;
    tokio::fs::rename(&partial_path, &installer_path)
        .await
        .map_err(|err| format!("准备更新安装包失败：{}", err))?;
    emit_update_progress(&app, "校验完成，正在启动安装…", downloaded, total, 100);

    let mut command = Command::new(&installer_path);
    command.args(["/S", "/UPDATE"]);
    #[cfg(target_os = "windows")]
    command.creation_flags(0x08000000);
    command
        .spawn()
        .map_err(|err| format!("无法启动更新安装程序：{}", err))?;

    tokio::time::sleep(Duration::from_millis(350)).await;
    app.exit(0);
    Ok(())
}

#[tauri::command]
fn is_widget_visible(app: tauri::AppHandle) -> bool {
    match app.get_webview_window("widget") {
        Some(win) => win.is_visible().unwrap_or(false),
        None => false,
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let networks = Networks::new_with_refreshed_list();
    let mut system = System::new_all();
    system.refresh_cpu_usage();

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|_app, _args, _cwd| {}))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--autostart"]),
        ))
        .manage(AppState {
            networks: Mutex::new(networks),
            system: Mutex::new(system),
            hardware_sensor_cache: Arc::new(Mutex::new(HardwareSensorCache::default())),
            hardware_monitor_roots: Mutex::new(Vec::new()),
        })
        .invoke_handler(tauri::generate_handler![
            get_network_stats,
            is_widget_visible,
            get_network_latency,
            fetch_latest_release_info,
            download_and_install_update,
            fetch_system_status,
            fetch_music_info,
            fetch_netease_music_info,
            control_system_media,
            fetch_synced_lyrics,
            get_random_cover_url,
            fetch_latest_notification,
            get_hardware_stats,
            open_app_by_aumid,
            open_music_app,
            open_netease_music,
            is_foreground_fullscreen,
            force_window_topmost,
            set_window_bounds,
        ])
        .setup(|app| {
            cleanup_legacy_shortcuts();

            let mut hardware_monitor_roots = Vec::new();
            if let Ok(resource_dir) = app.path().resource_dir() {
                hardware_monitor_roots.push(resource_dir);
            }
            if let Some(state) = app.try_state::<AppState>() {
                *state.hardware_monitor_roots.lock().unwrap() = hardware_monitor_roots;
            }
            hide_hardware_monitor_windows();

            let args: Vec<String> = std::env::args().collect();
            let is_autostart = args.iter().any(|arg| arg == "--autostart");

            if let Some(main_window) = app.get_webview_window("main") {
                if !is_autostart {
                    let _ = main_window.show();
                    let _ = main_window.set_focus();
                }
            }

            let quit_item = MenuItem::with_id(app, "quit", "强制退出", true, None::<&str>)?;
            let tray_menu = Menu::with_items(app, &[&quit_item])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("FlowIsland")
                .menu(&tray_menu)
                .on_menu_event(move |_app_handle, event| {
                    if event.id == "quit" {
                        std::process::exit(0);
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        ..
                    } = event
                    {
                        if let Some(main_window) = tray.app_handle().get_webview_window("main") {
                            let _ = main_window.show();
                            let _ = main_window.unminimize();
                            let _ = main_window.set_focus();
                        }
                    }
                })
                .build(app)?;

            if let Some(main_window) = app.get_webview_window("main") {
                let w_clone = main_window.clone();
                main_window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = w_clone.hide();
                    }
                });
            }

            let widget_window = ensure_widget_window(app)?;
            apply_widget_window_style(&widget_window);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
