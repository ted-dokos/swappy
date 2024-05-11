use clap::Parser;

mod swap;
#[cfg(target_os = "windows")]
mod win_main;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Print info about the active monitors and windows (no swaps occur).
    /// 
    /// Blah blah blah here's more info.
    #[arg(short, long, default_value_t = false)]
    info: bool,

    /// Swap a sub-region of monitor A.
    #[arg(long = "subrectangle_A")]
    subrectangle_a: Option<String>,

    /// Swap a sub-region of monitor B.
    #[arg(long = "subrectangle_B")]
    subrectangle_b: Option<String>,

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
