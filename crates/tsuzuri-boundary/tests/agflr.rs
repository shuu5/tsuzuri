//! 終える前の門の床の記録と撃ち直しの歯（接頭辞 agflr_・設計ノート surface-wave29c 行 ag-floor-rec・判断の記録 ADR-63 の決定 (4)）。
//! 中核の `floor` の純関数を直に撃ち、tz の binary の tz hook agent-stop に係の終わりの入力を標準入力で渡して、歯ごとの置き場
//! （CARGO_TARGET_TMPDIR の下）を --drafts で、偽の道具（tz と scribe2 の殻の script・撃たれた cwd と引数を log に足す）を --tz と
//! --scribe2 で渡す。参照の file の 7 節と 11 節の句は、手引きが天井の文で振る舞いを持たないので、節の塊の中の字を照らす。
#![cfg(test)]

use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::thread;
use std::time::Duration;

use tsuzuri_core::agent::spec::Spec;
use tsuzuri_core::agent::stop::floor::{
    Kind, Rec, Rerun, Verdict, judge, locate, log_line, record, rows, table_rc,
};

/// 見本の契約表（行 r0 の頭は 3 行目で verify は 2 行・行 r1 の頭は 9 行目で verify は 3 行）。
const TOML: &str = "schema = 1\n\n[[contract]]\nid = \"r0\"\ntitle = \"t\"\nverify = [\"a\", \"b\"]\ndone = \"d\"\n\n\
[[contract]]\nid = \"r1\"\ntitle = \"t\"\nverify = [\"a\", \"b\", \"c\"]\ndone = \"d\"\n";

#[test]
fn agflr_rows_are_the_patch_outputs_once_in_order() {
    let outs = [
        "w/notes.md",
        "w/patch/b.patch",
        "a.patch",
        "w/patch/b.patch",
        "w/patch/.patch",
        "c.patches",
    ]
    .map(String::from);
    assert_eq!(rows(&outs), ["b", "a"]);
    assert!(rows(&["notes.md".to_string()]).is_empty());
}

#[test]
fn agflr_record_reads_six_tab_fields_and_skips_the_rest() {
    let text = "#r9\t0\t0\t0\t0\t0\nr0\t0\t0\t0\t0\t\nr1\t0\t1\t0\t2\t0,101\nr2\t0\t0\t0\t0\nr3\t0\tx\t0\t0\t0\n\
r4\t0\t0\t0\t0\t0,,1\n\t0\t0\t0\t0\t0\nr5\t2\t0\t0\t0\t101\nr1\t0\t0\t0\t0\t0,0\n";
    let got = record(text);
    assert_eq!(got.keys().collect::<Vec<_>>(), ["r0", "r1", "r5"]);
    let rec = |rcs: [i32; 4], verify: &[i32]| Rec {
        rcs,
        verify: verify.to_vec(),
    };
    assert_eq!(got.get("r0"), Some(&rec([0; 4], &[])));
    assert_eq!(got.get("r1"), Some(&rec([0; 4], &[0, 0])));
    assert_eq!(got.get("r5"), Some(&rec([2, 0, 0, 0], &[101])));
}

#[test]
fn agflr_locate_finds_the_row_head_and_its_verify_lines() {
    assert_eq!(locate(TOML, "r1"), Some((9, 3)));
    assert_eq!(locate(TOML, "r0"), Some((3, 2)));
    assert_eq!(locate(TOML, "r"), None);
    let bare = TOML.replacen("verify = [\"a\", \"b\"]\n", "", 1);
    assert_eq!(locate(&bare, "r0"), None);
    assert_eq!(locate(&TOML.replace("[[contract]]", "[x]"), "r1"), None);
}

#[test]
fn agflr_table_rc_counts_only_lines_naming_the_row_head() {
    let note = "contracts/n.toml";
    let out = "contracts: contracts/n.toml:3 contract-table:done-teeth: 行 r0\n\
contracts: contracts/m.toml:9 write-set-item-unresolved: x\ncontracts: contracts/n.toml:90 x\n  contracts: contracts/n.toml:9 x\n";
    assert_eq!(table_rc(out, 1, note, 9), 0);
    assert_eq!(table_rc(out, 1, note, 3), 1);
    let named =
        format!("{out}contracts: contracts/n.toml:9 contract-table:done-teeth-missing: 行 r1\n");
    assert_eq!(table_rc(&named, 1, note, 9), 1);
    assert_eq!(table_rc(out, 2, note, 3), 2);
    assert_eq!(table_rc("", 0, note, 9), 0);
}

