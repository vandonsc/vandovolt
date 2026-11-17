#![windows_subsystem = "windows"]
mod config;
mod devices;
mod icon;
mod panel;

use config::{Config, Style};
use devices::Level;
use std::{
    mem::{size_of, zeroed},
    ptr::{null, null_mut},
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
    thread,
    time::Instant,
};
use windows_sys::Win32::{
    Foundation::*,
    System::{LibraryLoader::GetModuleHandleW, Threading::CreateMutexW},
    UI::{HiDpi::*, Shell::*, WindowsAndMessaging::*},
};

const UPDATE: u32 = WM_APP + 1;
const TRAY: u32 = WM_APP + 2;
const DEVICE_IDS: [u32; 2] = [1, 2];
const DEVICE_NAMES: [&str; 2] = ["HyperX Cloud III Wireless", "NZXT Lift Elite Wireless"];
const IDM_STYLE_ICON_PERCENT: usize = 101;
const IDM_STYLE_PERCENT: usize = 102;
const IDM_STYLE_GAUGE: usize = 103;
const IDM_INTERVAL_BASE: usize = 200;
const IDM_STARTUP: usize = 301;
const IDM_ALERTS: usize = 302;
const IDM_HOVER_PANEL: usize = 303;
const IDM_REFRESH: usize = 401;
const IDM_EXIT: usize = 402;

static LEVELS: Mutex<[Option<Level>; 2]> = Mutex::new([None, None]);
static SETTINGS: Mutex<Option<Config>> = Mutex::new(None);
static BUSY: AtomicBool = AtomicBool::new(false);
static ALERTED: Mutex<[bool; 2]> = Mutex::new([false, false]);
static TASKBAR_CREATED: Mutex<u32> = Mutex::new(0);
static PANEL: std::sync::atomic::AtomicIsize = std::sync::atomic::AtomicIsize::new(0);
static HOVER_DEVICE: Mutex<Option<usize>> = Mutex::new(None);
static LAST_UPDATE: Mutex<Option<Instant>> = Mutex::new(None);
static TOOLTIP_FALLBACK: AtomicBool = AtomicBool::new(false);

fn w(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}
fn copy_w(dst: &mut [u16], value: &str) {
    dst.fill(0);
    let cap = dst.len().saturating_sub(1);
    for (a, b) in dst.iter_mut().take(cap).zip(value.encode_utf16()) {
        *a = b;
    }
}
fn settings() -> Config {
    SETTINGS.lock().unwrap().expect("settings initialized")
}
fn interval_ms(config: Config) -> u32 {
    config.interval_minutes * 60_000
}

fn poll(hwnd: HWND) {
    if BUSY.swap(true, Ordering::AcqRel) {
        return;
    }
    let handle = hwnd as usize;
    thread::spawn(move || {
        let next = devices::read();
        *LAST_UPDATE.lock().unwrap() = Some(Instant::now());
        let changed = {
            let mut levels = LEVELS.lock().unwrap();
            let changed = *levels != next;
            *levels = next;
            changed
        };
        BUSY.store(false, Ordering::Release);
        if changed {
            unsafe {
                PostMessageW(handle as HWND, UPDATE, 0, 0);
            }
        }
    });
}

