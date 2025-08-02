use std::mem;
use std::os::raw::c_void;

use windows::Win32::Foundation::*;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::System::Diagnostics::Debug::*;
use windows::Win32::System::SystemServices::*;
use windows::Win32::System::Threading::*;
use windows::Win32::UI::WindowsAndMessaging::*;

use windows_core::BOOL;

use mem_rs::prelude::*;

pub struct ModuleExport {
    pub name: String,
    pub addr: usize,
}

struct WindowData {
    id: u32,
    hwnd: HWND,
}

pub unsafe fn read_ascii_string(process: &Process, address: usize) -> String {
    let mut offset: usize = 0;
    let end_byte: u8 = 0x0;

    let mut output_string: String = String::from("");

    loop {
        let mut single_char_buf: [u8; 1] = [0];
        process.read_memory_abs(address + offset as usize, &mut single_char_buf);
        let single_char: u8 = std::ptr::read(single_char_buf.as_ptr() as *const _);

        if single_char == end_byte {
            break;
        }

        output_string.push(single_char as char);

        offset += 1;

        if offset > 512 {
            panic!("String too long!");
        }
    }

    return output_string;
}

pub unsafe fn get_module(process: &mut Process, module_name: &str) -> Option<ProcessModule> {
    return process.get_modules()
        .iter()
        .find(|m| m.name == module_name)
        .cloned();
}

pub unsafe fn get_exports(process: &mut Process, module: ProcessModule) -> Vec<ModuleExport> {
    process.refresh().expect("Failed to attach/refresh");

    let export_tuples = module.get_exports();
    let mut exports: Vec<ModuleExport> = Vec::new();

    for export_tuple in export_tuples.iter() {
        exports.push(ModuleExport {
            name: export_tuple.0.clone(),
            addr: export_tuple.1,
        });
    }

    return exports;
}

pub unsafe fn get_hwnd_by_id(process_id: u32) -> HWND {
    let mut window_data = Box::new(WindowData {
        id: process_id,
        hwnd: HWND(0 as *mut c_void),
    });

    let window_data_ptr: *mut WindowData = &mut *window_data;

    let _ = EnumWindows(
        Some(get_hwnd_by_id_callback),
        LPARAM(window_data_ptr as isize),
    );

    return window_data.hwnd;
}

#[allow(unused_mut)]
unsafe extern "system" fn get_hwnd_by_id_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let mut window_data: &mut WindowData = &mut *(lparam.0 as *mut WindowData);

    let mut window_id: u32 = 0;
    GetWindowThreadProcessId(hwnd, Some(&mut window_id));

    if window_id == window_data.id.try_into().unwrap() {
        window_data.hwnd = hwnd;
        return BOOL(0);
    }

    return BOOL(1);
}
