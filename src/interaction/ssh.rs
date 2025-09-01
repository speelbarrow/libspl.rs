#![cfg(feature = "ssh")]

use super::{Interaction, PID};
use openssh::{Child, Stdio};
pub use openssh::{KnownHosts, Session};
use std::{
    error::Error,
    io,
    path::PathBuf,
    pin::Pin,
    process::Output,
    str::FromStr,
    task::{Context, Poll},
    time::Duration,
};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

#[ouroboros::self_referencing]
pub struct SSH {
    session: Session,

    #[borrows(session)]
    #[covariant]
    pub process: Child<&'this Session>,

    name: String,
}
impl SSH {
    async fn is_linux(session: &Session) -> bool {
        match session.command("uname").output().await {
            Ok(Output { ref stdout, .. })
                if std::str::from_utf8(stdout).is_ok_and(|string| string.contains("Linux")) =>
            {
                true
            }
            _ => false,
        }
    }
}
impl AsyncRead for SSH {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        self.with_process_mut(|process| {
            Pin::new(process.stdout().as_mut().unwrap()).poll_read(cx, buf)
        })
    }
}
impl AsyncWrite for SSH {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        self.with_process_mut(|process| {
            Pin::new(process.stdin().as_mut().unwrap()).poll_write(cx, buf)
        })
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        self.with_process_mut(|process| Pin::new(process.stdin().as_mut().unwrap()).poll_flush(cx))
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), io::Error>> {
        self.with_process_mut(|process| {
            Pin::new(process.stdin().as_mut().unwrap()).poll_shutdown(cx)
        })
    }
}
impl Interaction for SSH {
    const TIMEOUT: Duration = Duration::from_millis(50);
    const REPEAT: usize = 3;

    async fn close(self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let pid = self.get_pid().await;
        let session = self.into_heads().session;
        if let Ok(pid) = pid {
            session
                .command("kill")
                .args(["-9", &pid.to_string()])
                .output()
                .await?;
        }
        session.close().await?;
        Ok(())
    }
}

/**
Creates a new SSH [session](Session) with the host at `url`. Then, launches `file` on the remote
host and returns an [`Interaction`] connected to that remote process.

Before launching `file`, this function will attempt to run `uname` on the remote system to detect if
it is running Linux. If so, `file` with be run with the command prefix `"stdbuf -o0 "` to avoid
Linux buffering/withholding remote program output.
*/
pub async fn interact(url: &str, file: &'static str) -> Result<SSH, Box<dyn Error + Send + Sync>> {
    Ok(SSHAsyncSendTryBuilder {
        session: Session::connect_mux(url, KnownHosts::Strict).await?,
        process_builder: |session: &Session| {
            Box::pin(async move {
                let mut shell = String::from(file);
                if SSH::is_linux(session).await {
                    shell.insert_str(0, "stdbuf -o0 sh -c '\n");
                    shell += "\n'";
                }
                session
                    .shell(shell)
                    .stdout(Stdio::piped())
                    .stdin(Stdio::piped())
                    .spawn()
                    .await
            })
        },
        name: {
            if let Some(name) = PathBuf::from_str(
                file.trim_matches('\'')
                    .trim()
                    .replace("\n", ";")
                    .replace("\\;", ";")
                    .split(';')
                    .last()
                    .unwrap()
                    .trim()
                    .trim_start_matches(['.', '/'])
                    .split(" ")
                    .next()
                    .unwrap(),
            )?
            .file_name()
            {
                name.to_string_lossy().to_string()
            } else {
                return Err(Box::new(io::Error::from(io::ErrorKind::InvalidFilename)));
            }
        },
    }
    .try_build()
    .await?)
}

impl PID for SSH {
    async fn get_pid(&self) -> Result<u32, Box<dyn Error + Send + Sync>> {
        if Self::is_linux(self.borrow_session()).await {
            let grepout = String::from_utf8(
                self.borrow_session()
                    .command("pgrep")
                    .arg(self.borrow_name())
                    .output()
                    .await
                    .unwrap()
                    .stdout,
            )
            .unwrap();
            if let Some(pid) = {
                let mut ids = grepout.split(|b| b == '\n').collect::<Vec<_>>();
                ids.pop();
                ids.pop()
            } {
                return Ok(pid.parse()?);
            }
        }
        Err(Box::new(io::Error::from(io::ErrorKind::NotFound)))
    }
}
