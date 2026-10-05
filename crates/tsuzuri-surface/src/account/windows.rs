//! account board の開いている窓の block（見本の `#winsp`・HOME の 1 段目の右）と、project board を名前つきの窓で開く手順（案 2）。
//! 見本は account/index.html の openWin・drawWins。窓の名・開く URL・一覧の移り方・button の字は純粋な関数で決め、
//! window の open と一覧の DOM は wasm の target のときだけ組む。
//! 一覧は browser の保存（鍵 tz-wins・1 項 1 行の tab 区切り）に残す。
//! 保存は store の get・set・on_change で撃ち、頁で初めて一覧を撃つときに 1 度だけ読んで生きている handle を重ねる。
//! 読み直した後の窓は handle を失い、開いたはず（NoHandle）になる。ほかの tab の書き替えは storage の event で置き直し、
//! project board の戻るが送る閉じの知らせ（message の event・送り手の hostname が同じときだけ）でその窓を閉じたにする。

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

/// 窓の状態（開いている・閉じた・開いたはず〔保存から戻したが、この頁が handle を持たない〕）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WinState {
    Open,
    Closed,
    NoHandle,
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
/// 在る project は位置を変えずに開いているにする（開いていた・開いたはずなら前面へ・閉じていたなら新しい）。
pub fn after_open(wins: &[Win], project: &str) -> (Vec<Win>, Opened) {
    let mut next = wins.to_vec();
    let how = match next.iter_mut().find(|w| w.project == project) {
        Some(w) => {
            let was = w.state;
            w.state = WinState::Open;
            match was {
                WinState::Open | WinState::NoHandle => Opened::Front,
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

/// 一覧の行と開く button の字（開いている・開いたはずの窓は前面へ・ほかは語の鍵 open_new）。
pub fn button_text(state: Option<WinState>) -> String {
    match state {
        Some(WinState::Open | WinState::NoHandle) => FRONT.to_string(),
        _ => label(OPEN_NEW_KEY),
    }
}

/// 一覧の行の状態の語。
pub fn state_word(state: WinState) -> &'static str {
    match state {
        WinState::Open => "開いている",
        WinState::Closed => "閉じた",
        WinState::NoHandle => "開いたはず（この窓の reload で handle なし）",
    }
}

/// 一覧の行の状態の記号の値（開いているは走行・閉じたは待ち・開いたはずは不明）。
pub fn state_mark(state: WinState) -> &'static str {
    match state {
        WinState::Open => "run",
        WinState::Closed => "wait",
        WinState::NoHandle => "unknown",
    }
}

/// 一覧の行の class（閉じた・開いたはずの窓は字を薄くする）。
pub fn row_class(state: WinState) -> &'static str {
    match state {
        WinState::Open => "",
        WinState::Closed => "w-closed",
        WinState::NoHandle => "w-nohandle",
    }
}

/// 窓の一覧の保存の鍵（見本の鍵と同じ・account board の origin の保存）。
pub const WINS_KEY: &str = "tz-wins";

/// 保存の字の状態の語（開いたはずは開いているで書く）。
fn saved_word(state: WinState) -> &'static str {
    match state {
        WinState::Open | WinState::NoHandle => "open",
        WinState::Closed => "closed",
    }
}

/// 保存に書く字: 項の順に project の名・tab・状態の語（open か closed）・改行。
pub fn wins_text(wins: &[Win]) -> String {
    wins.iter()
        .map(|w| format!("{}\t{}\n", w.project, saved_word(w.state)))
        .collect()
}

/// 保存の字から一覧を戻す: open の項は開いたはず（この頁は handle を持たない）・closed の項は閉じた。
/// tab がちょうど 1 つでない行・名の空の行・状態の語の違う行・前の行と同じ project の行は読み捨てる。
pub fn wins_from(text: &str) -> Vec<Win> {
    let mut out: Vec<Win> = Vec::new();
    for line in text.lines() {
        let mut parts = line.split('\t');
        let (Some(project), Some(word), None) = (parts.next(), parts.next(), parts.next()) else {
            continue;
        };
        let state = match word {
            "open" => WinState::NoHandle,
            "closed" => WinState::Closed,
            _ => continue,
        };
        if project.is_empty() || out.iter().any(|w| w.project == project) {
            continue;
        }
        out.push(Win {
            project: project.to_string(),
            name: win_name(project),
            state,
        });
    }
    out
}

/// 生きている handle の project の名の列に在る項を開いているにする（ほかの項はそのまま・列だけの名は足さない）。
pub fn with_live(wins: &[Win], live: &[String]) -> Vec<Win> {
    wins.iter()
        .map(|w| {
            let mut w = w.clone();
            if live.contains(&w.project) {
                w.state = WinState::Open;
            }
            w
        })
        .collect()
}

