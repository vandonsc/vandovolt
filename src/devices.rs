use hidapi::{HidApi, HidDevice};
use std::time::{Duration, Instant};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Level {
    pub percent: u8,
    pub charging: bool,
}
fn drain(d: &HidDevice) {
    let mut b = [0; 256];
    for _ in 0..8 {
        if !matches!(d.read_timeout(&mut b,0),Ok(n) if n>0) {
            break;
        }
    }
}
fn receive<T>(d: &HidDevice, parse: impl Fn(&[u8]) -> Option<T>) -> Option<T> {
    let end = Instant::now() + Duration::from_millis(1500);
    let mut b = [0; 256];
    while Instant::now() < end {
        let n = d.read_timeout(&mut b, 100).ok()?;
        if let Some(v) = parse(&b[..n]) {
            return Some(v);
        }
    }
    None
}
fn parse_mouse(b: &[u8]) -> Option<Level> {
    if b.len() < 27 || b[..3] != [0x4e, 2, 0x97] {
        return None;
    }
    let p = u16::from_le_bytes([b[17], b[18]]);
    if p > 100 {
        return None;
    }
    Some(Level {
        percent: p as u8,
        charging: b[22] & 1 != 0,
    })
}
fn mouse(d: &HidDevice) -> Option<Level> {
    drain(d);
    let mut q = [0; 64];
    q[..5].copy_from_slice(&[0x4e, 2, 0x81, 0, 0xb0]);
    d.write(&q).ok()?;
    receive(d, parse_mouse)
}
fn hyper_query<T>(d: &HidDevice, cmd: u8, parse: impl Fn(&[u8]) -> Option<T>) -> Option<T> {
    drain(d);
    let mut q = [0; 62];
    q[0] = 0x66;
    q[1] = cmd;
    if d.write(&q).is_err() {
        d.send_feature_report(&q).ok()?;
    }
    receive(d, parse)
}
fn parse_headset(b: &[u8]) -> Option<u8> {
    if b.len() < 5
        || b[0] != 0x66
        || !matches!(b[1], 0x89 | 0x0d)
        || (b[2] == 0 && b[3] == 0)
        || b[4] > 100
    {
        None
    } else {
        Some(b[4])
    }
}
fn headset(d: &HidDevice) -> Option<Level> {
    let percent = hyper_query(d, 0x89, parse_headset)?;
    let charging = hyper_query(d, 0x8a, |b| {
        if b.len() >= 3 && b[0] == 0x66 && matches!(b[1], 0x8a | 0x0c) && b[2] <= 2 {
            Some(b[2] == 1)
        } else {
            None
        }
    })
    .unwrap_or(false);
    Some(Level { percent, charging })
}
pub fn read() -> [Option<Level>; 2] {
    let mut result: [Option<Level>; 2] = [None, None];
    let Ok(api) = HidApi::new() else {
        return result;
    };
    for info in api.device_list() {
        let slot = match (
            info.vendor_id(),
            info.product_id(),
            info.usage_page(),
            info.usage(),
        ) {
            (0x03f0, 0x05b7 | 0x0c9d, 0xff13, 1) => 0,
            (0x1e71, 0x2101 | 0x2129, 0xffca, 1) => 1,
            _ => continue,
        };
        let Ok(d) = info.open_device(&api) else {
            continue;
        };
        let value = if slot == 0 { headset(&d) } else { mouse(&d) };
        if let Some(v) = value {
            if result[slot].is_none() || v.charging {
                result[slot] = Some(v);
            }
        }
    }
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mouse_frames() {
        let mut b = [0; 64];
        b[..3].copy_from_slice(&[0x4e, 2, 0x97]);
        b[17] = 77;
        b[22] = 1;
        assert_eq!(
            parse_mouse(&b),
            Some(Level {
                percent: 77,
                charging: true
            })
        );
        b[18] = 1;
        assert_eq!(parse_mouse(&b), None);
    }
    #[test]
    fn headset_frames() {
        assert_eq!(parse_headset(&[0x66, 0x89, 0x0e, 0xd7, 48]), Some(48));
        assert_eq!(parse_headset(&[0x66, 0x89, 0, 0, 48]), None);
        assert_eq!(parse_headset(&[0x66, 0x89, 1, 0, 101]), None);
    }
}
