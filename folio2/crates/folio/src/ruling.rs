//! 決定の欄の裁定 id（便 181・docs/design/delivery-181.md §1・判断の記録 ADR-31 決定 (1)(3)・責務の層 1 読む）。
//! 裁定 id の文法（`rulings`・欄の字から全部を切り出し、各々に形の種類 `Form` を付ける）と、決定の欄（骨格が書く欄
//! `SKELETON`・数えない役 `SKIP_ROLES`）と、正本の木から決定の欄を 1 つずつ拾う歩き手（`sites`）を持つ。
//! 床の判定（違反・まだ分からない）は `check.rs` の `check_rulings` が持ち、書き出し（ADR-31 決定 (4)・便 186・`emit`）は
//! 同じ歩き手と同じ関数を使う（行の番号だけは欄の鍵の行を file の字の event の印で引く）。凍結 anchor の承認一覧と、欄の決まりの外の置き場の印（`floor.rs`）も同じ関数で
//! 裁定 id の在否を見る（`has_ruling`）。正規表現は使わない（字の走査・文法の字は ASCII だけ）。
//! 文法の字面 `PATTERN`・形の種類・決定の欄の閉じた一覧 `FIELDS`・骨格の欄・数えない役は、判断の記録の欄の決まり
//! （adr/schema.yaml）の生成区間に写る（便 182・`floor_adr.rs` の FLOOR）。

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use yaml_rust2::parser::{Event, MarkedEventReceiver, Parser};
use yaml_rust2::scanner::Marker;

use crate::yaml::{json_str, Node};

/// 裁定 id の文法の写し（人が読む字面・床は字の走査で判定する）。語頭の台帳の id と、続く器の問いの印か notes の日時。
pub(crate) const PATTERN: &str = r"(?<![0-9A-Za-z_.-])[a-z][0-9]-[0-9a-z]+(\.[0-9]+)*(:[0-9]{8}T[0-9]{4}Z-[0-9]+| notes( [0-9]{4}-[0-9]{2}-[0-9]{2}( [0-9]{2}:[0-9][0-9x])?| [0-9]{2}:[0-9][0-9x])( JST)?)?";

/// 決定の欄の閉じた一覧（file と欄の道・ADR-31 決定 (1)）。判断の表の行（決定 (2)・`TABLE`）は便 183 で足した。
pub(crate) const FIELDS: [&str; 12] = [
    ENACTMENT, AMENDMENT, THRESHOLD, DISCIPLINE, RECORD, NOTE, STAMPS[0], STAMPS[1], STAMPS[2], STAMPS[3],
    STAMPS[4], TABLE,
];

const ENACTMENT: &str = "constitution.yaml meta.approval.ruling";
const AMENDMENT: &str = "constitution.yaml articles[].amended_by[].ruling";
const THRESHOLD: &str = "rules.yaml thresholds[].ruling";
const DISCIPLINE: &str = "rules.yaml discipline[].ruling";
const RECORD: &str = "adr/ADR-*.yaml approval.ruling";
const NOTE: &str = "design-note/*.yaml meta.approval[].ruling";
/// 判断の表（節の型 decision-table・どの設計ノートにも置ける）の各行の裁定の欄（便 183・ADR-31 決定 (2)(イ)(エ)）。
const TABLE: &str = "design-note/*.yaml sections[decision-table].rows[].ruling";

/// 承認欄の行の stamp を数える 5 正本（要件書・入口・天井の正本・相談窓口・索引の欄の決まり）。
const STAMPS: [&str; 5] = [
    "srs.yaml meta.approval[].stamp",
    "index.yaml meta.approval[].stamp",
    "ceiling.yaml meta.approval[].stamp",
    "intake.yaml meta.approval[].stamp",
    "graph.yaml meta.approval[].stamp",
];

/// 骨格（folio init）が書く欄。値が骨格の印（未記入）なら裁定の前（P-17.3）＝床は まだ分からない とする。
pub(crate) const SKELETON: [&str; 3] = [ENACTMENT, THRESHOLD, DISCIPLINE];

/// 承認欄の行（meta.approval[]）のうち数えない役（決定を持たない行）。
pub(crate) const SKIP_ROLES: [&str; 2] = ["作成", "レビュー"];

