#![cfg(feature = "tcp")]

use super::Interaction;
use std::time::Duration;
use tokio::{io::AsyncWriteExt, net::TcpStream};

impl Interaction for TcpStream {
    const TIMEOUT: Duration = Duration::from_millis(50);
    const REPEAT: usize = 5;

    async fn close(mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.shutdown().await?)
    }
}

/// Open a TCP [interaction](Interaction) using [tokio](tokio::net::TcpStream).
pub async fn interact(url: &'static str) -> std::io::Result<TcpStream> {
    Ok(TcpStream::connect(url).await?)
}
