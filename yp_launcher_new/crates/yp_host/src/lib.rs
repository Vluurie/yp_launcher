pub mod controller;
pub mod discovery;
pub mod game_probe;
pub mod gpu;
pub mod launch;
pub mod nams_cli;
pub mod nams_log;
pub mod platform;
pub mod reveal;
pub mod session;
pub mod shortcut;
pub mod vdf;
pub mod win10_compat;
pub mod wine;

pub use platform::{adapter_for, HostPlatform};