/// 裁定 id の形の種類（閉じた一覧・ADR-31 決定 (3)）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Form {
    /// 器の問いの形（台帳の id に「:」年月日 T 時分 Z「-」連番）
    Question,
    /// 台帳の id に notes の日付か時刻（と JST）
    NotesTime,
    /// 台帳の id だけ
    Bead,
}

impl Form {
    /// 形の種類の名（書き出しの form の字・欄の決まりの生成区間に写す字）。
    pub(crate) const NAMES: [&'static str; 3] = ["question", "notes-time", "bead"];

    pub(crate) fn name(self) -> &'static str {
        Self::NAMES[self as usize]
    }
}

/// 欄の字から切り出した裁定 id 1 つ（字は欄の字のまま・`bead` は台帳の id の部分）。書き出し（便 186）が 3 つとも読む。
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Ruling<'a> {
    pub(crate) text: &'a str,
    pub(crate) form: Form,
    pub(crate) bead: &'a str,
}

/// 欄の字から裁定 id を全部、出てくる順に切り出す（ADR-31 決定 (3)）。語頭（頭か、前の字が英数字・「_」・「-」・「.」で
/// ない位置）の台帳の id（英小字 1・数字 1・「-」・英小字か数字の並び・「.数字」の段を何段でも）を 1 つとし、直後に
/// 器の問いの印が続けば問いの形、notes の日付か時刻（分の 1 桁は x でもよい）と任意の JST が続けば日時の形、
/// どちらも無ければ台帳の id だけの形。切り出した字の終わりから続けて探す。
pub(crate) fn rulings(s: &str) -> Vec<Ruling<'_>> {
    let b = s.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        let Some(end) = bead_end(b, i) else {
            i += 1;
            continue;
        };
        let (stop, form) = match question_end(b, end) {
            Some(q) => (q, Form::Question),
            None => notes_end(b, end).map_or((end, Form::Bead), |n| (n, Form::NotesTime)),
        };
        out.push(Ruling {
            text: &s[i..stop],
            form,
            bead: &s[i..end],
        });
        i = stop;
    }
    out
}

/// 裁定 id を 1 つ以上切り出せるか（床の形の判定・凍結 anchor の承認一覧・外の置き場の印が使う）。
pub(crate) fn has_ruling(s: &str) -> bool {
    !rulings(s).is_empty()
}

/// 語をつなぐ字（語頭の判定・前の字がこれなら語頭でない）。ASCII でない字の byte はどれも当たらない。
fn joins(c: u8) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, b'_' | b'-' | b'.')
}

/// `i` から続く字の終わり（`f` に当たる字の並び）。
fn run(b: &[u8], i: usize, f: impl Fn(u8) -> bool) -> usize {
    b[i.min(b.len())..].iter().take_while(|&&c| f(c)).count() + i.min(b.len())
}

/// `i` からちょうど `n` 字の数字なら、その終わり。
fn digits(b: &[u8], i: usize, n: usize) -> Option<usize> {
    let end = i.checked_add(n)?;
    (end <= b.len() && b[i..end].iter().all(u8::is_ascii_digit)).then_some(end)
}

/// `i` が字 `lit` で始まれば、その終わり。
fn lit(b: &[u8], i: usize, lit: &str) -> Option<usize> {
    b.get(i..)?.starts_with(lit.as_bytes()).then_some(i + lit.len())
}

/// 語頭の台帳の id の終わり。
fn bead_end(b: &[u8], i: usize) -> Option<usize> {
    if i > 0 && joins(b[i - 1]) {
        return None;
    }
    let head = b.get(i..i + 3)?;
    if !(head[0].is_ascii_lowercase() && head[1].is_ascii_digit() && head[2] == b'-') {
        return None;
    }
    let mut end = run(b, i + 3, |c| c.is_ascii_lowercase() || c.is_ascii_digit());
    if end == i + 3 {
        return None;
    }
    while b.get(end) == Some(&b'.') && b.get(end + 1).is_some_and(u8::is_ascii_digit) {
        end = run(b, end + 1, |c| c.is_ascii_digit());
    }
    Some(end)
}

/// 器の問いの印（「:」年月日 8 桁「T」時分 4 桁「Z-」連番）の終わり。
fn question_end(b: &[u8], i: usize) -> Option<usize> {
    let i = digits(b, lit(b, i, ":")?, 8)?;
    let i = digits(b, lit(b, i, "T")?, 4)?;
    let i = lit(b, i, "Z-")?;
    let end = run(b, i, |c| c.is_ascii_digit());
    (end > i).then_some(end)
}