/// 撃ち直しの見本。
fn re(rcs: [i32; 4], verify: usize) -> Result<Rerun, String> {
    Ok(Rerun { rcs, verify })
}

/// 行ごとの (行 id・型・詳細)。
fn kinds(all: &[Verdict]) -> Vec<(&str, Kind, &str)> {
    all.iter()
        .map(|v| (v.row.as_str(), v.kind, v.detail.as_str()))
        .collect()
}

#[test]
fn agflr_judge_names_one_kind_per_row() {
    let ids = ["ok", "gone", "far", "lie", "red", "short", "vred", "both"].map(String::from);
    let text = "ok\t0\t0\t0\t0\t0,0\nfar\t0\t0\t0\t0\t0,0\nlie\t0\t0\t0\t0\t0,0\nred\t1\t0\t0\t0\t0,0\n\
short\t0\t0\t0\t0\t0\nvred\t0\t0\t0\t0\t0,101\nboth\t1\t0\t0\t1\t0,0\n";
    let (all, lacks) = judge(&ids, Some(text), |r| match r {
        "far" => Err("床の写し /D/try-w-fl が dir として無い".to_string()),
        "lie" => re([0, 0, 0, 1], 2),
        "red" | "both" => re([1, 0, 0, 0], 2),
        _ => re([0; 4], 2),
    });
    assert_eq!(
        kinds(&all),
        [
            ("ok", Kind::Ok, "-"),
            ("gone", Kind::NoRow, "-"),
            (
                "far",
                Kind::Unknown,
                "床の写し /D/try-w-fl が dir として無い"
            ),
            ("lie", Kind::Mismatch, "table=0/1"),
            ("red", Kind::Rc, "preflight=1"),
            ("short", Kind::NoRow, "verify=1/2"),
            ("vred", Kind::Rc, "verify3=101"),
            ("both", Kind::Mismatch, "preflight=1,table=1/0"),
        ]
    );
    assert_eq!(
        lacks,
        [
            "床の記録 floor.tsv に行 gone の行が無い（欄はタブで区切った 6 つ・rc は整数）",
            "床を撃ち直せない（まだ分からない）: 床の写し /D/try-w-fl が dir として無い",
            "床の記録の行 lie の 器の表の検査 の rc 0 が撃ち直しの rc 1 と違う",
            "床の記録の行 red の preflight の rc が 1（0 でない）",
            "床の記録の行 short の verify の rc が 1 個（verify の 2 行目から最後の行の 2 個が要る）",
            "床の記録の行 vred の verify の 3 行目の rc が 101（0 でない）",
            "床の記録の行 both の preflight の rc が 1（0 でない）",
            "床の記録の行 both の 器の表の検査 の rc 1 が撃ち直しの rc 0 と違う",
        ]
    );
}

#[test]
fn agflr_judge_without_a_record_or_with_one_cause_names_it_once() {
    let ids = ["a", "b"].map(String::from);
    let (all, lacks) = judge(&ids, None, |_| panic!("記録の無い時は撃ち直さない"));
    assert_eq!(
        kinds(&all),
        [("a", Kind::NoRecord, "-"), ("b", Kind::NoRecord, "-")]
    );
    assert_eq!(
        lacks,
        ["床の記録 floor.tsv が出力の dir に無い（差の file の行 a・b ごとに 1 行）"]
    );
    let text = "a\t0\t0\t0\t0\t\nb\t0\t0\t0\t0\t\n";
    let (all, lacks) = judge(&ids, Some(text), |_| Err("道具 x を撃てない".to_string()));
    assert_eq!(
        all.iter().map(|v| v.kind).collect::<Vec<_>>(),
        [Kind::Unknown; 2]
    );
    assert_eq!(
        lacks,
        ["床を撃ち直せない（まだ分からない）: 道具 x を撃てない"]
    );
    let (_, none) = judge(&[], None, |_| panic!("行の無い時は撃ち直さない"));
    assert!(none.is_empty());
}

