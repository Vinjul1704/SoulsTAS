pub(crate) mod shared;

#[cfg(target_arch = "x86")]
pub(crate) mod ds1;

#[cfg(target_arch = "x86_64")]
pub(crate) mod ds1r;

/*
#[cfg(target_arch = "x86")]
pub(crate) mod ds2;

#[cfg(target_arch = "x86_64")]
pub(crate) mod ds2sotfs;
*/

#[cfg(target_arch = "x86_64")]
pub(crate) mod ds3;

#[cfg(target_arch = "x86_64")]
pub(crate) mod sekiro;

#[cfg(target_arch = "x86_64")]
pub(crate) mod eldenring;

#[cfg(target_arch = "x86_64")]
pub(crate) mod armoredcore6;

#[cfg(target_arch = "x86_64")]
pub(crate) mod nightreign;


pub use shared::*;

#[cfg(target_arch = "x86")]
pub use ds1::*;

#[cfg(target_arch = "x86_64")]
pub use ds1r::*;

/*
#[cfg(target_arch = "x86")]
pub use ds2::*;

#[cfg(target_arch = "x86_64")]
pub use ds2sotfs::*;
*/

#[cfg(target_arch = "x86_64")]
pub use ds3::*;

#[cfg(target_arch = "x86_64")]
pub use sekiro::*;

#[cfg(target_arch = "x86_64")]
pub use eldenring::*;

#[cfg(target_arch = "x86_64")]
pub use armoredcore6::*;

#[cfg(target_arch = "x86_64")]
pub use nightreign::*;