/// notes の日時（「 notes」に「 年-月-日」か「 時:分」か両方・任意の「 JST」）の終わり。日付も時刻も無ければ None。
fn notes_end(b: &[u8], i: usize) -> Option<usize> {
    let mut end = lit(b, i, " notes")?;
    let date = lit(b, end, " ").and_then(|j| {
        let j = digits(b, lit(b, digits(b, j, 4)?, "-")?, 2)?;
        digits(b, lit(b, j, "-")?, 2)
    });
    if let Some(j) = date {
        end = j;
    }
    let time = lit(b, end, " ").and_then(|j| {
        let j = digits(b, lit(b, digits(b, j, 2)?, ":")?, 1)?;
        b.get(j).is_some_and(|&c| c.is_ascii_digit() || c == b'x').then_some(j + 1)
    });
    if let Some(j) = time {
        end = j;
    }
    if date.is_none() && time.is_none() {
        return None;
    }
    Some(lit(b, end, " JST").unwrap_or(end))
}

/// 拾った決定の欄 1 つ（一覧の項・file・欄の在り処の字・file の根からの欄の道〔一覧の番号は 0 始まり〕・欄を持つ索引の
/// 節点の id〔条・規則の表の行・判断の記録・節点の無い欄は None〕・値〔無い欄は None〕）。
pub(crate) struct Site<'a> {
    pub(crate) field: &'static str,
    pub(crate) file: String,
    pub(crate) at: String,
    pub(crate) path: String,
    pub(crate) node: Option<&'a str>,
    pub(crate) value: Option<&'a Node>,
}

impl Site<'_> {
    /// 骨格が書く欄か。
    pub(crate) fn skeleton(&self) -> bool {
        SKELETON.contains(&self.field)
    }
}

/// 歩き手が読む正本の木（読めたもの）。`graph` は索引の欄の決まり（床の 7 本の外・無ければ None）、`records` は判断の記録
/// （id と木）、`notes` は設計ノート（置き場からの相対の file 名と木）。
pub(crate) struct Tree<'a> {
    pub(crate) constitution: &'a Node,
    pub(crate) rules: &'a Node,
    pub(crate) srs: &'a Node,
    pub(crate) index: &'a Node,
    pub(crate) ceiling: &'a Node,
    pub(crate) intake: &'a Node,
    pub(crate) graph: Option<&'a Node>,
    pub(crate) records: &'a [(String, Node)],
    pub(crate) notes: Vec<(String, &'a Node)>,
}

/// 一覧の項の file 名（欄の道の前の語）。
fn file_of(field: &str) -> &str {
    field.split(' ').next().unwrap_or(field)
}

