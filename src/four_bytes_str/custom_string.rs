/// Functions dealing with a custom four-byte string.
/// For more details, see src/NOTE_ON_STRING.md
use std::{
    error::{self, Error},
    fmt::Display,
    sync::Arc,
};

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

pub type CustomStringBytesVec = Vec<u8>;
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

pub trait FixedCharsLengthByteSlice {
    fn slice_by_char_indice(&self, start: usize, end: usize) -> Self;
    fn chars_len(&self) -> usize;
    fn is_valid_custom_str_bytes(&self) -> bool;
}

impl FixedCharsLengthByteSlice for &CustomStringBytesSlice {
    fn slice_by_char_indice(&self, start: usize, end: usize) -> Self {
        self.get((start * BYTES_PER_CHAR)..(end * BYTES_PER_CHAR))
            .unwrap()
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

/// Returns character index
pub fn rfind_space_char_index(custom_text: &CustomStringBytesSlice) -> Option<usize> {
    assert_eq!(custom_text.len() % 4, 0);

    for index in (0..(custom_text.len() / BYTES_PER_CHAR)).rev() {
        if let SPACE_BYTE = &custom_text[(index) * BYTES_PER_CHAR..(index + 1) * BYTES_PER_CHAR] {
            return Some(index);
        }
    }
    None
}

/// Check if a white space (including left-to-right and right-to-left marks)
fn is_whitespace(custom_bytes: &CustomStringBytesSlice) -> bool {
    matches!(
        custom_bytes,
        [0, 0, 0, 9] // Character tabulation (HT) (\t) U+0009
            | [0, 0, 0, 10] // Line feed (LF) (\n) U+000A
            | [0, 0, 0, 11] // Line tabulation (VT) U+000B
            | [0, 0, 0, 12] // Form feed (FF) (\f) U+000C
            | [0, 0, 0, 13] // Carriage return (CR) (\r) U+000D
            | [0, 0, 0, 32] // Space U+0020
            | [0, 0, 194, 133] // Next line (NEL) U+0085
            | [0, 0, 194, 160] // No-break space (NBSP) U+00A0
            | [0, 0xe1, 0x9a, 0x80] // Ogham space mark U+1680
            | [0, 0xe1, 0xa0, 0x8e] // Mongolian vowel separator U+180E
            | [0, 0xe2, 0x80, 0x80] // En quad U+2000
            | [0, 0xe2, 0x80, 0x81] // Em quad U+2001
            | [0, 0xe2, 0x80, 0x82] // En space U+2002
            | [0, 0xe2, 0x80, 0x83] // Em space U+2003
            | [0, 0xe2, 0x80, 0x84] // Three-per-em space U+2004
            | [0, 0xe2, 0x80, 0x85] // Four-per-em space U+2005
            | [0, 0xe2, 0x80, 0x86] // Six-per-em space U+2006
            | [0, 0xe2, 0x80, 0x87] // Figure space U+2007
            | [0, 0xe2, 0x80, 0x88] // Punctuation space U+2008
            | [0, 0xe2, 0x80, 0x89] // Thin space U+2009
            | [0, 0xe2, 0x80, 0x8a] // Hair space U+200A
            | [0, 0xe2, 0x80, 0x8b] // Zero width space (ZWSP) U+200B
            | [0, 0xe2, 0x80, 0x8c] // Zero width non-joiner (ZWNJ) U+200C
            | [0, 0xe2, 0x80, 0x8d] // Zero width joiner (ZWJ) U+200D
            | [0, 0xe2, 0x80, 0x8e] // Left-to-right mark U+200E *(control character)
            | [0, 0xe2, 0x80, 0x8f] // Right-to-left mark U+200F *(control character)
            | [0, 0xe2, 0x80, 0xa8] // Line separator U+2028
            | [0, 0xe2, 0x80, 0xa9] // Paragraph seperator U+2029
            | [0, 0xe2, 0x80, 0xaf] // Narrow no-break space U+202F
            | [0, 0xe2, 0x81, 0x9f] // Medium mathematical space U+205F
            | [0, 0xe2, 0x81, 0xa0] // Word joiner U+2060
            | [0, 0xe3, 0x80, 0x80] // Ideographic space U+3000
            | [0, 0xef, 0xbb, 0xbf] // Zero width no-break space U+FEFF
    )
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

/// This name is WIP
pub trait FixedLengthCustomString<T: Sized + FixedLengthCustomString<T>> {
    /// start and end are character index.
    fn substring(&self, start: usize, end: usize) -> T;
    fn get_original_string(&self) -> &[u8];
}

/// The content inside this string is a vector of bytes,
/// ALWAYS with length % 4 == 0
///     
/// Every character is a valid utf-8 encoded byte padded left with 0
/// to make every character takes 4 bytes.
///
/// For example, Thai characters which use 3 bytes are represented by
/// [0, valid_first_byte, valid_second_byte, valid_third_byte].
///
/// ***Comparison***
///
/// String "กข " is represented by
/// \[224, 184, 129, 224, 184, 130, 32\]
///
/// CustomString "กข " is represented by
/// \[0, 224, 184, 129, 0, 224, 184, 130, 0, 0, 0, 32\]
#[derive(Clone, Debug)]
pub struct CustomString {
    /// full content
    content: Arc<CustomStringBytesVec>,
    /// full char unicode scalar value contents, corresponding to the full content
    chars_content: Arc<Vec<char>>,
    /// char index
    start: usize,
    /// char index
    end: usize,
}

impl CustomString {
    pub fn new(base_string: &str) -> Self {
        let content = to_four_bytes(base_string);
        let chars_content = Arc::new(base_string.chars().collect::<Vec<char>>());
        let length = content.len() / BYTES_PER_CHAR;
        Self {
            content: Arc::new(content),
            start: 0,
            end: length,
            chars_content,
        }
    }

    /// Returns a sub-slice from full content
    pub fn raw_content(&self) -> &[u8] {
        self.content
            .as_slice()
            .slice_by_char_indice(self.start, self.end)
    }

    pub fn is_full_string(&self) -> bool {
        self.start == 0 && self.end == self.content.len() / BYTES_PER_CHAR
    }

    /// Returns characters length
    pub fn chars_len(&self) -> usize {
        self.end - self.start
    }

    /// Returns underlying full string bytes length.
    pub fn full_string_bytes_len(&self) -> usize {
        self.content.len()
    }

    pub fn is_empty(&self) -> bool {
        self.chars_len() == 0
    }

    pub fn trim(&self) -> Self {
        let mut new_content: &[u8] = &self.content;

        while !new_content.is_empty() && is_whitespace(&new_content[0..BYTES_PER_CHAR]) {
            // trim left
            new_content = &new_content[BYTES_PER_CHAR..];
        }

        while !new_content.is_empty()
            && is_whitespace(&new_content[(new_content.len() - BYTES_PER_CHAR)..])
        {
            // trim right
            new_content = &new_content[..(new_content.len() - BYTES_PER_CHAR)];
        }

        let length = new_content.len() / BYTES_PER_CHAR;

        Self {
            content: Arc::new(Vec::from(new_content)),
            chars_content: self.chars_content.clone(),
            start: 0,
            end: length,
        }
    }

    pub fn get_chars_content(&self) -> &[char] {
        self.chars_content
            .as_slice()
            .get(self.start..self.end)
            .unwrap()
    }

    pub fn get_char_at(&self, index: usize) -> char {
        *self
            .chars_content
            .as_slice()
            .get(index + self.start)
            .unwrap()
    }

    pub fn convert_raw_bytes_to_std_string(input: &[u8]) -> String {
        let mut output_content: Vec<u8> = Vec::with_capacity(input.len() / 100);
        for index in 0..input.chars_len() {
            let extracted_bytes =
                trim_to_std_utf8(input.slice_by_char_indice(index, index + 1)).unwrap();
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
                trim_to_std_utf8(input.slice_by_char_indice(index, index + 1)).unwrap();
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

    /// The result substring contains an atomic RC to the same full Vec<u8> as the caller's content.  
    pub fn substring(&self, start: usize, end: usize) -> Self {
        let new_start = self.start + start;
        let new_end = self.start + end;

        let full_content = self.content.clone();
        Self {
            content: full_content,
            chars_content: self.chars_content.clone(),
            start: new_start,
            end: new_end,
        }
    }

    pub fn substring_as_bytes(&self, char_start: usize, char_end: usize) -> &[u8] {
        self.content
            .as_slice()
            .slice_by_char_indice(char_start, char_end)
    }
}

#[test]
fn check_slice() {
    let ex: &[u8] = &[255, 255, 255, 255, 0, 255, 111, 0];
    assert_eq!(ex.slice_by_char_indice(0, 1), &[255, 255, 255, 255]);
    assert_eq!(ex.slice_by_char_indice(1, 2), &[0, 255, 111, 0]);
    assert!("".is_empty());
}

#[test]
fn test_bytes() {
    let text = [
        "ไต้หวัน (แป่ะเอ๋ยี้: Tâi-oân; ไต่อวัน) หรือ ไถวาน ",
        "(อักษรโรมัน: Taiwan; จีนตัวย่อ: 台湾; จีนตัวเต็ม: 臺灣/台灣; พินอิน: ",
        "Táiwān; ไถวาน) หรือชื่อทางการว่า สาธารณรัฐจีน (จีนตัวย่อ: 中华民国; ",
        "จีนตัวเต็ม: 中華民國; พินอิน: Zhōnghuá ",
        "Mínguó) เป็นรัฐในทวีปเอเชียตะวันออก[7][8][9] ปัจจุบันประกอบด้วย",
        "เกาะใหญ่ 5 แห่ง คือ จินเหมิน (金門), ไต้หวัน, เผิงหู (澎湖), หมาจู่ ",
        "(馬祖), และอูชิว (烏坵) กับทั้งเกาะเล็กเกาะน้อยอีกจำนวนหนึ่ง ",
        "ท้องที่ดังกล่าวเรียกรวมกันว่า \"พื้นที่ไต้หวัน\" (臺灣地區)\n",
    ]
    .join("");
    let custom_string = CustomString::new(&text);
    assert_eq!(custom_string.full_string_bytes_len() % 4, 0);
}

#[test]
fn test_trim() {
    assert!(CustomString::new(" ").trim().is_empty());
    assert!(CustomString::new("  ").trim().is_empty());
    assert!(CustomString::new("\n").trim().is_empty());
    assert!(CustomString::new("  \t\n ").trim().is_empty());
    assert_eq!(CustomString::new(" abc ").trim().chars_len(), 3);
    assert_eq!(CustomString::new(" aก  ").trim().full_string_bytes_len(), 8); // 2 chars * 4 bytes
}