unsafe fn notify_data(hwnd: HWND, id: u32) -> NOTIFYICONDATAW {
    let mut data: NOTIFYICONDATAW = zeroed();
    data.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
    data.hWnd = hwnd;
    data.uID = id;
    data.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
    if !settings().hover_panel || TOOLTIP_FALLBACK.load(Ordering::Acquire) {
        data.uFlags |= NIF_SHOWTIP;
    }
    data.uCallbackMessage = TRAY;
    data.Anonymous.uVersion = NOTIFYICON_VERSION_4;
    data
}
unsafe fn update_icon(hwnd: HWND, index: usize, operation: u32) {
    let level = LEVELS.lock().unwrap()[index];
    let config = settings();
    let mut data = notify_data(hwnd, DEVICE_IDS[index]);
    data.hIcon = icon::create(index, level, config.style);
    let status = match level {
        Some(value) if value.charging => format!("{}% — carregando", value.percent),
        Some(value) => format!("{}%", value.percent),
        None => "indisponível".into(),
    };
    copy_w(
        &mut data.szTip,
        &format!("{} — {}", DEVICE_NAMES[index], status),
    );
    Shell_NotifyIconW(operation, &data);
    if operation == NIM_ADD {
        data.uFlags = NIF_MESSAGE;
        Shell_NotifyIconW(NIM_SETVERSION, &data);
    }
    if !data.hIcon.is_null() {
        DestroyIcon(data.hIcon);
    }
}
unsafe fn update_icons(hwnd: HWND, operation: u32) {
    for index in 0..2 {
        update_icon(hwnd, index, operation);
    }
}
unsafe fn delete_icons(hwnd: HWND) {
    for id in DEVICE_IDS {
        let data = notify_data(hwnd, id);
        Shell_NotifyIconW(NIM_DELETE, &data);
    }
}
unsafe fn alert(hwnd: HWND, index: usize, level: Level) {
    let mut data = notify_data(hwnd, DEVICE_IDS[index]);
    data.uFlags = NIF_INFO;
    copy_w(&mut data.szInfoTitle, "Bateria baixa");
    copy_w(
        &mut data.szInfo,
        &format!("{}: {}%", DEVICE_NAMES[index], level.percent),
    );
    data.dwInfoFlags = NIIF_WARNING;
    Shell_NotifyIconW(NIM_MODIFY, &data);
}

