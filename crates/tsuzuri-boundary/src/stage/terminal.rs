//! 端末の一覧の読み（行 i-1・要件 FR16・判断の記録 ADR-5 の決定 (2)）。
//! 器の host の面（state dir の追跡されない `host.toml`）の表 `[[device]]` の行を読み、端末の名で 1 行を引く。
//! 端末の値は code に焼かず（条 N-7）、TOML の部品を使わず中核の host.rs の読み手と同じ自前の読み方で読む。
//! 本文を受ける `lookup` と `names` は file も子の process も環境変数も触らず、file を読むのは `read_face` だけ
//! （面の置き場が移れば `read_face` と `FACE` だけを替える）。面の全体の形は器の validate が持ち、ここは引いた行だけを検める。

use std::fs;
use std::path::Path;

/// 端末の 1 行の表の見出し。
pub const HEADER: &str = "[[device]]";

/// 器の host の面の file の名（state dir の下）。
pub const FACE: &str = "host.toml";

/// 端末の 1 行の key（器の閉じた集合と同じ・必須は name・ssh・chrome・os、表示面では profile-dir も要る）。
pub const KEYS: [&str; 7] = [
    "name",
    "ssh",
    "chrome",
    "os",
    "display",
    "ime-env",
    "profile-dir",
];

/// 端末の OS（起動の argv の組み分けはこの値でだけ行う・条 N-7）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Os {
    Linux,
    Macos,
    Windows,
}

impl Os {
    pub const ALL: [Os; 3] = [Os::Linux, Os::Macos, Os::Windows];

    /// host の面の os の値の字。
    pub fn word(self) -> &'static str {
        match self {
            Os::Linux => "linux",
            Os::Macos => "macos",
            Os::Windows => "windows",
        }
    }
}

/// 引いた端末の 1 行（display は X の DISPLAY・ime-env は宣言の順の KEY と VALUE の組）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Terminal {
    pub name: String,
    pub ssh: String,
    pub chrome: String,
    pub os: Os,
    pub display: Option<String>,
    pub ime_env: Vec<(String, String)>,
    pub profile_dir: String,
}

/// 本文の `[[device]]` の行の列（宣言の順・行ごとに前後の空白を除き、空の行と # で始まる行を除いた字）。
/// ほかの見出しは別の表を始め、別の表の中の行と最初の見出しより前の行は読まない。
fn rows(host_toml: &str) -> Vec<Vec<&str>> {
    let mut rows: Vec<Vec<&str>> = Vec::new();
    let mut inside = false;
    for line in host_toml.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') {
            inside = line.split('#').next().unwrap_or(line).trim() == HEADER;
            if inside {
                rows.push(Vec::new());
            }
            continue;
        }
        match rows.last_mut() {
            Some(row) if inside => row.push(line),
            _ => {}
        }
    }
    rows
}

/// 行の key と値（最初の = の前後・前後の空白を除く）。
fn entry(line: &str) -> Option<(&str, &str)> {
    line.split_once('=').map(|(k, v)| (k.trim(), v.trim()))
}

/// 引用符 1 組で囲んだ字の中身（中に引用符を持たない・逃がしの字は解かずにそのまま読む）。
fn quoted(v: &str) -> Option<&str> {
    v.strip_prefix('"')?
        .strip_suffix('"')
        .filter(|s| !s.contains('"'))
}

/// 行の name の値（最初の name の行の値が引用符 1 組で囲んだ字のときだけ）。
fn row_name<'a>(row: &[&'a str]) -> Option<&'a str> {
    row.iter()
        .copied()
        .filter_map(entry)
        .find(|(k, _)| *k == "name")
        .and_then(|(_, v)| quoted(v))
}

/// 本文の端末の名の列（宣言の順・同じ名もそのまま・name の読めない行は数えない）。
pub fn names(host_toml: &str) -> Vec<String> {
    rows(host_toml)
        .iter()
        .filter_map(|r| row_name(r))
        .map(str::to_string)
        .collect()
}

/// 端末の名で 1 行を引く。名が無いか 2 つ在るか、引いた行の形が外れれば Err（引いた名と断った key を角括弧で囲んだ字の 1 行）。
pub fn lookup(host_toml: &str, name: &str) -> Result<Terminal, String> {
    let rows = rows(host_toml);
    let mut hits = rows.iter().filter(|r| row_name(r) == Some(name));
    match (hits.next(), hits.next()) {
        (Some(row), None) => check(name, row),
        (None, _) => {
            let known = names(host_toml);
            let known = if known.is_empty() {
                "なし".to_string()
            } else {
                known.join("・")
            };
            Err(format!(
                "端末 {name} は host の面の {HEADER} に無い（在る名: {known}・器の host の面に行を足すか在る名で引く）"
            ))
        }
        (Some(_), Some(_)) => Err(format!(
            "端末 {name} は host の面の {HEADER} に 2 つ在る（器の host の面の行の名を 1 つにする）"
        )),
    }
}

