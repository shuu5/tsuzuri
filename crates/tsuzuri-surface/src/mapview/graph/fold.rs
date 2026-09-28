//! グラフの組の箱の開き閉じ（行 g-graph-fold・host でも組む）。
//! 開いた箱の id の列（開いた順）と口の query の path・箱の右下の開き閉じの印・組の箱の 1 行目・開けなかった id の行。
//! 選ぶ・畳む・上限で畳み直すのは中核の crate が済ませ、ここは電文の group・fold・open・refused を写すだけ。

use tsuzuri_contract::graph::{BoxFold, GraphView, NodeKind, ViewNode, title36};

use super::{NODE_H, NODE_W, PATH, Pos, cut, esc};
use crate::mapview::band::{band_of, kind_key, kind_name};
use crate::mapview::encode;
use crate::vocab::label;

/// 開いた箱の id の列（開いた順・頁の間だけ持つ）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OpenList {
    ids: Vec<String>,
}

impl OpenList {
    pub fn ids(&self) -> &[String] {
        &self.ids
    }

    /// 箱を押した（Folded は列の末へ移す・Open は列から外す・Leaf は変えない・列を変えたときは真）。
    pub fn toggle(&mut self, id: &str, fold: BoxFold) -> bool {
        let before = self.ids.clone();
        match fold {
            BoxFold::Leaf => return false,
            BoxFold::Folded => {
                self.ids.retain(|x| x != id);
                self.ids.push(id.to_string());
            }
            BoxFold::Open => self.ids.retain(|x| x != id),
        }
        self.ids != before
    }

    /// 列を server の列に合わせる（畳み直された id と開けなかった id が外れる）。
    pub fn adopt(&mut self, ids: &[String]) {
        self.ids = ids.to_vec();
    }

    /// 口の path（列が空なら PATH・在れば query の open に各 id を encode して字 , でつなぐ）。
    pub fn path(&self) -> String {
        if self.ids.is_empty() {
            return PATH.to_string();
        }
        let open: Vec<String> = self.ids.iter().map(|id| encode(id)).collect();
        format!("{PATH}?open={}", open.join(","))
    }
}

/// 箱の開き閉じの語の鍵（Folded は開く・Open は畳み直す・Leaf は無し）。
pub fn fold_key(fold: BoxFold) -> Option<&'static str> {
    match fold {
        BoxFold::Leaf => None,
        BoxFold::Folded => Some("gf_unfold"),
        BoxFold::Open => Some("gf_fold"),
    }
}

/// 箱の右下の開き閉じの印（Leaf は None・Folded は ▸・Open は ▾・押すと開き閉じする button）。
pub fn mark_svg(n: &ViewNode, p: Pos) -> Option<String> {
    let key = fold_key(n.fold)?;
    let glyph = if n.fold == BoxFold::Open { "▾" } else { "▸" };
    let (x, y) = (f64::from(p.x + NODE_W) - 17.0, f64::from(p.y + NODE_H) - 15.0);
    Some(format!(
        r#"<g class="fold" data-fold="{id}" role="button" tabindex="0" aria-label="{aria}"><rect x="{x}" y="{y}" width="13" height="12" rx="2" fill="var(--panel-2)" stroke="var(--line-2)"/><text x="{}" y="{}" font-size="10" text-anchor="middle" fill="var(--ink-2)">{glyph}</text></g>"#,
        x + 6.5,
        y + 9.5,
        id = esc(&n.node.id),
        aria = esc(&label(key)),
    ))
}

/// 組の箱の 1 行目の字数。
pub const GROUP_TITLE_CHARS: usize = 20;

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

/// 組の箱の 1 行目（題・id を字にしない）と 2 行目（種類の語 · 組）と aria-label の字。
pub fn group_lines(n: &ViewNode, x: f64, y: f64, max: f64, fill: &str) -> (String, String, String) {
    let plain = plain_title(n);
    let title = cut(&title36(&plain), GROUP_TITLE_CHARS);
    let est: f64 = title
        .chars()
        .map(|c| if c.is_ascii() { 6.4 } else { 11.0 })
        .sum();
    let kind = label(kind_key(n.node.kind));
    let group = label("gf_group");
    let head = format!(
        r#"<text x="{}" y="{}" font-size="11" font-weight="700" fill="{fill}" textLength="{:.1}" lengthAdjust="spacingAndGlyphs">{}</text>"#,
        x + 21.0,
        y + 16.5,
        max.min(est),
        esc(&title)
    );
    let aria = format!("{plain} {kind} {group}");
    (aria, head, format!("{kind} · {group}"))
}

/// 開けなかった id の行（眺めの refused が空なら None・同じ id の組の箱が在れば題・無ければ id のまま）。
pub fn refused_line(view: &GraphView) -> Option<String> {
    let names: Vec<String> = view
        .refused
        .iter()
        .map(|id| {
            view.nodes
                .iter()
                .find(|n| n.group && n.node.id == *id)
                .map_or_else(|| id.clone(), plain_title)
        })
        .collect();
    (!names.is_empty()).then(|| format!("{} {}", label("gf_refused"), names.join("・")))
}

/// 凡例の 1 行目の組の見本（点線の縁と ▸）。
pub const LEGEND_GROUP: &str = r#"<svg viewBox="0 0 30 16" class="lk wide" aria-hidden="true"><rect x="1.5" y="1.5" width="27" height="13" rx="4" fill="var(--panel)" stroke="var(--line-2)" stroke-dasharray="4 2"/><text x="22" y="12" font-size="10" text-anchor="middle" fill="var(--ink-2)">▸</text></svg>"#;
