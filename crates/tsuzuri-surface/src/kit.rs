//! block に共通の部品（project の mod.rs から字を変えずに移した）: 中身の 3 値・状態の記号・一覧の 1 項・
//! 読めないときの理由・畳める段の開き閉じの記録。project の mod.rs が glob で再公開するので、crate::project の path でも使える。
//! DOM の部品は wasm の target のときだけ組み立てる。

use std::collections::BTreeMap;

use tsuzuri_contract::graph::{GraphDoc, NodeKind};
use tsuzuri_contract::ledger::LedgerRow;

use crate::mapview;
use crate::view::{self, Fetched};
use crate::vocab::label;

/// グラフの口（/api/graph）の読み（行 m-map-page で地図の block から移した・crate::project::map の path で使う）。
pub mod map;

/// block の中身（測れていない・0 件・中身あり）。0 件と測れていないを分ける（要件 NFR2）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Body<T> {
    /// 測れていない（理由の 1 行）。
    Unmeasured(&'static str),
    /// 測れて 0 件（その 1 行）。
    Empty(&'static str),
    Filled(T),
}

/// 状態の記号の 5 値（見本の ui.js の ST_V と同じ順・語の鍵）。
pub const STATES: [(&str, &str); 5] = [
    ("run", "st_run"),
    ("wait", "st_wait"),
    ("limit", "st_limit"),
    ("silent", "st_silent"),
    ("unknown", "st_unknown"),
];

/// 測れていないの記号の値。
pub const UNKNOWN: &str = "unknown";

/// 状態の記号の語の鍵（知らない値は測れていない）。
pub fn state_key(v: &str) -> &'static str {
    STATES
        .iter()
        .find(|(s, _)| *s == v)
        .map_or("st_unknown", |(_, k)| k)
}

/// 状態の記号の class（見本の `.st.st-<値>`）。
pub fn state_class(v: &str) -> String {
    format!("st st-{v}")
}

/// 一覧の 1 項（見本の `ul.items > li` = 印・番号・題・右に小さい字）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    /// 印の class（出所の帯 beads の丸・閉じていれば塗る）。
    pub shape: String,
    /// open の問いの印は赤（見本の nodeShape と同じ）。
    pub alert: bool,
    pub id: String,
    pub title: String,
    /// 状態の印と語と種類（`○ 未着手 · task`・`◷ 答え待ち · question`）。
    pub aside: String,
}

/// 印の色の上書き（open の問い）。
pub const ALERT_STYLE: &str = "color:var(--s-stop)";

/// 台帳の行の種類の語（地図の節点の種類の語の鍵から引く・行 g-ledger-kind）。
pub fn kind_word(row: &LedgerRow) -> String {
    label(mapview::band::kind_key(row.node_kind()))
}

/// 台帳の行を一覧の 1 項にする（種類は地図の節点と同じ読み・印と語は `view::row_mark`）。
pub fn item(row: &LedgerRow) -> Item {
    let open = matches!(row.status.as_str(), "open" | "in_progress");
    let alert = row.node_kind() == NodeKind::Question && row.status == "open";
    let mark = view::row_mark(row);
    Item {
        shape: format!("shape band-beads{}", if open { "" } else { " fill" }),
        alert,
        id: row.id.to_string(),
        title: row.title.clone(),
        aside: format!("{} {} · {}", mark.glyph, mark.word, kind_word(row)),
    }
}

/// 電文の節点を一覧の 1 項にする（印と問いの赤は眺めと同じ・右の字は空・節点に無い id は None）。
/// 表示の印と題を id で引くだけで、判定は数えない。
pub fn node_item(doc: &GraphDoc, id: &str) -> Option<Item> {
    let node = doc.nodes.iter().find(|n| n.id == id)?;
    Some(Item {
        shape: mapview::shape_class(doc, node),
        alert: mapview::open_question(doc, node),
        id: node.id.clone(),
        title: node.title.clone(),
        aside: String::new(),
    })
}

