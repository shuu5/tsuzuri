//! 規則の表の欄の決まりの床（便 51・docs/design/delivery-51.md §1・ADR-11 決定 (3)(イ) と (4)②・FR5 / FR19）。床の木 `FLOOR` は
//! 規則の表の正本 `design-intent/rules.yaml` の先頭の節 `schema` の正本で、葉は下の定数と同じ配列を指す（同じ一覧を 2 回書かない）。
//! `_note` で終わる欄は人が読む説明の注で、凍結 anchor tests/fixtures/schema/rules-region.txt の順と字面のまま持つ（`schema.rs` の
//! `derive` が書く）。値域 stage と種別と憲法の機構の対応の右辺は憲法から導出した型（`constitution_enums.rs`）の名を使う＝憲法の値と
//! 規則の表の値が 1 か所から出る。値域 kind と status は規則の表だけの値域なので、この file の型 `RuleKind` と `RuleStatus` が正本
//! （便 54・面の名札も床の木の表の鍵もこの型の name から出る）。
//! 床（`check.rs` の `check_rules`）は最上位の節の閉じた一覧を file の schema.top_level ではなく `RULES_TOP_LEVEL` から読む
//! （file の側で節を足して通す口を塞ぐ・N-3.1）。実の file の生成区間と命令 `folio schema` の対象に足すのは後続の段
//! （便 53）。行の欄の定数は床（`check.rs` の `check_rules`）も読む（便 128・未知の欄）。未使用の警告はこの file だけ黙らせる。

#![allow(dead_code)]

use std::path::Path;

use crate::adr;
use crate::constitution_enums::{MechanismKind, Stage};
use crate::floor::{self, Floor};
use crate::yaml::Node;

/// 規則の表の最上位の節の閉じた一覧（`FLOOR` の top_level・thresholds と discipline は人が書き、schema は生成区間・ほかの名は未知の節）。
pub const RULES_TOP_LEVEL: [&str; 3] = ["schema", "thresholds", "discipline"];

/// 違反の名札の閉じた一覧（便 156・便 203・台帳 f2-648.236）: folio2 の id の名札と、外の置き場で出す検査の名。外の置き場に
/// folio2 の id を出してよいのは、床が置き場の規則の表の行をその id の字で引いて値か対象を読むときだけ（行 R-8・R-16・R-17・
/// 名札に出るのは R-17）。この一覧の検査は
/// folio2 の条と規範文の意味から来るか（A-2・N-4・P-7・P-7.1・P-8）、行の値も対象も読まない（R-3・R-9・R-10・R-11）ので、外では
/// 置き場の表に同じ id が在っても検査の名（同じ id が置き場で別の意味を持ちうる・外の利用者は tsuzuri・持ち主の裁定 2026-09-27）。
/// folio2 の置き場は id のまま（便 156 の「表に行が無ければ検査の名」は外の置き場の規則に寄せた）。名札は文言の字面（行 D-11 の写しの外）。
pub const ABROAD_LABELS: [(&str, &str); 9] = [
    ("A-2", "改訂と判断の記録"),
    ("N-4", "改訂の承認"),
    ("P-7", "id の再利用と改番"),
    ("P-7.1", "id の再利用と改番"),
    ("P-8", "撤退条件"),
    ("R-3", "部品目録"),
    ("R-9", "語彙"),
    ("R-10", "平易文"),
    ("R-11", "強度と文末"),
];

/// 置き場の違反の名札の読み（便 203）。folio2 の置き場と名の無い口（`floor::abroad` が偽）は名札の字のまま、外の置き場は
/// `ABROAD_LABELS` の id を検査の名にする（出力の口で 1 度だけ引く・書く前と後の突き合わせは元の字で数える）。
pub struct Labels {
    abroad: bool,
}

impl Labels {
    /// 置き場 `dir` の読み（名は `adr::place_name`・外の判定は便 202 と同じ `floor::abroad`）。
    pub fn of(dir: &Path) -> Labels {
        Labels {
            abroad: floor::abroad(adr::place_name(dir).ok().as_deref()),
        }
    }

    /// 出力に出す名札。
    pub fn shown<'a>(&self, kind: &'a str) -> &'a str {
        if !self.abroad {
            return kind;
        }
        ABROAD_LABELS
            .iter()
            .find(|(id, _)| *id == kind)
            .map_or(kind, |(_, name)| name)
    }
}

