//! account board の開いている窓の block（見本の `#winsp`・HOME の 1 段目の右）と、project board を名前つきの窓で開く手順（便 h-win・案 2）。
//! 見本は account/index.html の openWin・drawWins。窓の名・開く URL・一覧の移り方・button の字は純粋な関数で決め、
//! window の open と一覧の DOM は wasm の target のときだけ組む。一覧は頁の一生の間だけ持ち、どこにも残さない（未決）。

use tsuzuri_contract::account::ProjectRow;

use crate::frame::{Block, Mode};
use crate::project::Body;
use crate::view::Fetched;
use crate::vocab::label;

pub const BLOCK: Block = Block {
    id: "winsp",
    heading: "open_windows",
    class: "panel",
};

/// block の中身（中身の関数がまだ無い: 測れていない）。
pub fn body(fetched: &Fetched) -> Body<()> {
    super::pending(fetched)
}

/// account board の窓の名（project board の「戻る」がこの名でこの窓を見つける）。
pub const ACCOUNT_WIN: &str = "tz-account";

/// project board の窓の名（`tz-<project の名>`）。
pub fn win_name(project: &str) -> String {
    format!("tz-{project}")
}

/// 開く URL の host の後ろの字（電文の board の port に今の mode を足す・board が無い project は開けない）。
/// project board は account board と同じ host で配る（host は頁の location から `board_href` で足す）。
pub fn open_url(row: &ProjectRow, mode: Mode) -> Option<String> {
    row.board.map(|port| format!(":{port}/?mode={}", mode.key()))
}

/// 開く URL（頁の location の protocol〔末尾にコロン〕と hostname〔port を含まない〕に host の後ろの字を足す）。
pub fn board_href(protocol: &str, hostname: &str, tail: &str) -> String {
    format!("{protocol}//{hostname}{tail}")
}

/// 窓の状態（開いている・閉じた）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WinState {
    Open,
    Closed,
}

/// 窓の一覧の 1 項（project の名・窓の名・状態）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Win {
    pub project: String,
    pub name: String,
    pub state: WinState,
}

/// 開いた結果（新しい窓で開いた・開いている窓を前面へ）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opened {
    New,
    Front,
}

/// 開いたときの一覧の移り方: 一覧に無い project は末尾に開いているで足し（新しい）、
/// 在る project は位置を変えずに開いているにする（開いていたなら前面へ・閉じていたなら新しい）。
pub fn after_open(wins: &[Win], project: &str) -> (Vec<Win>, Opened) {
    let mut next = wins.to_vec();
    let how = match next.iter_mut().find(|w| w.project == project) {
        Some(w) => {
            let was = w.state;
            w.state = WinState::Open;
            match was {
                WinState::Open => Opened::Front,
                WinState::Closed => Opened::New,
            }
        }
        None => {
            next.push(Win {
                project: project.to_string(),
                name: win_name(project),
                state: WinState::Open,
            });
            Opened::New
        }
    };
    (next, how)
}

/// handle の closed が真になったときの一覧の移り方: その project を閉じたにする（位置は変えない・無ければそのまま）。
pub fn after_close(wins: &[Win], project: &str) -> Vec<Win> {
    wins.iter()
        .map(|w| {
            let mut w = w.clone();
            if w.project == project {
                w.state = WinState::Closed;
            }
            w
        })
        .collect()
}

/// 一覧の中の project の状態（一覧に無ければ None）。
pub fn state_of(wins: &[Win], project: &str) -> Option<WinState> {
    wins.iter().find(|w| w.project == project).map(|w| w.state)
}

/// 開いている窓の button の字（語の辞書に鍵が無い）。
pub const FRONT: &str = "前面へ";

/// 開く button の語の鍵（開いている窓のほか）。
pub const OPEN_NEW_KEY: &str = "open_new";

/// board を持たない project の button の語の鍵（開けない）。
pub const NOT_YET_KEY: &str = "not_yet";

/// 一覧の行と開く button の字（開いている窓は前面へ・ほかは語の鍵 open_new）。
pub fn button_text(state: Option<WinState>) -> String {
    match state {
        Some(WinState::Open) => FRONT.to_string(),
        _ => label(OPEN_NEW_KEY),
    }
}

/// 一覧の行の状態の語。
pub fn state_word(state: WinState) -> &'static str {
    match state {
        WinState::Open => "開いている",
        WinState::Closed => "閉じた",
    }
}

/// 一覧の行の状態の記号の値（開いているは走行・閉じたは待ち）。
pub fn state_mark(state: WinState) -> &'static str {
    match state {
        WinState::Open => "run",
        WinState::Closed => "wait",
    }
}

/// 一覧の行の class（閉じた窓は字を薄くする）。
pub fn row_class(state: WinState) -> &'static str {
    match state {
        WinState::Open => "",
        WinState::Closed => "w-closed",
    }
}

/// 一覧が空のときの 1 行。
pub const EMPTY: &str = "0（まだ開いていない）";

#[cfg(target_arch = "wasm32")]
pub use dom::open;

#[cfg(target_arch = "wasm32")]
pub fn view() -> leptos::prelude::AnyView {
    dom::list_view()
}

/// 窓を開く手順と一覧の DOM（wasm の target のときだけ）。
#[cfg(target_arch = "wasm32")]
mod dom {
    use std::cell::{Cell, RefCell};
    use std::collections::BTreeMap;
    use std::time::Duration;

