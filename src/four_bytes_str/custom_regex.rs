use regex_syntax::{Parser, ParserBuilder, ast::{self, Literal}, hir::{Class, Literal as LiteralEnum}, hir::{ClassBytes, ClassUnicodeRange, Hir, HirKind}, is_meta_character};

fn test_regex_parser() {
    let abs = ast::parse::Parser::new().parse("(abc)+").unwrap();
    // Parser::new().p
    let hir = Parser::new().parse(r"[a-z0-9,-\-]").unwrap();
    // abc -> \x00\x00\x00

    // HirKind::
    // HirKind::
    // hir.to_string()
    // hir.()
    // println!("{}",hir.());
    println!("{:?}", hir);
    println!("{}",create_custom_bytes_regex(&hir));
    // println!("{}",create_custom_bytes_regex(&hir));
    //    Hir::
    // println!("{:?}",test);
}

fn create_custom_bytes_regex(hir: &Hir) -> String {
    match hir.kind() {
        HirKind::Empty => todo!(),
        HirKind::Literal(literal) => convert_literal(literal),
        HirKind::Class(class) => convert_class(class),
        HirKind::Anchor(anchor) => todo!(),
        HirKind::WordBoundary(bound) => todo!(),
        HirKind::Repetition(rep) => todo!(),
        HirKind::Group(group) => todo!(),
        HirKind::Concat(hirs) => iterate_concat_kind(hirs),
        HirKind::Alternation(hirs) => todo!(),
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
fn convert_class(class: &Class) -> String {
    match class {
        Class::Unicode(u) => {
            println!("{:?}", u);
            return u.ranges().to_four_byte_string();
        }
        Class::Bytes(b) => todo!(),
    }
    "test".to_string()
}
fn convert_literal(literal: &LiteralEnum) -> String {
    match literal {
        LiteralEnum::Unicode(a) => a.to_four_byte_string(),
        LiteralEnum::Byte(b) => todo!(),
    }
}
fn iterate_concat_kind(concat_members: &Vec<Hir>) -> String {
    let mut cus_str = String::new();
    for member in concat_members {
        match member.kind() {
            HirKind::Empty => todo!(),
            HirKind::Literal(literal) => {
                cus_str = cus_str + &convert_literal(literal);
            }
            HirKind::Class(_) => todo!(),
            HirKind::Anchor(_) => todo!(),
            HirKind::WordBoundary(_) => todo!(),
            HirKind::Repetition(_) => todo!(),
            HirKind::Group(_) => todo!(),
            HirKind::Concat(concat) => {
                cus_str = cus_str + &(iterate_concat_kind(concat));
            }
            HirKind::Alternation(_) => todo!(),
        }
    }
    cus_str
}

#[derive(PartialEq, Eq,Clone,Copy)]
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
fn escape_meta_character(c:char)->String{
    if is_meta_character(c)  {
        format!(r"\{}",c)
    }else {
        c.to_string()
    }
}
impl PadLeftZeroFourBytesRep for &[ClassUnicodeRange] {
    fn to_four_byte_string(&self) -> String {
        let urange = self;
        let char_classes = urange
            .iter()
            .map(|range| get_char_range_byte_class(range))
            .collect::<Vec<_>>();

        if char_classes.iter().all(|elem| match elem {
            Some(_) => true,
            None => false,
        }) {
            // must be the same class for every range pair!
            let the_class = char_classes.first().unwrap().unwrap();

            if char_classes.iter().all(|elem| elem.unwrap() == the_class) {
                let pad_left_0 = match the_class {
                    UTFBytesLength::One => r"\x00\x00\x00",
                    UTFBytesLength::Two => r"\x00\x00",
                    UTFBytesLength::Three => r"\x00",
                    UTFBytesLength::Four => r"",
                };
                let mut output_four_bytes_rep = format!("({}[", pad_left_0);
                println!("{:?}",&urange);
                // we want to create all syntax of \x00\x00\x00[a-z]
                for regex_range in urange.iter() {
                    let (start, end) = (regex_range.start(), regex_range.end());
                    output_four_bytes_rep.push_str(&escape_meta_character(start));
                    output_four_bytes_rep.push('-');
                    output_four_bytes_rep.push_str(&escape_meta_character(end));
                }
                format!("{}])", output_four_bytes_rep)
            }else{
                todo!()
            }
        }else{
            todo!()
        }
    }
}
impl PadLeftZeroFourBytesRep for char {
    fn to_four_byte_string(&self) -> String {
        let character = self;

        // let mut output:&[u8;4] = &[0;output_size];

        let mut bytes_buffer: [u8; 4] = [0; 4];

        character.encode_utf8(&mut bytes_buffer);
        // not leading zero yet
        let result = match bytes_buffer {
            [_a, 0, 0, 0] => {
                format!(r"\x00\x00\x00{}", character)
            }
            [_a, _b, 0, 0] => {
                format!(r"\x00\x00{}", character)
            }
            [_a, _b, _c, 0] => {
                format!(r"\x00{}", character)
            }
            _ => character.to_string(),
        };
        // let vec_of_bytes = Vec::with_capacity(4);

        result
    }
}
#[test]
fn tcc_regex_test_cases() {
    // _RE_TCC = (
    //     """\
    // เc็c
    // เcctาะ
    // เccีtยะ
    // เccีtย(?=[เ-ไก-ฮ]|$)
    // เcc็c
    // เcิc์c
    // เcิtc
    // เcีtยะ?
    // เcืtอะ?
    // เc[ิีุู]tย(?=[เ-ไก-ฮ]|$)
    // เctา?ะ?
    // cัtวะ
    // c[ัื]tc[ุิะ]?
    // c[ิุู]์
    // c[ะ-ู]t
    // c็
    // ct[ะาำ]?
    // แc็c
    // แcc์
    // แctะ
    // แcc็c
    // แccc์
    // โctะ
    // [เ-ไ]ct
    // """.replace(
    //         "c", "[ก-ฮ]"
    //     )
    //     .replace("t", "[่-๋]?")
    //     .split()
    // )
    // assert_eq!(create_custom_bytes_regex(""))
}
