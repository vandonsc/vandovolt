use crate::{config::Style, devices::Level};
use std::{mem::zeroed, ptr::null_mut};
use windows_sys::Win32::{Foundation::*, Graphics::Gdi::*, UI::WindowsAndMessaging::*};

const SUPERSAMPLE: i32 = 4;

fn w(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}
fn color(level: Option<Level>, normal: u32) -> u32 {
    match level {
        Some(l) if l.charging => 0x0089e66d,
        Some(l) if l.percent <= 10 => 0x006060ff,
        Some(_) => normal,
        None => 0x00999999,
    }
}
fn gauge_fill(level: Option<Level>) -> u32 {
    match level {
        Some(l) if l.charging => 0x0086ad69,
        Some(l) if l.percent <= 10 => 0x006868cf,
        Some(_) => 0x00c6c6c6,
        None => 0x007f7f7f,
    }
}

pub unsafe fn create(device: usize, level: Option<Level>, style: Style) -> HICON {
    create_sized(device, level, style, GetSystemMetrics(SM_CXSMICON).max(16))
}

pub unsafe fn create_sized(
    device: usize,
    level: Option<Level>,
    style: Style,
    output: i32,
) -> HICON {
    create_sized_colored(device, level, style, output, 0x00ffffff)
}

pub unsafe fn create_sized_colored(
    device: usize,
    level: Option<Level>,
    style: Style,
    output: i32,
    normal: u32,
) -> HICON {
    let size = output * SUPERSAMPLE;
    let screen = GetDC(null_mut());
    let mut source_bits = null_mut();
    let source = dib(screen, size, &mut source_bits);
    if source.is_null() {
        ReleaseDC(null_mut(), screen);
        return null_mut();
    }
    let dc = CreateCompatibleDC(screen);
    let old_bitmap = SelectObject(dc, source as _);
    SetBkMode(dc, TRANSPARENT as i32);
    let icon_color = color(level, normal);
    SetTextColor(dc, icon_color);
    // Approximately two physical pixels after downsampling: deliberately bold,
    // like Fluent/Lucide tray glyphs, instead of a fragile hairline.
    let pen = CreatePen(PS_SOLID, (size / 8).max(SUPERSAMPLE * 2), icon_color);
    let old_pen = SelectObject(dc, pen as _);
    let old_brush = SelectObject(dc, GetStockObject(NULL_BRUSH));
    match style {
        Style::Percent => draw_percent(dc, size, level, true),
        Style::IconPercent => {
            draw_badged_device(dc, device, size, level);
        }
        Style::Gauge => draw_integrated_gauge(dc, device, size, level, gauge_fill(level)),
    }
    SelectObject(dc, old_brush);
    SelectObject(dc, old_pen);
    SelectObject(dc, old_bitmap);
    DeleteObject(pen as _);
    DeleteDC(dc);

    let source_pixels =
        std::slice::from_raw_parts_mut(source_bits as *mut u32, (size * size) as usize);
    for pixel in source_pixels.iter_mut() {
        if *pixel & 0x00ff_ffff != 0 {
            *pixel |= 0xff00_0000;
        }
    }
    let mut target_bits = null_mut();
    let target = dib(screen, output, &mut target_bits);
    ReleaseDC(null_mut(), screen);
    if target.is_null() {
        DeleteObject(source as _);
        return null_mut();
    }
    let target_pixels =
        std::slice::from_raw_parts_mut(target_bits as *mut u32, (output * output) as usize);
    for y in 0..output {
        for x in 0..output {
            let mut a = 0u32;
            let mut r = 0u32;
            let mut g = 0u32;
            let mut b = 0u32;
            for sy in 0..SUPERSAMPLE {
                for sx in 0..SUPERSAMPLE {
                    let p = source_pixels
                        [((y * SUPERSAMPLE + sy) * size + x * SUPERSAMPLE + sx) as usize];
                    a += p >> 24;
                    r += (p >> 16) & 255;
                    g += (p >> 8) & 255;
                    b += p & 255;
                }
            }
            let samples = (SUPERSAMPLE * SUPERSAMPLE) as u32;
            target_pixels[(y * output + x) as usize] =
                (a / samples) << 24 | (r / samples) << 16 | (g / samples) << 8 | b / samples;
        }
    }
    let mask = CreateBitmap(output, output, 1, 1, null_mut());
    let info = ICONINFO {
        fIcon: 1,
        xHotspot: 0,
        yHotspot: 0,
        hbmMask: mask,
        hbmColor: target,
    };
    let result = CreateIconIndirect(&info);
    DeleteObject(mask as _);
    DeleteObject(target as _);
    DeleteObject(source as _);
    result
}

