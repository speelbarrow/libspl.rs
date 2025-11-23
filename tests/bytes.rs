use cucumber::{Parameter, World, gherkin::Step, given, then, when};
use derive_more::{Deref, DerefMut};
use libspl::{HexToBytes, Pad, Repeat, Side};
use num_traits::PrimInt;
use std::{fmt::Debug, num::IntErrorKind, str::FromStr};

#[derive(Debug, World)]
struct BytesWorld {
    bytes: Vec<u8>,
    hex: Option<u32>,
    integer: Option<u64>,
}
impl BytesWorld {
    const FINAL: usize = 32;
}
impl Default for BytesWorld {
    fn default() -> Self {
        Self {
            bytes: Vec::new(),
            hex: None,
            integer: None,
        }
    }
}

#[derive(Debug, Default, Deref, DerefMut, Parameter)]
#[param(name = "hex", regex = "0x([0-9A-Fa-f]+)")]
struct Hex<T: PrimInt>(T);
impl<T: PrimInt> FromStr for Hex<T> {
    type Err = T::FromStrRadixErr;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        Ok(Self(T::from_str_radix(string, 16)?))
    }
}

#[given(expr = "a final padded byte-string length of {int}")]
fn given_a_final_padded_byte_string_length_of(_: &mut BytesWorld, expected: usize) {
    assert_eq!(expected, BytesWorld::FINAL);
}

#[given(expr = "the byte-string {string}")]
fn given_the_byte_string(BytesWorld { bytes, .. }: &mut BytesWorld, string: String) {
    assert!(string.len() < BytesWorld::FINAL);
    *bytes = string.into_bytes();
}

#[given(expr = "the unsigned 32-bit hex value {hex}")]
fn the_unsigned_bit_hex_value_x(world: &mut BytesWorld, hex: Hex<u32>) {
    world.hex = Some(*hex)
}

#[when(expr = "I convert the unsigned 32-bit hex value to bytes")]
async fn i_convert_the_unsigned_bit_hex_value_to_bytes(
    BytesWorld { bytes, hex, .. }: &mut BytesWorld,
) {
    *bytes = hex.expect("hex").hex_to_bytes().await
}

#[when(regex = "I pad(?: the (left|right) side of)? the byte-string(?: with '(.)'s)?")]
async fn when_i_pad_the_byte_string(
    BytesWorld { bytes, .. }: &mut BytesWorld,
    side: String,
    with: String,
) {
    assert!(with.bytes().count() <= 1);
    let (clone, side) = (
        bytes.clone(),
        match side.as_str() {
            "left" => Some(Side::Left),
            "right" => Some(Side::Right),
            "" => None,
            string => unreachable!("expected '', 'left' or 'right', got '{}'", string),
        },
    );
    *bytes = match (side, with.as_bytes().get(0)) {
        (Some(side), Some(&byte)) => clone.pad_with::<{ BytesWorld::FINAL }>(side, byte).await,
        (Some(side), None) => clone.pad(side).await,
        (None, Some(&byte)) => {
            clone
                .pad_both_with::<{ BytesWorld::FINAL / 2 }, { BytesWorld::FINAL }>(byte)
                .await
        }
        (None, None) => {
            clone
                .pad_both::<{ BytesWorld::FINAL / 2 }, { BytesWorld::FINAL }>()
                .await
        }
    }
    .to_vec();
}

#[when(expr = "I repeat the first byte into an unsigned 64-bit integer")]
async fn i_repeat_the_first_byte_into_an_unsigned_64_bit_integer(
    BytesWorld { bytes, integer, .. }: &mut BytesWorld,
) {
    *integer = Some(u64::from_repeated(bytes[0]).await);
}

#[then("the sequence of the byte-string will be")]
fn then_the_sequence_of_the_byte_string_will_be(
    BytesWorld { bytes, .. }: &mut BytesWorld,
    step: &Step,
) {
    let [expected, actual] = [
        {
            let mut r = Vec::<u8>::new();
            for row in &step.table.as_ref().expect("table").rows {
                if let [byte, count] = row.as_slice() {
                    let byte = match byte.trim().as_bytes() {
                        [] => 0,
                        &[byte] => byte,
                        bytes => {
                            let trimmed = std::str::from_utf8(bytes)
                                .expect("`&bytes` is valid UTF-8")
                                .trim_start_matches("0x");
                            u8::from_str_radix(
                                trimmed,
                                if trimmed.len() != bytes.len() { 16 } else { 10 },
                            )
                            .unwrap()
                        }
                    };
                    r.append(&mut vec![
                        byte;
                        {
                            let trimmed = count.trim();
                            if trimmed.is_empty() {
                                1
                            } else {
                                match usize::from_str_radix(trimmed, 10) {
                                    Ok(ok) => ok,
                                    Err(error) if matches!(error.kind(), IntErrorKind::Empty) => {
                                        todo!()
                                    }
                                    otherwise => otherwise.expect("otherwise"),
                                }
                            }
                        }
                    ])
                } else {
                    panic!("expected 2 columns, found {}", row.len())
                }
            }
            r
        },
        bytes.clone(),
    ]
    .map(|bytes| String::from_utf8(bytes).expect("`bytes` is valid UTF-8"));
    assert_eq!(expected, actual);
}

#[then("I should have the bytes")]
fn i_should_have_the_bytes(BytesWorld { bytes, .. }: &mut BytesWorld, step: &Step) {
    let rows = &step.table.as_ref().expect("table").rows;
    assert_eq!(rows.len(), bytes.len());
    for (&actual, row) in bytes.iter().zip(rows) {
        let cell = &row[0];
        let expected = if cell.is_empty() {
            Ok(0)
        } else if cell.starts_with("0x") {
            u8::from_str_radix(cell.trim_start_matches("0x"), 16)
        } else {
            u8::from_str_radix(cell, 10)
        }
        .unwrap();
        assert_eq!(expected, actual);
    }
}

#[then(expr = "the unsigned 64-bit integer should equal {hex}")]
fn the_unsigned_64_bit_integer_should_equal(
    BytesWorld { integer, .. }: &mut BytesWorld,
    hex: Hex<u64>,
) {
    assert_eq!(*hex, integer.expect("integer"))
}

#[tokio::main]
async fn main() {
    BytesWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit("features/bytes.feature")
        .await
}
