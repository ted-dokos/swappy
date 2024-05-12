use clap::Parser;

mod swap;
#[cfg(target_os = "windows")]
mod win_main;

fn sub_rect_parser(s: &str) -> Result<swap::Rect, String> {
    let vec: Vec<u16> = s
        .split('-')
        .map(|x| x.parse::<u16>().unwrap())
        .collect();
    if vec.len() == 4 {
        return Ok(swap::Rect {
            left: vec[0] as i32,
            right: vec[1] as i32,
            top: vec[2] as i32,
            bottom: vec[3] as i32,
        });
    }
    return Err(concat!(
        "Subrectangle input should be a dash-separated list of four numbers: ",
        "left-right-top-bottom"
    )
    .to_owned());
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

    /// Swap a sub-region of monitor A.
    ///
    /// This should be a dash-separated list of four numbers left-right-top-bottom.
    /// This uses relative coordinates: the top left corner of the monitor should
    /// always be expressed as left=0, top=0, regardless of where the monitor is 
    /// located in virtual coordinates.
    #[arg(long = "subA", value_parser = sub_rect_parser)]
    subrectangle_a: Option<swap::Rect>,

    /// Swap a sub-region of monitor B.
    /// 
    /// This should be a dash-separated list of four numbers left-right-top-bottom.
    /// This uses relative coordinates: the top left corner of the monitor should
    /// always be expressed as left=0, top=0, regardless of where the monitor is 
    /// located in virtual coordinates.
    #[arg(long = "subB", value_parser = sub_rect_parser)]
    subrectangle_b: Option<swap::Rect>,

    #[arg(default_value_t = 0)]
    monitor_a: usize,

    #[arg(default_value_t = 1)]
    monitor_b: usize,
}
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