/// 決定の欄を一覧の順に全部拾う（ADR-31 決定 (1)）。欄を持つ入れ物（承認欄の表・行・承認欄の行）が在れば、欄が無くても
/// 1 つに数える（値 None）。入れ物が表でない・一覧でない形は拾わない（各 file の形の床が数える）。
pub(crate) fn sites<'a>(tree: &Tree<'a>) -> Vec<Site<'a>> {
    let mut out = Vec::new();
    let mut push = |field: &'static str, file: &str, at: String, path: String, node: Option<&'a str>, value: Option<&'a Node>| {
        out.push(Site {
            field,
            file: file.to_string(),
            at,
            path,
            node,
            value,
        });
    };
    let c = tree.constitution;
    if let Some(ap) = c.get("meta").and_then(|m| m.get("approval")).filter(|a| a.as_map().is_some()) {
        let path = "meta.approval.ruling";
        push(ENACTMENT, file_of(ENACTMENT), path.into(), path.into(), None, ap.get("ruling"));
    }
    for (a, article) in rows(c.get("articles")) {
        let id = article.get("id").and_then(Node::as_str);
        for (n, (k, am)) in rows(article.get("amended_by")).enumerate() {
            let at = format!("条 {} の amended_by[{n}].ruling", id.unwrap_or("?"));
            push(AMENDMENT, file_of(AMENDMENT), at, format!("articles[{a}].amended_by[{k}].ruling"), id, am.get("ruling"));
        }
    }
    for (field, section) in [(THRESHOLD, "thresholds"), (DISCIPLINE, "discipline")] {
        for (k, row) in rows(tree.rules.get(section)) {
            let id = row.get("id").and_then(Node::as_str);
            let at = format!("行 {} の ruling", id.unwrap_or("?"));
            push(field, file_of(field), at, format!("{section}[{k}].ruling"), id, row.get("ruling"));
        }
    }
    for (id, record) in tree.records {
        if let Some(ap) = record.get("approval").filter(|a| a.as_map().is_some()) {
            let path = "approval.ruling";
            push(RECORD, &format!("adr/{id}.yaml"), path.into(), path.into(), Some(id), ap.get("ruling"));
        }
    }
    for (file, root) in &tree.notes {
        for (n, row) in approval_rows(root) {
            let path = format!("meta.approval[{n}].ruling");
            push(NOTE, file, path.clone(), path, None, row.get("ruling"));
        }
    }
    for (file, root) in &tree.notes {
        let tables = rows(root.get("sections"))
            .filter(|(_, s)| s.get("type").and_then(Node::as_str) == Some(crate::floor_note::DECISION_TABLE));
        for (s, section) in tables {
            let n = section.get("n").and_then(Node::as_str).unwrap_or("?");
            for (k, row) in rows(section.get("rows")) {
                let id = row.get("id").and_then(Node::as_str).unwrap_or("?");
                let path = format!("sections[{s}].rows[{k}].ruling");
                push(TABLE, file, format!("§{n} の行 {id} の ruling"), path, None, row.get("ruling"));
            }
        }
    }
    let stamped = [Some(tree.srs), Some(tree.index), Some(tree.ceiling), Some(tree.intake), tree.graph];
    for (field, root) in STAMPS.into_iter().zip(stamped) {
        for (n, row) in root.into_iter().flat_map(approval_rows) {
            let role = row.get("role").and_then(Node::as_str);
            if !role.is_some_and(|r| SKIP_ROLES.contains(&r)) {
                let path = format!("meta.approval[{n}].stamp");
                push(field, file_of(field), path.clone(), path, None, row.get("stamp"));
            }
        }
    }
    out
}

/// 一覧の中の表の項と一覧の番号（一覧でなければ 0 個・表でない項は飛ばす・番号は飛ばす前の番号）。
fn rows(node: Option<&Node>) -> impl Iterator<Item = (usize, &Node)> {
    node.and_then(Node::as_seq)
        .unwrap_or_default()
        .iter()
        .enumerate()
        .filter(|(_, n)| n.as_map().is_some())
}

/// meta.approval の行（一覧の番号と表）。
fn approval_rows(root: &Node) -> impl Iterator<Item = (usize, &Node)> {
    rows(root.get("meta").and_then(|m| m.get("approval")))
}

/// 裁定 id の書き出し（ADR-31 決定 (4)・便 186）。歩き手が拾った欄の値（字だけ）から `rulings` で切り出した裁定 id を
/// 全部、欄の順・切り出した順に 1 件 1 行の JSON にする。欄は ruling・form・bead・node（節点の無い欄は null）・file
/// （置き場の根からの相対）・line（欄の鍵の行・1 始まり・引けなければ 0）・field（file の根からの欄の道）の順で空白を
/// 挟まない。値が字でない・無い欄は出さない（床の違反）。
pub(crate) fn emit(dir: &Path, sites: &[Site]) -> Vec<String> {
    let mut keys: HashMap<&str, HashMap<String, usize>> = HashMap::new();
    let mut out = Vec::new();
    for site in sites {
        let Some(Node::Scalar(s)) = site.value else {
            continue;
        };
        let line = keys
            .entry(&site.file)
            .or_insert_with(|| key_lines(&dir.join(&site.file)))
            .get(&site.path)
            .copied()
            .unwrap_or_default();
        for r in rulings(s) {
            let mut o = String::new();
            for (key, value) in [("ruling", Some(r.text)), ("form", Some(r.form.name())), ("bead", Some(r.bead)), ("node", site.node)] {
                o.push_str(if o.is_empty() { "{\"" } else { ",\"" });
                o.push_str(key);
                o.push_str("\":");
                match value {
                    Some(v) => json_str(v, &mut o),
                    None => o.push_str("null"),
                }
            }
            o.push_str(",\"file\":");
            json_str(&site.file, &mut o);
            o.push_str(&format!(",\"line\":{line},\"field\":"));
            json_str(&site.path, &mut o);
            o.push('}');
            out.push(o);
        }
    }
    out
}

