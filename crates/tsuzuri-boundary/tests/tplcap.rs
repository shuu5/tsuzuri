//! 計画の memo の上限の門の歯（接頭辞 tplcap_・設計ノート surface-v4a 行 t-plan-cap・裁定 t3-hub.92.7.2）。
//! 純関数の歯は fixture の字（索引は FR4 の 1 節点、台帳は label memo:plan を持つ bead fx-p.1 から fx-p.5 と fx-p.6・fx-p.7）から
//! 中核の `graph::build` でグラフを組んで `plan_cap` の関数を直に撃つ。口の歯は tz hook question-gate を偽の bd と偽の設計の道具
//! （撃たれた引数を記録の file に 1 行足してから作業場の字を出す script）で撃つ。作業場は CARGO_TARGET_TMPDIR の下の tplcap/<歯の名>。
#![cfg(test)]

use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use tsuzuri_boundary::server::ledger::BD_ARGS;
use tsuzuri_core::graph::{Graph, Inputs, build};
use tsuzuri_core::plan_cap::{self, CAP_ROW, Counts, Open, PlanGate, PlanWhy};

/// 索引の字（節点は id・種類・file・要約値・題のタブ区切り）。
const INDEX: &str = "FR4\t要件\tsrs.yaml\td-FR4\t節点 FR4\n";

/// 規則の表の上限の行（value は 4 本 以下）。
const CAP_RULES: &str = concat!(
    "thresholds:\n",
    "  - {id: R-45, article: \"P-28\", what: \"開いた計画の memo の本数の上限\", value: \"4 本 以下\", kind: \"deny\"}\n",
);

/// 上限の行が無く、行 R-33 の行の写しだけを持つ規則の表。
const R33_RULES: &str = concat!(
    "thresholds:\n",
    "  - {id: R-33, article: \"P-28\", what: \"計画だけの行の数の上限\", value: \"4 行 以下\", kind: \"deny\"}\n",
);

/// 台帳の bead の 1 本の JSON の字。
fn bead(id: &str, status: &str, labels: &[&str]) -> String {
    let labels = labels
        .iter()
        .map(|l| format!("\"{l}\""))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        r#"{{"id":"{id}","title":"題 {id}","status":"{status}","issue_type":"task","labels":[{labels}],"metadata":{{}}}}"#
    )
}

/// 台帳の字（bd の一覧の JSON の配列）。label memo:plan を持つ fx-p.1 から fx-p.4 のうち先の `open` 本が開き、
/// 残りは閉じる。fx-p.5 は同じ label の閉じた物、fx-p.6 は印の無い開いた memo、fx-p.7 は閉じた task。
fn ledger(open: usize) -> String {
    let plan = &["intake:memo", "memo:plan"][..];
    let mut beads: Vec<String> = (1..=4)
        .map(|n| {
            let status = if n <= open { "open" } else { "closed" };
            bead(&format!("fx-p.{n}"), status, plan)
        })
        .collect();
    beads.push(bead("fx-p.5", "closed", plan));
    beads.push(bead("fx-p.6", "open", &["intake:memo"]));
    beads.push(bead("fx-p.7", "closed", &[]));
    format!("[{}]", beads.join(","))
}

fn graph_of(ledger: &str) -> Graph {
    build(&Inputs {
        design_index: INDEX,
        ledger,
        events: "",
    })
}

/// JSON の字の中身（二重引用符と逆斜線に逆斜線を付ける）。
fn escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Bash の PreToolUse の hook の入力の字。
fn payload(command: &str) -> String {
    format!(
        r#"{{"session_id":"s-1","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{{"command":"{}"}}}}"#,
        escape(command)
    )
}

