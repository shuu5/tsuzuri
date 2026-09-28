//! 抜けの検査の頁（見本の gaps.html・便 g-gaps）: 不変条件の 12 本の判定を数の札（3 枚）と畳める段の一覧で見せる。
//! 判定は中核の crate が数え済みで、口（/api/graph・定数は map の module に 1 本）の電文 GraphDoc の invariants を写すだけにする。
//! 札の数・一覧の並び・段が最初に開いているか・名の表・20 件の切り方は純粋な関数にして host で試し、
//! DOM は wasm の target のときだけ、この値を順にたどって組み立てる。
//! 名指しの項のうち電文の節点に在るものは、節点の頁への link に節点の hover の card を付ける（行 g-card-adopt-c）。

use std::collections::BTreeMap;

use tsuzuri_contract::graph::{GraphDoc, InvariantCheck, Verdict};
use tsuzuri_contract::wire;

use super::{Body, Item, NOT_READ, node_item};
use crate::frame::Block;
use crate::view::Fetched;
use crate::widgets::hover::Card;
use crate::widgets::nodecard::card_of;

pub const BLOCK: Block = Block {
    id: "gaps",
    heading: "gaps",
    class: "panel",
};

/// この file が字を持つ口の path（無い・グラフの口は map の module が持つ・行 hs-derived）。
pub const PATHS: &[&str] = &[];

/// この file の畳める段の開き閉じの鍵の形（`{}` は不変条件の id・行 hs-derived）。
pub const FOLDS: &[&str] = &["gaps:{}"];

/// 口が読めないときの理由。
pub const REASON: &str =
    "不変条件の判定を運ぶ口が読めない（server にまだ無い・届かない・知らせが切れた）";

/// 本文が電文の型として読めないときの理由。
pub const UNREADABLE: &str = "導出グラフの本文が電文の型として読めない";

/// 電文に判定が 1 本も無いときの 1 行。
pub const EMPTY: &str = "電文に不変条件の判定が 1 本も無い";

/// 12 本の id と名（見本の字・本文の字なので語の辞書の鍵を使わない）。
pub const NAMES: [(&str, &str); 12] = [
    ("g-1", "線の両端が実在する"),
    ("g-2", "open の task は design-note の行を 1 つ指す"),
    ("g-3", "発効した ADR と rule に、あなたの決定が結ばれている"),
    ("g-4", "question は関わる所を 1 つ以上持つ"),
    ("g-5", "迷子の項目がない（全部が epic から辿れる）"),
    ("g-6", "blocks の線が輪になっていない"),
    ("g-7", "宙に浮いたあなたの決定がない"),
    ("g-8", "台帳の種類の約束を守っている"),
    ("g-9", "題で名指した項目に線が引かれている"),
    ("g-10", "番号の形が重ならない"),
    ("g-11", "run はどれも 1 つの task に属する"),
    ("g-12", "止まった run の question が記録にある"),
];

/// 名指しの一覧に出す件数の上限（超える分は残りの数の 1 行）。
pub const LIMIT: usize = 20;

/// 札と一覧の順（違反・まだ分からない・合格）。
pub const ORDER: [Verdict; 3] = [Verdict::Violation, Verdict::Unknown, Verdict::Pass];

/// 判定の記号（class `gi` に足す class・字・語の鍵）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mark {
    pub class: &'static str,
    pub glyph: &'static str,
    pub key: &'static str,
}

/// 判定の記号（見本の GI と同じ）。
pub fn mark(verdict: Verdict) -> Mark {
    match verdict {
        Verdict::Violation => Mark {
            class: "ng",
            glyph: "✕",
            key: "gap_ng",
        },
        Verdict::Unknown => Mark {
            class: "unknown",
            glyph: "?",
            key: "gap_unknown",
        },
        Verdict::Pass => Mark {
            class: "ok",
            glyph: "✓",
            key: "gap_ok",
        },
    }
}

/// 記号の class（見本の `.gi.<値>`）。
pub fn mark_class(mark: Mark) -> String {
    format!("gi {}", mark.class)
}

/// 1 本の名（表に無い id は id の字のまま）。
pub fn name(id: &str) -> &str {
    NAMES.iter().find(|(k, _)| *k == id).map_or(id, |(_, n)| n)
}

