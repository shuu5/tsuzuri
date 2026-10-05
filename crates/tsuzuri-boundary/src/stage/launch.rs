//! 端末の Chrome を起こす引数と tunnel の引数と窓の頁の選び（要件 FR16・判断の記録 ADR-5 の決定 (1)・ADR-15 の決定 (4)・ADR-24 の決定 (2)）。
//! 純粋な関数だけを持ち、file も子の process も環境変数も触らない（撃つのは `tunnel`）。
//! 起動の引数は端末の行の os の値でだけ組み分け、host の名を見ない（条 N-7）。profile の dir は層 A の行の字のままで、
//! code に既定の dir を持たない。窓の位置の旗を持たない（置き場は持ち主に任せる）。

use std::path::Path;

use super::json;
use super::terminal::{Os, Terminal};
use super::url::Board;

/// 端末の 127.0.0.1 だけで待つ remote debugging の port（表示面の専用の profile の Chrome だけが使う・端末ごとの値でない）。
pub const PORT: u16 = 9224;

/// 起こす窓の大きさ（2026-09-25 の実証の値・タイル型の窓の管理では効かない）。
pub const WINDOW_SIZE: &str = "900,760";

/// 鍵だけで入り、接続の待ちを 10 秒で切り、席の口座の接続の共有を使わない（閉じれば転送も消える・見積りの T1）。
const SSH_OPTIONS: [&str; 6] = [
    "-o",
    "BatchMode=yes",
    "-o",
    "ConnectTimeout=10",
    "-o",
    "ControlPath=none",
];

/// 遠くの shell の 1 語（一重引用符で包み、中の一重引用符は閉じて逃がして開き直す）。
pub fn quote(word: &str) -> String {
    format!("'{}'", word.replace('\'', r"'\''"))
}

/// 起こす Chrome に渡す env の組（display の DISPLAY・display-env・ime-env の順・どれも宣言の順）。
/// 同じ KEY が 2 度出れば Err（ime-env と display-env の間の重なりは lookup が断らないのでここで断る）。
pub fn launch_env(terminal: &Terminal) -> Result<Vec<(String, String)>, String> {
    let display = terminal
        .display
        .iter()
        .map(|d| ("DISPLAY".to_string(), d.clone()));
    let pairs = display
        .chain(terminal.display_env.iter().cloned())
        .chain(terminal.ime_env.iter().cloned());
    let mut out: Vec<(String, String)> = Vec::new();
    for (key, value) in pairs {
        if out.iter().any(|(k, _)| *k == key) {
            return Err(format!(
                "端末 {} の画面の env と ime-env が同じ KEY {key} を 2 度持つ（器の host の面の行の KEY を 1 度にする）",
                terminal.name
            ));
        }
        out.push((key, value));
    }
    Ok(out)
}

/// Chrome の argv（専用の profile・app mode・remote debugging の port・初めの問いを出さない・窓の大きさ）。
pub fn chrome_argv(terminal: &Terminal, url: &str) -> Vec<String> {
    vec![
        terminal.chrome.clone(),
        format!("--app={url}"),
        format!("--user-data-dir={}", terminal.profile_dir),
        format!("--remote-debugging-port={PORT}"),
        "--no-first-run".to_string(),
        "--no-default-browser-check".to_string(),
        format!("--window-size={WINDOW_SIZE}"),
    ]
}

