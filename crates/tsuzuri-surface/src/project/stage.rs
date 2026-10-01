//! block「表示先」（home の右の列・行 i-stage-own・要件 FR16・判断の記録 ADR-15 の決定 (1)(2)(6)・
//! 持ち主の裁定 t3-hub.59.1 と t3-hub.59.7 と t3-hub.52.29）: 窓の表示先の設定を読み、自分の project の分だけ置く・戻す。
//! 口は契約の stage の path（読みと書きは `PATH`・窓を開く頼みは `OPEN_PATH`）で、この file は口の字を書かない。
//! 読みは畳んだ block を開いた時と書きの後の 1 回だけ（GET 1 回で子 process が約 3 本立つ・判断の記録 ADR-23 の軽さの向き）。
//! だから知らせの登録の読みを使わず、変化の知らせでも読み直さず、上端の帯の最終の記録にも数えない。
//! 窓を開く button は持ち主が押す口で、席は押さない（server は頭 Origin の無い POST を断る）。
//! 字と本文と 1 行は純粋な関数にして host で試し、DOM は wasm の target のときだけ組み立てる。
//! account board の block（行 i-stage-all）も `Face` と `load` と `send` と純粋な関数を使う。

use tsuzuri_contract::stage::{Effective, OpenRequest, Origin, OwnTarget, StageTargets};
use tsuzuri_contract::wire;

use super::Body;
use crate::frame::Block;

pub use tsuzuri_contract::stage::{OPEN_PATH, PATH};

pub const BLOCK: Block = Block {
    id: "stage",
    heading: "stage_target",
    class: "panel fold mvp",
};

/// この file が字を持つ口の path（口の字は契約の型の crate に在り、ここは書かない）。
pub const PATHS: &[&str] = &[];

/// この file の畳める段の開き閉じの鍵の形（block 全体の 1 つ）。
pub const FOLDS: &[&str] = &["stage:block"];

/// 読みの応答が無い（届かない）ときの理由。
pub const UNREACHED: &str = "表示先の口に届かない（設定を読めていない）";

/// 200 の本文が電文として読めない・知らない断りのときの理由。
pub const BAD_READ: &str = "表示先の口の応答が読めない（設定の字が電文でない）";

/// 502 tz-failed のときの理由。
pub const TZ_DOWN: &str = "tz の口が失敗した（撃てないか rc が 0 でないか時間内に返らない）";

/// 403 origin のときの理由。
pub const REFUSED_READ: &str = "表示先の口が読みを断った（別の origin からの頼み）";

/// tz が撃てないか rc が 0 でないか時間内に返らない（server の 502 の語）。
pub const TZ_FAILED: &str = "tz-failed";

/// tz の標準出力が電文として読めない（server の 502 の語）。
pub const BAD_REPLY: &str = "bad-reply";

/// 別の origin からの頼みへの断り（server の 403 の語）。
pub const ORIGIN: &str = "origin";

/// 要求の本文が読めない（server の 400 の語）。
pub const BAD_BODY: &str = "bad-body";

/// 端末の名の形が違う（server の 400 の語）。
pub const BAD_NAME: &str = "bad-name";

/// project が群の宣言に無い（server の 404 の語）。
pub const NO_PROJECT: &str = "no-project";

/// 本文が大きすぎる（server の 413 の語）。
pub const TOO_LARGE: &str = "too-large";

/// 効く値が無いときの字。
pub const NO_EFFECTIVE: &str = "表示先なし（席の目と URL に落ちる）";

/// 効く値が全体の既定の出所の語。
pub const ORIGIN_DEFAULT: &str = "既定";

/// 効く値が project ごとの上書きの出所の語。
pub const ORIGIN_OVERRIDE: &str = "上書き";

/// 効く値が層 A の名に無いときに足す字。
pub const NOT_LISTED: &str = "層 A に無い";

/// 層 A の名が無いときの字。
pub const NO_NAMES: &str = "なし";

/// 層 A の名の行の頭と尾。
pub const NAMES_HEAD: &str = "PC の一覧（層 A）は";
pub const NAMES_TAIL: &str = "（器の設定・ここでは変えない）";

