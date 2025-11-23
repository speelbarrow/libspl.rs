use cucumber::{World, gherkin::Step, given, then, when};
use libspl::Interaction;
use std::{
    collections::VecDeque,
    error::Error,
    io::{self},
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};
use tokio::{
    io::{AsyncRead, AsyncWrite, AsyncWriteExt, DuplexStream, ReadBuf, duplex},
    join,
    time::sleep,
};

#[derive(Debug)]
struct TestInteraction(DuplexStream);
impl AsyncRead for TestInteraction {
    fn poll_read(
        mut self: Pin<&mut Self>,
        context: &mut Context<'_>,
        buffer: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_read(context, buffer)
    }
}
impl AsyncWrite for TestInteraction {
    fn poll_write(
        mut self: Pin<&mut Self>,
        context: &mut Context<'_>,
        buffer: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        Pin::new(&mut self.0).poll_write(context, buffer)
    }

    fn poll_flush(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_flush(context)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_shutdown(context)
    }
}
impl Interaction for TestInteraction {
    const TIMEOUT: Duration = Duration::from_millis(50);

    async fn close(mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.shutdown().await?;
        Ok(())
    }
}

#[derive(Debug, World)]
struct InteractionWorld {
    buffer: VecDeque<(String, Option<Duration>)>,
    chunk: Option<String>,
    duplex: DuplexStream,
    interaction: TestInteraction,
}
impl Default for InteractionWorld {
    fn default() -> Self {
        let (a, b) = duplex(2048);
        Self {
            buffer: Default::default(),
            chunk: Default::default(),
            duplex: a,
            interaction: TestInteraction(b),
        }
    }
}

#[given(expr = "an Interaction with a {int} millisecond timeout")]
fn given_an_interaction_with_a_millisecond_timeout(
    InteractionWorld { buffer, .. }: &mut InteractionWorld,
    millis: u128,
    step: &Step,
) {
    assert_eq!(millis, TestInteraction::TIMEOUT.as_millis());
    if let Some(table) = &step.table {
        assert_eq!(2, table.row_width());
        for row in &table.rows {
            if let [string, wait] = &row[..] {
                buffer.push_back((
                    string.clone(),
                    if !wait.is_empty() {
                        Some(TestInteraction::TIMEOUT.mul_f64(wait.parse::<f64>().expect("f64")))
                    } else {
                        None
                    },
                ))
            } else {
                unreachable!("expected exactly two values, got: {:#?}", row);
            }
        }
    }
}

#[when(regex = "I read (a|the last) chunk")]
async fn when_i_read_chunk(
    InteractionWorld {
        buffer,
        chunk,
        duplex,
        interaction,
        ..
    }: &mut InteractionWorld,
    function: String,
) {
    join!(
        async {
            while let Some((string, wait)) = buffer.pop_front() {
                duplex.write(string.as_bytes()).await.expect("write");
                if let Some(wait) = wait {
                    sleep(wait).await;
                }
            }
        },
        async {
            *chunk = Some(
                match function.as_str() {
                    "a" => interaction.read_chunk().await,
                    "the last" => interaction
                        .read_last_chunk()
                        .await
                        .map_err(|error| -> Box<dyn Error + Send + Sync> { Box::new(error) }),
                    _ => unreachable!(),
                }
                .expect("chunk"),
            );
        },
    );
}

#[then(expr = "the chunk I read should equal {string}")]
fn then_the_chunk_i_read_should_equal(
    InteractionWorld { chunk, .. }: &mut InteractionWorld,
    string: String,
) {
    assert_eq!(&string, chunk.as_ref().expect("chunk hasn't been read yet"))
}

#[tokio::main]
async fn main() {
    InteractionWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit("./features/interaction.feature")
        .await
}