/// file の欄の道ごとの鍵の行（読めない file は空・床が まだ分からない にする）。
fn key_lines(path: &Path) -> HashMap<String, usize> {
    fs::read_to_string(path).map(|t| lines_of(&t)).unwrap_or_default()
}

/// 字の欄の道ごとの鍵の行（1 始まり・同じ表の 2 度目の鍵は数えない＝読み手が最初の値を残すのと同じ）。parse できない字は
/// 読めた所まで（床が違反か まだ分からない にする）。
fn lines_of(text: &str) -> HashMap<String, usize> {
    let mut lines = Lines::default();
    let _ = Parser::new_from_str(text).load(&mut lines, true);
    lines.at
}

/// 欄の道の 1 段（一覧は読んでいる項の番号・表は読んでいる値の鍵〔鍵を待つ間は None〕）。
enum Step {
    Seq(usize),
    Map(Option<String>),
}

/// 鍵の行を読む event の受け手（床の読み手 `yaml.rs` と同じ yaml-rust2 の event と印）。
#[derive(Default)]
struct Lines {
    stack: Vec<Step>,
    at: HashMap<String, usize>,
}

impl Lines {
    /// 今の欄の道（`a.b[0].c` の形）。
    fn path(&self) -> String {
        let mut p = String::new();
        for step in &self.stack {
            match step {
                Step::Seq(n) => p.push_str(&format!("[{n}]")),
                Step::Map(Some(k)) => {
                    if !p.is_empty() {
                        p.push('.');
                    }
                    p.push_str(k);
                }
                Step::Map(None) => {}
            }
        }
        p
    }

    /// 値を 1 つ読み終えた（一覧なら次の番号へ・表なら次の鍵を待つ）。
    fn done(&mut self) {
        match self.stack.last_mut() {
            Some(Step::Seq(n)) => *n += 1,
            Some(Step::Map(k)) => *k = None,
            None => {}
        }
    }
}