unsafe fn add_item(menu: HMENU, id: usize, text: &str, flags: u32) {
    AppendMenuW(menu, flags, id, w(text).as_ptr());
}
unsafe fn popup_menu(hwnd: HWND) {
    hide_panel(hwnd);
    let config = settings();
    let levels = *LEVELS.lock().unwrap();
    let menu = CreatePopupMenu();
    add_item(menu, 0, "Peripheral Battery", MF_STRING | MF_DISABLED);
    let headset = levels[0].map_or_else(
        || "Headset: indisponível".into(),
        |l| {
            format!(
                "Headset: {}%{}",
                l.percent,
                if l.charging { " (carregando)" } else { "" }
            )
        },
    );
    let mouse = levels[1].map_or_else(
        || "Mouse: indisponível".into(),
        |l| {
            format!(
                "Mouse: {}%{}",
                l.percent,
                if l.charging { " (carregando)" } else { "" }
            )
        },
    );
    add_item(menu, 0, &headset, MF_STRING | MF_DISABLED);
    add_item(menu, 0, &mouse, MF_STRING | MF_DISABLED);
    AppendMenuW(menu, MF_SEPARATOR, 0, null());

    let styles = CreatePopupMenu();
    add_item(
        styles,
        IDM_STYLE_ICON_PERCENT,
        "Ícone + porcentagem",
        MF_STRING
            | if config.style == Style::IconPercent {
                MF_CHECKED
            } else {
                0
            },
    );
    add_item(
        styles,
        IDM_STYLE_PERCENT,
        "Somente porcentagem",
        MF_STRING
            | if config.style == Style::Percent {
                MF_CHECKED
            } else {
                0
            },
    );
    add_item(
        styles,
        IDM_STYLE_GAUGE,
        "Ícone com indicador de carga",
        MF_STRING
            | if config.style == Style::Gauge {
                MF_CHECKED
            } else {
                0
            },
    );
    AppendMenuW(
        menu,
        MF_POPUP,
        styles as usize,
        w("Estilo de exibição").as_ptr(),
    );

    let intervals = CreatePopupMenu();
    for minutes in [1u32, 5, 10, 15, 30] {
        let label = if minutes == 1 {
            "1 minuto".into()
        } else {
            format!(
                "{} minutos{}",
                minutes,
                if minutes == 5 { " (padrão)" } else { "" }
            )
        };
        add_item(
            intervals,
            IDM_INTERVAL_BASE + minutes as usize,
            &label,
            MF_STRING
                | if config.interval_minutes == minutes {
                    MF_CHECKED
                } else {
                    0
                },
        );
    }
    AppendMenuW(
        menu,
        MF_POPUP,
        intervals as usize,
        w("Intervalo de atualização").as_ptr(),
    );
    add_item(
        menu,
        IDM_STARTUP,
        "Iniciar com o Windows",
        MF_STRING
            | if config.start_with_windows {
                MF_CHECKED
            } else {
                0
            },
    );
    add_item(
        menu,
        IDM_ALERTS,
        "Alertas de bateria baixa",
        MF_STRING | if config.alerts { MF_CHECKED } else { 0 },
    );
    add_item(
        menu,
        IDM_HOVER_PANEL,
        "Exibir painel de bateria ao passar o mouse",
        MF_STRING | if config.hover_panel { MF_CHECKED } else { 0 },
    );
    AppendMenuW(menu, MF_SEPARATOR, 0, null());
    add_item(menu, IDM_REFRESH, "Atualizar agora", MF_STRING);
    add_item(menu, IDM_EXIT, "Sair", MF_STRING);

    let mut point: POINT = zeroed();
    GetCursorPos(&mut point);
    SetForegroundWindow(hwnd);
    let command = TrackPopupMenu(
        menu,
        TPM_RETURNCMD | TPM_RIGHTBUTTON,
        point.x,
        point.y,
        0,
        hwnd,
        null(),
    );
    DestroyMenu(menu);
    handle_command(hwnd, command as usize);
    PostMessageW(hwnd, WM_NULL, 0, 0);
}
unsafe fn handle_command(hwnd: HWND, command: usize) {
    let mut config = settings();
    let mut icons_changed = false;
    let mut timer_changed = false;
    match command {
        IDM_STYLE_ICON_PERCENT => {
            config.style = Style::IconPercent;
            icons_changed = true;
        }
        IDM_STYLE_PERCENT => {
            config.style = Style::Percent;
            icons_changed = true;
        }
        IDM_STYLE_GAUGE => {
            config.style = Style::Gauge;
            icons_changed = true;
        }
        IDM_STARTUP => {
            let enabled = !config.start_with_windows;
            if Config::apply_startup(enabled) {
                config.start_with_windows = enabled;
            }
        }
        IDM_ALERTS => {
            config.alerts = !config.alerts;
            if !config.alerts {
                *ALERTED.lock().unwrap() = [false, false];
            }
        }
        IDM_HOVER_PANEL => {
            config.hover_panel = !config.hover_panel;
            TOOLTIP_FALLBACK.store(false, Ordering::Release);
            icons_changed = true;
            if !config.hover_panel {
                hide_panel(hwnd);
            }
        }
        IDM_REFRESH => {
            poll(hwnd);
            return;
        }
        IDM_EXIT => {
            DestroyWindow(hwnd);
            return;
        }
        value
            if value > IDM_INTERVAL_BASE
                && matches!(value - IDM_INTERVAL_BASE, 1 | 5 | 10 | 15 | 30) =>
        {
            config.interval_minutes = (value - IDM_INTERVAL_BASE) as u32;
            timer_changed = true;
        }
        _ => return,
    }
    *SETTINGS.lock().unwrap() = Some(config);
    config.save();
    if timer_changed {
        SetTimer(hwnd, 1, interval_ms(config), None);
    }
    if icons_changed {
        for index in 0..2 {
            update_icon(hwnd, index, NIM_MODIFY);
        }
    }
}

unsafe fn hide_panel(hwnd: HWND) {
    KillTimer(hwnd, 2);
    *HOVER_DEVICE.lock().unwrap() = None;
    panel::hide(PANEL.load(Ordering::Acquire) as HWND);
}

