//! 節点の card（見本の ui.js の cardContent の節点の枝・行 g-card-node）: 電文の節点 1 つから hover の card の 4 行と詳しくを組む。
//! 題・種類と帯と状態の行・id と要約の行・出所の file の行（36 字の切りは hover の Card の rows に任せる）。
//! 要約は電文にまだ無いので「要約なし」・出所は節点の file（無ければ帯の path）を短くし、元の字は詳しくに置く。
//! 地図の面の札と行が同じ id と電文から同じ card を引けるように、id から引く `card_of` も置く（host でも組む）。

use tsuzuri_contract::graph::{GraphDoc, GraphNode};

use crate::mapview::band::{band_of, kind_key};
use crate::mapview::list::{NO_GIST, NO_STATE};
use crate::mapview::state;
use crate::vocab::label;
use crate::widgets::hover::{Card, ELLIPSIS};

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

/// 節点の card（題・種類と帯と状態・id と要約・出所・詳しくは出所の元の字）。
pub fn node_card(doc: &GraphDoc, node: &GraphNode) -> Card {
    let band = band_of(node.kind);
    let title = node.title.split_whitespace().collect::<Vec<_>>().join(" ");
    let raw = node
        .file
        .clone()
        .unwrap_or_else(|| band.path().to_string());
    let src = short_path(&raw);
    let more = if src == raw { Vec::new() } else { vec![raw] };
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
            state(doc, node).unwrap_or(NO_STATE)
        ),
        value: format!("{} {NO_GIST}", node.id),
        src,
        more,
    }
}

/// id の節点の card（電文の nodes の前から見て最初の同じ id・無ければ None）。
pub fn card_of(doc: &GraphDoc, id: &str) -> Option<Card> {
    doc.nodes
        .iter()
        .find(|n| n.id == id)
        .map(|n| node_card(doc, n))
}
