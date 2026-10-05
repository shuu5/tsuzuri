//! 頁の枠: 帯の下の部品・頁ごとの列と block の並び・見出しの語の鍵・class の名・mode と頁の URL。
//! 純粋な値だけを持ち、block の中身は持たない（中身は project の下の module ごとに置く）。
//! 並びと鍵と class は見本（docs/design/mock3 の index.html・ask.html・map.html・ui.css）に揃える。
//! 節点の頁: query の page=node と id で開く。
//! 帯の「戻る」: account board の窓へ戻る段の列（`back_steps`）。
//! 閉じられない窓の注記: 窓を探した結果（`back_how`）と、close が効かなかったときの注記の字（`back_note`）。
//! 頁は src/pages の下に 1 頁 1 file（判断の記録 ADR-13）: 列挙 PageId は組み立ての script が生成し、
//! ここは頁の定義（`PageDef`）から頁の枠・link・snapshot を導く（頁の変種の名を持たない）。
//! 頁の link の押し（`Press`）は、新しい窓や tab・保存でない左の押しかを決める。
//! tab と頁の切り替え・header の部品の並び・頁の定義の nav の欄は 1 枚の画面への切り替えで使われなくなり消した。

use crate::mapview::encode;
pub use crate::pages::PageId;

/// 1 つの block の枠（DOM の id・見出しの語の鍵・section の class）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Block {
    pub id: &'static str,
    pub heading: &'static str,
    pub class: &'static str,
}

/// block を縦に積む列（見本の `.stack`）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Column {
    pub class: &'static str,
    pub blocks: Vec<Block>,
}

impl PageId {
    /// URL の query の `page=` から頁を決める（id の同じ頁・無い・知らない値は home）。
    pub fn from_query(search: &str) -> Self {
        let want = param(search, "page");
        PageId::ALL
            .into_iter()
            .find(|p| Some(p.id()) == want)
            .unwrap_or(PageId::Home)
    }
}

/// 1 つの頁の定義（src/pages の下の file ごとの `PAGE`）。
#[derive(Debug, Clone, Copy)]
pub struct PageDef {
    /// 頁の見出しの語の鍵（頁の題の語）。
    pub heading: &'static str,
    /// 列を包む class。
    pub class: &'static str,
    /// 頁の列。
    pub columns: fn() -> Vec<Column>,
}

/// 1 つの頁の枠（頁の id・見出しの語の鍵・列を包む class・列）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page {
    pub id: PageId,
    pub heading: &'static str,
    pub class: &'static str,
    pub columns: Vec<Column>,
}

impl Page {
    /// block の id の並び（列の順・列の中の順）。
    pub fn block_ids(&self) -> Vec<&'static str> {
        self.columns
            .iter()
            .flat_map(|c| c.blocks.iter().map(|b| b.id))
            .collect()
    }
}

/// 列の class（見本の `.stack`）。
pub const STACK: &str = "stack";

/// 右の列の class（見本の ask.html の `aside.side.stack`）。
pub const SIDE: &str = "side stack";

/// 頁の枠（頁の定義の値のとおり）。
pub fn page(id: PageId) -> Page {
    let def = id.def();
    Page {
        id,
        heading: def.heading,
        class: def.class,
        columns: (def.columns)(),
    }
}

/// 帯の下の 1 つの部品（役・語の鍵・class・中の語の鍵）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeaderPart {
    pub part: &'static str,
    pub key: &'static str,
    pub class: &'static str,
    pub items: &'static [&'static str],
}

/// 最終の記録の部品（帯の下の右端の小さな札の時刻の chip・語の鍵は「?」の注釈の鍵）。
pub const UPDATED: HeaderPart = HeaderPart {
    part: "updated",
    key: "last_record",
    class: "chip num",
    items: &[],
};

/// 席の pill の部品（帯の下の右端の小さな札に描く・行 g-seatpill・見本の ui.js の topHTML の `.seatpill`）。
/// 中身は seatpill の module が描く。
pub const SEAT: HeaderPart = HeaderPart {
    part: "seat",
    key: "seat",
    class: "seatpill",
    items: &[],
};

