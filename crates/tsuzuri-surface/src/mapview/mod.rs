//! 地図の面の部品（便 g-map・見本の map.html）と、面に共通の小道具。地図の頁と 6 面の切り替えは行 m-map-page で消した。
//! 電文（契約の型の GraphDoc）の読みは kit の下の map（crate::project::map の path）が持ち、ここの関数は読めた電文だけを受ける。
//! 帯と種類の対応は band・近傍の図は around・近傍と節点の card が引くグラフの部品は graph。
//! 圧縮の面と一覧の面は行 m-map-compact で、表の面と 2 つの木の面は行 m-map-tree で、グラフの面の DOM は行 m-map-graph で消した。
//! 並べ方と絞りと数え方と URL の query は純粋な関数にして host で試し、DOM は wasm の target のときだけ組み立てる。

pub mod around;
pub mod band;
pub mod graph;

use std::cmp::Ordering;
use std::fmt::Write;

use tsuzuri_contract::graph::{GraphDoc, GraphNode, NodeKind};

use band::{Band, band_of};

/// 節点の状態の字（bead は属性の状態・走行は属性の段・設計文書の節点と属性の無い節点は None）。
pub fn state<'a>(doc: &'a GraphDoc, node: &GraphNode) -> Option<&'a str> {
    match band_of(node.kind) {
        Band::Beads => doc.beads.get(&node.id).map(|b| b.status.as_str()),
        Band::Pipeline => doc.runs.get(&node.id).and_then(|r| r.stage.as_deref()),
        _ => None,
    }
}

/// 動いている状態（open と in_progress）。
pub fn is_open(state: Option<&str>) -> bool {
    matches!(state, Some("open" | "in_progress"))
}

/// open の問い（印は赤・見本の isOpenQ）。
pub fn open_question(doc: &GraphDoc, node: &GraphNode) -> bool {
    node.kind == NodeKind::Question && state(doc, node) == Some("open")
}

/// 節点の印の class（形は帯の色・動いている節点は塗らない・見本の nodeShape）。
pub fn shape_class(doc: &GraphDoc, node: &GraphNode) -> String {
    let open = is_open(state(doc, node));
    format!(
        "shape {}{}",
        band_of(node.kind).class_name(),
        if open { "" } else { " fill" }
    )
}

/// id の自然な順（字の中の数の並びは数として比べる: `ADR-2` は `ADR-10` より前・同じなら字の順）。
pub fn natural(a: &str, b: &str) -> Ordering {
    let mut xs = a.chars().peekable();
    let mut ys = b.chars().peekable();
    loop {
        match (xs.peek().copied(), ys.peek().copied()) {
            (None, None) => return a.cmp(b),
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(x), Some(y)) if x.is_ascii_digit() && y.is_ascii_digit() => {
                let m = digits(&mut xs);
                let n = digits(&mut ys);
                let o = number_order(&m, &n);
                if o != Ordering::Equal {
                    return o;
                }
            }
            (Some(x), Some(y)) => {
                if x != y {
                    return x.cmp(&y);
                }
                xs.next();
                ys.next();
            }
        }
    }
}

fn digits(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> String {
    let mut out = String::new();
    while let Some(c) = chars.peek().copied().filter(char::is_ascii_digit) {
        out.push(c);
        chars.next();
    }
    out
}

/// 数の並びの大小（頭の 0 を除いた桁の数、次に字の順）。
fn number_order(m: &str, n: &str) -> Ordering {
    let m = m.trim_start_matches('0');
    let n = n.trim_start_matches('0');
    m.len().cmp(&n.len()).then_with(|| m.cmp(n))
}

/// query の 1 つの値（`%XX` と `+` を戻した字・無ければ None）。
pub fn param(search: &str, key: &str) -> Option<String> {
    search
        .trim_start_matches('?')
        .split('&')
        .filter(|kv| !kv.is_empty())
        .map(|kv| kv.split_once('=').unwrap_or((kv, "")))
        .find(|(k, _)| decode(k) == key)
        .map(|(_, v)| decode(v))
}

/// query の 1 つの値を置き換える（None か空の値は消す・無ければ末尾に足す・ほかの値と順はそのまま）。
pub fn set_param(search: &str, key: &str, value: Option<&str>) -> String {
    let value = value.filter(|v| !v.is_empty());
    let mut placed = false;
    let mut parts: Vec<String> = Vec::new();
    for kv in search.trim_start_matches('?').split('&') {
        if kv.is_empty() {
            continue;
        }
        let k = kv.split_once('=').map_or(kv, |(k, _)| k);
        if decode(k) != key {
            parts.push(kv.to_string());
        } else if !placed {
            placed = true;
            if let Some(v) = value {
                parts.push(format!("{key}={}", encode(v)));
            }
        }
    }
    if !placed && let Some(v) = value {
        parts.push(format!("{key}={}", encode(v)));
    }
    format!("?{}", parts.join("&"))
}

/// query の値の字を `%XX` にする（英数字と `-` `_` `.` `~` のほかは全部）。
pub fn encode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(char::from(b));
        } else {
            let _ = write!(out, "%{b:02X}");
        }
    }
    out
}

/// query の値の `%XX` と `+` を戻す（読めない `%` はそのまま・UTF-8 でない並びは置き換える）。
pub fn decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while let Some(&c) = bytes.get(i) {
        let hex = (c == b'%')
            .then(|| bytes.get(i + 1..i + 3))
            .flatten()
            .and_then(|h| std::str::from_utf8(h).ok())
            .and_then(|h| u8::from_str_radix(h, 16).ok());
        match (hex, c) {
            (Some(b), _) => {
                out.push(b);
                i += 3;
            }
            (None, b'+') => {
                out.push(b' ');
                i += 1;
            }
            (None, b) => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(target_arch = "wasm32")]
pub use dom::{band_chip, current, navigate};

/// 面に共通の DOM の部品（wasm の target のときだけ）。
#[cfg(target_arch = "wasm32")]
mod dom {
    use leptos::prelude::*;

    use super::band::Band;
    use crate::vocab::label;

    /// 今の URL の query（読めなければ空）。
    pub fn current() -> String {
        window().location().search().unwrap_or_default()
    }

    /// 今の URL の query を変えて履歴に残し（`push` なら積む・でなければ置き換える）、面の signal に置く。
    /// 変える元は signal でなく今の URL（mode の切り替えが URL を書き換えても消さない）。
    pub fn navigate(search: RwSignal<String>, change: impl FnOnce(&str) -> String, push: bool) {
        let url = change(&current());
        if let Ok(history) = window().history() {
            let null = web_sys::wasm_bindgen::JsValue::NULL;
            let _ = if push {
                history.push_state_with_url(&null, "", Some(&url))
            } else {
                history.replace_state_with_url(&null, "", Some(&url))
            };
        }
        search.set(url);
    }

    /// 帯の chip（見本の bandChip）。
    pub fn band_chip(band: Band) -> AnyView {
        view! {
            <span class=band.chip_class() data-term=band.key() tabindex="0"><i></i>{label(band.key())}</span>
        }
        .into_any()
    }
}