#[test]
fn agflr_kinds_and_log_line_keep_the_measure_form() {
    let names = [
        Kind::NoRecord,
        Kind::NoRow,
        Kind::Unknown,
        Kind::Mismatch,
        Kind::Rc,
        Kind::Ok,
    ]
    .map(Kind::as_str);
    assert_eq!(
        names,
        ["no-record", "no-row", "unknown", "mismatch", "rc", "ok"]
    );
    let v = Verdict {
        row: "r1".into(),
        kind: Kind::Mismatch,
        detail: "table=0/1".into(),
        lacks: vec!["x".into()],
    };
    assert_eq!(
        log_line("20261005T1200Z", false, &v),
        "20261005T1200Z\t1\tr1\tmismatch\ttable=0/1\n"
    );
    assert_eq!(
        log_line("20261005T1200Z", true, &v),
        "20261005T1200Z\t2\tr1\tmismatch\ttable=0/1\n"
    );
}

/// 係の記録の、席への SendMessage の在る行。
const TOLD: &str = r#"{"type":"assistant","message":{"id":"A","content":[{"type":"tool_use","id":"t1","name":"SendMessage","input":{"to":"team-lead","message":"済み"}}]}}"#;

/// 歯ごとの置き場（前の撃ちの残りを消して作る）: 係 w300a（係の id a77・対象 t3-hub.87・出す物 `outputs`）の札と結びと記録と出す物と、
/// 床の写し（見本の契約表 contracts/n.toml と dir design-intent）。`rec` が在れば床の記録を置く。
fn whole(name: &str, outputs: &[&str], rec: Option<&str>) -> PathBuf {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("agflr")
        .join(name);
    let _ = fs::remove_dir_all(&root);
    let agent = root.join("drafts/w300a");
    let floor = root.join("drafts/try-w300a-fl");
    for dir in [
        agent.join("w/patch"),
        root.join("drafts/.agents"),
        root.join("s"),
        root.join("bin"),
        floor.join("contracts"),
        floor.join("design-intent"),
    ] {
        fs::create_dir_all(dir).unwrap();
    }
    let outs: Vec<String> = outputs.iter().map(|o| format!("{o:?}")).collect();
    let spec = format!(
        r#"{{"name":"w300a","type":"tsuzuri:drafter","budget":1000,"build":"重","target":"t3-hub.87","outputs":[{}],"spawned":1,"agent_id":"a77","ended":null}}"#,
        outs.join(",")
    );
    fs::write(agent.join("spec.json"), spec).unwrap();
    for o in outputs {
        fs::write(agent.join("w").join(o), "x\n").unwrap();
    }
    if let Some(text) = rec {
        fs::write(agent.join("w/floor.tsv"), text).unwrap();
    }
    fs::write(root.join("drafts/.agents/a77"), "w300a\n").unwrap();
    fs::write(root.join("s/agent-a77.jsonl"), format!("{TOLD}\n")).unwrap();
    fs::write(floor.join("contracts/n.toml"), TOML).unwrap();
    root
}

/// 偽の道具を置く（tz は check と derive の rc・scribe2 は preflight の rc と、器の表の検査が出す行と rc 1）。撃たれた cwd と引数を log に足す。
fn fake(root: &Path, (check, derive, pre): (i32, i32, i32), lines: &[&str]) {
    let log = |tool: &str| {
        format!(
            "echo \"$(pwd -P) $*\" >> '{}'\n",
            root.join(format!("{tool}.log")).display()
        )
    };
    let tz = format!(
        "#!/bin/sh\n{}case \"$1\" in\ncheck) exit {check} ;;\nderive) exit {derive} ;;\nesac\nexit 9\n",
        log("tz")
    );
    let said: String = lines.iter().map(|l| format!("echo '{l}'\n")).collect();
    let scribe2 = format!(
        "#!/bin/sh\n{}case \"$1\" in\npipe) exit {pre} ;;\ncontracts)\n{said}exit 1 ;;\nesac\nexit 9\n",
        log("scribe2")
    );
    for (name, body) in [("tz", tz), ("scribe2", scribe2)] {
        let path = root.join("bin").join(name);
        fs::write(&path, body).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    }
}

