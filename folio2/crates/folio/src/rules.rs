//! 規則の表の欄の決まりの床（便 51・docs/design/delivery-51.md §1・ADR-11 決定 (3)(イ) と (4)②・FR5 / FR19）。床の木 `FLOOR` は
//! 規則の表の正本 `design-intent/rules.yaml` の先頭の節 `schema` の正本で、葉は下の定数と同じ配列を指す（同じ一覧を 2 回書かない）。
//! `_note` で終わる欄は人が読む説明の注で、凍結 anchor tests/fixtures/schema/rules-region.txt の順と字面のまま持つ（`schema.rs` の
//! `derive` が書く）。値域 stage と種別と憲法の機構の対応の右辺は憲法から導出した型（`constitution_enums.rs`）の名を使う＝憲法の値と
//! 規則の表の値が 1 か所から出る。値域 kind と status は規則の表だけの値域なので、この file の定数が正本。
//! 床（`check.rs` の `check_rules`）は最上位の節の閉じた一覧を file の schema.top_level ではなく `RULES_TOP_LEVEL` から読む
//! （file の側で節を足して通す口を塞ぐ・N-3.1）。実の file の生成区間と命令 `folio schema` の対象に足すのは後続の段
//! （便 53）＝それまで `FLOOR` と行の欄の定数の読み手は歯だけなので、未使用の警告はこの file だけ黙らせる。

#![allow(dead_code)]

use crate::constitution_enums::{MechanismKind, Stage};
use crate::schema::Floor;

/// 規則の表の最上位の節の閉じた一覧（`FLOOR` の top_level・thresholds と discipline は人が書き、schema は生成区間・ほかの名は未知の節）。
pub const RULES_TOP_LEVEL: [&str; 3] = ["schema", "thresholds", "discipline"];

/// 閾値の行（R-n）が必ず持つ欄。
pub const THRESHOLD_REQUIRED: [&str; 9] = [
    "id", "article", "what", "value", "kind", "status", "ruling", "ruled_at", "stage",
];

/// 閾値の行が持ってよい欄。
pub const THRESHOLD_OPTIONAL: [&str; 5] =
    ["basis", "projection", "same_failure", "population", "note"];

/// 作法の行（D-n）が必ず持つ欄。
pub const DISCIPLINE_REQUIRED: [&str; 7] = [
    "id", "article", "what", "kind", "status", "ruling", "ruled_at",
];

/// 作法の行が持ってよい欄。
pub const DISCIPLINE_OPTIONAL: [&str; 1] = ["note"];

/// 種別の値域（規則の表だけの値域・順も固定）。
pub const RULE_KIND: [&str; 4] = ["deny", "build-check", "detect", "human-review"];

/// 状態の値域（規則の表だけの値域・順は file の順）。
pub const RULE_STATUS: [&str; 3] = ["仮", "凍結", "未定"];

/// 種別 deny に対応する憲法の機構（`kind_map_to_constitution.deny`・名は憲法から導出した型の name）。
const KIND_DENY_MAPS_TO: [&str; 2] = [
    MechanismKind::Reject.name(),
    MechanismKind::BuildCheck.name(),
];

/// 種別 build-check に対応する憲法の機構。
const KIND_BUILD_CHECK_MAPS_TO: [&str; 1] = [MechanismKind::BuildCheck.name()];

/// 種別 detect に対応する憲法の機構。
const KIND_DETECT_MAPS_TO: [&str; 1] = [MechanismKind::None.name()];

/// 凍結から外したもの（測る仕組みが別）。
const EXCLUDED_WHAT: [&str; 2] = ["時間（「60 分以内」）", "費用（「300k token 以下」）"];

