use bytecount::num_chars;
pub const BYTES_PER_CHAR: usize = 4;
const VALID_ONE_BYTE_UTF8_FIRST_BYTE_MAX_VALUE: u8 = 0b01111111_u8;

const VALID_TWO_BYTE_UTF8_FIRST_BYTE_RANGE: (u8, u8) = (0b11000000_u8, 0b11011111_u8);
const VALID_TWO_BYTE_UTF8_SECOND_BYTE_RANGE: (u8, u8) = (0b10000000_u8, 0b10111111_u8);

const VALID_THREE_BYTE_UTF8_FIRST_BYTE_RANGE: (u8, u8) = (0b11100000_u8, 0b11110111_u8);
const VALID_THREE_BYTE_UTF8_SECOND_BYTE_RANGE: (u8, u8) = (0b10000000_u8, 0b10111111_u8);
const VALID_THREE_BYTE_UTF8_THIRD_BYTE_RANGE: (u8, u8) = (0b10000000_u8, 0b10111111_u8);

const VALID_FOUR_BYTE_UTF8_FIRST_BYTE_RANGE: (u8, u8) = (0b11110000_u8, 0b11110111_u8);
const VALID_FOUR_BYTE_UTF8_SECOND_BYTE_RANGE: (u8, u8) = (0b10000000_u8, 0b10111111_u8);
const VALID_FOUR_BYTE_UTF8_THIRD_BYTE_RANGE: (u8, u8) = (0b10000000_u8, 0b10111111_u8);
const VALID_FOUR_BYTE_UTF8_FOURTH_BYTE_RANGE: (u8, u8) = (0b10000000_u8, 0b10111111_u8);
const SPACE_BYTE: &[u8] = &[0, 0, 0, 32];
type PreparedCustomBytes = (Option<u8>, Option<u8>, Option<u8>, Option<u8>);
use std::{
    borrow::Borrow,
    error::{self, Error},
    fmt::Display,
};

pub type ValidUTF8BytesVec = Vec<u8>;
pub type CustomStringBytesVec = Vec<u8>;
pub type ValidUTF8BytesSlice = [u8];
pub type CustomStringBytesSlice = [u8];

fn is_in_range<T: PartialEq + PartialOrd>(value: T, range: (T, T)) -> bool {
    value >= range.0 && value <= range.1
}
#[derive(Debug, Clone)]
enum InvalidCustomStringErrorType {
    InvalidLength(usize),
    InvalidFormat,
}
#[derive(Debug, Clone)]
struct InvalidCustomStringByteError {
    error_type: InvalidCustomStringErrorType,
    invalid_sequence: Option<Vec<u8>>,
}
impl Display for InvalidCustomStringByteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.error_type {
            InvalidCustomStringErrorType::InvalidFormat => {
                write!(
                    f,
                    "Invalid custom bytes: {:?}",
                    self.invalid_sequence.as_ref().unwrap()
                )
            }
            InvalidCustomStringErrorType::InvalidLength(length) => {
                write!(f, "Invalid bytes length: {}", length)
            }
        }
    }
}
impl InvalidCustomStringByteError {
    pub fn new_invalid_length(invalid_data: &[u8]) -> Self {
        Self {
            error_type: InvalidCustomStringErrorType::InvalidLength(invalid_data.len()),
            invalid_sequence: None,
        }
    }
    pub fn new_invalid_format(invalid_data: &[u8]) -> Self {
        Self {
            error_type: InvalidCustomStringErrorType::InvalidFormat,
            invalid_sequence: Some(invalid_data.into()),
        }
    }
}
impl Error for InvalidCustomStringByteError {}

/** returns bytes index */
pub fn rfind_space(custom_text: &CustomStringBytesSlice) -> Option<usize> {
    assert_eq!(custom_text.len() % 4, 0);

    for index in (0..(custom_text.len() / BYTES_PER_CHAR)).rev() {
        if let SPACE_BYTE = &custom_text[(index) * BYTES_PER_CHAR..(index + 1) * BYTES_PER_CHAR] {
            return Some((index) * BYTES_PER_CHAR);
        }
    }
    None
}

