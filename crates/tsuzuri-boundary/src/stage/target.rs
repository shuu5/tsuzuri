//! 表示先の設定（要件 FR16・判断の記録 ADR-15 の決定 (1)(2)(6)）。
//! 置き場は tsuzuri 自前の追跡されない 1 つの file（XDG の設定の dir の下の `DIR` の下の `FILE`）で、器の host の面は書かない。
//! 2 段で持つ: 全体の既定（根の鍵 `DEFAULT`）と project ごとの上書き（表 `PROJECT`）。表 `SHOWN` は端末ごとの印
//! （--to を省いた撃ちが初めて窓を起こした epoch 秒）で、印の在る端末の窓は起こし直さない（持ち主が閉じた窓は開き直さない）。
//! 書式は TOML の小さな部分（表 2 つ・鍵は引用符で囲んだ名・値は引用符で囲んだ名か数字だけの数）で、外の依存を使わず
//! 自前の読み手と書き手で読み書きする。形の外れた字は行の番号を名指して断り、黙って既定へ落とさない（条 P-7）。
//! 書きは同じ dir の一時の file に mode 0600 で書いて rename で置き換える（途中で落ちても半端な file を残さない）。
//! この file は設定を読み書きするだけで、端末の窓を前に出す・動かす語を持たない。

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::process;

/// 設定の dir の下の tsuzuri の dir の名。
pub const DIR: &str = "tsuzuri";

/// 表示先の設定の file の名。
pub const FILE: &str = "stage.toml";

/// 全体の既定の根の鍵。
pub const DEFAULT: &str = "default";

/// project ごとの上書きの表の見出し。
pub const PROJECT: &str = "[project]";

/// 初めて見せた印の表の見出し。
pub const SHOWN: &str = "[shown]";

/// 書く file の最初の行。
pub const HEAD: &str = "# tsuzuri の表示先の設定（tz stage target が書く・追跡されない file）";

/// 表示先の設定（既定・project の名から端末の名・端末の名から初めて窓を起こした epoch 秒）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Targets {
    pub default: Option<String>,
    pub projects: BTreeMap<String, String>,
    pub shown: BTreeMap<String, u64>,
}

/// 効く値の出所。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    Default,
    Project,
}

impl Origin {
    /// 出力の語。
    pub fn word(self) -> &'static str {
        match self {
            Origin::Default => "既定",
            Origin::Project => "上書き",
        }
    }
}

impl Targets {
    /// project に効く端末の名と出所（上書きが在ればそれ・無ければ既定・どちらも無ければ None）。
    pub fn effective(&self, project: &str) -> Option<(&str, Origin)> {
        match self.projects.get(project) {
            Some(name) => Some((name.as_str(), Origin::Project)),
            None => self.default.as_deref().map(|name| (name, Origin::Default)),
        }
    }

    /// project の上書きを置く（在れば替える）。
    pub fn set_project(&mut self, project: &str, name: &str) {
        self.projects.insert(project.to_string(), name.to_string());
    }

    /// 全体の既定を置き、project ごとの上書きを全部外して外した数を返す（印は変えない）。
    pub fn set_all(&mut self, name: &str) -> usize {
        self.default = Some(name.to_string());
        let dropped = self.projects.len();
        self.projects.clear();
        dropped
    }

    /// project の上書きを外す（在った時だけ真）。
    pub fn clear_project(&mut self, project: &str) -> bool {
        self.projects.remove(project).is_some()
    }
}

/// 設定の file の path（XDG_CONFIG_HOME が絶対の path ならその下、ほかは絶対の HOME の下の .config の下・
/// XDG の決まりどおり相対の値と空の値は捨てる・どちらも使えなければ None）。
pub fn path(xdg_config_home: Option<&OsStr>, home: Option<&OsStr>) -> Option<PathBuf> {
    let xdg = xdg_config_home.map(Path::new).filter(|p| p.is_absolute());
    let base = match xdg {
        Some(dir) => dir.to_path_buf(),
        None => home
            .map(Path::new)
            .filter(|p| p.is_absolute())?
            .join(".config"),
    };
    Some(base.join(DIR).join(FILE))
}

/// 設定に書ける名か（空でなく、引用符・逆斜線・字 =・制御の字を含まない）。
/// 書く字に逃がしの字を持たず、read が 1 行を最初の = で鍵と値に割っても名が割れないので、書ける名は読める。
pub fn shaped(name: &str) -> bool {
    !name.is_empty()
        && !name
            .chars()
            .any(|c| matches!(c, '"' | '\\' | '=') || c.is_control())
}

/// 引用符で囲んだ書ける名の中身。
fn quoted(v: &str) -> Option<&str> {
    v.strip_prefix('"')?
        .strip_suffix('"')
        .filter(|s| shaped(s))
}

/// 読んでいる表。
#[derive(Clone, Copy, PartialEq, Eq)]
enum Table {
    Root,
    Project,
    Shown,
}

