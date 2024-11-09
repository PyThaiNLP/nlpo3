// SPDX-FileCopyrightText: 2024 PyThaiNLP Project
// SPDX-License-Identifier: Apache-2.0

/**
 * Rules for TCC (Thai Character Cluster) tokenization.
*/
use crate::four_bytes_str::custom_regex::regex_pattern_to_custom_pattern;
use lazy_static::lazy_static;
use regex::bytes::Regex;
// เc็ck 1
// เcctาะk 2
// เccีtยะk 3
// เccีtย(?=[เ-ไก-ฮ]|$)k look ahead 1
// เcc็ck 4
// เcิc์ck 5
// เcิtck  6
// เcีtยะ?k 7
// เcืtอะ?k 8
// เc[ิีุู]tย(?=[เ-ไก-ฮ]|$)k look ahead 2
// เctา?ะ?k 9
// cัtวะk 10
// c[ัื]tc[ุิะ]?k 11
// c[ิุู]์ 12
// c[ะ-ู]tk 13
// cรรc์ 14
// c็ 15
// ct[ะาำ]?k 16
// ck 17
// แc็c 18
// แcc์ 19
// แctะ 20
// แcc็c 21
// แccc์ 22
// โctะ 23
// [เ-ไ]ct 24
// ก็
// อึ
// หึ
pub fn replace_tcc_symbol(tcc_pattern: &str) -> String {
    tcc_pattern
        .replace('k', "(cc?[dิ]?[์])?")
        .replace('c', "[ก-ฮ]")
        .replace('t', "[่-๋]?")
        .replace('d', &"อูอุ".replace('อ', ""))
}
lazy_static! {
    pub static ref NON_LOOKAHEAD_TCC: Regex = Regex::new(
        &[
            r"^เc็ck",                //1
            r"^เcctาะk",            //2
            r"^เccีtยะk",         //3
            r"^เcc็ck",               //4
            r"^เcิc์ck",            //5
            r"^เcิtck",               //6
            r"^เcีtยะ?k",         //7
            r"^เcืtอะ?k",         //8
            r"^เctา?ะ?k",           //9
            r"^cัtวะk",             //10
            r"^c[ัื]tc[ุิะ]?k", //11
            r"^c[ิุู]์k",         //12
            r"^c[ะ-ู]tk",             //13
            r"^cรรc์ ็",          //14
            r"^c็",                     //15
            r"^ct[ะาำ]?k",          //16
            r"^ck",                       //17
            r"^แc็c",                 //18
            r"^แcc์",                 //19
            r"^แctะ",                 //20
            r"^แcc็c",                //21
            r"^แccc์",                //22
            r"^โctะ",                 //23
            r"^[เ-ไ]ct",              //24
            r"^ก็",
            r"^อึ",
            r"^หึ",
            r"^(เccีtย)[เ-ไก-ฮ]k",           // look ahead 1
            r"^(เc[ิีุู]tย)[เ-ไก-ฮ]k", // look ahead 2
        ].map(|pattern| {
                regex_pattern_to_custom_pattern(&replace_tcc_symbol(pattern)).unwrap()
            }).join("|")
    ).unwrap();
    pub static ref LOOKAHEAD_TCC: Regex = Regex::new(
        &[
    r"^(เccีtย)[เ-ไก-ฮ]k",           //เccีtย(?=[เ-ไก-ฮ]|$)
    r"^(เc[ิีุู]tย)[เ-ไก-ฮ]k" //เc[ิีุู]tย(?=[เ-ไก-ฮ]|$)
]
            .map(|pattern| {
                regex_pattern_to_custom_pattern(&replace_tcc_symbol(pattern)).unwrap()
            }).join("|")
    )
    .unwrap();
}