/// 閾値の行（R-n）が必ず持つ欄。
pub const THRESHOLD_REQUIRED: [&str; 9] = [
    "id", "article", "what", "value", "kind", "status", "ruling", "ruled_at", "stage",
];

/// 行の欄 refs の字（便 91・ADR-13 決定 (3-b)（イ））。その行の散文が依っている、article の条以外の id の一覧。
/// 床（`check.rs` の `check_rule_refs`）は id の形と、その行自身の id でも article の値でもないことと、一覧であることを数える。
/// 実在は refs.rs と link.rs の網が数える（重ねない）。
pub const ROW_REFS: &str = "refs";

/// 行の欄 key の字（便 179）。行の id は置き場ごとに意味が違うので、道具は値を読む閾値の行を id でなくこの欄で引く。
/// 値は `KEYS` の閉じた一覧の 1 つで、同じ値の閾値の行は置き場に 1 本まで（床は `key_violations`）。
pub const ROW_KEY: &str = "key";

/// 閾値の行が持ってよい欄。
pub const THRESHOLD_OPTIONAL: [&str; 7] = [
    "basis",
    "projection",
    "same_failure",
    "population",
    "note",
    ROW_REFS,
    ROW_KEY,
];

/// 設計ノートの面の章の上限の行の印（欄 key の値・面の生成器 `face_note.rs` と床 `note.rs` が `chapter_cap` で読む）。
pub const NOTE_CHAPTERS: &str = "note-chapters";

/// 計画の名札の行の印（欄 key の値・値は計画のノートの文書 id・床 `plan.rs` と導出の命令 `derive.rs` が `plan_note` で読む・便 183）。
pub const PLAN_NOTE: &str = crate::floor_note::PLAN_KEY;

/// 編集時の止めの本数の下限の行の印（欄 key の値・値は「<正の整数> 本以上」・床 `polarity.rs` が `in_loop_min` で読む・便 200）。
pub const IN_LOOP_MIN: &str = "in-loop-min";

/// 欄 key の閉じた一覧（道具が値を読む口の名・便 179・便 183 で plan-note・便 200 で in-loop-min を足した）。
pub const KEYS: [&str; 3] = [NOTE_CHAPTERS, PLAN_NOTE, IN_LOOP_MIN];

/// 章の上限の値の形「<正の整数> 章 以下」の数の後ろの字。
const CHAPTER_TAIL: &str = " 章 以下";

/// 置き場の規則の表の閾値の行のうち、欄 key が `key` の 1 本（便 179）。無い・2 本以上は Err（呼び手は まだ分からない にする）。
pub fn keyed<'a>(rules: &'a Node, key: &str) -> Result<&'a Node, String> {
    let rows = keyed_rows(rules, key);
    match rows[..] {
        [row] => Ok(row),
        [] => Err(format!("欄 {ROW_KEY} が {key} の閾値の行が無い")),
        _ => Err(format!("欄 {ROW_KEY} が {key} の閾値の行が {} 本ある", rows.len())),
    }
}

/// 置き場の規則の表の閾値の行のうち、欄 key が `key` の行の全部（書かれた順）。
fn keyed_rows<'a>(rules: &'a Node, key: &str) -> Vec<&'a Node> {
    rules
        .get(RULES_TOP_LEVEL[1])
        .and_then(Node::as_seq)
        .unwrap_or_default()
        .iter()
        .filter(|r| r.get(ROW_KEY).and_then(Node::as_str) == Some(key))
        .collect()
}

/// 計画のノートの文書 id（欄 key が plan-note の閾値の行の value・判断の記録 ADR-31 決定 (2)(ア)・便 183）。行が無ければ None
/// （計画のノートの床を掛けない）。2 本以上か値が文書 id の形でなければ Err（呼び手は まだ分からない にする・行 R-19 の欄と同じ）。
pub fn plan_note(rules: &Node) -> Result<Option<&str>, String> {
    let rows = keyed_rows(rules, PLAN_NOTE);
    let row = match rows[..] {
        [] => return Ok(None),
        [row] => row,
        _ => return Err(format!("欄 {ROW_KEY} が {PLAN_NOTE} の閾値の行が {} 本ある", rows.len())),
    };
    let value = row.get("value").and_then(Node::as_str).unwrap_or_default();
    if crate::shelf::is_doc_id(value) {
        return Ok(Some(value));
    }
    let id = row.get("id").and_then(Node::as_str).unwrap_or("?");
    Err(format!("行 {id} の value「{value}」が文書 id の形でない"))
}

