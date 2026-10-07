//! 契約の本文の散文の門の歯（接頭辞 tprsb_・設計ノート surface-v4a 行 t-prose-bead・判断の記録 ADR-72 の決定 (2)）。
//! 口 tz check --prose と、中核の contract_gate の関数（writes・judge・output）を直に撃ち、口の歯は tz hook question-gate を
//! 偽の bd と偽の設計の道具（撃たれた引数を記録の file に 1 行足してから作業場の字を出す script）で、tz hook agent-stop を
//! 係の記録と出力の dir（--drafts）で撃つ。作業場は CARGO_TARGET_TMPDIR の下の tprsb/<歯の名>。
#![cfg(test)]

use std::cell::Cell;
use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use tsuzuri_boundary::server::ledger::BD_ARGS;
use tsuzuri_core::contract_gate::{
    Body, ContractGate, ContractWhy, ContractWrite, judge, output, writes,
};

/// 規則の行 R-16 の value の印の 1 つ目（main の rules.yaml と同じ字）。
const MARK: &str = "しなければならない";

/// toy の設計文書の規則の表（節 thresholds に行 R-16 の value を main の rules.yaml と同じ字で写した行）。
const RULES: &str = r#"thresholds:
  - id: R-16
    article: "N-2"
    what: "設計ノートの散文の門（規範の印の一覧と、印を持つ文に許さない「数と単位」の単位の一覧）"
    value:
      marks:
        - "しなければならない"
        - "してはならない"
        - "してはいけない"
        - "SHALL"
        - "MUST"
      prohibition:
        word: "禁止"
        clause_ends:
          - "。"
          - "）"
          - "・"
          - "、"
          - null
      units:
        - "秒"
        - "分"
        - "時間"
        - "日"
        - "件"
        - "本"
        - "行"
        - "byte"
        - "KB"
        - "MB"
        - "%"
        - "s"
        - "ms"
"#;

/// 見本の本文の 7 行（2 行目が参照 id 無し・3 行目が数と単位・4 行目から 6 行目が code fence・7 行目が表の行）。
fn sample() -> String {
    [
        format!("条 P-28.2 のとおり席は記帳{MARK}。"),
        format!("席は記帳{MARK}。"),
        format!("条 P-28.2 のとおり席は 3 本を記帳{MARK}。"),
        "```".to_string(),
        format!("席は記帳{MARK}。"),
        "```".to_string(),
        format!("| 席は記帳{MARK}。 |"),
    ]
    .join("\n")
        + "\n"
}

/// 見本の 1 行目だけの本文。
fn clean() -> String {
    format!("条 P-28.2 のとおり席は記帳{MARK}。\n")
}

/// 歯ごとの置き場（前の撃ちの残りを消して作る）。
fn fresh(name: &str) -> PathBuf {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("tprsb")
        .join(name);
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}

/// 置き場 `root` の下の設計文書の dir（規則の表を `rules` の字で置く）。
fn design(root: &Path, rules: &str) -> PathBuf {
    let dir = root.join("design-intent");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("rules.yaml"), rules).unwrap();
    dir
}

/// tz check --dir <dir> --prose <file> の結果。
fn check(dir: &Path, file: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tz"))
        .args(["check", "--dir"])
        .arg(dir)
        .arg("--prose")
        .arg(file)
        .output()
        .unwrap()
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).unwrap()
}

/// 見本の 7 行の file は rc 1 で、違反の行を 1 つずつ名指して要約の行で閉じる。
#[test]
fn tprsb_check_prose_names_each_hole() {
    let root = fresh("check-holes");
    let dir = design(&root, RULES);
    let file = root.join("body.md");
    fs::write(&file, sample()).unwrap();
    let out = check(&dir, &file);
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    let shown = file.display();
    let want = format!(
        "{shown}: 行 2: no-pointer 席は記帳{MARK}。\n\
         {shown}: 行 3: number-with-unit 条 P-28.2 のとおり席は 3 本を記帳{MARK}。\n\
         folio check --prose: 不合格（印を持つ文 3・違反 2）\n"
    );
    assert_eq!(text(&out.stdout), want);
}