/** return char index */
pub fn rfind_space_char_index(custom_text: &CustomStringBytesSlice) -> Option<usize> {
    assert_eq!(custom_text.len() % 4, 0);

    for index in (0..(custom_text.len() / BYTES_PER_CHAR)).rev() {
        if let SPACE_BYTE = &custom_text[(index) * BYTES_PER_CHAR..(index + 1) * BYTES_PER_CHAR] {
            return Some(index);
        }
    }
    None
}

/**
 bytes length = 32, char len = 8
 index 0..8 reverse
 7..=0
 28 ..
*/
fn is_whitespace(custom_bytes: &CustomStringBytesSlice) -> bool {
    matches!(custom_bytes, [0, 0, 0, 9]
        | [0, 0, 0, 10]
        | [0, 0, 0, 11]
        | [0, 0, 0, 12]
        | [0, 0, 0, 13]
        | [0, 0, 0, 32]
        | [0, 0, 194, 133]
        | [0, 0xe2, 0x80, 0x8e]
        | [0, 0xe2, 0x80, 0x8f]
        | [0, 0xe2, 0x80, 0xa8]
        | [0, 0xe2, 0x80, 0xa9])
}

fn to_four_bytes(input: &str) -> CustomStringBytesVec {
    let output_size = num_chars(input.as_bytes());
    let mut output_vec: Vec<u8> = Vec::with_capacity(output_size * 2);
    // let mut output:&[u8;4] = &[0;output_size];
    for character in input.chars() {
        let mut bytes_buffer: [u8; 4] = [0; 4];

        character.encode_utf8(&mut bytes_buffer);
        // not leading zero yet
        let arranged_buffer = match bytes_buffer {
            [a, 0, 0, 0] => [0, 0, 0, a],
            [a, b, 0, 0] => [0, 0, a, b],
            [a, b, c, 0] => [0, a, b, c],
            _ => bytes_buffer,
        };
        // let vec_of_bytes = Vec::with_capacity(4);

        output_vec.extend_from_slice(&arranged_buffer);
    }
    output_vec
}

fn trim_to_std_utf8(
    input: &CustomStringBytesSlice,
) -> Result<PreparedCustomBytes, Box<dyn error::Error>> {
    if input.len() % 4 != 0 {
        Err(InvalidCustomStringByteError::new_invalid_length(input).into())
    } else {
        match input {
            [0, 0, 0, one_byte_char]
                if one_byte_char <= &VALID_ONE_BYTE_UTF8_FIRST_BYTE_MAX_VALUE =>
            {
                Ok((None, None, None, Some(*one_byte_char)))
            }
            [0, 0, first_byte, second_byte]
                if is_in_range(*first_byte, VALID_TWO_BYTE_UTF8_FIRST_BYTE_RANGE)
                    && is_in_range(*second_byte, VALID_TWO_BYTE_UTF8_SECOND_BYTE_RANGE) =>
            {
                Ok((None, None, Some(*first_byte), Some(*second_byte)))
            }
            [0, first_byte, second_byte, third_byte]
                if is_in_range(*first_byte, VALID_THREE_BYTE_UTF8_FIRST_BYTE_RANGE)
                    && is_in_range(*second_byte, VALID_THREE_BYTE_UTF8_SECOND_BYTE_RANGE)
                    && is_in_range(*third_byte, VALID_THREE_BYTE_UTF8_THIRD_BYTE_RANGE) =>
            {
                Ok((
                    None,
                    Some(*first_byte),
                    Some(*second_byte),
                    Some(*third_byte),
                ))
            }
            [first_byte, second_byte, third_byte, fourth_byte]
                if is_in_range(*first_byte, VALID_FOUR_BYTE_UTF8_FIRST_BYTE_RANGE)
                    && is_in_range(*second_byte, VALID_FOUR_BYTE_UTF8_SECOND_BYTE_RANGE)
                    && is_in_range(*third_byte, VALID_FOUR_BYTE_UTF8_THIRD_BYTE_RANGE)
                    && is_in_range(*fourth_byte, VALID_FOUR_BYTE_UTF8_FOURTH_BYTE_RANGE) =>
            {
                Ok((
                    Some(*first_byte),
                    Some(*second_byte),
                    Some(*third_byte),
                    Some(*fourth_byte),
                ))
            }
            _ => Err(InvalidCustomStringByteError::new_invalid_format(input).into()),
        }
    }
}

