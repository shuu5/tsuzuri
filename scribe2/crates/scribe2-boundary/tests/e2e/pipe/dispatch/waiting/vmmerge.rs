//! memo の審査の判定の語 merge と keep の理由の型の歯（接頭辞 `vmmerge_`・設計 contracts/surface-v4b.toml#v-memo-merge）。
//! 親 `waiting.rs` の helper を `use super::*` で使う。

use super::*;

/// 偽の lens に `json` の 1 行を返させ、`memo` を撃って `verdict` の file の `(key, 文字列の値)` の列を返す。
fn judged_with(repo: &Path, state: &Path, bd: &str, (memo, name): (&str, &str), json: &str) -> (Output, Vec<(String, String)>) {
    let rules = dispatch_rules(state);
    let lens = memo_lens_cmd(state, name, &format!("{json}\n"), 0);
    let out = memo_lens(repo, state, bd, memo, (&lens, &rules));
    let verdict = string_pairs(&place_file(state, memo, "verdict"));
    (out, verdict)
}

/// (1) 最後の行が merge と into の JSON の lens の周は、`verdict` の file が merge と into を持ち、stdout が `verdict=merge` の 1 行で、
/// `MemoJudged` の detail は merge だけ。
#[test]
fn vmmerge_merge_writes_the_target() {
    let (repo, state) = memo_repo();
    let bd = fake_bd(&state, &unmet_memos(&["s2-m.1", "s2-m.2"]));
    let json = "{\"verdict\":\"merge\",\"evidence\":\"ev-same\",\"into\":\"s2-m.2\",\"sketch\":\"\"}";
    let (out, verdict) = judged_with(&repo, &state, &bd, ("s2-m.1", "merge"), json);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "{}", told(&out));
    assert_eq!(stdout_of(&out), "memo-lens memo=s2-m.1 verdict=merge\n", "{}", told(&out));
    assert_eq!((pair_of(&verdict, "verdict"), pair_of(&verdict, "into")), ("merge".to_owned(), "s2-m.2".to_owned()), "{verdict:?}");
    assert_eq!(pair_of(&verdict, "evidence"), "ev-same", "{verdict:?}");
    let events = judged_lines(&state);
    assert_eq!(events.len(), 1, "MemoJudged は 1 行: {events:?}");
    let event = string_pairs(events.first().map_or("", String::as_str));
    assert_eq!(pair_of(&event, "detail"), "merge", "detail は語の字だけ: {event:?}");
    clean(&[&repo, &state]);
}

/// (2) merge で into が無い・空・memo 自身の id・字 `a b` の 4 つの JSON は、どれも `verdict` が unparsed で evidence が空でない
/// （同じ歯の fit な into の merge は merge で、4 つは into の良し悪しで分かれる）。
#[test]
fn vmmerge_merge_without_a_fit_target_is_unparsed() {
    let (repo, state) = memo_repo();
    let bd = fake_bd(&state, &unmet_memos(&["s2-m.1", "s2-m.2", "s2-m.3", "s2-m.4", "s2-m.5"]));
    let shape = |into: &str| format!("{{\"verdict\":\"merge\",\"evidence\":\"e\",\"sketch\":\"\"{into}}}");
    let cases = [
        ("s2-m.1", "no-into", shape(""), "unparsed"),
        ("s2-m.2", "empty-into", shape(",\"into\":\"\""), "unparsed"),
        ("s2-m.3", "self-into", shape(",\"into\":\"s2-m.3\""), "unparsed"),
        ("s2-m.4", "space-into", shape(",\"into\":\"a b\""), "unparsed"),
        ("s2-m.5", "fit-into", shape(",\"into\":\"s2-m.1\""), "merge"),
    ];
    for (memo, name, json, word) in cases {
        let (_, verdict) = judged_with(&repo, &state, &bd, (memo, name), &json);
        assert_eq!(pair_of(&verdict, "verdict"), word, "{name}: {verdict:?}");
        assert_eq!(word == "merge", pair_of(&verdict, "evidence") == "e", "{name}: unparsed は理由を evidence に書く: {verdict:?}");
        assert!(!pair_of(&verdict, "evidence").is_empty(), "{name}: {verdict:?}");
    }
    clean(&[&repo, &state]);
}

