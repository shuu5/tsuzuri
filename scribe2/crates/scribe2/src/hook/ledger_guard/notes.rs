//! 起票の門の notes の段（設計 docs/design/ledger-form.md §18・FR81 (b)）。
//!
//! `bd` / `bdw` の update と create の `--notes` / `--append-notes`（`--flag value` と `--flag=value`・値は次の語を無条件に取る）
//! と、`note` の本文の語と `--file` の file を notes の本文として読み、本文のどれかの行が裁定の行
//! （[`is_ruling_line`]・判定はその 1 本だけが持つ）なら止める。`--stdin`・値の無い flag・開けない file・`$` か backtick を
//! 含む値は裁定の行が無いと言えないので読めない側で止める。本文を読めた分だけを読み手へ渡し、台帳の接頭辞は裁定の行が
//! 読める周にだけ引く。
//!
//! 3 つ目の段は、読めた本文の頭の行の札の中か札の直後の UTC の時刻の字（[`head_time`]・Z の付いた 5 形だけ）の下の端が今より
//! 先の書きを止める（記帳の時刻は書いた時の事実・判断の記録 ADR-44 の決定 (3)・閾値を持たないので rules 行を読まない）。

use super::{deny, is_assignment, LedgerDecision, CLIENTS};
use crate::fleet::cli::format_utc;
use crate::fleet::epoch_of;
use crate::ledger::close_reason::is_ruling_line;
use crate::name::NAME;

/// 本文を書く flag（update と create・`--notes` は置き換え・`--append-notes` は足す）。
const SETS: [&str; 2] = ["--notes", "--append-notes"];

/// 本文を書く flag を持つ subcommand。
const FLAGGED: [&str; 2] = ["create", "update"];

/// 本文を語と file で渡す subcommand。
const NOTE: &str = "note";

/// `note` の本文の file の flag。
const FILE: &str = "--file";

/// 標準入力から読む flag。
const STDIN: &str = "--stdin";

/// 裁定の行を書く席の書きの記録の語。
const RULING_LINE: &str = "notes-ruling-line";

/// 本文を読めない書きの記録の語。
const UNREADABLE: &str = "notes-unreadable";

/// 頭の時刻が今より先の書きの記録の語。
const HEAD_FUTURE: &str = "notes-head-future";

/// 時刻の字の日付と時の形（`0` は数字 1 字・ほかは字のまま）。
const STAMP: &[u8; 14] = b"0000-00-00T00:";

/// 本文 1 つの読み（読めた字か、読めない理由）。
type Text = Result<String, String>;

/// segment の語が notes を書く口なら、subcommand と本文の読みの列を返す（それ以外は空）。
fn texts_of(words: &[String], read: &impl Fn(&str) -> Option<String>) -> Vec<(&'static str, Text)> {
    let mut rest = words.iter().skip_while(|word| is_assignment(word));
    let Some(client) = rest.next().and_then(|word| word.rsplit('/').next()) else {
        return Vec::new();
    };
    if !CLIENTS.contains(&client) {
        return Vec::new();
    }
    let rest: Vec<&str> = rest.map(String::as_str).collect();
    let Some(at) = rest.iter().position(|word| !word.starts_with('-')) else {
        return Vec::new();
    };
    let args = rest.get(at.saturating_add(1)..).unwrap_or_default();
    let name = rest.get(at).copied().unwrap_or_default();
    if let Some(sub) = FLAGGED.into_iter().find(|sub| *sub == name) {
        return flagged(args).into_iter().map(|text| (sub, text)).collect();
    }
    if name == NOTE {
        return note(args, read).into_iter().map(|text| (NOTE, text)).collect();
    }
    Vec::new()
}

/// 語を `--flag` と `=` の後ろの値に割る（flag でない語は値なし・flag の名は語のまま）。
fn split_flag(word: &str) -> (&str, Option<&str>) {
    match word.split_once('=') {
        Some((flag, value)) if word.starts_with('-') => (flag, Some(value)),
        _ => (word, None),
    }
}

/// 展開が要る字を含む値は読めない（展開の後の字を門は知らない）。
fn plain(what: &str, value: &str) -> Text {
    if value.contains(['$', '`']) {
        return Err(format!("{what} に $ か backtick（展開の後の字を門は知らない）"));
    }
    Ok(value.to_owned())
}

