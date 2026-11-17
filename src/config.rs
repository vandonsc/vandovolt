use std::{env, fs, path::PathBuf, ptr::null_mut};
use windows_sys::Win32::System::Registry::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Style {
    IconPercent,
    Percent,
    Gauge,
}

#[derive(Clone, Copy)]
pub struct Config {
    pub style: Style,
    pub interval_minutes: u32,
    pub start_with_windows: bool,
    pub alerts: bool,
    pub hover_panel: bool,
}

fn w(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}

fn paths() -> Option<(PathBuf, PathBuf)> {
    let local = PathBuf::from(env::var_os("LOCALAPPDATA")?);
    Some((
        local.join(r"VandoVolt\config.ini"),
        local.join(r"PeripheralBattery\config.ini"),
    ))
}

fn registry_value_exists(name: &str) -> bool {
    let mut size = 0u32;
    unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            w("Software\\Microsoft\\Windows\\CurrentVersion\\Run").as_ptr(),
            w(name).as_ptr(),
            RRF_RT_REG_SZ,
            null_mut(),
            null_mut(),
            &mut size,
        ) == 0
    }
}

fn startup_exists() -> bool {
    registry_value_exists("VandoVolt")
}

fn migrate_config() -> Option<PathBuf> {
    let (current, legacy) = paths()?;
    if current.exists() || !legacy.is_file() {
        return Some(current);
    }
    let directory = current.parent()?;
    if fs::create_dir_all(directory).is_err() {
        return Some(current);
    }
    let temporary = directory.join("config.ini.migrating");
    if fs::copy(&legacy, &temporary).is_ok()
        && fs::read(&legacy).ok() == fs::read(&temporary).ok()
        && fs::rename(&temporary, &current).is_ok()
    {
        return Some(current);
    }
    let _ = fs::remove_file(temporary);
    Some(current)
}

impl Default for Config {
    fn default() -> Self {
        Self {
            style: Style::IconPercent,
            interval_minutes: 5,
            start_with_windows: startup_exists(),
            alerts: true,
            hover_panel: true,
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let mut result = Self::default();
        let Some(path) = migrate_config() else {
            return result;
        };
        migrate_startup_entry();
        let Ok(text) = fs::read_to_string(path) else {
            return result;
        };
        for line in text.lines() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            match key.trim() {
                "style" => {
                    result.style = match value.trim() {
                        "percent" => Style::Percent,
                        "gauge" => Style::Gauge,
                        _ => Style::IconPercent,
                    }
                }
                "interval" => {
                    if let Ok(value) = value.trim().parse() {
                        if matches!(value, 1 | 5 | 10 | 15 | 30) {
                            result.interval_minutes = value;
                        }
                    }
                }
                "startup" => result.start_with_windows = value.trim() == "true",
                "alerts" => result.alerts = value.trim() != "false",
                "hover_panel" => result.hover_panel = value.trim() != "false",
                _ => {}
            }
        }
        // The registry is authoritative because installers or the user may have
        // changed this setting outside the application.
        result.start_with_windows = startup_exists();
        result
    }

    pub fn save(self) {
        let Some((path, _)) = paths() else { return };
        if let Some(directory) = path.parent() {
            let _ = fs::create_dir_all(directory);
        }
        let style = match self.style {
            Style::IconPercent => "icon_percent",
            Style::Percent => "percent",
            Style::Gauge => "gauge",
        };
        let text = format!(
            "style={style}\ninterval={}\nstartup={}\nalerts={}\nhover_panel={}\n",
            self.interval_minutes, self.start_with_windows, self.alerts, self.hover_panel
        );
        let _ = fs::write(path, text);
    }

    pub fn apply_startup(enabled: bool) -> bool {
        let key_name = w("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
        let value_name = w("VandoVolt");
        let legacy_value_name = w("PeripheralBattery");
        let mut key: HKEY = null_mut();
        unsafe {
            if RegCreateKeyExW(
                HKEY_CURRENT_USER,
                key_name.as_ptr(),
                0,
                std::ptr::null(),
                0,
                KEY_SET_VALUE,
                null_mut(),
                &mut key,
                null_mut(),
            ) != 0
            {
                return false;
            }
            let result = if enabled {
                let Ok(exe) = env::current_exe() else {
                    RegCloseKey(key);
                    return false;
                };
                let value = w(&format!("\"{}\"", exe.display()));
                let written = RegSetValueExW(
                    key,
                    value_name.as_ptr(),
                    0,
                    REG_SZ,
                    value.as_ptr() as *const u8,
                    (value.len() * 2) as u32,
                ) == 0;
                if written {
                    let _ = RegDeleteValueW(key, legacy_value_name.as_ptr());
                }
                written
            } else {
                let current_removed =
                    RegDeleteValueW(key, value_name.as_ptr()) == 0 || !startup_exists();
                let legacy_removed = RegDeleteValueW(key, legacy_value_name.as_ptr()) == 0
                    || !registry_value_exists("PeripheralBattery");
                current_removed && legacy_removed
            };
            RegCloseKey(key);
            result
        }
    }
}

fn migrate_startup_entry() {
    if registry_value_exists("PeripheralBattery") {
        let _ = Config::apply_startup(true);
    }
}
