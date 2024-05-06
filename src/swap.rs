#[derive(Default, PartialEq, Eq)]
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
}

pub fn calculate_swap_coords(
    from_monitor: MonitorInfo,
    to_monitor: MonitorInfo,
    window: Rect,
) -> Rect {
    if are_same_size(&from_monitor, &to_monitor) {
        return window.translate(
            to_monitor.rect.left - from_monitor.rect.left,
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

fn are_same_size(monitor_a: &MonitorInfo, monitor_b: &MonitorInfo) -> bool {
    (monitor_a.rect.right - monitor_a.rect.left) == (monitor_b.rect.right - monitor_b.rect.left)
        && (monitor_a.rect.bottom - monitor_a.rect.top)
            == (monitor_b.rect.bottom - monitor_b.rect.top)
}