unsafe fn schedule_panel(hwnd: HWND, device: usize) {
    if !settings().hover_panel || TOOLTIP_FALLBACK.load(Ordering::Acquire) {
        return;
    }
    *HOVER_DEVICE.lock().unwrap() = Some(device);
    SetTimer(hwnd, 2, 500, None);
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if message == *TASKBAR_CREATED.lock().unwrap() {
        update_icons(hwnd, NIM_ADD);
        return 0;
    }
    match message {
        WM_TIMER => {
            if wparam == 1 {
                poll(hwnd);
            } else if wparam == 2 {
                KillTimer(hwnd, 2);
                if let Some(device) = *HOVER_DEVICE.lock().unwrap() {
                    let level = LEVELS.lock().unwrap()[device];
                    let updated = *LAST_UPDATE.lock().unwrap();
                    if !panel::show(
                        PANEL.load(Ordering::Acquire) as HWND,
                        hwnd,
                        device,
                        level,
                        updated,
                    ) {
                        TOOLTIP_FALLBACK.store(true, Ordering::Release);
                        update_icons(hwnd, NIM_MODIFY);
                    }
                }
            }
            0
        }
        UPDATE => {
            update_icons(hwnd, NIM_MODIFY);
            let config = settings();
            let data = *LEVELS.lock().unwrap();
            let mut flags = ALERTED.lock().unwrap();
            for index in 0..2 {
                match data[index] {
                    Some(level)
                        if config.alerts
                            && level.percent <= 10
                            && !level.charging
                            && !flags[index] =>
                    {
                        alert(hwnd, index, level);
                        flags[index] = true;
                    }
                    Some(level) if level.percent > 10 || level.charging => flags[index] = false,
                    _ => {}
                }
            }
            0
        }
        WM_SETTINGCHANGE | WM_DPICHANGED | WM_DISPLAYCHANGE => {
            update_icons(hwnd, NIM_MODIFY);
            0
        }
        TRAY => {
            let event = (lparam as u32) & 0xffff;
            let id = (lparam as u32) >> 16;
            let device = match id {
                1 => Some(0),
                2 => Some(1),
                _ => None,
            };
            if event == NIN_POPUPOPEN && device.is_some() {
                schedule_panel(hwnd, device.unwrap());
            } else if event == NIN_POPUPCLOSE {
                hide_panel(hwnd);
            } else if event == WM_CONTEXTMENU || event == WM_RBUTTONUP {
                popup_menu(hwnd);
            } else if event == WM_LBUTTONUP {
                poll(hwnd);
            }
            0
        }
        WM_DESTROY => {
            KillTimer(hwnd, 1);
            hide_panel(hwnd);
            let panel = PANEL.swap(0, Ordering::AcqRel) as HWND;
            panel::destroy(panel);
            delete_icons(hwnd);
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, message, wparam, lparam),
    }
}

fn main() {
    unsafe {
        SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        let mutex = CreateMutexW(null(), 0, w("Local\\PeripheralBatteryRust").as_ptr());
        if mutex.is_null() {
            return;
        }
        if GetLastError() == 183 {
            CloseHandle(mutex);
            return;
        }
        *SETTINGS.lock().unwrap() = Some(Config::load());
        *TASKBAR_CREATED.lock().unwrap() = RegisterWindowMessageW(w("TaskbarCreated").as_ptr());
        let instance = GetModuleHandleW(null());
        let class = w("PeripheralBatteryMessageWindow");
        let mut window_class: WNDCLASSW = zeroed();
        window_class.lpfnWndProc = Some(window_proc);
        window_class.hInstance = instance;
        window_class.lpszClassName = class.as_ptr();
        if RegisterClassW(&window_class) == 0 {
            CloseHandle(mutex);
            return;
        }
        // This window is never shown; it only receives shell, timer and application messages.
        let hwnd = CreateWindowExW(
            WS_EX_TOOLWINDOW,
            class.as_ptr(),
            w("Peripheral Battery").as_ptr(),
            WS_OVERLAPPED,
            0,
            0,
            0,
            0,
            null_mut(),
            null_mut(),
            instance,
            null(),
        );
        if hwnd.is_null() {
            CloseHandle(mutex);
            return;
        }
        PANEL.store(panel::create(instance) as isize, Ordering::Release);
        update_icons(hwnd, NIM_ADD);
        SetTimer(hwnd, 1, interval_ms(settings()), None);
        poll(hwnd);
        let mut message: MSG = zeroed();
        while GetMessageW(&mut message, null_mut(), 0, 0) > 0 {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
        CloseHandle(mutex);
    }
}
