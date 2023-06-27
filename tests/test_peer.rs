use libairx::network::peer::Peer;

/// Equation of two peers is determined only by their hosts.
#[test]
fn host() {
    let peer1 = Peer::new(&String::from("114.51.41.91"), 9818, Some(&String::from("B612")));
    let peer2 = Peer::new(&String::from("114.51.41.91"), 9819, Some(&String::from("M78")));
    let peer3 = Peer::new(&String::from("111.111.11.1"), 9819, Some(&String::from("Jarilo-VI")));

    assert!(peer1 == peer2);
    assert!(peer1 != peer3);
    assert_eq!(peer2.to_string(), String::from("M78@114.51.41.91:9819"));
}
