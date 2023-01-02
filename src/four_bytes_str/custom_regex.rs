// This is a result of an attempt to create a formatter
// which translates normal, human readable thai regex
// into 4-bytes zero-left-pad bytes regex pattern string

use anyhow::{Error as AnyError, Result};
use regex_syntax::{
    hir::{Anchor, Class, Group, Literal as LiteralEnum, Repetition},
    hir::{ClassUnicodeRange, Hir, HirKind},
    is_meta_character, Parser,
};
use std::{error::Error, fmt::Display};
trait ToCustomStringRepr {
    fn to_custom_byte_repr(&self) -> Result<String>;
}

#[derive(Debug, Clone)]
enum UnsupportedCustomRegexParserError {
    ByteLiteral,
    ByteClass,
    DifferentRanges(char, char),
    RepetitionRange,
    AnchorStartLine,
    AnchorEndLine,
}
enum IterableHirKind {
    Alternation(Vec<Hir>),
    Concat(Vec<Hir>),
}

impl Display for UnsupportedCustomRegexParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ByteLiteral => {
                write!(f, "Byte literal is not supported")
            }
            UnsupportedCustomRegexParserError::ByteClass => {
                write!(f, "Byte class is not supported")
            }
            UnsupportedCustomRegexParserError::DifferentRanges(a, b) => {
                write!(
                    f,
                    "Different byte length range is not supported {} {}",
                    a, b
                )
            }
            UnsupportedCustomRegexParserError::RepetitionRange => todo!(),
            UnsupportedCustomRegexParserError::ByteLiteral => todo!(),
            UnsupportedCustomRegexParserError::AnchorStartLine => todo!(),
            UnsupportedCustomRegexParserError::AnchorEndLine => todo!(),
        }
    }
}
impl Error for UnsupportedCustomRegexParserError {}

