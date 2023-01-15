mod network;
mod transmission;
mod util;
mod service;

use service::airx_service;

fn main() {
    let airx = airx_service::AirXService::default();
    airx.run();
}