/** The content inside this string is a vector of bytes - ALWAYS with length % 4 == 0

    Every character is a valid utf-8 encoded byte padded left with 0 to make every character takes 4 bytes.

    For example, Thai characters which use 3 bytes are represented by
    [0, valid_first_byte, valid_second_byte, valid_third_byte].

    ***Comparison***
    String "กข " is represented by
    \[224, 184, 129, 224, 184, 130, 32\]

    CustomString "กข " is represented by
    \[0, 224, 184, 129, 0, 224, 184, 130, 0, 0, 0, 32\]
*/

#[derive(Clone)]
pub struct CustomString {
    content: Vec<u8>,
    length: usize,
}

impl CustomString {
    pub fn from(four_byte_vec: ValidUTF8BytesVec) -> Self {
        let content = four_byte_vec;
        let length = content.len() / BYTES_PER_CHAR;
        Self { content, length }
    }
    pub fn new(base_string: &str) -> Self {
        let content = to_four_bytes(base_string);
        let length = content.len() / BYTES_PER_CHAR;
        Self { content, length }
    }
    pub fn substring_as_custom_bytes(&self, char_start: usize, char_end: usize) -> &[u8] {
        &self.content[(char_start * BYTES_PER_CHAR)..(char_end * BYTES_PER_CHAR)]
    }
    pub fn raw_content(&self) -> &[u8] {
        self.content.borrow()
    }
    /** Returns characters length */
    pub fn chars_len(&self) -> usize {
        self.length
    }
    pub fn is_empty(&self) -> bool {
        self.content.len() == 0
    }
    pub fn len(&self) -> usize {
        self.content.len()
    }
    pub fn trim(&self) -> Self {
        let mut new_content: &[u8] = &self.content;
        while is_whitespace(&new_content[0..BYTES_PER_CHAR]) {
            // trim left
            new_content = &new_content[BYTES_PER_CHAR..];
        }

        while is_whitespace(&new_content[(new_content.len() - BYTES_PER_CHAR)..]) {
            // trim left
            new_content = &new_content[..(new_content.len() - BYTES_PER_CHAR)];
            // new_content.drain((self.content.len()-BYTES_PER_CHAR)..(self.content.len()));
        }
        let length = new_content.len() / BYTES_PER_CHAR;

        Self {
            content: Vec::from(new_content),
            length,
        }
    }

