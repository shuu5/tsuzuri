//! folio の検査の file `src/anchor.rs` の段の形の歯: 親 `check_anchor` が段を順に 1 度ずつ呼び、段どうしは呼び合わない。
//! 段の形は出す字に現れない（段が次の段を呼ぶ数珠の形でも同じ字を同じ順に出す）ので、source の字を関数の範囲の中で照らし、数を断言する。
//! 否定の見本は本物の source から 1 句だけ外した写し（呼びの入れ替え・外し・2 度・段の末の次の段の呼び・段と親の宣言の名替え）。
#![cfg(test)]

/// 照らす source（crate の manifest の dir から読む・歯の file の置き場に依らない）。
const SRC: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/anchor.rs"));

/// 親の名。
const PARENT: &str = "check_anchor";

/// 親が順に呼ぶ段の名。
const STAGES: [&str; 9] = [
    "read_current",
    "read_chain",
    "check_git_records",
    "check_index",
    "check_versions",
    "check_reuse",
    "check_pairs",
    "check_current",
    "finish",
];

/// `fn <name>(` か `fn <name><` の宣言から、行の頭の `}` の行の前までの字（関数の範囲）。
fn fn_span<'s>(src: &'s str, name: &str) -> Option<&'s str> {
    let head = format!("fn {name}");
    let at = src
        .match_indices(&head)
        .map(|(i, _)| i)
        .find(|&i| matches!(src[i + head.len()..].chars().next(), Some('(' | '<')))?;
    let len = src[at..].find("\n}\n")?;
    Some(&src[at..at + len])
}

/// 範囲の中の `<name>(` の呼びの位置（前の字が名の字でない物だけ・`fn <name>(` の宣言を除く）。
fn calls(span: &str, name: &str) -> Vec<usize> {
    let pat = format!("{name}(");
    span.match_indices(&pat)
        .map(|(i, _)| i)
        .filter(|&i| {
            let before = &span[..i];
            !before.ends_with(|c: char| c.is_ascii_alphanumeric() || c == '_')
                && !before.ends_with("fn ")
        })
        .collect()
}

/// 親の外れ: 呼びの数が 1 でない段と、前の段より前で呼ぶ段。
fn parent_faults(parent: &str) -> Vec<String> {
    let mut faults = Vec::new();
    let mut last: Option<(&str, usize)> = None;
    for stage in STAGES {
        let at = calls(parent, stage);
        if at.len() != 1 {
            faults.push(format!("親が段 {stage} を {} 度呼ぶ", at.len()));
            continue;
        }
        if let Some((prev, pos)) = last
            && at[0] < pos
        {
            faults.push(format!("親の段 {stage} の呼びが段 {prev} の前"));
        }
        last = Some((stage, at[0]));
    }
    faults
}

/// 段の外れ: 本体の無い段と、ほかの段を呼ぶ段。
fn stage_faults(src: &str) -> Vec<String> {
    let mut faults = Vec::new();
    for stage in STAGES {
        let Some(body) = fn_span(src, stage) else {
            faults.push(format!("段 {stage} の本体が無い"));
            continue;
        };
        for other in STAGES.iter().filter(|o| **o != stage) {
            if !calls(body, other).is_empty() {
                faults.push(format!("段 {stage} が段 {other} を呼ぶ"));
            }
        }
    }
    faults
}

/// 形の外れの全部（親の本体が無ければその 1 件だけ）。
fn shape_faults(src: &str) -> Vec<String> {
    let Some(parent) = fn_span(src, PARENT) else {
        return vec![format!("親 {PARENT} の本体が無い")];
    };
    let mut faults = parent_faults(parent);
    faults.extend(stage_faults(src));
    faults
}

/// 親の本体で呼ぶ段の名（呼びの位置の順・2 度の呼びは 2 度並ぶ）。
fn parent_order(src: &str) -> Vec<&'static str> {
    let parent = fn_span(src, PARENT).expect("親の本体が無い");
    let mut at: Vec<(usize, &'static str)> = STAGES
        .iter()
        .flat_map(|s| calls(parent, s).into_iter().map(move |i| (i, *s)))
        .collect();
    at.sort_unstable();
    at.into_iter().map(|(_, s)| s).collect()
}

/// 本物の source の字 `from` を `to` に替えた写し（`from` がちょうど 1 度在ることを先に断言する）。
fn mutate(from: &str, to: &str) -> String {
    assert_eq!(
        SRC.matches(from).count(),
        1,
        "見本の元の字が 1 度でない: {from}"
    );
    SRC.replacen(from, to, 1)
}

#[test]
fn achain_parent_calls_the_nine_stages_once_in_order() {
    assert_eq!(parent_order(SRC), STAGES.to_vec());
    let parent = fn_span(SRC, PARENT).expect("親の本体が無い");
    assert_eq!(parent_faults(parent), Vec::<String>::new());
}

#[test]
fn achain_no_stage_calls_another_stage() {
    let found = STAGES.iter().filter(|s| fn_span(SRC, s).is_some()).count();
    assert_eq!(found, 9, "段の本体の数");
    assert_eq!(stage_faults(SRC), Vec::<String>::new());
}

#[test]
fn achain_one_clause_mutants_are_named() {
    let line = |stage: &str| format!("    {stage}(&cur, &chain, report);\n");
    let (versions, reuse, pairs) = (
        line("check_versions"),
        line("check_reuse"),
        line("check_pairs"),
    );
    let reuse_body = fn_span(SRC, "check_reuse").expect("段 check_reuse の本体が無い");
    let cases: [(String, String); 6] = [
        (
            mutate(&format!("{versions}{reuse}"), &format!("{reuse}{versions}")),
            "親の段 check_reuse の呼びが段 check_versions の前".to_string(),
        ),
        (
            mutate(&pairs, ""),
            "親が段 check_pairs を 0 度呼ぶ".to_string(),
        ),
        (
            mutate(&pairs, &format!("{pairs}{pairs}")),
            "親が段 check_pairs を 2 度呼ぶ".to_string(),
        ),
        (
            mutate(
                reuse_body,
                &format!("{reuse_body}\n    check_pairs(cur, chain, report);"),
            ),
            "段 check_reuse が段 check_pairs を呼ぶ".to_string(),
        ),
        (
            mutate("fn check_reuse(", "fn check_reuse_x("),
            "段 check_reuse の本体が無い".to_string(),
        ),
        (
            mutate("fn check_anchor(", "fn check_anchor_x("),
            "親 check_anchor の本体が無い".to_string(),
        ),
    ];
    for (src, want) in &cases {
        assert_eq!(shape_faults(src), vec![want.clone()]);
    }
    assert_eq!(shape_faults(SRC), Vec::<String>::new());
}