    use leptos::prelude::*;
    use tsuzuri_contract::account::AccountDoc;

    use super::{
        BLOCK, EMPTY, NOT_YET_KEY, Win, after_close, after_open, board_href, button_text, open_url,
        row_class, state_mark, state_word, win_name,
    };
    use crate::account::{PATH, doc};
    use crate::frame::Mode;
    use crate::project::{section, state_icon};
    use crate::vocab::label;
    use crate::widgets::help::HelpCtx;

    /// 空の URL で開いた新しい窓の href。
    const BLANK: &str = "about:blank";

    /// handle の closed を確かめる間隔。
    const SWEEP: Duration = Duration::from_secs(1);

    thread_local! {
        /// 窓の一覧（頁の一生の間だけ）。
        static WINS: ArcRwSignal<Vec<Win>> = ArcRwSignal::new(Vec::new());
        /// この頁が開いた窓の handle（project の名ごと・reload で消える）。
        static HANDLES: RefCell<BTreeMap<String, web_sys::Window>> = const { RefCell::new(BTreeMap::new()) };
        /// closed の確かめを始めたか。
        static SWEEPING: Cell<bool> = const { Cell::new(false) };
    }

    fn wins() -> ArcRwSignal<Vec<Win>> {
        WINS.with(Clone::clone)
    }

    /// handle の closed が真になった窓を一覧で閉じたにし、handle を捨てる。
    fn sweep() {
        let dead: Vec<String> = HANDLES.with_borrow(|h| {
            h.iter()
                .filter(|(_, w)| w.closed().unwrap_or(true))
                .map(|(p, _)| p.clone())
                .collect()
        });
        if dead.is_empty() {
            return;
        }
        HANDLES.with_borrow_mut(|h| {
            for p in &dead {
                h.remove(p);
            }
        });
        wins().update(|l| {
            for p in &dead {
                *l = after_close(l, p);
            }
        });
    }

    /// project board を名前つきの窓で開く: handle が生きていれば前面へ、無ければ空の URL と窓の名で開き、
    /// about:blank なら URL（頁の location の protocol と hostname に `url` の字を足す）を入れる（新しい）・
    /// そうでなければ前面へ。location が読めなければ開かない。
    pub fn open(project: &str, url: &str) {
        let location = window().location();
        let (Ok(protocol), Ok(hostname)) = (location.protocol(), location.hostname()) else {
            return;
        };
        let url = &board_href(&protocol, &hostname, url);
        sweep();
        let live = HANDLES.with_borrow(|h| h.get(project).cloned());
        let win = match live {
            Some(w) => w,
            None => {
                let Ok(Some(w)) = window().open_with_url_and_target("", &win_name(project)) else {
                    return;
                };
                if w.location().href().is_ok_and(|href| href == BLANK) {
                    let _ = w.location().set_href(url);
                }
                HANDLES.with_borrow_mut(|h| h.insert(project.to_string(), w.clone()));
                w
            }
        };
        let _ = win.focus();
        wins().update(|l| *l = after_open(l, project).0);
        if !SWEEPING.replace(true) {
            set_interval(sweep, SWEEP);
        }
    }

    /// 開いている窓の一覧（開いた順・空なら 0 の 1 行）。
    pub fn list_view() -> AnyView {
        let fetched = crate::net::read(PATH);
        let read = Memo::new(move |_| fetched.with(doc).ok());
        let mode = use_context::<HelpCtx>().map(|c| c.mode);
        let list = wins();
        let rows = move || {
            let all = list.get();
            if all.is_empty() {
                return view! { <li>{state_icon(state_mark(super::WinState::Closed))}<span class="ttl small muted">{EMPTY}</span></li> }
                    .into_any();
            }
            all.into_iter()
                .map(|w| row(w, read, mode))
                .collect_view()
                .into_any()
        };
        let body = view! { <ul class="items wins">{rows}</ul> }.into_any();
        section(BLOCK, ().into_any(), body)
    }

    /// 一覧の 1 行（board が無い・まだ読めていない project の button は開けない）。
    fn row(w: Win, read: Memo<Option<AccountDoc>>, mode: Option<RwSignal<Mode>>) -> AnyView {
        let url = {
            let project = w.project.clone();
            move || {
                let m = mode.map_or(Mode::Beginner, |m| m.get());
                read.with(|d| {
                    d.as_ref()
                        .and_then(|d| d.projects.iter().find(|p| p.name == project))
                        .and_then(|p| open_url(p, m))
                })
            }
        };
        let text = {
            let url = url.clone();
            let state = w.state;
            move || match url() {
                Some(_) => button_text(Some(state)),
                None => label(NOT_YET_KEY),
            }
        };
        let off = {
            let url = url.clone();
            move || url().is_none()
        };
        let click = {
            let project = w.project.clone();
            move |_| {
                if let Some(u) = url() {
                    open(&project, &u);
                }
            }
        };
        view! {
            <li class=row_class(w.state)>
                {state_icon(state_mark(w.state))}
                <span class="ttl"><span data-t="">{w.project.clone()}</span>" "<span class="small muted mono">{w.name.clone()}</span>" "<span class="small">{state_word(w.state)}</span></span>
                <button type="button" class="btn" disabled=off on:click=click>{text}</button>
            </li>
        }
        .into_any()
    }
}
