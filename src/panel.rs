use crate::devices::Level;
use std::{
    mem::{size_of, zeroed},
    ptr::{null, null_mut},
    sync::Mutex,
    time::Instant,
};
use windows_sys::Win32::{
    Foundation::*,
    Graphics::{Dwm::*, Gdi::*, GdiPlus::*},
    System::Registry::*,
    UI::{HiDpi::GetDpiForWindow, Shell::*, WindowsAndMessaging::*},
};

#[derive(Clone, Copy)]
struct Data {
    device: usize,
    level: Option<Level>,
    updated: Option<Instant>,
}

static DATA: Mutex<Data> = Mutex::new(Data {
    device: 0,
    level: None,
    updated: None,
});
static GDIPLUS_TOKEN: Mutex<usize> = Mutex::new(0);
const NAMES: [&str; 2] = ["HyperX Cloud III Wireless", "NZXT Lift Elite Wireless"];

fn w(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}
unsafe fn scale(hwnd: HWND, value: i32) -> i32 {
    (value * GetDpiForWindow(hwnd) as i32 + 48) / 96
}

fn light_theme() -> bool {
    let mut value = 1u32;
    let mut size = size_of::<u32>() as u32;
    unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            w("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize").as_ptr(),
            w("AppsUseLightTheme").as_ptr(),
            RRF_RT_REG_DWORD,
            null_mut(),
            &mut value as *mut _ as *mut _,
            &mut size,
        ) != 0
            || value != 0
    }
}

unsafe fn start_gdiplus() -> bool {
    let mut token = GDIPLUS_TOKEN.lock().unwrap();
    if *token != 0 {
        return true;
    }
    let input = GdiplusStartupInput {
        GdiplusVersion: 1,
        DebugEventCallback: 0,
        SuppressBackgroundThread: 0,
        SuppressExternalCodecs: 0,
    };
    GdiplusStartup(&mut *token, &input, null_mut()) == 0
}

pub unsafe fn create(instance: HINSTANCE) -> HWND {
    if !start_gdiplus() {
        return null_mut();
    }
    let class = w("VandoVoltHoverPanel");
    let mut wc: WNDCLASSW = zeroed();
    wc.lpfnWndProc = Some(proc);
    wc.hInstance = instance;
    wc.lpszClassName = class.as_ptr();
    wc.hCursor = LoadCursorW(null_mut(), IDC_ARROW);
    RegisterClassW(&wc);
    let hwnd = CreateWindowExW(
        WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
        class.as_ptr(),
        w("VandoVolt - Battery status").as_ptr(),
        WS_POPUP,
        0,
        0,
        1,
        1,
        null_mut(),
        null_mut(),
        instance,
        null(),
    );
    if !hwnd.is_null() {
        let corner = DWMWCP_ROUND;
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE as u32,
            &corner as *const _ as *const _,
            size_of::<i32>() as u32,
        );
    }
    hwnd
}

pub unsafe fn destroy(hwnd: HWND) {
    if !hwnd.is_null() {
        DestroyWindow(hwnd);
    }
    let mut token = GDIPLUS_TOKEN.lock().unwrap();
    if *token != 0 {
        GdiplusShutdown(*token);
        *token = 0;
    }
}

pub unsafe fn show(
    panel: HWND,
    owner: HWND,
    device: usize,
    level: Option<Level>,
    updated: Option<Instant>,
) -> bool {
    if panel.is_null() {
        return false;
    }
    let identifier = NOTIFYICONIDENTIFIER {
        cbSize: size_of::<NOTIFYICONIDENTIFIER>() as u32,
        hWnd: owner,
        uID: (device + 1) as u32,
        guidItem: zeroed(),
    };
    let mut icon_rect: RECT = zeroed();
    if Shell_NotifyIconGetRect(&identifier, &mut icon_rect) < 0 {
        return false;
    }
    *DATA.lock().unwrap() = Data {
        device,
        level,
        updated,
    };
    let width = scale(panel, 348);
    let height = scale(panel, 176);
    let monitor = MonitorFromRect(&icon_rect, MONITOR_DEFAULTTONEAREST);
    let mut info: MONITORINFO = zeroed();
    info.cbSize = size_of::<MONITORINFO>() as u32;
    GetMonitorInfoW(monitor, &mut info);
    let mut x = (icon_rect.left + icon_rect.right - width) / 2;
    x = x.clamp(info.rcWork.left, info.rcWork.right - width);
    let mut y = icon_rect.top - height - scale(panel, 10);
    if y < info.rcWork.top {
        y = icon_rect.bottom + scale(panel, 10);
    }
    let dark = if light_theme() { 0i32 } else { 1i32 };
    DwmSetWindowAttribute(
        panel,
        DWMWA_USE_IMMERSIVE_DARK_MODE as u32,
        &dark as *const _ as *const _,
        size_of::<i32>() as u32,
    );
    InvalidateRect(panel, null(), 0);
    SetWindowPos(
        panel,
        HWND_TOPMOST,
        x,
        y,
        width,
        height,
        SWP_NOACTIVATE | SWP_SHOWWINDOW,
    ) != 0
}