/// 印を持つ文が全部通る file と、印を持つ文の無い file は rc 0 で要約の 1 行だけ。
#[test]
fn tprsb_check_prose_passes_clean_text() {
    let root = fresh("check-clean");
    let dir = design(&root, RULES);
    let file = root.join("body.md");
    for (body, want) in [
        (clean(), "folio check --prose: 合格（印を持つ文 1・違反 0）\n"),
        (
            "席は記帳する。\n".to_string(),
            "folio check --prose: 合格（印を持つ文 0・違反 0）\n",
        ),
    ] {
        fs::write(&file, body).unwrap();
        let out = check(&dir, &file);
        assert_eq!(out.status.code(), Some(0), "{out:?}");
        assert_eq!(text(&out.stdout), want);
    }
}

/// 規則の表が読めない・file が読めない周は rc 2 で、理由の 1 行と まだ分からない の要約の 1 行。
#[test]
fn tprsb_check_prose_unknowns() {
    let root = fresh("check-unknown");
    let good = design(&root, RULES);
    let ok = root.join("ok.md");
    fs::write(&ok, clean()).unwrap();
    let bin = root.join("bin.md");
    fs::write(&bin, [0xff_u8, 0xfe, 0x80, b'\n']).unwrap();
    let norow = root.join("norow");
    let no_r16 = design(&norow, "thresholds: []\n");
    let no_table = root.join("none");
    fs::create_dir_all(&no_table).unwrap();
    let cases = [
        (no_r16, ok.clone()),
        (no_table, ok.clone()),
        (good.clone(), root.join("missing.md")),
        (good.clone(), bin),
        (good, root.clone()),
    ];
    for (dir, file) in cases {
        let out = check(&dir, &file);
        assert_eq!(out.status.code(), Some(2), "{dir:?} {file:?} {out:?}");
        let stdout = text(&out.stdout);
        let lines: Vec<&str> = stdout.lines().collect();
        assert_eq!(lines.len(), 2, "{stdout}");
        assert!(lines[0].starts_with("# まだ分からない: "), "{stdout}");
        assert_eq!(lines[1], "folio check --prose: まだ分からない");
    }
}

/// Bash の PreToolUse の hook の入力の字（道具の名 `tool`）。
fn payload_of(tool: &str, command: &str) -> String {
    let escaped = command
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n");
    format!(
        r#"{{"session_id":"s-1","hook_event_name":"PreToolUse","tool_name":"{tool}","tool_input":{{"command":"{escaped}"}}}}"#
    )
}

fn payload(command: &str) -> String {
    payload_of("Bash", command)
}

/// 契約の行を持つ --acceptance の値（1 行目が [[contract]]・shell の引用符つき）。
const ACC: &str = "'[[contract]]\ndesign = contracts/x.toml#r1'";

fn file(path: &str) -> ContractWrite {
    ContractWrite::Contract {
        body: Body::File(path.to_string()),
    }
}

fn body(body: Body) -> ContractWrite {
    ContractWrite::Contract { body }
}

fn pick(command: &str) -> Vec<ContractWrite> {
    writes(&payload(command))
}

const BD: &str = "bdw update fx-c.1";

/// 甲の本文の形（file・file でない形・無し）。
fn picks_contract_forms() {
    let bd = BD;
    assert_eq!(pick(&format!("{bd} --acceptance {ACC} --body-file /abs/b.md")), [file("/abs/b.md")]);
    assert_eq!(pick(&format!("{bd} --acceptance={ACC} --body-file=/abs/b.md")), [file("/abs/b.md")]);
    assert_eq!(
        pick(&format!("X=1 /usr/bin/bd create 題 --acceptance {ACC} --body-file /abs/b.md")),
        [file("/abs/b.md")]
    );
    for tail in ["--body-file b.md", "--body-file -", "-d 本文", "--description=本文", "--stdin"] {
        assert_eq!(
            pick(&format!("{bd} --acceptance {ACC} {tail}")),
            [body(Body::NotFile)],
            "{tail}"
        );
    }
    assert_eq!(pick(&format!("{bd} --acceptance {ACC}")), [body(Body::Absent)]);
}

/// 欄 acceptance が design の行なら甲でも乙でもなく、何度在っても最後の値で判じる。
fn picks_last_acceptance() {
    let (bd, design) = (BD, "'design = contracts/x.toml#r1'");
    assert_eq!(
        pick(&format!("{bd} --acceptance {design} --acceptance {ACC} --body-file /abs/b.md")),
        [file("/abs/b.md")]
    );
    assert!(pick(&format!("{bd} --acceptance {ACC} --acceptance {design} --body-file /abs/b.md")).is_empty());
    assert!(pick(&format!("bd update fx-c.1 --acceptance {design} --body-file /abs/b.md")).is_empty());
}

