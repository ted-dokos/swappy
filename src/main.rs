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

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Print info about the active monitors and windows (no swaps occur).
    ///
    /// Can be used to get screen measurements, and figure out which monitor
    /// corresponds to which index.
    #[arg(short, long)]
    info: bool,

    #[arg(value_parser = region_parser, default_value_t = Region::Monitor(0))]
    monitor_a: Region,

    #[arg(value_parser = region_parser, default_value_t = Region::Monitor(1))]
    monitor_b: Region,

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
    if s.trim().starts_with('{') {
        let nums: Vec<Result<i32, ParseIntError>> =
            s.split(',').map(|x| x.parse::<i32>()).collect();

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
