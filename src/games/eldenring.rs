use std::mem;
use std::os::raw::c_void;

use std::{thread, time::Duration};

use windows::Win32::Foundation::*;
use windows::Win32::System::Memory::*;
use windows::Win32::System::Threading::*;

use mem_rs::prelude::*;

use crate::utils::mem::*;

pub static mut EXPORTS_ER: Vec<ExportedFunction> = Vec::new();


pub unsafe fn er_fps_limit_get(process: &Process) -> f32 {
    let address = EXPORTS_ER
        .iter()
        .find(|f| f.name == "ER_FPS_CUSTOM_LIMIT")
        .expect("Couldn't find ER_FPS_CUSTOM_LIMIT")
        .addr;

    let mut val_buf: [u8; mem::size_of::<f32>()] = [0; mem::size_of::<f32>()];
    process.read_memory_abs(address, &mut val_buf);

    return mem::transmute(val_buf);
}

pub unsafe fn er_fps_limit_set(process: &Process, param: f32) {
    let address = EXPORTS_ER
        .iter()
        .find(|f| f.name == "ER_FPS_CUSTOM_LIMIT")
        .expect("Couldn't find ER_FPS_CUSTOM_LIMIT")
        .addr;

    let mut val_buf: [u8; mem::size_of::<f32>()] = mem::transmute(param);
    process.write_memory_abs(address, &mut val_buf);
}


pub unsafe fn er_fps_patch_get(process: &Process) -> bool {
    let address = EXPORTS_ER
        .iter()
        .find(|f| f.name == "ER_FPS_PATCH_ENABLED")
        .expect("Couldn't find ER_FPS_PATCH_ENABLED")
        .addr;

    let mut val_buf: [u8; mem::size_of::<bool>()] = [0; mem::size_of::<bool>()];
    process.read_memory_abs(address, &mut val_buf);

    return mem::transmute(val_buf);
}

pub unsafe fn er_fps_patch_set(process: &Process, param: bool) {
    let address = EXPORTS_ER
        .iter()
        .find(|f| f.name == "ER_FPS_PATCH_ENABLED")
        .expect("Couldn't find ER_FPS_PATCH_ENABLED")
        .addr;

    let mut val_buf: [u8; mem::size_of::<bool>()] = mem::transmute(param);
    process.write_memory_abs(address, &mut val_buf);
}


pub unsafe fn er_frame_advance_get(process: &Process) -> bool {
    let address = EXPORTS_ER
        .iter()
        .find(|f| f.name == "ER_FRAME_ADVANCE_ENABLED")
        .expect("Couldn't find ER_FRAME_ADVANCE_ENABLED")
        .addr;

    let mut val_buf: [u8; mem::size_of::<bool>()] = [0; mem::size_of::<bool>()];
    process.read_memory_abs(address, &mut val_buf);

    return mem::transmute(val_buf);
}

pub unsafe fn er_frame_advance_set(process: &Process, param: bool) {
    let address = EXPORTS_ER
        .iter()
        .find(|f| f.name == "ER_FRAME_ADVANCE_ENABLED")
        .expect("Couldn't find ER_FRAME_ADVANCE_ENABLED")
        .addr;

    let mut val_buf: [u8; mem::size_of::<bool>()] = mem::transmute(param);
    process.write_memory_abs(address, &mut val_buf);
}


pub unsafe fn er_frame_advance_next(process: &Process) {
    let address = EXPORTS_ER
        .iter()
        .find(|f| f.name == "ER_FRAME_RUNNING")
        .expect("Couldn't find ER_FRAME_RUNNING")
        .addr;

    let mut val_buf: [u8; mem::size_of::<bool>()] = mem::transmute(true);
    process.write_memory_abs(address, &mut val_buf);
}

pub unsafe fn er_frame_advance_wait(process: &Process) {
    let address = EXPORTS_ER
        .iter()
        .find(|f| f.name == "ER_FRAME_RUNNING")
        .expect("Couldn't find ER_FRAME_RUNNING")
        .addr;

    let mut val: bool = true;
    let mut val_buf: [u8; mem::size_of::<bool>()] = [0; mem::size_of::<bool>()];

    while val {
        process.read_memory_abs(address, &mut val_buf);
        val = mem::transmute(val_buf);

        thread::sleep(Duration::from_micros(10));
    }
}
