use std::mem;
use std::os::raw::c_void;

use std::{thread, time::Duration};

use windows::Win32::Foundation::*;
use windows::Win32::System::Memory::*;
use windows::Win32::System::Threading::*;

use mem_rs::prelude::*;

use crate::utils::mem::*;

pub static mut EXPORTED_FUNCS_ER: Vec<ExportedFunction> = Vec::new();
pub static mut FRAME_RUNNING_ADDR: Option<usize> = None;

pub unsafe fn er_fps_limit_get(process: &Process) -> f32 {
    let func_addr = EXPORTED_FUNCS_ER
        .iter()
        .find(|f| f.name == "fps_limit_get")
        .expect("Couldn't find fps_limit_get")
        .addr;

    let handle = OpenProcess(
        PROCESS_CREATE_THREAD
            | PROCESS_QUERY_INFORMATION
            | PROCESS_VM_OPERATION
            | PROCESS_VM_WRITE
            | PROCESS_VM_READ,
        false,
        process.get_id(),
    )
    .unwrap();

    let param_addr = VirtualAllocEx(
        handle,
        None,
        mem::size_of::<f32>(),
        MEM_COMMIT | MEM_RESERVE,
        PAGE_EXECUTE_READWRITE,
    );

    let thread = CreateRemoteThread(
        handle,
        None,
        0,
        Some(*(&func_addr as *const _ as *const extern "system" fn(*mut c_void) -> u32)),
        Some(param_addr),
        0,
        None,
    );

    let _ = WaitForSingleObject(thread.unwrap(), 10000);

    let mut param_buf: [u8; mem::size_of::<f32>()] = [0; mem::size_of::<f32>()];
    process.read_memory_abs(param_addr as usize, &mut param_buf);
    let param: f32 = mem::transmute(param_buf);

    let _ = VirtualFreeEx(handle, param_addr, 0, MEM_RELEASE);
    let _ = CloseHandle(handle);

    return param;
}

pub unsafe fn er_fps_limit_set(process: &Process, param: f32) {
    let func_addr = EXPORTED_FUNCS_ER
        .iter()
        .find(|f| f.name == "fps_limit_set")
        .expect("Couldn't find fps_limit_set")
        .addr;

    let handle = OpenProcess(
        PROCESS_CREATE_THREAD
            | PROCESS_QUERY_INFORMATION
            | PROCESS_VM_OPERATION
            | PROCESS_VM_WRITE
            | PROCESS_VM_READ,
        false,
        process.get_id(),
    )
    .unwrap();

    let param_addr = VirtualAllocEx(
        handle,
        None,
        mem::size_of::<f32>(),
        MEM_COMMIT | MEM_RESERVE,
        PAGE_EXECUTE_READWRITE,
    );
    let mut param_buf = param.to_ne_bytes();
    process.write_memory_abs(param_addr as usize, &mut param_buf);

    let thread = CreateRemoteThread(
        handle,
        None,
        0,
        Some(*(&func_addr as *const _ as *const extern "system" fn(*mut c_void) -> u32)),
        Some(param_addr),
        0,
        None,
    );

    let _ = WaitForSingleObject(thread.unwrap(), 10000);
    let _ = VirtualFreeEx(handle, param_addr, 0, MEM_RELEASE);
    let _ = CloseHandle(handle);
}

pub unsafe fn er_fps_patch_get(process: &Process) -> bool {
    let func_addr = EXPORTED_FUNCS_ER
        .iter()
        .find(|f| f.name == "fps_patch_get")
        .expect("Couldn't find fps_patch_get")
        .addr;

    let handle = OpenProcess(
        PROCESS_CREATE_THREAD
            | PROCESS_QUERY_INFORMATION
            | PROCESS_VM_OPERATION
            | PROCESS_VM_WRITE
            | PROCESS_VM_READ,
        false,
        process.get_id(),
    )
    .unwrap();

    let param_addr = VirtualAllocEx(
        handle,
        None,
        mem::size_of::<bool>(),
        MEM_COMMIT | MEM_RESERVE,
        PAGE_EXECUTE_READWRITE,
    );

    let thread = CreateRemoteThread(
        handle,
        None,
        0,
        Some(*(&func_addr as *const _ as *const extern "system" fn(*mut c_void) -> u32)),
        Some(param_addr),
        0,
        None,
    );

    let _ = WaitForSingleObject(thread.unwrap(), 10000);

    let mut param_buf: [u8; mem::size_of::<bool>()] = [0; mem::size_of::<bool>()];
    process.read_memory_abs(param_addr as usize, &mut param_buf);
    let param: bool = mem::transmute(param_buf);

    let _ = VirtualFreeEx(handle, param_addr, 0, MEM_RELEASE);
    let _ = CloseHandle(handle);

    return param;
}

pub unsafe fn er_fps_patch_set(process: &Process, param: bool) {
    let func_addr = EXPORTED_FUNCS_ER
        .iter()
        .find(|f| f.name == "fps_patch_set")
        .expect("Couldn't find fps_patch_set")
        .addr;

    let handle = OpenProcess(
        PROCESS_CREATE_THREAD
            | PROCESS_QUERY_INFORMATION
            | PROCESS_VM_OPERATION
            | PROCESS_VM_WRITE
            | PROCESS_VM_READ,
        false,
        process.get_id(),
    )
    .unwrap();

    let param_addr = VirtualAllocEx(
        handle,
        None,
        mem::size_of::<bool>(),
        MEM_COMMIT | MEM_RESERVE,
        PAGE_EXECUTE_READWRITE,
    );
    let mut param_buf: [u8; mem::size_of::<bool>()] = mem::transmute(param);
    process.write_memory_abs(param_addr as usize, &mut param_buf);

    let thread = CreateRemoteThread(
        handle,
        None,
        0,
        Some(*(&func_addr as *const _ as *const extern "system" fn(*mut c_void) -> u32)),
        Some(param_addr),
        0,
        None,
    );

    let _ = WaitForSingleObject(thread.unwrap(), 10000);
    let _ = VirtualFreeEx(handle, param_addr, 0, MEM_RELEASE);
    let _ = CloseHandle(handle);
}