pub unsafe fn hide(panel: HWND) {
    if !panel.is_null() {
        ShowWindow(panel, SW_HIDE);
    }
}

unsafe extern "system" fn proc(hwnd: HWND, message: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match message {
        WM_MOUSEACTIVATE => MA_NOACTIVATE as isize,
        WM_ERASEBKGND => 1,
        WM_PAINT => {
            paint(hwnd);
            0
        }
        _ => DefWindowProcW(hwnd, message, wp, lp),
    }
}

#[derive(Clone, Copy)]
struct Palette {
    background: u32,
    foreground: u32,
    muted: u32,
    surface: u32,
    border: u32,
    track: u32,
    accent: u32,
}
fn palette(light: bool, level: Option<Level>) -> Palette {
    let accent = match level {
        Some(l) if l.charging => 0xff69ad86,
        Some(l) if l.percent <= 10 => 0xffcf6868,
        Some(_) => {
            if light {
                0xff397fc3
            } else {
                0xff5ca4e6
            }
        }
        None => 0xff8d8d8d,
    };
    if light {
        Palette {
            background: 0xfffafafa,
            foreground: 0xff1b1b1b,
            muted: 0xff616161,
            surface: 0xfff0f0f0,
            border: 0xffe1e1e1,
            track: 0xffe4e4e4,
            accent,
        }
    } else {
        Palette {
            background: 0xff202020,
            foreground: 0xfff5f5f5,
            muted: 0xffb6b6b6,
            surface: 0xff2a2a2a,
            border: 0xff3e3e3e,
            track: 0xff424242,
            accent,
        }
    }
}

