use std::os::raw::c_void;

use std::path::PathBuf;
use std::env;

use windows::Win32::Foundation::{HWND, LPARAM};
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

pub unsafe fn get_module(process: &mut Process, module_name: &str) -> Option<ProcessModule> {
    return process
        .get_modules()
        .iter()
        .find(|m| m.name == module_name)
        .cloned();
}

pub unsafe fn get_or_inject_module(process: &mut Process, module_name: &str) -> Option<ProcessModule> {
    if let Some(module_existing) = get_module(process, module_name) {
        return Some(module_existing);
    } else {
        let exe_path = env::current_exe().unwrap();
        let module_path = PathBuf::from(exe_path)
            .parent()
            .unwrap()
            .join(module_name);

        process
            .inject_dll(module_path.into_os_string().to_str().unwrap())
            .expect("Failed to inject module!");

        if let Some(module_injected) = get_module(process, module_name) {
            return Some(module_injected);
        } else {
            return None;
        }
    }
}

pub unsafe fn get_exports(module: ProcessModule) -> Vec<ModuleExport> {
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
