//! 表の面（見本の map.html の table）: 種類と種類の辺の数の行列（行 = 辺の from の種類・列 = to の種類）。
//! 辺が 1 本も無い種類は行にも列にも出さない。数の cell を押すと一覧の面へ移り、その種類の組で絞る。
//! 幅の狭い画面のための一覧（`.mxlist`）も同じ数を出す。両端のどちらかが節点に無い辺は数えない。
//! 行列の数の cell は辺の型の名を経験者だけの注釈に持つ（見本の data-tip-expert・行 g-map-tips）。

use tsuzuri_contract::graph::{EdgeType, GraphDoc, NodeKind};
use tsuzuri_contract::wire;

use super::band::band_of;
use super::kinds_by_id;

/// 行列の数の在る 1 つの cell（種類の組・辺の数・辺の型）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    pub from: NodeKind,
    pub to: NodeKind,
    pub count: usize,
    /// 辺の型（契約の型の順・重複なし）。
    pub types: Vec<EdgeType>,
}

impl Cell {
    /// 狭い画面の一覧の印の class（from の種類の帯）。
    pub fn shape(&self) -> String {
        format!("shape {} fill", band_of(self.from).class_name())
    }

    /// 辺の型の名（電文の字を「 · 」でつなぐ）。
    pub fn types_text(&self) -> String {
        self.types
            .iter()
            .map(|t| edge_name(*t))
            .collect::<Vec<_>>()
            .join(" · ")
    }
}

/// 辺の型の電文の字。
pub fn edge_name(t: EdgeType) -> String {
    wire::encode(&t)
        .map(|s| s.trim_matches('"').to_string())
        .unwrap_or_default()
}

/// 種類と種類の辺の数の行列。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Matrix {
    /// 行と列の種類（辺の端に 1 度でも出る種類・契約の型の順）。
    pub kinds: Vec<NodeKind>,
    /// 数の在る cell（行の順、次に列の順）。
    pub cells: Vec<Cell>,
}

impl Matrix {
    pub fn cell(&self, from: NodeKind, to: NodeKind) -> Option<&Cell> {
        self.cells.iter().find(|c| c.from == from && c.to == to)
    }

    /// 組の辺の数（無ければ 0）。
    pub fn count(&self, from: NodeKind, to: NodeKind) -> usize {
        self.cell(from, to).map_or(0, |c| c.count)
    }
}

/// 電文の辺から行列を数える。
pub fn matrix(doc: &GraphDoc) -> Matrix {
    let kinds_of = kinds_by_id(doc);
    let mut cells: Vec<Cell> = Vec::new();
    for e in &doc.edges {
        let (Some(&from), Some(&to)) = (kinds_of.get(e.from.as_str()), kinds_of.get(e.to.as_str()))
        else {
            continue;
        };
        match cells.iter_mut().find(|c| c.from == from && c.to == to) {
            Some(c) => {
                c.count += 1;
                if !c.types.contains(&e.edge_type) {
                    c.types.push(e.edge_type);
                }
            }
            None => cells.push(Cell {
                from,
                to,
                count: 1,
                types: vec![e.edge_type],
            }),
        }
    }
    for c in &mut cells {
        c.types.sort();
    }
    cells.sort_by_key(|c| (c.from, c.to));
    let kinds = NodeKind::ALL
        .into_iter()
        .filter(|k| cells.iter().any(|c| c.from == *k || c.to == *k))
        .collect();
    Matrix { kinds, cells }
}

#[cfg(target_arch = "wasm32")]
pub use dom::view;

/// 表の面の DOM（wasm の target のときだけ）。
#[cfg(target_arch = "wasm32")]
mod dom {
    use leptos::prelude::*;
    use tsuzuri_contract::graph::{GraphDoc, NodeKind};

    use super::{Cell, matrix};
    use crate::mapview::band::{band_of, kind_key};
    use crate::mapview::list::{pair_value, with_pair};
    use crate::mapview::{navigate, unread_reasons};
    use crate::project::unmeasured;
    use crate::vocab::label;
    use crate::widgets::help::{expert_tip, h2};

    /// 表の行（行の見出しと、組の在る升だけ押せる数の button）。
    fn matrix_rows(m: &super::Matrix, search: RwSignal<String>) -> impl IntoView + use<> {
        m.kinds
            .iter()
            .map(|a| {
                let cells = m
                    .kinds
                    .iter()
                    .map(|b| match m.cell(*a, *b) {
                        Some(c) => view! {
                            <td use:expert_tip=c.types_text()>{pair_button(c, search, "")}</td>
                        }
                        .into_any(),
                        None => view! { <td></td> }.into_any(),
                    })
                    .collect_view();
                view! { <tr><th>{kind_head(*a)}</th>{cells}</tr> }
            })
            .collect_view()
    }

    pub fn view(doc: &GraphDoc, search: RwSignal<String>) -> AnyView {
        let m = matrix(doc);
        let unread = unread_reasons(doc)
            .into_iter()
            .map(unmeasured)
            .collect_view();
        if m.kinds.is_empty() {
            return view! {
                {unread}
                <header>{h2("matrix")}</header>
                <div class="empty"><span>{label("matrix")}</span><b class="num">"0"</b></div>
            }
            .into_any();
        }
        let head = m
            .kinds
            .iter()
            .map(|k| view! { <th>{kind_head(*k)}</th> })
            .collect_view();
        let rows = matrix_rows(&m, search);
        let list = m
            .cells
            .iter()
            .map(|c| {
                view! {
                    <li>
                        <span class=c.shape() aria-hidden="true"></span>
                        <span class="pair">
                            {label(kind_key(c.from))}" → "{label(kind_key(c.to))}" "
                            <code class="small muted">{c.types_text()}</code>
                        </span>
                        {pair_button(c, search, "num")}
                    </li>
                }
            })
            .collect_view();
        view! {
            {unread}
            <header>{h2("matrix")}</header>
            <div class="mxwrap">
                <table class="mx">
                    <thead><tr><th></th>{head}</tr></thead>
                    <tbody>{rows}</tbody>
                </table>
            </div>
            <ul class="items mxlist">{list}</ul>
        }
        .into_any()
    }

    /// 行と列の見出し（帯の chip の色と種類の語）。
    fn kind_head(kind: NodeKind) -> AnyView {
        let band = band_of(kind);
        view! {
            <span class=band.chip_class() data-term=band.key()><i></i></span>
            {label(kind_key(kind))}
        }
        .into_any()
    }

    /// 数の button（押すと一覧の面へ移り、その組で絞る）。
    fn pair_button(c: &Cell, search: RwSignal<String>, class: &'static str) -> AnyView {
        let (from, to) = (c.from, c.to);
        let click = move |_| navigate(search, |s| with_pair(s, Some((from, to))), true);
        view! {
            <button type="button" class=class data-pair=pair_value(from, to) on:click=click>
                {c.count}
            </button>
        }
        .into_any()
    }
}
