#![cfg(feature = "ssh")]

use super::Interaction;
use openssh::{Child, Error as SSHError, Stdio};
pub use openssh::{KnownHosts, Session};
use std::{
    io::{Error as IOError, Result as IOResult},
    path::PathBuf,
    pin::Pin,
    str::FromStr,
    time::Duration,
};
use tokio::io::{stdin, AsyncBufReadExt, AsyncRead, AsyncWrite, BufReader};

#[ouroboros::self_referencing]
pub struct SSH {
    session: Session,

    #[borrows(session)]
    #[covariant]
    pub process: Child<&'this Session>,

    name: String,
}

impl SSH {
    /// See [`connect_leak`](connect_leak).
    async fn leak_pid(&self) {
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
        if let Some(v) = {
            let mut ids = grepout.split(|b| b == '\n').collect::<Vec<_>>();
            ids.pop();
            ids.pop()
        } {
            println!("PID is {}. Waiting . . .", v);
            println!("[Press ENTER to continue]");

            BufReader::new(stdin())
                .read_line(&mut String::new())
                .await
                .unwrap();
        }
    }
}

/**
Creates a new SSH [session](Session) with the host at `url`. Then, launches `file` on the remote
host and returns an [`Interaction`] connected to that remote process.
*/
pub async fn connect(url: &str, file: &'static str) -> Result<SSH, SSHError> {
    Ok(SSHAsyncSendTryBuilder {
        session: Session::connect_mux(url, KnownHosts::Strict).await?,
        process_builder: |session: &Session| {
            Box::pin(async move {
                session
                    .shell("stdbuf -o0 ".to_owned() + file)
                    .stdout(Stdio::piped())
                    .stdin(Stdio::piped())
                    .spawn()
                    .await
            })
        },
        name: PathBuf::from_str(file)
            .unwrap()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned(),
    }
    .try_build()
    .await?)
}

/**
Like [`connect`](connect), but pauses the process as soon as it launches. Then, uses
`pgrep` on the remote host to find the process's PID and reports it back to you, and then waits
for the user to press ENTER.
*/
pub async fn connect_leak(url: &str, file: &'static str) -> Result<SSH, SSHError> {
    let r = connect(url, file).await?;
    r.leak_pid().await;
    Ok(r)
}

impl AsyncWrite for SSH {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, IOError>> {
        self.with_process_mut(|process| {
            Pin::new(process.stdin().as_mut().unwrap()).poll_write(cx, buf)
        })
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), IOError>> {
        self.with_process_mut(|process| Pin::new(process.stdin().as_mut().unwrap()).poll_flush(cx))
    }

    fn poll_shutdown(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), IOError>> {
        self.with_process_mut(|process| {
            Pin::new(process.stdin().as_mut().unwrap()).poll_shutdown(cx)
        })
    }
}

impl AsyncRead for SSH {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<IOResult<()>> {
        self.with_process_mut(|process| {
            Pin::new(process.stdout().as_mut().unwrap()).poll_read(cx, buf)
        })
    }
}

impl Interaction for SSH {
    const TIMEOUT: Duration = Duration::from_millis(50);
    const REPEAT: usize = 3;
}
