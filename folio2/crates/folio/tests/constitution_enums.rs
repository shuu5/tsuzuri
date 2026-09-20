//! 憲法の値域の導出の歯（便 49・docs/design/delivery-49.md §1 (d) 1）。build.rs を path で取り込み、
//! 純粋な関数 `constitution_enums`（憲法の正本の文字列 → 導出した Rust の source か理由の文）を直に呼ぶ。
//! 実の正本で Ok・鍵の数だけ型が出る（数は file から数える）・変異 5 つがそれぞれ Err で文に鍵の名が入る。

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
    assert_eq!(text.matches(from).count(), 1, "変異の的「{from}」が正本に 1 つでない");
    text.replace(from, to)
}

const RETREAT_LINE: &str = "retreat_kind: [spike, measure, ruling]";

#[test]
fn constitution_enums_derives_types_from_the_real_constitution() {
    let source = build::constitution_enums(&constitution()).expect("実の憲法から導出できない");
    assert!(source.contains("pub enum RetreatKind {"), "RetreatKind の型が無い");
    assert!(source.contains("pub enum MechanismLive {"), "MechanismLive の型が無い");
    assert!(source.contains("    M0,\n"), "M0 の名が無い");
    assert!(source.contains("    MustNot,\n"), "MustNot の名が無い");
    assert!(source.contains("    V1Incident,\n"), "V1Incident の名が無い");
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
    assert!(err.contains("retreat_kind") && err.contains("spike"), "{err}");
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