/// update と create の本文を書く flag の値の列（値は次の語を無条件に取る・値の無い flag は読めない）。
fn flagged(args: &[&str]) -> Vec<Text> {
    let mut found = Vec::new();
    let mut words = args.iter().copied();
    while let Some(word) = words.next() {
        let (flag, inline) = split_flag(word);
        if !SETS.contains(&flag) {
            continue;
        }
        let value = inline.or_else(|| words.next());
        found.push(value.map_or_else(|| Err(format!("{flag} の値が無い")), |text| plain(&format!("{flag} の値"), text)));
    }
    found
}

/// `note <id> [text...]` の本文（id の後の flag でない語を空白 1 つで繋いだ字）と `--file` の file と `--stdin`。
fn note(args: &[&str], read: &impl Fn(&str) -> Option<String>) -> Vec<Text> {
    let mut found = Vec::new();
    let mut bare: Vec<&str> = Vec::new();
    let mut words = args.iter().copied();
    while let Some(word) = words.next() {
        let (flag, inline) = split_flag(word);
        if !word.starts_with('-') {
            bare.push(word);
        } else if flag == STDIN && inline != Some("false") {
            found.push(Err(format!("{STDIN}（標準入力の字を門は知らない）")));
        } else if flag == FILE {
            let path = inline.or_else(|| words.next());
            found.push(path.map_or_else(|| Err(format!("{FILE} の値が無い")), |path| file_text(path, read)));
        }
    }
    let text = bare.iter().skip(1).copied().collect::<Vec<_>>().join(" ");
    if !text.is_empty() {
        found.push(plain("note の本文", &text));
    }
    found
}

/// `--file` の file の字（`-` は標準入力・`$` か backtick を含む path・開けない file は読めない）。
fn file_text(path: &str, read: &impl Fn(&str) -> Option<String>) -> Text {
    let path = path.trim();
    if path == "-" {
        return Err(format!("{FILE} が - （標準入力の字を門は知らない）"));
    }
    plain(&format!("{FILE} の path"), path)?;
    read(path).ok_or_else(|| format!("{FILE} の file を開けない"))
}

/// 本文の頭の行（最初の空でない行の頭の空白を除いた字が `[` で始まり `]` を持つ時だけ）の、札の中で最初に現れる時刻の字か、
/// 札に無ければ `]` の直後の空白を除いた頭の時刻の字と、その下の端の UNIX 秒（[`token_at`]）。
fn head_time(text: &str) -> Option<(&str, u64)> {
    let head = text.lines().map(str::trim_start).find(|line| !line.is_empty())?;
    let (tag, after) = head.strip_prefix('[')?.split_once(']')?;
    let inner = tag.char_indices().find_map(|(at, _)| {
        let rest = tag.get(at..)?;
        token_at(rest).and_then(|(len, secs)| Some((rest.get(..len)?, secs)))
    });
    inner.or_else(|| {
        let rest = after.trim_start();
        token_at(rest).and_then(|(len, secs)| Some((rest.get(..len)?, secs)))
    })
}

/// 字の頭の UTC の時刻の字の byte の長さと下の端の UNIX 秒。形は 5 つ: 分（`HH:MMZ`）・秒（`HH:MM:SSZ`）・小数秒（`HH:MM:SS.<数字>Z`・
/// 小数は捨てる）・10 分（`HH:MxZ`・分 M0）・時（`HH:xxZ`・分 00）。暦の外と Z の無い字は `None`。
fn token_at(text: &str) -> Option<(usize, u64)> {
    let bytes = text.as_bytes();
    let fits = bytes.len() > STAMP.len() && bytes.iter().zip(STAMP).all(|(found, want)| if *want == b'0' { found.is_ascii_digit() } else { found == want });
    let rest = text.get(STAMP.len()..).filter(|_| fits)?;
    let digits = |from: usize, len: usize| rest.get(from..from.saturating_add(len)).filter(|found| found.bytes().all(|byte| byte.is_ascii_digit()));
    let (minute, second, len) = if rest.starts_with("xxZ") {
        ("00".to_owned(), "00", 3)
    } else if let Some(ten) = digits(0, 1).filter(|_| rest.get(1..3) == Some("xZ")) {
        (format!("{ten}0"), "00", 3)
    } else if let Some(minute) = digits(0, 2).filter(|_| rest.get(2..3) == Some("Z")) {
        (minute.to_owned(), "00", 3)
    } else {
        let (minute, second) = (digits(0, 2)?, digits(3, 2).filter(|_| rest.get(2..3) == Some(":"))?);
        let fraction = rest.get(5..)?.strip_prefix('.').map_or(0, |tail| tail.bytes().take_while(u8::is_ascii_digit).count().saturating_add(1));
        let close = 5_usize.saturating_add(fraction);
        (rest.get(close..close.saturating_add(1)) == Some("Z") && fraction != 1).then_some(())?;
        (minute.to_owned(), second, close.saturating_add(1))
    };
    let stamp = format!("{}T{}:{minute}:{second}Z", text.get(..10)?, text.get(11..13)?);
    Some((STAMP.len().saturating_add(len), epoch_of(&stamp)?))
}

