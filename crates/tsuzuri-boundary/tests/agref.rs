//! 規律の手引きの参照の file の歯（接頭辞 agref_・設計ノート surface-wave29c 行 ag-ref と ag-ref2 と ag-read-scope・surface-wave29d 行 ag-guide-fold・判断の記録 ADR-63 の決定 (2) と帰結の 9 番目の項・設計ノート surface-v4a 行 ag-designer・判断の記録 ADR-72 の決定 (3)・契約の bead 行 ag-bead-draft・判断の記録 ADR-74 の決定 (1)(6)）。
//! plugin/skills/agent-discipline/reference.md が、設計係に残す 46 句を決めた節の塊の中に 1 度ずつ持ち、外した実装と道具の手順の 14 の語と写さない案内の幅の値と直す前の群の頭の予算の字を持たず、見出しが 8 節であることを見る。
//! 決めた節の塊の中に 1 度ずつ持つことを見る。手引きは天井の文で振る舞いを持たないので字を照らす。器の歯の名は scribe2/crates の
//! 歯の dir を名で探して照らす。否定の見本は今の file から句を 1 つだけ外すか動かして作る。外の依存を使わない。
#![cfg(test)]

use std::fs;
use std::path::{Path, PathBuf};

/// 参照の file の置き場（workspace の根から）。
const REFERENCE: &str = "plugin/skills/agent-discipline/reference.md";

/// 参照の file の見出しの行（字 ## と空白で始まる行・この順）。
const HEADS: [&str; 8] = [
    "## 1. 置き場",
    "## 2. 契約の行の書き方",
    "## 3. 歯の置き方（段の束ね・判断の記録 ADR-32）",
    "## 4. 否定の見本",
    "## 5. 出す物と公開",
    "## 6. 道具の撃ち方と読む範囲",
    "## 7. 検証の群（判断の記録 ADR-61）",
    "## 8. 床と受付の照らし",
];

/// 句の名と、句を置く節の番号（0 は最初の見出しの前の頭）と、節の塊の中に 1 度だけ在る字。
const CLAUSES: [(&str, u8, &str); 46] = [
    (
        "single-home",
        0,
        "席の起草の決まりの文はこの file だけに置き",
    ),
    ("chain", 2, "行は数珠つなぎにしない"),
    ("depends-why", 2, "なぜ並べられないかを節に 1 文で書く"),
    (
        "placement-order",
        2,
        "置き場の物差しの順は、領域（domain）の親に置く",
    ),
    (
        "placement-compromise",
        2,
        "節に「置き場の妥協」の 1 文を書く",
    ),
    ("goal-count", 2, "歯の本数を書かない"),
    ("doc-id", 2, "module の頭の doc に行の id を書かない"),
    ("help-width", 2, "器の入れ子の歯（接頭辞 cli_help_）"),
    ("source-teeth", 3, "振る舞いで測れない訳を節に 1 文書く"),
    ("detour", 6, "回り込まない"),
    ("tokens", 6, "token を節約する"),
    ("budget", 7, "予算は token ちょうど 150000"),
    (
        "idle-check",
        4,
        "働きを通らない照らし（足す物が 0 の時に通る等式・実装が作る物を種で先に置く toy など）は審査で落ちる",
    ),
    ("findings", 5, "出す物の file の名に findings も使わない"),
    ("reply", 5, "最後の返事（席への知らせ）は 10 行以内"),
    (
        "write-tree",
        6,
        "試しは commit せず、git write-tree の木の hash で撃つ",
    ),
    ("scope-named", 6, "読む範囲は頼みの文が名指す"),
    (
        "scope-items",
        6,
        "手本にする前の係の出す物の file と節（1〜2 本）・要件と判断の記録の id と節・読む code の file と fn",
    ),
    (
        "no-cat",
        6,
        "file を丸ごと cat しない。grep -n で行を当て、sed -n で当てた範囲だけを読む",
    ),
    (
        "prior-outputs",
        6,
        "前の係の出す物は、頼みの文が名指した file と節だけを読む",
    ),
    ("id-grep", 6, "要件と判断の記録は、id を grep -A で読む"),
    (
        "watch-only",
        2,
        "不在を見る歯（見張り）には「無い」だけを、等しさを見る歯には「一致する」だけを言わせ",
    ),
    (
        "impl-name",
        2,
        "置き場を「impl X の中」と名指す時は、X を base の宣言と grep で照らす",
    ),
    (
        "section-material",
        2,
        "歯が比べる字そのもの（値の具体）と、数を決める着地済みの fn の数え方を節に書く",
    ),
    ("row-numbers", 2, "done に行の番号と行の数を持たせない"),
    (
        "own-names",
        2,
        "節に語の一覧（main の commit・語の数 N・空白で区切る）を字で置き",
    ),
    (
        "move-marks",
        2,
        "git mv で移す行は、移す元を ~、移す先を + で書く",
    ),
    (
        "eq-after-landing",
        2,
        "前の行が足す file を =<path> で名指す行は、前の行の着地の後に置く",
    ),
    (
        "env-names",
        3,
        "名の列を先に assert_eq し、名が合った後にだけ値を比べる",
    ),
    (
        "first-refusal",
        4,
        "src で最初に断る検めを照らしてから見本を書き",
    ),
    (
        "clause-kinds",
        4,
        "順は 2 つ以上で入力・数・字・逆の順がどれも違う並び",
    ),
    ("arm-text", 4, "判じの呼び（match の式）を字で見る歯は"),
    (
        "all-items",
        4,
        "指された項だけでなく done の全部の項について",
    ),
    (
        "note-fix",
        2,
        "生きたノートに在る行の直しは、席が頼みでそのノートを名指した時だけ",
    ),
    ("row-fields", 2, "要る時の growth で、欄 depends は書かない"),
    (
        "quotes",
        2,
        "行の title と done と節に、二重引用符と逆斜線を書かない",
    ),
    (
        "dep-notes",
        2,
        "notes.md に「依存: <行 id> → <行 id か bead id>」の 1 行ずつで書き、席が bd dep add で張る",
    ),
    (
        "src-growth",
        2,
        "crate の src の file を書く行は、その file ごとに growth を書く",
    ),
    (
        "bead-caps",
        2,
        "開いた契約の bead は 40 本まで・本文は 64 KB 以下・acceptance は 24 KB 以下",
    ),
    ("args", 2, "引数を 5 つまでにする"),
    (
        "filter-words",
        3,
        "xtask/tests/filter-words.txt の語と群の module の名のどれとも部分の字で重ならない",
    ),
    (
        "toy-seed",
        4,
        "e2e の toy の種に実装が作る file と宣言を置かない",
    ),
    (
        "floor-bead",
        8,
        "tz check --dir <写し>/design-intent --prose <本文の file の絶対 path>",
    ),
    (
        "floor-preflight",
        8,
        "scribe2 pipe preflight --contract <契約の file の絶対 path> --bead <名>-floor",
    ),
    (
        "floor-doc",
        8,
        "設計文書の dir（design-intent）を書く起草（判断の記録・要件・生きたノートの行の直し）だけが",
    ),
    (
        "seat-bead",
        8,
        "席は契約の file から欄 section と goal を除いた字を bead の欄 acceptance に",
    ),
];

