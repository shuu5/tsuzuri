//! 問いの起票の門の方針の検査の口の歯（設計ノート surface-wave13b 行 f-premises・要件 FR8・接頭辞 fprem_）。
//! 偽の bd は作業場の ledger.json を、偽の設計の道具は作業場の index.tsv を出す script（どちらも引数を見ない）。
//! 方針の問いは境界の server の policy が書く形（create_write の題・本文・labels と line の notes）で台帳に足す。
//! 境界の歯は JSON を読まないので、台帳と metadata と hook の入力の字を歯の中で字として組み、tz の標準出力の字を、
//! 同じ字から中核の関数で組んだ値の字と比べる。

use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use tsuzuri_boundary::server::policy;
use tsuzuri_contract::graph::NodeKind;
use tsuzuri_contract::ledger::{BeadId, LedgerWrite, QUESTION_LABEL};
use tsuzuri_contract::surface::RulingId;
use tsuzuri_core::gate;
use tsuzuri_core::graph::build::{SCOPE_ALL, TYPED_LINES};
use tsuzuri_core::graph::{Graph, Inputs, build};

const INDEX: &str = "guard/question-4/index.tsv";
const LEDGER: &str = "guard/question-4/ledger.json";

/// 処分の metadata の not-relevant の中身（FR4 の処分の要る 4 つ）。
const NR_FOUR: &str = r#""P-14":"問いは判断の代行に触れない","R-1":"規則の行の値は変えない","FR3":"数え方は変えない","ADR-2":"面の技術の決めは先送りのまま""#;

/// 方針の逐語。
const VERBATIM: &str = "急がない";

/// verify の filter の語（main 671060f の contracts の語を畳んだ 145 語と、並行と後の行の接頭辞 5 語と gpface_）。
const FILTER_WORDS: [&str; 151] = [
    "aaround_",
    "accept_",
    "acchold_",
    "account_",
    "acctcore_",
    "acctdoc_",
    "accthb_",
    "accthome_",
    "acctled_",
    "acctlook_",
    "acctpcore_",
    "acctproj_",
    "acctsess_",
    "acctwin_",
    "acctwire_",
    "afocus_",
    "aord_",
    "apop_",
    "askcard_",
    "athr_",
    "batchpanel_",
    "bhalf_",
    "board_min_",
    "bport_",
    "brand_",
    "btuck_",
    "cadopt_",
    "cdorm_",
    "cgdom_",
    "cmark_",
    "contract_form_",
    "cround_",
    "csled_",
    "denv_",
    "dnkind_",
    "dnrow_",
    "ecache_",
    "epolq_",
    "flight_",
    "fmark_",
    "frame_",
    "fserve_",
    "fstop_",
    "gapspage_",
    "gbnote_",
    "gfix_",
    "gfresh_",
    "ghb_",
    "gjst_",
    "glabel_",
    "gnav_",
    "gpface_",
    "gpill_",
    "gpulse_",
    "graph_",
    "gsum_",
    "gtuck_",
    "gview_",
    "hacols_",
    "harest_",
    "hasplit_",
    "hbconf_",
    "hbmark_",
    "hbpost_",
    "hbproc_",
    "hbroute_",
    "hcard_",
    "hcled_",
    "hcnx_",
    "hcproj_",
    "hcsess_",
    "hdchip_",
    "hfig_",
    "hnunk_",
    "hook_",
    "hruling_",
    "hsblock_",
    "hsderive_",
    "hshist_",
    "hspage_",
    "hsym_",
    "iclose_",
    "ilink_",
    "kcli_",
    "klink_",
    "launch_",
    "lcard_",
    "ledgerblock_",
    "lhome_",
    "lidle_",
    "lsnap_",
    "lspark_",
    "lstore_",
    "mapview_",
    "mkeys_",
    "mlink_",
    "mstore_",
    "mtree_",
    "nact_",
    "nbatch_",
    "ncard_",
    "nextstep_",
    "nodepage_",
    "nstall_",
    "nsum_",
    "nsumw_",
    "ntime_",
    "nxact_",
    "parts_",
    "pclosed_",
    "pfold_",
    "pgz_",
    "pipe_",
    "plimit_",
    "pmisfit_",
    "pmore_",
    "pquest_",
    "pqueue_",
    "project_",
    "ptitle_",
    "punmap_",
    "pwhole_",
    "qblock_",
    "qgate_",
    "qkey_",
    "question_",
    "relay_",
    "rhold_",
    "runsdoc_",
    "rvk_",
    "saxis_",
    "sclosed_",
    "seatblock_",
    "seatcard_",
    "server_",
    "sesplit_",
    "shb_",
    "skeleton_",
    "smore_",
    "stage_",
    "stats_",
    "stcli_",
    "steady_",
    "sxaxis_",
    "ticker_",
    "tipx_",
    "topbar_",
    "tz_",
    "urpanel_",
    "uword_",
    "wstrip_",
];

