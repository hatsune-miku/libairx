extern crate airx;

#[test]
fn it_works() {
    assert!(airx::airx_version() > 20230000);
    assert!(airx::airx_version() < 20240000);
}
