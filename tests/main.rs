extern crate airx;

use airx::lib_generic;

#[test]
fn it_works() {
    assert!(lib_generic::airx_version() > 20230000);
    assert!(lib_generic::airx_version() < 20240000);
}