/// 乙は --acceptance の無い本文だけの直し（名指しの id は語 update の後の - で始まらない語）で、一続きは command の順に拾う。
fn picks_body_only() {
    let bd = BD;
    let two = pick(&format!("{bd} --acceptance {ACC} --body-file /abs/1.md; bd update fx-c.2 --body-file /abs/2.md"));
    assert_eq!(two.len(), 2, "{two:?}");
    assert_eq!(two[0], file("/abs/1.md"));
    assert!(matches!(&two[1], ContractWrite::BodyOnly { ids } if ids.contains(&"fx-c.2".to_string())));
    let only = pick("bd update fx-c.1 --body-file /abs/b.md");
    assert!(matches!(&only[..], [ContractWrite::BodyOnly { ids }] if ids.contains(&"fx-c.1".to_string())), "{only:?}");
    assert!(matches!(&pick("bd update fx-c.1 -d 本文")[..], [ContractWrite::BodyOnly { .. }]));
    assert!(pick("bd update fx-c.1 --title 題").is_empty());
}

/// 0 件: bd list・頭の語が echo・Bash でない道具。
fn picks_nothing() {
    assert!(pick("bd list --json").is_empty());
    assert!(pick(&format!("echo bdw update fx-c.1 --acceptance {ACC} --body-file /abs/b.md")).is_empty());
    let other = payload_of("Write", &format!("bdw update fx-c.1 --acceptance {ACC} --body-file /abs/b.md"));
    assert!(writes(&other).is_empty());
}

/// 契約の書きの見分け: 甲は本文の形（file・file でない形・無し）を持ち、乙は名指しの id を持つ。
#[test]
fn tprsb_writes_pick_contract_writes() {
    picks_contract_forms();
    picks_last_acceptance();
    picks_body_only();
    picks_nothing();
}

/// 台帳の字（fx-c.1 の acceptance が契約の行を、fx-c.2 の acceptance が design の行を持つ）。
const LEDGER: &str = r#"[{"id":"fx-c.1","title":"契約","acceptance_criteria":"[[contract]]\ndesign = contracts/x.toml#r1"},{"id":"fx-c.2","title":"別","acceptance_criteria":"design = contracts/x.toml#r2"}]"#;

fn only(id: &str) -> ContractWrite {
    ContractWrite::BodyOnly {
        ids: vec![id.to_string(), "/abs/b.md".to_string()],
    }
}

/// 撃ちの関数の撃たれた回数を数える（rc ごとの偽の散文の門）。
struct Shots(Cell<u32>);

impl Shots {
    fn fire(&self, rc: u8) -> impl Fn(&str) -> (u8, String) + '_ {
        move |path| {
            self.0.set(self.0.get() + 1);
            (
                rc,
                format!("{path}: 行 2: no-pointer 席は記帳\nfolio check --prose: 不合格（印を持つ文 3・違反 1）\n"),
            )
        }
    }

    fn count(&self) -> u32 {
        self.0.get()
    }
}

fn form() -> ContractGate {
    ContractGate::Deny { why: ContractWhy::BodyForm, lines: Vec::new() }
}

/// 甲は本文の形で止め、本文が file なら撃ちの関数の rc で答えが変わる。
fn judges_contract_writes(shots: &Shots) {
    let fire = |rc| shots.fire(rc);
    assert_eq!(judge(&[body(Body::NotFile)], fire(0), ""), form());
    assert_eq!(judge(&[body(Body::Absent)], fire(0), ""), form());
    assert_eq!(shots.count(), 0);
    assert_eq!(
        judge(&[file("/abs/b.md")], fire(1), ""),
        ContractGate::Deny {
            why: ContractWhy::Prose,
            lines: vec!["/abs/b.md: 行 2: no-pointer 席は記帳".to_string()],
        }
    );
    for rc in [2, 3] {
        assert_eq!(
            judge(&[file("/abs/b.md")], fire(rc), ""),
            ContractGate::Unknown {
                why: ContractWhy::ProseUnknown,
                lines: vec!["/abs/b.md: 行 2: no-pointer 席は記帳".to_string()],
            }
        );
    }
    assert_eq!(judge(&[file("/abs/b.md")], fire(0), ""), ContractGate::Allow);
    assert_eq!(shots.count(), 4);
}

