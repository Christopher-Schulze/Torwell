use arti_client::TorClient;
use tor_rtcompat::PreferredRuntime;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use anyhow::Result;
use log::error;

pub async fn start_socks_proxy(
    client: TorClient<PreferredRuntime>,
    port: u16
) -> Result<u16> {
    let listener = if port == 0 {
        TcpListener::bind("127.0.0.1:0").await?
    } else {
        TcpListener::bind(("127.0.0.1", port)).await?
    };
    let local_port = listener.local_addr()?.port();

    tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((socket, _)) => {
                    let client = client.clone();
                    tokio::spawn(async move {
                        if let Err(_e) = handle_socks_conn(socket, client).await {
                            // debug!("SOCKS error: {}", e);
                        }
                    });
                }
                Err(e) => error!("SOCKS accept error: {}", e),
            }
        }
    });

    Ok(local_port)
}

async fn handle_socks_conn(mut socket: TcpStream, client: TorClient<PreferredRuntime>) -> Result<()> {
    // Handshake
    let mut buf = [0u8; 2];
    socket.read_exact(&mut buf).await?;
    if buf[0] != 0x05 { return Ok(()); } // Only SOCKS5
    let nmethods = buf[1] as usize;
    let mut methods = vec![0u8; nmethods];
    socket.read_exact(&mut methods).await?;

    // Select No Auth (0x00)
    socket.write_all(&[0x05, 0x00]).await?;

    // Request
    let mut head = [0u8; 4];
    socket.read_exact(&mut head).await?;
    if head[0] != 0x05 || head[1] != 0x01 { return Ok(()); } // CONNECT only

    let addr_type = head[3];
    let dest: String;
    let port: u16;

    match addr_type {
        0x01 => { // IPv4
            let mut ip = [0u8; 4];
            socket.read_exact(&mut ip).await?;
            dest = std::net::Ipv4Addr::from(ip).to_string();
        }
        0x03 => { // Domain
            let mut len = [0u8; 1];
            socket.read_exact(&mut len).await?;
            let mut domain = vec![0u8; len[0] as usize];
            socket.read_exact(&mut domain).await?;
            dest = String::from_utf8_lossy(&domain).to_string();
        }
        0x04 => { // IPv6
             let mut ip = [0u8; 16];
             socket.read_exact(&mut ip).await?;
             dest = std::net::Ipv6Addr::from(ip).to_string();
        }
        _ => return Ok(()),
    }

    let mut p = [0u8; 2];
    socket.read_exact(&mut p).await?;
    port = u16::from_be_bytes(p);

    // Connect via Tor
    let tor_stream = client.connect((dest.as_str(), port)).await;

    match tor_stream {
        Ok(mut stream) => {
             socket.write_all(&[0x05, 0x00, 0x00, 0x01, 0,0,0,0, 0,0]).await?;
             let _ = tokio::io::copy_bidirectional(&mut socket, &mut stream).await;
        }
        Err(_) => {
             socket.write_all(&[0x05, 0x04, 0x00, 0x01, 0,0,0,0, 0,0]).await?; // Host unreachable
        }
    }
    Ok(())
}