/// 断りの 1 行（引いた名と断った key を角括弧で囲んだ字・次の 1 手を添える）。
fn refuse(name: &str, key: &str, why: &str) -> String {
    format!("端末 {name} の [{key}] {why}（器の host の面の {HEADER} の行を直す）")
}

/// X の DISPLAY の形（: の後に 10 進の数字・その後に . と 10 進の数字が続いてもよい）。
fn x_display(v: &str) -> bool {
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    v.strip_prefix(':')
        .is_some_and(|rest| match rest.split_once('.') {
            Some((d, s)) => digits(d) && digits(s),
            None => digits(rest),
        })
}

/// ime-env の値（1 行で閉じた角括弧の中のコンマで分けた引用符 1 組の字の列・各要素は KEY=VALUE）。
/// KEY は英大文字と数字と下線だけで先頭が数字でなく、VALUE は空でなく、同じ KEY は 1 度だけ。外れれば None。
fn ime_pairs(v: &str) -> Option<Vec<(String, String)>> {
    let inner = v.strip_prefix('[')?.strip_suffix(']')?.trim();
    let mut out: Vec<(String, String)> = Vec::new();
    if inner.is_empty() {
        return Some(out);
    }
    for item in inner.split(',') {
        let (key, value) = quoted(item.trim())?.split_once('=')?;
        let shaped = key.chars().next().is_some_and(|c| !c.is_ascii_digit())
            && key
                .chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_');
        if !shaped || value.is_empty() || out.iter().any(|(k, _)| k == key) {
            return None;
        }
        out.push((key.to_string(), value.to_string()));
    }
    Some(out)
}

/// 引いた行を検めて端末にする（ほかの行の形は見ない）。
fn check(name: &str, row: &[&str]) -> Result<Terminal, String> {
    let mut seen: Vec<(&str, &str)> = Vec::new();
    for line in row.iter().copied() {
        let Some((key, value)) = entry(line) else {
            return Err(refuse(name, line, "は = の無い行"));
        };
        if !KEYS.contains(&key) {
            return Err(refuse(name, key, "は端末の行の欄でない（器の閉じた集合の外・欄を足さない）"));
        }
        if seen.iter().any(|(k, _)| *k == key) {
            return Err(refuse(name, key, "が 2 度在る"));
        }
        if key != "ime-env" && !matches!(quoted(value), Some(s) if !s.is_empty()) {
            return Err(refuse(
                name,
                key,
                &format!("の値 {value} は引用符 1 組で囲んだ空でない字でない"),
            ));
        }
        seen.push((key, value));
    }
    let get = |key: &str| seen.iter().find(|(k, _)| *k == key).map(|(_, v)| *v);
    let text = |key: &str| get(key).and_then(quoted).map(str::to_string);
    let need = |key: &str| text(key).ok_or_else(|| refuse(name, key, "が無い"));
    let ssh = need("ssh")?;
    let chrome = need("chrome")?;
    let os_word = need("os")?;
    if ssh.contains(char::is_whitespace) || ssh.starts_with('-') {
        return Err(refuse(
            name,
            "ssh",
            &format!("の値 {ssh} は空白を含むか - で始まる（ssh の旗と読まれる）"),
        ));
    }
    let os = Os::ALL
        .into_iter()
        .find(|o| o.word() == os_word)
        .ok_or_else(|| {
            let words: Vec<&str> = Os::ALL.iter().map(|o| o.word()).collect();
            refuse(
                name,
                "os",
                &format!("の値 {os_word} は {} のどれでもない", words.join("・")),
            )
        })?;
    let display = text("display");
    if let Some(d) = display.as_deref().filter(|d| !x_display(d)) {
        return Err(refuse(
            name,
            "display",
            &format!("の値 {d} は X の DISPLAY の形でない（Wayland の画面の環境は器の欄 display-env を待つ）"),
        ));
    }
    let profile_dir = text("profile-dir").ok_or_else(|| {
        refuse(
            name,
            "profile-dir",
            "が無い（表示面は既定の profile を補わない・remote debugging の口は専用の profile-dir で開く）",
        )
    })?;
    let ime_env = match get("ime-env") {
        None => Vec::new(),
        Some(v) => ime_pairs(v).ok_or_else(|| {
            refuse(
                name,
                "ime-env",
                &format!("の値 {v} は 1 行で閉じた KEY=VALUE の字の配列でない（KEY は英大文字と数字と下線で先頭が数字でない・VALUE は空でない・KEY は 1 度だけ）"),
            )
        })?,
    };
    Ok(Terminal {
        name: name.to_string(),
        ssh,
        chrome,
        os,
        display,
        ime_env,
        profile_dir,
    })
}

/// state dir の下の `FACE` を UTF-8 の字として読む（中身は検めない・読めなければ path の字を含む Err）。
pub fn read_face(state_dir: &Path) -> Result<String, String> {
    let path = state_dir.join(FACE);
    fs::read_to_string(&path)
        .map_err(|e| format!("host の面 {} を読めない: {e}（state dir と器の host の面を確かめる）", path.display()))
}
