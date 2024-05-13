#![cfg_attr(not(target_os = "linux"), no_std)]
#![cfg_attr(not(target_os = "linux"), no_main)]

#[cfg(not(target_os = "linux"))]
mod lang_item;

#[cfg(target_os = "linux")]
fn main() {
    crate::top::runtime_main(0, 0);
}
