//! 役の一致を一時の file に書かず流れのまま読む歯（設計 docs/design/reverse-index.md §4 形 2・形 3・形 8・接頭辞 `vixpipe_`）。
//!
//! 撃つ口は実 binary の `pipe index build` で、toy repo と偽の宣言の command は親の索引の組み立ての歯の置き場（`idxb_place`）を
//! 使う。期待の側は直す前の読み（一致の全文を a1 の `read_roles` で読み、`join` と `render` で表にする）で組み、器が流れのまま
//! 読んで置いた表と記録の字と比べる。一致の stream は埋めの行で pipe の大きさ（64 KiB）の 4 倍を越え、読み手が子と並んで
//! 読み切らなければ子が書き切れない大きさにする。

use super::*;
use vessel::pipe::index::{flat, join, read_roles, scip};

/// 埋めの行の数（9 語の外の ruleId の行・1 行 約 230 byte で 256 KiB を越える）。
const VXP_PAD: usize = 1500;

/// 2 本目の偽の役の検出の command 名。
const VXP_ROLES2: &str = "index-fake-roles2";

/// 期待の表の字: SCIP の fixture と一致の全文を a1 の読み手と結びで組む（直す前の file の全文の読みと同じ道）。
fn vxp_table(text: &str) -> String {
    let docs = scip::read_scip(&idx_scip_bytes()).unwrap_or_default();
    let matches = read_roles(text).map(|read| read.matches).unwrap_or_default();
    flat::render(&join(&docs, &matches, &idx_bodies()).unwrap_or_default())
}

/// 埋めの行（9 語の外の ruleId・捨てて数える）を `count` 行。
fn vxp_pad(count: usize) -> String {
    let line = idx_role_line("unrelated-rule", "src/a.rs", (0, 1), Some(&"x".repeat(80)));
    (0..count).map(|_| format!("{line}\n")).collect()
}

/// 偽の役の検出（`name`）が `lead` を撃ってから `stream` の byte を stdout へ出し、`tail` を撃つ。
fn vxp_roles(place: &IdxPlace, name: &str, stream: &[u8], (lead, tail): (&str, &str)) {
    let source = place.state.join(format!("{name}.stream"));
    assert!(fs::write(&source, stream).is_ok(), "一致の見本を書ける");
    idxb_script(place, (name, "roles"), &format!("{lead}cat '{}'\n{tail}", source.display()));
}

/// `pipe index build` を撃って結末の行を返す（時間の上限は 60 秒・読み手が詰まった周を早く落とす）。
fn vxp_run(place: &IdxPlace) -> String {
    let rules = place.rules("rules-vxp.toml", &[(IDXB_TIMEOUT_ROW, &IDXB_TIMEOUT_ROW.replace("600", "60"))]);
    idxb_line(&place.build(Some(&rules), &[]))
}

/// 結末の行の鍵の記録の `name=` の値。
fn vxp_record(place: &IdxPlace, line: &str, name: &str) -> String {
    let record = fs::read_to_string(place.dir().join(format!("{}.rec", idxb_field(line, "key")))).unwrap_or_default();
    record.lines().find_map(|found| found.strip_prefix(&format!("{name}="))).unwrap_or_default().to_owned()
}

/// 失敗の周の共通の断言（語・記録の失敗の語と行の数・表を置かず記録だけが残る）。
fn vxp_failed(place: &IdxPlace, line: &str, word: &str) {
    let key = idxb_field(line, "key");
    assert_eq!(idxb_word(line), format!("failed:{word}"), "{line}");
    assert_eq!((vxp_record(place, line, "failed"), vxp_record(place, line, "rows")), (word.to_owned(), "0".to_owned()), "記録の失敗の語と行の数");
    assert_eq!(place.names(), [format!("{key}.rec")], "表を置かず、記録だけが残る（一致の file も無い）");
}

/// fixture の一致の 1 行目。
fn vxp_first() -> String {
    idx_roles_text().lines().next().unwrap_or_default().to_owned()
}