/// 設計ノートの章の上限（欄 key が note-chapters の閾値の行の value「<正の整数> 章 以下」の数・便 179）。
/// 面の生成器と床が同じ関数で読む。行が無い・2 本以上・値の形が違うは Err（道具は既定の値を持たない・P-4.2）。
pub fn chapter_cap(rules: &Node) -> Result<usize, String> {
    let row = keyed(rules, NOTE_CHAPTERS)?;
    let value = row.get("value").and_then(Node::as_str).unwrap_or_default();
    value
        .strip_suffix(CHAPTER_TAIL)
        .filter(|n| !n.starts_with('0') && !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
        .and_then(|n| n.parse().ok())
        .ok_or_else(|| {
            let id = row.get("id").and_then(Node::as_str).unwrap_or("?");
            format!("行 {id} の value「{value}」が「<正の整数>{CHAPTER_TAIL}」の形でない")
        })
}

/// 編集時の止めの本数の下限（欄 key が in-loop-min の閾値の行の id と value「<正の整数> 本以上」の数・便 200・ADR-33 決定 (5)）。
/// 行が無ければ None（数えない）。2 本以上か値の形が違えば Err（呼び手は まだ分からない にする）。
pub fn in_loop_min(rules: &Node) -> Result<Option<(String, usize)>, String> {
    let rows = keyed_rows(rules, IN_LOOP_MIN);
    let row = match rows[..] {
        [] => return Ok(None),
        [row] => row,
        _ => return Err(format!("欄 {ROW_KEY} が {IN_LOOP_MIN} の閾値の行が {} 本ある", rows.len())),
    };
    let id = row.get("id").and_then(Node::as_str).unwrap_or("?").to_string();
    let value = row.get("value").and_then(Node::as_str).unwrap_or_default();
    value
        .strip_suffix(" 本以上")
        .filter(|n| !n.starts_with('0') && !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
        .and_then(|n| n.parse().ok())
        .map(|n| Some((id.clone(), n)))
        .ok_or_else(|| format!("行 {id} の value「{value}」が「<正の整数> 本以上」の形でない"))
}

/// 欄 key の床（便 179）: 値が `KEYS` に無い行と、同じ値を持つ 2 本目以降の閾値の行の字（種別 schema の違反・`check.rs` が出す）。
pub fn key_violations(rules: &Node) -> Vec<String> {
    let mut seen: Vec<&str> = Vec::new();
    let mut out = Vec::new();
    let rows = rules.get(RULES_TOP_LEVEL[1]).and_then(Node::as_seq).unwrap_or_default();
    for row in rows {
        let Some(v) = row.get(ROW_KEY) else { continue };
        let id = row.get("id").and_then(Node::as_str).unwrap_or("?");
        match v.as_str() {
            Some(k) if KEYS.contains(&k) && seen.contains(&k) => {
                out.push(format!("行 {id} の {ROW_KEY}「{k}」を持つ閾値の行が 2 本以上ある"));
            }
            Some(k) if KEYS.contains(&k) => seen.push(k),
            _ => out.push(format!(
                "行 {id} の {ROW_KEY}「{}」が閉じた一覧 {KEYS:?} に無い",
                v.as_str().unwrap_or("?")
            )),
        }
    }
    out
}

/// 作法の行（D-n）が必ず持つ欄。
pub const DISCIPLINE_REQUIRED: [&str; 7] = [
    "id", "article", "what", "kind", "status", "ruling", "ruled_at",
];

/// 作法の行が持ってよい欄。
pub const DISCIPLINE_OPTIONAL: [&str; 2] = ["note", ROW_REFS];

/// 規則の表の行の種別（規則の表だけの値域・便 54・憲法から導出した型と同じ形）。値の字面を書く唯一の所。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleKind {
    /// deny
    Deny,
    /// build-check
    BuildCheck,
    /// detect
    Detect,
    /// human-review
    HumanReview,
}
impl RuleKind {
    /// 全部（生成区間の順）
    pub const ALL: [RuleKind; 4] = [
        RuleKind::Deny,
        RuleKind::BuildCheck,
        RuleKind::Detect,
        RuleKind::HumanReview,
    ];
    /// 名の字面（生成区間の順・長さは値の数）
    pub const NAMES: [&str; 4] = ["deny", "build-check", "detect", "human-review"];
    /// 規則の表の名
    pub const fn name(self) -> &'static str {
        match self {
            RuleKind::Deny => "deny",
            RuleKind::BuildCheck => "build-check",
            RuleKind::Detect => "detect",
            RuleKind::HumanReview => "human-review",
        }
    }
    /// 規則の表の名から引く（無ければ None）
    pub fn from_name(name: &str) -> Option<RuleKind> {
        match name {
            "deny" => Some(RuleKind::Deny),
            "build-check" => Some(RuleKind::BuildCheck),
            "detect" => Some(RuleKind::Detect),
            "human-review" => Some(RuleKind::HumanReview),
            _ => None,
        }
    }
}

