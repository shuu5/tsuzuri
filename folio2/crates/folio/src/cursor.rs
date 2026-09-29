//! 正本の cursor（便 107・docs/design/delivery-107.md §1 (b)・ADR-15 決定 (2)(3)・責務の層 1 読む）。取り出しの結果の型 `R`・
//! 正本 1 file を型付きで読む `load`・型付きの木を欄の道つきで辿る `X`・字面の 5 字の逃がし `esc`・id の形 `safe_id` を持つ。
//! `face.rs`（便 14）から字を変えずに降ろした。正本の byte を型に直して辿るだけで、HTML の骨格も名札も知らない。

use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::yaml::{self, Value};

pub type R<T> = Result<T, String>;

thread_local! {
    /// 読んだ正本の写しと「読めない」の字（`memo` のあいだだけ在る・便 187）。
    static MEMO: RefCell<Option<Memo>> = const { RefCell::new(None) };
}

#[derive(Default)]
struct Memo {
    trees: HashMap<PathBuf, Value>,
    unreadable: Vec<String>,
}

/// `f` のあいだ、`load` は同じ file を 2 度目からは読み直さず、1 度目に読めた木の写しを返す（床が面を組むとき、
/// 面ごとに同じ正本を読み直さない・便 187）。読めなかった file は覚えない。返すのは `f` の値と、そのあいだに
/// `unreadable` へ渡った「読めない」の字の一覧（床の面の段が まだ分からない と形の誤りを分ける・P-4.2）。
pub fn memo<T>(f: impl FnOnce() -> T) -> (T, Vec<String>) {
    MEMO.with(|m| *m.borrow_mut() = Some(Memo::default()));
    let out = f();
    let memo = MEMO.with(|m| m.borrow_mut().take()).unwrap_or_default();
    (out, memo.unreadable)
}

/// 「読めない」（fs の読みの失敗・UTF-8 でない・YAML として読めない・file でない）の字。`memo` のあいだは覚えて返す
/// （便 187）。重複キーのような形の誤りには使わない。
pub fn unreadable(msg: String) -> String {
    MEMO.with(|m| {
        if let Some(m) = m.borrow_mut().as_mut() {
            m.unreadable.push(msg.clone());
        }
    });
    msg
}

/// 正本 1 file を型付きで読む。無い・読めない・UTF-8 でない・重複キー・空の文書は Err（まだ分からない）。
pub fn load(dir: &Path, name: &str) -> R<Value> {
    let key = dir.join(name);
    if let Some(v) = MEMO.with(|m| m.borrow().as_ref().and_then(|m| m.trees.get(&key).cloned())) {
        return Ok(v);
    }
    let v = read(dir, name)?;
    MEMO.with(|m| {
        if let Some(m) = m.borrow_mut().as_mut() {
            m.trees.insert(key, v.clone());
        }
    });
    Ok(v)
}

fn read(dir: &Path, name: &str) -> R<Value> {
    let bytes = fs::read(dir.join(name)).map_err(|e| unreadable(format!("{name}: 読めない: {e}")))?;
    let text = String::from_utf8(bytes).map_err(|_| unreadable(format!("{name}: UTF-8 でない")))?;
    let doc = yaml::parse(&text).map_err(|e| unreadable(format!("{name}: 読めない: {e}")))?;
    if let Some(d) = doc.duplicates.first() {
        return Err(format!("{name}: 重複キー「{}」（{} 行）", d.key, d.line));
    }
    yaml::parse_typed(&text).map_err(|e| unreadable(format!("{name}: {e}")))
}

// ── 木を辿る口（便 11 の render.rs の X と同じ形）──

/// 型付きの木の 1 点（欄の道つき）。無い欄と型違いは文言つきの Err。
pub struct X<'a> {
    pub v: &'a Value,
    pub at: String,
}

impl<'a> X<'a> {
    pub fn root(v: &'a Value, at: &str) -> Self {
        X {
            v,
            at: at.to_string(),
        }
    }

    pub(crate) fn child(&self, v: &'a Value, seg: &str) -> Self {
        X {
            v,
            at: format!("{}{seg}", self.at),
        }
    }

