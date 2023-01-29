mod hack;
mod network;
mod service;
mod transmission;
mod util;

#[export_name = "airx_version"]
pub fn airx_version() -> i32 {
    1
}

pub fn airx_start_service() {
    let airx = service::airx_service::AirXService::default();
    airx.run(std::env::args().collect::<Vec<String>>());
}