/// 規則の表の行の状態（規則の表だけの値域・便 54・憲法から導出した型と同じ形）。値の字面を書く唯一の所。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleStatus {
    /// 仮
    Provisional,
    /// 凍結
    Frozen,
    /// 未定
    Undecided,
}
impl RuleStatus {
    /// 全部（生成区間の順）
    pub const ALL: [RuleStatus; 3] = [
        RuleStatus::Provisional,
        RuleStatus::Frozen,
        RuleStatus::Undecided,
    ];
    /// 名の字面（生成区間の順・長さは値の数）
    pub const NAMES: [&str; 3] = ["仮", "凍結", "未定"];
    /// 規則の表の名
    pub const fn name(self) -> &'static str {
        match self {
            RuleStatus::Provisional => "仮",
            RuleStatus::Frozen => "凍結",
            RuleStatus::Undecided => "未定",
        }
    }
    /// 規則の表の名から引く（無ければ None）
    pub fn from_name(name: &str) -> Option<RuleStatus> {
        match name {
            "仮" => Some(RuleStatus::Provisional),
            "凍結" => Some(RuleStatus::Frozen),
            "未定" => Some(RuleStatus::Undecided),
            _ => None,
        }
    }
}

/// 種別 deny に対応する憲法の機構（`kind_map_to_constitution.deny`・名は憲法から導出した型の name）。
const KIND_DENY_MAPS_TO: [&str; 2] = [
    MechanismKind::Reject.name(),
    MechanismKind::BuildCheck.name(),
];

/// 種別 build-check に対応する憲法の機構。
const KIND_BUILD_CHECK_MAPS_TO: [&str; 1] = [MechanismKind::BuildCheck.name()];

/// 種別 detect に対応する憲法の機構。
const KIND_DETECT_MAPS_TO: [&str; 1] = [MechanismKind::None.name()];

/// 凍結から外したもの（測る仕組みが別）。外すのは人の作業の時間と AI の費用だけで、道具が決定的に測れる機械の待ち時間は
/// 外さない（規則の表の行に載せてよい・便 197 に同乗・台帳 f2-648.227）。
const EXCLUDED_WHAT: [&str; 2] = ["人の作業の時間（「60 分以内」）", "AI の費用（「300k token 以下」）"];