    pub fn convert_raw_bytes_to_std_string(input: &[u8]) -> String {
        let mut output_content: Vec<u8> = Vec::with_capacity(input.len() / 100);
        for index in 0..input.chars_len() {
            let extracted_bytes =
                trim_to_std_utf8(&input.slice_by_char_indice(index, index + 1)).unwrap();
            match extracted_bytes {
                (None, None, None, Some(first_byte)) => {
                    output_content.push(first_byte);
                }
                (None, None, Some(first_byte), Some(second_byte)) => {
                    output_content.push(first_byte);
                    output_content.push(second_byte);
                }
                (None, Some(first_byte), Some(second_byte), Some(third_byte)) => {
                    output_content.push(first_byte);
                    output_content.push(second_byte);
                    output_content.push(third_byte);
                }
                (Some(first_byte), Some(second_byte), Some(third_byte), Some(fourth_byte)) => {
                    output_content.push(first_byte);
                    output_content.push(second_byte);
                    output_content.push(third_byte);
                    output_content.push(fourth_byte);
                }
                _ => panic!("error"),
            }
        }
        let output =
            unsafe { String::from(std::str::from_utf8_unchecked(output_content.as_slice())) };
        output
    }
    pub fn convert_raw_bytes_to_utf8_bytes(input: &[u8]) -> Vec<u8> {
        let mut output_content: Vec<u8> = Vec::with_capacity(input.len() / 100);
        for index in 0..input.chars_len() {
            let extracted_bytes =
                trim_to_std_utf8(&input.slice_by_char_indice(index, index + 1)).unwrap();
            match extracted_bytes {
                (None, None, None, Some(first_byte)) => {
                    output_content.push(first_byte);
                }
                (None, None, Some(first_byte), Some(second_byte)) => {
                    output_content.push(first_byte);
                    output_content.push(second_byte);
                }
                (Some(first_byte), Some(second_byte), Some(third_byte), None) => {
                    output_content.push(first_byte);
                    output_content.push(second_byte);
                    output_content.push(third_byte);
                }
                (Some(first_byte), Some(second_byte), Some(third_byte), Some(fourth_byte)) => {
                    output_content.push(first_byte);
                    output_content.push(second_byte);
                    output_content.push(third_byte);
                    output_content.push(fourth_byte);
                }
                _ => panic!("error"),
            }
        }
        output_content.shrink_to_fit();
        output_content
    }
}

pub trait FixedCharsLengthByteSlice {
    fn slice_by_char_indice(&self, start: usize, end: usize) -> Self;
    fn chars_len(&self) -> usize;
    fn is_valid_custom_str_bytes(&self) -> bool;
}

impl FixedCharsLengthByteSlice for &CustomStringBytesSlice {
    fn slice_by_char_indice(&self, start: usize, end: usize) -> Self {
        unsafe { &self.get_unchecked((start * BYTES_PER_CHAR)..(end * BYTES_PER_CHAR)) }
    }
    fn chars_len(&self) -> usize {
        self.len() / BYTES_PER_CHAR
    }
    fn is_valid_custom_str_bytes(&self) -> bool {
        if self.len() % 4 != 0 {
            return false;
        }
        for index in 0..self.chars_len() {
            let current_slice = self.slice_by_char_indice(index, index + 1);
            match current_slice {
                [0, 0, 0, one_byte_char]
                    if one_byte_char <= &VALID_ONE_BYTE_UTF8_FIRST_BYTE_MAX_VALUE => {}
                [0, 0, first_byte, second_byte]
                    if is_in_range(*first_byte, VALID_TWO_BYTE_UTF8_FIRST_BYTE_RANGE)
                        && is_in_range(*second_byte, VALID_TWO_BYTE_UTF8_SECOND_BYTE_RANGE) => {}
                [0, first_byte, second_byte, third_byte]
                    if is_in_range(*first_byte, VALID_THREE_BYTE_UTF8_FIRST_BYTE_RANGE)
                        && is_in_range(*second_byte, VALID_THREE_BYTE_UTF8_SECOND_BYTE_RANGE)
                        && is_in_range(*third_byte, VALID_THREE_BYTE_UTF8_THIRD_BYTE_RANGE) => {}
                [first_byte, second_byte, third_byte, fourth_byte]
                    if is_in_range(*first_byte, VALID_FOUR_BYTE_UTF8_FIRST_BYTE_RANGE)
                        && is_in_range(*second_byte, VALID_FOUR_BYTE_UTF8_SECOND_BYTE_RANGE)
                        && is_in_range(*third_byte, VALID_FOUR_BYTE_UTF8_THIRD_BYTE_RANGE)
                        && is_in_range(*fourth_byte, VALID_FOUR_BYTE_UTF8_FOURTH_BYTE_RANGE) => {}
                _ => {
                    return false;
                }
            }
        }
        true
    }
}
#[test]
fn check_slice() {
    let ex: &[u8] = &[255, 255, 255, 255, 0, 255, 111, 0];
    assert_eq!(ex.slice_by_char_indice(0, 1), &[255, 255, 255, 255]);
    assert_eq!(ex.slice_by_char_indice(1, 2), &[0, 255, 111, 0]);
}
