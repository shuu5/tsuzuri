//! 行 v-ci-split の歯: 歯でない段の割りの振り分け（xtask の src/spread.rs・判断の記録 ADR-34 の決定 (9)）の表と判じを撃ち、
//! check と入れ子の段の呼びを字で読む。
#![cfg(test)]

#[path = "../src/spread.rs"]
mod spread;

use std::path::PathBuf;

use spread::{SLOTS, SURFACE, TABLE, Turn};

/// repo の根（xtask の manifest の dir の 1 つ上）からの相対の path の file を読む。
fn read_root(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn strings(xs: &[&str]) -> Vec<String> {
    xs.iter().map(|s| (*s).to_string()).collect()
}

/// 割り count:k/n で段を撃つ役の番号の列。
fn runners(key: &str, n: u64) -> Vec<u64> {
    (1..=n)
        .filter(|k| {
            let part = format!("count:{k}/{n}");
            spread::turn(key, Some(part.as_str())) == Ok(Turn::Run)
        })
        .collect()
}

#[test]
fn vsplit_table_fixed() {
    assert_eq!(
        TABLE,
        [
            ("cargo build --workspace", 0),
            ("cargo nextest run --workspace", 0),
            ("cargo clippy --workspace --all-targets -- -D warnings", 2),
            (
                "cargo clippy -p tsuzuri-surface --target wasm32-unknown-unknown -- -D warnings",
                8
            ),
            ("surface-build", 7),
            ("nested build --workspace", 1),
            ("nested nextest run --workspace", 0),
            ("nested clippy --workspace --all-targets -- -D warnings", 5),
            ("nested xtask check", 6),
        ]
    );
    assert_eq!(SLOTS, 8);
    assert_eq!(SURFACE, "surface-build");
    let mut spread: Vec<u64> = TABLE
        .iter()
        .map(|(_, at)| *at)
        .filter(|at| *at != 0)
        .collect();
    spread.sort_unstable();
    assert_eq!(spread, [1, 2, 5, 6, 7, 8], "歯でない段は役に 1 つずつ");
}

#[test]
fn vsplit_each_key_once() {
    for n in [8, 1, 3, 12] {
        for (key, at) in TABLE {
            let got = runners(key, n);
            if *at == 0 {
                assert_eq!(got, (1..=n).collect::<Vec<_>>(), "{key} は {n} の役の全部");
            } else {
                assert_eq!(got.len(), 1, "{key} は {n} の役のちょうど 1 つ: {got:?}");
            }
        }
    }
    let owners: Vec<(&str, Vec<u64>)> = TABLE
        .iter()
        .filter(|(_, at)| *at != 0)
        .map(|(key, _)| (*key, runners(key, 8)))
        .collect();
    assert_eq!(
        owners,
        [
            (
                "cargo clippy --workspace --all-targets -- -D warnings",
                vec![2]
            ),
            (
                "cargo clippy -p tsuzuri-surface --target wasm32-unknown-unknown -- -D warnings",
                vec![8]
            ),
            ("surface-build", vec![7]),
            ("nested build --workspace", vec![1]),
            (
                "nested clippy --workspace --all-targets -- -D warnings",
                vec![5]
            ),
            ("nested xtask check", vec![6]),
        ]
    );
    assert_eq!(
        spread::turn("surface-build", Some("count:1/8")),
        Ok(Turn::Elsewhere(7, 8))
    );
    assert_eq!(
        spread::turn("surface-build", Some("count:1/3")),
        Ok(Turn::Run),
        "7 は 3 の役で回すと 1"
    );
    assert_eq!(
        spread::line("surface-build", 7, 8),
        "段 surface-build は割りの役 7/8 が撃つ"
    );
}

#[test]
fn vsplit_no_partition_runs_all() {
    for (key, _) in TABLE {
        assert_eq!(spread::turn(key, None), Ok(Turn::Run), "{key}");
    }
}

#[test]
fn vsplit_nested_stages_spread() {
    let steps: [&[&str]; 4] = [
        &["build", "--workspace"],
        &["nextest", "run", "--workspace", "--partition", "count:3/8"],
        &[
            "clippy",
            "--workspace",
            "--all-targets",
            "--",
            "-D",
            "warnings",
        ],
        &["xtask", "check"],
    ];
    let keys: Vec<String> = steps
        .iter()
        .map(|s| spread::key("nested", &strings(s)))
        .collect();
    assert_eq!(
        keys,
        strings(&[
            "nested build --workspace",
            "nested nextest run --workspace",
            "nested clippy --workspace --all-targets -- -D warnings",
            "nested xtask check",
        ])
    );
    let at: Vec<Vec<u64>> = keys.iter().map(|k| runners(k, 8)).collect();
    assert_eq!(at, [vec![1], (1..=8).collect(), vec![5], vec![6]]);
    assert_eq!(
        spread::key(
            "cargo",
            &strings(&["nextest", "run", "--workspace", "--partition", "count:2/8"])
        ),
        "cargo nextest run --workspace"
    );
}

#[test]
fn vsplit_unknown_key_refused() {
    // 否定の見本は、正しい見本から測る句の 1 つだけを外した形（鍵だけが表に無い・割りの字の句の 1 つだけが崩れる）。
    assert_eq!(
        spread::turn("cargo build --workspace", Some("count:1/8")),
        Ok(Turn::Run)
    );
    let want = Err("段 cargo fmt --check が振り分けの表に無い".to_string());
    assert_eq!(spread::turn("cargo fmt --check", None), want);
    assert_eq!(spread::turn("cargo fmt --check", Some("count:1/8")), want);
    assert_eq!(
        spread::turn("surface-build", Some("count:7/8")),
        Ok(Turn::Run)
    );
    for part in [
        "1/8",
        "count:7-8",
        "count:x/8",
        "count:7/y",
        "count:0/8",
        "count:9/8",
    ] {
        assert_eq!(
            spread::turn("surface-build", Some(part)),
            Err(format!("割りの字を読めない: {part}")),
            "{part}"
        );
    }
}

#[test]
fn vsplit_check_wiring() {
    let main = read_root("xtask/src/main.rs");
    assert_eq!(main.lines().filter(|l| *l == "mod spread;").count(), 1);
    let (_, check) = main.split_once("fn check(").expect("fn check");
    let (check, _) = check.split_once("\n}\n").expect("fn check の終わり");
    in_order(
        check,
        &[
            "for step in CHECK_STEPS {",
            "let rc = cargo_step(root, &cargo, step, part);",
            "let rc = match turn(spread::SURFACE, part) {",
            "surface_build(root)",
            "Ok(false) => 0,",
            "nested::run(root, &cargo, part)",
        ],
    );
    let (_, step) = main.split_once("fn cargo_step(").expect("fn cargo_step");
    let (step, _) = step.split_once("\n}\n").expect("fn cargo_step の終わり");
    in_order(
        step,
        &[
            "match turn(&spread::key(\"cargo\", &args), part) {",
            "Ok(false) => return 0,",
            "Command::new(cargo)",
        ],
    );
    let (_, turn) = main.split_once("fn turn(").expect("fn turn");
    let (turn, _) = turn.split_once("\n}\n").expect("fn turn の終わり");
    in_order(
        turn,
        &[
            "match spread::turn(key, part) {",
            "Ok(spread::Turn::Run) => Ok(true),",
            "spread::line(key, owner, n)",
            "Ok(false)",
            "Err(2)",
        ],
    );
    let nested = read_root("xtask/src/nested.rs");
    assert!(
        nested.contains(
            "    for shot in plan(root, &base, &dirs, part) {\n        match turn(&spread::key(\"nested\", &shot.args), part) {\n            Ok(true) => {}\n            Ok(false) => continue,\n            Err(rc) => return rc,\n        }\n"
        ),
        "入れ子の段は撃つ前に判じ、ほかの役の段は撃たない"
    );
}

/// 字の列が text の中にちょうど 1 度ずつ、この順で在る。
fn in_order(text: &str, order: &[&str]) {
    let at: Vec<usize> = order
        .iter()
        .map(|s| {
            assert_eq!(text.matches(s).count(), 1, "{s}");
            text.find(s).expect(s)
        })
        .collect();
    assert!(
        at.windows(2).all(|w| w[0] < w[1]),
        "字の順: {order:?} {at:?}"
    );
}