    fn entries(&self) -> R<&'a [(Value, Value)]> {
        self.v
            .as_map()
            .ok_or_else(|| format!("{}: 表でない", self.at))
    }

    /// 必須の欄（無ければ Err）。
    pub fn f(&self, key: &str) -> R<X<'a>> {
        self.entries()?;
        self.v
            .get(key)
            .map(|v| self.child(v, &format!(".{key}")))
            .ok_or_else(|| format!("{}: 欄 {key} が無い", self.at))
    }

    /// 任意の欄（無い・null は None）。
    pub fn g(&self, key: &str) -> R<Option<X<'a>>> {
        self.entries()?;
        Ok(self
            .v
            .get(key)
            .filter(|v| !matches!(v, Value::Null))
            .map(|v| self.child(v, &format!(".{key}"))))
    }

    /// 一覧の要素。
    pub fn seq(&self) -> R<Vec<X<'a>>> {
        let items = self
            .v
            .as_seq()
            .ok_or_else(|| format!("{}: 一覧でない", self.at))?;
        Ok(items
            .iter()
            .enumerate()
            .map(|(i, v)| self.child(v, &format!("[{i}]")))
            .collect())
    }

    /// 表の（キー, 値）を書かれた順に（キーは文字列だけ）。
    pub fn pairs(&self) -> R<Vec<(&'a str, X<'a>)>> {
        self.entries()?
            .iter()
            .map(|(k, v)| {
                let k = k
                    .as_str()
                    .ok_or_else(|| format!("{}: 表のキーが文字列でない", self.at))?;
                Ok((k, self.child(v, &format!(".{k}"))))
            })
            .collect()
    }

    /// scalar の字面（文字列・整数・小数・日付）。null・真偽・一覧・表は Err。
    pub fn text(&self) -> R<String> {
        match self.v {
            Value::Str(_) | Value::Int(_) | Value::Float(_) | Value::Date(_) => Ok(self.v.py_str()),
            _ => Err(format!("{}: 文字列でない", self.at)),
        }
    }

    /// escape した scalar の字面。
    pub fn e(&self) -> R<String> {
        Ok(esc(&self.text()?))
    }

    /// 必須の欄の escape した字面。
    pub fn ef(&self, key: &str) -> R<String> {
        self.f(key)?.e()
    }

    /// id と href に使う id（文字列で、英数字と「-」「.」だけ）。
    pub fn id(&self) -> R<&'a str> {
        let s = self
            .v
            .as_str()
            .ok_or_else(|| format!("{}: 文字列でない", self.at))?;
        safe_id(s).map_err(|e| format!("{}: {e}", self.at))
    }

    /// 整数（0 以上）。
    pub fn count(&self) -> R<u64> {
        match self.v {
            Value::Int(n) => n
                .parse()
                .map_err(|_| format!("{}: 0 以上の整数でない", self.at)),
            _ => Err(format!("{}: 整数でない", self.at)),
        }
    }

    /// 名札の表引き（表に無い値は Err）。
    pub fn lookup<T: Copy>(&self, table: &[(&str, T)], what: &str) -> R<T> {
        if let Value::Str(s) = self.v
            && let Some((_, t)) = table.iter().find(|(k, _)| k == s)
        {
            return Ok(*t);
        }
        Err(format!(
            "{}: {what} の表に無い値「{}」",
            self.at,
            self.v.py_str()
        ))
    }

    /// 憲法の値域の表引き（正本の値 → 導出した型・便 50 (b)）。`from_name` は型の関数 from_name。値域に無い値は Err
    /// （文言は `lookup` と同じ）。
    pub fn parse<T>(&self, from_name: fn(&str) -> Option<T>, what: &str) -> R<T> {
        if let Value::Str(s) = self.v
            && let Some(t) = from_name(s)
        {
            return Ok(t);
        }
        Err(format!(
            "{}: {what} の表に無い値「{}」",
            self.at,
            self.v.py_str()
        ))
    }

    /// 機械のための面の字面（正本の値のまま・scalar だけ）。
    pub fn raw(&self) -> R<String> {
        match self.v {
            Value::Null => Ok("null".to_string()),
            Value::Bool(b) => Ok(b.to_string()),
            Value::Seq(_) | Value::Map(_) => Err(format!("{}: scalar でない", self.at)),
            other => Ok(esc(&other.py_str())),
        }
    }
}

// ── 字面の口 ──

/// 便 11 と同じ 5 字の escape。
pub fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#x27;"),
            c => out.push(c),
        }
    }
    out
}

/// id と href に使う id は英数字と「-」「.」だけ（空は受けない）。
pub fn safe_id(s: &str) -> R<&str> {
    if !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '.'))
    {
        Ok(s)
    } else {
        Err(format!("id「{s}」が英数字と「-」「.」以外を含む"))
    }
}
