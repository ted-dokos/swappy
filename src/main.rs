use clap::Parser;

mod swap;
#[cfg(target_os = "windows")]
mod win_main;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = false)]
    info: bool,
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