/// id の数（`g-10` の 10・数でなければ None）。
fn number(id: &str) -> Option<u64> {
    id.rsplit('-').next().and_then(|n| n.parse().ok())
}

/// 数の札の 1 枚（記号と数・語は記号の鍵）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tile {
    pub mark: Mark,
    pub count: usize,
}

/// 数の札の 3 枚（違反・まだ分からない・合格の順）。
pub fn tiles(checks: &[InvariantCheck]) -> Vec<Tile> {
    ORDER
        .iter()
        .map(|v| Tile {
            mark: mark(*v),
            count: checks.iter().filter(|c| c.verdict == *v).count(),
        })
        .collect()
}

/// 名指しの一覧（20 件まで）と残りの数（超えなければ None）。
pub fn cut(ids: &[String]) -> (Vec<String>, Option<usize>) {
    let shown = ids.iter().take(LIMIT).cloned().collect();
    let rest = ids.len().checked_sub(LIMIT).filter(|n| *n > 0);
    (shown, rest)
}

/// 一覧の 1 行（畳める段の見出しと中身）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub id: String,
    pub verdict: Verdict,
    pub mark: Mark,
    pub name: String,
    /// 違反の数（0 なら出さない）。
    pub count: Option<u32>,
    /// 段が最初に開いているか（違反の行だけ）。
    pub open: bool,
    pub named: Vec<String>,
    pub more: Option<usize>,
}

/// 判定の名（見本の gaps.html が経験者の字に使う名・電文の serde の名とは違反だけ違う）。
pub fn verdict_name(verdict: Verdict) -> &'static str {
    match verdict {
        Verdict::Pass => "pass",
        Verdict::Violation => "fail",
        Verdict::Unknown => "unknown",
    }
}

/// 各行の見出しの経験者だけの注釈（見本の `data-tip-expert` の字・id と判定の名）。
pub fn summary_tip(row: &Row) -> String {
    format!("{} · {}", row.id, verdict_name(row.verdict))
}

/// 一覧の行（違反・まだ分からない・合格の順・同じ判定の中は id の数の順）。
pub fn rows(checks: &[InvariantCheck]) -> Vec<Row> {
    let mut sorted: Vec<&InvariantCheck> = checks.iter().collect();
    sorted.sort_by_key(|c| {
        let rank = ORDER.iter().position(|v| *v == c.verdict);
        let n = number(&c.id);
        (rank, n.is_none(), n, c.id.clone())
    });
    sorted
        .into_iter()
        .map(|c| {
            let (named, more) = cut(&c.ids);
            Row {
                id: c.id.clone(),
                verdict: c.verdict,
                mark: mark(c.verdict),
                name: name(&c.id).to_string(),
                count: Some(c.violations).filter(|n| *n > 0),
                open: c.verdict == Verdict::Violation,
                named,
                more,
            }
        })
        .collect()
}

/// 頁の中身（札と一覧と、名指しの id のうち電文の節点に在るものの一覧の 1 項）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gaps {
    pub tiles: Vec<Tile>,
    pub rows: Vec<Row>,
    pub found: BTreeMap<String, Item>,
    /// 名指しの id のうち電文の節点に在るものの card_of の値（鍵は found と同じ）。
    pub cards: BTreeMap<String, Card>,
}

/// 口の本文を電文に読む（まだ読んでいない・読めない・電文の型として読めないは理由）。
fn doc(fetched: &Fetched) -> Result<GraphDoc, &'static str> {
    match fetched {
        Fetched::NotRead => Err(NOT_READ),
        Fetched::Failed => Err(REASON),
        Fetched::Body(text) => wire::decode::<GraphDoc>(text).map_err(|_| UNREADABLE),
    }
}

/// 口の本文を判定の列に読む（理由は電文に読むときと同じ）。
pub fn checks(fetched: &Fetched) -> Result<Vec<InvariantCheck>, &'static str> {
    doc(fetched).map(|d| d.invariants)
}

