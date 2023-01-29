mod hack;
mod network;
mod service;
mod transmission;
mod util;

use service::airx_service;
use std::env;

fn main() {
    let airx = airx_service::AirXService::default();
    airx.run(env::args().collect::<Vec<String>>());
}