/// 復帰つきの改行・空の行・空白だけの行・埋めの行を挟み、末の行が改行で終わらない stream で、置いた表は全文の読みの表と byte で
/// 等しく、記録の rows・files・dropped も全文の読みの数と等しい。
#[test]
fn vixpipe_stream_table_equals_the_whole_text_read() {
    let place = idxb_with_fixtures();
    let lines: Vec<String> = idx_roles_text().lines().map(str::to_owned).collect();
    let (head, rest) = lines.split_at(6);
    let marked: String = head.iter().map(|line| format!("{line}\r\n\n  \t\n")).collect();
    let text = format!("{marked}{}{}", vxp_pad(VXP_PAD), rest.join("\n"));
    assert!(text.len() > 4 * 65536 && !text.ends_with('\n'), "埋めは pipe の 4 倍を越え、末の行は改行で終わらない");
    let want = read_roles(&text).unwrap_or_default();
    assert_eq!(want.dropped, VXP_PAD + 1, "全文の読みは埋めの行と fixture の 1 行を捨てる");
    vxp_roles(&place, IDXB_ROLES, text.as_bytes(), ("", ""));
    let line = vxp_run(&place);
    assert_eq!(idxb_word(&line), "built", "{line}");
    let table = fs::read_to_string(place.dir().join(format!("{}.tsv", idxb_field(&line, "key")))).unwrap_or_default();
    let rows = flat::read_table(&table).ok().flatten().map(|found| found.len()).unwrap_or_default();
    assert_ne!(vxp_table(&text), vxp_table(""), "一致は表を替える");
    assert_eq!(table, vxp_table(&text), "表の字は全文の読みの表と等しい");
    let shape = ["rows", "dropped"].map(|name| vxp_record(&place, &line, name));
    assert_eq!(shape, [rows.to_string(), want.dropped.to_string()], "記録の行の数と捨てた数");
    clean(&[&place.repo, &place.state]);
}

/// 役の行の子の stdout は pipe で、撃ち中の置き場の鍵の file は SCIP の出力と印の 2 つだけ（一致の file を作らない）、撃ち終えた
/// 後は表と記録だけが残る。
#[test]
fn vixpipe_roles_stream_through_a_pipe_and_leave_no_file() {
    let place = idxb_with_fixtures();
    let seen = place.state.join("vxp-seen");
    let lead = format!(
        "if test -p /dev/stdout; then echo pipe > '{seen}'; else echo other > '{seen}'; fi\nls '{dir}' >> '{seen}'\n",
        seen = seen.display(),
        dir = place.dir().display()
    );
    vxp_roles(&place, IDXB_ROLES, idx_roles_text().as_bytes(), (&lead, ""));
    let line = vxp_run(&place);
    assert_eq!(idxb_word(&line), "built", "{line}");
    let key = idxb_field(&line, "key");
    let listed = fs::read_to_string(&seen).unwrap_or_default();
    assert_eq!(listed.lines().next(), Some("pipe"), "子の stdout は pipe: {listed}");
    let keyed: Vec<&str> = listed.lines().skip(1).filter(|name| name.starts_with(&format!("{key}."))).collect();
    assert_eq!(keyed, [format!("{key}.0.scip"), format!("{key}.lock")], "撃ち中の鍵の file は SCIP の出力と印だけ: {listed}");
    assert_eq!(place.names(), [format!("{key}.rec"), format!("{key}.tsv")], "撃ち終えた後は表と記録だけ");
    clean(&[&place.repo, &place.state]);
}

/// JSON でない行の後に埋めの行が続く stream（子は rc 0）は failed:unreadable で、記録の stderr は全文の読みの理由（空の行と
/// 空白だけの行も数えた行の番号）と等しい。
#[test]
fn vixpipe_unreadable_line_is_named_like_the_whole_text_read() {
    let place = idxb_with_fixtures();
    let text = format!("{}\n\n   \n{{\"ruleId\":\n{}{}", vxp_first(), vxp_pad(VXP_PAD), idx_roles_text());
    let want = read_roles(&text).map(|_| String::new()).unwrap_or_else(|err| err.to_string());
    assert!(want.starts_with("roles: 4 行目:"), "全文の読みは 4 行目を名指す: {want}");
    vxp_roles(&place, IDXB_ROLES, text.as_bytes(), ("", ""));
    let line = vxp_run(&place);
    vxp_failed(&place, &line, "unreadable");
    assert_eq!(vxp_record(&place, &line, "stderr"), want, "記録の理由は全文の読みと同じ");
    clean(&[&place.repo, &place.state]);
}

