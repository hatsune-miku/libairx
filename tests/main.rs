extern crate libairx;

#[test]
fn it_works() {
    assert!(libairx::airx_version() > 20230000);
    assert!(libairx::airx_version() < 20240000);
}