/// 置く button の字。
pub const PUT: &str = "置く";

/// 既定に戻す button の字（全体の既定の名を括弧で添える前）。
pub const CLEAR: &str = "既定に戻す";

/// 窓を開く button の字。
pub const OPEN: &str = "窓を開く";

/// 窓を開く button の注記。
pub const OPEN_NOTE: &str = "閉じた窓を開き直す（席は開き直さない）";

/// 口に届かなかった送りの字。
pub const NOT_REACHED: &str = "口に届かない";

/// 断りの 1 行の頭。
pub const REFUSED: &str = "口が断った・";

/// 200 で標準出力が空だったときの字。
pub const DONE: &str = "済み";

/// 読みの応答から中身を決める（`reply` は状態の数と本文の字・届かなければ None）。
/// 200 は本文（前後の空白と改行を除いて）が電文として読めれば `Filled`・読めなければ理由の 1 行。
pub fn content(reply: Option<(u16, &str)>) -> Body<StageTargets> {
    match reply {
        None => Body::Unmeasured(UNREACHED),
        Some((200, text)) => match wire::decode::<StageTargets>(text.trim()) {
            Ok(targets) => Body::Filled(targets),
            Err(_) => Body::Unmeasured(BAD_READ),
        },
        Some((502, text)) if text.trim() == TZ_FAILED => Body::Unmeasured(TZ_DOWN),
        Some((403, text)) if text.trim() == ORIGIN => Body::Unmeasured(REFUSED_READ),
        Some(_) => Body::Unmeasured(BAD_READ),
    }
}

/// 出所の語。
pub fn origin_word(origin: Origin) -> &'static str {
    match origin {
        Origin::Default => ORIGIN_DEFAULT,
        Origin::Override => ORIGIN_OVERRIDE,
    }
}

/// 効く値の字（名 · 出所の語・層 A に無ければ続けて · 層 A に無い・効く値が無ければ表示先なしの字）。
pub fn effective_line(effective: Option<&Effective>) -> String {
    let Some(e) = effective else {
        return NO_EFFECTIVE.to_string();
    };
    let line = format!("{} · {}", e.name, origin_word(e.origin));
    if e.listed {
        line
    } else {
        format!("{line} · {NOT_LISTED}")
    }
}

/// 層 A の名の行（並べるだけで変えない）。
pub fn names_line(names: &[String]) -> String {
    let list = if names.is_empty() {
        NO_NAMES.to_string()
    } else {
        names.join("・")
    };
    format!("{NAMES_HEAD} {list}{NAMES_TAIL}")
}

/// 自分の project の分（電文の project の行の効く値・全体の既定・層 A の名）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Own {
    pub project: String,
    pub effective: Option<Effective>,
    pub default: Option<String>,
    pub names: Vec<String>,
}

/// 電文の project の行を projects から引いて自分の分にする（行が無ければ効く値なし）。
pub fn own(targets: &StageTargets) -> Own {
    let effective = targets
        .projects
        .iter()
        .find(|p| p.project == targets.project)
        .and_then(|p| p.effective.clone());
    Own {
        project: targets.project.clone(),
        effective,
        default: targets.default.clone(),
        names: targets.names.clone(),
    }
}

impl Own {
    /// 効く値が project ごとの上書きか。
    pub fn overridden(&self) -> bool {
        self.effective
            .as_ref()
            .is_some_and(|e| e.origin == Origin::Override)
    }

    /// select が初めに選ぶ名（効く値が層 A に在ればそれ・無ければ層 A の最初・層 A が空なら None）。
    pub fn picked(&self) -> Option<String> {
        self.effective
            .as_ref()
            .filter(|e| self.names.contains(&e.name))
            .map(|e| e.name.clone())
            .or_else(|| self.names.first().cloned())
    }

    /// 既定に戻す button の字（全体の既定の名があれば括弧で添える）。
    pub fn clear_label(&self) -> String {
        match &self.default {
            Some(name) => format!("{CLEAR}（{name}）"),
            None => CLEAR.to_string(),
        }
    }
}

