//! 規律の手引きの参照の file の歯（接頭辞 agref_・設計ノート surface-wave29c 行 ag-ref と ag-ref2 と ag-read-scope・判断の記録 ADR-63 の決定 (2) と帰結の 9 番目の項）。
//! plugin/skills/agent-discipline/reference.md が、頼みの共通の文から移した 3 項と残りの 7 項と読む範囲の 5 句と天井の (a)〜(d) と案内の幅の歯の名と群の頭の予算の字を、
//! 決めた節の塊の中に 1 度ずつ持つことを見る。手引きは天井の文で振る舞いを持たないので字を照らす。器の歯の名は scribe2/crates の
//! 歯の dir を名で探して照らす。否定の見本は今の file から句を 1 つだけ外すか動かして作る。外の依存を使わない。
#![cfg(test)]

use std::fs;
use std::path::{Path, PathBuf};

/// 参照の file の置き場（workspace の根から）。
const REFERENCE: &str = "plugin/skills/agent-discipline/reference.md";

/// 句の名と、句を置く節の番号（0 は最初の見出しの前の頭）と、節の塊の中に 1 度だけ在る字。
const CLAUSES: [(&str, u8, &str); 24] = [
    (
        "single-home",
        0,
        "席の起草の決まりの文はこの file だけに置き",
    ),
    ("chain", 3, "行は数珠つなぎにしない"),
    ("depends-why", 3, "なぜ並べられないかを節に 1 文で書く"),
    (
        "placement-order",
        3,
        "置き場の物差しの順は、領域（domain）の親に置く",
    ),
    (
        "placement-compromise",
        3,
        "節に「置き場の妥協」の 1 文を書く",
    ),
    ("goal-count", 3, "歯の本数を書かない"),
    ("doc-id", 3, "module の頭の doc に行の id を書かない"),
    ("help-width", 3, "器の入れ子の歯（接頭辞 cli_help_）"),
    ("source-teeth", 5, "振る舞いで測れない訳を節に 1 文書く"),
    ("detour", 9, "回り込まない"),
    ("tokens", 9, "token を節約する"),
    ("budget", 10, "予算は token ちょうど 150000"),
    (
        "tmp-index",
        1,
        "一時の index（GIT_INDEX_FILE）の照らしも写しの中で撃つ",
    ),
    ("tmpdir", 1, "TMPDIR は /tmp/<名>-t にする"),
    (
        "row-count",
        3,
        "行の数は contracts/<ノート>.toml の id = の行で数える",
    ),
    (
        "idle-check",
        6,
        "働きを通らない照らし（足す物が 0 の時に通る等式など）は審査で落ちる",
    ),
    ("findings", 8, "出す物の file の名に findings も使わない"),
    ("reply", 8, "最後の返事（席への知らせ）は 10 行以内"),
    (
        "write-tree",
        9,
        "試しは commit せず、git write-tree の木の hash で撃つ",
    ),
    ("scope-named", 9, "読む範囲は頼みの文が名指す"),
    (
        "scope-items",
        9,
        "手本にする前の係の出す物の file と節（1〜2 本）・要件と判断の記録の id と節・読む code の file と fn",
    ),
    (
        "no-cat",
        9,
        "file を丸ごと cat しない。grep -n で行を当て、sed -n で当てた範囲だけを読む",
    ),
    (
        "prior-outputs",
        9,
        "前の係の出す物は、頼みの文が名指した file と節だけを読む",
    ),
    ("id-grep", 9, "要件と判断の記録は、id を grep -A で読む"),
];

/// file のどこにも無い字の名と字（写さない案内の幅の値と、直す前の群の頭の予算の字）。
const ABSENT: [(&str, &str); 2] = [("width-copied", "100 字"), ("old-budget", "150000 以下")];

/// 器の案内の幅の歯の fn の頭（手引きが名指す接頭辞）。
const HELP_FN: &str = "fn cli_help_";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn reference() -> String {
    fs::read_to_string(root().join(REFERENCE)).expect("参照の file を読む")
}

/// 節 n の塊（見出しの行 ## n. から次の ## の行の前まで・0 は最初の見出しの前）。
fn section(text: &str, n: u8) -> String {
    let head = format!("## {n}. ");
    let mut inside = n == 0;
    let mut out = String::new();
    for line in text.lines() {
        if line.starts_with("## ") {
            inside = line.starts_with(&head);
        }
        if inside {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

/// 節の塊の中に 1 度だけ在らない句の名と、file に在る無いはずの字の名（表の順）。
fn faults(text: &str) -> Vec<&'static str> {
    let mut out: Vec<&'static str> = CLAUSES
        .iter()
        .filter(|(_, n, words)| section(text, *n).matches(words).count() != 1)
        .map(|(name, _, _)| *name)
        .collect();
    out.extend(
        ABSENT
            .iter()
            .filter(|(_, words)| text.contains(words))
            .map(|(name, _)| *name),
    );
    out
}

/// dir の下の .rs の file の、頭の空白を除いて字 fn cli_help_ で始まる行の数。
fn help_fns(dir: &Path) -> usize {
    let Ok(entries) = fs::read_dir(dir) else {
        return 0;
    };
    entries
        .flatten()
        .map(|entry| entry.path())
        .map(|path| {
            if path.is_dir() {
                help_fns(&path)
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                let text = fs::read_to_string(&path).unwrap_or_default();
                text.lines()
                    .filter(|line| line.trim_start().starts_with(HELP_FN))
                    .count()
            } else {
                0
            }
        })
        .sum()
}

#[test]
fn agref_reference_holds_each_clause_once_in_its_section() {
    assert_eq!(faults(&reference()), Vec::<&str>::new());
}

#[test]
fn agref_one_clause_removed_doubled_or_moved_is_named() {
    let text = reference();
    for (name, _, words) in CLAUSES {
        let removed = text.replacen(words, "", 1);
        assert_eq!(faults(&removed), vec![name], "外した {name}");
        let doubled = text.replacen(words, &format!("{words}・{words}"), 1);
        assert_eq!(faults(&doubled), vec![name], "重ねた {name}");
        let moved = format!("{removed}\n- {words}\n");
        assert_eq!(faults(&moved), vec![name], "最後の節へ動かした {name}");
    }
}

#[test]
fn agref_copied_width_and_old_budget_are_named() {
    let text = reference();
    let width = format!("{text}\n- 器の案内の行は描いて 100 字以内。\n");
    assert_eq!(faults(&width), vec!["width-copied"]);
    let old = text.replacen("ちょうど 150000", "150000 以下", 1);
    assert_eq!(faults(&old), vec!["budget", "old-budget"]);
}

#[test]
fn agref_help_width_tooth_lives_in_the_vessel() {
    let crates = root().join("scribe2/crates");
    let found: usize = fs::read_dir(&crates)
        .expect("器の crates の dir を読む")
        .flatten()
        .map(|entry| help_fns(&entry.path().join("tests")))
        .sum();
    assert!(found > 0, "器の歯の dir に {HELP_FN} の歯が無い");
}
