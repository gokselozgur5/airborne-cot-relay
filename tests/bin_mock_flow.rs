use std::net::{Ipv4Addr, UdpSocket};
use std::time::Duration;

fn main() {
    // Create a UDP socket bound to localhost:5000
    let socket = UdpSocket::bind("127.0.0.1:5000").unwrap();

    // Send a datagram with version 2.0 and uid starting with TRACK-
    let payload = b"<CotEvent xmlns=\"http://www.example.com/cot/2.0\"><uid>TRACK-1234</uid><data>Hello, world!</data></CotEvent>";
    socket.send_to(payload, Ipv4Addr::new(127, 0, 0, 1)).unwrap();

    // Wait for the datagram to be received
    let mut buffer = [0; 1024];
    let (src_addr, _) = socket.recv_from(&mut buffer).unwrap();

    // Parse the received datagram
    let received_event = CotEvent::from_xml(buffer).unwrap();

    // Check if the parsed event meets the acceptance criteria
    assert_eq!(received_event.version(), "2.0");
    assert!(received_event.uid().starts_with("TRACK-"));

    println!("Received datagram: {:?}", received_event);
}
