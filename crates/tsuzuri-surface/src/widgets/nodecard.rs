//! 節点の card（見本の ui.js の cardContent の節点の枝・行 g-card-node）: 電文の節点 1 つから hover の card の 4 行と詳しくを組む。
//! 題・種類と帯と状態の行・id と要約の行・出所の file の行（36 字の切りは hover の Card の rows に任せる）。
//! 要約は電文の節点の概要（plain・無ければ eng）を写して切り、無ければ「要約なし」（行 g-summary）。
//! 出所は節点の file と行（無ければ帯の path）を短くし、長い概要を折った行と出所の全部の字は詳しくに置く。
//! 地図の面の札と行が同じ id と電文から同じ card を引けるように、id から引く `card_of` も置く（host でも組む）。
//! card の中身は節点と状態の字から組む `card_for` 1 つで、近傍の図と一覧は眺めの節点から `view_cards` で引く（行 g-card-around）。

use std::collections::BTreeMap;

use tsuzuri_contract::graph::{GraphDoc, GraphNode, ViewNode};

use crate::mapview::band::{band_of, kind_key};
use crate::mapview::graph::cut;
use crate::mapview::graph::fold::{fold_key, plain_title};
use crate::mapview::list::{NO_GIST, NO_STATE};
use crate::mapview::state;
use crate::vocab::label;
use crate::widgets::hover::{Card, ELLIPSIS};

/// file は在るが行の番号が無いときの、出所の全部の字の後ろの字（見本の srcText）。
pub const NO_LINE: &str = "（行は測れていない）";

/// card の値の行と詳しくの折りの字数（見本の cardContent の 34）。
const CARD_CHARS: usize = 34;

/// 値の行に概要を置く最小の字数（見本の room が 6 以上）。
const MIN_ROOM: usize = 6;

/// 概要を詳しくに折って置く字数の閾（これを越えれば折る）。
const FOLD_OVER: usize = 20;

/// 節点の概要の字（plain が空でなければ plain・そうでなく eng が空でなければ eng・どちらでもなければ None）。
pub fn gist(node: &GraphNode) -> Option<&str> {
    [&node.plain, &node.eng]
        .into_iter()
        .find_map(|s| s.as_deref().filter(|s| !s.is_empty()))
}

/// 出所の全部の字（file が在れば file とコロンと行か file と NO_LINE・無ければ帯の path・見本の srcText）。
pub fn full_src(node: &GraphNode) -> String {
    match (&node.file, node.line) {
        (Some(file), Some(n)) => format!("{file}:{n}"),
        (Some(file), None) => format!("{file}{NO_LINE}"),
        (None, _) => band_of(node.kind).path().to_string(),
    }
}

/// 字を 34 字以下の行に折る（、。・，）の後で片に分け、片が溢れる前で行を替える・見本の chunk）。
pub fn fold_rows(s: &str) -> Vec<String> {
    let mut pieces: Vec<String> = Vec::new();
    let mut piece = String::new();
    for c in s.chars() {
        piece.push(c);
        if matches!(c, '、' | '。' | '・' | '，' | '）') {
            pieces.push(std::mem::take(&mut piece));
        }
    }
    if !piece.is_empty() {
        pieces.push(piece);
    }
    let mut rows = Vec::new();
    let mut line: Vec<char> = Vec::new();
    for p in pieces {
        let mut p: Vec<char> = p.chars().collect();
        if line.len() + p.len() > CARD_CHARS {
            if !line.is_empty() {
                rows.push(line.drain(..).collect());
            }
            while p.len() > CARD_CHARS {
                rows.push(p.drain(..CARD_CHARS).collect());
            }
        }
        line.extend(p);
    }
    if !line.is_empty() {
        rows.push(line.into_iter().collect());
    }
    rows
}

/// 出所の path を短くする（最初の「・」より前の字の、斜線で区切った片が 3 つ以上なら末尾の 2 片・見本の baseName）。
pub fn short_path(path: &str) -> String {
    let head = path.split_once('・').map_or(path, |(head, _)| head);
    let parts: Vec<&str> = head.split('/').collect();
    if parts.len() >= 3 {
        format!("{ELLIPSIS}/{}", parts[parts.len() - 2..].join("/"))
    } else {
        head.to_string()
    }
}

/// 節点の card（電文の状態を引いて `card_for` に渡す）。
pub fn node_card(doc: &GraphDoc, node: &GraphNode) -> Card {
    card_for(node, state(doc, node))
}

/// 節点と状態の字の card（題・種類と帯と状態・id と要約・出所と行・詳しくは長い概要を折った行と出所の全部の字）。
/// 状態の字は判じずそのまま種類の行に置く（無ければ「状態なし」）。
pub fn card_for(node: &GraphNode, status: Option<&str>) -> Card {
    let band = band_of(node.kind);
    let title = node.title.split_whitespace().collect::<Vec<_>>().join(" ");
    let g = gist(node);
    let value = match g {
        None => format!("{} {NO_GIST}", node.id),
        Some(g) => {
            let id = cut(&node.id, CARD_CHARS);
            let room = CARD_CHARS as isize - node.id.chars().count() as isize - 1;
            if room >= MIN_ROOM as isize {
                format!("{id} {}", cut(g, room as usize))
            } else {
                id
            }
        }
    };
    let src = match (&node.file, node.line) {
        (Some(file), Some(n)) => format!("{}:{n}", short_path(file)),
        (Some(file), None) => short_path(file),
        (None, _) => short_path(band.path()),
    };
    let mut more = match g {
        Some(g) if g.chars().count() > FOLD_OVER => fold_rows(g),
        _ => Vec::new(),
    };
    let full = full_src(node);
    if full != src {
        more.push(full);
    }
    Card {
        title: if title.is_empty() {
            node.id.clone()
        } else {
            title
        },
        kind: format!(
            "{} · {} · {}",
            label(kind_key(node.kind)),
            band.name(),
            status.unwrap_or(NO_STATE)
        ),
        value,
        src,
        more,
    }
}

/// 組の箱の card（圧縮された cell・題・種類と帯と組・子の数・開き閉じの語・詳しくは無し）。
/// 箱の id（~ の字）と「要約なし」は出さない。
pub fn group_card(n: &ViewNode) -> Card {
    let kind = n.node.kind;
    Card {
        title: plain_title(n),
        kind: format!(
            "{}・{}・{}",
            label(kind_key(kind)),
            band_of(kind).name(),
            label("gf_group")
        ),
        value: format!("{} {}", label("children"), n.kids),
        src: fold_key(n.fold).map(label).unwrap_or_default(),
        more: Vec::new(),
    }
}

/// 眺めの節点の列の id ごとの card（同じ id が 2 つ在れば前の節点の値・組の箱は `group_card`・近傍の図と一覧が引く）。
pub fn view_cards(nodes: &[ViewNode]) -> BTreeMap<String, Card> {
    let mut cards = BTreeMap::new();
    for n in nodes {
        cards.entry(n.node.id.clone()).or_insert_with(|| {
            if n.group {
                group_card(n)
            } else {
                card_for(&n.node, n.status.as_deref())
            }
        });
    }
    cards
}

/// id の節点の card（電文の nodes の前から見て最初の同じ id・無ければ None）。
pub fn card_of(doc: &GraphDoc, id: &str) -> Option<Card> {
    doc.nodes
        .iter()
        .find(|n| n.id == id)
        .map(|n| node_card(doc, n))
}