/// 乙は撃たずに台帳の字で決める。
fn judges_body_only(shots: &Shots) {
    let fire = |rc| shots.fire(rc);
    let before = shots.count();
    assert_eq!(judge(&[only("fx-c.1")], fire(0), LEDGER), form());
    assert_eq!(judge(&[only("fx-c.2")], fire(0), LEDGER), ContractGate::Allow);
    assert_eq!(judge(&[only("fx-c.9")], fire(0), "[]"), ContractGate::Allow);
    for unread in ["", "壊れた"] {
        assert_eq!(
            judge(&[only("fx-c.2")], fire(0), unread),
            ContractGate::Unknown { why: ContractWhy::LedgerUnread, lines: Vec::new() }
        );
    }
    assert_eq!(shots.count(), before);
}

/// command の順に判じ、最初に通さない答えで止まる（後の下書きの撃ちは起きない）。
fn judges_in_order(shots: &Shots) {
    let fire = |rc| shots.fire(rc);
    let before = shots.count();
    let through = judge(&[file("/abs/b.md"), only("fx-c.1"), body(Body::Absent)], fire(0), LEDGER);
    assert_eq!(through, form());
    assert_eq!(shots.count(), before + 1);
    let stopped = judge(&[body(Body::Absent), file("/abs/b.md")], fire(0), LEDGER);
    assert_eq!(stopped, form());
    assert_eq!(shots.count(), before + 1);
}

/// 判じ: 撃ちの関数が返す rc ごとに答えが変わり、乙は台帳の字で決まる。最初に通さない答えで止まり、後の下書きの撃ちは起きない。
#[test]
fn tprsb_judge_answers() {
    let shots = Shots(Cell::new(0));
    judges_contract_writes(&shots);
    judges_body_only(&shots);
    judges_in_order(&shots);
}

