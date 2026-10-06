//! 欄 `patch` の差が替える定義（設計 docs/design/reverse-index.md §6 の項目 (iii)）。
//!
//! 差の file の替える base の行を、索引の表の occurrence で定義の symbol へ引く。`-` 行は、その行の定義と、行の occurrence を囲む
//! 定義を取り、どちらも無い行は行を含む定義の広がりで引く。`+` 行の塊（`-` 行の直後でない物）は、塊の前の行を含む定義の広がりで
//! 引く。定義の広がりは、file の最初の定義の行から、その定義に直に囲まれた occurrence の最後の行までである。module の symbol と
//! 字だけの行は項目にしない（局所の symbol は SCIP の読みが落とす）。test の中の定義は項目にせず数え、どの定義にも引けない `-` 行・表に 1 行も無い
//! 既存の file・新しい file を数える。I/O は持たない（差の字は呼び手が ref の木から読む）。

use crate::pipe::index::flat::Row;
use std::collections::{BTreeMap, BTreeSet};

/// 差が替える定義と数え。
#[derive(Debug, Default, PartialEq, Eq)]
pub(in crate::pipe) struct Changed {
    /// 替える定義の symbol（差の file の順と行の順・重ねない・test の中の定義を除く）。
    pub defs: Vec<String>,
    /// 替える定義のうち test の中の定義の数。
    pub tests: usize,
    /// 表に在る file の `-` 行のうち、どの定義にも引けない行の数（module の注・空の行・item の間の括弧など）。
    pub unmapped: usize,
    /// 表に 1 行も無い既存の file（差の順）。
    pub outside: Vec<String>,
    /// 新しい file の数。
    pub fresh: usize,
}

/// 差の file 1 本の替える base の行。
#[derive(Debug, Default)]
struct Touched {
    /// base 側の path。
    path: String,
    /// 新しい file か。
    fresh: bool,
    /// `-` 行の行番号。
    removed: Vec<usize>,
    /// `+` 行の塊（`-` 行の直後でない物）の前の行の行番号。
    before: Vec<usize>,
}

/// hunk 見出し `-l[,n] +l[,n] @@ …` の base の開始行。
fn hunk_base(rest: &str) -> Option<usize> {
    let (range, _) = rest.split_once(" @@")?;
    range.split(' ').next()?.strip_prefix('-')?.split(',').next()?.parse().ok()
}

/// `git diff` の本文を file ごとの替える base の行に読む（見出しの読めない hunk か file の外の hunk は `None`）。
fn touched(diff: &str) -> Option<Vec<Touched>> {
    let mut files: Vec<Touched> = Vec::new();
    let (mut at, mut last) = (None::<usize>, ' ');
    for line in diff.lines() {
        if let Some(rest) = line.strip_prefix("diff --git a/") {
            let path = rest.split_once(" b/").map_or(rest, |(base, _)| base);
            files.push(Touched { path: path.to_owned(), ..Touched::default() });
            at = None;
            continue;
        }
        let file = files.last_mut()?;
        if let Some(rest) = line.strip_prefix("@@ ") {
            at = Some(hunk_base(rest)?);
            last = ' ';
            continue;
        }
        let Some(cursor) = at.as_mut() else {
            file.fresh |= line.starts_with("new file mode ");
            continue;
        };
        match line.chars().next() {
            Some('-') => {
                file.removed.push(*cursor);
                *cursor = cursor.saturating_add(1);
            }
            Some('+') if last == ' ' => file.before.push(cursor.saturating_sub(1)),
            Some('\\') => continue,
            Some('+') => {}
            _ => *cursor = cursor.saturating_add(1),
        }
        last = line.chars().next().unwrap_or(' ');
    }
    Some(files)
}

/// 項目にできる定義の symbol か（空・module・字だけの行でない・局所の symbol は SCIP の読みが落とす）。
fn definable(symbol: &str) -> bool {
    !symbol.is_empty() && !symbol.ends_with('/') && !symbol.starts_with(crate::pipe::index::flat::TEXT_ONLY_PREFIX)
}

/// file 1 本の定義の広がり（symbol → (最初の定義の行・直に囲まれた occurrence の最後の行)・表の行は位置の順）。
fn extents(rows: &[&Row]) -> BTreeMap<String, (usize, usize)> {
    let mut found: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    for row in rows.iter().filter(|row| row.definition && definable(&row.symbol)) {
        found.entry(row.symbol.clone()).or_insert((row.line, row.line));
    }
    for row in rows {
        if let Some(span) = found.get_mut(&row.enclosing) {
            span.1 = span.1.max(row.line);
        }
    }
    found
}

/// 行を含む最も狭い広がりの定義。
fn within(spans: &BTreeMap<String, (usize, usize)>, line: usize) -> Option<&str> {
    spans
        .iter()
        .filter(|(_, (from, to))| *from <= line && line <= *to)
        .min_by_key(|(_, (from, to))| (to - from, usize::MAX - from))
        .map(|(symbol, _)| symbol.as_str())
}

/// `-` 行 1 本の定義（行の定義と行の occurrence を囲む定義・無ければ行を含む広がりの定義）。
fn removed_defs<'a>(rows: &[&'a Row], spans: &'a BTreeMap<String, (usize, usize)>, line: usize) -> Vec<&'a str> {
    let mut found: Vec<&str> = Vec::new();
    for row in rows.iter().filter(|row| row.line == line) {
        let own = (row.definition && definable(&row.symbol)).then_some(row.symbol.as_str());
        for symbol in own.into_iter().chain(definable(&row.enclosing).then_some(row.enclosing.as_str())) {
            if !found.contains(&symbol) {
                found.push(symbol);
            }
        }
    }
    if found.is_empty() {
        found.extend(within(spans, line));
    }
    found
}

/// 差 `diff` が替える定義を、索引の表 `rows` で引く（差の見出しが読めない周は `None`）。
pub(in crate::pipe) fn changed(rows: &[Row], diff: &str) -> Option<Changed> {
    let mut by_path: BTreeMap<&str, Vec<&Row>> = BTreeMap::new();
    for row in rows {
        by_path.entry(row.path.as_str()).or_default().push(row);
    }
    let tested: BTreeSet<&str> = rows.iter().filter(|row| row.definition && row.test).map(|row| row.symbol.as_str()).collect();
    let mut out = Changed::default();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    for file in touched(diff)? {
        if file.fresh {
            out.fresh += 1;
            continue;
        }
        let Some(own) = by_path.get(file.path.as_str()) else {
            out.outside.push(file.path);
            continue;
        };
        let spans = extents(own);
        let mut picked: Vec<(usize, &str)> = Vec::new();
        for line in &file.removed {
            let defs = removed_defs(own, &spans, *line);
            out.unmapped += usize::from(defs.is_empty());
            picked.extend(defs.into_iter().map(|symbol| (*line, symbol)));
        }
        picked.extend(file.before.iter().filter_map(|line| within(&spans, *line).map(|symbol| (*line, symbol))));
        picked.sort_by_key(|(line, _)| *line);
        for (_, symbol) in picked {
            if !seen.insert(symbol.to_owned()) {
                continue;
            }
            if tested.contains(symbol) {
                out.tests += 1;
            } else {
                out.defs.push(symbol.to_owned());
            }
        }
    }
    Some(out)
}