/// file のどこにも無い字の名と字（写さない案内の幅の値と、直す前の群の頭の予算の字と、外した実装と道具の手順の語）。
const ABSENT: [(&str, &str); 16] = [
    ("width-copied", "100 字"),
    ("old-budget", "150000 以下"),
    ("patch-path", "docs/design/patch/"),
    ("patch", "差の file"),
    ("floor-record", "floor.tsv"),
    ("bite", "噛み"),
    ("fold", "畳み"),
    ("apply-script", "適用の script"),
    ("target-dir", "CARGO_TARGET_DIR"),
    ("prototype", "試作"),
    ("note-fragment", "行と節の断片"),
    ("note-cap", "ノートの行は 32 まで"),
    ("note-depends", "depends は同じノートの行だけ"),
    ("plan-rule", "規則の行 R-33"),
    ("design-pointer", "--design contracts/"),
    ("folio-derive", "folio derive"),
];

/// 器の案内の幅の歯の fn の頭（手引きが名指す接頭辞）。
const HELP_FN: &str = "fn cli_help_";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn reference() -> String {
    fs::read_to_string(root().join(REFERENCE)).expect("参照の file を読む")
}

/// 字 ## と空白で始まる行（見出しの行）を順に。
fn heads(text: &str) -> Vec<&str> {
    text.lines().filter(|l| l.starts_with("## ")).collect()
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
fn agref_reference_heads_are_the_eight_sections() {
    assert_eq!(heads(&reference()), HEADS);
}

#[test]
fn agref_one_clause_removed_doubled_or_moved_is_named() {
    let text = reference();
    for (name, n, words) in CLAUSES {
        let removed = text.replacen(words, "", 1);
        assert_eq!(faults(&removed), vec![name], "外した {name}");
        let doubled = text.replacen(words, &format!("{words}・{words}"), 1);
        assert_eq!(faults(&doubled), vec![name], "重ねた {name}");
        let moved = if n == 0 {
            format!("{removed}\n- {words}\n")
        } else {
            format!("- {words}\n{removed}")
        };
        assert_eq!(faults(&moved), vec![name], "動かした {name}");
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
fn agref_old_procedure_words_are_named() {
    let text = reference();
    for (name, words) in ABSENT.into_iter().skip(2) {
        let added = format!("{text}\n- {words}\n");
        assert_eq!(faults(&added), vec![name], "足した {name}");
    }
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