/// notes の段（裁定の行が先・読めない書きが次・頭の時刻が今より先の書きが 3 つ目・当たらなければ `None`）。`read` は cwd から
/// 解いた file の字・`prefix` は台帳の接頭辞（本文を読めた周にだけ引く）・`now` は今の UNIX 秒。
pub(super) fn judge(
    segments: &[Vec<String>],
    read: &impl Fn(&str) -> Option<String>,
    prefix: impl Fn() -> Option<String>,
    now: u64,
) -> Option<LedgerDecision> {
    let found: Vec<(&str, Text)> = segments.iter().flat_map(|words| texts_of(words, read)).collect();
    let prefix = found.iter().any(|(_, text)| text.is_ok()).then(prefix).flatten();
    let ruled = found.iter().find_map(|(sub, text)| {
        let hit = text.as_ref().is_ok_and(|text| text.lines().any(|line| is_ruling_line(line, prefix.as_deref())));
        hit.then_some(sub)
    });
    if let Some(sub) = ruled {
        let why = "裁定の行を書くのは器の口（seat ruling bind・裁定面の答えの口）だけ — 発話を bind で問いへ結ぶ";
        return Some(refuse(sub, RULING_LINE, why.to_owned()));
    }
    if let Some((sub, what)) = found.iter().find_map(|(sub, text)| text.as_ref().err().map(|what| (sub, what))) {
        let why = format!("notes の本文を読めない（{what}） — 本文を file に書いて bdw note <id> --file <path> で足す");
        return Some(refuse(sub, UNREADABLE, why));
    }
    let (sub, token) = found.iter().find_map(|(sub, text)| {
        let (token, secs) = head_time(text.as_ref().ok()?)?;
        (secs > now).then_some((sub, token))
    })?;
    let why = format!(
        "notes の頭の時刻 {token} が今の UTC {} より先 — 頭の時刻は書く直前に date -u +%Y-%m-%dT%H:%MZ の値で書く（見込みの時刻や日本時間の字に Z を付けない）",
        format_utc(now)
    );
    Some(refuse(sub, HEAD_FUTURE, why))
}

/// 断り文 1 行（既存の形・出所は ledger-form.md §18）。
fn refuse(sub: &str, what: &str, why: String) -> LedgerDecision {
    deny(what, format!("{NAME}: deny bd {sub} は起票の門が止める reason={what}（{why}・ledger-form.md §18）"))
}

#[cfg(test)]
mod tests {
    use super::{head_time, judge, texts_of, LedgerDecision, Text};
    use crate::hook::ledger_guard::segments;

    /// 歯の時計（2026-10-05T00:10:30Z の UNIX 秒）。
    const NOW: u64 = 1_791_159_030;

    /// 本文の file を 1 つだけ持つ読み手（path が `f` のとき `line`）。
    fn reads(path: &str) -> Option<String> {
        (path == "f").then(|| "batch:x | s2-1 | 逐語".to_owned())
    }

    /// command 行の最初の segment が書く本文の読み。
    fn texts(line: &str) -> Vec<Text> {
        let words = segments(line).into_iter().next().unwrap_or_default();
        texts_of(&words, &reads).into_iter().map(|(_, text)| text).collect()
    }

    /// 台帳の接頭辞を持たない周の判定の語（通すなら `None`）。
    fn what(line: &str) -> Option<String> {
        match judge(&segments(line), &reads, || None, NOW) {
            Some(LedgerDecision::Deny { what, .. }) => Some(what),
            _ => None,
        }
    }

