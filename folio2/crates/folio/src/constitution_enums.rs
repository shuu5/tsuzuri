//! 憲法の値域の置き場（便 49・docs/design/delivery-49.md §1 (b)・ADR-11 決定 (4)②）。憲法の正本
//! `design-intent/constitution.yaml` の schema.enums に在る鍵を、組み立て時に build.rs が閉じた一覧（型・ALL・NAMES・
//! name・from_name）へ導出し `OUT_DIR` に書いたものを取り込む。人は鍵も値も書かない（決定 (3)(ア)・P-6.4）。
//! 本便で使うのは RetreatKind（判断の記録の床）と Strength（注入）の 2 つ。残りは面の生成器の表（便 50）が使う。

#![allow(dead_code)]

include!(concat!(env!("OUT_DIR"), "/constitution_enums.rs"));

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yaml;

    fn constitution() -> yaml::Node {
        let text = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../design-intent/constitution.yaml"
        ))
        .expect("憲法の正本が読めない");
        yaml::parse(&text).expect("憲法の正本が YAML として読めない").root
    }

    /// 鍵と導出した型の対（file の鍵がこの列と過不足なく一致することも見る）。
    const PAIRS: [(&str, &[&str]); 10] = [
        ("tier", &Tier::NAMES),
        ("binds", &Binds::NAMES),
        ("pattern", &Pattern::NAMES),
        ("strength", &Strength::NAMES),
        ("rationale_kind", &RationaleKind::NAMES),
        ("mechanism_kind", &MechanismKind::NAMES),
        ("mechanism_live", &MechanismLive::NAMES),
        ("retreat_kind", &RetreatKind::NAMES),
        ("stage", &Stage::NAMES),
        ("polarity", &Polarity::NAMES),
    ];

    #[test]
    fn constitution_enums_names_match_the_file_in_length_text_and_order() {
        let root = constitution();
        let enums = root
            .get("schema")
            .and_then(|s| s.get("enums"))
            .and_then(yaml::Node::as_map)
            .expect("schema.enums が表でない");
        let file_keys: Vec<&str> = enums.iter().map(|(k, _)| k.as_str()).collect();
        let pair_keys: Vec<&str> = PAIRS.iter().map(|(k, _)| *k).collect();
        assert_eq!(file_keys, pair_keys, "schema.enums の鍵が歯の対と過不足なく一致しない");
        for ((key, body), (_, names)) in enums.iter().zip(PAIRS.iter()) {
            let file_values: Vec<&str> = body
                .as_seq()
                .unwrap_or_else(|| panic!("schema.enums.{key} が一覧でない"))
                .iter()
                .map(|v| v.as_str().unwrap_or_else(|| panic!("schema.enums.{key} に文字列でない値")))
                .collect();
            assert_eq!(&file_values[..], *names, "schema.enums.{key} の値の列が導出した NAMES と違う");
        }
    }

    /// 導出の側から独立の凍結の針（P-10.1）。
    #[test]
    fn constitution_enums_frozen_needles_for_retreat_kind_and_strength() {
        assert_eq!(RetreatKind::NAMES, ["spike", "measure", "ruling"]);
        assert_eq!(Strength::NAMES, ["must", "must-not", "should"]);
    }

    #[test]
    fn constitution_enums_from_name_is_the_inverse_of_name_and_rejects_the_outside() {
        for (i, k) in RetreatKind::ALL.iter().enumerate() {
            assert_eq!(k.name(), RetreatKind::NAMES[i]);
            assert_eq!(RetreatKind::from_name(k.name()), Some(*k));
        }
        for (i, s) in Strength::ALL.iter().enumerate() {
            assert_eq!(s.name(), Strength::NAMES[i]);
            assert_eq!(Strength::from_name(s.name()), Some(*s));
        }
        assert_eq!(Strength::from_name("must-not"), Some(Strength::MustNot));
        assert_eq!(MechanismLive::from_name("M0"), Some(MechanismLive::M0));
        assert_eq!(MechanismLive::from_name("delivery-0"), Some(MechanismLive::Delivery0));
        assert_eq!(RationaleKind::from_name("v1-incident"), Some(RationaleKind::V1Incident));
        assert!(RetreatKind::from_name("spike ").is_none());
        assert!(RetreatKind::from_name("Spike").is_none());
        assert!(Strength::from_name("may").is_none());
        assert!(Strength::from_name("").is_none());
    }
}