/// 窓を起こす ssh の program の後の引数（遠くの shell の 1 語で setsid の -f で切り離し、入出力を捨てて待たずに返る）。
/// 形の外の URL・linux でない端末・画面の欄の無い行・/ か ~/ で始まらない profile-dir は断る。
pub fn launch_argv(terminal: &Terminal, url: &str) -> Result<Vec<String>, String> {
    let name = &terminal.name;
    let shaped = (url.starts_with("http://") || url.starts_with("https://"))
        && !url.chars().any(|c| c.is_whitespace() || c.is_control());
    if !shaped {
        return Err(format!(
            "URL {url:?} は http:// か https:// で始まる空白と制御の字の無い字でない（任意の script を URL で運ばない）"
        ));
    }
    if terminal.os != Os::Linux {
        return Err(format!(
            "端末 {name} の [os] {} の端末の窓を起こす形をまだ持たない（起こせるのは linux の端末だけ）",
            terminal.os.word()
        ));
    }
    if terminal.display.is_none() && terminal.display_env.is_empty() {
        return Err(format!(
            "端末 {name} の行に [display] も [display-env] も無い（器の host の面の行に画面の欄を書く）"
        ));
    }
    let profile = &terminal.profile_dir;
    if !(profile.starts_with('/') || profile.starts_with("~/")) {
        return Err(format!(
            "端末 {name} の [profile-dir] の値 {profile} は / か ~/ で始まらない（器の host の面の行を直す）"
        ));
    }
    let env = launch_env(terminal)?;
    let mut words: Vec<String> = ["setsid", "-f", "env"].into_iter().map(quote).collect();
    words.extend(env.iter().map(|(k, v)| quote(&format!("{k}={v}"))));
    for (i, word) in chrome_argv(terminal, url).iter().enumerate() {
        // 一重引用符の中の ~ は解かれないので、~ だけを遠くの shell の HOME に替える。
        words.push(match (i, profile.strip_prefix("~/")) {
            (2, Some(rest)) => format!(
                "{}\"$HOME\"{}",
                quote("--user-data-dir="),
                quote(&format!("/{rest}"))
            ),
            _ => quote(word),
        });
    }
    let mut argv: Vec<String> = SSH_OPTIONS.iter().map(|w| w.to_string()).collect();
    argv.push("--".to_string());
    argv.push(terminal.ssh.clone());
    argv.push(format!("{} </dev/null >/dev/null 2>&1", words.join(" ")));
    Ok(argv)
}

/// 端末の 127.0.0.1 の PORT を socket の path へ引く ssh の引数（転送に失敗すれば ssh が終わる・Os で分けない）。
pub fn tunnel_argv(terminal: &Terminal, socket: &Path) -> Vec<String> {
    let mut argv = vec![
        "-N".to_string(),
        "-o".to_string(),
        "ExitOnForwardFailure=yes".to_string(),
    ];
    argv.extend(SSH_OPTIONS.iter().map(|w| w.to_string()));
    argv.push("-L".to_string());
    argv.push(format!("{}:127.0.0.1:{PORT}", socket.display()));
    argv.push("--".to_string());
    argv.push(terminal.ssh.clone());
    argv
}

/// /json/list の本文の最初の頁の target の websocket の path（URL を見ない・頁の無い本文・配列でない本文は None）。
pub fn page_resource(list: &str) -> Option<String> {
    json::items(list)?
        .into_iter()
        .find_map(page_item)
        .map(|(path, _)| path)
}

/// /json/list の項が頁の target なら、websocket の path と URL（url が字でなければ空の字）。
fn page_item(item: &str) -> Option<(String, String)> {
    let kind = json::member(item, "type").and_then(json::unquote)?;
    let socket = json::member(item, "webSocketDebuggerUrl").and_then(json::unquote)?;
    let rest = socket.strip_prefix("ws://")?;
    let path = &rest[rest.find('/')?..];
    if kind != "page" || !path.starts_with("/devtools/page/") {
        return None;
    }
    let url = json::member(item, "url")
        .and_then(json::unquote)
        .unwrap_or_default();
    Some((path.to_string(), url))
}

/// /json/list の本文の頁のうち、URL が board の shows の頁（自分の board で account board の頁でない）の最初の項の
/// websocket の path（行 i-board-win・頁の無い本文・配列でない本文は None）。
pub fn board_page(list: &str, board: &Board) -> Option<String> {
    json::items(list)?
        .into_iter()
        .filter_map(page_item)
        .find_map(|(path, url)| board.shows(&url).then_some(path))
}

/// 窓の頁を選ぶ（覚えた path の頁が一覧に在ればその path を URL によらず・無ければ board_page の頁・行 i-board-win）。
/// 頁でない項の path を覚えていても選ばない。
pub fn pick(list: &str, remembered: Option<&str>, board: &Board) -> Option<String> {
    if let Some(path) = remembered
        && json::items(list)?
            .into_iter()
            .filter_map(page_item)
            .any(|(found, _)| found == path)
    {
        return Some(path.to_string());
    }
    board_page(list, board)
}

/// /json/list の本文の、websocket の path が path の項の URL（項が無いか url が字でなければ None・行 i-stage-guard）。
pub fn page_url(list: &str, path: &str) -> Option<String> {
    json::items(list)?.into_iter().find_map(|item| {
        let socket = json::member(item, "webSocketDebuggerUrl").and_then(json::unquote)?;
        let rest = socket.strip_prefix("ws://")?;
        let found = &rest[rest.find('/')?..];
        if found != path {
            return None;
        }
        json::member(item, "url").and_then(json::unquote)
    })
}
