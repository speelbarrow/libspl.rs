#![cfg(feature = "tcp")]

use super::Interaction;
use std::time::Duration;
use tokio::net::TcpStream;

impl Interaction for TcpStream {
    const TIMEOUT: Duration = Duration::from_millis(50);
    const REPEAT: usize = 5;
}

/// Open a TCP [interaction](Interaction) using [tokio](tokio::net::TcpStream).
pub async fn connect(url: &'static str) -> std::io::Result<TcpStream> {
    Ok(TcpStream::connect(url).await?)
}