#[test]
fn tcc_regex_test_cases() {
    // เc็c 1 1
    // เcctาะ 2 2
    // เccีtยะ 3 3
    // เcc็c 4 4
    // เcิc์c 5 5
    // เcิtc 6 6
    // เcีtยะ? 7 7
    // เcืtอะ? 8 8
    // เctา?ะ? 9 9
    // cัtวะ 10
    // c[ัื]tc[ุิะ]? 11
    // c[ิุู]์ 12
    // c[ะ-ู]t 13
    // c็ 14
    // ct[ะาำ]? 15
    // แc็c 16
    // แcc์ 17
    // แctะ 18
    // แcc็c 19
    // แccc์ 20
    // โctะ 21
    // [เ-ไ]ct 22

    let case_1 = replace_tcc_symbol("^เc็ck");
    let case_2 = replace_tcc_symbol("^เcctาะ");
    let case_3 = replace_tcc_symbol("^เccีtยะ");
    let case_4 = replace_tcc_symbol("^เcc็c");
    let case_5 = replace_tcc_symbol("^เcิc์c");
    let case_6 = replace_tcc_symbol("^เcิtc");
    let case_7 = replace_tcc_symbol("^เcีtยะ?");
    let case_8 = replace_tcc_symbol("^เcืtอะ?");
    let case_9 = replace_tcc_symbol("^เctา?ะ?");
    let case_10 = replace_tcc_symbol("^cัtวะ");
    let case_11 = replace_tcc_symbol("^c[ัื]tc[ุิะ]?");
    let case_12 = replace_tcc_symbol("^c[ิุู]์");
    let case_13 = replace_tcc_symbol("^c[ะ-ู]t");
    let case_14 = replace_tcc_symbol("^c็");
    let case_15 = replace_tcc_symbol("^ct[ะาำ]?");
    let case_16 = replace_tcc_symbol("^แc็c");
    let case_17 = replace_tcc_symbol("^แcc์");
    let case_18 = replace_tcc_symbol("^แctะ");
    let case_19 = replace_tcc_symbol("^แcc็c");
    let case_20 = replace_tcc_symbol("^แccc์");
    let case_21 = replace_tcc_symbol("^โctะ");
    let case_22 = replace_tcc_symbol("^[เ-ไ]ct");

    // This is the only Karan case.
    assert_eq!(
        regex_pattern_to_custom_pattern(&case_1).unwrap(),
        r"^\x00เ\x00[ก-ฮ]\x00็\x00[ก-ฮ](\x00[ก-ฮ](\x00[ก-ฮ])?(\x00[ิุ-ู])?\x00[์])?"
    );
    assert_eq!(
        regex_pattern_to_custom_pattern(&case_2).unwrap(),
        r"^\x00เ\x00[ก-ฮ]\x00[ก-ฮ](\x00[่-๋])?\x00า\x00ะ"
    );

    assert_eq!(
        regex_pattern_to_custom_pattern(&case_3).unwrap(),
        r"^\x00เ\x00[ก-ฮ]\x00[ก-ฮ]\x00ี(\x00[่-๋])?\x00ย\x00ะ"
    );

    assert_eq!(
        regex_pattern_to_custom_pattern(&case_4).unwrap(),
        r"^\x00เ\x00[ก-ฮ]\x00[ก-ฮ]\x00็\x00[ก-ฮ]"
    );
    assert_eq!(
        regex_pattern_to_custom_pattern(&case_5).unwrap(),
        r"^\x00เ\x00[ก-ฮ]\x00ิ\x00[ก-ฮ]\x00์\x00[ก-ฮ]"
    );
    assert_eq!(
        regex_pattern_to_custom_pattern(&case_6).unwrap(),
        r"^\x00เ\x00[ก-ฮ]\x00ิ(\x00[่-๋])?\x00[ก-ฮ]"
    );
    assert_eq!(
        regex_pattern_to_custom_pattern(&case_7).unwrap(),
        r"^\x00เ\x00[ก-ฮ]\x00ี(\x00[่-๋])?\x00ย(\x00ะ)?"
    );
    assert_eq!(
        regex_pattern_to_custom_pattern(&case_8).unwrap(),
        r"^\x00เ\x00[ก-ฮ]\x00ื(\x00[่-๋])?\x00อ(\x00ะ)?"
    );
    assert_eq!(
        regex_pattern_to_custom_pattern(&case_9).unwrap(),
        r"^\x00เ\x00[ก-ฮ](\x00[่-๋])?(\x00า)?(\x00ะ)?"
    );

    assert_eq!(
        regex_pattern_to_custom_pattern(&case_10).unwrap(),
        r"^\x00[ก-ฮ]\x00ั(\x00[่-๋])?\x00ว\x00ะ"
    );
    assert_eq!(
        regex_pattern_to_custom_pattern(&case_11).unwrap(),
        r"^\x00[ก-ฮ]\x00[ัื](\x00[่-๋])?\x00[ก-ฮ](\x00[ะิุ])?"
    );
    assert_eq!(
        regex_pattern_to_custom_pattern(&case_12).unwrap(),
        r"^\x00[ก-ฮ]\x00[ิุ-ู]\x00์"
    );
    assert_eq!(
        regex_pattern_to_custom_pattern(&case_13).unwrap(),
        r"^\x00[ก-ฮ]\x00[ะ-ู](\x00[่-๋])?"
    );
    assert_eq!(
        regex_pattern_to_custom_pattern(&case_14).unwrap(),
        r"^\x00[ก-ฮ]\x00็"
    );
    assert_eq!(
        regex_pattern_to_custom_pattern(&case_15).unwrap(),
        r"^\x00[ก-ฮ](\x00[่-๋])?(\x00[ะา-ำ])?"
    );
    assert_eq!(
        regex_pattern_to_custom_pattern(&case_16).unwrap(),
        r"^\x00แ\x00[ก-ฮ]\x00็\x00[ก-ฮ]"
    );
    assert_eq!(
        regex_pattern_to_custom_pattern(&case_17).unwrap(),
        r"^\x00แ\x00[ก-ฮ]\x00[ก-ฮ]\x00์"
    );
    assert_eq!(
        regex_pattern_to_custom_pattern(&case_18).unwrap(),
        r"^\x00แ\x00[ก-ฮ](\x00[่-๋])?\x00ะ"
    );
    assert_eq!(
        regex_pattern_to_custom_pattern(&case_19).unwrap(),
        r"^\x00แ\x00[ก-ฮ]\x00[ก-ฮ]\x00็\x00[ก-ฮ]"
    );
    assert_eq!(
        regex_pattern_to_custom_pattern(&case_20).unwrap(),
        r"^\x00แ\x00[ก-ฮ]\x00[ก-ฮ]\x00[ก-ฮ]\x00์"
    );
    assert_eq!(
        regex_pattern_to_custom_pattern(&case_21).unwrap(),
        r"^\x00โ\x00[ก-ฮ](\x00[่-๋])?\x00ะ"
    );
    assert_eq!(
        regex_pattern_to_custom_pattern(&case_22).unwrap(),
        r"^\x00[เ-ไ]\x00[ก-ฮ](\x00[่-๋])?"
    );

    let look_ahead_case_1 = replace_tcc_symbol(r"^(เccีtย)[เ-ไก-ฮ]");
    let look_ahead_1_regex = regex_pattern_to_custom_pattern(&look_ahead_case_1).unwrap();
    let look_ahead_case_2 = replace_tcc_symbol(r"^(เc[ิีุู]tย)[เ-ไก-ฮ]");
    let look_ahead_2_regex = regex_pattern_to_custom_pattern(&look_ahead_case_2).unwrap();
    assert!(
        (look_ahead_1_regex == r"^(\x00เ\x00[ก-ฮ]\x00[ก-ฮ]\x00ี(\x00[่-๋])?\x00ย)\x00[เ-ไก-ฮ]"
            || look_ahead_1_regex == r"^(\x00เ\x00[ก-ฮ]\x00[ก-ฮ]\x00ี(\x00[่-๋])?\x00ย)\x00[ก-ฮเ-ไ]")
    );
    assert_eq!(
        look_ahead_2_regex,
        r"^(\x00เ\x00[ก-ฮ]\x00[ิ-ีุ-ู](\x00[่-๋])?\x00ย)\x00[ก-ฮเ-ไ]"
    );
}

