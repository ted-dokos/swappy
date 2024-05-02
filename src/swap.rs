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
    if !are_same_size(&from_monitor, &to_monitor) {
        return window;
    }
    return window.translate(
        to_monitor.rect.left - from_monitor.rect.left,
        to_monitor.rect.top - from_monitor.rect.top,
    );
}

fn are_same_size(monitor_a: &MonitorInfo, monitor_b: &MonitorInfo) -> bool {
    (monitor_a.rect.right - monitor_a.rect.left) == (monitor_b.rect.right - monitor_b.rect.left)
        && (monitor_a.rect.bottom - monitor_a.rect.top)
            == (monitor_b.rect.bottom - monitor_b.rect.top)
}