/// UTF-8 でない byte を持つ stream は、それより前の JSON でない行より先に UTF-8 でないことを名指す（全文の読みと同じ順）。同じ
/// stream から UTF-8 でない行を外すと、JSON でない行の番号を名指す。
#[test]
fn vixpipe_invalid_utf8_is_named_before_an_unreadable_line() {
    let place = idxb_with_fixtures();
    let lead = format!("{}\n{{\"ruleId\":\n{}", vxp_first(), vxp_pad(VXP_PAD));
    let broken = [lead.as_bytes(), b"\xff\n", idx_roles_text().as_bytes()].concat();
    assert!(String::from_utf8(broken.clone()).is_err(), "全文は UTF-8 でない");
    vxp_roles(&place, IDXB_ROLES, &broken, ("", ""));
    let line = vxp_run(&place);
    vxp_failed(&place, &line, "unreadable");
    assert_eq!(vxp_record(&place, &line, "stderr"), format!("{IDXB_ROLES} {{tree}}: stream did not contain valid UTF-8"), "UTF-8 でないことを先に名指す");
    vxp_roles(&place, IDXB_ROLES, format!("{lead}{}", idx_roles_text()).as_bytes(), ("", ""));
    let plain = vxp_run(&place);
    vxp_failed(&place, &plain, "unreadable");
    assert!(vxp_record(&place, &plain, "stderr").starts_with("roles: 2 行目:"), "UTF-8 の行を外すと JSON でない行を名指す");
    clean(&[&place.repo, &place.state]);
}

/// 一致を書く途中で、書きかけの行を残して落ちた子（rc 3 と SIGKILL）は failed:rc で、記録の stderr の末は stdout でなく stderr の
/// 最後の行から組む。
#[test]
fn vixpipe_tool_dying_midway_is_failed_rc_from_stderr() {
    let place = idxb_with_fixtures();
    for (end, rc) in [("exit 3", "3"), ("kill -KILL $$", "-1")] {
        let tail = format!("printf '{{\"ruleId\":'\necho boom >&2\n{end}\n");
        let text = format!("{}{}", idx_roles_text(), vxp_pad(VXP_PAD));
        vxp_roles(&place, IDXB_ROLES, text.as_bytes(), ("", &tail));
        let line = vxp_run(&place);
        vxp_failed(&place, &line, "rc");
        assert_eq!(vxp_record(&place, &line, "stderr"), format!("{IDXB_ROLES} {{tree}}: rc {rc} boom"), "{end}: stderr の末");
    }
    clean(&[&place.repo, &place.state]);
}

/// 失敗の順は file を読んでいた形と同じ: 読めない一致の役の行の後の役の行が rc 1 なら failed:rc（後の行を名指す）、SCIP と一致の
/// 両方が読めない周は SCIP の理由を名指す。
#[test]
fn vixpipe_failures_keep_the_order_of_the_file_read() {
    let decl = format!("index-scip = [\"{IDXB_SCIP} {{tree}} {{out}}\"]\nindex-roles = [\"{IDXB_ROLES} {{tree}}\", \"{VXP_ROLES2} {{tree}}\"]\n");
    let place = idxb_place(&decl, "", "");
    let copy = format!("cp '{}' \"$2\"\n", place.state.join("idx.scip").display());
    idxb_script(&place, (IDXB_SCIP, "scip"), &copy);
    let broken = format!("{}\n{{\"ruleId\":\n{}", vxp_first(), vxp_pad(VXP_PAD));
    vxp_roles(&place, IDXB_ROLES, broken.as_bytes(), ("", ""));
    vxp_roles(&place, VXP_ROLES2, b"", ("", "echo late >&2\nexit 1\n"));
    let line = vxp_run(&place);
    vxp_failed(&place, &line, "rc");
    assert_eq!(vxp_record(&place, &line, "stderr"), format!("{VXP_ROLES2} {{tree}}: rc 1 late"), "後の行の rc を名指す");
    vxp_roles(&place, VXP_ROLES2, b"", ("", ""));
    idxb_script(&place, (IDXB_SCIP, "scip"), "printf '\\022\\200' > \"$2\"\n");
    let want = scip::read_scip(b"\x12\x80").map(|_| String::new()).unwrap_or_else(|err| err.to_string());
    let both = vxp_run(&place);
    vxp_failed(&place, &both, "unreadable");
    assert_eq!(vxp_record(&place, &both, "stderr"), want, "SCIP の理由を一致より先に名指す");
    clean(&[&place.repo, &place.state]);
}
