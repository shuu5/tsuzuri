//! 検証役の型の本文の観点の歯（接頭辞 agvlns_・ADR-83 の決定 (3) の段 1 と段 3・行 t-verifier-lens と t-verifier-lens-2）。
//! plugin/agents/verifier.md の本文が 6 つの観点を、この file の定数 LENSES の字のまま、はしごの行の後で返りの形の節の前に 1 行ずつ持つことを見る。型の定義は Claude Code が係を起こす時に読む設定の字で、歯から係を起こして所見を測れないので字を照らす。
#![cfg(test)]

use std::fs;
use std::path::{Path, PathBuf};

/// 検証役の型の file（workspace の根から）。
const VERIFIER: &str = "plugin/agents/verifier.md";

/// 6 つの観点の語（ADR-83 の段 1 の順・語と順は替えない）。
const HEADS: [&str; 6] = [
    "退けた案へ戻る道",
    "役の混ざり",
    "ゴールの結び",
    "承認の字と記録の字の食い違い",
    "前の段の消す物",
    "引いた決まりと前の記録の字が今も効くか",
];

/// 観点の行の字の正本（HEADS の順）。
const LENSES: [&str; 6] = [
    "- 観点 退けた案へ戻る道（撤退と推奨の行き先が退けた案か）。",
    "- 観点 役の混ざり（設計係か席が実装か契約の字を作る形・契約の字は設計係の出す物だけで実装の字は便だけが書く決まりに照らし、稿の全文の句を 1 つずつ見て、誰の手かが席か設計係と読める句のうち契約か実装の字を書く・直す・起草する物を、決めの本文だけでなく順や手続きや注の中の句も含め、在りかごとに 1 件ずつ拾う）。",
    "- 観点 ゴールの結び（判断の記録 ADR-83 の決定 (1) のどの G に結ぶか）。",
    "- 観点 承認の字と記録の字の食い違い（記録と頼みが承認の逐語に引き金・動き・問う句を足すか削るか）。",
    "- 観点 前の段の消す物（前の段の memo が名指す消す物が code に残るまま、後の記録が残す決めを置いていないか）。",
    "- 観点 引いた決まりと前の記録の字が今も効くか（status・外した便・宣言の鍵）。",
];

/// 観点の行の頭。
const MARK: &str = "- 観点 ";

/// はしごの行の末と改行（観点の区間の始め）。
const AFTER: &str = "（条 P-27.2）の有無を書く。\n";

/// 返りの形の節の見出し（観点の区間の終わり）。
const SHAPE: &str = "## 返りの形";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(rel: &str) -> String {
    fs::read_to_string(root().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// 検証役の型の今の字。
fn current() -> String {
    read(VERIFIER)
}

/// 正本の観点の行 6 つ。
fn wanted() -> Vec<String> {
    LENSES.iter().map(|l| l.to_string()).collect()
}

/// 行から MARK と語と括弧の開きと末の「）。」を外した括弧の中の字。空か括弧か改行を持てば None。
fn inner<'a>(head: &str, line: &'a str) -> Option<&'a str> {
    let rest = line
        .strip_prefix(MARK)?
        .strip_prefix(head)?
        .strip_prefix('（')?
        .strip_suffix("）。")?;
    if rest.is_empty() || rest.contains(['（', '）', '\n']) {
        return None;
    }
    Some(rest)
}

/// 本文の観点の行の並びと、区間の外の観点の行の数。頭か AFTER か SHAPE が無ければ None。
fn lenses(text: &str) -> Option<(Vec<String>, usize)> {
    let rest = text.strip_prefix("---\n")?;
    let (_, body) = rest.split_once("\n---\n")?;
    let (before, after) = body.split_once(AFTER)?;
    let (span, tail) = after.split_once(SHAPE)?;
    let marked = |s: &str| s.lines().filter(|l| l.starts_with(MARK)).count();
    let seq = span
        .lines()
        .filter(|l| l.starts_with(MARK))
        .map(str::to_string)
        .collect();
    Some((seq, marked(before) + marked(tail)))
}

