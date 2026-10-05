//! 出所の帯（見本の ui.js の BANDS・BAND_PATH・BEADS_LANES）と、節点の種類から帯と語の鍵への閉じた表。
//! 種類の対応はこの file の表（`KINDS`）の 1 か所だけに置き、ほかの file は関数で引く。
//! URL の query に残す種類の名は語の鍵の「k:」の後の字（見本の地図の種類の名と同じ）。

use tsuzuri_contract::graph::{GraphSource, NodeKind};

/// 出所の帯（閉じた 7・順は憲法の順位と同じ）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Band {
    Constitution,
    Rules,
    Adr,
    Srs,
    DesignNote,
    Beads,
    Pipeline,
}

impl Band {
    /// 閉じた一覧（順も固定）。
    pub const ALL: [Band; 7] = [
        Band::Constitution,
        Band::Rules,
        Band::Adr,
        Band::Srs,
        Band::DesignNote,
        Band::Beads,
        Band::Pipeline,
    ];

    /// 帯の名（見本の BANDS の字・URL の query の band の値）。
    pub fn name(self) -> &'static str {
        match self {
            Band::Constitution => "constitution",
            Band::Rules => "rules",
            Band::Adr => "ADR",
            Band::Srs => "SRS",
            Band::DesignNote => "design-note",
            Band::Beads => "beads",
            Band::Pipeline => "pipeline",
        }
    }

    /// 帯の class（`band-` に帯の名の小文字をつないだ名・見本の bandCls）。
    pub fn class_name(self) -> &'static str {
        match self {
            Band::Constitution => "band-constitution",
            Band::Rules => "band-rules",
            Band::Adr => "band-adr",
            Band::Srs => "band-srs",
            Band::DesignNote => "band-design-note",
            Band::Beads => "band-beads",
            Band::Pipeline => "band-pipeline",
        }
    }

    /// 帯の語の鍵（「b:」に帯の名）。
    pub fn key(self) -> &'static str {
        match self {
            Band::Constitution => "b:constitution",
            Band::Rules => "b:rules",
            Band::Adr => "b:ADR",
            Band::Srs => "b:SRS",
            Band::DesignNote => "b:design-note",
            Band::Beads => "b:beads",
            Band::Pipeline => "b:pipeline",
        }
    }

    /// 出所の path（見本の BAND_PATH の字）。
    pub fn path(self) -> &'static str {
        match self {
            Band::Constitution => "design-intent/constitution.yaml",
            Band::Rules => "design-intent/rules.yaml",
            Band::Adr => "design-intent/adr/ADR-n.yaml",
            Band::Srs => "design-intent/srs.yaml",
            Band::DesignNote => "design-intent/design-note/*.yaml・docs/design/*.md",
            Band::Beads => ".beads/issues.jsonl",
            Band::Pipeline => "<state dir>/fleet/events.jsonl",
        }
    }

    /// 帯の節点を組む出所（設計の索引が 5 帯・台帳が beads・器の event の記録が pipeline）。
    pub fn source(self) -> GraphSource {
        match self {
            Band::Beads => GraphSource::Ledger,
            Band::Pipeline => GraphSource::Runs,
            _ => GraphSource::Design,
        }
    }

    /// 帯の名から帯（知らない名は None）。
    pub fn from_name(name: &str) -> Option<Band> {
        Band::ALL.into_iter().find(|b| b.name() == name)
    }

    /// 帯の chip の class（見本の bandChip）。
    pub fn chip_class(self) -> String {
        format!("bchip {}", self.class_name())
    }
}

/// 種類の表の 1 行（種類・帯・語の鍵）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KindRow {
    pub kind: NodeKind,
    pub band: Band,
    pub key: &'static str,
}

const fn kind_row(kind: NodeKind, band: Band, key: &'static str) -> KindRow {
    KindRow { kind, band, key }
}

/// 節点の種類から帯と語の鍵への閉じた表（21 行・順は契約の型の `NodeKind::ALL`）。
pub const KINDS: [KindRow; 21] = [
    kind_row(NodeKind::Article, Band::Constitution, "k:条"),
    kind_row(NodeKind::Norm, Band::Constitution, "k:規範文"),
    kind_row(NodeKind::Rule, Band::Rules, "k:規則行"),
    kind_row(NodeKind::Goal, Band::Srs, "k:目的"),
    kind_row(NodeKind::Req, Band::Srs, "k:要件"),
    kind_row(NodeKind::Nfr, Band::Srs, "k:非機能要件"),
    kind_row(NodeKind::Ac, Band::Srs, "k:受入基準"),
    kind_row(NodeKind::Constraint, Band::Srs, "k:制約"),
    kind_row(NodeKind::Actor, Band::Srs, "k:登場人物"),
    kind_row(NodeKind::Output, Band::Srs, "k:出力"),
    kind_row(NodeKind::Adr, Band::Adr, "k:判断の記録"),
    kind_row(NodeKind::NoteRow, Band::DesignNote, "k:設計ノートの行"),
    kind_row(NodeKind::Epic, Band::Beads, "k:epic"),
    kind_row(NodeKind::Task, Band::Beads, "k:契約"),
    kind_row(NodeKind::Memo, Band::Beads, "k:memo"),
    kind_row(NodeKind::Question, Band::Beads, "k:問い"),
    kind_row(NodeKind::Ruling, Band::Beads, "k:裁定"),
    kind_row(NodeKind::Receipt, Band::Beads, "k:受け"),
    kind_row(NodeKind::Policy, Band::Beads, "k:方針"),
    kind_row(NodeKind::Run, Band::Pipeline, "k:走行"),
    kind_row(NodeKind::Commit, Band::Pipeline, "k:commit"),
];

/// 種類の語の鍵の頭。
pub const KIND_KEY_PREFIX: &str = "k:";

#[expect(clippy::expect_used, reason = "種類の表は種類の 21 個の全部を持つ")]
fn row(kind: NodeKind) -> &'static KindRow {
    KINDS
        .iter()
        .find(|r| r.kind == kind)
        .expect("種類の表は種類の 21 個の全部を持つ")
}

/// 種類の帯。
pub fn band_of(kind: NodeKind) -> Band {
    row(kind).band
}

/// 種類の語の鍵。
pub fn kind_key(kind: NodeKind) -> &'static str {
    row(kind).key
}

/// 種類の名（語の鍵の「k:」の後の字・URL の query の kind と pair の値）。
pub fn kind_name(kind: NodeKind) -> &'static str {
    let key = kind_key(kind);
    key.strip_prefix(KIND_KEY_PREFIX).unwrap_or(key)
}

/// 種類の名から種類（知らない名は None）。
pub fn kind_from_name(name: &str) -> Option<NodeKind> {
    KINDS.iter().map(|r| r.kind).find(|k| kind_name(*k) == name)
}

/// beads の帯の種類の行（見本の BEADS_LANES と同じ順）。
pub const BEADS_LANES: [NodeKind; 7] = [
    NodeKind::Epic,
    NodeKind::Task,
    NodeKind::Memo,
    NodeKind::Question,
    NodeKind::Ruling,
    NodeKind::Receipt,
    NodeKind::Policy,
];
