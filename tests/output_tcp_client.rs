use std::net::{TcpListener, TcpStream};
use std::thread;

use airborne_cot_relay::output::TcpClientSink;
use airborne_cot_relay::output::Backoff;

#[tokio::test]
async fn test_tcp_client_sink() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let (tx, rx) = tokio::sync::mpsc::channel(1);

    thread::spawn(async move {
        while let Ok(payload) = rx.recv() {
            println!("Received payload: {:?}", payload);
            if payload == b"hello" {
                listener.accept().await.unwrap().shutdown_stdin().unwrap();
            }
        }
    });

    let mut sink = TcpClientSink::builder(Backoff::default())
        .with_backoff(Backoff::exponential(Duration::from_secs(1)))
        .build()
        .await
        .unwrap();

    sink.send(b"hello").await.unwrap();
    sink.close().await.unwrap();

    let mut stream = listener.accept().await.unwrap();
    let mut buffer = [0; 1024];
    match stream.read(&mut buffer).await {
        Ok(n) => assert_eq!(n, 5), // "hello" is 5 bytes long
        Err(e) => panic!("Failed to read from TCP stream: {:?}", e),
    }

    let mut stream = listener.accept().await.unwrap();
    match stream.read(&mut buffer).await {
        Ok(n) => assert_eq!(n, 0), // No more data expected
        Err(e) => panic!("Failed to read from TCP stream: {:?}", e),
    }
}
