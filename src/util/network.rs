use std::{net::IpAddr, time::Duration};
use interfaces::Interface;
use mdns::Error;

#[cfg(test)]
mod network {
    use std::io::{stdout, Write};
    use std::pin::Pin;
    use std::thread::sleep;
    use std::time::Duration;
    use mdns::Error;
    use futures::{pin_mut, StreamExt};
    use futures::executor::block_on;
    use futures::Future;

    enum ThreadKind {
        Server,
        Client,
    }

    const SERVER_NAME: &'static str = "_airx._tcp.local";

    async fn f(kind: ThreadKind) -> Result<(), String> {
        match kind {
            ThreadKind::Server => {
                println!("Start discovery...");
                let discovery = match mdns::discover::all(
                    SERVER_NAME,
                    std::time::Duration::from_secs(5)) {
                    Ok(d) => d,
                    Err(e) => return Err(e.to_string())
                };

                let mut stream = discovery.listen();
                futures::pin_mut!(stream);

                while let Some(Ok(response)) = stream.next().await {
                    println!("Server: {}", response.ip_addr().unwrap());
                    break;
                }
                Ok(())
            }
            ThreadKind::Client => {
                match mdns::resolve::one(SERVER_NAME, "127.0.0.1", Duration::from_secs(6)).await {
                    Ok(Some(response)) => {
                        println!("Client: {}", response.ip_addr().unwrap());
                        Ok(())
                    },
                    Ok(None) => Err(String::from("None")),
                    Err(e) => Err(e.to_string())
                }
            }
        }
    }


    #[tokio::test]
    async fn test() {
        println!("Start test...");
        let f1 = f(ThreadKind::Server);
        let f2 = f(ThreadKind::Client);

        let res = futures::future::join(f1, f2).await;
    }
}