    /// 値の対: `--flag value` と `--flag=value`・何度でも・値は次の語を無条件（`-` で始まる本文も値）。
    #[test]
    fn hook_notes_ruling_reads_flag_value_pairs() {
        let ok = |text: &str| Ok(text.to_owned());
        assert_eq!(texts("bd update s2-1 --append-notes a --notes=b"), [ok("a"), ok("b")]);
        assert_eq!(texts("X=1 scripts/bdw create t --parent s2-1 --append-notes --x --notes -y"), [ok("--x"), ok("-y")]);
        assert_eq!(texts("bdw update s2-1 --append-notes="), [ok("")]);
        assert_eq!(texts("bdw update s2-1 --design --notes=d"), [ok("d")], "ほかの flag の語は読まない");
        assert_eq!(texts("bdw update s2-1 --append-notes").len(), 1, "値の無い flag は読めない 1 件");
        assert!(texts("bdw update s2-1 --append-notes").iter().all(Result::is_err));
        assert!(texts("bdw close s2-1 --notes x").is_empty() && texts("echo bd update --notes x").is_empty());
    }

    /// note の本文: id の後の flag でない語を空白 1 つで繋ぎ、`--file` の file の字を読む。
    #[test]
    fn hook_notes_ruling_joins_note_words_and_reads_the_file() {
        let ok = |text: &str| Ok(text.to_owned());
        assert_eq!(texts("bdw note s2-1 a b 'c  d'"), [ok("a b c  d")]);
        assert_eq!(texts("bd note s2-1 --file f"), [ok("batch:x | s2-1 | 逐語")]);
        assert_eq!(texts("bd note s2-1 --file=f"), [ok("batch:x | s2-1 | 逐語")]);
        assert!(texts("bd note s2-1").is_empty(), "本文の無い note は空");
    }

    /// 読めない字: `--stdin`・値の無い flag・開けない file・`-` の file・`$` か backtick の値。
    #[test]
    fn hook_notes_ruling_marks_unreadable_forms() {
        for line in [
            "bd note s2-1 --stdin",
            "bd note s2-1 --file",
            "bd note s2-1 --file nope",
            "bd note s2-1 --file -",
            "bd note s2-1 '$(cat f)'",
            "bd note s2-1 --file '$F'",
            "bdw update s2-1 --append-notes '$(cat f)'",
            "bdw create t --parent s2-1 --notes=`x`",
        ] {
            assert_eq!(what(line).as_deref(), Some("notes-unreadable"), "{line}");
        }
        assert_eq!(what("bd note s2-1 --stdin=false ok"), None, "=false の --stdin は読めない形でない");
    }

    /// 裁定の行は読めない書きより先に当たり、裁定の行を含まない本文は通る。
    #[test]
    fn hook_notes_ruling_orders_the_words_and_passes_plain_text() {
        assert_eq!(what("bdw update s2-1 --append-notes 'a | batch:x | b'").as_deref(), Some("notes-ruling-line"));
        assert_eq!(what("bdw update s2-1 --append-notes '$(x)' --append-notes 'batch:x'").as_deref(), Some("notes-ruling-line"));
        assert_eq!(what("bd note s2-1 --file f").as_deref(), Some("notes-ruling-line"));
        assert_eq!(what("bdw update s2-1 --append-notes 'plain' --design 'batch:x'"), None);
        assert_eq!(what("bdw update s2-1 --append-notes '### 出所'"), None);
        assert_eq!(what("bdw update s2-1 --append-notes '裁定 batch:x を引く'"), None);
    }

    /// 頭の時刻の file を 2 つ持つ読み手（path が `h` のとき 2 分先の頭・`f` のとき前の日の頭）。
    fn heads(path: &str) -> Option<String> {
        match path {
            "h" => Some("[席 2026-10-05T00:12Z] x\n".to_owned()),
            "f" => Some("[席 2026-10-04T23:59Z] x\n".to_owned()),
            _ => None,
        }
    }

    /// 歯の時計で撃った判定の断りの (語, 1 行)（通すなら `None`）。
    fn head_deny(line: &str) -> Option<(String, String)> {
        match judge(&segments(line), &heads, || None, NOW) {
            Some(LedgerDecision::Deny { what, line }) => Some((what, line)),
            _ => None,
        }
    }