/// 台帳の一覧の口が読めないときの理由。
pub const LEDGER_UNREAD: &str =
    "台帳の一覧の口が読めない（届かない・知らせが切れた）ので今の一覧が正しいと言えない";

/// 口をまだ読んでいないときの理由（頁を開いた直後）。
pub const NOT_READ: &str = "この block の口をまだ読んでいない";

/// 口の本文を読めたが、中身を組む関数がまだ無いときの理由（後の block の便が足す）。
pub const NO_CONTENT: &str = "この block の中身はまだ無い";

/// 中身の関数がまだ無い block の中身: 3 値のどれを受けても測れていない（理由は 3 値ごとに違う）。
/// `unread` は口が読めないときの理由。
pub fn pending(fetched: &Fetched, unread: &'static str) -> Body<()> {
    Body::Unmeasured(match fetched {
        Fetched::NotRead => NOT_READ,
        Fetched::Body(_) => NO_CONTENT,
        Fetched::Failed => unread,
    })
}

/// 畳める段の開き閉じの鍵の形（Module の ALL の順に各 module の FOLDS をつないだ列・`{}` は不変条件の id か
/// 問いの id・便 g-steady・行 hs-derived）。
pub fn fold_keys() -> Vec<&'static str> {
    crate::project::Module::ALL
        .into_iter()
        .flat_map(|m| m.folds().iter().copied())
        .collect()
}

/// 鍵の字が fold_keys の形のどれかに合うか（`{}` の所は空でない字）。
pub fn fold_key_ok(key: &str) -> bool {
    fold_keys().iter().any(|form| match form.strip_suffix("{}") {
        Some(prefix) => key
            .strip_prefix(prefix)
            .is_some_and(|rest| !rest.is_empty()),
        None => key == *form,
    })
}

/// 畳める段の開き閉じの記録（鍵ごとに 1 つの値・頁の一生の間だけ持ち、URL にも保存の口にも書かない）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Folds(BTreeMap<String, bool>);

impl Folds {
    /// 記録した値（まだ無ければ None）。
    pub fn recorded(&self, key: &str) -> Option<bool> {
        self.0.get(key).copied()
    }

    /// 今の開き閉じ（記録に値が無ければ呼び手が与えた初めの値）。
    pub fn open(&self, key: &str, initial: bool) -> bool {
        self.recorded(key).unwrap_or(initial)
    }

    /// 値を置く（ほかの鍵の値は変えない）。
    pub fn set(&mut self, key: &str, open: bool) {
        self.0.insert(key.to_string(), open);
    }

    /// toggle の後の書き戻し: 今の開き閉じが出している値と違うときだけ置く（置いたら true）。
    /// 初めの値のとおりに開いた・閉じた toggle は置かないので、記録に値が無い鍵は初めの値に従い続ける。
    pub fn write_back(&mut self, key: &str, open: bool, initial: bool) -> bool {
        let changed = self.open(key, initial) != open;
        if changed {
            self.set(key, open);
        }
        changed
    }
}

#[cfg(target_arch = "wasm32")]
pub use dom::{body_view, fold, item_view, section, state_icon, unmeasured};

/// block に共通の DOM の部品（wasm の target のときだけ）。
#[cfg(target_arch = "wasm32")]
mod dom {
    use std::cell::RefCell;

    use leptos::prelude::*;

    use super::{ALERT_STYLE, Body, Folds, Item, UNKNOWN, state_class, state_key};
    use crate::frame::{Block, Mode, node_href};
    use crate::vocab::label;
    use crate::widgets::help::{HelpCtx, h2};
    use crate::widgets::hover::{Card, attach_some};

    thread_local! {
        /// 畳める段の開き閉じの記録（頁の一生の間だけ）。
        static FOLDS: RefCell<Folds> = RefCell::new(Folds::default());
    }

