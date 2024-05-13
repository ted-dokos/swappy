#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rect {
    pub left: i32,
    pub right: i32,
    pub top: i32,
    pub bottom: i32,
}
impl Rect {
    pub fn translate(self: &Self, x: i32, y: i32) -> Rect {
        Rect {
            left: self.left + x,
            right: self.right + x,
            top: self.top + y,
            bottom: self.bottom + y,
        }
    }
}

#[derive(Default, PartialEq, Eq)]
pub struct MonitorInfo {
    pub rect: Rect,
    pub sub_rect: Option<Rect>,
}
impl MonitorInfo {
    pub fn New(left: i32, right: i32, top: i32, bottom: i32) -> Self {
        return MonitorInfo {
            rect: Rect {
                left,
                right,
                top,
                bottom,
            },
            sub_rect: None,
        };
    }
}

pub fn calculate_swap_coords(
    from_monitor: MonitorInfo,
    to_monitor: MonitorInfo,
    window: Rect,
) -> Rect {
    let from_rect = get_monitor_region(&from_monitor);
    let to_rect = get_monitor_region(&to_monitor);
    if are_same_size(&from_rect, &to_rect) {
        return window.translate(
            to_rect.left - from_monitor.rect.left,
            to_monitor.rect.top - from_monitor.rect.top,
        );
    }
    
    let from_width = from_monitor.rect.right - from_monitor.rect.left;
    let left_pct = (window.left - from_monitor.rect.left) as f64 / from_width as f64;
    let right_pct = (window.right - from_monitor.rect.left) as f64 / from_width as f64;
    let from_height = from_monitor.rect.bottom - from_monitor.rect.top;
    let top_pct = (window.top - from_monitor.rect.top) as f64 / from_height as f64;
    let bottom_pct = (window.bottom - from_monitor.rect.top) as f64 / from_height as f64;

    let to_width = to_monitor.rect.right - to_monitor.rect.left;
    let to_height = to_monitor.rect.bottom - to_monitor.rect.top;
    Rect {
        left: to_monitor.rect.left + (left_pct * to_width as f64) as i32,
        right: to_monitor.rect.left + (right_pct * to_width as f64) as i32,
        top: to_monitor.rect.top + (top_pct * to_height as f64) as i32,
        bottom: to_monitor.rect.top + (bottom_pct * to_height as f64) as i32,
    }
}

fn get_monitor_region(m: &MonitorInfo) -> Rect {
    if m.sub_rect.is_none() {
        return m.rect;
    }
    let sub_rect = &m.sub_rect.unwrap();
    return Rect {
        left: m.rect.left + sub_rect.left,
        right: m.rect.left + sub_rect.right,
        top: m.rect.top + sub_rect.top,
        bottom: m.rect.top + sub_rect.bottom,
    };
}

fn are_same_size(rect_1: &Rect, rect_2: &Rect) -> bool {
    return (rect_1.right - rect_1.left) == (rect_2.right - rect_2.left)
        && (rect_1.bottom - rect_1.top) == (rect_2.bottom - rect_2.top);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hd_monitor(x: i32, y: i32) -> MonitorInfo {
        return MonitorInfo::New(x, x + 1920, y, y + 1080);
    }
    fn qhd_monitor(x: i32, y: i32) -> MonitorInfo {
        return MonitorInfo::New(x, x + 2560, y, y + 1440);
    }
    fn fourk_monitor(x: i32, y: i32) -> MonitorInfo {
        return MonitorInfo::New(x, x + 1920 * 2, y, y + 2160);
    }

    #[test]
    fn test_calculate_swap_coords_qhd_pair() {
        let from = qhd_monitor(0, 0);
        let to = qhd_monitor(2560, 0);
        let window = Rect {
            left: 0,
            right: 1280,
            top: 0,
            bottom: 1440,
        };

        assert_eq!(
            calculate_swap_coords(from, to, window),
            Rect {
                left: 2560,
                right: 3840,
                top: 0,
                bottom: 1440
            }
        )
    }
    #[test]
    fn test_calculate_swap_coords_hd_to_4k() {
        let from = hd_monitor(0, 0);
        let to = fourk_monitor(1920, 0);
        let window = Rect {
            left: 0,
            right: 960,
            top: 0,
            bottom: 1080,
        };
        assert_eq!(
            calculate_swap_coords(from, to, window),
            Rect {
                left: 1920,
                right: 1920 * 2,
                top: 0,
                bottom: 2160,
            }
        )
    }
}
