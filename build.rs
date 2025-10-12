use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let manifest_dir = env::var_os("CARGO_MANIFEST_DIR").unwrap();
    let target = env::var_os("TARGET").unwrap();
    let profile = env::var_os("PROFILE").unwrap();

    let binary_path = Path::new(&manifest_dir)
        .join("target")
        .join(target)
        .join(profile);

    let target_arch = env::var_os("CARGO_CFG_TARGET_ARCH").unwrap();


    let soulstas_patches_path = env::var_os("CARGO_CDYLIB_FILE_SOULSTAS_PATCHES_soulstas_patches").unwrap();

    if target_arch == "x86_64" {
        let soulmods_path = env::var_os("CARGO_CDYLIB_FILE_SOULMODS_soulmods").unwrap();
        
        let _ = fs::copy(
            Path::new(&soulmods_path),
            binary_path.join("soulmods_x64.dll"),
        );

        let _ = fs::copy(
            Path::new(&soulstas_patches_path),
            binary_path.join("soulstas_patches_x64.dll"),
        );
    } else if target_arch == "x86" {
        let _ = fs::copy(
            Path::new(&soulstas_patches_path),
            binary_path.join("soulstas_patches_x86.dll"),
        );
    }
}