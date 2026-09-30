//! block「表示先」（HOME の最後の段・行 i-stage-all・要件 FR16・持ち主の答え t3-hub.59.22 の案 A）: 窓の表示先の設定を読み、
//! 全部の project に一括で置く・project ごとに置く・上書きを外す・窓を開く。口は契約の stage の path
//! （読みは project board の block と同じ `PATH`・書きは `ALL_PATH`・窓を開く頼みは `OPEN_PATH`）で、この file は口の字を書かない。
//! 読みは畳んだ block を開いた時と書きの後の 1 回だけ（移動の block と同じく畳める段の記録は持たない）。
//! 状態と読みと送りは project board の block（`project::stage`）の `Face` と `load` と `send` を使う。
//! 表の行と要求の本文は純粋な関数にして host で試し、DOM は wasm の target のときだけ組み立てる。

use tsuzuri_contract::stage::{Origin, StageTargets, Targets};
use tsuzuri_contract::wire;

use crate::frame::Block;
use crate::project::stage::{NOT_LISTED, NO_EFFECTIVE, origin_word};

pub use tsuzuri_contract::stage::ALL_PATH;

pub const BLOCK: Block = Block {
    id: "stage",
    heading: "stage_target",
    class: "panel fold mvp",
};

/// 全体の既定の行の頭。
pub const DEFAULT_HEAD: &str = "全体の既定";

/// 一括の行の頭と、一括で置く button の字。
pub const ALL_HEAD: &str = "全部の project を";
pub const ALL_PUT: &str = "一括で置く（上書きを外す）";

/// 表の見出し（project・効く PC・既定か上書きか・最後の列は空）。
pub const HEADS: [&str; 4] = ["project", "効く PC", "既定か上書きか", ""];

/// 表の 1 行（project の名・効く名・出所・効く名が層 A に在るか・窓を開く要求の project）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    pub project: String,
    pub name: Option<String>,
    pub origin: Option<Origin>,
    pub listed: bool,
    /// 窓を開く要求の project（電文の project〔server の --repo の project〕は None）。
    pub open: Option<String>,
}

impl Line {
    /// 効く値が project ごとの上書きか。
    pub fn overridden(&self) -> bool {
        self.origin == Some(Origin::Override)
    }

    /// 出所の chip の字（出所が無ければ None・効く名が層 A に無ければ続けて · 層 A に無い）。
    pub fn chip(&self) -> Option<String> {
        let word = origin_word(self.origin?);
        Some(if self.listed {
            word.to_string()
        } else {
            format!("{word} · {NOT_LISTED}")
        })
    }

    /// 効く PC の字（無ければ表示先なしの字）。
    pub fn shown(&self) -> &str {
        self.name.as_deref().unwrap_or(NO_EFFECTIVE)
    }
}

/// 電文の projects の順に表の行を組む。
pub fn lines(targets: &StageTargets) -> Vec<Line> {
    targets
        .projects
        .iter()
        .map(|p| {
            let e = p.effective.as_ref();
            Line {
                project: p.project.clone(),
                name: e.map(|e| e.name.clone()),
                origin: e.map(|e| e.origin),
                listed: e.is_some_and(|e| e.listed),
                open: (p.project != targets.project).then(|| p.project.clone()),
            }
        })
        .collect()
}

/// 全体の既定の字（無ければ表示先なしの字）。
pub fn default_text(default: Option<&str>) -> &str {
    default.unwrap_or(NO_EFFECTIVE)
}

/// select が初めに選ぶ名（効く名が層 A に在ればそれ・無ければ層 A の最初・層 A が空なら None）。
pub fn picked(line: &Line, names: &[String]) -> Option<String> {
    line.name
        .clone()
        .filter(|n| line.listed && names.contains(n))
        .or_else(|| names.first().cloned())
}

/// 一括で置く要求の本文（電文の字にできなければ空の本文で、server が断る）。
pub fn all_body(to: &str) -> String {
    wire::encode(&Targets::All { to: to.to_string() }).unwrap_or_default()
}

/// project ごとに置く（Some）か上書きを外す（None）要求の本文（電文の字にできなければ空の本文）。
pub fn project_body(project: &str, to: Option<&str>) -> String {
    wire::encode(&Targets::Project {
        project: project.to_string(),
        to: to.map(str::to_string),
    })
    .unwrap_or_default()
}

#[cfg(target_arch = "wasm32")]
pub fn view() -> leptos::prelude::AnyView {
    dom::view()
}

/// 表示先の DOM（wasm の target のときだけ）。
#[cfg(target_arch = "wasm32")]
mod dom {
    use leptos::prelude::*;
    use tsuzuri_contract::stage::StageTargets;

    use super::{
        ALL_HEAD, ALL_PATH, ALL_PUT, BLOCK, DEFAULT_HEAD, HEADS, Line, all_body, default_text,
        lines, picked, project_body,
    };
    use crate::project::stage::{
        CLEAR, OPEN, OPEN_NOTE, OPEN_PATH, PUT, content, names_line, open_body,
    };
    use crate::project::stage::{Face, load, send};
    use crate::project::{Body, NOT_READ, body_view, unmeasured};
    use crate::widgets::help::h2;