/// 席の pill を出す頁か（home は block「orchestrator と口座」が同じ状態を出すので出さない・見本の init の pill.remove）。
pub fn seat_shown(page: PageId) -> bool {
    page != PageId::Home
}

/// account board の窓を新しく開くときの URL（同じ index.html の query の board が account）。
pub const ACCOUNT_URL: &str = "?board=account";

/// 「戻る」の 1 段（見本の ui.js の backToBoard）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackStep {
    /// account board の窓（`tz-account`）を前面へ。
    Front,
    /// account board を新しい窓で開く（URL の字）。
    OpenNew(&'static str),
    /// 自分の窓を閉じる。
    CloseSelf,
}

/// 「戻る」の段の列: account board の窓が在れば前面へ、無ければ `ACCOUNT_URL` を新しい窓で開き、
/// どちらの後も自分の窓を閉じる。
pub fn back_steps(account_open: bool) -> Vec<BackStep> {
    let first = if account_open {
        BackStep::Front
    } else {
        BackStep::OpenNew(ACCOUNT_URL)
    };
    vec![first, BackStep::CloseSelf]
}

/// 空の URL で開いた新しい窓の href。
pub const BLANK: &str = "about:blank";

/// 窓の名で account board の窓を探した結果（見本の ui.js の backToBoard の front・opened・blocked）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackHow {
    /// 在った窓を前面へ出した（href が読めない別の origin の窓も在った窓と見る）。
    Front,
    /// 窓が無く、空の窓を新しく開いた。
    Opened,
    /// 窓が返らなかった（popup の許可が要る）。
    Blocked,
}

/// 窓が返ったかと窓の URL（読めなければ None）から結果を決める: 返らなければ Blocked・
/// about:blank なら Opened・ほか（読めない URL を含む）は Front。
pub fn back_how(returned: bool, href: Option<&str>) -> BackHow {
    match (returned, href) {
        (false, _) => BackHow::Blocked,
        (true, Some(BLANK)) => BackHow::Opened,
        (true, _) => BackHow::Front,
    }
}

/// 自分の窓の close の後、閉じたかを見るまでの待ち（ミリ秒）。
pub const BACK_NOTE_MS: u64 = 300;

/// 閉じられない窓の注記の class（見本の `.winnote`）。
pub const BACK_NOTE: &str = "winnote";

/// 注記の太字の語の鍵。
pub const BACK_NOTE_KEY: &str = "win_close_hand";

/// 注記の小さい字: 探した結果と、閉じられない理由。
pub fn back_note(how: BackHow) -> String {
    let done = match how {
        BackHow::Front => "account board の窓は前面に出した",
        BackHow::Opened => "account board を新しい窓で開いた",
        BackHow::Blocked => "account board の窓を開けなかった（popup の許可が要る）",
    };
    format!("{done}・この窓は script が開いた窓でないので閉じられない")
}

/// 題の字（project の名）。
pub const BRAND: &str = "tsuzuri";

/// 表示の型（初心者 = 説明の「?」を出す・経験者 = 型の名を出し「?」を出さない）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Beginner,
    Expert,
}

impl Mode {
    pub const ALL: [Mode; 2] = [Mode::Beginner, Mode::Expert];

    /// URL の query の `mode=` から決める（無い・知らない値は初心者）。
    pub fn from_query(search: &str) -> Self {
        match param(search, "mode") {
            Some("expert") => Mode::Expert,
            _ => Mode::Beginner,
        }
    }

    /// URL と語彙の鍵に使う名（`beginner` / `expert`）。
    pub fn key(self) -> &'static str {
        match self {
            Mode::Beginner => "beginner",
            Mode::Expert => "expert",
        }
    }

    /// body の class（見本の ui.css は `body.mode-beginner .q` で「?」を出す）。
    pub fn body_class(self) -> &'static str {
        match self {
            Mode::Beginner => "mode-beginner",
            Mode::Expert => "mode-expert",
        }
    }
}

