mod network;
mod transmission;
mod util;
mod service;
mod hack;

use std::env;
use service::airx_service;

fn main() {
    let airx = airx_service::AirXService::default();
    airx.run(env::args().collect::<Vec<String>>());
}
