//! 相談の窓（判断の記録 ADR-29 決定 (4)(9)）: 帯の相談の口が開く窓。
//! 上の段は頼みの form（題〔省ける・自由な字か bead の id〕・形〔話す・問う〕・model〔`MODELS`〕・送る）で、
//! 送ると口 POST /api/consult/request に電文 ConsultRequest を送る（面は窓を開かない・席が受けて開く）。
//! 下の段は口 GET /api/consult の一覧（退いていない窓・処分の無い所見・受けの無い頼み）で、受けの無い頼みは
//! 頼みの時刻から `DELIVER_SPAN` を越えると `LATE` を添える。読めない段は測れていない。
//! 帯の相談の口は色の付く数を持たない（所見の処分は席の手番・決定 (9)）。窓を開いた link が bead の id を持てば、
//! 題の初めの字にする（質問の窓と同じ `AskFocus`）。札と行の吹き出しの相談の口は `consult_href` の link で、
//! 押すとその bead の id を題に入れてこの窓を開く（hover では開かない）。字と並びは純粋な関数にして host で試し、窓の DOM（`body`）は
//! wasm の target のときだけ組む。

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::consult::{
    ConsultBoard, ConsultRequest, FindingRow, Form, RequestId, RequestRow, WindowRow, WindowState,
    Word,
};
use tsuzuri_contract::wire;

use crate::frame::Mode;
use crate::mapview::encode;
use crate::project::Body;
use crate::project::ask::FOCUS_KEY;
use crate::topbar::{Win, win_href};
use crate::view::Fetched;
use crate::vocab::label;

pub use tsuzuri_contract::consult::{PATH, REQUEST_PATH};

/// 帯の相談の口と窓の題の語の鍵。
pub const CONSULT_KEY: &str = "consult";

/// 吹き出しの相談の口の語の鍵（行 cs-pop）。
pub const POP_KEY: &str = "pop_consult";

/// 受けの無い頼みを「席に届いていない」と見る経過（秒・server の配達の撃ち直しの幅と同じ 1800 秒）。
pub const DELIVER_SPAN: EpochSecs = 1800;

/// `DELIVER_SPAN` を越えた受けの無い頼みに添える字。
pub const LATE: &str = "席に届いていない";

/// 頼みの form で選べる model（先頭が既定）。
pub const MODELS: [&str; 3] = ["fable", "opus", "sonnet"];

/// 一覧の口が読めない時の理由。
pub const UNREAD: &str = "相談の一覧が読めない";

/// 3 つの段の見出しの語の鍵（段の順は窓・所見・頼み）。
pub const PART_KEYS: [&str; 3] = ["cs_windows", "cs_findings", "cs_requests"];

/// 3 つの段の 0 件の 1 行（窓・所見・頼み）。
pub const NONE_LINES: [&str; 3] = [
    "開いている相談の窓は無い",
    "処分の無い所見は無い",
    "受けの無い頼みは無い",
];

/// 一覧の 1 行（id・題・状態の字・席に届いていないか）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub id: String,
    pub topic: String,
    pub note: String,
    pub late: bool,
}

/// 吹き出しの相談の口の先（相談の窓を開いた home の頁・bead の id を題の初めの字に渡す・行 cs-pop）。
pub fn consult_href(id: &str, mode: Mode) -> String {
    format!(
        "{}&{FOCUS_KEY}={}",
        win_href(Win::Consult, mode),
        encode(id)
    )
}

/// 頼みの電文（題は前後の空白を除いて空なら無し・model が `MODELS` に無ければ None）。
pub fn request(topic: &str, form: Form, model: &str) -> Option<String> {
    if !MODELS.contains(&model) {
        return None;
    }
    let topic = Some(topic.trim())
        .filter(|t| !t.is_empty())
        .map(str::to_string);
    wire::encode(&ConsultRequest {
        topic,
        form,
        model: model.to_string(),
    })
    .ok()
}

/// 頼みの口の応答の 1 行（200 は頼みの id・403 は読むだけ・400 は字・503 は台帳・届かなければ届かない）。
pub fn sent_line(reply: Option<(u16, &str)>) -> String {
    match reply {
        Some((200, body)) => match wire::decode::<RequestId>(body) {
            Ok(id) => format!("頼み {id} を送った（席が受けて窓を開く）"),
            Err(_) => "頼みの応答が読めない".to_string(),
        },
        Some((403, _)) => "読むだけの board なので送れない".to_string(),
        Some((400, _)) => "頼みの字が書けない（題の字を見直す）".to_string(),
        Some((code, body)) => format!("頼みを置けなかった（状態 {code}・{body}）"),
        None => "board に届かない".to_string(),
    }
}

/// 口の本文の一覧（読めなければ `UNREAD`）。
pub fn board(f: &Fetched) -> Result<ConsultBoard, &'static str> {
    match f {
        Fetched::Body(t) => wire::decode(t).map_err(|_| UNREAD),
        Fetched::NotRead | Fetched::Failed => Err(UNREAD),
    }
}

/// 題の字（無ければ「題なし」）。
fn topic_of(topic: Option<&String>) -> String {
    topic.map_or_else(|| "題なし".to_string(), Clone::clone)
}

/// 段の読みを行の列にする（Unknown は測れていない・0 件は `none` の 1 行）。
fn rows<T>(
    r: &Reading<Vec<T>>,
    none: &'static str,
    f: impl Fn(&T) -> Option<Row>,
) -> Body<Vec<Row>> {
    match r {
        Reading::Unknown => Body::Unmeasured(UNREAD),
        Reading::Known(v) => {
            let out: Vec<Row> = v.iter().filter_map(f).collect();
            if out.is_empty() {
                Body::Empty(none)
            } else {
                Body::Filled(out)
            }
        }
    }
}

