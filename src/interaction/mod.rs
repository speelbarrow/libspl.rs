#![cfg(feature = "interaction")]

use std::{error::Error, future::Future, string::FromUtf8Error, time::Duration};
use tokio::{
    io::{
        AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, Error as IOError, copy, stdin,
        stdout,
    },
    join,
    sync::mpsc::{UnboundedReceiver, unbounded_channel},
    time::timeout,
};

pub mod ssh;
pub mod stdio;
pub mod tcp;

/// A read-write stream that reacts to input.
#[trait_variant::make(Send)]
pub trait Interaction: AsyncReadExt + AsyncWriteExt + Unpin + Sized {
    const TIMEOUT: Duration;
    const REPEAT: usize = 1;

    /// Reads the last chunk. See [`read_chunk`](Interaction::read_chunk)
    async fn read_last_chunk(&mut self) -> Result<String, FromUtf8Error> {
        async {
            let mut buf = Vec::new();
            let mut dropped = vec![false; Self::REPEAT];
            'a: loop {
                match timeout(Self::TIMEOUT, self.read_u8()).await {
                    Ok(Ok(b)) => {
                        dropped = vec![false; Self::REPEAT];
                        buf.push(b);
                    }
                    _ => {
                        for i in 0..Self::REPEAT {
                            if !dropped[i] {
                                dropped[i] = true;
                                continue 'a;
                            }
                        }
                        return Ok(String::from_utf8(buf)?);
                    }
                }
            }
        }
    }

    /**
    Reads one "chunk" of remote input. A chunk "ends" when no new data is received for
    [`TIMEOUT`](Interaction::TIMEOUT) amount of time. This does not apply to the first byte read --
    the function will wait indefinitely until it receives *some* data from the remote stream.
    */
    async fn read_chunk(&mut self) -> Result<String, FromUtf8Error> {
        async {
            Ok(String::from_utf8(vec![self.read_u8().await.unwrap()])?
                + &self.read_last_chunk().await?)
        }
    }

    /**
    Like [`run`](Interaction::run), but forwards input received from the remote process over an
    [unbounded channel](tokio::sync::mpsc::unbounded_channel).
    */
    fn run_with_channel<'a, I>(
        &mut self,
        input: I,
    ) -> (
        UnboundedReceiver<String>,
        impl Future<Output = Result<(), Box<dyn Error + Send + Sync>>> + Send,
    )
    where
        I: IntoIterator<Item = &'a [u8]> + Send,
        <I as IntoIterator>::IntoIter: Send,
    {
        let (sender, receiver) = unbounded_channel();
        let future = async move {
            let mut stdout = stdout();
            for i in input {
                let chunk = self.read_chunk().await?;
                if !sender.is_closed() {
                    for part in chunk.split("\n") {
                        sender.send(part.to_owned())?;
                    }
                }
                stdout.write_all(chunk.as_bytes()).await?;
                let (r1, r2) = join!(self.write_all(i), async {
                    stdout.write_all(i).await?;
                    stdout.write_u8(b'\n').await?;
                    Ok::<(), IOError>(())
                });
                r1?;
                r2?;
            }

            let chunk = self.read_last_chunk().await?;
            for string in chunk.split("\n") {
                if !sender.is_closed() {
                    sender.send(string.to_owned()).unwrap();
                }
            }
            stdout.write_all(chunk.as_bytes()).await?;

            copy(self, &mut stdout).await?;
            Ok(())
        };
        (receiver, future)
    }

    /**
    Executes a series of transactions as such:
    1. Wait for data from the remote stream (see [`read_chunk`](Interaction::read_chunk))
    2. Pops one `&[u8]` from the top of `input` and writes it to the remote stream.
    3. Repeat.
    */
    async fn run<'a, I>(&mut self, input: I) -> Result<(), Box<dyn Error + Send + Sync>>
    where
        I: IntoIterator<Item = &'a [u8]> + Send,
        <I as IntoIterator>::IntoIter: Send,
    {
        self.run_with_channel(input).1
    }
}

/**
An [Interaction] that can retrieve the PID of the underlying process.
```
use libspl::{interact, PID};

# use std::error::Error;
# #[tokio::main]
# async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
interact!(stdio, "ls").await?.leak_pid().await;
# Ok(())
# }
```
*/
#[trait_variant::make(Send)]
pub trait PID: Interaction + Send + Sync {
    async fn get_pid(&self) -> Result<u32, Box<dyn Error + Send + Sync>>;

    /**
    Pauses the [Interaction], writes the [PID](PID::get_pid) to the console, and waits for user
    confirmation.

    See the [trait documentation](PID).
    */
    async fn leak_pid(&self) -> &Self {
        async move {
            match self.get_pid().await {
                Ok(pid) => println!("PID is {}", pid),
                Err(error) => println!("Failed to retrieve PID with error: {}", error),
            }

            /*
            Using `print` here causes the line not to be shown until after ENTER is pressed, even if
            `stdout.flush` is called afterwards.
            */
            let mut stdout = stdout();
            stdout.write(b"[Press ENTER to continue]").await.unwrap();
            stdout.flush().await.unwrap();

            BufReader::new(stdin())
                .read_line(&mut String::new())
                .await
                .unwrap();
            self
        }
    }
}

/**
Shorthand for creating new [Interaction]s.

Supported recipes:
- [`ssh`]
  ```no_run
  use libspl::interact;

  # #[tokio::main]
  # async fn main() {
  let _ = interact!(ssh, "www.example.com", "/path/to/executable").await.unwrap();
  # }
  ```
- [`stdio`]
  ```no_run
  use libspl::interact;

  # #[tokio::main]
  # async fn main() {
  let _ = interact!(stdio, "/path/to/executable").await.unwrap();
  # }
  ```
  ```no_run
  use libspl::interact;

  # #[tokio::main]
  # async fn main() {
  let _ = interact!(stdio, "/path/to/executable", "--arg1", "--arg2").await.unwrap();
  # }
  ```
- [`tcp`]
  ```no_run
  use libspl::interact;

  # #[tokio::main]
  # async fn main() {
  let _ = interact!(tcp, "www.example.com:65535").await.unwrap();
  # }
  ```
*/
#[macro_export]
macro_rules! interact {
    (stdio, $path: expr) => {
        ::libspl::interaction::stdio::interact::<[&str; 0]>($path, None)
    };
    (stdio, $path: expr$(, $argument: expr)+) => {
        interact!(@internal stdio, $path, Some([$( $argument ),+]))
    };
    ($method: ident$(, $argument: expr )*) => {
        interact!(@internal $method$(, $argument)*)
    };
    (@internal $method: ident$(, $argument: expr )*) => {
        ::libspl::interaction::$method::interact($( $argument ),*)
    };
}
