//! 憲法の値域の導出の歯（便 49・docs/design/delivery-49.md §1 (d) 1）。build.rs を path で取り込み、
//! 純粋な関数 `constitution_enums`（憲法の正本の文字列 → 導出した Rust の source か理由の文）を直に呼ぶ。
//! 実の正本で Ok・鍵の数だけ型が出る（数は file から数える）・変異 5 つがそれぞれ Err で文に鍵の名が入る。
//! 便 50（delivery-50.md §1 (d)(e) 4）: 定数 ENUMS（鍵の名と NAMES の対の列）が鍵の数だけ対を持つ。
//! 便 128（delivery-128.md §1 (c) 歯 6）: 純粋な関数 `constitution_fields`（憲法の 5 部位の欄の一覧）の Ok と Err の 4 つの形。

#[allow(dead_code)]
#[path = "../build.rs"]
mod build;

use yaml_rust2::{Yaml, YamlLoader};

fn constitution() -> String {
    std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../design-intent/constitution.yaml"
    ))
    .expect("憲法の正本が読めない")
}

/// 実の正本の schema.enums の鍵の数（file から数える・定数を書かない）。
fn enum_key_count(text: &str) -> usize {
    let docs = YamlLoader::load_from_str(text).expect("憲法の正本が YAML として読めない");
    docs[0]
        .as_hash()
        .and_then(|m| m.get(&Yaml::String("schema".to_string())))
        .and_then(Yaml::as_hash)
        .and_then(|m| m.get(&Yaml::String("enums".to_string())))
        .and_then(Yaml::as_hash)
        .expect("schema.enums が表でない")
        .len()
}

/// 正本の 1 か所を書き換えた変異（置き換えが当たらなければ歯の側の誤り）。
fn mutate(text: &str, from: &str, to: &str) -> String {
    assert_eq!(
        text.matches(from).count(),
        1,
        "変異の的「{from}」が正本に 1 つでない"
    );
    text.replace(from, to)
}

const RETREAT_LINE: &str = "retreat_kind: [spike, measure, ruling]";

#[test]
fn constitution_enums_derives_types_from_the_real_constitution() {
    let source = build::constitution_enums(&constitution()).expect("実の憲法から導出できない");
    assert!(
        source.contains("pub enum RetreatKind {"),
        "RetreatKind の型が無い"
    );
    assert!(
        source.contains("pub enum MechanismLive {"),
        "MechanismLive の型が無い"
    );
    assert!(source.contains("    M0,\n"), "M0 の名が無い");
    assert!(source.contains("    MustNot,\n"), "MustNot の名が無い");
    assert!(
        source.contains("    V1Incident,\n"),
        "V1Incident の名が無い"
    );
}

#[test]
fn constitution_enums_emits_one_type_per_key_of_the_file() {
    let text = constitution();
    let keys = enum_key_count(&text);
    assert!(keys > 0);
    let source = build::constitution_enums(&text).unwrap();
    assert_eq!(source.matches("pub enum ").count(), keys);
    assert_eq!(source.matches("pub const NAMES: [&str; ").count(), keys);
    assert_eq!(source.matches("pub const ALL: [").count(), keys);
    assert_eq!(source.matches("pub fn from_name(").count(), keys);
}

/// 便 50 §1 (d)(e) 4: 導出した source に定数 ENUMS（鍵の名と NAMES の対の列）が在り、実の憲法の鍵の数だけ対を持つ。
#[test]
fn constitution_enums_emits_the_key_table_with_one_pair_per_key() {
    let text = constitution();
    let keys = enum_key_count(&text);
    let source = build::constitution_enums(&text).unwrap();
    let head = format!("pub const ENUMS: [(&str, &[&str]); {keys}] = [\n");
    let at = source.find(&head).expect("定数 ENUMS が無い");
    let body = &source[at + head.len()..];
    let end = body.find("];\n").expect("ENUMS が閉じていない");
    let rows: Vec<&str> = body[..end].lines().collect();
    assert_eq!(rows.len(), keys, "ENUMS の対の数が鍵の数と違う");
    assert!(
        rows.contains(&"    (\"retreat_kind\", &RetreatKind::NAMES),"),
        "{rows:?}"
    );
    assert!(rows.contains(&"    (\"tier\", &Tier::NAMES),"), "{rows:?}");
    assert_eq!(source.matches("pub const ENUMS: ").count(), 1);
}

#[test]
fn constitution_enums_rejects_a_constitution_without_schema_enums() {
    let text = mutate(&constitution(), "\n  enums:\n", "\n  enums_gone:\n");
    let err = build::constitution_enums(&text).expect_err("schema.enums が無いのに Ok");
    assert!(err.contains("schema.enums"), "{err}");
}

#[test]
fn constitution_enums_rejects_an_empty_list() {
    let text = mutate(&constitution(), RETREAT_LINE, "retreat_kind: []");
    let err = build::constitution_enums(&text).expect_err("空の一覧なのに Ok");
    assert!(err.contains("retreat_kind"), "{err}");
}

#[test]
fn constitution_enums_rejects_a_value_written_twice() {
    let text = mutate(
        &constitution(),
        RETREAT_LINE,
        "retreat_kind: [spike, measure, ruling, spike]",
    );
    let err = build::constitution_enums(&text).expect_err("同じ値が 2 度在るのに Ok");
    assert!(
        err.contains("retreat_kind") && err.contains("spike"),
        "{err}"
    );
}

#[test]
fn constitution_enums_rejects_a_number_as_a_value() {
    let text = mutate(
        &constitution(),
        RETREAT_LINE,
        "retreat_kind: [spike, 1, ruling]",
    );
    let err = build::constitution_enums(&text).expect_err("数の値なのに Ok");
    assert!(err.contains("retreat_kind"), "{err}");
}

/// 便 128 §1 (c) 歯 6: 憲法の 5 部位の欄の一覧の導出（constitution_fields）は実の正本で Ok で 5 部位の名が全部出て、
/// 部位が表でない・required が字の一覧でない・optional に字でない値・同じ名の 2 度、はそれぞれ Err で理由の文に部位の名が入る。
#[test]
fn f128_constitution_fields_refuses_broken_parts() {
    let text = constitution();
    let source = build::constitution_fields(&text).expect("実の憲法から導出できない");
    for part in ["meta", "precedence", "article", "mechanism", "statement"] {
        assert!(source.contains(&format!("    ({part:?}, &")), "部位 {part} が無い: {source}");
    }
    let cases = [
        (
            "meta",
            "\n  meta:\n    required: [",
            "\n  meta: x\n  meta_moved:\n    required: [",
        ),
        (
            "statement",
            "required: [id, pattern, strength, text]",
            "required: x",
        ),
        (
            "article",
            "optional: [relations, retreat,",
            "optional: [1, relations, retreat,",
        ),
        ("mechanism", "required: [kind, live]", "required: [kind, live, kind]"),
    ];
    for (part, from, to) in cases {
        let err = build::constitution_fields(&mutate(&text, from, to))
            .expect_err(&format!("{part} の変異なのに Ok"));
        assert!(err.contains(part), "{part}: {err}");
    }
}

#[test]
fn constitution_enums_rejects_two_values_that_collapse_to_one_variant() {
    // a-b と a--b は variant がどちらも AB になる組（実測）。
    let text = mutate(
        &constitution(),
        RETREAT_LINE,
        "retreat_kind: [spike, measure, ruling, a-b, a--b]",
    );
    let err = build::constitution_enums(&text).expect_err("同じ名に潰れるのに Ok");
    assert!(err.contains("retreat_kind") && err.contains("AB"), "{err}");
}
