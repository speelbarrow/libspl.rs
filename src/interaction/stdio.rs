#![cfg(feature = "stdio")]

use super::{Interaction, PID};
use std::{
    error::Error,
    ffi::OsStr,
    io,
    ops::{Deref, DerefMut},
    path::Path,
    pin::Pin,
    process,
    task::{Context, Poll},
    time::Duration,
};
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    process::{Child, Command},
};

pub struct Stdio(Child);
impl Deref for Stdio {
    type Target = Child;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Stdio {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl AsyncRead for Stdio {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new((&mut **self).stdout.as_mut().unwrap()).poll_read(cx, buf)
    }
}
impl AsyncWrite for Stdio {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        Pin::new((&mut **self).stdin.as_mut().unwrap()).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        Pin::new((&mut **self).stdin.as_mut().unwrap()).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), io::Error>> {
        Pin::new((&mut **self).stdin.as_mut().unwrap()).poll_shutdown(cx)
    }
}
impl Interaction for Stdio {
    const TIMEOUT: Duration = Duration::ZERO;
}

/// Launch a [child process](tokio::process::Child) for interaction.
pub async fn interact<I>(
    path: impl AsRef<Path>,
    arguments: Option<I>,
) -> Result<Stdio, Box<dyn Error + Send + Sync>>
where
    I: IntoIterator,
    <I as IntoIterator>::Item: AsRef<OsStr>,
{
    let mut command = Command::new(path.as_ref());
    command
        .stderr(process::Stdio::inherit())
        .stdin(process::Stdio::piped())
        .stdout(process::Stdio::piped());
    if let Some(arguments) = arguments {
        command.args(arguments);
    }
    Ok(Stdio(command.spawn()?))
}

impl PID for Stdio {
    async fn get_pid(&self) -> Result<u32, Box<dyn Error + Send + Sync>> {
        if let Some(pid) = self.0.id() {
            Ok(pid)
        } else {
            Err(Box::new(io::Error::from(io::ErrorKind::NotFound)))
        }
    }
}