fn read_fixture(rel: &str) -> String {
    fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures")
            .join(rel),
    )
    .unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn graph_of(index: &str, ledger: &str) -> Graph {
    build(&Inputs {
        design_index: index,
        ledger,
        events: "",
    })
}

fn strings(ids: &[&str]) -> Vec<String> {
    ids.iter().map(|s| s.to_string()).collect()
}

/// JSON の字の中身（二重引用符・逆斜線・改行・復帰に逆斜線を付ける）。
fn escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

/// JSON の字の値。
fn quoted(s: &str) -> String {
    format!("\"{}\"", escape(s))
}

/// fixture の ledger.json の配列に bead の JSON の字を足した台帳の字。
fn ledger_with(extra: &[String]) -> String {
    let base = read_fixture(LEDGER);
    let body = base
        .trim_end()
        .strip_suffix(']')
        .expect("ledger.json は配列");
    let mut out = body.trim_end().to_string();
    for bead in extra {
        out.push_str(",\n");
        out.push_str(bead);
    }
    out.push_str("\n]\n");
    out
}

/// 方針の問い fx-p.1 の方針の id（分は 20260928T1049Z）。
fn policy_id() -> RulingId {
    let q = BeadId::new("fx-p.1").expect("bead の id");
    RulingId::for_question(&q, "20260928T1049Z", 1).expect("方針の id")
}

/// 境界の server の policy が書く形の closed の方針の問い fx-p.1（範囲 `scope`）。
fn policy_question(scope: &str) -> String {
    let root = BeadId::new("fx-q").expect("根の id");
    let LedgerWrite::CreateChild {
        title,
        description,
        labels,
        ..
    } = policy::create_write(&root, scope, VERBATIM)
    else {
        panic!("create_write は子を作る書き");
    };
    assert_eq!(
        labels,
        [QUESTION_LABEL.to_string(), policy::scope_label(scope)]
    );
    let labels: Vec<String> = labels.iter().map(|l| quoted(l)).collect();
    format!(
        r#"{{"id":"fx-p.1","title":{},"description":{},"status":"closed","issue_type":"task","labels":[{}],"notes":{},"parent":"fx-q","dependencies":[{{"issue_id":"fx-p.1","depends_on_id":"fx-q","type":"parent-child"}}]}}"#,
        quoted(&title),
        quoted(&description),
        labels.join(","),
        quoted(&policy::line(&policy_id(), scope, VERBATIM)),
    )
}

/// closed の問い fx-q.3（touches FR3・premises は方針 fx-p.1）。
fn premised_question() -> String {
    format!(
        r#"{{"id":"fx-q.3","title":"先に方針を挙げた問い","status":"closed","issue_type":"task","labels":["intake:question"],"metadata":{{"touches":["FR3"],"premises":{}}},"dependencies":[{{"issue_id":"fx-q.3","depends_on_id":"fx-q","type":"parent-child"}}]}}"#,
        quoted(policy_id().as_str())
    )
}

