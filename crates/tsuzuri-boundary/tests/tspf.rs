//! 接頭辞 tspf_・設計ノート surface-v4a 行 t-stop-preflight の歯。
//! 終える前の門の契約の file の撃ち直しの歯（偽の scribe2 を PATH の頭に置き、tz hook agent-stop を撃つ）。
#![cfg(test)]

use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use tsuzuri_core::agent::spec::Spec;

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

/// 係の記録（頼みの行だけ・SendMessage の呼びは無い）。
fn asked() -> String {
    r#"{"type":"user","message":{"content":"頼み"}}"#.to_string() + "\n"
}

/// 歯ごとの置き場（前の撃ちの残りを消して作る）。drafts/ と、係の記録を置く親の記録の dir。
fn place(name: &str) -> PathBuf {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("tspf")
        .join(name);
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("drafts")).unwrap();
    fs::create_dir_all(root.join("s/subagents")).unwrap();
    root
}

/// 欠けの無い終わりの置き場: 係 w218a の札（出す物 notes.md）と係の id a77 の結び・係の記録・w/notes.md（要点の見出し付き）。
/// 偽の scribe2（置き場の bin）と空の dir empty も置く。
fn whole(name: &str) -> PathBuf {
    let root = place(name);
    let dir = root.join("drafts/w218a");
    fs::create_dir_all(dir.join("w")).unwrap();
    fs::write(
        dir.join("spec.json"),
        r#"{"name":"w218a","type":"tsuzuri:drafter","budget":1000,"build":"なし","target":"t3-hub.87","outputs":["notes.md"],"spawned":1,"agent_id":"a77","ended":null}"#,
    )
    .unwrap();
    fs::write(dir.join("w/notes.md"), "# 要点\n分かった所\n").unwrap();
    fs::create_dir_all(root.join("drafts/.agents")).unwrap();
    fs::write(root.join("drafts/.agents/a77"), "w218a\n").unwrap();
    fs::write(root.join("s/subagents/agent-a77.jsonl"), asked()).unwrap();
    fs::create_dir_all(root.join("empty")).unwrap();
    fs::create_dir_all(root.join("bin")).unwrap();
    let fake = root.join("bin/scribe2");
    let script = format!(
        "#!/bin/sh\necho \"$*\" >> {calls}\ncase \"$4\" in *bad.toml) echo 'refuse 1' >&2; exit 1;; esac\nexit 0\n",
        calls = root.join("calls.txt").display()
    );
    fs::write(&fake, script).unwrap();
    fs::set_permissions(&fake, fs::Permissions::from_mode(0o755)).unwrap();
    root
}

/// 係の出力の dir（出す物と契約の file の置き場）。
fn w(root: &Path) -> PathBuf {
    root.join("drafts/w218a/w")
}

/// 契約の file `names` を出力の dir の子 contract の直下に置く（中身は字 schema = 1 だけ）。
fn contracts(root: &Path, names: &[&str]) {
    let dir = w(root).join("contract");
    fs::create_dir_all(&dir).unwrap();
    for name in names {
        fs::write(dir.join(name), "schema = 1\n").unwrap();
    }
}

/// 最後の答え（1 行目 DONE・要点 1 行・最後の行に出力の dir の path・JSON の字のまま改行は \n の 2 字）。
fn said(root: &Path) -> String {
    format!("DONE\\n要点 1 行\\n{}", w(root).display())
}

/// 偽の scribe2 の在る PATH（置き場の bin と元の PATH）。
fn with_fake(root: &Path) -> String {
    format!(
        "{}:{}",
        root.join("bin").display(),
        std::env::var("PATH").unwrap_or_default()
    )
}

/// scribe2 の無い PATH（置き場の空の dir だけ）。
fn without(root: &Path) -> String {
    root.join("empty").display().to_string()
}