/// 頁への link（同じ path の query だけ・mode を URL に残す）。
pub fn href(page: PageId, mode: Mode) -> String {
    if page == PageId::Home {
        format!("?mode={}", mode.key())
    } else {
        format!("?page={}&mode={}", page.id(), mode.key())
    }
}

/// 頁の link の押し（mouse の button の番号と修飾の鍵・行 g-nav）。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Press {
    pub button: i16,
    pub ctrl: bool,
    pub meta: bool,
    pub shift: bool,
    pub alt: bool,
}

impl Press {
    /// 左の button（番号 0）で、修飾の鍵（ctrl・meta・shift・alt）のどれも無い押し（新しい窓や tab・保存の押しでない）。
    pub fn plain(self) -> bool {
        self.button == 0 && !(self.ctrl || self.meta || self.shift || self.alt)
    }
}

/// app の窓（display-mode が standalone）かを問う media query。
pub const STANDALONE_QUERY: &str = "(display-mode: standalone)";

/// window.open の features で app の窓を開く語（普通の tab の頁から渡すと普通の popup になるので app の窓の中の時だけ渡す）。
pub const POPUP: &str = "popup";

/// 名前つきの窓を開く時の features: app の窓の中なら `POPUP`・ほかは空の字。
pub fn window_features(standalone: bool) -> &'static str {
    if standalone { POPUP } else { "" }
}

/// 節点の頁への link（節点の id は `%XX` にする・mode を URL に残す）。
pub fn node_href(id: &str, mode: Mode) -> String {
    format!("?page=node&id={}&mode={}", encode(id), mode.key())
}

/// query の 1 つの値（`?a=1&b=2` の形・無ければ None）。
pub fn param<'a>(search: &'a str, key: &str) -> Option<&'a str> {
    search
        .trim_start_matches('?')
        .split('&')
        .filter_map(|kv| kv.split_once('=').or(Some((kv, ""))))
        .find(|(k, _)| *k == key)
        .map(|(_, v)| v)
}

/// query の 1 つの値を置き換える（無ければ末尾に足す・ほかの値と順はそのまま）。
pub fn with_param(search: &str, key: &str, value: &str) -> String {
    let mut found = false;
    let mut parts: Vec<String> = search
        .trim_start_matches('?')
        .split('&')
        .filter(|kv| !kv.is_empty())
        .map(|kv| {
            let k = kv.split_once('=').map_or(kv, |(k, _)| k);
            if k == key && !found {
                found = true;
                format!("{key}={value}")
            } else {
                kv.to_string()
            }
        })
        .collect();
    if !found {
        parts.push(format!("{key}={value}"));
    }
    format!("?{}", parts.join("&"))
}

/// 頁 1 つの枠の値の JSON の字（tests/snapshots/pages の下の同じ id の file と 1 字も違わないことを歯が見る）。
pub fn page_snapshot(id: PageId) -> String {
    let mut out = page_json(&page(id));
    out.push('\n');
    out
}

fn page_json(page: &Page) -> String {
    let columns: Vec<String> = page
        .columns
        .iter()
        .map(|c| {
            let blocks: Vec<String> = c
                .blocks
                .iter()
                .map(|b| {
                    format!(
                        "            {{\"id\": {}, \"heading\": {}, \"class\": {}}}",
                        quote(b.id),
                        quote(b.heading),
                        quote(b.class)
                    )
                })
                .collect();
            format!(
                "        {{\n          \"class\": {},\n          \"blocks\": [\n{}\n          ]\n        }}",
                quote(c.class),
                blocks.join(",\n")
            )
        })
        .collect();
    format!(
        "    {{\n      \"id\": {},\n      \"heading\": {},\n      \"class\": {},\n      \"columns\": [\n{}\n      ]\n    }}",
        quote(page.id.id()),
        quote(page.heading),
        quote(page.class),
        columns.join(",\n")
    )
}

fn quote(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}
