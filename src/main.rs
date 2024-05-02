mod swap;

use std::{
    iter::FilterMap,
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
        EnumWindows, GetWindowInfo, GetWindowTextLengthW,
        GetWindowTextW, IsWindowVisible, MoveWindow, WINDOWINFO, WS_EX_TOOLWINDOW,
    },
};

use crate::swap::calculate_swap_coords;

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
    let text_len_w = GetWindowTextLengthW(window);
    let mut text_buffer_w = vec![u16::default(); (text_len_w + 1) as usize];
    GetWindowTextW(window, &mut text_buffer_w);
    let name = String::from_utf16(&text_buffer_w).unwrap_or("UTF16 ERROR".to_owned());
    add_info(
        window_infos,
        WindowInfo {
            handle: window,
            rect: window_info.rcWindow,
            frame,
            monitor,
            name,
        },
    );
    return TRUE;
}

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
    monitor_a: usize,
    #[arg(default_value_t = 1)]
    monitor_b: usize,
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
    name: String,
}

fn main() {
    let args = Args::parse();
    let monitor_infos = get_monitor_infos();
    let window_infos = get_window_infos();
    if args.info {
        println!("Monitor information:");
        println!("monitor_infos = {:#?}", monitor_infos);
        println!("-----------------------------------------------");
        println!("Window Information:");
        println!("window_infos = {:#?}", window_infos);
        return;
    }
    window_infos.iter().for_each(|window_info| {
        if window_info.monitor == monitor_infos[args.monitor_a].handle {
            let new_rect: swap::Rect = calculate_swap_coords(
                swap::MonitorInfo {
                    rect: win32_rect_to_internal_rect(monitor_infos[args.monitor_a].rect),
                },
                swap::MonitorInfo {
                    rect: win32_rect_to_internal_rect(monitor_infos[args.monitor_b].rect),
                },
                win32_rect_to_internal_rect(window_info.frame),
            );
            unsafe { let _ = MoveWindow(
                window_info.handle,
                new_rect.left + (window_info.rect.left - window_info.frame.left),
                new_rect.top + (window_info.rect.top - window_info.frame.top),
                new_rect.right - new_rect.left,
                new_rect.bottom - new_rect.top,
                true,
            ); };
        }
        if window_info.monitor == monitor_infos[args.monitor_b].handle {
            let new_rect: swap::Rect = calculate_swap_coords(
                swap::MonitorInfo {
                    rect: win32_rect_to_internal_rect(monitor_infos[args.monitor_b].rect),
                },
                swap::MonitorInfo {
                    rect: win32_rect_to_internal_rect(monitor_infos[args.monitor_a].rect),
                },
                win32_rect_to_internal_rect(window_info.frame),
            );
            unsafe { let _ = MoveWindow(
                window_info.handle,
                new_rect.left + (window_info.rect.left - window_info.frame.left),
                new_rect.top + (window_info.rect.top - window_info.frame.top),
                new_rect.right - new_rect.left,
                new_rect.bottom - new_rect.top,
                true,
            ); };
        }
    });
    let filtered_windows: Vec<&WindowInfo> = window_infos
        .iter()
        .filter(|window_info| -> bool {
            window_info.monitor == monitor_infos[args.monitor_a].handle
                || window_info.monitor == monitor_infos[args.monitor_b].handle
        })
        .collect();
    println!("filtered = {:#?}", filtered_windows);
}

fn win32_rect_to_internal_rect(r: RECT) -> swap::Rect {
    swap::Rect {
        left: r.left,
        right: r.right,
        top: r.top,
        bottom: r.bottom,
    }
}