/// (1) 本文の観点の行の並びは正本から組む 6 行と一致し、区間の外に観点の行は無い。
#[test]
fn agvlns_verifier_body_holds_the_six_lenses_in_order() {
    let want = wanted();
    assert_eq!(want.len(), 6);
    for line in &want {
        assert!(!line.ends_with("（）。"), "括弧の中が空: {line}");
    }
    assert_eq!(lenses(&current()), Some((want, 0)));
}

/// 行 i を外した並び。
fn without(want: &[String], i: usize) -> Vec<String> {
    let mut v = want.to_vec();
    v.remove(i);
    v
}

/// 見本が今の字と違い（空振りでない）、lenses が期待の組を返す。
fn assert_sample(sample: &str, now: &str, expect: (Vec<String>, usize), what: &str) {
    assert_ne!(sample, now, "{what}: 見本が今の字と同じ");
    assert_eq!(lenses(sample), Some(expect), "{what}");
}

/// 観点の行 i を外す・2 度にする・字を足す・前へ後ろへ動かす見本を見る。
fn assert_line_samples(now: &str, want: &[String], i: usize) {
    let line = &want[i];
    let nl = format!("{line}\n");

    let removed = now.replacen(&nl, "", 1);
    assert_sample(&removed, now, (without(want, i), 0), "外す");

    let doubled = now.replacen(&nl, &format!("{nl}{nl}"), 1);
    let mut twice = want.to_vec();
    twice.insert(i, line.clone());
    assert_sample(&doubled, now, (twice, 0), "2 度");

    let grown = format!("{}字）。", line.strip_suffix("）。").unwrap());
    let altered = now.replacen(line, &grown, 1);
    let mut grown_seq = want.to_vec();
    grown_seq[i] = grown;
    assert_sample(&altered, now, (grown_seq, 0), "字を足す");

    let front = removed.replacen("足す提案（走査・", &format!("{nl}足す提案（走査・"), 1);
    assert_sample(&front, &removed, (without(want, i), 1), "前へ");

    let back = removed.replacen("## 返りの形\n", &format!("## 返りの形\n{nl}"), 1);
    assert_sample(&back, &removed, (without(want, i), 1), "後ろへ");
}

/// (2) 観点の行を外す・2 度にする・字を足す・動かす・入れ替える・置き換えを戻す見本は落ちる。
#[test]
fn agvlns_lens_removed_doubled_moved_or_altered_is_seen() {
    let now = current();
    let want = wanted();
    for i in 0..want.len() {
        assert_line_samples(&now, &want, i);
    }

    let swapped = now.replacen(
        &format!("{}\n{}\n", want[0], want[1]),
        &format!("{}\n{}\n", want[1], want[0]),
        1,
    );
    assert_ne!(swapped, now);
    let mut swapped_seq = want.clone();
    swapped_seq.swap(0, 1);
    assert_eq!(lenses(&swapped), Some((swapped_seq, 0)));

    let expanded = "判断の記録 ADR-83 の決定 (1)";
    let reverted = now.replacen(expanded, "決定 (1)", 1);
    assert_ne!(reverted, now);
    let mut reverted_seq = want.clone();
    reverted_seq[2] = want[2].replacen(expanded, "決定 (1)", 1);
    assert_eq!(lenses(&reverted), Some((reverted_seq, 0)));
}

/// (3) LENSES の各行は MARK と HEADS の語と括弧の開きで始まり「）。」で終わり、括弧の中は空でなく括弧と改行を持たない。
#[test]
fn agvlns_lens_lines_keep_the_heads_in_order() {
    for i in 0..6 {
        assert!(inner(HEADS[i], LENSES[i]).is_some(), "{i} 番目");
    }
    assert_eq!(inner(HEADS[0], LENSES[1]), None, "語をずらした組");
    let nested = format!("{}（字）。", LENSES[1].strip_suffix("）。").unwrap());
    assert_eq!(inner(HEADS[1], &nested), None, "括弧の中に組を足した字");
    let no_stop = LENSES[1].strip_suffix('。').unwrap();
    assert_eq!(inner(HEADS[1], no_stop), None, "末の句点を外した字");
    let empty = format!("{MARK}{}（）。", HEADS[1]);
    assert_eq!(inner(HEADS[1], &empty), None, "括弧の中が空");
}
