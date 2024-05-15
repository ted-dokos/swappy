use crate::{
    swap::{self, calculate_swap_coords},
    Args, Region,
};
use std::{
    mem::{size_of, transmute},
    os::raw::c_void,
};
use windows::Win32::{
    Foundation::{BOOL, HWND, LPARAM, RECT, TRUE},
    Graphics::{
        Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED, DWMWA_EXTENDED_FRAME_BOUNDS},
        Gdi::{EnumDisplayMonitors, MonitorFromWindow, HDC, HMONITOR, MONITOR_DEFAULTTONULL},
    },
    UI::WindowsAndMessaging::{
        EnumWindows, GetWindowInfo, GetWindowTextLengthW, GetWindowTextW, IsWindowVisible,
        MoveWindow, WINDOWINFO, WS_EX_TOOLWINDOW,
    },
};

pub fn win_main(args: Args) {
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
    let region_a_rect = get_rect_from_region(&args.monitor_a, &monitor_infos);
    let region_b_rect = get_rect_from_region(&args.monitor_b, &monitor_infos);

    /// Idea A: let all of the calculations happen in calculate_swap_coords.
    /// where does this succeed or fail?
    /// It succeeds in giving me a single easy call,
    /// assuming I can marshall the data into the right shape.
    /// It fails if Linux or Mac have a significantly different model
    /// for enumerating monitor and window data.
    /// I haven't looked at Linux yet,
    /// but Mac appears to use the same style of global coordinate system as Windows.
    ///
    /// Idea B: provide some base functions and do the 'should I swap?' logic out here.
    /// Where does this succeed or fail?
    /// It succeeds if the increased flexibility is useful for the other OS impls.
    /// It 'fails' if I have to duplicate logic across the OS impls.
    // // Uses MoveWindow. Some other options to consider: SetWindowPlacement and SetWindowPos.
    // let move_window_from_to = |window_info: &WindowInfo,
    //                            from_monitor: &MonitorInfo,
    //                            from_sub_rect: Option<swap::Rect>,
    //                            to_monitor: &MonitorInfo,
    //                            to_sub_rect: Option<swap::Rect>| {
    //     let new_frame = swap::calculate_swap_coords(
    //         swap::MonitorInfo {
    //             rect: win32_rect_to_internal_rect(from_monitor.rect),
    //             sub_rect: from_sub_rect,
    //         },
    //         swap::MonitorInfo {
    //             rect: win32_rect_to_internal_rect(to_monitor.rect),
    //             sub_rect: to_sub_rect,
    //         },
    //         win32_rect_to_internal_rect(window_info.frame),
    //     );
    //     unsafe {
    //         let _ = MoveWindow(
    //             window_info.handle,
    //             new_frame.left + (window_info.rect.left - window_info.frame.left),
    //             new_frame.top + (window_info.rect.top - window_info.frame.top),
    //             new_frame.right - new_frame.left
    //                 + (window_info.frame.left - window_info.rect.left)
    //                 + (window_info.rect.right - window_info.frame.right),
    //             new_frame.bottom - new_frame.top
    //                 + (window_info.frame.top - window_info.rect.top)
    //                 + (window_info.rect.bottom - window_info.frame.bottom),
    //             true,
    //         );
    //     }
    // };
    window_infos.iter().for_each(|window_info| {
        let old_rect = win32_rect_to_internal_rect(window_info.frame);
        let new_rect = calculate_swap_coords(
            region_a_rect,
            region_b_rect,
            old_rect,
            args.overlap_threshold,
        );
        if old_rect == new_rect {
            return;
        }
        let margin_left = window_info.rect.left - window_info.frame.left;
        let margin_right = window_info.rect.right - window_info.frame.right;
        let margin_top = window_info.rect.top - window_info.frame.top;
        let margin_bottom = window_info.rect.bottom - window_info.frame.bottom;
        unsafe {
            let _ = MoveWindow(
                window_info.handle,
                new_rect.left + margin_left,
                new_rect.top + margin_top,
                new_rect.right - new_rect.left - margin_left + margin_right,
                new_rect.bottom - new_rect.top - margin_top + margin_bottom,
                true,
            );
        }

        // if window_info.monitor == monitor_infos[args.monitor_a].handle {
        //     move_window_from_to(
        //         window_info,
        //         &monitor_infos[args.monitor_a],
        //         args.subrectangle_a,
        //         &monitor_infos[args.monitor_b],
        //         args.subrectangle_b,
        //     );
        // }
        // if window_info.monitor == monitor_infos[args.monitor_b].handle {
        //     move_window_from_to(
        //         window_info,
        //         &monitor_infos[args.monitor_b],
        //         args.subrectangle_b,
        //         &monitor_infos[args.monitor_a],
        //         args.subrectangle_a,
        //     );
        // }
    });
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
            _name: name,
        },
    );
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

fn add_info<T>(v: &mut Vec<T>, info: T) {
    v.push(info);
}

fn get_rect_from_region(region: &Region, monitor_infos: &Vec<MonitorInfo>) -> swap::Rect {
    match region {
        Region::Monitor(index) => {
            return win32_rect_to_internal_rect(monitor_infos[*index as usize].rect)
        }
        Region::Rect(r) => return r.clone(),
    }
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
    _name: String,
}

fn win32_rect_to_internal_rect(r: RECT) -> swap::Rect {
    swap::Rect {
        left: r.left,
        right: r.right,
        top: r.top,
        bottom: r.bottom,
    }
}