/// tz hook agent-stop に、係の id `head` の終わりの入力（止めた後の終わりか `again`・最後の文 `last`）を、子の PATH `path` で渡した結果。
fn stop(root: &Path, head: &str, again: bool, last: &str, path: &str) -> Output {
    let input = format!(
        r#"{{"session_id":"00000000-0000-4000-8000-000000000000","transcript_path":"{}","cwd":"/W","permission_mode":"bypassPermissions",{head}"hook_event_name":"SubagentStop","stop_hook_active":{again},"agent_transcript_path":"{}","last_assistant_message":"{last}"}}"#,
        root.join("s.jsonl").display(),
        root.join("s/subagents/agent-a77.jsonl").display()
    );
    let mut child = Command::new(env!("CARGO_BIN_EXE_tz"))
        .args(["hook", "agent-stop", "--repo"])
        .arg(root)
        .arg("--drafts")
        .arg(root.join("drafts"))
        .env("PATH", path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let _ = child.stdin.take().unwrap().write_all(input.as_bytes());
    child.wait_with_output().unwrap()
}

/// 係の終わりの頭の欄（係の id a77）。
const SUB: &str = r#""agent_id":"a77","agent_type":"tsuzuri:drafter","#;

/// 係 w218a の札。
fn spec(root: &Path) -> Spec {
    Spec::parse(&fs::read_to_string(root.join("drafts/w218a/spec.json")).unwrap()).unwrap()
}

/// 偽の scribe2 の撃ちの記録の行（無ければ空）。
fn calls(root: &Path) -> Vec<String> {
    fs::read_to_string(root.join("calls.txt"))
        .unwrap_or_default()
        .lines()
        .map(str::to_string)
        .collect()
}

/// 契約の file `file`（子 contract からの相対の名）の 1 回の撃ちの字。
fn shot(root: &Path, file: &str) -> String {
    format!(
        "pipe preflight --contract {} --bead t3-hub.87 --repo {} --state-dir {}",
        w(root).join("contract").join(file).display(),
        root.display(),
        root.join("drafts/w218a/pf-state").display()
    )
}

/// bad.toml の通らない欠けの字。
const BAD: &str =
    "契約の file contract/bad.toml の preflight が通らない（rc が 0 でない: refuse 1）";

/// 契約の file を名の順に撃ち、通らない file だけを欠けにして 1 度目の終わりを rc 2 で止める。
#[test]
fn tspf_contract_files_are_preflighted_in_name_order() {
    let root = whole("order");
    contracts(&root, &["bad.toml", "a.toml", "memo.md"]);
    fs::create_dir_all(w(&root).join("contract/sub")).unwrap();
    fs::write(w(&root).join("contract/sub/x.toml"), "schema = 1\n").unwrap();
    let out = stop(&root, SUB, false, &said(&root), &with_fake(&root));
    assert_eq!(out.status.code(), Some(2));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains(BAD), "{err}");
    assert!(!err.contains("contract/a.toml"), "{err}");
    assert_eq!(
        calls(&root),
        vec![shot(&root, "a.toml"), shot(&root, "bad.toml")]
    );
    assert_eq!(spec(&root).ended, None);
    let state = root.join("drafts/w218a/pf-state");
    assert!(state.is_dir());
    assert_eq!(fs::read_dir(state).unwrap().count(), 0);
}

/// 2 度目の終わりは通らない欠けを STOP-GATE.txt に書いて rc 0 で通し、札に終えの印を書く。
#[test]
fn tspf_second_end_records_the_hole_and_passes() {
    let root = whole("second");
    contracts(&root, &["a.toml", "bad.toml"]);
    let path = with_fake(&root);
    let first = stop(&root, SUB, false, &said(&root), &path);
    assert_eq!(first.status.code(), Some(2));
    let out = stop(&root, SUB, true, &said(&root), &path);
    assert_eq!(out.status.code(), Some(0));
    let gate = fs::read_to_string(w(&root).join("STOP-GATE.txt")).unwrap();
    assert_eq!(gate, format!("{BAD}\n"));
    assert!(spec(&root).ended.is_some());
}

/// 契約の file が無い終わりは偽の scribe2 を撃たず、a.toml だけの終わりは 1 度だけ撃って何も出さずに通す。
#[test]
fn tspf_no_contract_file_fires_nothing() {
    let none = whole("none");
    let memo = whole("memo");
    contracts(&memo, &["memo.md"]);
    // 本文 memo.md は散文の門が撃たれる（規則の表を置き、字は印を持つ文が参照 id を持つ 1 行で門を通す）。
    let design = memo.join("design-intent");
    fs::create_dir_all(&design).unwrap();
    fs::write(design.join("rules.yaml"), RULES).unwrap();
    fs::write(
        w(&memo).join("contract/memo.md"),
        "条 P-28.2 のとおり席は記帳しなければならない。\n",
    )
    .unwrap();
    for root in [&none, &memo] {
        let out = stop(root, SUB, false, &said(root), &with_fake(root));
        assert_eq!(out.status.code(), Some(0));
        assert!(!root.join("calls.txt").exists());
        assert!(spec(root).ended.is_some());
    }
    let one = whole("one");
    contracts(&one, &["a.toml"]);
    let out = stop(&one, SUB, false, &said(&one), &with_fake(&one));
    assert_eq!((out.status.code(), out.stderr.len()), (Some(0), 0));
    assert_eq!(calls(&one), vec![shot(&one, "a.toml")]);
    assert!(spec(&one).ended.is_some());
}

/// scribe2 の無い PATH は起動できない欠けにして 1 度目の終わりを止める。
#[test]
fn tspf_missing_scribe2_holds_once() {
    let root = whole("missing");
    contracts(&root, &["a.toml"]);
    let out = stop(&root, SUB, false, &said(&root), &without(&root));
    assert_eq!(out.status.code(), Some(2));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("契約の file contract/a.toml の preflight が通らない（起動できない）"),
        "{err}"
    );
}

/// 空でない state dir は偽の scribe2 を撃たず、state dir の欠けの 1 行で 1 度目の終わりを止める。
#[test]
fn tspf_state_dir_must_be_empty() {
    let root = whole("state");
    contracts(&root, &["a.toml"]);
    let state = root.join("drafts/w218a/pf-state");
    fs::create_dir_all(&state).unwrap();
    fs::write(state.join("x"), "x").unwrap();
    let out = stop(&root, SUB, false, &said(&root), &with_fake(&root));
    assert_eq!(out.status.code(), Some(2));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains(&format!(
            "preflight の空の state dir {} が無い（作れないか空でない）",
            state.display()
        )),
        "{err}"
    );
    assert!(!root.join("calls.txt").exists());
}