impl MarkedEventReceiver for Lines {
    fn on_event(&mut self, ev: Event, mark: Marker) {
        match ev {
            Event::Scalar(key, ..) if matches!(self.stack.last(), Some(Step::Map(None))) => {
                self.stack.pop();
                self.stack.push(Step::Map(Some(key)));
                let path = self.path();
                self.at.entry(path).or_insert(mark.line());
            }
            Event::Scalar(..) | Event::Alias(_) => self.done(),
            Event::SequenceStart(..) => self.stack.push(Step::Seq(0)),
            Event::MappingStart(..) => self.stack.push(Step::Map(None)),
            Event::SequenceEnd | Event::MappingEnd => {
                self.stack.pop();
                self.done();
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cut(s: &str) -> Vec<(&str, &str, &str)> {
        rulings(s)
            .into_iter()
            .map(|r| (r.form.name(), r.text, r.bead))
            .collect()
    }

    /// 歯（便 181）: 3 つの形を 1 つの欄から全部、出てくる順に切り出す（AC28 の書き出しの 3 行と同じ字）。
    #[test]
    fn f181_the_three_forms_are_all_cut_in_order() {
        let s = "t3-hub.56:20260927T2259Z-1・f2-648 notes 2026-09-28 07:18 JST（t3-hub.1）";
        assert_eq!(
            cut(s),
            [
                ("question", "t3-hub.56:20260927T2259Z-1", "t3-hub.56"),
                ("notes-time", "f2-648 notes 2026-09-28 07:18 JST", "f2-648"),
                ("bead", "t3-hub.1", "t3-hub.1"),
            ]
        );
        // tsuzuri の形（覚え書きの台帳の id が先・問いの形は後）も全部拾う
        let t = "t3-hub.1（裁定 id = t3-hub.58:20260927T2357Z-1・board）";
        assert_eq!(
            cut(t),
            [
                ("bead", "t3-hub.1", "t3-hub.1"),
                ("question", "t3-hub.58:20260927T2357Z-1", "t3-hub.58"),
            ]
        );
    }

    /// 歯（便 181）: 語頭・「.数字」の段・日付だけ・時刻だけ・分の x・JST の有無・notes の日時の無い notes。
    #[test]
    fn f181_the_grammar_reads_word_heads_steps_and_notes_times() {
        assert_eq!(cut("s2-07l.214.3 user 2026-09-24T22:39Z"), [("bead", "s2-07l.214.3", "s2-07l.214.3")]);
        assert_eq!(cut("f2-648.1 notes 20:2x（発効承認）"), [("notes-time", "f2-648.1 notes 20:2x", "f2-648.1")]);
        assert_eq!(cut("f2-648.39 notes 2026-09-18・"), [("notes-time", "f2-648.39 notes 2026-09-18", "f2-648.39")]);
        assert_eq!(cut("f2-648 notes 2026-09-12 JST"), [("notes-time", "f2-648 notes 2026-09-12 JST", "f2-648")]);
        assert_eq!(cut("f2-648.2 notes 2026-09-13 09:35"), [("notes-time", "f2-648.2 notes 2026-09-13 09:35", "f2-648.2")]);
        assert_eq!(cut("（f2-648 notes・f2-648.6 notes）"), [("bead", "f2-648", "f2-648"), ("bead", "f2-648.6", "f2-648.6")]);
        // 時刻の分が 2 桁でない・問いの印が欠ける は台帳の id だけ
        assert_eq!(cut("f2-648 notes 20:2"), [("bead", "f2-648", "f2-648")]);
        assert_eq!(cut("t3-hub.56:20260927T2259Z"), [("bead", "t3-hub.56", "t3-hub.56")]);
        assert_eq!(cut("f2-648.x"), [("bead", "f2-648", "f2-648")]);
        // 語頭でない（前が英数字・「_」・「-」・「.」）・英大字・「-」の後が空・台帳の id の形でない
        for none in ["xf2-648", "_f2-648", "a-f2-648", "v.f2-648", "F2-648", "f2-", "f2-Ab", "G16=A（受入 (f)）", "未記入", "", "ab1-x"] {
            assert!(!has_ruling(none), "{none}");
        }
        for some in ["(f2-648)", "=t3-hub.1", "裁定f2-648", "#s2-07l.149"] {
            assert!(has_ruling(some), "{some}");
        }
    }

    /// 歯（便 181）: 骨格の欄は憲法の発効の承認と規則の表の 2 節だけで、数えない役は 作成 と レビュー だけ。
    #[test]
    fn f181_the_skeleton_and_the_skipped_roles_are_closed() {
        assert_eq!(SKELETON, [ENACTMENT, THRESHOLD, DISCIPLINE]);
        assert_eq!(SKIP_ROLES, ["作成", "レビュー"]);
        assert_eq!([Form::Question, Form::NotesTime, Form::Bead].map(Form::name), Form::NAMES);
    }

    /// 歯（便 186）: 欄の道ごとの鍵の行は、段の形・流れの形（行をまたぐ表と引用符の鍵を含む）・一覧の中の一覧で鍵の書かれた
    /// 行を指し、値が段の字（|）でも鍵の行を指す。同じ表の 2 度目の鍵は数えない。
    #[test]
    fn f186_key_lines_follow_block_and_flow_maps() {
        let text = "meta:\n  approval:\n    - {role: 作成, stamp: 起草}\n    - role: 承認\n      stamp: f2-648\nrows:\n  - {id: R-1,\n     \"ruling\": t3-hub.1}\n  - [a, {ruling: x}]\nruling: |\n  f2-1\nruling: 2 度目\n";
        let at = lines_of(text);
        let want = [("meta.approval[0].stamp", 3), ("meta.approval[1].stamp", 5), ("rows[0].id", 7), ("rows[0].ruling", 8), ("rows[1][1].ruling", 9), ("ruling", 10)];
        for (path, line) in want {
            assert_eq!(at.get(path), Some(&line), "{path}");
        }
        assert_eq!(at.len(), 11, "{at:?}");
    }

    /// 歯（便 182）: 決定の欄の一覧は 12 種類で重ならず、骨格の欄と 5 正本の stamp の欄を含む（便 183 で判断の表の行を足した）。
    #[test]
    fn f182_the_fields_are_closed_and_hold_the_skeleton() {
        let mut seen = FIELDS.to_vec();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen.len(), 12);
        assert!(SKELETON.iter().chain(&STAMPS).all(|f| FIELDS.contains(f)));
    }
}
