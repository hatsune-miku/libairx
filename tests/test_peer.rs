use airx::network::peer::Peer;

/// Equation of two peers is determined only by their hosts.
#[test]
fn host() {
    let peer1 = Peer::new(&String::from("114.51.41.91"), 9818);
    let peer2 = Peer::new(&String::from("114.51.41.91"), 9819);
    let peer3 = Peer::new(&String::from("111.111.11.1"), 9819);

    assert!(peer1 == peer2);
    assert!(peer1 != peer3);
}