#[test]
fn newmm_exception_match_cases() {
    assert_eq!(
        r"^(\x00\x00\x00\r)?\x00\x00\x00\n",
        regex_pattern_to_custom_pattern(r"(?x)^\r?\n").unwrap()
    );

    assert_eq!(
        r"^(\x00\x00\x00[\t ])+",
        regex_pattern_to_custom_pattern(r"^[ \t]+").unwrap()
    );
    assert_eq!(
        r"^(\x00\x00\x00[\-A-Za-z])+",
        regex_pattern_to_custom_pattern(r"(?x)^[-a-zA-Z]+").unwrap()
    );
    assert_eq!(
        r"^(\x00[๐-๙])+(\x00\x00\x00[,\.](\x00[๐-๙])+)*",
        regex_pattern_to_custom_pattern(r"(?x)^[๐-๙]+([,\.][๐-๙]+)*").unwrap()
    );
    assert_eq!(
        r"^(\x00\x00\x00[0-9])+(\x00\x00\x00[,\.](\x00\x00\x00[0-9])+)*",
        regex_pattern_to_custom_pattern(r"(?x)^[0-9]+([,\.][0-9]+)*").unwrap()
    );
    assert_eq!(
        r"^(\x00[ก-ฮ]){0,2}$",
        regex_pattern_to_custom_pattern(r"^[ก-ฮ]{0,2}$").unwrap()
    )
}