/// 理由の閉じた 4 語と、答えの deny の JSON の字。
#[test]
fn tprsb_output_words() {
    let words: Vec<&str> = ContractWhy::ALL.iter().map(|w| w.word()).collect();
    assert_eq!(
        words,
        ["contract-body-form", "contract-prose", "contract-prose-unknown", "contract-ledger-unread"]
    );
    assert_eq!(output(&ContractGate::Allow), None);
    let reason = |text: &str| format!(r#""permissionDecisionReason":"{text}"#);
    let lines = vec!["a: 行 2: no-pointer 席".to_string(), "b: 行 3: number-with-unit 席".to_string()];
    for why in ContractWhy::ALL {
        let stop = output(&ContractGate::Deny { why, lines: Vec::new() }).unwrap();
        assert!(stop.starts_with(r#"{"hookSpecificOutput":{"#), "{stop}");
        assert!(stop.contains(r#""permissionDecision":"deny""#), "{stop}");
        assert!(stop.contains(r#""hookEventName":"PreToolUse""#), "{stop}");
        assert!(
            stop.contains(&reason(&format!("契約の本文の散文の門は止める（{}） 次の一手 = ", why.word()))),
            "{stop}"
        );
        let unknown = output(&ContractGate::Unknown { why, lines: lines.clone() }).unwrap();
        assert!(
            unknown.contains(&reason(&format!(
                "契約の本文の散文の門は まだ分からない（{}） a: 行 2: no-pointer 席 / b: 行 3: number-with-unit 席 次の一手 = ",
                why.word()
            ))),
            "{unknown}"
        );
    }
    let next = |why| output(&ContractGate::Deny { why, lines: Vec::new() }).unwrap();
    let form = next(ContractWhy::BodyForm);
    assert!(form.contains("--acceptance") && form.contains("--body-file") && form.contains("絶対 path"), "{form}");
    assert!(next(ContractWhy::Prose).contains("参照 id"));
    for why in [ContractWhy::ProseUnknown, ContractWhy::LedgerUnread] {
        assert!(next(why).contains("読めるようになってから書き直す"));
    }
}

/// 起票の門の歯の作業場（repo と記録の置き場と偽の bd と偽の設計の道具）。
struct Place {
    root: PathBuf,
    repo: PathBuf,
    log: PathBuf,
}

impl Place {
    fn new(name: &str) -> Place {
        let root = fresh(name);
        let (repo, log) = (root.join("repo"), root.join("log"));
        fs::create_dir_all(&repo).unwrap();
        fs::create_dir_all(&log).unwrap();
        design(&repo, RULES);
        fs::write(root.join("ledger.json"), LEDGER).unwrap();
        fs::write(root.join("index.tsv"), "").unwrap();
        for (program, out) in [("bd", "ledger.json"), ("folio", "index.tsv")] {
            let path = root.join(program);
            let script = format!(
                "#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{}'\nexec cat '{}'\n",
                log.join(format!("{program}.log")).display(),
                root.join(out).display()
            );
            fs::write(&path, script).unwrap();
            fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        }
        Place { root, repo, log }
    }

    /// 偽の program が撃たれた回ごとの引数の行。
    fn calls(&self, program: &str) -> Vec<String> {
        fs::read_to_string(self.log.join(format!("{program}.log")))
            .map(|t| t.lines().map(str::to_string).collect())
            .unwrap_or_default()
    }

    /// 作業場の下に本文の file `name` を置いて絶対 path を返す。
    fn put(&self, name: &str, content: &str) -> PathBuf {
        let path = self.root.join(name);
        fs::write(&path, content).unwrap();
        path
    }

    /// tz hook question-gate を repo と偽の program で撃ち、標準入力に payload を書いて終わりまで待つ。
    fn tz(&self, payload: &str) -> Output {
        let mut child = Command::new(env!("CARGO_BIN_EXE_tz"))
            .args(["hook", "question-gate", "--repo"])
            .arg(&self.repo)
            .arg("--bd")
            .arg(self.root.join("bd"))
            .arg("--folio")
            .arg(self.root.join("folio"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let mut stdin = child.stdin.take().unwrap();
        let _ = stdin.write_all(payload.as_bytes());
        drop(stdin);
        child.wait_with_output().unwrap()
    }

    /// repo の中の file の path と byte の一覧（書かれていないことを比べる）。
    fn tree(&self) -> Vec<(PathBuf, Vec<u8>)> {
        let mut out = Vec::new();
        let mut stack = vec![self.repo.clone()];
        while let Some(dir) = stack.pop() {
            for entry in fs::read_dir(&dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    out.push((path.clone(), Vec::new()));
                    stack.push(path);
                } else {
                    out.push((path.clone(), fs::read(&path).unwrap()));
                }
            }
        }
        out.sort();
        out
    }
}

/// 契約の書き（甲）の起票の command（短い題の門を通す metadata の short つき）。
fn create_with(body_file: &Path) -> String {
    payload(&format!(
        "bdw create --metadata='{{\"short\":\"契約\"}}' --acceptance {ACC} --body-file {} 契約の題",
        body_file.display()
    ))
}

/// 甲の本文の file が散文の門で落ちれば deny の 1 行を出して止め、通れば何も出さず、偽の bd と偽の設計の道具は撃たない。
#[test]
fn tprsb_bin_contract_body_is_gated() {
    let place = Place::new("bin-body");
    let before = place.tree();
    let bad = place.put("bad.md", &sample());
    let out = place.tz(&create_with(&bad));
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let stdout = text(&out.stdout);
    assert!(stdout.ends_with('\n') && stdout.matches('\n').count() == 1, "{stdout}");
    for want in ["contract-prose", "行 2", "no-pointer", "number-with-unit"] {
        assert!(stdout.contains(want), "{want}: {stdout}");
    }
    assert!(stdout.contains(r#""permissionDecision":"deny""#), "{stdout}");
    let good = place.put("good.md", &clean());
    let out = place.tz(&create_with(&good));
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    assert!(out.stdout.is_empty(), "{out:?}");
    assert!(place.calls("bd").is_empty(), "偽の bd を撃つ");
    assert!(place.calls("folio").is_empty(), "偽の設計の道具を撃つ");
    assert!(before == place.tree(), "repo の byte が変わる");
}

/// 本文だけの直し（乙）は台帳を 1 度だけ読み、名指しの bead が契約なら止め、契約でなければ通す。
#[test]
fn tprsb_bin_body_only_update() {
    let want_call = [BD_ARGS.join(" ")];
    for (id, stops) in [("fx-c.1", true), ("fx-c.2", false)] {
        let place = Place::new(&format!("bin-only-{id}"));
        let file = place.put("body.md", &clean());
        let before = place.tree();
        let out = place.tz(&payload(&format!("bd update {id} --body-file {}", file.display())));
        assert_eq!(out.status.code(), Some(0), "{out:?}");
        let stdout = text(&out.stdout);
        if stops {
            assert!(stdout.ends_with('\n') && stdout.matches('\n').count() == 1, "{stdout}");
            assert!(stdout.contains("contract-body-form"), "{stdout}");
        } else {
            assert!(stdout.is_empty(), "{stdout}");
        }
        assert_eq!(place.calls("bd"), want_call, "{id}");
        assert!(place.calls("folio").is_empty(), "偽の設計の道具を撃つ");
        assert!(before == place.tree(), "repo の byte が変わる");
    }
}

/// 係の終える前の門の置き場: 係 w218a の札（出す物 notes.md）と係の id a77 の結び・係の記録・w/notes.md（要点の見出し付き）と、
/// toy の repo（design-intent の規則の表）。
fn whole(name: &str) -> PathBuf {
    let root = fresh(name);
    let dir = root.join("drafts/w218a");
    fs::create_dir_all(dir.join("w/contract")).unwrap();
    fs::create_dir_all(root.join("s/subagents")).unwrap();
    fs::create_dir_all(root.join("drafts/.agents")).unwrap();
    design(&root, RULES);
    fs::write(
        dir.join("spec.json"),
        r#"{"name":"w218a","type":"tsuzuri:drafter","budget":1000,"build":"なし","target":"t3-hub.87","outputs":["notes.md"],"spawned":1,"agent_id":"a77","ended":null}"#,
    )
    .unwrap();
    fs::write(dir.join("w/notes.md"), "# 要点\n分かった所\n").unwrap();
    fs::write(root.join("drafts/.agents/a77"), "w218a\n").unwrap();
    let asked = r#"{"type":"user","message":{"content":"頼み"}}"#;
    fs::write(root.join("s/subagents/agent-a77.jsonl"), format!("{asked}\n")).unwrap();
    root
}

/// 係の出力の dir。
fn w(root: &Path) -> PathBuf {
    root.join("drafts/w218a/w")
}

/// tz hook agent-stop に係 a77 の終わり（止めた後の終わりか `again`）を toy の repo と共に渡した結果。
fn stop(root: &Path, again: bool) -> Output {
    let last = format!("DONE\\n要点 1 行\\n{}", w(root).display());
    let input = format!(
        r#"{{"session_id":"00000000-0000-4000-8000-000000000000","transcript_path":"{}","cwd":"/W","permission_mode":"bypassPermissions","agent_id":"a77","agent_type":"tsuzuri:drafter","hook_event_name":"SubagentStop","stop_hook_active":{again},"agent_transcript_path":"{}","last_assistant_message":"{last}"}}"#,
        root.join("s.jsonl").display(),
        root.join("s/subagents/agent-a77.jsonl").display()
    );
    let mut child = Command::new(env!("CARGO_BIN_EXE_tz"))
        .args(["hook", "agent-stop", "--repo"])
        .arg(root)
        .arg("--drafts")
        .arg(root.join("drafts"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let _ = child.stdin.take().unwrap().write_all(input.as_bytes());
    child.wait_with_output().unwrap()
}

/// 出力の dir の子 contract の .md の本文が散文の門で落ちる係の終わりは、1 度目を rc 2 で止め、2 度目を通して STOP-GATE.txt に書く。
/// 落ちない .md と .md でない file は欠けに数えない。
#[test]
fn tprsb_stop_holds_a_failing_body() {
    let root = whole("stop");
    let contract = w(&root).join("contract");
    fs::write(contract.join("a.md"), sample()).unwrap();
    fs::write(contract.join("b.md"), clean()).unwrap();
    fs::write(contract.join("c.txt"), sample()).unwrap();
    let hole = format!("契約の本文 {} が散文の門で落ちる（", contract.join("a.md").display());
    let first = stop(&root, false);
    assert_eq!(first.status.code(), Some(2), "{first:?}");
    let err = text(&first.stderr);
    assert!(err.contains(&hole), "{err}");
    assert!(err.contains("行 2: no-pointer") && err.contains("行 3: number-with-unit"), "{err}");
    assert!(!err.contains("b.md") && !err.contains("c.txt"), "{err}");
    let second = stop(&root, true);
    assert_eq!(second.status.code(), Some(0), "{second:?}");
    let gate = fs::read_to_string(w(&root).join("STOP-GATE.txt")).unwrap();
    assert!(gate.contains(&hole), "{gate}");
    assert!(!gate.contains("b.md") && !gate.contains("c.txt"), "{gate}");
}
