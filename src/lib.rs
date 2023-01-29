mod network;
mod service;
mod transmission;
mod util;

#[export_name = "airx_version"]
pub fn airx_version() -> i32 {
    114514
}

pub fn airx_start_service() {}