unsafe fn paint(hwnd: HWND) {
    let mut ps: PAINTSTRUCT = zeroed();
    let target = BeginPaint(hwnd, &mut ps);
    let mut client: RECT = zeroed();
    GetClientRect(hwnd, &mut client);
    let memory = CreateCompatibleDC(target);
    let bitmap = CreateCompatibleBitmap(target, client.right, client.bottom);
    let old_bitmap = SelectObject(memory, bitmap as _);
    let mut graphics: *mut GpGraphics = null_mut();
    if GdipCreateFromHDC(memory, &mut graphics) == 0 {
        GdipSetSmoothingMode(graphics, SmoothingModeAntiAlias8x8);
        GdipSetTextRenderingHint(graphics, TextRenderingHintAntiAliasGridFit);
        let data = *DATA.lock().unwrap();
        let colors = palette(light_theme(), data.level);
        GdipGraphicsClear(graphics, colors.border);
        fill_round(
            graphics,
            1.0,
            1.0,
            (client.right - 2) as f32,
            (client.bottom - 2) as f32,
            scale(hwnd, 13) as f32,
            colors.background,
        );
        let tile = (scale(hwnd, 16), scale(hwnd, 18), scale(hwnd, 58));
        fill_round(
            graphics,
            tile.0 as f32,
            tile.1 as f32,
            tile.2 as f32,
            tile.2 as f32,
            scale(hwnd, 13) as f32,
            colors.surface,
        );
        draw_panel_device(graphics, hwnd, data.device, colors.accent);

        let left = scale(hwnd, 88) as f32;
        draw_text(
            graphics,
            hwnd,
            NAMES[data.device],
            left,
            scale(hwnd, 15) as f32,
            15.5,
            FontStyleBold,
            colors.foreground,
        );
        let status = match data.level {
            Some(l) if l.charging => "Conectado · carregando",
            Some(l) if l.percent <= 10 => "Conectado · bateria baixa",
            Some(_) => "Conectado · descarregando",
            None => "Desconectado",
        };
        fill_ellipse(
            graphics,
            left,
            scale(hwnd, 46) as f32,
            scale(hwnd, 7) as f32,
            colors.accent,
        );
        draw_text(
            graphics,
            hwnd,
            status,
            left + scale(hwnd, 13) as f32,
            scale(hwnd, 39) as f32,
            12.5,
            FontStyleRegular,
            colors.muted,
        );
        let percent = data
            .level
            .map_or_else(|| "—".into(), |l| format!("{}%", l.percent));
        draw_text(
            graphics,
            hwnd,
            &percent,
            left,
            scale(hwnd, 66) as f32,
            32.0,
            FontStyleBold,
            colors.foreground,
        );

        let bar_x = left;
        let bar_y = scale(hwnd, 122) as f32;
        let bar_width = scale(hwnd, 242) as f32;
        fill_round(
            graphics,
            bar_x,
            bar_y,
            bar_width,
            scale(hwnd, 7) as f32,
            scale(hwnd, 4) as f32,
            colors.track,
        );
        if let Some(level) = data.level {
            let fill = (bar_width * level.percent as f32 / 100.0).max(scale(hwnd, 7) as f32);
            fill_round(
                graphics,
                bar_x,
                bar_y,
                fill,
                scale(hwnd, 7) as f32,
                scale(hwnd, 4) as f32,
                colors.accent,
            );
        }
        let elapsed = data.updated.map_or_else(
            || "Ainda não atualizado".into(),
            |time| {
                let seconds = time.elapsed().as_secs();
                if seconds < 60 {
                    "Atualizado agora".into()
                } else if seconds < 3600 {
                    format!("Atualizado há {} min", seconds / 60)
                } else {
                    format!("Atualizado há {} h", seconds / 3600)
                }
            },
        );
        draw_text(
            graphics,
            hwnd,
            &elapsed,
            left,
            scale(hwnd, 145) as f32,
            12.0,
            FontStyleRegular,
            colors.muted,
        );
        GdipDeleteGraphics(graphics);
    }
    BitBlt(
        target,
        0,
        0,
        client.right,
        client.bottom,
        memory,
        0,
        0,
        SRCCOPY,
    );
    SelectObject(memory, old_bitmap);
    DeleteObject(bitmap as _);
    DeleteDC(memory);
    EndPaint(hwnd, &ps);
}

unsafe fn draw_panel_device(graphics: *mut GpGraphics, hwnd: HWND, device: usize, color: u32) {
    let mut pen = null_mut();
    GdipCreatePen1(color, scale(hwnd, 2) as f32 + 0.5, UnitPixel, &mut pen);
    if pen.is_null() {
        return;
    }
    GdipSetPenStartCap(pen, LineCapRound);
    GdipSetPenEndCap(pen, LineCapRound);
    GdipSetPenLineJoin(pen, LineJoinRound);
    if device == 0 {
        let x = scale(hwnd, 24) as f32;
        let y = scale(hwnd, 25) as f32;
        let width = scale(hwnd, 42) as f32;
        let height = scale(hwnd, 34) as f32;
        GdipDrawArc(graphics, pen, x, y, width, height, 180.0, 180.0);
        GdipDrawBezier(
            graphics,
            pen,
            scale(hwnd, 30) as f32,
            scale(hwnd, 44) as f32,
            scale(hwnd, 23) as f32,
            scale(hwnd, 44) as f32,
            scale(hwnd, 23) as f32,
            scale(hwnd, 66) as f32,
            scale(hwnd, 31) as f32,
            scale(hwnd, 66) as f32,
        );
        GdipDrawBezier(
            graphics,
            pen,
            scale(hwnd, 60) as f32,
            scale(hwnd, 44) as f32,
            scale(hwnd, 67) as f32,
            scale(hwnd, 44) as f32,
            scale(hwnd, 67) as f32,
            scale(hwnd, 66) as f32,
            scale(hwnd, 59) as f32,
            scale(hwnd, 66) as f32,
        );
    } else {
        let mut path = null_mut();
        GdipCreatePath(FillModeAlternate, &mut path);
        let cx = scale(hwnd, 45) as f32;
        let top = scale(hwnd, 24) as f32;
        let right = scale(hwnd, 59) as f32;
        let bottom = scale(hwnd, 70) as f32;
        let left = scale(hwnd, 31) as f32;
        GdipAddPathBezier(
            path,
            cx,
            top,
            right,
            top,
            right,
            scale(hwnd, 34) as f32,
            right,
            scale(hwnd, 43) as f32,
        );
        GdipAddPathBezier(
            path,
            right,
            scale(hwnd, 43) as f32,
            right,
            scale(hwnd, 61) as f32,
            scale(hwnd, 53) as f32,
            bottom,
            cx,
            bottom,
        );
        GdipAddPathBezier(
            path,
            cx,
            bottom,
            scale(hwnd, 37) as f32,
            bottom,
            left,
            scale(hwnd, 61) as f32,
            left,
            scale(hwnd, 43) as f32,
        );
        GdipAddPathBezier(
            path,
            left,
            scale(hwnd, 43) as f32,
            left,
            scale(hwnd, 34) as f32,
            left,
            top,
            cx,
            top,
        );
        GdipClosePathFigure(path);
        GdipDrawPath(graphics, pen, path);
        GdipDeletePath(path);
        GdipDrawLine(
            graphics,
            pen,
            cx,
            scale(hwnd, 26) as f32,
            cx,
            scale(hwnd, 40) as f32,
        );
    }
    GdipDeletePen(pen);
}