/// 置く・戻すの要求の本文（Some が置く名・None が上書きを外す・電文の字にできなければ空の本文で、server が断る）。
pub fn own_body(to: Option<&str>) -> String {
    wire::encode(&OwnTarget {
        to: to.map(str::to_string),
    })
    .unwrap_or_default()
}

/// 窓を開く要求の本文（None が --repo の project・電文の字にできなければ空の本文）。
pub fn open_body(project: Option<&str>) -> String {
    wire::encode(&OpenRequest {
        project: project.map(str::to_string),
    })
    .unwrap_or_default()
}

/// 送った後の 1 行。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// 200: tz の標準出力の字（改行を除く）か、空なら済みの字。
    Done(String),
    /// 届かない・断られた: その 1 行。
    Refused(String),
}

impl Outcome {
    /// block に出す 1 行。
    pub fn line(&self) -> String {
        match self {
            Outcome::Done(line) | Outcome::Refused(line) => line.clone(),
        }
    }
}

/// server の語から断りの理由（知らない語は語のまま）。
fn reason(status: u16, word: &str) -> String {
    match (status, word) {
        (403, ORIGIN) => "別の origin からの頼みを断った".to_string(),
        (400, BAD_BODY) => "要求の本文が読めない".to_string(),
        (400, BAD_NAME) => "端末の名の形が違う".to_string(),
        (404, NO_PROJECT) => "project が群の宣言に無い".to_string(),
        (502, TZ_FAILED) => TZ_DOWN.to_string(),
        _ => word.to_string(),
    }
}

