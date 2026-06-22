pub mod collector;
pub mod domain;
pub mod service;
mod unsupported;
#[cfg(any(target_os = "windows", test))]
mod windows;
