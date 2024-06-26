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

pub fn calculate_swap_coords(
    region_a: Rect,
    region_b: Rect,
    window: Rect,
    overlap_threshold: f32,
) -> Rect {
    let overlaps_a = window_overlap(window, region_a) >= overlap_threshold;
    let overlaps_b = window_overlap(window, region_b) >= overlap_threshold;
    if !overlaps_a && !overlaps_b {
        return window;
    }
    let from_rect = if overlaps_a { region_a } else { region_b };
    let to_rect = if overlaps_a { region_b } else { region_a };
    let window_clamped = clamp_to_region(window, from_rect).unwrap_or(window);
    if are_same_size(&from_rect, &to_rect) {
        return window_clamped
            .translate(to_rect.left - from_rect.left, to_rect.top - from_rect.top);
    }
    let from_width = from_rect.right - from_rect.left;
    let from_height = from_rect.bottom - from_rect.top;
    let to_width = to_rect.right - to_rect.left;
    let to_height = to_rect.bottom - to_rect.top;
    let left = if window_clamped.left == from_rect.left {
        to_rect.left
    } else {
        let left_pct = (window_clamped.left - from_rect.left) as f32 / from_width as f32;
        to_rect.left + (left_pct * to_width as f32) as i32
    };
    let right = if window_clamped.right == from_rect.right {
        to_rect.right
    } else {
        let right_pct = (window_clamped.right - from_rect.left) as f32 / from_width as f32;
        to_rect.left + (right_pct * to_width as f32) as i32
    };
    let top = if window_clamped.top == from_rect.top {
        to_rect.top
    } else {
        let top_pct = (window_clamped.top - from_rect.top) as f32 / from_height as f32;
        to_rect.top + (top_pct * to_height as f32) as i32
    };
    let bottom = if window_clamped.bottom == from_rect.bottom {
        to_rect.bottom
    } else {
        let bottom_pct = (window_clamped.bottom - from_rect.top) as f32 / from_height as f32;
        to_rect.top + (bottom_pct * to_height as f32) as i32
    };
    Rect {
        left,
        right,
        top,
        bottom,
    }
}

/// Calculate what percentage of window overlaps with region, by area.
fn window_overlap(window: Rect, region: Rect) -> f32 {
    let width = window.right - window.left;
    let height = window.bottom - window.top;
    let maybe_clamped = clamp_to_region(window, region);
    match maybe_clamped {
        Ok(clamped) => {
            let clamped_width = clamped.right - clamped.left;
            let clamped_height = clamped.bottom - clamped.top;
            return (clamped_width as f32 / width as f32) * (clamped_height as f32 / height as f32);
        }
        Err(_) => {
            return 0.0;
        }
    }
}

/// Clamp a rectangle to live entirely within a region, if possible.
fn clamp_to_region(rect: Rect, region: Rect) -> Result<Rect, NoOverlapError> {
    if rect.right <= region.left
        || rect.left >= region.right
        || rect.bottom <= rect.top
        || rect.top >= region.bottom
    {
        return Err(NoOverlapError {});
    }
    let left = if rect.left <= region.left {
        region.left
    } else {
        rect.left
    };
    let right = if rect.right >= region.right {
        region.right
    } else {
        rect.right
    };
    let top = if rect.top <= region.top {
        region.top
    } else {
        rect.top
    };
    let bottom = if rect.bottom >= region.bottom {
        region.bottom
    } else {
        rect.bottom
    };
    return Ok(Rect {
        left,
        right,
        top,
        bottom,
    });
}

struct NoOverlapError {}

fn are_same_size(rect_1: &Rect, rect_2: &Rect) -> bool {
    return (rect_1.right - rect_1.left) == (rect_2.right - rect_2.left)
        && (rect_1.bottom - rect_1.top) == (rect_2.bottom - rect_2.top);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hd_monitor(x: i32, y: i32) -> Rect {
        return Rect {
            left: x,
            right: x + 1920,
            top: y,
            bottom: y + 1080,
        };
    }
    fn qhd_monitor(x: i32, y: i32) -> Rect {
        return Rect {
            left: x,
            right: x + 2560,
            top: y,
            bottom: y + 1440,
        };
    }
    fn fourk_monitor(x: i32, y: i32) -> Rect {
        return Rect {
            left: x,
            right: x + 3840,
            top: y,
            bottom: y + 2160,
        };
    }

    #[test]
    fn test_qhd_left_third_to_right_two_thirds() {
        let window = Rect {
            left: 0,
            right: 853,
            top: 720,
            bottom: 1440,
        };
        let a = Rect {
            left: 0,
            right: 850,
            top: 0,
            bottom: 1440,
        };
        let b = Rect {
            left: 850,
            right: 2560,
            top: 0,
            bottom: 1440,
        };
        assert_eq!(
            calculate_swap_coords(a, b, window, 0.8),
            Rect {
                left: 850,
                right: 2560,
                top: 720,
                bottom: 1440
            }
        )
    }
    #[test]
    fn test_calculate_swap_coords_qhd_pair() {
        let a = qhd_monitor(0, 0);
        let b = qhd_monitor(2560, 0);
        let window = Rect {
            left: 0,
            right: 1280,
            top: 0,
            bottom: 1440,
        };

        assert_eq!(
            calculate_swap_coords(a, b, window, 0.8),
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
        let a = hd_monitor(0, 0);
        let b = fourk_monitor(1920, 0);
        let window = Rect {
            left: 0,
            right: 960,
            top: 0,
            bottom: 1080,
        };
        assert_eq!(
            calculate_swap_coords(a, b, window, 0.8),
            Rect {
                left: 1920,
                right: 1920 * 2,
                top: 0,
                bottom: 2160,
            }
        )
    }
}