/// 閉じの知らせの字の頭（project board の戻るが account board の窓へ送る・後ろは自分の窓の名）。
pub const CLOSED_MSG: &str = "tz-closed:";

/// 閉じの知らせの字（CLOSED_MSG に窓の名をつなぐ）。
pub fn closed_message(win: &str) -> String {
    format!("{CLOSED_MSG}{win}")
}

/// origin の字の hostname（http か https・port は有っても無くても・IPv6 は角括弧のまま・ほかの形は None）。
fn origin_hostname(origin: &str) -> Option<&str> {
    let rest = origin
        .strip_prefix("http://")
        .or_else(|| origin.strip_prefix("https://"))?;
    let end = if rest.starts_with('[') {
        rest.find(']')? + 1
    } else {
        rest.find(':').unwrap_or(rest.len())
    };
    let (host, tail) = rest.split_at(end);
    let port_ok = tail.is_empty()
        || tail
            .strip_prefix(':')
            .is_some_and(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()));
    (!host.is_empty() && port_ok).then_some(host)
}

/// 閉じの知らせの project の名: 送り手の origin の hostname がこの頁の hostname と同じで（port は見ない）、
/// 字が CLOSED_MSG と頭 tz- と空でない project の名のときだけ。
pub fn closed_project(data: &str, origin: &str, hostname: &str) -> Option<String> {
    if origin_hostname(origin)? != hostname {
        return None;
    }
    let project = data.strip_prefix(CLOSED_MSG)?.strip_prefix("tz-")?;
    (!project.is_empty()).then(|| project.to_string())
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
        BLOCK, EMPTY, NOT_YET_KEY, WINS_KEY, Win, after_close, after_open, board_href, button_text,
        closed_project, open_url, row_class, state_mark, state_word, win_name, wins_from, wins_text,
        with_live,
    };
    use crate::account::{PATH, doc};
    use crate::frame::Mode;
    use crate::project::{section, state_icon};
    use crate::store;
    use crate::vocab::label;
    use crate::widgets::help::HelpCtx;

    /// 空の URL で開いた新しい窓の href。
    const BLANK: &str = "about:blank";

    /// handle の closed を確かめる間隔。
    const SWEEP: Duration = Duration::from_secs(1);

    thread_local! {
        /// 窓の一覧（保存の写しを読んで置き、移すたびに保存に書く）。
        static WINS: ArcRwSignal<Vec<Win>> = ArcRwSignal::new(Vec::new());
        /// この頁が開いた窓の handle（project の名ごと・reload で消える）。
        static HANDLES: RefCell<BTreeMap<String, web_sys::Window>> = const { RefCell::new(BTreeMap::new()) };
        /// closed の確かめを始めたか。
        static SWEEPING: Cell<bool> = const { Cell::new(false) };
        /// 保存の一覧を読み、event の受け手を付けたか（頁で 1 度）。
        static LOADED: Cell<bool> = const { Cell::new(false) };
    }

    /// 窓の一覧（初めて撃たれたときに保存の一覧を置き、保存の書き替えと閉じの知らせの受け手を付ける）。
    fn wins() -> ArcRwSignal<Vec<Win>> {
        let list = WINS.with(Clone::clone);
        if !LOADED.replace(true) {
            reload(&list);
            store::on_change(WINS_KEY, || reload(&wins()));
            let _ = window_event_listener(leptos::ev::message, closed_by_message);
        }
        list
    }

    /// 保存の一覧に生きている handle を重ねて置く（保存が無い・使えないときは空の一覧）。
    fn reload(list: &ArcRwSignal<Vec<Win>>) {
        let saved = wins_from(&store::get(WINS_KEY).unwrap_or_default());
        let live: Vec<String> = HANDLES.with_borrow(|h| h.keys().cloned().collect());
        list.set(with_live(&saved, &live));
    }

    /// 一覧を移して保存に書く。
    fn change(f: impl FnOnce(&mut Vec<Win>)) {
        let list = wins();
        list.update(f);
        store::set(WINS_KEY, &list.with_untracked(|l| wins_text(l)));
    }

    /// project board の戻るの閉じの知らせ: 送り手の hostname が頁と同じなら handle を捨てて一覧で閉じたにする。
    fn closed_by_message(e: web_sys::MessageEvent) {
        let Some(data) = e.data().as_string() else {
            return;
        };
        let Ok(hostname) = window().location().hostname() else {
            return;
        };
        let Some(project) = closed_project(&data, &e.origin(), &hostname) else {
            return;
        };
        HANDLES.with_borrow_mut(|h| h.remove(&project));
        change(|l| *l = after_close(l, &project));
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
        change(|l| {
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
                let Ok(Some(w)) = crate::board::open_named("", &win_name(project)) else {
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
        change(|l| *l = after_open(l, project).0);
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
