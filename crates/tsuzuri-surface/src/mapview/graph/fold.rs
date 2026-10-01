//! グラフの組の箱の開き閉じ（行 g-graph-fold・host でも組む）。
//! 箱の開き閉じの語の鍵と、組の箱の持ち主に読める題（節点の card が引く）。
//! 選ぶ・畳む・上限で畳み直すのは中核の crate が済ませ、ここは電文の group・fold を写すだけ。
//! 開いた箱の列と口の path・右下の印・組の箱の 1 行目・開けなかった id の行は行 m-map-graph で消した。

use tsuzuri_contract::graph::{BoxFold, NodeKind, ViewNode};

use crate::mapview::band::{band_of, kind_name};
use crate::vocab::label;

/// 箱の開き閉じの語の鍵（Folded は開く・Open は畳み直す・Leaf は無し）。
pub fn fold_key(fold: BoxFold) -> Option<&'static str> {
    match fold {
        BoxFold::Leaf => None,
        BoxFold::Folded => Some("gf_unfold"),
        BoxFold::Open => Some("gf_fold"),
    }
}

/// 箱の持ち主に読める題（組でない箱は電文の題・組の箱は id と種類から組み、~ の id を字にしない・行 g-graph-label）。
pub fn plain_title(n: &ViewNode) -> String {
    if n.group {
        group_title(&n.node.id, n.node.kind).unwrap_or_else(|| n.node.title.clone())
    } else {
        n.node.title.clone()
    }
}

/// 組の箱の id と種類から題（~ で始まらない組でない id は None）。
/// 塊は親の題と覆う子の番号の範囲・帯は帯の見出しの語・条の系は種類の名と系・ノートは id の行・ほかは種類の名。
fn group_title(id: &str, kind: NodeKind) -> Option<String> {
    if let Some((parent, range)) = id.rsplit_once('~')
        && let Some((a, b)) = range.split_once('-')
        && !parent.is_empty()
        && [a, b].iter().all(|s| !s.is_empty() && s.bytes().all(|c| c.is_ascii_digit()))
    {
        let head = if parent.starts_with('~') {
            group_title(parent, kind).unwrap_or_else(|| parent.to_string())
        } else {
            parent.to_string()
        };
        return Some(format!("{head} の {a}〜{b}"));
    }
    let rest = id.strip_prefix('~')?;
    Some(if rest.starts_with("b:") {
        label(band_of(kind).key())
    } else if let Some(sys) = rest.strip_prefix("art:") {
        format!("{} {sys}", kind_name(NodeKind::Article))
    } else if let Some(note) = rest.strip_prefix("note:") {
        format!("{note} の行")
    } else {
        kind_name(kind).to_string()
    })
}