unsafe fn solid(color: u32) -> *mut GpSolidFill {
    let mut brush = null_mut();
    GdipCreateSolidFill(color, &mut brush);
    brush
}
unsafe fn rounded_path(x: f32, y: f32, width: f32, height: f32, radius: f32) -> *mut GpPath {
    let mut path = null_mut();
    GdipCreatePath(FillModeAlternate, &mut path);
    let diameter = radius * 2.0;
    GdipAddPathArc(path, x, y, diameter, diameter, 180.0, 90.0);
    GdipAddPathArc(
        path,
        x + width - diameter,
        y,
        diameter,
        diameter,
        270.0,
        90.0,
    );
    GdipAddPathArc(
        path,
        x + width - diameter,
        y + height - diameter,
        diameter,
        diameter,
        0.0,
        90.0,
    );
    GdipAddPathArc(
        path,
        x,
        y + height - diameter,
        diameter,
        diameter,
        90.0,
        90.0,
    );
    GdipClosePathFigure(path);
    path
}
unsafe fn fill_round(
    graphics: *mut GpGraphics,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    radius: f32,
    color: u32,
) {
    if width <= 0.0 || height <= 0.0 {
        return;
    }
    let path = rounded_path(
        x,
        y,
        width,
        height,
        radius.min(width / 2.0).min(height / 2.0),
    );
    let brush = solid(color);
    GdipFillPath(graphics, brush as *mut GpBrush, path);
    GdipDeleteBrush(brush as *mut GpBrush);
    GdipDeletePath(path);
}
unsafe fn fill_ellipse(graphics: *mut GpGraphics, x: f32, y: f32, size: f32, color: u32) {
    let brush = solid(color);
    GdipFillEllipse(graphics, brush as *mut GpBrush, x, y, size, size);
    GdipDeleteBrush(brush as *mut GpBrush);
}
#[allow(clippy::too_many_arguments)]
unsafe fn draw_text(
    graphics: *mut GpGraphics,
    hwnd: HWND,
    value: &str,
    x: f32,
    y: f32,
    size: f32,
    style: i32,
    color: u32,
) {
    let mut family = null_mut();
    if GdipCreateFontFamilyFromName(
        w("Segoe UI Variable Text").as_ptr(),
        null_mut(),
        &mut family,
    ) != 0
    {
        GdipCreateFontFamilyFromName(w("Segoe UI").as_ptr(), null_mut(), &mut family);
    }
    if family.is_null() {
        return;
    }
    let mut font = null_mut();
    let dpi = GetDpiForWindow(hwnd) as f32 / 96.0;
    GdipCreateFont(family, size * dpi, style, UnitPixel, &mut font);
    if !font.is_null() {
        let brush = solid(color);
        let value = w(value);
        let layout = RectF {
            X: x,
            Y: y,
            Width: scale(hwnd, 250) as f32,
            Height: scale(hwnd, 48) as f32,
        };
        GdipDrawString(
            graphics,
            value.as_ptr(),
            (value.len() - 1) as i32,
            font,
            &layout,
            null(),
            brush as *mut GpBrush,
        );
        GdipDeleteBrush(brush as *mut GpBrush);
        GdipDeleteFont(font);
    }
    GdipDeleteFontFamily(family);
}