/// tz hook agent-stop に係の終わり（止めた後の終わりか `again`）と、--repo と --drafts の後の引数 `extra` を渡した結果。
fn hook(root: &Path, again: bool, extra: &[String]) -> Output {
    let input = format!(
        r#"{{"session_id":"00000000-0000-4000-8000-000000000000","agent_id":"a77","agent_type":"tsuzuri:drafter","hook_event_name":"SubagentStop","stop_hook_active":{again},"agent_transcript_path":"{}","last_assistant_message":"済み。出す物は {} に在る"}}"#,
        root.join("s/agent-a77.jsonl").display(),
        root.join("drafts/w300a/w").display()
    );
    let mut child = Command::new(env!("CARGO_BIN_EXE_tz"))
        .args(["hook", "agent-stop", "--repo"])
        .arg(root)
        .arg("--drafts")
        .arg(root.join("drafts"))
        .args(extra)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let _ = child.stdin.take().unwrap().write_all(input.as_bytes());
    child.wait_with_output().unwrap()
}

/// 道具を置き場の bin の下に向けた終わり（tz は --tz の 2 語・scribe2 は bin の下の名 `scribe2` を --scribe2= の 1 語）。
fn stop(root: &Path, again: bool, scribe2: &str) -> Output {
    let tz = root.join("bin/tz").display().to_string();
    let s2 = format!("--scribe2={}", root.join("bin").join(scribe2).display());
    hook(root, again, &["--tz".to_string(), tz, s2])
}

/// 門の記録の行の、時刻を除いた欄（終わりの回・行 id・型・詳細）。
fn gate_rows(root: &Path) -> Vec<String> {
    fs::read_to_string(root.join("drafts/w300a/w/floor-gate.tsv"))
        .unwrap_or_default()
        .lines()
        .map(|l| {
            let (at, rest) = l.split_once('\t').unwrap_or_default();
            assert!(
                at.len() == 14 && at.ends_with('Z') && at.chars().nth(8) == Some('T'),
                "{l}"
            );
            rest.to_string()
        })
        .collect()
}

/// 札の終えの印。
fn ended(root: &Path) -> Option<u64> {
    Spec::parse(&fs::read_to_string(root.join("drafts/w300a/spec.json")).unwrap())
        .unwrap()
        .ended
}

#[test]
fn agflr_hook_lets_an_agent_without_patches_end_as_before() {
    let root = whole("plain", &["notes.md"], None);
    fake(&root, (0, 0, 0), &[]);
    let out = stop(&root, false, "scribe2");
    assert_eq!(
        (out.status.code(), out.stdout.len(), out.stderr.len()),
        (Some(0), 0, 0)
    );
    assert!(ended(&root).is_some());
    for file in [
        "drafts/w300a/w/floor-gate.tsv",
        "tz.log",
        "scribe2.log",
        "drafts/w300a/floor-state",
    ] {
        assert!(!root.join(file).exists(), "{file}");
    }
}

#[test]
fn agflr_hook_refuses_an_empty_missing_or_twice_given_tool_flag() {
    let root = whole("flags", &["notes.md", "patch/r1.patch"], None);
    for (extra, why) in [
        (vec!["--tz"], "--tz の値が無い"),
        (vec!["--scribe2="], "--scribe2 の値が空"),
        (vec!["--tz", "a", "--tz=b"], "--tz が 2 度ある"),
    ] {
        let extra: Vec<String> = extra.into_iter().map(String::from).collect();
        let out = hook(&root, false, &extra);
        let err = String::from_utf8_lossy(&out.stderr);
        assert_eq!(out.status.code(), Some(1), "{extra:?}");
        assert!(
            err.starts_with(&format!(
                "tz hook agent-stop: {why}\nusage: tz hook agent-stop "
            )),
            "{err}"
        );
    }
    assert!(!root.join("drafts/w300a/w/floor-gate.tsv").exists());
}

#[test]
fn agflr_hook_holds_a_patch_agent_without_a_record_and_runs_nothing() {
    let root = whole("norec", &["notes.md", "patch/r1.patch"], None);
    fake(&root, (0, 0, 0), &[]);
    let out = stop(&root, false, "scribe2");
    assert_eq!((out.status.code(), out.stdout.len()), (Some(2), 0));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.starts_with("係の終える前の門は止める（床の記録 floor.tsv が出力の dir に無い（差の file の行 r1 ごとに 1 行）） 次の一手 = "),
        "{err}"
    );
    assert_eq!(gate_rows(&root), ["1\tr1\tno-record\t-"]);
    assert_eq!(ended(&root), None);
    assert!(!root.join("tz.log").exists() && !root.join("scribe2.log").exists());
}

