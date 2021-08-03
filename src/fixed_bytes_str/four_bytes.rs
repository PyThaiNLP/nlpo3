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
//pub type ValidUTF8BytesSlice = [u8];
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
/*
pub fn rfind_space(custom_text: &CustomStringBytesSlice) -> Option<usize> {
    assert_eq!(custom_text.len() % 4, 0);

    for index in (0..(custom_text.len() / BYTES_PER_CHAR)).rev() {
        if let SPACE_BYTE = &custom_text[(index) * BYTES_PER_CHAR..(index + 1) * BYTES_PER_CHAR] {
            return Some((index) * BYTES_PER_CHAR);
        }
    }
    None
}
*/

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
    matches!(
        custom_bytes,
        [0, 0, 0, 9]
            | [0, 0, 0, 10]
            | [0, 0, 0, 11]
            | [0, 0, 0, 12]
            | [0, 0, 0, 13]
            | [0, 0, 0, 32]
            | [0, 0, 194, 133]
            | [0, 0xe2, 0x80, 0x8e]
            | [0, 0xe2, 0x80, 0x8f]
            | [0, 0xe2, 0x80, 0xa8]
            | [0, 0xe2, 0x80, 0xa9]
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
    assert_eq!("".is_empty(), true);
}

#[test]
fn test_byte() {
    let long_text = [
        "ไต้หวัน (แป่ะเอ๋ยี้: Tâi-oân; ไต่อวัน) หรือ ไถวาน ",
        "(อักษรโรมัน: Taiwan; จีนตัวย่อ: 台湾; จีนตัวเต็ม: 臺灣/台灣; พินอิน: ",
        "Táiwān; ไถวาน) หรือชื่อทางการว่า สาธารณรัฐจีน (จีนตัวย่อ: 中华民国; ",
        "จีนตัวเต็ม: 中華民國; พินอิน: Zhōnghuá ",
        "Mínguó) เป็นรัฐในทวีปเอเชียตะวันออก[7][8][9] ปัจจุบันประกอบด้วย",
        "เกาะใหญ่ 5 แห่ง คือ จินเหมิน (金門), ไต้หวัน, เผิงหู (澎湖), หมาจู่ ",
        "(馬祖), และอูชิว (烏坵) กับทั้งเกาะเล็กเกาะน้อยอีกจำนวนหนึ่ง ",
        "ท้องที่ดังกล่าวเรียกรวมกันว่า \"พื้นที่ไต้หวัน\" (臺灣地區)\n",
        "ไต้หวันด้านตะวันตกติดกับจีนแผ่นดินใหญ่ ด้านตะวันออกและตะวันออก",
        "เฉียงเหนือติดกับญี่ปุ่น และด้านใต้ติดกับฟิลิปปินส์ กรุงไทเปเป็น",
        "เมืองหลวง ส่วนไทเปใหม่เป็นเขตปกครองที่จัดตั้งขึ้นใหม่ กินพื้นที่",
        "กรุงไทเปและเป็นเขตซึ่งประชากรหนาแน่นที่สุดในเวลานี้\n",
        "เกาะไต้หวันเดิมเป็นที่อยู่ของชนพื้นเมือง และมีชาวจีนจากแผ่นดิน",
        "ใหญ่เข้ามาอาศัยร่วมด้วย จนกระทั่งชาววิลันดาและสเปนเดินทางเข้า",
        "มาในยุคสำรวจเมื่อศตวรรษที่ 17 และมาตั้งบ้านเรือนกลายเป็นนิคม",
        "ใหญ่โต ต่อมาปี 1662 ราชวงศ์หมิงในแผ่นดินใหญ่ถูกราชวงศ์ชิงแทนที่ ",
        "เจิ้ง เฉิงกง (鄭成功) ขุนศึกหมิง รวมกำลังหนีมาถึงเกาะไต้หวัน ",
        "และรุกไล่ฝรั่งออกไปได้อย่างราบคาบ เขาจึงตั้งราชอาณาจักรตงหนิง ",
        "(東寧) ขึ้นบนเกาะเพื่อ \"โค่นชิงฟื้นหมิง\" แต่ในปี 1683 ราชวงศ์",
        "ชิงปราบปรามอาณาจักรตงหนิงและเข้าครอบครองไต้หวันเป็นผลสำเร็จ ",
        "ไต้หวันจึงกลายเป็นมณฑลหนึ่งของจีน อย่างไรก็ดี ความบาดหมางระหว่าง",
        "จีนกับญี่ปุ่นเป็นเหตุให้ญี่ปุ่นได้ไต้หวันไปในปี 1895\n",
        "ก่อนเสียไต้หวันคืนแก่จีนหลังสงครามโลกครั้งที่สอง ช่วงนั้น มีการ",
        "เปลี่ยนแปลงการปกครองในจีน พรรคก๊กมินตั๋ง ได้เป็นใหญ่ ",
        "แต่ไม่นานก็เสียทีให้แก่พรรคคอมมิวนิสต์จีน พรรคก๊กมินตั๋งจึงหนี",
        "มายังเกาะไต้หวันและสถาปนาสาธารณรัฐจีนขึ้นบนเกาะแยกต่างหาก ",
        "ส่วนฝ่ายคอมมิวนิสต์จีนที่เป็นฝ่ายได้รับชัยชนะได้สถาปนาสาธารณรัฐ",
        "ประชาชนจีนบนแผ่นดินใหญ่ อย่างไรก็ดี จีนยังคงถือว่า ไต้หวันเป็น",
        "มณฑลหนึ่งของตน และไต้หวันเองก็ยังมิได้รับการยอมรับจากนานาชาติ",
        "ว่าเป็นประเทศเอกราชมาจนบัดนี้\n",
        "ในช่วงทศวรรษ 1980 ถึงต้นทศวรรษ 1990 การเมืองการปกครอง",
        "สาธารณรัฐจีน (ไต้หวัน) เจริญรุ่งเรืองจนเป็นประชาธิปไตยที่มีพรรค",
        "การเมืองหลายพรรคและมีการเลือกตั้งทั่วหน้า ในช่วงกลางศตวรรษที่ ",
        "20 เศรษฐกิจไต้หวันงอกงามอย่างรวดเร็ว ไต้หวันจึงกลายเป็นประเทศ",
        "พัฒนาแล้ว ได้ชื่อว่าเป็นหนึ่งในสี่เสือแห่งเอเชีย มีอุตสาหกรรม",
        "ล้ำหน้า และมีเศรษฐกิจใหญ่โตเป็นอันดับที่ 19 ของโลก[11][12] ",
        "อุตสาหกรรมที่ใช้เทคโนโลยีชั้นสูงของไต้หวันยังมีบทบาทสำคัญมากใน",
        "เศรษฐกิจโลก เป็นเหตุให้ไต้หวันได้เป็นสมาชิกองค์การการค้าโลกและ",
        "ความร่วมมือทางเศรษฐกิจเอเชีย-แปซิฟิก เสรีภาพของสื่อมวลชน เสรี",
        "ภาพทางเศรษฐกิจ การสาธารณสุข[13]การศึกษา และดัชนีการพัฒนามนุษย์ใน",
        "ไต้หวันยังได้รับการจัดอยู่ในอันดับสูงด้วย[14][4][15]\n",
        "สาธารณรัฐจีน มีลักษณะเป็นกลุ่มเกาะ ภูมิประเทศติดกับทะเล ไม่ติด",
        "กับประเทศใดเลย ห่างจากเกาะทางทิศเหนือและทิศตะวันตกเป็นสาธารณรัฐ",
        "ประชาชนจีน ทิศใต้เป็นประเทศฟิลิปปินส์และทะเลจีนใต้ ส่วนทิศ",
        "ตะวันออกเป็นมหาสมุทรแปซิฟิก\n",
        "ในปี ค.ศ. 1638 หลังการพ่ายแพ้ของหลานชายของเจิ้ง เฉิงกง ",
        "จากการบุกโจมตีทางทัพเรือของราชวงศ์ชิงแมนจูที่นำทัพโดยชื่อ หลาง",
        "จากทางใต้ของมณฑลฝูเจี้ยน ทำให้ราชวงศ์ชิงผนวกยึดเกาะไต้หวันเป็น",
        "ส่วนหนึ่งสำเร็จ และวางไว้ภายใต้เขตอำนาจของมณฑลฝูเจี้ยน ราชสำนัก",
        "ราชวงศ์ชิงพยายามลดการละเมิดสิทธิ์และความไม่ลงรอยกันในพื้นที่โดย",
        "ออกกฎหมายเพื่อจัดการตรวจคนเข้าเมืองและเคารพสิทธิในที่ดินของชน",
        "พื้นเมืองไต้หวัน ผู้อพยพจากฝูเจี้ยนทางใต้ส่วนใหญ่ยังคงเดินทางไป",
        "ไต้หวัน เขตแดนระหว่างดินแดนที่เสียภาษีและสิ่งที่ถูกพิจารณาว่า",
        "เป็นดินแดน \"เขตอันตราย\" เปลี่ยนไปทางทิศตะวันออกโดยชาวพื้นเมือง",
        "บางคนเข้ารีตรับวัฒนธรรมแบบจีน ในขณะที่คนอื่นถอยกลับเข้าในภูเขา ",
        "ในช่วงเวลานี้มีความขัดแย้งจำนวนมากระหว่างกลุ่มชาวฮั่นด้วยกันเอง",
        "จากภูมิภาคต่าง ๆ ของฝูเจี้ยนทางใต้โดยเฉพาะอย่างยิ่งระหว่างเฉวียน",
        "โจวกับฉางโจว และระหว่างฝูเจี้ยนตอนใต้และชาวพื้นเมืองไต้หวัน\n",
        "พ.ศ. 2454 (ค.ศ. 1911) การจลาจลอู่ฮั่นในประเทศจีน เป็นจุดเริ่มต้น",
        "การล่มสลายของราชวงศ์ชิง เมื่อพรรคคอมมิวนิสต์จีนเข้ามีอำนาจในจีน",
        "แผ่นดินใหญ่เมื่อ พ.ศ. 2492 (1949) พรรคก๊กมินตั๋ง พรรคการเมือง",
        "ชาตินิยมของจีนที่เป็นฝ่ายแพ้ก็พาผู้คนอพยพหนีออกจากแผ่นดินใหญ่มา",
        "ตั้งหลักที่ไต้หวัน เพื่อวางแผนกลับไปครองอำนาจในจีนต่อไป\n",
        "ชาวจีนมากกว่า 1 ล้าน 5 แสนคน อพยพตามมาอยู่ที่เกาะไต้หวันในยุคที่",
        "เหมา เจ๋อตง มีอำนาจเต็มที่ในจีนแผ่นดินใหญ่ ผู้นำของประเทศทั้งสอง",
        "จีนคือผู้นำพรรคคอมมิวนิสต์กับผู้นำสาธารณรัฐจีนบนเกาะไต้หวัน แย่ง",
        "กันเป็นกระบอกเสียงของประชาชนจีนในเวทีโลก แต่เสียงของนานาประเทศ",
        "ส่วนใหญ่เกรงอิทธิพลของจีนแผ่นดินใหญ่ จึงให้การยอมรับจีนแผ่นดิน",
        "ใหญ่มากกว่า\n",
        "ในปี พ.ศ. 2514 (ค.ศ. 1971) ก่อนที่นายพล เจียง ไคเช็ก",
        "(ภาษาจีน: 蔣中正) จะถึงอสัญกรรมไม่กี่ปี สาธารณรัฐจีนซึ่งเป็น",
        "ประเทศที่ร่วมก่อตั้งองค์การสหประชาชาติได้สูญเสียสมาชิกภาพใน",
        "ฐานะตัวแทนชาวจีนให้กับสาธารณรัฐประชาชนจีน ในปี พ.ศ. 2521 (1978)",
        "สหประชาชาติประกาศรับรองจีนเดียวคือจีนแผ่นดินใหญ่และตัดสัมพันธ์",
        "ทางการเมืองกับสาธารณรัฐจีน ทั้งสหรัฐอเมริกาก็ได้ถอนการรับรองว่า",
        "สาธารณรัฐจีนมีฐานะเป็นรัฐ ไต้หวันจึงกลายเป็นเพียงดินแดนที่จีน",
        "อ้างว่าเป็นส่วนหนึ่งของสาธารณรัฐประชาชนจีนตั้งแต่นั้นเป็นต้นมา\n",
        "เมื่อเจียง ไคเช็ก ถึงแก่อสัญกรรมในปี พ.ศ. 2518 (1975) ลูกชาย",
        "ที่ชื่อ เจี่ยง จิงกั๋ว ได้เป็นผู้สืบทอดการปกครอง",
        "ไต้หวันต่อและเริ่มกระบวนการ วางรากฐานไปสู่ประชาธิปไตย\n",
        "หลังจากที่ประธานาธิบดี เจียง จิงกั๋ว เสียชีวิต ไต้หวันจึงได้เข้า",
        "สู่ระบอบประชาธิปไตยเต็มรูปแบบ ประธานาธิบดีคนใหม่ ซึ่งเกิดใน",
        "ไต้หวัน ชื่อ หลี่ เติงฮุย ขึ้นบริหารประเทศ โดยการสนับสนุนของ",
        "เจี่ยง จิงกั๋ว ทั้งที่ หลี่ เติงฮุย นั้นเคลื่อนไหว",
        "สนับสนุนเอกราชไต้หวัน นาย รัฐบาลจีนที่ปักกิ่งได้ตั้ง",
        "ฉายาประธานาธิบดีไต้หวันคนใหม่ว่า \"จิ้งจกปากหวาน\" ",
        "ช่วงเวลาที่นายหลี่ เติงฮุย เป็นประธานาธิบดี การเมืองของไต้หวัน",
        "เกิดการแตกแยกออกเป็น 3 ฝ่ายคือ 1) พวกก๊กมินตั๋ง ที่ต้องการกลับ",
        "ไปรวมประเทศกับจีนแผ่นดินใหญ่ (รวมจีนแผ่นดินใหญ่ภายใต้การปกครอง",
        "ของสาธารณรัฐจีน) 2) พวกที่ต้องการให้ไต้หวันเป็นประเทศอิสระไม่",
        "เกี่ยวข้องกับจีนแผ่นดินใหญ่ และ 3) พวกที่ต้องการดำรงฐานะของ",
        "ประเทศไว้ดังเดิมต่อไป\n",
        "ไต้หวันกับจีนแผ่นดินใหญ่นัดเจรจาหาทางออกของข้อขัดแย้งทางการเมือง",
        "ครั้งแรกที่สิงคโปร์เมื่อปี พ.ศ. 2536 (ค.ศ. 1993) แต่ปรากฏว่าจีน",
        "แผ่นดินใหญ่ประวิงเวลาลงนามในสัญญาหลายฉบับที่เป็นข้อตกลงร่วมกัน ",
        "ทำให้ผลการเจรจาคราวนั้นไม่ก้าวหน้าไปถึงไหน ความสัมพันธ์ระหว่าง",
        "สองจีนเลวร้ายลงทุกที เมื่อประธานาธิบดี หลี่ เติงฮุย เดินทางไป",
        "เยือนสหรัฐอเมริกาและได้รับการยอมรับอย่างเอิกเกริก ทำให้จีนแผ่น",
        "ดินใหญ่ไม่พอใจอย่างมาก จึงข่มขวัญไต้หวันกับประเทศที่ให้การสนับ",
        "สนุนไต้หวัน ด้วยการทำการซ้อมรบขึ้นใกล้ ๆ เกาะไต้หวัน สหรัฐ",
        "อเมริกาออกมาแสดงอาการปกป้องคุ้มครองไต้หวันด้วยการส่งกำลังกอง",
        "เรือรบของสหรัฐฯ มาป้วนเปี้ยนอยู่ในน่านน้ำที่จีนซ้อมรบ\n",
        "ขณะที่โลกกำลังล่อแหลมกับสถานการณ์ที่ตึงเครียดในน่านน้ำจีนมาก",
        "ขึ้นทุกทีนั้น ไต้หวันก็จัดให้มีการเลือกตั้งครั้งใหม่ และในการ",
        "เลือกตั้งครั้งใหม่นั้นเอง ไต้หวันก็ได้นายหลี่ เติงฮุย เป็น",
        "ประธานาธิบดีอีกครั้ง\n",
        "ไต้หวันเข้าสู่สภาวะวิกฤต เมื่อเกิดแผ่นดินไหวครั้งร้ายแรงที่สุดใน",
        "ประวัติศาสตร์ในเดือนกันยายน พ.ศ. 2542 (ค.ศ. 1999) ทำให้ประชากร",
        "ส่วนมากที่เป็นชาวพื้นเมืองเสียชีวิตไป 2,000 คน ทั้งเมืองมีแต่",
        "เศษซากปรักหักพังจากภัยธรรมชาติ และช่วงนี้ไต้หวันต้องเผชิญความ",
        "ยากลำบาก จีนแผ่นดินใหญ่ก็เพิ่มความกดดันไม่ให้นานาชาติ",
        "เข้ามายุ่งเกี่ยวกับไต้หวันแม้ในยามคับขันเช่นนี้ โดยประกาศว่า ",
        "หากมีประเทศใดจะเข้าไปให้ความช่วยเหลือไต้หวัน จะต้องได้รับอนุญาต",
        "จากจีนก่อน ซึ่งคำประกาศของจีนแผ่นดินใหญ่สวนทางกับเมตตาธรรมของ",
        "ประเทศทั่วโลกที่ต้องการให้ความช่วยเหลือไต้หวัน\n",
        "เดือนมีนาคม พ.ศ. 2543 (ค.ศ. 2000) มีการเลือกตั้งใหม่ในไต้หวัน ",
        "ชาวไต้หวันเลือกผู้แทนจากพรรคประชาธิปไตยก้าวหน้า คือ นายเฉิน สุย",
        "เปี่ยน เป็นประธานาธิบดีคนใหม่ของไต้หวัน ผู้ประกาศนโยบายการเมือง",
        "แข็งกร้าวว่าไต้หวันต้องการแยกตัวเป็นอิสระจากจีนแผ่นดินใหญ่ ยุติ",
        "ยุคของพรรคชาตินิยมที่ยังฝักใฝ่แผ่นดินใหญ่อยู่ จีนแผ่นดินใหญ่จึง",
        "ถือว่าเป็นกบฏต่อการปกครองของจีน เพราะแต่ไหนแต่ไร ไต้หวันไม่เคย",
        "ประกาศอย่างเป็นทางการว่าเป็นประเทศอิสระแยกจากจีน และจีนพูดอยู่",
        "เสมอว่าไต้หวันเป็นเด็กในปกครองที่ค่อนข้างจะหัวดื้อและเกเร หาก",
        "ไต้หวันประกาศว่าเป็นอิสระจากจีนเมื่อใด จีนก็จะยกกำลังจัดการ",
        "กับไต้หวันทันที\n",
        "ในขณะที่ความสัมพันธ์ทางการเมืองระหว่างสองจีนในสายตาชาวโลก",
        "เลวร้ายลง จีนทั้งสองกลับมีการติดต่อทางการค้ากันมากขึ้น มีการ",
        "ผ่อนปรนอนุญาตให้ชาวไต้หวันเดินทางไปจีนแผ่นดินใหญ่เพื่อเยี่ยม",
        "ญาติได้ เกิดปรากฏการณ์สำคัญคือนักธุรกิจไต้หวันหอบเงินทุนกว่า ",
        "20,000 ล้านดอลลาร์สหรัฐ ไปลงทุนดำเนินธุรกิจทางตอนใต้ของจีน",
        "แผ่นดินใหญ่ จนกระทั่งขณะนี้ชาวไต้หวันกลายเป็นนักลงทุนรายใหญ่",
        "เป็นลำดับ 2 ของจีน\n",
        "วันที่ 24 พฤษภาคม 2560 ศาลรัฐธรรมนูญวินิจฉัยว่ากฎหมายสมรส",
        "ปัจจุบันในเวลานั้น ละเมิดรัฐธรรมนูญ โดยปฏิเสธสิทธิสมรสของคู่รัก",
        "เพศเดียวกันชาวไต้หวัน ศาลวินิจฉัยว่าหากสภานิติบัญญัติไม่ผ่าน",
        "การแก้ไขกฎหมายที่เพียงพอต่อกฎหมายสมรสของไต้หวันภายในสองปี ",
        "การสมรสเพศเดียวกันจะชอบด้วยกฎหมายโดยอัตโนมัติในไต้หวัน[17] ",
        "วันที่ 17 พฤษภาคม 2562 สภานิติบัญญัติไต้หวันอนุมัติ",
        "ร่างกฎหมายทำให้การสมรสเพศเดียวกันชอบด้วยกฎหมาย",
        " ทำให้เป็นประเทศแรกในทวีปเอเชียที่ผ่านกฎหมายดังกล่าว[18][19]",
    ]
    .join("");
    let custom_string = CustomString::new(&long_text);
    assert_eq!(custom_string.len() % 4, 0);
}
