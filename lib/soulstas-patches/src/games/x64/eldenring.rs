// This file is part of the SoulSplitter distribution (https://github.com/FrankvdStam/SoulSplitter).
// Copyright (c) 2022 Frank van der Stam.
// https://github.com/FrankvdStam/SoulSplitter/blob/main/LICENSE
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.
//
// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

use std::{thread, time::Duration, mem};

use ilhook::x64::{Hooker, HookType, Registers, CallbackOption, HookFlags, HookPoint};
use mem_rs::prelude::*;

use log::info;

use windows::Win32::UI::Input::XboxController::*;

use crate::util::GLOBAL_VERSION;


static mut FRAME_ADVANCE_HOOK: Option<HookPoint> = None;
static mut XINPUT_HOOK: Option<HookPoint> = None;


#[unsafe(no_mangle)]
#[used]
pub static mut ER_FRAME_ADVANCE_ENABLED: bool = false;

#[unsafe(no_mangle)]
#[used]
pub static mut ER_FRAME_RUNNING: bool = false;

#[unsafe(no_mangle)]
#[used]
pub static mut ER_XINPUT_PATCH_ENABLED: bool = false;

#[unsafe(no_mangle)]
#[used]
pub static mut ER_XINPUT_STATE: XINPUT_STATE = XINPUT_STATE {
    dwPacketNumber: 0,
    Gamepad: XINPUT_GAMEPAD {
        wButtons: XINPUT_GAMEPAD_BUTTON_FLAGS(0),
        bLeftTrigger: 0,
        bRightTrigger: 0,
        sThumbLX: 0,
        sThumbLY: 0,
        sThumbRX: 0,
        sThumbRY: 0,
    }
};

pub type XInputGetState = unsafe extern "system" fn(dw_user_index: u32, p_state: *mut XINPUT_STATE) -> u32;


#[allow(unused_assignments)]
pub fn init_eldenring()
{
    unsafe
    {
        info!("version: {}", GLOBAL_VERSION);
        
        // Get ER process
        let mut process = Process::new_with_memory_type("eldenring.exe", MemoryType::Direct);
        process.refresh().unwrap();


        // AoB scan for frame advance patch
        let fn_frame_advance_address = process.scan_abs("frame_advance", "e8 ? ? ? ? e8 ? ? ? ? 84 c0 74 4f", 21, Vec::new()).unwrap().get_base_address();
        info!("Frame advance at 0x{:x}", fn_frame_advance_address);

        // Enable frame advance patch
        FRAME_ADVANCE_HOOK = Some(Hooker::new(fn_frame_advance_address, HookType::JmpBack(frame_advance), CallbackOption::None, 0, HookFlags::empty()).hook().unwrap());


        // Find XInputGetState function in XINPUT1_4.dll
        let xinput_module = process.get_modules().iter().find(|m| m.name == "XINPUT1_4.dll").cloned().expect("Couldn't find XINPUT1_4.dll");
        let xinput_fn_addr = xinput_module.get_exports().iter().find(|e| e.0 == "XInputGetState").expect("Couldn't find XInputGetState").1;
        info!("XInputGetState at 0x{:x}", xinput_fn_addr);

        // Hook XInputGetState
        XINPUT_HOOK = Some(Hooker::new(xinput_fn_addr, HookType::Retn(xinput_fn), CallbackOption::None, 0, HookFlags::empty()).hook().unwrap());
    }
}


// Frame advance patch
unsafe extern "win64" fn frame_advance(_registers: *mut Registers, _:usize)
{
    unsafe
    {
        if ER_FRAME_ADVANCE_ENABLED
        {
            ER_FRAME_RUNNING = false;

            while !ER_FRAME_RUNNING && ER_FRAME_ADVANCE_ENABLED {
                thread::sleep(Duration::from_micros(10));
            }
        }
    }
}


pub unsafe extern "win64" fn xinput_fn(registers: *mut Registers, orig_func_ptr: usize, _: usize) -> usize
{
    unsafe {
        let dw_user_index = (*registers).rcx as u32;
        let p_state = (*registers).rdx as *mut XINPUT_STATE;

        if !ER_XINPUT_PATCH_ENABLED {
            let orig_func: XInputGetState = mem::transmute(orig_func_ptr);
            return orig_func(dw_user_index, p_state) as usize;
        }

        (*p_state) = ER_XINPUT_STATE;

        // ERROR_SUCCESS = 0x0
        // ERROR_DEVICE_NOT_CONNECTED = 0x48F
        return 0x0;
    }
}