    /// 畳める段の開き閉じ（details の prop の open に結ぶ）と、toggle の event で記録へ書き戻す handler。
    /// `initial` は記録に値が無いときの初めの値（台帳の「詳しく」は mode から取る）。
    pub fn fold(
        key: String,
        initial: impl Fn() -> bool + Clone + 'static,
    ) -> (
        impl Fn() -> bool + Clone + 'static,
        impl Fn(web_sys::Event) + 'static,
    ) {
        let open = {
            let (key, initial) = (key.clone(), initial.clone());
            move || {
                let init = initial();
                FOLDS.with_borrow(|f| f.open(&key, init))
            }
        };
        let toggle = move |ev: web_sys::Event| {
            let now = event_target::<web_sys::Element>(&ev).has_attribute("open");
            let init = untrack(&initial);
            FOLDS.with_borrow_mut(|f| f.write_back(&key, now, init));
        };
        (open, toggle)
    }

    /// 砂時計（限度の記号・見本の IC.hourglass）。
    const HOURGLASS: &str = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linejoin="round" aria-hidden="true"><path d="M6 3h12M6 21h12"/><path d="M7 3c0 5 5 6 5 9s-5 4-5 9h10c0-5-5-6-5-9s5-4 5-9" /><path d="M9.5 19h5l-2.5-2.5z" fill="currentColor" stroke="none"/></svg>"#;

    /// block の section（見出しの横に `extra`・下に `body`）。
    pub fn section(block: Block, extra: AnyView, body: AnyView) -> AnyView {
        view! {
            <section class=block.class id=block.id>
                <header>{h2(block.heading)}{extra}</header>
                {body}
            </section>
        }
        .into_any()
    }

    /// 状態の記号（5 値・色 + 形 + 動き）。
    pub fn state_icon(v: &'static str) -> AnyView {
        let aria = label(state_key(v));
        if v == "limit" {
            view! { <span class=state_class(v) role="img" aria-label=aria data-stv=v inner_html=HOURGLASS></span> }
                .into_any()
        } else {
            view! { <span class=state_class(v) role="img" aria-label=aria data-stv=v><i></i></span> }.into_any()
        }
    }

    /// 測れていないの印と理由の 1 行（0 件と区別する）。
    pub fn unmeasured(reason: &'static str) -> AnyView {
        view! {
            <div class="empty">
                {state_icon(UNKNOWN)}
                <span>{label(state_key(UNKNOWN))}</span>
                <span class="small muted">{reason}</span>
            </div>
        }
        .into_any()
    }

    /// 中身の無い block の中身（測れていない・0 件の 1 行・中身ありは何も出さない）。
    pub fn body_view(body: Body<()>) -> AnyView {
        match body {
            Body::Unmeasured(reason) => unmeasured(reason),
            Body::Empty(line) => view! { <div class="empty"><span>{line}</span></div> }.into_any(),
            Body::Filled(()) => ().into_any(),
        }
    }

    /// 今の mode を返す関数（context が無ければ今の URL の query から・link に mode を残す）。
    fn mode_of() -> impl Fn() -> Mode + Copy + Send + Sync + 'static {
        let ctx = use_context::<HelpCtx>();
        let url = Mode::from_query(&window().location().search().unwrap_or_default());
        move || ctx.map_or(url, |c| c.mode.get())
    }

    /// 一覧の 1 項（`lead` は印の代わりに置く番号など・None なら印）。題は節点の頁への link で、節点の card が在れば付ける。
    pub fn item_view(item: &Item, number: Option<usize>, card: Option<Card>) -> AnyView {
        let lead = match number {
            Some(n) => view! { <span class="nb">{n}</span> }.into_any(),
            None => {
                let style = if item.alert { ALERT_STYLE } else { "" };
                view! { <span class=item.shape.clone() style=style aria-hidden="true"></span> }
                    .into_any()
            }
        };
        let mode = mode_of();
        let id = item.id.clone();
        let href = move || node_href(&id, mode());
        view! {
            <li>
                {lead}
                <a class="ttl" href=href use:attach_some=card><span class="nid">{item.id.clone()}</span>" "<span data-t="">{item.title.clone()}</span></a>
                <span class="aside">{item.aside.clone()}</span>
            </li>
        }
        .into_any()
    }
}