/// 窓の段（退いた窓は載せない・状態は形と状態の語と所見の数と口座・口座の無い窓は語の鍵 consult_account_unknown の語・
/// 判断の記録 ADR-55 決定 (4)）。
pub fn windows(r: &Reading<Vec<WindowRow>>) -> Body<Vec<Row>> {
    rows(r, NONE_LINES[0], |w| {
        (w.state != WindowState::Retired).then(|| Row {
            id: w.id.to_string(),
            topic: topic_of(w.topic.as_ref()),
            note: format!(
                "{}・{}／所見 {}（処分なし {}）／口座 {}",
                w.form.word(),
                w.state.word(),
                w.findings,
                w.undisposed,
                w.account
                    .clone()
                    .unwrap_or_else(|| label("consult_account_unknown"))
            ),
            late: false,
        })
    })
}

/// 所見の段（席が受けたか）。
pub fn findings(r: &Reading<Vec<FindingRow>>) -> Body<Vec<Row>> {
    rows(r, NONE_LINES[1], |f| {
        Some(Row {
            id: f.id.to_string(),
            topic: topic_of(f.topic.as_ref()),
            note: if f.received.is_some() {
                "席が受けた"
            } else {
                "席が受けていない"
            }
            .to_string(),
            late: false,
        })
    })
}

/// 頼みの段（`now` が頼みの時刻から `DELIVER_SPAN` を越えたら `LATE`・越えなければ待ち）。
pub fn requests(r: &Reading<Vec<RequestRow>>, now: EpochSecs) -> Body<Vec<Row>> {
    rows(r, NONE_LINES[2], |q| {
        let late = now.saturating_sub(q.at) > DELIVER_SPAN;
        Some(Row {
            id: q.id.to_string(),
            topic: topic_of(q.topic.as_ref()),
            note: if late { LATE } else { "席の受けを待つ" }.to_string(),
            late,
        })
    })
}

#[cfg(target_arch = "wasm32")]
pub use dom::body;

#[cfg(target_arch = "wasm32")]
mod dom {
    use leptos::ev;
    use leptos::prelude::*;
    use leptos::task::spawn_local;
    use tsuzuri_contract::consult::{Form, Word};

    use super::{
        MODELS, PART_KEYS, PATH, REQUEST_PATH, Row, board, findings, request, requests, sent_line,
        windows,
    };
    use crate::askwin::AskFocus;
    use crate::project::{Body, body_view, unmeasured};
    use crate::vocab::label;

    fn rows_view(body: Body<Vec<Row>>) -> AnyView {
        match body {
            Body::Filled(rows) => rows
                .into_iter()
                .map(|r| {
                    let class = if r.late { "csr late" } else { "csr" };
                    view! { <div class=class><code>{r.id}</code><span class="cst">{r.topic}</span><span class="csn">{r.note}</span></div> }
                })
                .collect_view()
                .into_any(),
            Body::Empty(line) => body_view(Body::Empty(line)),
            Body::Unmeasured(reason) => unmeasured(reason),
        }
    }

    fn form_view(
        topic: RwSignal<String>,
        form: RwSignal<Form>,
        model: RwSignal<String>,
        sent: RwSignal<Option<String>>,
    ) -> AnyView {
        let send = move |_| {
            let Some(body) = request(
                &topic.get_untracked(),
                form.get_untracked(),
                &model.get_untracked(),
            ) else {
                return;
            };
            spawn_local(async move {
                let reply = crate::net::post(REQUEST_PATH, body).await;
                sent.set(Some(sent_line(
                    reply.as_ref().map(|(st, t)| (*st, t.as_str())),
                )));
                crate::net::reload_all();
            });
        };
        let forms = [Form::Talk, Form::Ask].map(|f| {
            view! { <label><input type="radio" name="csform" prop:checked=move || form.get() == f on:change=move |_| form.set(f)/>{f.word()}</label> }
        });
        let models = MODELS
            .map(|m| view! { <option value=m selected=move || model.get() == m>{m}</option> });
        view! {
            <div class="csf">
                <input type="text" placeholder="題（省ける・bead の id か自由な字）" prop:value=move || topic.get() on:input=move |ev: ev::Event| topic.set(event_target_value(&ev))/>
                <span class="csfm">{forms}</span>
                <select on:change=move |ev: ev::Event| model.set(event_target_value(&ev))>{models}</select>
                <button type="button" class="btn" on:click=send>"送る ›"</button>
            </div>
            {move || sent.get().map(|s| view! { <div class="small" role="status">{s}</div> })}
        }
        .into_any()
    }

    /// 相談の窓の本文（頼みの form と一覧の 3 つの段）。
    pub fn body() -> AnyView {
        let doc = crate::net::read(PATH);
        let now = crate::net::ticker();
        let first = use_context::<AskFocus>().and_then(|f| f.0.get_untracked());
        let topic = RwSignal::new(first.unwrap_or_default());
        let form = RwSignal::new(Form::Talk);
        let model = RwSignal::new(MODELS[0].to_string());
        let sent = RwSignal::new(None);
        let lists = move || match doc.with(board) {
            Err(reason) => unmeasured(reason),
            Ok(b) => view! {
                <h4 data-v=PART_KEYS[0]>{label(PART_KEYS[0])}</h4>{rows_view(windows(&b.windows))}
                <h4 data-v=PART_KEYS[1]>{label(PART_KEYS[1])}</h4>{rows_view(findings(&b.findings))}
                <h4 data-v=PART_KEYS[2]>{label(PART_KEYS[2])}</h4>{rows_view(requests(&b.requests, now.get()))}
            }
            .into_any(),
        };
        view! { <div class="cswin">{form_view(topic, form, model, sent)}{lists}</div> }.into_any()
    }
}