impl ToCustomStringRepr for Hir {
    fn to_custom_byte_repr(&self) -> Result<String> {
        self.kind().to_custom_byte_repr()
    }
}
impl ToCustomStringRepr for HirKind {
    fn to_custom_byte_repr(&self) -> Result<String> {
        match self {
            HirKind::Empty => todo!(),
            HirKind::Literal(l) => l.to_custom_byte_repr(),
            HirKind::Class(c) => c.to_custom_byte_repr(),
            HirKind::Anchor(a) => a.to_custom_byte_repr(),
            HirKind::WordBoundary(_) => todo!(),
            HirKind::Repetition(r) => r.to_custom_byte_repr(),
            HirKind::Group(g) => g.to_custom_byte_repr(),
            HirKind::Concat(c) => IterableHirKind::Concat(c.to_vec()).to_custom_byte_repr(),
            HirKind::Alternation(a) => {
                IterableHirKind::Alternation(a.to_vec()).to_custom_byte_repr()
            }
        }
    }
}
impl ToCustomStringRepr for Anchor {
    fn to_custom_byte_repr(&self) -> Result<String> {
        match self {
            Anchor::StartLine => todo!(),
            Anchor::EndLine => todo!(),
            Anchor::StartText => Ok("^".to_string()),
            Anchor::EndText => Ok("$".to_string()),
        }
    }
}
impl ToCustomStringRepr for LiteralEnum {
    fn to_custom_byte_repr(&self) -> Result<String> {
        match self {
            LiteralEnum::Unicode(a) => Ok(a.to_four_byte_string()),
            LiteralEnum::Byte(b) => Err(AnyError::new(
                UnsupportedCustomRegexParserError::ByteLiteral,
            )),
        }
    }
}
impl ToCustomStringRepr for Class {
    fn to_custom_byte_repr(&self) -> Result<String> {
        match self {
            Class::Unicode(u) => Ok(u.ranges().to_four_byte_string()),
            Class::Bytes(_) => Err(AnyError::from(UnsupportedCustomRegexParserError::ByteClass)),
        }
    }
}
impl ToCustomStringRepr for Repetition {
    fn to_custom_byte_repr(&self) -> Result<String> {
        let symbol: Result<String> = match &self.kind {
            regex_syntax::hir::RepetitionKind::ZeroOrOne => Ok("?".to_string()),
            regex_syntax::hir::RepetitionKind::ZeroOrMore => Ok("*".to_string()),
            regex_syntax::hir::RepetitionKind::OneOrMore => Ok("+".to_string()),
            regex_syntax::hir::RepetitionKind::Range(r) => match r {
                regex_syntax::hir::RepetitionRange::Exactly(e) => Ok(format!("{{{}}}", e)),
                regex_syntax::hir::RepetitionRange::AtLeast(l) => Ok(format!("{{{},}}", l)),
                regex_syntax::hir::RepetitionRange::Bounded(start, end) => {
                    Ok(format!("{{{},{}}}", start, end))
                }
            },
        };

        let repeated_expression = match &self.hir.kind() {
            HirKind::Empty => todo!(),
            HirKind::Literal(l) => l.to_custom_byte_repr(),
            HirKind::Class(c) => c.to_custom_byte_repr(),
            HirKind::Anchor(a) => a.to_custom_byte_repr(),
            HirKind::WordBoundary(_) => todo!(),
            HirKind::Repetition(r) => r.to_custom_byte_repr(),
            HirKind::Group(g) => g.to_custom_byte_repr(),
            HirKind::Concat(c) => IterableHirKind::Concat(c.to_vec()).to_custom_byte_repr(),
            HirKind::Alternation(a) => {
                IterableHirKind::Alternation(a.to_vec()).to_custom_byte_repr()
            }
        };
        if let HirKind::Group(_) = &self.hir.kind() {
            Ok(repeated_expression? + &symbol?)
        } else {
            Ok("(".to_owned() + &repeated_expression? + ")" + &symbol?)
        }
    }
}
impl ToCustomStringRepr for IterableHirKind {
    fn to_custom_byte_repr(&self) -> Result<String> {
        match self {
            IterableHirKind::Alternation(a) => {
                let mut cus_str = String::new();
                for member in a {
                    match member.kind() {
                        HirKind::Empty => todo!(),
                        HirKind::Literal(literal) => {
                            if !cus_str.is_empty() {
                                cus_str = cus_str
                                    + "|"
                                    + format!("({})", &literal.to_custom_byte_repr()?).as_str();
                            } else {
                                cus_str = format!("({})", &literal.to_custom_byte_repr()?);
                            }
                        }
                        HirKind::Class(c) => {
                            if !cus_str.is_empty() {
                                cus_str = cus_str
                                    + "|"
                                    + format!("({})", &c.to_custom_byte_repr()?).as_str();
                            } else {
                                cus_str = format!("({})", &c.to_custom_byte_repr()?);
                            }
                        }
                        HirKind::Anchor(a) => {
                            if !cus_str.is_empty() {
                                cus_str = cus_str
                                    + "|"
                                    + format!("({})", &a.to_custom_byte_repr()?).as_str();
                            } else {
                                cus_str = format!("({})", &a.to_custom_byte_repr()?);
                            }
                        }
                        HirKind::WordBoundary(_) => todo!(),
                        HirKind::Repetition(r) => {
                            if !cus_str.is_empty() {
                                cus_str = cus_str
                                    + "|"
                                    + format!("({})", &r.to_custom_byte_repr()?).as_str();
                            } else {
                                cus_str = format!("({})", &r.to_custom_byte_repr()?);
                            }
                        }
                        HirKind::Group(g) => {
                            if !cus_str.is_empty() {
                                cus_str = cus_str
                                    + "|"
                                    + format!("({})", &g.to_custom_byte_repr()?).as_str();
                            } else {
                                cus_str = format!("({})", &g.to_custom_byte_repr()?);
                            }
                        }
                        HirKind::Concat(concat) => {
                            if !cus_str.is_empty() {
                                cus_str = cus_str
                                    + "|"
                                    + format!(
                                        "({})",
                                        (&IterableHirKind::Concat(concat.to_vec())
                                            .to_custom_byte_repr()?)
                                    )
                                    .as_str();
                            } else {
                                cus_str = IterableHirKind::Concat(concat.to_vec())
                                    .to_custom_byte_repr()?;
                            }
                        }
                        HirKind::Alternation(alternation) => {
                            cus_str = cus_str
                                + &IterableHirKind::Alternation(alternation.to_vec())
                                    .to_custom_byte_repr()?;
                        }
                    }
                }
                Ok(cus_str)
            }
            IterableHirKind::Concat(c) => {
                let mut cus_str = String::new();
                for member in c {
                    match member.kind() {
                        HirKind::Empty => todo!(),
                        HirKind::Literal(literal) => {
                            cus_str = cus_str + &literal.to_custom_byte_repr()?;
                        }
                        HirKind::Class(c) => cus_str = cus_str + &c.to_custom_byte_repr()?,
                        HirKind::Anchor(a) => cus_str = cus_str + &a.to_custom_byte_repr()?,
                        HirKind::WordBoundary(_) => todo!(),
                        HirKind::Repetition(r) => cus_str = cus_str + &r.to_custom_byte_repr()?,
                        HirKind::Group(g) => cus_str = cus_str + &g.to_custom_byte_repr()?,
                        HirKind::Concat(concat) => {
                            cus_str = cus_str
                                + &IterableHirKind::Concat(concat.to_vec())
                                    .to_custom_byte_repr()?;
                        }
                        HirKind::Alternation(alternation) => {
                            cus_str = cus_str
                                + &(IterableHirKind::Alternation(alternation.to_vec())
                                    .to_custom_byte_repr()?);
                        }
                    }
                }
                Ok(cus_str)
            }
        }
    }
}
impl ToCustomStringRepr for Group {
    fn to_custom_byte_repr(&self) -> Result<String> {
        let recur = match self.hir.kind() {
            HirKind::Empty => todo!(),
            HirKind::Literal(lit) => lit.to_custom_byte_repr(),
            HirKind::Class(c) => c.to_custom_byte_repr(),
            HirKind::Anchor(a) => a.to_custom_byte_repr(),
            HirKind::WordBoundary(_) => todo!(),
            HirKind::Repetition(_) => todo!(),
            HirKind::Group(g) => g.to_custom_byte_repr(),
            HirKind::Concat(c) => IterableHirKind::Concat(c.to_vec()).to_custom_byte_repr(),
            HirKind::Alternation(a) => {
                IterableHirKind::Alternation(a.to_vec()).to_custom_byte_repr()
            }
        };
        Ok("(".to_owned() + &recur? + ")")
    }
}