/// 頁の中身（測れていない・0 本・札と一覧）。測れていないときは合格と出さない（要件 NFR2）。
pub fn body(fetched: &Fetched) -> Body<Gaps> {
    match doc(fetched) {
        Err(reason) => Body::Unmeasured(reason),
        Ok(d) if d.invariants.is_empty() => Body::Empty(EMPTY),
        Ok(d) => {
            let rows = rows(&d.invariants);
            let found = rows
                .iter()
                .flat_map(|r| &r.named)
                .filter_map(|id| node_item(&d, id).map(|item| (id.clone(), item)))
                .collect();
            let cards = rows
                .iter()
                .flat_map(|r| &r.named)
                .filter_map(|id| card_of(&d, id).map(|card| (id.clone(), card)))
                .collect();
            Body::Filled(Gaps {
                tiles: tiles(&d.invariants),
                rows,
                found,
                cards,
            })
        }
    }
}

/// 見出しは頁の題（h1）にする（見本の gaps.html と同じ）。
#[cfg(target_arch = "wasm32")]
pub fn view() -> leptos::prelude::AnyView {
    dom::view()
}

/// 抜けの検査の頁の DOM（wasm の target のときだけ）。
#[cfg(target_arch = "wasm32")]
mod dom {
    use std::collections::BTreeMap;

    use leptos::prelude::*;

    use super::{BLOCK, Body, Card, Gaps, Item, Row, Tile, body, mark_class, summary_tip};
    use crate::project::{body_view, fold, item_view, map, unmeasured};
    use crate::vocab::label;
    use crate::widgets::help::{expert_tip, h1, hs};

    fn tile_view(t: Tile) -> AnyView {
        view! {
            <div class="gtile">
                <span class=mark_class(t.mark) aria-hidden="true">{t.mark.glyph}</span>
                <span class="n">{t.count}</span>
                {hs(t.mark.key)}
            </div>
        }
        .into_any()
    }

    fn row_view(
        r: Row,
        found: &BTreeMap<String, Item>,
        cards: &BTreeMap<String, Card>,
    ) -> AnyView {
        let tip = summary_tip(&r);
        // 名指しか残りの数が在るときだけ一覧を出す。
        let has_list = !r.named.is_empty() || r.more.is_some();
        // 電文の節点に在る id は節点の頁への link に節点の card を付ける・無い id は頁が無いので字だけ。
        let named = r
            .named
            .into_iter()
            .map(|id| match found.get(&id) {
                Some(item) => item_view(item, None, cards.get(&id).cloned()),
                None => view! { <li><span class="ttl"><span class="nid">{id}</span></span></li> }
                    .into_any(),
            })
            .collect_view();
        let more = r.more.map(|n| {
            view! { <li><span class="gi unknown" aria-hidden="true">"+"</span><span class="ttl num">{n}</span></li> }
        });
        let list = has_list.then(|| view! { <ul class="items">{named}{more}</ul> });
        let initial = r.open;
        let (open, toggle) = fold(format!("gaps:{}", r.id), move || initial);
        view! {
            <details class="gitem" prop:open=open on:toggle=toggle data-id=r.id>
                <summary use:expert_tip=tip>
                    <span class=mark_class(r.mark) aria-label=label(r.mark.key)>{r.mark.glyph}</span>
                    <span class="ttl">{r.name}</span>
                    <span class="aside">{r.count.map(|n| n.to_string()).unwrap_or_default()}</span>
                </summary>
                <div class="body">{list}</div>
            </details>
        }
        .into_any()
    }

    fn panel(inner: AnyView) -> AnyView {
        view! { <section class=BLOCK.class id=BLOCK.id>{inner}</section> }.into_any()
    }

    fn filled(g: Gaps) -> AnyView {
        let Gaps {
            tiles,
            rows,
            found,
            cards,
        } = g;
        let tiles = tiles.into_iter().map(tile_view).collect_view();
        let rows = rows
            .into_iter()
            .map(|r| row_view(r, &found, &cards))
            .collect_view()
            .into_any();
        view! {
            <div class="gtiles">{tiles}</div>
            {panel(rows)}
        }
        .into_any()
    }

    pub fn view() -> AnyView {
        let fetched = crate::net::read(map::PATH);
        let content = move || match fetched.with(body) {
            Body::Unmeasured(reason) => panel(unmeasured(reason)),
            Body::Empty(line) => panel(body_view(Body::Empty(line))),
            Body::Filled(g) => filled(g),
        };
        view! {
            <div class="row">{h1(BLOCK.heading)}</div>
            {content}
        }
        .into_any()
    }
}
