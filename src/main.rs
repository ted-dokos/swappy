use std::num::ParseIntError;

use clap::Parser;

mod swap;
#[cfg(target_os = "windows")]
mod win_main;

fn main() {
    let args = Args::parse();
    #[cfg(target_os = "windows")]
    {
        win_main::win_main(args);
    }
    #[cfg(target_os = "linux")]
    {
        println!("Linux not supported yet");
    }
    #[cfg(target_os = "macos")]
    {
        println!("Mac not supported yet");
    }
}

/// Swap windows between monitors or regions of your workspace.
///
/// Swappy takes in two regions as input.
/// They can be monitors, or arbitrary regions of your workspace.
/// It then swaps windows that are "inside" these regions.
///
/// A window is inside a region if a large portion of its area
/// is inside the region. The threshold for this is customizeable.
///
/// Swappy performs clamping as part of the swaps:
/// a window being swapped will first be fit
/// entirely inside the region it is leaving.
///
/// Swappy also performs boundary snapping:
/// windows with an edge on the boundary of a region
/// are guaranteed to remain on region boundaries after a swap.
///
/// # Examples
///
/// =======================================
///
/// Get information about your monitors and active windows:
///
/// > swappy.exe --info
///
/// =======================================
///
/// Swap the contents of the first two monitors:
///
/// > swappy.exe
///
/// =======================================
///
/// Swap the contents of two QHD monitors,
/// the second of which has its bottom 40 pixels taken up
/// by a taskbar:
///
/// > swappy.exe "0,0,2560,1440" "2560,0,5120,1400"
///
/// =======================================
///
/// Swap the left and middle thirds of a WQHD monitor:
///
/// > swappy.exe "0,0,1146,1440" "1146,0,2293,1440"
///
/// =======================================
#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    /// Print info about the active monitors and windows (no swaps occur).
    ///
    /// Can be used to get screen measurements, and figure out which monitor
    /// corresponds to which index.
    #[arg(short, long)]
    info: bool,

    /// First region to switch contents.
    ///
    /// This field can be a single positive integer,
    /// in which case it represents the index of one of your monitors.
    /// The region is the entirety of the monitor.
    ///
    /// It can also be a string of four integers, "left, top, right, bottom".
    /// In this case, the region is the rectangle spanned
    /// by (left, top) to (right, bottom).
    #[arg(value_parser = region_parser, default_value_t = Region::Monitor(0))]
    region_a: Region,

    /// Second region to switch contents.
    ///
    /// This field can be a single positive integer,
    /// in which case it represents the index of one of your monitors.
    /// The region is the entirety of the monitor.
    ///
    /// It can also be a string of four integers, "left, top, right, bottom".
    /// In this case, the region is the rectangle spanned
    /// by (left, top) to (right, bottom).
    #[arg(value_parser = region_parser, default_value_t = Region::Monitor(1))]
    region_b: Region,

    /// Threshold used to decide if a window is inside a region.
    #[arg(short, long, default_value_t = 0.80)]
    overlap_threshold: f32,
}

#[derive(Clone, Debug)]
enum Region {
    Monitor(u32),
    Rect(swap::Rect),
}
impl std::fmt::Display for Region {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Region::Monitor(idx) => {
                return write!(f, "{}", idx);
            }
            Region::Rect(rect) => {
                return write!(
                    f,
                    "{{{}, {}, {}, {}}}",
                    rect.left, rect.top, rect.right, rect.bottom
                );
            }
        }
    }
}

fn region_parser(s: &str) -> Result<Region, String> {
    if s.contains(',') {
        let nums: Vec<Result<i32, ParseIntError>> =
            s.split(',').map(|x| x.trim().parse::<i32>()).collect();
        let err_str = format!(
            concat!(
                "Could not parse rectangular region input {}. ",
                "Should be a string of the form \"left, top, right, bottom\"."
            ),
            s
        );
        if nums.iter().any(|res| res.is_err()) {
            return Err(err_str.to_owned());
        }
        if nums.len() != 4 {
            return Err(err_str.to_owned());
        }
        return Ok(Region::Rect(swap::Rect {
            left: nums[0].clone().unwrap(),
            right: nums[2].clone().unwrap(),
            top: nums[1].clone().unwrap(),
            bottom: nums[3].clone().unwrap(),
        }));
    }
    let monitor_index = s.parse::<u32>();
    if monitor_index.is_err() {
        return Err(format!("Could not parse region input {}.", s).to_owned());
    }
    return Ok(Region::Monitor(monitor_index.unwrap()));
}