/// 設定の字を読む（空の行と井桁で始まる行は飛ばす・表は `PROJECT` と `SHOWN` だけで 1 度ずつ・根の鍵は `DEFAULT` だけ・
/// 同じ鍵の 2 度は断る）。形の外れた字は行の番号を名指す Err。
pub fn read(text: &str) -> Result<Targets, String> {
    let mut out = Targets::default();
    let mut table = Table::Root;
    let mut seen: Vec<Table> = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let n = i + 1;
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let refuse = |why: String| format!("{n} 行目: {why}");
        if line.starts_with('[') {
            table = match line {
                PROJECT => Table::Project,
                SHOWN => Table::Shown,
                _ => return Err(refuse(format!("知らない表 {line}"))),
            };
            if seen.contains(&table) {
                return Err(refuse(format!("表 {line} が 2 度ある")));
            }
            seen.push(table);
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            return Err(refuse("鍵 = 値 の形でない".to_string()));
        };
        let (key, value) = (key.trim(), value.trim());
        let name = |v: &str| {
            quoted(v)
                .map(str::to_string)
                .ok_or_else(|| refuse(format!("値 {v} は引用符で囲んだ名でない")))
        };
        let twice = || refuse(format!("鍵 {} が 2 度ある", quoted(key).unwrap_or(key)));
        if table == Table::Root {
            if key != DEFAULT {
                return Err(refuse(format!("知らない鍵 {key}")));
            }
            if out.default.is_some() {
                return Err(twice());
            }
            out.default = Some(name(value)?);
            continue;
        }
        let Some(inner) = quoted(key) else {
            return Err(refuse(format!("鍵 {key} は引用符で囲んだ名でない")));
        };
        let fresh = if table == Table::Project {
            out.projects.insert(inner.to_string(), name(value)?).is_none()
        } else {
            let at = Some(value)
                .filter(|v| !v.is_empty() && v.bytes().all(|b| b.is_ascii_digit()))
                .and_then(|v| v.parse::<u64>().ok())
                .ok_or_else(|| refuse(format!("値 {value} は epoch 秒の数でない")))?;
            out.shown.insert(inner.to_string(), at).is_none()
        };
        if !fresh {
            return Err(twice());
        }
    }
    Ok(out)
}

/// 設定の字（`HEAD` の行・既定・空行と `PROJECT` の表・空行と `SHOWN` の表の順・空の表は書かない・名の順）。
pub fn render(targets: &Targets) -> String {
    let mut out = format!("{HEAD}\n");
    if let Some(name) = &targets.default {
        out.push_str(&format!("{DEFAULT} = \"{name}\"\n"));
    }
    if !targets.projects.is_empty() {
        out.push_str(&format!("\n{PROJECT}\n"));
        for (project, name) in &targets.projects {
            out.push_str(&format!("\"{project}\" = \"{name}\"\n"));
        }
    }
    if !targets.shown.is_empty() {
        out.push_str(&format!("\n{SHOWN}\n"));
        for (name, at) in &targets.shown {
            out.push_str(&format!("\"{name}\" = {at}\n"));
        }
    }
    out
}

/// 設定の file を読む（無ければ空の設定・ほかの読めない誤りと形の誤りは path を名指す Err）。
pub fn load(path: &Path) -> Result<Targets, String> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(Targets::default()),
        Err(e) => {
            return Err(format!("表示先の設定 {} を読めない: {e}", path.display()));
        }
    };
    read(&text).map_err(|e| {
        format!(
            "表示先の設定 {} の {e}（黙って既定へ落とさない・file を直すか消して tz stage target で書き直す）",
            path.display()
        )
    })
}

/// 設定の file を書く（dir を作り、同じ dir の一時の file に mode 0600 で書いて sync し、rename で置き換える）。
pub fn save(path: &Path, targets: &Targets) -> Result<(), String> {
    let dir = path.parent().unwrap_or(Path::new("."));
    fs::create_dir_all(dir)
        .map_err(|e| format!("表示先の設定の dir {} を作れない: {e}", dir.display()))?;
    let temp = dir.join(format!(".{FILE}.{}", process::id()));
    let _ = fs::remove_file(&temp);
    let written = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&temp)
        .and_then(|mut file| {
            file.write_all(render(targets).as_bytes())?;
            file.sync_all()
        })
        .and_then(|()| fs::rename(&temp, path));
    written.map_err(|e| {
        let _ = fs::remove_file(&temp);
        format!("表示先の設定 {} を書けない: {e}", path.display())
    })
}

/// 端末に初めて窓を起こした印を書く（読み直して印が無い時だけ足して書き、書いた時だけ真）。
pub fn mark(path: &Path, name: &str, at: u64) -> Result<bool, String> {
    let mut targets = load(path)?;
    if targets.shown.contains_key(name) {
        return Ok(false);
    }
    targets.shown.insert(name.to_string(), at);
    save(path, &targets)?;
    Ok(true)
}

/// 層 A の名を空白で並べた字（無ければ 無し）。
pub(crate) fn listed(names: &[String]) -> String {
    if names.is_empty() {
        "無し".to_string()
    } else {
        names.join(" ")
    }
}

/// 端末の名が層 A（host の面の [[device]]）に在るか（無ければ在る名を並べて断る）。
pub fn known(name: &str, names: &[String]) -> Result<(), String> {
    if names.iter().any(|n| n == name) {
        return Ok(());
    }
    Err(format!(
        "端末の名 {name} は層 A（host の面の [[device]]）に無い（在る名は {}）",
        listed(names)
    ))
}