fn words(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

/// command を拾った書きの列にする。
fn picked(command: &str) -> Vec<Open> {
    plan_cap::opens(&payload(command))
}

/// command を台帳 `open` 本の開いた計画の memo と上限 `cap` で判じる。
fn judged(command: &str, open: usize, cap: Option<usize>) -> PlanGate {
    plan_cap::judge(&picked(command), &graph_of(&ledger(open)), cap)
}

fn cap_deny(open: usize, add: usize) -> PlanGate {
    PlanGate::Deny {
        why: PlanWhy::Cap,
        counts: Some(Counts { open, add, cap: 4 }),
    }
}

fn late_deny() -> PlanGate {
    PlanGate::Deny {
        why: PlanWhy::LateLabel,
        counts: None,
    }
}

const UNREAD: PlanGate = PlanGate::Unknown {
    why: PlanWhy::Unread,
};

/// 後からの印の 4 形。
const LATE: [&str; 4] = [
    "bdw update fx-p.6 --add-label memo:plan",
    "bdw update fx-p.6 --set-labels=intake:memo,memo:plan",
    "bd label add fx-p.6 memo:plan",
    "bd tag fx-p.6 memo:plan",
];

#[test]
fn tplcap_cap_reads_the_row() {
    assert_eq!(CAP_ROW, "R-45");
    assert_eq!(plan_cap::cap(CAP_RULES), Ok(4));
    let row = |value: &str| format!("  - {{id: R-45, what: \"上限\", value: \"{value}\", kind: \"deny\"}}\n");
    assert_eq!(plan_cap::cap(&row("0 本 以下")), Ok(0));
    assert_eq!(plan_cap::cap(&format!("thresholds:\n{}", row("12 本 以下"))), Ok(12));
    // 行の無い字・別の行だけの字・行 R-33 の写しだけの字。
    assert!(plan_cap::cap("").is_err());
    assert!(plan_cap::cap("thresholds:\n  - {id: R-44, value: \"4 本 以下\"}\n").is_err());
    assert!(plan_cap::cap("thresholds:\n  - {id: R-450, value: \"4 本 以下\"}\n").is_err());
    assert!(plan_cap::cap(R33_RULES).is_err());
    // 2 本の行。
    assert!(plan_cap::cap(&format!("{}{}", row("4 本 以下"), row("4 本 以下"))).is_err());
    // value が無いか、形が違う。
    assert!(plan_cap::cap("  - {id: R-45, what: \"上限\"}\n").is_err());
    for value in ["4 行 以下", "4 本 未満", "4 本以下", "本 以下", "-1 本 以下", "+4 本 以下", "四 本 以下", "4"] {
        assert!(plan_cap::cap(&row(value)).is_err(), "{value}");
    }
}

#[test]
fn tplcap_opens_pick_plan_writes() {
    let strs = |c: &str| picked(c);
    let marked = "bdw create --labels=intake:memo,memo:plan --metadata='{\"short\":\"計画\"}' 計画の memo";
    assert_eq!(strs(marked), [Open::Marked]);
    assert_eq!(strs("bd create -l memo:plan 題"), [Open::Marked]);
    assert_eq!(strs("FOO=1 /usr/bin/bd create --parent fx-p.1 --label=memo:plan 題"), [Open::Marked]);
    let inherit = [Open::Inherits("fx-p.1".to_string())];
    assert_eq!(strs("bd create --parent fx-p.1 題"), inherit);
    assert_eq!(strs("bdw create --parent=fx-p.1 --labels=intake:memo 題"), inherit);
    assert_eq!(strs("bd create --parent fx-p.1 --no-inherit-labels 題"), []);
    assert_eq!(strs("bd create 題"), []);
    let revive = |args: &[&str]| [Open::Revives(words(args))];
    assert_eq!(strs("bd reopen fx-p.5"), revive(&["reopen", "fx-p.5"]));
    assert_eq!(
        strs("bd update fx-p.5 --status open"),
        revive(&["update", "fx-p.5", "--status", "open"])
    );
    assert_eq!(
        strs("bdw update fx-p.5 --status=in_progress"),
        revive(&["update", "fx-p.5", "--status=in_progress"])
    );
    for late in LATE {
        assert_eq!(strs(late), [Open::Late], "{late}");
    }
    assert_eq!(strs("bdw update fx-p.6 --add-label=intake:memo,memo:plan"), [Open::Late]);
    // 2 つの一続きは command の順に拾う。
    assert_eq!(
        strs("bdw create --labels=memo:plan 題; bd create --parent fx-p.1 題"),
        [Open::Marked, Open::Inherits("fx-p.1".to_string())]
    );
    assert_not_picked(marked);
}

/// 読み・closed への更新・label を外す旗・頭の語が違う一続き・Bash でない道具は数えない（`marked` は印の起票の command）。
fn assert_not_picked(marked: &str) {
    let edit = format!(
        r#"{{"tool_name":"Edit","tool_input":{{"command":"{}"}}}}"#,
        escape(marked)
    );
    for command in [
        "bd list -l memo:plan",
        "bd show fx-p.5",
        "bd update fx-p.5 --status closed",
        "bd update fx-p.6 --remove-label memo:plan",
        "bd update fx-p.6 --notes x",
        "echo bd create --labels=memo:plan",
        "echo bd reopen fx-p.5",
        "ls -la",
    ] {
        assert!(picked(command).is_empty(), "{command}");
    }
    assert!(plan_cap::opens(&edit).is_empty());
    assert!(plan_cap::opens("not json").is_empty());
}

#[test]
fn tplcap_judge_counts_at_the_edge() {
    let create = "bdw create --labels=intake:memo,memo:plan 計画";
    assert_eq!(judged(create, 4, Some(4)), cap_deny(4, 1));
    assert_eq!(judged(create, 3, Some(4)), PlanGate::Allow);
    let two = format!("{create}; {create}");
    assert_eq!(judged(&two, 3, Some(4)), cap_deny(3, 2));
    assert_eq!(judged(&two, 2, Some(4)), PlanGate::Allow);
    // 値は規則の表から読んだ上限で、閉じた bead と印の無い開いた bead は開いた本数に入らない。
    assert_eq!(judged(create, 4, Some(5)), PlanGate::Allow);
    assert_eq!(
        judged(create, 0, Some(0)),
        PlanGate::Deny {
            why: PlanWhy::Cap,
            counts: Some(Counts { open: 0, add: 1, cap: 0 })
        }
    );
    // 印の無い起票は値を読まずに通す。
    assert_eq!(judged("bd create --labels=intake:memo 題", 4, None), PlanGate::Allow);
}

#[test]
fn tplcap_judge_counts_inherited_labels() {
    let under = |parent: &str, flags: &str| format!("bd create --parent {parent} {flags} 題");
    assert_eq!(judged(&under("fx-p.1", ""), 4, Some(4)), cap_deny(4, 1));
    assert_eq!(judged(&under("fx-p.1", "--no-inherit-labels"), 4, Some(4)), PlanGate::Allow);
    assert_eq!(judged(&under("fx-p.6", ""), 4, Some(4)), PlanGate::Allow);
    assert_eq!(judged(&under("fx-p.9", ""), 4, Some(4)), PlanGate::Allow);
    // 印の label を持つ閉じた親も継ぐ（印を持つかで判じる）。
    assert_eq!(judged(&under("fx-p.5", ""), 4, Some(4)), cap_deny(4, 1));
    // 印の起票は 1 本で、親の印と二重には数えない。
    assert_eq!(judged(&under("fx-p.1", "--labels=memo:plan"), 3, Some(4)), PlanGate::Allow);
}

#[test]
fn tplcap_judge_counts_revived_plans() {
    for command in ["bd reopen fx-p.5", "bd update fx-p.5 --status open"] {
        assert_eq!(judged(command, 4, Some(4)), cap_deny(4, 1), "{command}");
        assert_eq!(judged(command, 3, Some(4)), PlanGate::Allow, "{command}");
    }
    // 閉じた task（印が無い）・もう開いている計画の memo・閉じる更新は足さない。
    for command in ["bd reopen fx-p.7", "bd reopen fx-p.1", "bd update fx-p.5 --status closed"] {
        assert_eq!(judged(command, 4, Some(4)), PlanGate::Allow, "{command}");
    }
    // id ごとに 1 本で、同じ id は 1 度。3 本の台帳は fx-p.4 も閉じている。
    assert_eq!(judged("bd reopen fx-p.5 fx-p.4", 3, Some(4)), cap_deny(3, 2));
    assert_eq!(judged("bd reopen fx-p.5; bd reopen fx-p.5", 3, Some(4)), PlanGate::Allow);
}

#[test]
fn tplcap_judge_refuses_late_labels() {
    for late in LATE {
        assert_eq!(judged(late, 0, Some(4)), late_deny(), "{late}");
        assert_eq!(judged(late, 4, None), late_deny(), "{late}");
    }
    // 印の起票と並べても止める。
    let both = "bdw create --labels=memo:plan 題; bd tag fx-p.6 memo:plan";
    assert_eq!(judged(both, 0, Some(4)), late_deny());
    // 印でない label の後からの足しは数えない。
    assert_eq!(judged("bdw update fx-p.6 --add-label other", 0, Some(4)), PlanGate::Allow);
}

#[test]
fn tplcap_judge_unread_is_not_a_pass() {
    let create = "bdw create --labels=intake:memo,memo:plan 計画";
    assert_eq!(judged(create, 0, None), UNREAD);
    assert_eq!(judged("bd create 題", 0, None), PlanGate::Allow);
    // 台帳の字が空のグラフでは、親の印を判じられないので継ぐ起票の候補も数える。
    let empty = graph_of("");
    let judge = |command: &str| plan_cap::judge(&picked(command), &empty, Some(4));
    assert_eq!(judge(create), UNREAD);
    assert_eq!(judge("bd create --parent fx-p.1 題"), UNREAD);
    assert_eq!(judge("bd create --parent fx-p.1 --no-inherit-labels 題"), PlanGate::Allow);
    assert_eq!(judge("bd create 題"), PlanGate::Allow);
}

#[test]
fn tplcap_output_words() {
    let words: Vec<&str> = PlanWhy::ALL.iter().map(|w| w.word()).collect();
    assert_eq!(words, ["plan-cap", "plan-late-label", "plan-cap-unread"]);
    assert_eq!(plan_cap::output(&PlanGate::Allow), None);
    let create = "bdw create --labels=intake:memo,memo:plan 計画";
    let deny = plan_cap::output(&judged(create, 4, Some(4))).expect("止める答え");
    assert!(deny.contains(r#""permissionDecision":"deny""#), "{deny}");
    assert!(deny.contains(r#""hookEventName":"PreToolUse""#), "{deny}");
    assert!(
        deny.contains(r#""permissionDecisionReason":"計画の memo の上限の門は止める（plan-cap） 開いた計画の memo 4・足す 1・上限 4 次の一手 = "#),
        "{deny}"
    );
    assert!(deny.contains("契約の bead へ移してから置き直す"), "{deny}");
    let late = plan_cap::output(&judged(LATE[2], 0, Some(4))).expect("止める答え");
    assert!(
        late.contains(r#""permissionDecisionReason":"計画の memo の上限の門は止める（plan-late-label） 次の一手 = "#),
        "{late}"
    );
    assert!(late.contains("印 memo:plan は起票の時に label で置き、後から足さない"), "{late}");
    let unknown = plan_cap::output(&judged(create, 0, None)).expect("まだ分からない答え");
    assert!(
        unknown.contains(r#""permissionDecisionReason":"計画の memo の上限の門は まだ分からない（plan-cap-unread） 次の一手 = "#),
        "{unknown}"
    );
    assert!(unknown.contains("規則の表の上限の行と台帳が読めるようになってから置き直す"), "{unknown}");
    assert!(!unknown.contains("開いた計画の memo"), "{unknown}");
}

/// drop で path を消す守り（dir なら中身ごと・file なら file を・誤りは捨てる）。
struct Tidy(PathBuf);

impl Drop for Tidy {
    fn drop(&mut self) {
        let _ = match fs::symlink_metadata(&self.0) {
            Ok(meta) if meta.is_dir() => fs::remove_dir_all(&self.0),
            _ => fs::remove_file(&self.0),
        };
    }
}

/// 歯ごとの作業場（.git が file の repo・設計文書の dir の rules.yaml・記録の置き場・偽の bd と偽の設計の道具）。
struct Place {
    root: PathBuf,
    repo: PathBuf,
    log: PathBuf,
    _git: Tidy,
}

impl Place {
    /// 規則の表の字 `rules` と、開いた計画の memo `open` 本の台帳の偽の bd を持つ作業場。
    fn new(name: &str, rules: &str, open: usize) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("tplcap")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        let (repo, log) = (root.join("repo"), root.join("log"));
        let design = repo.join("design-intent");
        fs::create_dir_all(&design).expect("設計文書の dir");
        fs::create_dir_all(&log).expect("記録の置き場");
        fs::write(design.join("rules.yaml"), rules).expect("規則の表");
        let git = repo.join(".git");
        let tidy = Tidy(git.clone());
        fs::write(&git, "gitdir: /nonexistent/tplcap\n").expect(".git の file");
        fs::write(root.join("ledger.json"), ledger(open)).expect("台帳の写し");
        fs::write(root.join("index.tsv"), INDEX).expect("索引の写し");
        for (program, out) in [("bd", "ledger.json"), ("folio", "index.tsv")] {
            let path = root.join(program);
            let body = format!(
                "#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{}'\nexec cat '{}'\n",
                log.join(format!("{program}.log")).display(),
                root.join(out).display()
            );
            fs::write(&path, body).expect("偽の program");
            fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).expect("権限");
        }
        Place {
            root,
            repo,
            log,
            _git: tidy,
        }
    }

    /// 偽の program が撃たれた回ごとの引数の行。
    fn calls(&self, program: &str) -> Vec<String> {
        fs::read_to_string(self.log.join(format!("{program}.log")))
            .map(|t| t.lines().map(str::to_string).collect())
            .unwrap_or_default()
    }

    /// tz hook question-gate を偽の program で撃ち、標準入力に payload を書いて閉じ、終わりまで待つ。
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
            .expect("tz を撃つ");
        let mut stdin = child.stdin.take().expect("標準入力");
        let _ = stdin.write_all(payload.as_bytes());
        drop(stdin);
        child.wait_with_output().expect("tz の終わり")
    }

    /// repo の中の file の path と byte の一覧（書かれていないことを比べる）。
    fn tree(&self) -> Vec<(PathBuf, Vec<u8>)> {
        let mut out = Vec::new();
        let mut stack = vec![self.repo.clone()];
        while let Some(dir) = stack.pop() {
            for entry in fs::read_dir(&dir).expect("dir を読む") {
                let path = entry.expect("entry").path();
                if path.is_dir() {
                    out.push((path.clone(), Vec::new()));
                    stack.push(path);
                } else {
                    out.push((path.clone(), fs::read(&path).expect("file を読む")));
                }
            }
        }
        out.sort();
        out
    }
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).expect("UTF-8")
}

/// 短い題の門を通す metadata を持つ印の起票の command。
const MARKED: &str =
    "bdw create --labels=intake:memo,memo:plan --metadata='{\"short\":\"計画\"}' 計画の memo";

#[test]
fn tplcap_bin_plan_create_is_counted() {
    let place = Place::new("counted", CAP_RULES, 4);
    let before = place.tree();
    let p = payload(MARKED);
    let out = place.tz(&p);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let want = plan_cap::output(&plan_cap::judge(
        &plan_cap::opens(&p),
        &graph_of(&ledger(4)),
        Some(4),
    ))
    .map(|t| format!("{t}\n"))
    .expect("止める答え");
    assert!(want.contains("plan-cap") && want.contains("開いた計画の memo 4・足す 1・上限 4"), "{want}");
    assert_eq!(text(&out.stdout), want);
    assert_eq!(place.calls("bd"), [BD_ARGS.join(" ")]);
    assert!(before == place.tree(), "repo の byte が変わる");

    // 開いた計画の memo が 3 本の台帳では同じ起票が通る（標準出力は空）。
    let place = Place::new("counted-pass", CAP_RULES, 3);
    let pass = payload("bdw create --labels=memo:plan --metadata='{\"short\":\"計画\"}' 計画の memo");
    let out = place.tz(&pass);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    assert!(out.stdout.is_empty(), "{out:?}");
    assert_eq!(place.calls("bd").len(), 1);
}

#[test]
fn tplcap_bin_without_the_row() {
    let place = Place::new("without", R33_RULES, 4);
    let before = place.tree();
    let out = place.tz(&payload(MARKED));
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let want = plan_cap::output(&PlanGate::Unknown {
        why: PlanWhy::Unread,
    })
    .map(|t| format!("{t}\n"))
    .expect("まだ分からない答え");
    assert!(want.contains("まだ分からない（plan-cap-unread）"), "{want}");
    assert_eq!(text(&out.stdout), want);

    // 印も --parent も無い create は替わらず通し、偽の bd も撃たない。
    let plain = payload("bd create --metadata='{\"short\":\"計画\"}' 計画の memo");
    let place2 = Place::new("without-plain", R33_RULES, 4);
    let out = place2.tz(&plain);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    assert!(out.stdout.is_empty(), "{out:?}");
    assert!(place2.calls("bd").is_empty(), "偽の bd を撃つ");
    assert!(before == place.tree(), "repo の byte が変わる");
}