fn get_char_range_byte_class(class_range: &ClassUnicodeRange) -> Option<UTFBytesLength> {
    // currently allow only the same byte length
    let start_class = char_class(class_range.start());
    let end_class = char_class(class_range.end());
    if start_class == end_class {
        Some(start_class)
    } else {
        None
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum UTFBytesLength {
    One,
    Two,
    Three,
    Four,
}
fn char_class(character: char) -> UTFBytesLength {
    let mut bytes_buffer: [u8; 4] = [0; 4];

    character.encode_utf8(&mut bytes_buffer);
    match bytes_buffer {
        [_a, 0, 0, 0] => UTFBytesLength::One,
        [_a, _b, 0, 0] => UTFBytesLength::Two,
        [_a, _b, _c, 0] => UTFBytesLength::Three,
        _ => UTFBytesLength::Four,
    }
}

fn is_in_range<T: PartialEq + PartialOrd>(value: T, range: (T, T)) -> bool {
    value >= range.0 && value <= range.1
}

trait PadLeftZeroFourBytesRep {
    fn to_four_byte_string(&self) -> String;
}
fn escape_meta_character(c: char) -> String {
    if is_meta_character(c) {
        format!(r"\{}", c)
    } else if c.is_whitespace() {
        format!("{:?}", c).replace('\'', "")
    } else {
        c.to_string()
    }
}
impl PadLeftZeroFourBytesRep for &[ClassUnicodeRange] {
    fn to_four_byte_string(&self) -> String {
        let urange = self;
        let char_classes = urange
            .iter()
            .map(get_char_range_byte_class)
            .collect::<Vec<_>>();

        if char_classes.iter().all(|elem| elem.is_some()) {
            // must be the same class for every range pair!
            let the_class = char_classes.first().unwrap().unwrap();

            if char_classes.iter().all(|elem| elem.unwrap() == the_class) {
                let pad_left_0 = match the_class {
                    UTFBytesLength::One => r"\x00\x00\x00",
                    UTFBytesLength::Two => r"\x00\x00",
                    UTFBytesLength::Three => r"\x00",
                    UTFBytesLength::Four => r"",
                };
                let mut output_four_bytes_rep: Vec<String> = vec![];
                // we want to create all syntax of \x00\x00\x00[a-z]
                for regex_range in urange.iter() {
                    let (start, end) = (regex_range.start(), regex_range.end());
                    if start == end {
                        output_four_bytes_rep
                            .push(escape_meta_character(end).to_string().replace("'", ""));
                    } else {
                        output_four_bytes_rep.push(
                            format!(
                                r"{}-{}",
                                escape_meta_character(start),
                                escape_meta_character(end)
                            )
                            .replace('\'', ""),
                        );
                    }
                }
                format!(r"{}[{}]", pad_left_0, output_four_bytes_rep.join(""))
            } else {
                println!("{:?}", self);
                todo!()
            }
        } else {
            todo!()
        }
    }
}
impl PadLeftZeroFourBytesRep for char {
    fn to_four_byte_string(&self) -> String {
        let character = self;

        let mut bytes_buffer: [u8; 4] = [0; 4];
        character.encode_utf8(&mut bytes_buffer);
        // not leading zero yet
        let result = match bytes_buffer {
            [_a, 0, 0, 0] => {
                if character.is_alphanumeric() || (character.is_whitespace() && *character == ' ') {
                    format!(r"\x00\x00\x00{}", character)
                } else {
                    format!(r"\x00\x00\x00{:?}", character).replace('\'', "")
                }
            }
            [_a, _b, 0, 0] => {
                format!(r"\x00\x00{}", character)
            }
            [_a, _b, _c, 0] => {
                format!(r"\x00{}", character)
            }
            _ => character.to_string(),
        };
        result
    }
}

pub fn regex_pattern_to_custom_pattern(regex_pattern: &str) -> Result<String> {
    let hir = Parser::new().parse(regex_pattern)?;
    hir.to_custom_byte_repr()
}