/// 床の定数の木（値は実の rules.yaml の schema 節の字面と 1 字も違わない・足したのは `_note` で終わる欄だけ）。
/// `_note` で終わる欄は人が読む説明の注で、`folio schema` の導出だけが使う。
pub(crate) const FLOOR: Floor = Floor::Map(&[
    ("version", Floor::Num(1)),
    ("top_level", Floor::Strs(&RULES_TOP_LEVEL)),
    (
        "top_level_note",
        Floor::Val(
            "最上位の節の閉じた一覧（ほかの節は床が落とす）。thresholds と discipline は人が書き、schema は生成区間",
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
        "refs_note",
        Floor::Val(
            "その行の散文が依っている、article の条以外の id の一覧（ほかの行・要件書の id・判断の記録・別の条と規範文）。各項は id の形（P-5.2）で、その行自身の id と article の値は書かない。未解決は行 R-4 の 1 つ目の数えが拾い、判断の記録の未実在は A-2 の網が拾う（R-4 の what と値と母集団は変えない）",
        ),
    ),
    (
        "enums",
        Floor::Map(&[
            ("kind", Floor::Strs(&RuleKind::NAMES)),
            ("status", Floor::Strs(&RuleStatus::NAMES)),
            ("stage", Floor::Strs(&Stage::NAMES)),
            ("key", Floor::Strs(&KEYS)),
        ]),
    ),
    (
        "enums_note",
        Floor::Val("stage は憲法の値域 stage と同じ値（実装は憲法から導出した名の列を使う）"),
    ),
    (
        "key_note",
        Floor::Val(
            "閾値の行の欄 key は、道具がその行の値を読む口の名（閉じた一覧 enums.key）。行の id は置き場ごとに意味が違うので、道具は値を読む行を id でなくこの欄で引く。同じ値の閾値の行は置き場に 1 本まで。行が無いときの扱いは key ごとに違う。note-chapters（値 = 設計ノートの章の上限「<正の整数> 章 以下」）は、行が無いか 2 本以上在るか値の形が違えば、章の上限の判定が まだ分からない（道具は既定の値を持たない）。plan-note（値 = 計画のノートの文書 id・種別 build-check・段 post・判断の記録 ADR-31 決定 (2)(ア)）は、行が無ければ計画のノートの床を掛けない（行の索引と計画だけの行の節を名札の行が名指すノートの外に置けない決まりは、行が無くても掛かる）。2 本以上在るか値が文書 id の形でなければ、計画のノートの判定が まだ分からない。in-loop-min（値 = 極性一覧の段が in-loop の仕掛けの本数の下限「<正の整数> 本以上」・判断の記録 ADR-33 決定 (5)）は、行が無ければ下限を数えず床の標準エラーに 1 行出す。2 本以上在るか値の形が違えば、下限の判定が まだ分からない",
        ),
    ),
    (
        "kind_meaning",
        Floor::Map(&[
            (
                RuleKind::Deny.name(),
                Floor::Val("機械が測って、値域の外なら落とす（上限の超過・下限の不足・固定の値との違い。憲法の reject / build-check に対応）"),
            ),
            (
                RuleKind::BuildCheck.name(),
                Floor::Val("生成時の検査で数え、違反なら落とす（憲法の build-check に対応）"),
            ),
            (
                RuleKind::Detect.name(),
                Floor::Val("記録・起票のみ・止めない（憲法の none に対応）"),
            ),
            (
                RuleKind::HumanReview.name(),
                Floor::Val("人が守る作法（憲法の human-review に対応・D 行）"),
            ),
        ]),
    ),
    (
        "kind_map_to_constitution",
        Floor::Map(&[
            (RuleKind::Deny.name(), Floor::Strs(&KIND_DENY_MAPS_TO)),
            (
                RuleKind::BuildCheck.name(),
                Floor::Strs(&KIND_BUILD_CHECK_MAPS_TO),
            ),
            (RuleKind::Detect.name(), Floor::Strs(&KIND_DETECT_MAPS_TO)),
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
                    "人の作業の時間と AI の費用は測る仕組みが別で凍結できない。M1 の実地試験（非エンジニア 1 回）で初めて数値を決める（要件書 not_frozen）。道具が決定的に測れる機械の待ち時間は除外に当たらず、規則の表の行に載せてよい。",
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
        assert_eq!(crate::floor::derive(&FLOOR), anchor);
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

    /// 便 179 (c)5: 章の上限は欄 key が note-chapters の閾値の行の value「<正の整数> 章 以下」だけを読み、行の id に依らない。
    /// 行が無い（欄 key の無い行・開発規律の行に在る）・2 本・値の形が違う は Err（既定の値に倒れない）。
    #[test]
    fn f179_chapter_cap_reads_the_keyed_row_and_only_the_positive_integer_form() {
        let doc = |rows: &str| crate::yaml::parse(&format!("thresholds:\n{rows}discipline: []\n")).unwrap().root;
        let row = |id: &str, v: &str| format!("  - {{id: {id}, value: \"{v}\", key: note-chapters}}\n");
        assert_eq!(chapter_cap(&doc(&row("R-19", "12 章 以下"))), Ok(12));
        assert_eq!(chapter_cap(&doc(&row("R-26", "40 章 以下"))), Ok(40));
        for bad in ["12章以下", "12 章", "12 章 以上", "0 章 以下", "012 章 以下", "１２ 章 以下", "-1 章 以下", " 章 以下", "99999999999999999999999 章 以下"] {
            let e = chapter_cap(&doc(&row("R-19", bad))).unwrap_err();
            assert!(e.contains("R-19") && e.contains("の形でない"), "{bad}: {e}");
        }
        let none = chapter_cap(&doc("  - {id: R-19, value: \"12 章 以下\"}\n")).unwrap_err();
        assert!(none.contains("note-chapters の閾値の行が無い"), "{none}");
        let two = chapter_cap(&doc(&format!("{}{}", row("R-19", "12 章 以下"), row("R-26", "40 章 以下"))));
        assert!(two.unwrap_err().contains("2 本ある"));
        let discipline = "thresholds: []\ndiscipline:\n  - {id: D-1, value: \"12 章 以下\", key: note-chapters}\n";
        let e = chapter_cap(&crate::yaml::parse(discipline).unwrap().root).unwrap_err();
        assert!(e.contains("行が無い"), "{e}");
    }

    /// 便 179 (c)5: 欄 key の床は、閉じた一覧に無い値と、同じ値を持つ 2 本目の閾値の行を字にし、1 本だけなら何も出さない。
    #[test]
    fn f179_key_is_a_closed_list_and_one_threshold_row_per_key() {
        let v = |rows: &str| key_violations(&crate::yaml::parse(&format!("thresholds:\n{rows}")).unwrap().root);
        assert!(v("  - {id: R-19, key: note-chapters}\n  - {id: R-1}\n").is_empty());
        let typo = v("  - {id: R-19, key: note-chapter}\n");
        assert_eq!(typo.len(), 1, "{typo:?}");
        assert!(typo[0].contains("行 R-19 の key「note-chapter」が閉じた一覧"), "{typo:?}");
        let listed = v("  - {id: R-19, key: [note-chapters]}\n");
        assert_eq!(listed.len(), 1, "{listed:?}");
        let two = v("  - {id: R-19, key: note-chapters}\n  - {id: R-26, key: note-chapters}\n");
        assert_eq!(two, ["行 R-26 の key「note-chapters」を持つ閾値の行が 2 本以上ある"]);
        assert_eq!(KEYS, ["note-chapters", "plan-note", "in-loop-min"]);
    }

    /// 便 183 (c): 計画のノートの文書 id は欄 key が plan-note の閾値の行の value だけを読む。行が無ければ None（掛けない）、
    /// 2 本・値が文書 id の形でない・開発規律の行に在る（閾値の行でない＝無いと同じ）を分ける。
    #[test]
    fn f183_plan_note_reads_the_keyed_row_and_none_means_off() {
        let doc = |rows: &str| crate::yaml::parse(&format!("thresholds:\n{rows}discipline: []\n")).unwrap().root;
        let row = |id: &str, v: &str| format!("  - {{id: {id}, value: \"{v}\", key: plan-note}}\n");
        assert_eq!(plan_note(&doc(&row("R-27", "surface-plan"))), Ok(Some("surface-plan")));
        assert_eq!(plan_note(&doc("  - {id: R-19, value: \"12 章 以下\", key: note-chapters}\n")), Ok(None));
        for bad in ["Surface-plan", "surface_plan", "", "1plan", "surface-plan.yaml", "計画"] {
            let e = plan_note(&doc(&row("R-27", bad))).unwrap_err();
            assert!(e.contains("R-27") && e.contains("文書 id の形でない"), "{bad}: {e}");
        }
        let two = doc(&format!("{}{}", row("R-27", "a"), row("R-28", "b")));
        assert!(plan_note(&two).unwrap_err().contains("2 本ある"));
        let discipline = "thresholds: []\ndiscipline:\n  - {id: D-1, value: surface-plan, key: plan-note}\n";
        assert_eq!(plan_note(&crate::yaml::parse(discipline).unwrap().root), Ok(None));
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

    /// 違反の名札（便 203）: folio2 の置き場（外でない）は名札のまま、外の置き場は一覧の 9 つの id を検査の名にする（置き場の表を
    /// 見ない＝同じ id を持つ置き場でも検査の名・期待の字は手書き）。一覧に無い名札（行を引く R-17 と検査の名）は変えない。
    #[test]
    fn f203_labels_name_the_check_abroad_even_where_the_place_has_the_id() {
        let home = Labels { abroad: false };
        let abroad = Labels { abroad: true };
        for (id, name) in [
            ("A-2", "改訂と判断の記録"),
            ("N-4", "改訂の承認"),
            ("P-7", "id の再利用と改番"),
            ("P-7.1", "id の再利用と改番"),
            ("P-8", "撤退条件"),
            ("R-3", "部品目録"),
            ("R-9", "語彙"),
            ("R-10", "平易文"),
            ("R-11", "強度と文末"),
        ] {
            assert_eq!((home.shown(id), abroad.shown(id)), (id, name));
        }
        for kind in ["adr", "schema", "R-8", "R-16", "R-17", "P-18", "polarity"] {
            assert_eq!((home.shown(kind), abroad.shown(kind)), (kind, kind));
        }
    }
}