    pub fn view() -> AnyView {
        let f = Face::new();
        let toggle = move |ev: web_sys::Event| {
            if event_target::<web_sys::Element>(&ev).has_attribute("open") {
                load(f);
            }
        };
        let body = move || match f.read.get() {
            None => unmeasured(NOT_READ),
            Some(reply) => match content(reply.as_ref().map(|(s, t)| (*s, t.as_str()))) {
                Body::Unmeasured(reason) => unmeasured(reason),
                Body::Empty(line) => body_view(Body::Empty(line)),
                Body::Filled(targets) => filled(f, &targets),
            },
        };
        view! {
            <details class=BLOCK.class id=BLOCK.id on:toggle=toggle>
                <summary>{h2(BLOCK.heading)}</summary>
                {body}
                {move || {
                    f.line
                        .get()
                        .map(|o| view! { <div class="small" role="status">{o.line()}</div> })
                }}
            </details>
        }
        .into_any()
    }

    /// select の選択肢（`chosen` の名を選んでおく）。
    fn options(
        names: &[String],
        chosen: impl Fn() -> Option<String> + Clone + Send + Sync + 'static,
    ) -> impl IntoView {
        names
            .iter()
            .map(|n| {
                let (name, mark) = (n.clone(), n.clone());
                let chosen = chosen.clone();
                view! { <option value=name prop:selected=move || chosen().as_deref() == Some(mark.as_str())>{n.clone()}</option> }
            })
            .collect_view()
    }

    /// 読めた設定の中身（全体の既定・一括・project ごとの表・層 A の名）。
    fn filled(f: Face, targets: &StageTargets) -> AnyView {
        let names = targets.names.clone();
        let default = default_text(targets.default.as_deref()).to_string();
        let names_row = names_line(&names);
        let first = names.first().cloned();
        let chosen = move || f.pick.get().or_else(|| first.clone());
        let all_options = options(&names, chosen.clone());
        let can_put = {
            let chosen = chosen.clone();
            move || chosen().is_some() && !f.sending.get()
        };
        let put_all = move |_| {
            if let Some(to) = chosen() {
                send(f, ALL_PATH, all_body(&to), true);
            }
        };
        let rows = lines(targets)
            .into_iter()
            .map(|line| row(f, line, &names))
            .collect_view();
        let heads = HEADS
            .iter()
            .map(|h| view! { <th>{*h}</th> })
            .collect_view();
        view! {
            <div class="stgrow"><span>{DEFAULT_HEAD}</span><span class="mono">{default}</span></div>
            <div class="stgrow">
                <span>{ALL_HEAD}</span>
                <select on:change=move |ev| f.pick.set(Some(event_target_value(&ev)))>{all_options}</select>
                <button type="button" class="btn sm primary" disabled=move || !can_put() on:click=put_all>{ALL_PUT}</button>
            </div>
            <div class="stgscroll">
                <table class="stgtbl">
                    <thead><tr>{heads}</tr></thead>
                    <tbody>{rows}</tbody>
                </table>
            </div>
            <div class="small muted">{names_row}</div>
        }
        .into_any()
    }

    /// 表の 1 行（効く PC・出所の chip・選ぶ・置く・既定に戻す・窓を開く）。
    fn row(f: Face, line: Line, names: &[String]) -> impl IntoView {
        let pick = RwSignal::new(None::<String>);
        let first = picked(&line, names);
        let chosen = move || pick.get().or_else(|| first.clone());
        let select = options(names, chosen.clone());
        let can_put = {
            let chosen = chosen.clone();
            move || chosen().is_some() && !f.sending.get()
        };
        let put = {
            let project = line.project.clone();
            move |_| {
                if let Some(to) = chosen() {
                    send(f, ALL_PATH, project_body(&project, Some(&to)), true);
                }
            }
        };
        let clear = line.overridden().then(|| {
            let project = line.project.clone();
            view! {
                <button type="button" class="btn sm" disabled=move || f.sending.get() on:click=move |_| send(f, ALL_PATH, project_body(&project, None), true)>{CLEAR}</button>
            }
        });
        let open = line.open.clone();
        let chip = line.chip().map(|c| view! { <span class="chip">{c}</span> });
        view! {
            <tr>
                <td class="mono">{line.project.clone()}</td>
                <td class="mono">{line.shown().to_string()}</td>
                <td>{chip}</td>
                <td>
                    <div class="stgrow">
                        <select on:change=move |ev| pick.set(Some(event_target_value(&ev)))>{select}</select>
                        <button type="button" class="btn sm primary" disabled=move || !can_put() on:click=put>{PUT}</button>
                        {clear}
                        <button type="button" class="btn sm" title=OPEN_NOTE disabled=move || f.sending.get() on:click=move |_| send(f, OPEN_PATH, open_body(open.as_deref()), false)>{OPEN}</button>
                    </div>
                </td>
            </tr>
        }
    }
}