    /// 頭の行の時刻の字の読み: 札の中で最初に現れる字か札の直後の頭の字で、5 形を下の端の秒に読む。読まない形は正の 1 つ目から 1 句だけ外す。
    #[test]
    fn vnhead_reads_the_first_utc_token_of_the_head() {
        let at = |minute: u64, second: u64| NOW - 630 + minute * 60 + second;
        for (text, token, secs) in [
            ("[席 2026-10-05T00:12Z] x", "2026-10-05T00:12Z", at(12, 0)),
            ("[移し 席 2026-10-05T00:2xZ] x", "2026-10-05T00:2xZ", at(20, 0)),
            ("[planner 2026-10-05T00:xxZ] x", "2026-10-05T00:xxZ", at(0, 0)),
            ("[席 2026-10-05T00:11:05Z 整理] x", "2026-10-05T00:11:05Z", at(11, 5)),
            ("[席 2026-10-05T00:11:05.250Z] x", "2026-10-05T00:11:05.250Z", at(11, 5)),
            ("[keep] 2026-10-05T00:2xZ x", "2026-10-05T00:2xZ", at(20, 0)),
            ("\n  \n[席 2026-10-05T00:12Z] x", "2026-10-05T00:12Z", at(12, 0)),
            ("[席 2026-10-05T00:12Z 2026-10-05T00:30Z] x", "2026-10-05T00:12Z", at(12, 0)),
        ] {
            assert_eq!(head_time(text), Some((token, secs)), "{text}");
        }
        for text in [
            "[席 2026-10-05T00:12] x",
            "[席 2026-10-05 00:12Z] x",
            "[席 2026-10-05T00:12 JST] x",
            "[席 2026-10-05] x",
            "x [席 2026-10-05T00:12Z] x",
            "y\n[席 2026-10-05T00:12Z] x",
            "[keep] x 2026-10-05T00:2xZ",
            "[席 2026-13-05T00:12Z] x",
            "[席 2026-10-05T00:12:05.Z] x",
            "席 2026-10-05T00:12Z x",
        ] {
            assert_eq!(head_time(text), None, "{text}");
        }
    }

    /// 下の端が今より後の頭は 4 つの口で notes-head-future、今と同じか前の頭と読まない頭は通す。
    #[test]
    fn vnhead_future_heads_are_refused_and_present_or_past_heads_pass() {
        let word = |line: &str| head_deny(line).map(|(what, _)| what);
        for line in [
            "bdw update s2-1 --append-notes '[席 2026-10-05T00:12Z] x'",
            "bdw update s2-1 --append-notes '[席 2026-10-05T09:10Z] 日本時間に Z'",
            "bdw update s2-1 --append-notes='[移し 席 2026-10-05T00:2xZ] x'",
            "bdw update s2-1 --append-notes '[席 2026-10-05T00:10:31Z] x'",
            "bdw update s2-1 --append-notes '[keep] 2026-10-05T00:2xZ x'",
            "bdw create t --parent s2-1 --notes '[席 2026-10-05T00:12Z] x'",
            "bdw note s2-1 '[席 2026-10-05T00:12Z]' x",
            "bd note s2-1 --file h",
        ] {
            assert_eq!(word(line).as_deref(), Some("notes-head-future"), "{line}");
        }
        for line in [
            "bdw update s2-1 --append-notes '[席 2026-10-05T00:10Z] x'",
            "bdw update s2-1 --append-notes '[席 2026-10-05T00:1xZ] x'",
            "bdw update s2-1 --append-notes '[席 2026-10-05T00:10:30Z] x'",
            "bdw update s2-1 --append-notes '[席 2026-10-04T23:59Z] x'",
            "bdw update s2-1 --append-notes '[席 2026-10-05T00:12] x'",
            "bdw update s2-1 --append-notes 'x [席 2026-10-05T00:12Z] x'",
            "bdw update s2-1 --design '[席 2026-10-05T00:12Z] x'",
            "bd note s2-1 --file f",
        ] {
            assert_eq!(word(line), None, "{line}");
        }
    }

    /// 断りの 1 行は時刻の字と今の UTC と date の経路を持ち、同じ command の裁定の行と読めない書きが先に当たる。
    #[test]
    fn vnhead_refusal_names_the_token_and_the_date_route_after_the_other_words() {
        let (what, line) = head_deny("bdw update s2-1 --append-notes '[席 2026-10-05T00:12Z] x'").unwrap_or_default();
        assert_eq!(what, "notes-head-future");
        for needle in ["deny bd update は起票の門が止める reason=notes-head-future", "2026-10-05T00:12Z", "2026-10-05T00:10:30Z", "date -u +%Y-%m-%dT%H:%MZ"] {
            assert!(line.contains(needle), "{needle}: {line}");
        }
        let future = "--append-notes '[席 2026-10-05T00:12Z] x'";
        let first = |line: &str| head_deny(line).map(|(what, _)| what);
        assert_eq!(first(&format!("bdw update s2-1 {future} --append-notes 'a | batch:x | b'")).as_deref(), Some("notes-ruling-line"));
        assert_eq!(first(&format!("bdw update s2-1 {future} --append-notes '$(x)'")).as_deref(), Some("notes-unreadable"));
    }
}