#[test]
fn agflr_hook_passes_a_record_the_rerun_agrees_with() {
    let root = whole(
        "agree",
        &["notes.md", "patch/r1.patch"],
        Some("# 頭\nr1\t0\t0\t0\t0\t0,0\n"),
    );
    fake(
        &root,
        (0, 0, 0),
        &["contracts: contracts/n.toml:3 contract-table:done-teeth: 行 r0"],
    );
    let out = stop(&root, false, "scribe2");
    assert_eq!(
        (out.status.code(), out.stdout.len(), out.stderr.len()),
        (Some(0), 0, 0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(gate_rows(&root), ["1\tr1\tok\t-"]);
    assert!(ended(&root).is_some());
    let floor = fs::canonicalize(root.join("drafts/try-w300a-fl")).unwrap();
    let state = fs::canonicalize(root.join("drafts/w300a/floor-state")).unwrap();
    assert_eq!(fs::read_dir(&state).unwrap().count(), 0);
    let tz = fs::read_to_string(root.join("tz.log")).unwrap();
    let f = floor.display();
    assert_eq!(
        tz,
        format!("{f} check\n{f}/design-intent derive --dir . --out ../contracts --check\n")
    );
    let mut seen: Vec<String> = fs::read_to_string(root.join("scribe2.log"))
        .unwrap()
        .lines()
        .map(String::from)
        .collect();
    seen.sort();
    let want = [
        format!("{f} contracts check --repo {f} --base main"),
        format!(
            "{f} pipe preflight --state-dir {} --repo {f} --design contracts/n.toml#r1 --bead t3-hub.87",
            state.display()
        ),
    ];
    assert_eq!(seen, want);
}

/// 止める見本の置き場と、その 1 度目の終わりの標準エラーと門の記録の行。
fn held(
    name: &str,
    rec: &str,
    rcs: (i32, i32, i32),
    lines: &[&str],
    scribe2: &str,
) -> (PathBuf, String, Vec<String>) {
    let root = whole(name, &["notes.md", "patch/r1.patch"], Some(rec));
    fake(&root, rcs, lines);
    if name == "nofloor" {
        fs::rename(root.join("drafts/try-w300a-fl"), root.join("drafts/away")).unwrap();
    }
    let out = stop(&root, false, scribe2);
    assert_eq!(
        (out.status.code(), out.stdout.len()),
        (Some(2), 0),
        "{name}"
    );
    assert_eq!(ended(&root), None, "{name}");
    let err = String::from_utf8_lossy(&out.stderr).into_owned();
    let rows = gate_rows(&root);
    (root, err, rows)
}

#[test]
fn agflr_hook_holds_each_kind_with_its_hole() {
    let ok = "r1\t0\t0\t0\t0\t0,0\n";
    let named = ["contracts: contracts/n.toml:9 contract-table:done-teeth-missing: 行 r1"];
    let (_, err, rows) = held("mismatch", ok, (0, 0, 0), &named, "scribe2");
    assert!(
        err.contains("床の記録の行 r1 の 器の表の検査 の rc 0 が撃ち直しの rc 1 と違う"),
        "{err}"
    );
    assert_eq!(rows, ["1\tr1\tmismatch\ttable=0/1"]);
    let (_, err, rows) = held("rc", "r1\t0\t0\t0\t0\t0,1\n", (0, 0, 0), &[], "scribe2");
    assert!(
        err.contains("床の記録の行 r1 の verify の 3 行目の rc が 1（0 でない）"),
        "{err}"
    );
    assert_eq!(rows, ["1\tr1\trc\tverify3=1"]);
    let (_, err, rows) = held("pre", ok, (0, 0, 1), &[], "scribe2");
    assert!(
        err.contains("床の記録の行 r1 の preflight の rc 0 が撃ち直しの rc 1 と違う"),
        "{err}"
    );
    assert_eq!(rows, ["1\tr1\tmismatch\tpreflight=0/1"]);
    let (_, err, rows) = held("tzcheck", ok, (1, 2, 0), &[], "scribe2");
    assert!(
        err.contains("tz check の rc 0 が撃ち直しの rc 1")
            && err.contains("tz derive --check の rc 0 が撃ち直しの rc 2"),
        "{err}"
    );
    assert_eq!(rows, ["1\tr1\tmismatch\tcheck=0/1,derive=0/2"]);
    let (_, err, rows) = held("norow", "r0\t0\t0\t0\t0\t0\n", (0, 0, 0), &[], "scribe2");
    assert!(
        err.contains("床の記録 floor.tsv に行 r1 の行が無い"),
        "{err}"
    );
    assert_eq!(rows, ["1\tr1\tno-row\t-"]);
}

#[test]
fn agflr_hook_names_what_it_cannot_find_and_lets_the_second_stop_go() {
    let ok = "r1\t0\t0\t0\t0\t0,0\n";
    let (root, err, rows) = held("nofloor", ok, (0, 0, 0), &[], "scribe2");
    let floor = root.join("drafts/try-w300a-fl");
    let want = format!(
        "床を撃ち直せない（まだ分からない）: 床の写し {} が dir として無い",
        floor.display()
    );
    assert!(err.contains(&want), "{err}");
    assert_eq!(
        rows,
        [format!(
            "1\tr1\tunknown\t床の写し {} が dir として無い",
            floor.display()
        )]
    );
    let (_, err, rows) = held("notool", ok, (0, 0, 0), &[], "none");
    assert!(
        err.contains("床を撃ち直せない（まだ分からない）: 道具 ")
            && err.contains("bin/none を撃てない"),
        "{err}"
    );
    assert!(
        rows.len() == 1 && rows.iter().all(|r| r.starts_with("1\tr1\tunknown\t道具 ")),
        "{rows:?}"
    );
    let again = stop(&root, true, "scribe2");
    assert_eq!((again.status.code(), again.stdout.len()), (Some(0), 0));
    assert!(ended(&root).is_some());
    assert_eq!(
        gate_rows(&root).get(1).map(String::as_str),
        Some(
            format!(
                "2\tr1\tunknown\t床の写し {} が dir として無い",
                floor.display()
            )
            .as_str()
        )
    );
    let gate = fs::read_to_string(root.join("drafts/w300a/w/STOP-GATE.txt")).unwrap();
    assert_eq!(gate, format!("{want}\n"));
}

/// process group `group` に、終わっていない（zombie でない）process が在るか（/proc の stat の状態と group の欄）。
fn group_alive(group: &str) -> bool {
    fs::read_dir("/proc").unwrap().flatten().any(|e| {
        let stat = fs::read_to_string(e.path().join("stat")).unwrap_or_default();
        let rest: Vec<&str> = stat
            .rsplit_once(')')
            .map(|(_, r)| r.split_whitespace().collect())
            .unwrap_or_default();
        !matches!(rest.first(), None | Some(&"Z" | &"X")) && rest.get(2) == Some(&group)
    })
}

/// 器の表の検査（bin の hang）が自分の pid（group の id）を書いてから子の sleep を待ち続け、tz がその字を待ってから `tail` を撃って
/// 返す置き場で、床の記録の在る 1 度目の終わりを撃ち、門の記録の行と group の id を返す（撃ちの順を固める・nointent は design-intent が無い）。
fn hung(name: &str, tail: &str) -> (Vec<String>, String) {
    let rec = "r1\t0\t0\t0\t0\t0,0\n";
    let root = whole(name, &["notes.md", "patch/r1.patch"], Some(rec));
    fake(&root, (0, 0, 0), &[]);
    if name == "nointent" {
        fs::remove_dir_all(root.join("drafts/try-w300a-fl/design-intent")).unwrap();
    }
    let pid = root.join("table.pid");
    let hang = format!(
        "#!/bin/sh\ncase \"$1\" in\ncontracts) echo $$ > '{0}.w'; mv '{0}.w' '{0}'; sleep 60; exit 1 ;;\nesac\nexit 0\n",
        pid.display()
    );
    let tz = format!(
        "#!/bin/sh\ni=0\nwhile [ ! -s '{}' ] && [ $i -lt 200 ]; do sleep 0.05; i=$((i+1)); done\n{tail}\nexit 0\n",
        pid.display()
    );
    for (tool, body) in [("hang", hang), ("tz", tz)] {
        let path = root.join("bin").join(tool);
        fs::write(&path, body).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    }
    let out = stop(&root, false, "hang");
    assert_eq!(
        (out.status.code(), out.stdout.len()),
        (Some(2), 0),
        "{name}"
    );
    let group = fs::read_to_string(&pid).unwrap().trim().to_string();
    (gate_rows(&root), group)
}

/// group の process が 5 秒の内に居なくなるか（見た後に group へ KILL を送って残りを片付ける）。
fn gone(group: &str) -> bool {
    let gone = (0..100).any(|_| {
        let alive = group_alive(group);
        if alive {
            thread::sleep(Duration::from_millis(50));
        }
        !alive
    });
    let _ = Command::new("kill")
        .args(["-KILL", "--", &format!("-{group}")])
        .status();
    gone
}

#[test]
fn agflr_hook_stops_the_table_group_when_another_rerun_fails() {
    let (rows, group) = hung("nointent", "");
    let left = !gone(&group);
    assert!(
        rows.len() == 1
            && rows
                .iter()
                .all(|r| r.starts_with("1\tr1\tunknown\t道具 ") && r.contains("design-intent")),
        "{rows:?}"
    );
    assert!(
        !left,
        "design-intent が無い周の器の表の検査の group {group} が残る"
    );
    let (rows, group) = hung("signal", "kill -9 $$");
    let left = !gone(&group);
    assert_eq!(rows, ["1\tr1\tunknown\ttz check が signal で止まった"]);
    assert!(
        !left,
        "tz check が signal で止まった周の器の表の検査の group {group} が残る"
    );
}

/// 参照の file の置き場（workspace の根から）。
const REFERENCE: &str = "plugin/skills/agent-discipline/reference.md";

/// 句の名と、句を置く節の番号と、節の塊の中に 1 度だけ在る字。
const CLAUSES: [(&str, u8, &str); 7] = [
    ("record", 7, "出力の dir に床の記録 floor.tsv を置く"),
    (
        "fields",
        7,
        "欄はタブで区切った 6 つ（行 id・preflight の rc・tz check の rc・derive --check の rc・器の表の検査の rc・verify の 2 行目から最後の行の rc を , で繋いだ字）",
    ),
    (
        "table",
        7,
        "行の頭（contracts/<ノート>.toml の [[contract]] の行）を名指す断りが無ければ 0・在れば 1",
    ),
    (
        "branch",
        7,
        "床の写しの行は branch に commit し、main を動かさない",
    ),
    ("gate", 7, "床の写し <置き場>/try-<名>-fl で安い 4 本"),
    ("not-pass", 7, "記録の合格は完成でも審査の合格でもない"),
    (
        "kfold-cap",
        11,
        "verify に命令 kfold-cap を、書く範囲に持つ群と作る群を全部名指した 1 行で置く",
    ),
];

fn reference() -> String {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    fs::read_to_string(root.join(REFERENCE)).expect("参照の file を読む")
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

/// 節の塊の中に 1 度だけ在らない句の名（表の順）。
fn faults(text: &str) -> Vec<&'static str> {
    CLAUSES
        .iter()
        .filter(|(_, n, words)| section(text, *n).matches(words).count() != 1)
        .map(|(name, _, _)| *name)
        .collect()
}

#[test]
fn agflr_reference_holds_the_floor_clauses_once_in_their_sections() {
    assert_eq!(faults(&reference()), Vec::<&str>::new());
}

#[test]
fn agflr_reference_clause_removed_doubled_or_moved_is_named() {
    let text = reference();
    for (name, _, words) in CLAUSES {
        let removed = text.replacen(words, "", 1);
        assert_eq!(faults(&removed), vec![name], "外した {name}");
        let doubled = text.replacen(words, &format!("{words}・{words}"), 1);
        assert_eq!(faults(&doubled), vec![name], "重ねた {name}");
        let moved = format!("- {words}\n{removed}");
        assert_eq!(faults(&moved), vec![name], "頭の塊へ動かした {name}");
    }
}
