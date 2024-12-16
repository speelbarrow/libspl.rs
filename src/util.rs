use std::{
    fmt::LowerHex,
    num::ParseIntError,
    ops::{BitOrAssign, ShlAssign},
    string::FromUtf8Error,
};

#[derive(Debug)]
pub enum HexToBytesError {
    ParseError(ParseIntError),
    UTF8Error(FromUtf8Error),
}
impl From<ParseIntError> for HexToBytesError {
    fn from(error: ParseIntError) -> Self {
        Self::ParseError(error)
    }
}
impl From<FromUtf8Error> for HexToBytesError {
    fn from(error: FromUtf8Error) -> Self {
        Self::UTF8Error(error)
    }
}

/**
Parses a number into a [byte](u8) vector where each byte holds the value of a hex-pair from the
input.
*/
#[trait_variant::make(Send)]
pub trait HexToBytes: LowerHex + Sync {
    async fn hex_to_bytes(&self) -> Vec<u8> {
        async move {
            let s = {
                let mut r = format!("{:x}", self);
                if r.len() % 2 == 1 {
                    r = "0".to_owned() + &r
                }
                r
            };

            let mut r = Vec::new();
            for chunk in s.as_bytes().chunks(2) {
                r.push(u8::from_str_radix(&String::from_utf8(chunk.to_vec()).unwrap(), 16).unwrap())
            }
            r
        }
    }
}
impl<T: ?Sized + Send + Sync + LowerHex> HexToBytes for T {}

/// Describes which side of an array should be padded/truncated. See [`Pad`].
pub enum Side {
    Left,
    Right,
}

/// Make a u8 iterable into one of a specific length, padding as needed.
#[trait_variant::make(Send)]
pub trait Pad: Sized + Sync + IntoIterator<Item = u8>
where
    <Self as IntoIterator>::IntoIter: DoubleEndedIterator,
{
    /// Shorthand to call [`pad_with`](Pad::pad_with) with `with = 0`.
    async fn pad<const FINAL: usize>(self, side: Side) -> [u8; FINAL] {
        self.pad_with(side, 0)
    }

    /// Shorthand to call [`pad_both_with`](Pad::pad_both_with) with `with = 0`.
    async fn pad_both<const L: usize, const R: usize>(self) -> [u8; R] {
        self.pad_both_with::<L, R>(0)
    }

    /**
    Shorthand to call [`pad_with`](Pad::pad_with) twice, using the first constant template argument
    when padding the left side, and the second constant template argument when padding the right
    side.

    In other words:
    - The first template argument should be equal to the desired length of the array after padding
      ONLY the left side.
    - The second template argument should be equal to the desired length of the array after the
      WHOLE operation is complete (i.e. the length of the output).
    */
    async fn pad_both_with<const L: usize, const R: usize>(self, with: u8) -> [u8; R] {
        async move {
            return self
                .pad_with::<L>(Side::Left, with)
                .await
                .pad_with::<R>(Side::Right, with)
                .await;
        }
    }

    /**
    When `self.len` <= FINAL: Adds zeroes to the side of `self` until `length = FINAL`.

    When `self.len` > FINAL: Removes elements from the side of `self` until `length = FINAL`.

    Consumes the input and outputs a new [u8] array padded with `with`s.
    */
    async fn pad_with<const FINAL: usize>(self, side: Side, with: u8) -> [u8; FINAL] {
        async move {
            let mut r: [u8; FINAL] = [0; FINAL];
            let mut iterator: Box<dyn DoubleEndedIterator<Item = u8>> = Box::new(self.into_iter());

            for index in {
                let mut r = (0..FINAL).collect::<Vec<_>>();
                if let Side::Left = side {
                    r.reverse();
                    iterator = Box::new(iterator.rev());
                }
                r
            } {
                if let Some(byte) = iterator.next() {
                    r[index] = byte;
                } else {
                    r[index] = with;
                }
            }

            r
        }
    }
}
impl<T> Pad for T
where
    T: Sync + Send + IntoIterator<Item = u8>,
    <T as IntoIterator>::IntoIter: DoubleEndedIterator,
{
}

