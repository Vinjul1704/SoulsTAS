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

#![allow(unsafe_op_in_unsafe_fn)]
#[allow(unused_imports)]

use crate::games::*;
mod console;
mod games;
mod util;

use std::ffi::c_void;
use std::{env, panic, thread};
use std::process::exit;
use mem_rs::prelude::Process;
use windows::Win32::Foundation::{HINSTANCE};
use windows::Win32::System::SystemServices::DLL_PROCESS_ATTACH;

use crate::console::init_console;
use log::{error, info};
use windows::core::BOOL;
use crate::util::{GLOBAL_HMODULE, GLOBAL_VERSION, Version};


#[unsafe(no_mangle)]
#[used]
pub static mut SOULSTAS_PATCHES_INITIALIZED: bool = false;


#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "system" fn DllMain(
    module: HINSTANCE,
    call_reason: u32,
    _reserved: c_void,
) -> BOOL
{
    unsafe
    {
        if call_reason == DLL_PROCESS_ATTACH
        {
            GLOBAL_HMODULE = module;
            GLOBAL_VERSION = Version::from_file_version_info(env::current_exe().unwrap());
            thread::spawn(dispatched_dll_main);
        }

        BOOL(1)
    }
}

fn dispatched_dll_main()
{
    if cfg!(debug_assertions)
    {
        init_console();
    }

    //Redirect panics
    panic::set_hook(Box::new(|i| {
        error!("panic");
        error!("{}", i);
        exit(-1);
    }));

    let process_name = Process::get_current_process_name().unwrap();
    info!("process: {}", process_name);

    #[cfg(target_arch = "x86_64")]
    match process_name.to_lowercase().as_str()
    {
        "armoredcore6.exe" => init_armoredcore6(),
        "darksoulsremastered.exe" => init_darksouls1remastered(),
        "darksoulsii.exe" => init_darksouls2sotfs(),
        "darksoulsiii.exe" => init_darksouls3(),
        "eldenring.exe" => init_eldenring(),
        "sekiro.exe" => init_sekiro(),
        "nightreign.exe" => init_nightreign(),
        _ => info!("no supported process found")
    }

    #[cfg(target_arch = "x86")]
    match process_name.to_lowercase().as_str()
    {
        "darksouls.exe" => init_darksouls1(), // TODO: Handle DATA.exe
        _ => info!("no supported process found")
    }

    unsafe { SOULSTAS_PATCHES_INITIALIZED = true };
}