/// (3) keep で why が no-material・wait-row・wait-owner の 3 つの JSON は、どれも `verdict` が keep で why がその字。
#[test]
fn vmmerge_keep_carries_one_of_three_reasons() {
    let (repo, state) = memo_repo();
    let bd = fake_bd(&state, &unmet_memos(&["s2-m.1", "s2-m.2", "s2-m.3"]));
    for (memo, why) in [("s2-m.1", "no-material"), ("s2-m.2", "wait-row"), ("s2-m.3", "wait-owner")] {
        let json = format!("{{\"verdict\":\"keep\",\"evidence\":\"e\",\"why\":\"{why}\",\"sketch\":\"\"}}");
        let (_, verdict) = judged_with(&repo, &state, &bd, (memo, &format!("keep-{why}")), &json);
        assert_eq!((pair_of(&verdict, "verdict"), pair_of(&verdict, "why")), ("keep".to_owned(), why.to_owned()), "{verdict:?}");
    }
    clean(&[&repo, &state]);
}

/// (4) keep で why が無い JSON と why が字 `later` の JSON は、`verdict` が unparsed（evidence は空でない）。
#[test]
fn vmmerge_keep_without_a_reason_is_unparsed() {
    let (repo, state) = memo_repo();
    let bd = fake_bd(&state, &unmet_memos(&["s2-m.1", "s2-m.2"]));
    let cases = [
        ("s2-m.1", "no-why", "{\"verdict\":\"keep\",\"evidence\":\"e\",\"sketch\":\"\"}"),
        ("s2-m.2", "later", "{\"verdict\":\"keep\",\"evidence\":\"e\",\"why\":\"later\",\"sketch\":\"\"}"),
    ];
    for (memo, name, json) in cases {
        let (_, verdict) = judged_with(&repo, &state, &bd, (memo, name), json);
        assert_eq!(pair_of(&verdict, "verdict"), "unparsed", "{name}: {verdict:?}");
        assert!(!pair_of(&verdict, "evidence").is_empty(), "{name}: {verdict:?}");
    }
    clean(&[&repo, &state]);
}

/// (5) 開いた memo s2-m.1 と s2-m.2 と閉じた memo s2-m.3 の台帳で s2-m.1 を撃つと、置き場の material は見出し `## ほかの開いた memo` を 1 度持ち、
/// その後に s2-m.2 の id を持ち、s2-m.3 と自分の id は持たない。
#[test]
fn vmmerge_material_lists_the_other_open_memos() {
    let (repo, state) = memo_repo();
    let mut beads = unmet_memos(&["s2-m.1", "s2-m.2"]);
    beads.push(memo_of("s2-m.3", "closed", Some(MEMO_CREATED), &["引き金: 期日 2999-01-01T00:00Z"], ""));
    let bd = fake_bd(&state, &beads);
    let json = "{\"verdict\":\"keep\",\"evidence\":\"e\",\"why\":\"no-material\",\"sketch\":\"\"}";
    let (out, _) = judged_with(&repo, &state, &bd, ("s2-m.1", "material"), json);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "{}", told(&out));
    let material = place_file(&state, "s2-m.1", "material");
    let heading = "## ほかの開いた memo";
    assert_eq!(material.matches(heading).count(), 1, "見出しは 1 度: {material}");
    let tail = material.split_once(heading).map_or("", |(_, rest)| rest);
    assert!(tail.contains("- s2-m.2 "), "開いた別の memo の行: {material}");
    assert!(!tail.contains("s2-m.3"), "閉じた memo は載せない: {material}");
    assert!(!tail.contains("s2-m.1"), "自分は載せない: {material}");
    clean(&[&repo, &state]);
}
