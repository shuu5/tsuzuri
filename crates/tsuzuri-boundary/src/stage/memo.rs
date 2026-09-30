//! 端末の窓の覚え（行 i-board-win・要件 FR16・判断の記録 ADR-24 の決定 (2)）。
//! 端末ごとの file（口座の dir の下の tzst-端末の名.win）に、project の名と窓の頁の target の path を 1 行ずつ
//! 字 tab で分けて覚える。頁の target の id は navigate で変わらないので、席が窓を開発中の app の頁へ移しても同じ窓を引ける。
//! 形の外の行は読まずに捨て、同じ名の行が 2 つ在れば後の行が勝つ。書くのは path が変わる時だけで、mode 0600 の file に置き換える。

use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Write};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process;

/// 覚える頁の path の頭（websocket の path の頁の target）。
pub const PAGE_PREFIX: &str = "/devtools/page/";

/// 端末の名の覚えの file の path（base の下の tzst-<名>.win・錠の file と同じ名の形の外は None）。
pub fn path(base: &Path, terminal: &str) -> Option<PathBuf> {
    let shaped = !terminal.is_empty()
        && !terminal.starts_with('.')
        && terminal
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'));
    shaped.then(|| base.join(format!("tzst-{terminal}.win")))
}

/// project の名の形（空でなく、字 tab と制御の字を含まない）。
fn name_shaped(name: &str) -> bool {
    !name.is_empty() && !name.chars().any(char::is_control)
}

/// 頁の path の形（/devtools/page/ で始まり、その後が空でなく、空白と制御の字を含まない）。
fn page_shaped(page: &str) -> bool {
    page.strip_prefix(PAGE_PREFIX).is_some_and(|id| !id.is_empty())
        && !page.chars().any(|c| c.is_whitespace() || c.is_control())
}

/// file の字を (project の名, 頁の path) の列にする（tab の数の違う行・空の名の行・形の外の path の行は捨て、
/// 同じ名の行が 2 つ在れば後の行が勝つ）。
pub fn read(text: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    for line in text.lines() {
        let mut parts = line.split('\t');
        let (Some(name), Some(page), None) = (parts.next(), parts.next(), parts.next()) else {
            continue;
        };
        if !name_shaped(name) || !page_shaped(page) {
            continue;
        }
        out.retain(|(known, _)| known != name);
        out.push((name.to_string(), page.to_string()));
    }
    out
}

/// 列を file の字にする（1 行に名・字 tab・path・改行）。
pub fn render(entries: &[(String, String)]) -> String {
    entries
        .iter()
        .map(|(name, page)| format!("{name}\t{page}\n"))
        .collect()
}

/// project の名で覚えた頁の path。
pub fn get<'a>(entries: &'a [(String, String)], project: &str) -> Option<&'a str> {
    entries
        .iter()
        .find(|(name, _)| name == project)
        .map(|(_, page)| page.as_str())
}

/// file に覚えた project の頁の path（file が無いか読めなければ None）。
pub fn recall(file: &Path, project: &str) -> Option<String> {
    let text = fs::read_to_string(file).ok()?;
    get(&read(&text), project).map(str::to_string)
}

/// file に project の頁の path を覚える（書いたら真・同じ path なら書かずに偽）。
/// tab などの制御の字を含むか空の名と、形の外の path は書かずに Err。file は mode 0600 で置き換える。
pub fn put(file: &Path, project: &str, page: &str) -> Result<bool, String> {
    if !name_shaped(project) {
        return Err(format!(
            "project の名 {project:?} は窓の覚えに書けない（空でなく、字 tab と制御の字を含まない名）"
        ));
    }
    if !page_shaped(page) {
        return Err(format!(
            "頁の path {page:?} は窓の覚えに書けない（{PAGE_PREFIX} で始まる空白と制御の字の無い字）"
        ));
    }
    let mut entries = match fs::read_to_string(file) {
        Ok(text) => read(&text),
        Err(e) if e.kind() == ErrorKind::NotFound => Vec::new(),
        Err(e) => return Err(format!("窓の覚え {} を読めない: {e}", file.display())),
    };
    if get(&entries, project) == Some(page) {
        return Ok(false);
    }
    entries.retain(|(name, _)| name != project);
    entries.push((project.to_string(), page.to_string()));
    let mut temp = file.as_os_str().to_os_string();
    temp.push(format!(".{}", process::id()));
    let temp = PathBuf::from(temp);
    let written = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(&temp)
        .and_then(|mut f| {
            f.set_permissions(fs::Permissions::from_mode(0o600))?;
            f.write_all(render(&entries).as_bytes())
        })
        .and_then(|()| fs::rename(&temp, file));
    if let Err(e) = written {
        let _ = fs::remove_file(&temp);
        return Err(format!("窓の覚え {} を書けない: {e}", file.display()));
    }
    Ok(true)
}