pub unsafe fn er_frame_advance_get(process: &Process) -> bool {
    let func_addr = EXPORTED_FUNCS_ER
        .iter()
        .find(|f| f.name == "frame_advance_get")
        .expect("Couldn't find frame_advance_get")
        .addr;

    let handle = OpenProcess(
        PROCESS_CREATE_THREAD
            | PROCESS_QUERY_INFORMATION
            | PROCESS_VM_OPERATION
            | PROCESS_VM_WRITE
            | PROCESS_VM_READ,
        false,
        process.get_id(),
    )
    .unwrap();

    let param_addr = VirtualAllocEx(
        handle,
        None,
        mem::size_of::<bool>(),
        MEM_COMMIT | MEM_RESERVE,
        PAGE_EXECUTE_READWRITE,
    );

    let thread = CreateRemoteThread(
        handle,
        None,
        0,
        Some(*(&func_addr as *const _ as *const extern "system" fn(*mut c_void) -> u32)),
        Some(param_addr),
        0,
        None,
    );

    let _ = WaitForSingleObject(thread.unwrap(), 10000);

    let mut param_buf: [u8; mem::size_of::<bool>()] = [0; mem::size_of::<bool>()];
    process.read_memory_abs(param_addr as usize, &mut param_buf);
    let param: bool = mem::transmute(param_buf);

    let _ = VirtualFreeEx(handle, param_addr, 0, MEM_RELEASE);
    let _ = CloseHandle(handle);

    return param;
}

pub unsafe fn er_frame_advance_set(process: &Process, param: bool) {
    let func_addr = EXPORTED_FUNCS_ER
        .iter()
        .find(|f| f.name == "frame_advance_set")
        .expect("Couldn't find frame_advance_set")
        .addr;

    let handle = OpenProcess(
        PROCESS_CREATE_THREAD
            | PROCESS_QUERY_INFORMATION
            | PROCESS_VM_OPERATION
            | PROCESS_VM_WRITE
            | PROCESS_VM_READ,
        false,
        process.get_id(),
    )
    .unwrap();

    let param_addr = VirtualAllocEx(
        handle,
        None,
        mem::size_of::<bool>(),
        MEM_COMMIT | MEM_RESERVE,
        PAGE_EXECUTE_READWRITE,
    );
    let mut param_buf: [u8; mem::size_of::<bool>()] = mem::transmute(param);
    process.write_memory_abs(param_addr as usize, &mut param_buf);

    let thread = CreateRemoteThread(
        handle,
        None,
        0,
        Some(*(&func_addr as *const _ as *const extern "system" fn(*mut c_void) -> u32)),
        Some(param_addr),
        0,
        None,
    );

    let _ = WaitForSingleObject(thread.unwrap(), 10000);
    let _ = VirtualFreeEx(handle, param_addr, 0, MEM_RELEASE);
    let _ = CloseHandle(handle);
}

pub unsafe fn er_frame_advance_get_pointers(process: &Process) {
    let func_addr = EXPORTED_FUNCS_ER
        .iter()
        .find(|f| f.name == "frame_advance_get_pointers")
        .expect("Couldn't find frame_advance_get_pointers")
        .addr;

    let handle = OpenProcess(
        PROCESS_CREATE_THREAD
            | PROCESS_QUERY_INFORMATION
            | PROCESS_VM_OPERATION
            | PROCESS_VM_WRITE
            | PROCESS_VM_READ,
        false,
        process.get_id(),
    )
    .expect("Failed to get handle for frame_advance_get_pointers");

    let param_addr = VirtualAllocEx(
        handle,
        None,
        mem::size_of::<usize>(),
        MEM_COMMIT | MEM_RESERVE,
        PAGE_EXECUTE_READWRITE,
    );

    let thread = CreateRemoteThread(
        handle,
        None,
        0,
        Some(*(&func_addr as *const _ as *const extern "system" fn(*mut c_void) -> u32)),
        Some(param_addr),
        0,
        None,
    )
    .expect("Failed to call frame_advance_get_pointers");

    let _ = WaitForSingleObject(thread, 10000);

    let mut param_buf: [u8; mem::size_of::<usize>()] = [0; mem::size_of::<usize>()];
    process.read_memory_abs(param_addr as usize, &mut param_buf);
    let param: usize = mem::transmute(param_buf);

    if param != 0 {
        FRAME_RUNNING_ADDR = Some(param);
    }

    let _ = VirtualFreeEx(handle, param_addr, 0, MEM_RELEASE);
    let _ = CloseHandle(handle);
}

pub unsafe fn er_frame_advance_next(process: &Process) {
    let mut frame_running_buf: [u8; mem::size_of::<bool>()] = mem::transmute(true);
    process.write_memory_abs(
        FRAME_RUNNING_ADDR.expect("FRAME_RUNNING_ADDR not set"),
        &mut frame_running_buf,
    );
}

pub unsafe fn er_frame_advance_wait(process: &Process) {
    let mut frame_running: bool = true;
    let mut frame_running_buf: [u8; mem::size_of::<bool>()] = [0; mem::size_of::<bool>()];

    while frame_running {
        process.read_memory_abs(
            FRAME_RUNNING_ADDR.expect("FRAME_RUNNING_ADDR not set"),
            &mut frame_running_buf,
        );
        frame_running = mem::transmute(frame_running_buf);

        thread::sleep(Duration::from_micros(10));
    }
}
