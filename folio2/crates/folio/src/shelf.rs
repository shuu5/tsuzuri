//! 入口の棚の閉じた一覧と設計ノートの状態の閉じた一覧・文書 id の形の置き場（便 109・docs/design/delivery-109.md §1 (b)・
//! ADR-15・層 1 読む）。棚の一覧は `face_labels.rs` から、状態の表と id の形は `face_note.rs` から字を変えずに降ろした。
//! 読むのは入口の正本の検査（`entrance.rs`）と面の生成器（`face.rs`・`face_index.rs`・`face_note.rs`）と図の命令（`figure.rs`）。

/// 入口の棚の文書（便 16）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shelf {
    /// 棚の置き場（見本の css の shelf-grid の class）
    pub place: &'static str,
    /// 読める面の file（面が無ければ None）
    pub face: Option<&'static str>,
    /// 原語の札
    pub en: &'static str,
    /// minimap でこの文書の前に置く区切り
    pub sep: &'static str,
}

/// 文書の id → 棚（並びは棚の置き場の順）。
pub const SHELF_DOCS: &[(&str, Shelf)] = &[
    (
        "constitution",
        Shelf {
            place: "shelf-c",
            face: Some("constitution.html"),
            en: "CONSTITUTION",
            sep: "→",
        },
    ),
    (
        "srs",
        Shelf {
            place: "shelf-s",
            face: Some("srs.html"),
            en: "SRS",
            sep: "→",
        },
    ),
    (
        "design-note",
        Shelf {
            place: "shelf-d",
            face: None,
            en: "DESIGN",
            sep: "→",
        },
    ),
    (
        "adr",
        Shelf {
            place: "shelf-adr",
            face: None,
            en: "ADR",
            sep: "｜",
        },
    ),
];

/// 関係の id → 棚の置き場（shelf-link の class）。
pub const SHELF_RELATIONS: &[(&str, &str)] = &[
    ("binds", "shelf-l1"),
    ("before-build", "shelf-l2"),
    ("inside", "shelf-branch branch"),
    ("amends", "up"),
];

/// 付録の id → （憲法の章の番号・数の単位）。
pub const ANNEXES: &[(&str, (u8, &str))] = &[("vocabulary", (7, "語")), ("rules", (5, "行"))];

/// 棚の凡例の id → sw の class。
pub const SHELF_LEGEND: &[(&str, &str)] = &[
    ("readable", "sw ok"),
    ("absent", "sw neutral"),
    ("binds", "sw line"),
    ("inside", "sw dash"),
];

/// 文書の状態 → 名札（状態の行は関数 status）。
pub(crate) const STATUS: &[(&str, &str)] = &[
    ("draft", "下書き"),
    ("effective", "発効"),
    ("retired", "廃止"),
    ("example", "見本"),
];

/// 文書 id の形（欄の決まり id_pattern = 英小文字で始まり 英小文字・数字・ハイフン）。
pub(crate) fn is_doc_id(s: &str) -> bool {
    let mut cs = s.chars();
    cs.next().is_some_and(|c| c.is_ascii_lowercase())
        && cs.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}
