use std::{mem::size_of, os::raw::c_void, str::from_utf8};

use windows::Win32::{
    Foundation::{BOOL, HWND, LPARAM, RECT, TRUE},
    Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED, DWMWA_EXTENDED_FRAME_BOUNDS},
    UI::WindowsAndMessaging::{
        EnumWindows, GetWindowInfo, GetWindowTextA, GetWindowTextLengthA, GetWindowTextLengthW,
        GetWindowTextW, IsWindowVisible, MoveWindow, WINDOWINFO, WS_EX_APPWINDOW, WS_EX_TOOLWINDOW,
    },
};

unsafe extern "system" fn enum_proc(window: HWND, _lp: LPARAM) -> BOOL {
    if !IsWindowVisible(window).as_bool() {
        return TRUE;
    }

    let mut window_info = WINDOWINFO::default();
    let _ = GetWindowInfo(window, &mut window_info as *mut WINDOWINFO);

    let is_tool_window = (window_info.dwExStyle.0 & WS_EX_TOOLWINDOW.0) != 0;
    if is_tool_window {
        return TRUE;
    }
    let mut cloaked = 0;
    let _ = DwmGetWindowAttribute(
        window,
        DWMWA_CLOAKED,
        &mut cloaked as *mut _ as *mut c_void,
        4,
    );
    let is_cloaked = cloaked != 0;
    if is_cloaked {
        return TRUE;
    }

    let rc_window = window_info.rcWindow;
    println!(
        "Window coords: ({}, {}) to ({}, {})",
        rc_window.left, rc_window.top, rc_window.right, rc_window.bottom
    );
    let text_len = GetWindowTextLengthA(window);
    let mut text_buffer = vec![u8::default(); text_len as usize];
    let text_len_w = GetWindowTextLengthW(window);
    let mut text_buffer_w = vec![u16::default(); text_len_w as usize];
    GetWindowTextA(window, &mut text_buffer);
    GetWindowTextW(window, &mut text_buffer_w);

    let _ = MoveWindow(
        window,
        rc_window.left + 100,
        rc_window.top,
        rc_window.right - rc_window.left,
        rc_window.bottom - rc_window.top,
        true,
    );
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
    println!("WS_EX_TOOLWINDOW = {}", is_tool_window);
    println!("DWMWA_CLOAKED = {}", is_cloaked);
    println!("-----------------------------------------------");
    return TRUE;
}
fn main() {
    println!("Hello, world!");
    let lparam = LPARAM { 0: 0 };
    unsafe {
        let _ = EnumWindows(Some(enum_proc), lparam);
    };
}
