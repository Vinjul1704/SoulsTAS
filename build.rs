use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let soulmods_path = env::var_os("CARGO_CDYLIB_FILE_SOULMODS_soulmods").unwrap();

    let manifest_dir = env::var_os("CARGO_MANIFEST_DIR").unwrap();
    let target = env::var_os("TARGET").unwrap();
    let profile = env::var_os("PROFILE").unwrap();

    let binary_path = Path::new(&manifest_dir)
        .join("target")
        .join(target)
        .join(profile);

    let _ = fs::copy(Path::new(&soulmods_path), binary_path.join("soulmods.dll"));
}