unsafe fn dib(dc: HDC, size: i32, bits: &mut *mut core::ffi::c_void) -> HBITMAP {
    let mut info: BITMAPINFO = zeroed();
    info.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
    info.bmiHeader.biWidth = size;
    info.bmiHeader.biHeight = -size;
    info.bmiHeader.biPlanes = 1;
    info.bmiHeader.biBitCount = 32;
    info.bmiHeader.biCompression = BI_RGB;
    CreateDIBSection(dc, &info, DIB_RGB_COLORS, bits, null_mut(), 0)
}
unsafe fn text_font(dc: HDC, height: i32, weight: i32) -> (HFONT, HGDIOBJ) {
    let font = CreateFontW(
        -height,
        0,
        0,
        0,
        weight,
        0,
        0,
        0,
        DEFAULT_CHARSET as u32,
        OUT_DEFAULT_PRECIS as u32,
        CLIP_DEFAULT_PRECIS as u32,
        ANTIALIASED_QUALITY as u32,
        DEFAULT_PITCH as u32,
        w("Segoe UI").as_ptr(),
    );
    let old = SelectObject(dc, font as _);
    (font, old)
}
unsafe fn fitted_text(dc: HDC, text: &str, area: RECT, mut height: i32, weight: i32) {
    let value = w(text);
    loop {
        let (font, old) = text_font(dc, height, weight);
        let mut bounds: RECT = zeroed();
        DrawTextW(
            dc,
            value.as_ptr(),
            (value.len() - 1) as i32,
            &mut bounds,
            DT_CALCRECT | DT_SINGLELINE,
        );
        let width = bounds.right - bounds.left;
        let text_height = bounds.bottom - bounds.top;
        if width <= area.right - area.left || height <= SUPERSAMPLE * 4 {
            TextOutW(
                dc,
                area.left + ((area.right - area.left - width).max(0) / 2),
                area.top + ((area.bottom - area.top - text_height).max(0) / 2),
                value.as_ptr(),
                (value.len() - 1) as i32,
            );
            SelectObject(dc, old);
            DeleteObject(font as _);
            break;
        }
        SelectObject(dc, old);
        DeleteObject(font as _);
        height -= SUPERSAMPLE;
    }
}
unsafe fn draw_percent(dc: HDC, size: i32, level: Option<Level>, percent_sign: bool) {
    let text = level.map_or_else(
        || "--".into(),
        |l| {
            if percent_sign {
                format!("{}%", l.percent)
            } else {
                l.percent.to_string()
            }
        },
    );
    fitted_text(
        dc,
        &text,
        RECT {
            left: 0,
            top: 0,
            right: size,
            bottom: size,
        },
        size * 13 / 16,
        700,
    );
}
unsafe fn draw_badged_device(dc: HDC, device: usize, size: i32, level: Option<Level>) {
    if device == 0 {
        draw_headset(dc, size / 16, size / 16, size * 7 / 8, size * 7 / 8);
    } else {
        draw_mouse(dc, size * 9 / 32, size / 32, size * 7 / 16, size * 15 / 16);
    }
    let badge = RECT {
        left: size * 7 / 16,
        top: size * 9 / 16,
        right: size,
        bottom: size,
    };
    let brush = CreateSolidBrush(0x00282828);
    let old_brush = SelectObject(dc, brush as _);
    let old_pen = SelectObject(dc, GetStockObject(NULL_PEN));
    RoundRect(
        dc,
        badge.left,
        badge.top,
        badge.right,
        badge.bottom,
        size / 8,
        size / 8,
    );
    SelectObject(dc, old_pen);
    SelectObject(dc, old_brush);
    DeleteObject(brush as _);
    SetTextColor(dc, 0x00ffffff);
    let text = level.map_or_else(|| "--".into(), |l| l.percent.to_string());
    fitted_text(dc, &text, badge, size * 5 / 16, 700);
}
unsafe fn draw_headset(dc: HDC, x: i32, y: i32, width: i32, height: i32) {
    let mid = y + height / 2;
    Arc(dc, x, y, x + width, y + height, x + width, mid, x, mid);
    let cup = width * 7 / 24;
    let inset = height / 12;
    let top = mid + inset;
    let bottom = y + height - inset;
    let left_cup = [
        POINT { x: x + cup, y: top },
        POINT { x, y: top },
        POINT { x, y: bottom },
        POINT {
            x: x + cup,
            y: bottom,
        },
    ];
    let right_cup = [
        POINT {
            x: x + width - cup,
            y: top,
        },
        POINT {
            x: x + width,
            y: top,
        },
        POINT {
            x: x + width,
            y: bottom,
        },
        POINT {
            x: x + width - cup,
            y: bottom,
        },
    ];
    PolyBezier(dc, left_cup.as_ptr(), left_cup.len() as u32);
    PolyBezier(dc, right_cup.as_ptr(), right_cup.len() as u32);
}
unsafe fn draw_mouse(dc: HDC, x: i32, y: i32, width: i32, height: i32) {
    RoundRect(dc, x, y, x + width, y + height, width, width);
    MoveToEx(dc, x + width / 2, y, null_mut());
    LineTo(dc, x + width / 2, y + height * 2 / 5);
}
unsafe fn draw_device_outline(dc: HDC, device: usize, x: i32, y: i32, width: i32, height: i32) {
    if device == 0 {
        draw_headset(dc, x, y, width, height);
    } else {
        draw_mouse(dc, x, y, width, height);
    }
}
unsafe fn draw_integrated_gauge(
    dc: HDC,
    device: usize,
    size: i32,
    level: Option<Level>,
    fill_color: u32,
) {
    // The silhouette occupies 87.5% x 93.75% of the canvas. Optical margins
    // keep the bold antialiased stroke from being clipped by the shell.
    let x = size / 16;
    let y = size / 32;
    let width = size * 7 / 8;
    let height = size * 15 / 16;
    let percent = level.map_or(0, |l| l.percent as i32);
    let brush = CreateSolidBrush(fill_color);
    if device == 1 {
        let mouse_width = width * 5 / 7;
        let mouse_x = x + (width - mouse_width) / 2;
        let inner = RECT {
            left: mouse_x + width / 10,
            top: y + height / 12,
            right: mouse_x + mouse_width - width / 10,
            bottom: y + height - height / 12,
        };
        let region = CreateRoundRectRgn(
            inner.left,
            inner.top,
            inner.right + 1,
            inner.bottom + 1,
            mouse_width,
            mouse_width,
        );
        SelectClipRgn(dc, region);
        let fill_top = inner.bottom - (inner.bottom - inner.top) * percent / 100;
        FillRect(
            dc,
            &RECT {
                left: inner.left,
                top: fill_top,
                right: inner.right,
                bottom: inner.bottom,
            },
            brush,
        );
        SelectClipRgn(dc, null_mut());
        DeleteObject(region as _);
    } else {
        let gap = (size / 32).max(SUPERSAMPLE);
        let cup_top = y + height / 2 + gap;
        let cup_bottom = y + height - gap;
        let fill_top = cup_bottom - (cup_bottom - cup_top) * percent / 100;
        for (left, right) in [
            (x + gap, x + width * 7 / 24 - gap),
            (x + width * 17 / 24 + gap, x + width - gap),
        ] {
            let region = CreateRoundRectRgn(
                left,
                cup_top,
                right + 1,
                cup_bottom + 1,
                width / 4,
                width / 4,
            );
            SelectClipRgn(dc, region);
            FillRect(
                dc,
                &RECT {
                    left,
                    top: fill_top,
                    right,
                    bottom: cup_bottom,
                },
                brush,
            );
            SelectClipRgn(dc, null_mut());
            DeleteObject(region as _);
        }
    }
    DeleteObject(brush as _);
    let outline_color = if level.is_some() {
        0x00f2f2f2
    } else {
        0x00999999
    };
    let outline_pen = CreatePen(PS_SOLID, (size / 8).max(SUPERSAMPLE * 2), outline_color);
    let previous_pen = SelectObject(dc, outline_pen as _);
    if device == 0 {
        draw_device_outline(dc, device, x, y, width, height);
    } else {
        let mouse_width = width * 5 / 7;
        draw_device_outline(
            dc,
            device,
            x + (width - mouse_width) / 2,
            y,
            mouse_width,
            height,
        );
    }
    SelectObject(dc, previous_pen);
    DeleteObject(outline_pen as _);
}