/// 応答から送った後の 1 行を決める（`reply` は状態の数と本文の字・届かなければ None）。
pub fn outcome(reply: Option<(u16, &str)>) -> Outcome {
    match reply {
        None => Outcome::Refused(NOT_REACHED.to_string()),
        Some((200, text)) => {
            let text = text.trim();
            Outcome::Done(if text.is_empty() { DONE } else { text }.to_string())
        }
        Some((413, text)) if text.trim() == TOO_LARGE => {
            Outcome::Refused(format!("{REFUSED}413 {TOO_LARGE}"))
        }
        Some((status, text)) => {
            Outcome::Refused(format!("{REFUSED}{}（{status}）", reason(status, text.trim())))
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use dom::{Face, load, send};

#[cfg(target_arch = "wasm32")]
pub fn view() -> leptos::prelude::AnyView {
    dom::view()
}

/// 表示先の窓の中身（開いた時に 1 回読む・帯の設定の中の口が開く・行 g-win-parts）。
#[cfg(target_arch = "wasm32")]
pub fn inner() -> leptos::prelude::AnyView {
    dom::inner()
}

/// 表示先の DOM（wasm の target のときだけ）。
#[cfg(target_arch = "wasm32")]
mod dom {
    use leptos::prelude::*;
    use leptos::task::spawn_local;
    use tsuzuri_contract::stage::StageTargets;

    use super::{
        BLOCK, Body, OPEN, OPEN_NOTE, OPEN_PATH, Outcome, PATH, PUT, content, effective_line,
        names_line, open_body, outcome, own, own_body,
    };
    use crate::project::{NOT_READ, body_view, fold, unmeasured};
    use crate::widgets::help::h2;

    /// 読みの応答（状態の数と本文の字・届かなければ None）。
    type Reply = Option<(u16, String)>;

    /// block の状態（読みの応答・選んだ名・送っている間・送った後の 1 行）。
    #[derive(Clone, Copy)]
    pub struct Face {
        /// まだ読んでいなければ None。
        pub read: RwSignal<Option<Reply>>,
        /// 持ち主が select で選んだ名（選んでいなければ初めの名）。
        pub pick: RwSignal<Option<String>>,
        pub sending: RwSignal<bool>,
        pub line: RwSignal<Option<Outcome>>,
    }

    impl Face {
        pub fn new() -> Self {
            Face {
                read: RwSignal::new(None),
                pick: RwSignal::new(None),
                sending: RwSignal::new(false),
                line: RwSignal::new(None),
            }
        }
    }

    impl Default for Face {
        fn default() -> Self {
            Self::new()
        }
    }

    /// 口を 1 回読んで応答を置く（選んだ名は捨てて、読んだ効く値に従う）。
    pub fn load(f: Face) {
        spawn_local(async move {
            let reply = crate::net::get(PATH).await;
            f.pick.set(None);
            f.read.set(Some(reply));
        });
    }

    /// 書きか窓を開く頼みを 1 回送り、送った後の 1 行を置く（`reread` なら書きの後に読み直す・送っている間は送らない）。
    pub fn send(f: Face, path: &'static str, body: String, reread: bool) {
        if f.sending.get_untracked() {
            return;
        }
        f.sending.set(true);
        spawn_local(async move {
            let reply = crate::net::post(path, body).await;
            f.line
                .set(Some(outcome(reply.as_ref().map(|(s, t)| (*s, t.as_str())))));
            f.sending.set(false);
            if reread {
                load(f);
            }
        });
    }

    pub fn view() -> AnyView {
        let f = Face::new();
        let (open, record) = fold("stage:block".to_string(), || false);
        let toggle = move |ev: web_sys::Event| {
            let now = event_target::<web_sys::Element>(&ev).has_attribute("open");
            record(ev);
            if now {
                load(f);
            }
        };
        view! {
            <details class=BLOCK.class id=BLOCK.id prop:open=open on:toggle=toggle>
                <summary>{h2(BLOCK.heading)}</summary>
                {body(f)}
            </details>
        }
        .into_any()
    }

    /// 窓の中身（作った時に 1 回読む）。
    pub fn inner() -> AnyView {
        let f = Face::new();
        load(f);
        body(f)
    }

    /// 読みの中身と送った後の 1 行。
    fn body(f: Face) -> AnyView {
        let body = move || match f.read.get() {
            None => unmeasured(NOT_READ),
            Some(reply) => match content(reply.as_ref().map(|(s, t)| (*s, t.as_str()))) {
                Body::Unmeasured(reason) => unmeasured(reason),
                Body::Empty(line) => body_view(Body::Empty(line)),
                Body::Filled(targets) => filled(f, &targets),
            },
        };
        view! {
            {body}
            {move || {
                f.line
                    .get()
                    .map(|o| view! { <div class="small" role="status">{o.line()}</div> })
            }}
        }
        .into_any()
    }

    /// 読めた設定の中身（効く値・置く・戻す・窓を開く・層 A の名）。
    fn filled(f: Face, targets: &StageTargets) -> AnyView {
        let mine = own(targets);
        let effective = effective_line(mine.effective.as_ref());
        let names_row = names_line(&mine.names);
        let clear = mine.overridden().then(|| {
            view! {
                <button type="button" class="btn sm" disabled=move || f.sending.get() on:click=move |_| send(f, PATH, own_body(None), true)>{mine.clear_label()}</button>
            }
        });
        let first = mine.picked();
        let chosen = {
            let first = first.clone();
            move || f.pick.get().or_else(|| first.clone())
        };
        let options = mine
            .names
            .iter()
            .map(|n| {
                let (name, mark) = (n.clone(), n.clone());
                let chosen = chosen.clone();
                view! { <option value=name prop:selected=move || chosen().as_deref() == Some(mark.as_str())>{n.clone()}</option> }
            })
            .collect_view();
        let can_put = {
            let chosen = chosen.clone();
            move || chosen().is_some() && !f.sending.get()
        };
        let put = move |_| {
            if let Some(to) = chosen() {
                send(f, PATH, own_body(Some(&to)), true);
            }
        };
        view! {
            <div class="stgrow"><span class="mono">{effective}</span></div>
            <div class="stgrow">
                <span class="mono">{mine.project.clone()}</span>
                <select on:change=move |ev| f.pick.set(Some(event_target_value(&ev)))>{options}</select>
                <button type="button" class="btn sm primary" disabled=move || !can_put() on:click=put>{PUT}</button>
                {clear}
            </div>
            <div class="stgrow">
                <button type="button" class="btn sm" disabled=move || f.sending.get() on:click=move |_| send(f, OPEN_PATH, open_body(None), false)>{OPEN}</button>
                <span class="small muted">{OPEN_NOTE}</span>
            </div>
            <div class="small muted">{names_row}</div>
        }
        .into_any()
    }
}
