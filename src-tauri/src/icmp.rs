use rand::random;
use std::io;
use std::net::{IpAddr, SocketAddr};
use surge_ping::{Client, Config, ICMP, PingIdentifier, PingSequence};

async fn resolve_host(host: &str) -> io::Result<IpAddr> {
    if let Ok(ip) = host.parse() {
        return Ok(ip);
    }
    let mut addrs = tokio::net::lookup_host((host, 0)).await?;
    addrs
        .next()
        .map(|a| a.ip())
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "failed to resolve host"))
}

pub async fn ping_host(host: &str, count: u8) -> io::Result<u64> {
    let ip = resolve_host(host).await?;
    ping_ip(ip, count).await
}

pub async fn ping_ip(ip: IpAddr, count: u8) -> io::Result<u64> {
    let config = match ip {
        IpAddr::V4(_) => Config::default(),
        IpAddr::V6(_) => Config::builder().kind(ICMP::V6).build(),
    };
    let client = Client::new(&config)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let mut pinger = client
        .pinger(ip, PingIdentifier(random()))
        .await;
    let mut total: u128 = 0;
    for seq in 0..count {
        let (_, dur) = pinger
            .ping(PingSequence(seq.into()), &[])
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        total += dur.as_millis();
    }
    Ok((total / count as u128) as u64)
}

pub async fn ping_host_series(host: &str, count: u8) -> io::Result<Vec<u64>> {
    let ip = resolve_host(host).await?;
    ping_ip_series(ip, count).await
}

pub async fn ping_ip_series(ip: IpAddr, count: u8) -> io::Result<Vec<u64>> {
    let config = match ip {
        IpAddr::V4(_) => Config::default(),
        IpAddr::V6(_) => Config::builder().kind(ICMP::V6).build(),
    };
    let client = Client::new(&config)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let mut pinger = client
        .pinger(ip, PingIdentifier(random()))
        .await;
    let mut out = Vec::new();
    for seq in 0..count {
        let (_, dur) = pinger
            .ping(PingSequence(seq.into()), &[])
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        out.push(dur.as_millis() as u64);
    }
    Ok(out)
}
