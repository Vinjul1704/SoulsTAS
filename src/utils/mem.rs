use std::mem;
use std::os::raw::c_void;

use windows::Win32::Foundation::*;
use windows::Win32::Foundation::{BOOL, HWND, LPARAM, WPARAM};
use windows::Win32::System::Diagnostics::Debug::*;
use windows::Win32::System::SystemServices::*;
use windows::Win32::System::Threading::*;
use windows::Win32::UI::WindowsAndMessaging::*;

use mem_rs::prelude::*;

pub struct ExportedFunction {
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
    process.refresh().expect("Failed to attach/refresh");

    let process_handle_result = OpenProcess(
        PROCESS_CREATE_THREAD
            | PROCESS_QUERY_INFORMATION
            | PROCESS_VM_OPERATION
            | PROCESS_VM_WRITE
            | PROCESS_VM_READ,
        false,
        process.get_id(),
    );

    let process_handle = process_handle_result.unwrap();
    let process_modules = Process::get_process_modules(process_handle);
    let _ = CloseHandle(process_handle);

    return process_modules
        .iter()
        .find(|m| m.name == module_name)
        .cloned();
}

pub unsafe fn get_exports(process: &mut Process, module: ProcessModule) -> Vec<ExportedFunction> {
    let mut dos_header_buf: [u8; mem::size_of::<IMAGE_DOS_HEADER>()] =
        [0; mem::size_of::<IMAGE_DOS_HEADER>()];
    process.read_memory_abs(module.base_address, &mut dos_header_buf);
    let dos_header: IMAGE_DOS_HEADER = std::ptr::read(dos_header_buf.as_ptr() as *const _);

    let mut nt_headers_buf: [u8; mem::size_of::<IMAGE_NT_HEADERS64>()] =
        [0; mem::size_of::<IMAGE_NT_HEADERS64>()];
    process.read_memory_abs(
        module.base_address + dos_header.e_lfanew as usize,
        &mut nt_headers_buf,
    );
    let nt_headers: IMAGE_NT_HEADERS64 = std::ptr::read(nt_headers_buf.as_ptr() as *const _);

    let mut export_table_buf: [u8; mem::size_of::<IMAGE_EXPORT_DIRECTORY>()] =
        [0; mem::size_of::<IMAGE_EXPORT_DIRECTORY>()];
    process.read_memory_abs(
        module.base_address + nt_headers.OptionalHeader.DataDirectory[0].VirtualAddress as usize,
        &mut export_table_buf,
    );
    let export_table: IMAGE_EXPORT_DIRECTORY =
        std::ptr::read(export_table_buf.as_ptr() as *const _);

    let name_offset_table = module.base_address + export_table.AddressOfNames as usize;
    let ordinal_table = module.base_address + export_table.AddressOfNameOrdinals as usize;
    let function_offset_table = module.base_address + export_table.AddressOfFunctions as usize;

    let mut funcs: Vec<ExportedFunction> = Vec::new();

    for i in 0..export_table.NumberOfNames {
        let mut func_name_offset_buf: [u8; mem::size_of::<u32>()] = [0; mem::size_of::<u32>()];
        process.read_memory_abs(
            name_offset_table + i as usize * mem::size_of::<u32>(),
            &mut func_name_offset_buf,
        );
        let func_name_offset: u32 = std::ptr::read(func_name_offset_buf.as_ptr() as *const _);

        let func_name =
            read_ascii_string(&process, module.base_address + func_name_offset as usize);

        let mut ordinal_index_buf: [u8; mem::size_of::<u16>()] = [0; mem::size_of::<u16>()];
        process.read_memory_abs(
            ordinal_table + i as usize * mem::size_of::<u16>(),
            &mut ordinal_index_buf,
        );
        let ordinal_index: u16 = std::ptr::read(ordinal_index_buf.as_ptr() as *const _);

        let mut func_offset_buf: [u8; mem::size_of::<usize>()] = [0; mem::size_of::<usize>()];
        process.read_memory_abs(
            function_offset_table + ordinal_index as usize * mem::size_of::<u32>(),
            &mut func_offset_buf,
        );
        let func_offset: u32 = std::ptr::read(func_offset_buf.as_ptr() as *const _);

        let func_addr: usize = module.base_address + func_offset as usize;

        funcs.push(ExportedFunction {
            name: func_name,
            addr: func_addr,
        });
    }

    return funcs;
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
