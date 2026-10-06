//! 設計係の型と手引きの参照の file が起草の印と出す物の 4 行を持つことの歯（接頭辞 agmrk_・設計ノート surface-v4b 行 ag-drafter-marks）。型と参照の file が 4 行を決めた塊の決めた行に 1 度ずつ持つことを見る。
//! 型と手引きは係を起こす時に読む設定の字で振る舞いを持たないので字を照らす。外の依存を使わない。
#![cfg(test)]

use std::fs;
use std::path::{Path, PathBuf};

/// 設計係の型の置き場（workspace の根から）。
const DRAFTER: &str = "plugin/agents/drafter.md";

/// 参照の file の置き場（workspace の根から）。
const REFERENCE: &str = "plugin/skills/agent-discipline/reference.md";

/// 1 つ目の字（名 self-check・型の file）。
const SELF_CHECK: &str = "節の字は done で全角の二重鉤括弧『』の印で囲む。";

/// 2 つ目の字（名 output-contract・型の file）。
const OUTPUT_CONTRACT: &str = "契約の file（contract/<行 id>.toml）。行ごとに 1 file を、出す物の dir の子 contract の直下に置く。終える前の門が file ごとに preflight --contract で撃ち直す。";

/// 3 つ目の字（名 ref-mark・参照の file）。
const REF_MARK: &str = "done の項が節の字を言う時は、その字を全角の二重鉤括弧『』の印で囲んで書き、同じ字を節の本文に書く（preflight の断り section-literal が照らす）。印を節の字の外に使わない。";

/// 4 つ目の字（名 ref-contract・参照の file）。
const REF_CONTRACT: &str = "契約の行は 1 行ずつ、出す物の dir（<名>/w/）の子 contract の直下に <行 id>.toml の名で書く（字 schema = 1 と空の行と、契約表の導出の形の [[contract]] の 1 行・欄 goal に節の本文）。終える前の門は頭の出す物の欄を見ず、その直下の名の末 .toml の file を全部 preflight --contract で撃ち直し、通らない file を名指して 1 度目の終わりを止める。";

/// 名・file・見出しの頭の字・行の頭の字・前の行の頭の字。
const LINES: [(&str, &str, &str, &str, Option<&str>); 4] = [
    (
        "self-check",
        DRAFTER,
        "## 自己検査",
        "4. 節と done の字の一致。",
        None,
    ),
    ("output-contract", DRAFTER, "## 出力", "- ", None),
    (
        "ref-mark",
        REFERENCE,
        "## 2.",
        "- ",
        Some("- done が節の字や値を言う時は"),
    ),
    ("ref-contract", REFERENCE, "## 8.", "- ", None),
];

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(path: &str) -> String {
    fs::read_to_string(root().join(path)).expect("file を読む")
}

/// 名の行の字。
fn line_text(name: &str) -> &'static str {
    match name {
        "self-check" => SELF_CHECK,
        "output-contract" => OUTPUT_CONTRACT,
        "ref-mark" => REF_MARK,
        "ref-contract" => REF_CONTRACT,
        _ => panic!("表に無い名: {name}"),
    }
}

/// 字 head で始まる行から、次の字 ## と空白で始まる行の前までの行を改行で繋いだ字。その行が無ければ空の字。
fn block(text: &str, head: &str) -> String {
    text.lines()
        .skip_while(|l| !l.starts_with(head))
        .enumerate()
        .take_while(|(i, l)| *i == 0 || !l.starts_with("## "))
        .map(|(_, l)| l)
        .collect::<Vec<_>>()
        .join("\n")
}

/// 表の順に、崩れた項の名を 1 度ずつ。
fn faults(drafter: &str, reference: &str) -> Vec<&'static str> {
    LINES
        .iter()
        .filter(|(name, file, head, lead, prev)| {
            let text = if *file == DRAFTER { drafter } else { reference };
            let line = line_text(name);
            let lines: Vec<&str> = text.lines().collect();
            let at = lines.iter().position(|l| l.contains(line));
            text.matches(line).count() != 1
                || block(text, head).matches(line).count() != 1
                || at.is_none_or(|i| {
                    !lines[i].starts_with(lead)
                        || !lines[i].ends_with(line)
                        || prev.is_some_and(|p| i == 0 || !lines[i - 1].starts_with(p))
                })
        })
        .map(|(name, ..)| *name)
        .collect()
}

/// 項の file だけを f で崩した 2 file の字。
fn broken(name: &str, f: impl Fn(&str) -> String) -> (String, String) {
    let (drafter, reference) = (read(DRAFTER), read(REFERENCE));
    let file = LINES.iter().find(|l| l.0 == name).expect("表に在る名").1;
    if file == DRAFTER {
        (f(&drafter), reference)
    } else {
        (drafter, f(&reference))
    }
}

/// 行の字を空の字に 1 度だけ替えた字。
fn removed(text: &str, name: &str) -> String {
    text.replacen(line_text(name), "", 1)
}

/// 行の字を、行の字と字・と行の字を繋いだ字に 1 度だけ替えた字。
fn doubled(text: &str, name: &str) -> String {
    let line = line_text(name);
    text.replacen(line, &format!("{line}・{line}"), 1)
}

/// 外した字の最初の字 ## と空白で始まる行の前に、字 - と空白と行の字と改行の 1 行を入れた字。
fn moved(text: &str, name: &str) -> String {
    let rest = removed(text, name);
    let at = rest
        .lines()
        .scan(0, |pos, l| {
            let start = *pos;
            *pos += l.len() + 1;
            Some((start, l))
        })
        .find(|(_, l)| l.starts_with("## "))
        .map(|(start, _)| start)
        .expect("見出しの行が在る");
    let mut out = rest;
    out.insert_str(at, &format!("- {}\n", line_text(name)));
    out
}

#[test]
fn agmrk_types_hold_the_four_lines() {
    assert_eq!(faults(&read(DRAFTER), &read(REFERENCE)), Vec::<&str>::new());
}

#[test]
fn agmrk_line_removed_doubled_or_moved_is_named() {
    for (name, ..) in LINES {
        for (how, make) in [
            ("外した", removed as fn(&str, &str) -> String),
            ("2 度にした", doubled),
            ("動かした", moved),
        ] {
            let (drafter, reference) = broken(name, |t| make(t, name));
            assert_eq!(faults(&drafter, &reference), vec![name], "{name} を{how}");
        }
    }
}

#[test]
fn agmrk_line_moved_inside_its_block_is_named() {
    let anchor = "5. 環境や時刻に依る振る舞いは値で測る。";
    let (drafter, reference) = broken("self-check", |t| {
        let rest = removed(t, "self-check");
        assert_eq!(rest.matches(anchor).count(), 1);
        rest.replacen(anchor, &format!("{anchor}{SELF_CHECK}"), 1)
    });
    assert_eq!(faults(&drafter, &reference), vec!["self-check"]);

    let (drafter, reference) = broken("ref-mark", |t| {
        let rest = t.replacen(&format!("- {REF_MARK}\n"), "", 1);
        assert_eq!(rest.matches("## 3.").count(), 1);
        rest.replacen("## 3.", &format!("- {REF_MARK}\n## 3."), 1)
    });
    assert_eq!(faults(&drafter, &reference), vec!["ref-mark"]);
}