/// 床の定数の木（値は実の rules.yaml の schema 節の字面と 1 字も違わない・足したのは `_note` の 3 欄だけ）。
/// `_note` で終わる欄は人が読む説明の注で、`folio schema` の導出だけが使う。
pub(crate) const FLOOR: Floor = Floor::Map(&[
    ("version", Floor::Num(1)),
    ("top_level", Floor::Strs(&RULES_TOP_LEVEL)),
    (
        "top_level_note",
        Floor::Val(
            "最上位の節の閉じた一覧（ほかの節は床が落とす・N-3）。thresholds と discipline は人が書き、schema は生成区間",
        ),
    ),
    (
        "threshold_row",
        Floor::Map(&[
            ("required", Floor::Strs(&THRESHOLD_REQUIRED)),
            ("optional", Floor::Strs(&THRESHOLD_OPTIONAL)),
        ]),
    ),
    (
        "discipline_row",
        Floor::Map(&[
            ("required", Floor::Strs(&DISCIPLINE_REQUIRED)),
            ("optional", Floor::Strs(&DISCIPLINE_OPTIONAL)),
        ]),
    ),
    (
        "enums",
        Floor::Map(&[
            ("kind", Floor::Strs(&RULE_KIND)),
            ("status", Floor::Strs(&RULE_STATUS)),
            ("stage", Floor::Strs(&Stage::NAMES)),
        ]),
    ),
    (
        "enums_note",
        Floor::Val("stage は憲法の値域 stage と同じ値（実装は憲法から導出した名の列を使う）"),
    ),
    (
        "kind_meaning",
        Floor::Map(&[
            (
                "deny",
                Floor::Val("機械が測って、超過なら落とす（憲法の reject / build-check に対応）"),
            ),
            (
                "build-check",
                Floor::Val("生成時の検査で数え、違反なら落とす（憲法の build-check に対応）"),
            ),
            (
                "detect",
                Floor::Val("記録・起票のみ・止めない（憲法の none に対応）"),
            ),
            (
                "human-review",
                Floor::Val("人が守る作法（憲法の human-review に対応・D 行）"),
            ),
        ]),
    ),
    (
        "kind_map_to_constitution",
        Floor::Map(&[
            ("deny", Floor::Strs(&KIND_DENY_MAPS_TO)),
            ("build-check", Floor::Strs(&KIND_BUILD_CHECK_MAPS_TO)),
            ("detect", Floor::Strs(&KIND_DETECT_MAPS_TO)),
        ]),
    ),
    (
        "kind_map_to_constitution_note",
        Floor::Val(
            "R 行にだけ適用する（D 行は作法＝条の機構とは別）。右辺は憲法の値域 mechanism_kind の値",
        ),
    ),
    (
        "excluded",
        Floor::Map(&[
            ("what", Floor::Strs(&EXCLUDED_WHAT)),
            (
                "why",
                Floor::Val(
                    "測る仕組みが別で凍結できない。M1 の実地試験（非エンジニア 1 回）で初めて数値を決める（要件書 not_frozen）。",
                ),
            ),
        ]),
    ),
    (
        "reverse_reference",
        Floor::Val(
            "各行は article 欄で条を指し、その条の relations.rules に行 id が載る（R-4 の双方向）。",
        ),
    ),
]);

#[cfg(test)]
mod tests {
    use super::*;

    /// 床の木の導出は凍結 anchor（設計判断の席が独立の実装で組んだ・P-10.1）と byte 一致（便 51 §1 (c)1）。
    /// anchor を書き換えて合わせてはいけない（変えるなら設計判断の席へ問う）。
    #[test]
    fn rules_floor_derives_the_frozen_anchor_byte_for_byte() {
        let anchor = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/schema/rules-region.txt"
        ))
        .unwrap();
        assert_eq!(crate::schema::derive(&FLOOR), anchor);
    }

    /// 凍結の針（便 51 §1 (c)2）: 値域 stage は憲法から導出した名の列と同じ・最上位の節は 3 値（字面を直に書く）。
    #[test]
    fn rules_floor_stage_comes_from_the_constitution_and_top_level_is_the_three_sections() {
        let Floor::Map(fields) = &FLOOR else {
            panic!("FLOOR は表");
        };
        let Some((_, Floor::Map(enums))) = fields.iter().find(|(k, _)| *k == "enums") else {
            panic!("enums が表でない");
        };
        let Some((_, Floor::Strs(stage))) = enums.iter().find(|(k, _)| *k == "stage") else {
            panic!("enums.stage が一覧でない");
        };
        assert_eq!(*stage, &Stage::NAMES[..]);
        assert_eq!(Stage::NAMES, ["in-loop", "post"]);
        let Some((_, Floor::Strs(top))) = fields.iter().find(|(k, _)| *k == "top_level") else {
            panic!("top_level が一覧でない");
        };
        assert_eq!(*top, ["schema", "thresholds", "discipline"]);
        assert_eq!(RULES_TOP_LEVEL, ["schema", "thresholds", "discipline"]);
    }

    /// 種別と憲法の機構の対応の右辺は憲法の値域 mechanism_kind の名（1 か所から出る）。
    #[test]
    fn rules_floor_kind_map_targets_are_constitution_mechanism_names() {
        for name in KIND_DENY_MAPS_TO
            .iter()
            .chain(&KIND_BUILD_CHECK_MAPS_TO)
            .chain(&KIND_DETECT_MAPS_TO)
        {
            assert!(MechanismKind::from_name(name).is_some(), "{name}");
        }
        assert_eq!(KIND_DENY_MAPS_TO, ["reject", "build-check"]);
        assert_eq!(KIND_DETECT_MAPS_TO, ["none"]);
    }
}
