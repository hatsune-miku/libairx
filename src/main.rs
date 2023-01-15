mod network;
mod transmission;
mod util;
mod service;
mod r#unsafe;

use service::airx_service;

fn main() {
    let airx = airx_service::AirXService::default();
    airx.run();
}
