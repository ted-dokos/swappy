use std::{
    mem::{size_of, transmute},
    os::raw::c_void,
    str::from_utf8,
};

use clap::Parser;
use windows::Win32::{
    Foundation::{BOOL, HWND, LPARAM, RECT, TRUE},
    Graphics::{
        Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED, DWMWA_EXTENDED_FRAME_BOUNDS},
        Gdi::{EnumDisplayMonitors, MonitorFromWindow, HDC, HMONITOR, MONITOR_DEFAULTTONULL},
    },
    UI::WindowsAndMessaging::{
        EnumWindows, GetWindowInfo, GetWindowTextA, GetWindowTextLengthA, GetWindowTextLengthW,
        GetWindowTextW, IsWindowVisible, WINDOWINFO, WS_EX_APPWINDOW, WS_EX_TOOLWINDOW,
    },
};

unsafe extern "system" fn enum_dsp_get_monitor_infos(
    monitor: HMONITOR,
    _device_ctx: HDC,
    rect: *mut RECT,
    app_data: LPARAM,
) -> BOOL {
    let v: &mut Vec<MonitorInfo> = transmute(app_data.0);
    add_info(
        v,
        MonitorInfo {
            handle: monitor,
            rect: *rect,
        },
    );
    return TRUE;
}

unsafe extern "system" fn enum_wnd_get_window_infos(window: HWND, app_data: LPARAM) -> BOOL {
    let window_info = get_window_info(window);
    if !is_relevant_window(window, window_info) {
        return TRUE;
    }
    let mut frame = RECT::default();
    let _ = DwmGetWindowAttribute(
        window,
        DWMWA_EXTENDED_FRAME_BOUNDS,
        &mut frame as *mut _ as *mut c_void,
        size_of::<RECT>() as u32,
    );
    let monitor = MonitorFromWindow(window, MONITOR_DEFAULTTONULL);
    let window_infos: &mut Vec<WindowInfo> = transmute(app_data.0);
    add_info(window_infos, WindowInfo{
        handle: window,
        rect: window_info.rcWindow,
        frame,
        monitor,
    });
    return TRUE;
}

unsafe extern "system" fn enum_wnd_display_window_infos(window: HWND, _lp: LPARAM) -> BOOL {
    let window_info = get_window_info(window);
    if !is_relevant_window(window, window_info) {
        return TRUE;
    }
    let rc_window = window_info.rcWindow;
    println!(
        "Window coords: ({}, {}) to ({}, {})",
        rc_window.left, rc_window.top, rc_window.right, rc_window.bottom
    );
    let text_len = GetWindowTextLengthA(window);
    let mut text_buffer = vec![u8::default(); (text_len + 1) as usize];
    let text_len_w = GetWindowTextLengthW(window);
    let mut text_buffer_w = vec![u16::default(); (text_len_w + 1) as usize];
    GetWindowTextA(window, &mut text_buffer);
    GetWindowTextW(window, &mut text_buffer_w);

    // Some other options to consider: SetWindowPlacement and SetWindowPos.
    // let mut new_left = rc_window.left - 2560;
    // if rc_window.right < 100 {
    //     new_left = rc_window.left + 2560;
    // }
    // let _ = MoveWindow(
    //     window,
    //     new_left,
    //     rc_window.top,
    //     rc_window.right - rc_window.left,
    //     rc_window.bottom - rc_window.top,
    //     true,
    // );
    let mut frame_bounds_rect = RECT::default();
    let _ = DwmGetWindowAttribute(
        window,
        DWMWA_EXTENDED_FRAME_BOUNDS,
        &mut frame_bounds_rect as *mut _ as *mut c_void,
        size_of::<RECT>() as u32,
    );
    println!(
        "Frame bound coords: ({}, {}) to ({}, {})",
        frame_bounds_rect.left,
        frame_bounds_rect.top,
        frame_bounds_rect.right,
        frame_bounds_rect.bottom
    );

    println!("Name = {}", from_utf8(&text_buffer).unwrap_or("utf err"));
    println!(
        "Wide Name = {}",
        String::from_utf16(&text_buffer_w).unwrap_or(String::default())
    );
    println!(
        "WS_EX_APPWINDOW = {}",
        (window_info.dwExStyle.0 & WS_EX_APPWINDOW.0)
    );
    println!("-----------------------------------------------");
    return TRUE;
}

fn get_window_info(window: HWND) -> WINDOWINFO {
    let mut window_info = WINDOWINFO {
        cbSize: size_of::<WINDOWINFO>() as u32,
        ..Default::default()
    };
    let _ = unsafe { GetWindowInfo(window, &mut window_info as *mut WINDOWINFO) };
    window_info
}

// Various checks to exclude certain types of windows.
// This roughly corresponds to windows the user cares about moving, in my experience.
// However, this is different from the alt-tab approach
// mentioned by Raymond Chen in his blog post:
// https://devblogs.microsoft.com/oldnewthing/20071008-00/?p=24863
fn is_relevant_window(window: HWND, window_info: WINDOWINFO) -> bool {
    if !unsafe { IsWindowVisible(window).as_bool() } {
        return false;
    }
    let is_tool_window = (window_info.dwExStyle.0 & WS_EX_TOOLWINDOW.0) != 0;
    if is_tool_window {
        return false;
    }
    let mut cloaked = 0;
    let _ = unsafe {
        DwmGetWindowAttribute(
            window,
            DWMWA_CLOAKED,
            &mut cloaked as *mut _ as *mut c_void,
            4,
        )
    };
    return cloaked == 0;
}

fn get_monitor_infos() -> Vec<MonitorInfo> {
    let mut monitor_infos = Vec::<MonitorInfo>::new();
    unsafe {
        let _ = EnumDisplayMonitors(
            HDC::default(),
            None,
            Some(enum_dsp_get_monitor_infos),
            LPARAM {
                0: &mut monitor_infos as *mut _ as isize,
            },
        );
    };
    monitor_infos
}
fn get_window_infos() -> Vec<WindowInfo> {
    let mut window_infos = Vec::<WindowInfo>::new();
    unsafe {
        let _ = EnumWindows(
            Some(enum_wnd_get_window_infos),
            LPARAM {
                0: &mut window_infos as *mut _ as isize,
            },
        );
    }
    window_infos
}

fn add_info<T>(v: &mut Vec<T>, info: T) {
    v.push(info);
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = false)]
    info: bool,
    #[arg(default_value_t = 0)]
    monitor_a: u32,
    #[arg(default_value_t = 1)]
    monitor_b: u32,
}

#[derive(Debug)]
struct MonitorInfo {
    rect: RECT,
    handle: HMONITOR,
}

#[derive(Debug)]
struct WindowInfo {
    handle: HWND,
    rect: RECT,
    // Rectangle data from DWMWA_EXTENDED_FRAME_BOUNDS.
    // This is useful for programs that use invisible borders.
    // For example, a maximized vscode window's rect will have
    // (-8, -8) as its upper left corner. The frame data gives
    // (0, 0) instead.
    frame: RECT,
    monitor: HMONITOR,
}

fn main() {
    let args = Args::parse();
    let monitor_infos = get_monitor_infos();
    let window_infos = get_window_infos();

    if args.info {
        println!("monitor_infos = {:#?}", monitor_infos);
        println!("window_infos = {:#?}", window_infos);
        println!("-----------------------------------------------");
        println!("Window Information:");
        println!("-----------------------------------------------");
        unsafe {
            let _ = EnumWindows(Some(enum_wnd_display_window_infos), LPARAM::default());
        };
        return;
    }

    
}
