use libairx::util::shared_mutable::SharedMutable;
use std::thread;

#[test]
fn test_shared_mutable() {
    let shared = SharedMutable::new(1);
    let shared_clone1 = shared.clone();
    let shared_clone2 = shared.clone();

    let thread1 = thread::spawn(move || {
        *shared_clone1.lock().unwrap() += 2;
    });

    let thread2 = thread::spawn(move || {
        *shared_clone2.lock().unwrap() += 1;
    });

    thread1.join().unwrap();
    thread2.join().unwrap();

    assert_eq!(*shared.lock().unwrap(), 4);
}