/// 処分の metadata M の字（digest は `graph` の束の要約値・`premises` は JSON の字の値か無し）。
fn meta_m(graph: &Graph, premises: Option<&str>) -> String {
    let digest = gate::bundle_digest(graph, &strings(&["FR4"]));
    let premises = premises
        .map(|p| format!(r#","premises":{p}"#))
        .unwrap_or_default();
    format!(r#"{{"touches":["FR4"],"not-relevant":{{{NR_FOUR}}},"digest":"{digest}"{premises}}}"#)
}

/// 問いの起票の Bash の PreToolUse の hook の入力の字（metadata の字を一重引用符で囲む）。
fn payload(meta: &str) -> String {
    let command =
        format!("bdw create --parent=fx-q --labels=intake:question --metadata='{meta}' 方針の歯の問い");
    format!(
        r#"{{"session_id":"s-1","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{{"command":{}}}}}"#,
        quoted(&command)
    )
}

/// payload を `graph` で判じた答えの字と改行 1 つ（通すなら空の字）。
fn expected(payload: &str, graph: &Graph) -> String {
    gate::output(&gate::judge(&gate::drafts(payload), graph))
        .map(|t| format!("{t}\n"))
        .unwrap_or_default()
}

fn script(path: &Path, body: &str) {
    fs::write(path, format!("#!/bin/sh\n{body}\n")).expect("偽の program");
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("偽の program の権限");
}

/// 歯ごとの作業場（repo の dir・偽の bd と偽の設計の道具と、それらが出す字）。drop で作業場ごと消す。
struct Place {
    root: PathBuf,
    repo: PathBuf,
}

impl Drop for Place {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

impl Place {
    fn new(name: &str, ledger: &str) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("fpremhook")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        let repo = root.join("repo");
        let place = Place { root, repo };
        fs::create_dir_all(&place.repo).expect("repo の置き場");
        fs::write(place.root.join("index.tsv"), read_fixture(INDEX)).expect("索引の字");
        place.ledger(ledger);
        for (program, out) in [("bd", "ledger.json"), ("folio", "index.tsv")] {
            script(
                &place.root.join(program),
                &format!("exec cat '{}'", place.root.join(out).display()),
            );
        }
        place
    }

    /// 偽の bd が出す台帳の字を替える。
    fn ledger(&self, text: &str) {
        fs::write(self.root.join("ledger.json"), text).expect("台帳の字");
    }

    /// tz hook question-gate を撃ち、標準入力に payload を書いて閉じ、終わりまで待つ。
    fn tz(&self, payload: &str) -> Output {
        let mut child = Command::new(env!("CARGO_BIN_EXE_tz"))
            .arg("hook")
            .arg("question-gate")
            .arg("--repo")
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
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).expect("UTF-8")
}

#[test]
fn fprem_bin_new_policy_refused() {
    let index = read_fixture(INDEX);
    let ledger = ledger_with(&[policy_question(policy::SCOPE_ALL)]);
    let g = graph_of(&index, &ledger);
    let id = policy_id();
    let place = Place::new("new-policy", &ledger);

    let p = payload(&meta_m(&g, None));
    let out = place.tz(&p);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let want = expected(&p, &g);
    assert!(
        want.contains(&format!("（premises） id = {id}")) && want.contains("次の一手 = "),
        "{want}"
    );
    assert_eq!(text(&out.stdout), want);

    let p = payload(&meta_m(&g, Some(&format!("[{}]", quoted(id.as_str())))));
    assert_eq!(expected(&p, &g), "");
    let out = place.tz(&p);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    assert!(out.stdout.is_empty(), "{out:?}");

    let ledger = ledger_with(&[policy_question(policy::SCOPE_ALL), premised_question()]);
    place.ledger(&ledger);
    let g = graph_of(&index, &ledger);
    let p = payload(&meta_m(&g, None));
    assert_eq!(expected(&p, &g), "");
    let out = place.tz(&p);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    assert!(out.stdout.is_empty(), "{out:?}");

    let root = place.root.clone();
    drop(place);
    assert!(fs::symlink_metadata(&root).is_err(), "drop の後に作業場が残る");
}

#[test]
fn fprem_bin_scope_from_line() {
    assert_eq!(SCOPE_ALL, policy::SCOPE_ALL);
    assert!(TYPED_LINES.contains(&(policy::LINE_PREFIX, NodeKind::Policy)));
    let index = read_fixture(INDEX);
    let id = policy_id().to_string();
    for (scope, fr3) in [(policy::SCOPE_ALL, vec![id.clone()]), ("fx-q.1", Vec::new())] {
        let g = graph_of(&index, &ledger_with(&[policy_question(scope)]));
        let got: Vec<(String, String)> = g
            .policies
            .iter()
            .map(|(k, v)| (k.clone(), v.scope.clone()))
            .collect();
        assert_eq!(got, [(id.clone(), scope.to_string())], "{scope}");
        assert_eq!(
            gate::due_policies(&g, &strings(&["FR4"])),
            vec![id.clone()],
            "{scope}"
        );
        assert_eq!(gate::due_policies(&g, &strings(&["FR3"])), fr3, "{scope}");
    }
}

#[test]
fn fprem_bin_own_names_clean() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fpremhook.rs");
    let src = fs::read_to_string(&path).expect("tests/fpremhook.rs を読む");
    let lines: Vec<&str> = src.lines().collect();
    let names: Vec<&str> = lines
        .windows(2)
        .filter(|w| w[0].trim() == "#[test]")
        .map(|w| {
            let rest = w[1].trim().strip_prefix("fn ").expect("test の属性の次は fn");
            rest.split('(').next().unwrap_or(rest)
        })
        .collect();
    assert_eq!(names.len(), 3, "歯の数");
    for name in names {
        assert!(name.starts_with("fprem_"), "{name} は fprem_ で始まらない");
        for word in FILTER_WORDS {
            assert!(!name.contains(word), "{name} が {word} を含む");
        }
    }
}