/// See the [method documentation](Repeat::from_repeated).
#[trait_variant::make(Send)]
pub trait Repeat<From>: Sized
where
    Self: Copy + core::convert::From<From> + BitOrAssign + ShlAssign<usize>,
    From: Send + Sync,
{
    /**
    Creates a new integer value by repeating a smaller integer value to fill the expanded bits.

    For example:
    ```
    use libspl::util::Repeat;

    # #[tokio::main]
    # async fn main() {
    let byte = 0x7fu8;
    let (word, dword, qword) = tokio::join!(
        u16::from_repeated(byte),
        i32::from_repeated(byte as i8),
        u64::from_repeated(byte),
    );
    assert_eq!(word, 0x7f7f);
    assert_eq!(dword, 0x7f7f7f7f);
    assert_eq!(qword, 0x7f7f7f7f7f7f7f7f);
    # }
    ```
    */
    async fn from_repeated(from: From) -> Self {
        async {
            dbg!(size_of::<Self>());
            dbg!(size_of::<From>());
            let mut v = from.into();
            let mut r = v;

            for _ in 1..(size_of::<Self>() / size_of::<From>()) {
                v <<= 8 * size_of::<From>();
                r |= v;
            }

            r
        }
    }
}
impl<F, T> Repeat<F> for T
where
    T: Send + Copy + From<F> + BitOrAssign + ShlAssign<usize>,
    F: Send + Sync,
{
}

#[cfg(test)]
mod tests {
    use tokio::{join, test};

    use super::{HexToBytes, Pad, Repeat, Side};
    use rand::{thread_rng, Rng};

    #[test]
    async fn left_padded() {
        let mut rng = thread_rng();
        let (a, b, c, d) = (
            rng.gen::<u8>(),
            rng.gen::<u8>(),
            rng.gen::<u8>(),
            rng.gen::<u8>(),
        );
        let string = [a, b, c, d];

        assert_eq!(
            [&[0u8; 28], &string as &[u8]].concat(),
            string.pad::<32>(Side::Left).await.to_vec()
        );
        assert_eq!(
            [&[b'1'; 28], &string as &[u8]].concat(),
            string.pad_with::<32>(Side::Left, b'1').await.to_vec(),
        );
    }

    #[test]
    async fn hexbytes() {
        assert_eq!(
            0x10203040u32.hex_to_bytes().await,
            &[0x10u8, 0x20u8, 0x30u8, 0x40u8]
        );
    }

    #[test]
    async fn right_padded_hexbytes() {
        assert_eq!(
            &(0x12030.hex_to_bytes().await.pad::<4>(Side::Right).await) as &[u8],
            &[0x01, 0x20, 0x30, 0]
        );
    }

    #[test]
    async fn pad_both() {
        let mut rng = thread_rng();
        let (a, b, c, d) = (
            rng.gen::<u8>(),
            rng.gen::<u8>(),
            rng.gen::<u8>(),
            rng.gen::<u8>(),
        );
        let string = [a, b, c, d];
        assert_eq!(
            string.pad_both::<8, 16>().await,
            [0, 0, 0, 0, a, b, c, d, 0, 0, 0, 0, 0, 0, 0, 0]
        );

        assert_eq!(string.pad_both_with::<16, 8>(b'1').await, *b"11111111");
    }

    #[test]
    async fn repeat_random() {
        let rand = thread_rng().gen::<u8>();
        let (expected, actual) = join!(
            async {
                let mut r = rand as u64;
                for exponent in (2..=14).step_by(2) {
                    r += rand as u64 * 0x10u64.pow(exponent);
                }
                r
            },
            u64::from_repeated(rand)
        );
        assert_eq!(expected, actual);
    }
}
