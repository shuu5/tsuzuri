//! 問いの起票の門の方針の検査の歯（設計ノート surface-wave13b 行 f-premises・要件 FR8・接頭辞 fprem_）。
//! fixture: tests/fixtures/guard/question-4/ の index.tsv と ledger.json。台帳は ledger.json の配列に歯の中で
//! bead を足した字で、グラフは index.tsv と台帳と空の event log の字で組む。方針の id は fx の接頭辞の問いの形。
#![cfg(test)]

use std::collections::BTreeMap;
use std::path::Path;

use serde_json::{Value, json};
use tsuzuri_contract::graph::NodeKind;
use tsuzuri_core::gate::{self, Draft, Gate, Why};
use tsuzuri_core::graph::build::{PREMISES_KEY, SCOPE_ALL, SCOPE_FIELD, TYPED_LINES};
use tsuzuri_core::graph::{Graph, Inputs, PolicyAttr, build};

const DIR: &str = "tests/fixtures/guard/question-4";

/// 方針の id（今までの memo の 2 つと、方針の問い fx-p.1・fx-p.2・fx-p.3 の 3 つ）。
const OLD: &str = "fx-old:20260927T1034Z-1";
const BARE: &str = "fx-bare:20260927T1035Z-1";
const P1: &str = "fx-p.1:20260928T1049Z-1";
const P2: &str = "fx-p.2:20260928T1050Z-1";
const P3: &str = "fx-p.3:20260928T1051Z-1";

/// 挙げさせる方針（台帳 F・touches FR4）。
const DUE_FR4: [&str; 3] = [BARE, P1, P2];

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

fn read(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(DIR)
        .join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{name} を読む: {e}"))
}

fn strings(ids: &[&str]) -> Vec<String> {
    ids.iter().map(|s| s.to_string()).collect()
}

/// fixture の ledger.json の配列に `extra` を足した台帳の字。
fn ledger_with(extra: &[Value]) -> String {
    let mut beads: Value = serde_json::from_str(&read("ledger.json")).expect("ledger.json は JSON");
    beads
        .as_array_mut()
        .expect("配列")
        .extend(extra.iter().cloned());
    beads.to_string()
}

fn graph_of(index: &str, ledger: &str) -> Graph {
    build(&Inputs {
        design_index: index,
        ledger,
        events: "",
    })
}

/// fixture の index.tsv と、ledger.json に `extra` を足した台帳のグラフ。
fn graph_with(extra: &[Value]) -> Graph {
    graph_of(&read("index.tsv"), &ledger_with(extra))
}

/// 根 fx-q の下の closed の方針の問い（id は `fx-p.<n>`・方針の id は `<id>:<分>-1`）。
fn policy_question(n: u32, minute: &str, scope: &str) -> Value {
    let id = format!("fx-p.{n}");
    json!({
        "id": id,
        "title": format!("方針（範囲 = {scope}）"),
        "status": "closed",
        "issue_type": "task",
        "labels": ["intake:question", format!("policy-scope:{scope}")],
        "notes": format!("方針 id = {id}:{minute}-1・範囲 = {scope}・逐語 = 急がない"),
        "parent": "fx-q",
        "dependencies": [{"issue_id": id, "depends_on_id": "fx-q", "type": "parent-child"}],
    })
}

fn p1() -> Value {
    policy_question(1, "20260928T1049Z", "all")
}

fn p2() -> Value {
    policy_question(2, "20260928T1050Z", "fx-q.1")
}

fn p3() -> Value {
    policy_question(3, "20260928T1051Z", "fx-q.9")
}

/// 今までの方針の memo（範囲の欄を持つ行と、id だけの行）。
fn memo() -> Value {
    json!({
        "id": "fx-m",
        "title": "今までの方針",
        "status": "closed",
        "issue_type": "task",
        "labels": ["intake:memo"],
        "notes": format!("方針 id = {OLD}・範囲 = all・逐語 = 前\n方針 id = {BARE}"),
    })
}

/// 根 fx-q の下の問い（premises が無ければ鍵を持たない）。
fn question(id: &str, status: &str, touches: &[&str], premises: Option<Value>) -> Value {
    let mut meta = json!({"touches": touches});
    if let Some(p) = premises {
        meta[PREMISES_KEY] = p;
    }
    json!({
        "id": id,
        "title": format!("問い {id}"),
        "status": status,
        "issue_type": "task",
        "labels": ["intake:question"],
        "metadata": meta,
        "dependencies": [{"issue_id": id, "depends_on_id": "fx-q", "type": "parent-child"}],
    })
}

/// 台帳 F の足す 6 本。
fn ledger_f() -> Vec<Value> {
    vec![
        p1(),
        p2(),
        p3(),
        memo(),
        question("fx-q.3", "closed", &["FR3"], Some(json!(OLD))),
        question("fx-q.9", "open", &["NFR2"], None),
    ]
}

/// 処分の metadata M（touches FR4・not-relevant の 4 つ・digest は `graph` の束の要約値）。
fn meta_m(graph: &Graph) -> Value {
    json!({
        "touches": ["FR4"],
        "not-relevant": {
            "P-14": "問いは判断の代行に触れない",
            "R-1": "規則の行の値は変えない",
            "FR3": "数え方は変えない",
            "ADR-2": "面の技術の決めは先送りのまま",
        },
        "digest": gate::bundle_digest(graph, &strings(&["FR4"])),
    })
}

/// M の premises を `premises` にした字。
fn with_premises(mut m: Value, premises: Value) -> Value {
    m[PREMISES_KEY] = premises;
    m
}

/// 問いの起票の command（metadata の字を一重引用符で囲む）。
fn command(meta: &str) -> String {
    format!("bdw create --parent=fx-q --labels=intake:question --metadata='{meta}' 方針の歯の問い")
}

/// command を持つ Bash の PreToolUse の hook の入力の字。
fn payload(command: &str) -> String {
    json!({
        "session_id": "s-1",
        "hook_event_name": "PreToolUse",
        "tool_name": "Bash",
        "tool_input": {"command": command},
    })
    .to_string()
}

/// metadata の 1 つの下書きを判じる。
fn judge_meta(meta: &Value, graph: &Graph) -> Gate {
    let text = meta.to_string();
    let drafts = gate::drafts(&payload(&command(&text)));
    assert_eq!(drafts, vec![Draft { metadata: Some(text) }]);
    gate::judge(&drafts, graph)
}

fn deny(why: Why, ids: &[&str], digest: Option<&str>) -> Gate {
    Gate::Deny {
        why,
        ids: strings(ids),
        digest: digest.map(str::to_string),
    }
}

fn scopes(graph: &Graph) -> Vec<(String, String)> {
    graph
        .policies
        .iter()
        .map(|(id, p)| (id.clone(), p.scope.clone()))
        .collect()
}

fn pairs(items: &[(&str, &str)]) -> Vec<(String, String)> {
    items
        .iter()
        .map(|(a, b)| (a.to_string(), b.to_string()))
        .collect()
}

#[test]
fn fprem_scope_attr_from_line() {
    assert_eq!(SCOPE_FIELD, "範囲 = ");
    assert_eq!(SCOPE_ALL, "all");
    assert_eq!(PREMISES_KEY, "premises");
    assert!(TYPED_LINES.contains(&("方針 id = ", NodeKind::Policy)));

    let g = graph_with(&ledger_f());
    assert_eq!(
        scopes(&g),
        pairs(&[
            (BARE, "all"),
            (OLD, "all"),
            (P1, "all"),
            (P2, "fx-q.1"),
            (P3, "fx-q.9"),
        ])
    );
    assert_eq!(g.count_nodes(NodeKind::Policy), 5);
    assert_eq!(
        g.policies.get(P2),
        Some(&PolicyAttr {
            scope: "fx-q.1".to_string()
        })
    );

    let notes = [
        "方針 id = fx-e:20260928T1100Z-1・範囲 = ・逐語 = 空",
        "方針 id = fx-v:20260928T1101Z-1・逐語 = 急がない・範囲 = fx-q.1",
        "裁定 id = fx-r:20260928T1102Z-1・範囲 = fx-q.1",
        "方針 id = fx-d:20260928T1103Z-1・範囲 = fx-q.1・逐語 = 先",
    ]
    .join("\n");
    let odd = json!({
        "id": "fx-n",
        "title": "形の違う行",
        "status": "closed",
        "issue_type": "task",
        "notes": notes,
    });
    let again = json!({
        "id": "fx-n2",
        "title": "同じ方針の 2 つ目の行",
        "status": "open",
        "issue_type": "task",
        "notes": "方針 id = fx-d:20260928T1103Z-1・範囲 = all・逐語 = 後",
    });
    let g = graph_with(&[odd, again]);
    assert_eq!(
        scopes(&g),
        pairs(&[
            ("fx-d:20260928T1103Z-1", "fx-q.1"),
            ("fx-e:20260928T1100Z-1", "all"),
            ("fx-v:20260928T1101Z-1", "all"),
        ])
    );
    assert_eq!(g.count_nodes(NodeKind::Ruling), 1);

    let plain = graph_with(&[]);
    assert_eq!(plain.policies, BTreeMap::new());
}

#[test]
fn fprem_due_by_scope_and_once() {
    let g = graph_with(&ledger_f());
    let due = |touches: &[&str]| gate::due_policies(&g, &strings(touches));
    assert_eq!(due(&["FR4"]), strings(&DUE_FR4));
    assert_eq!(due(&["FR3"]), strings(&[BARE, P1]));
    for touches in [&["NFR2"][..], &["fx-q.9"], &["FR3", "NFR2"]] {
        assert_eq!(due(touches), strings(&[BARE, P1, P3]), "{touches:?}");
    }
    assert_eq!(due(&["fx-q.1"]), strings(&[BARE, P1, P2]));
    assert_eq!(due(&[]), strings(&[BARE, P1]));

    let once = graph_with(&[
        p1(),
        p2(),
        question("fx-q.4", "open", &["FR3"], Some(json!(P1))),
        question("fx-q.5", "closed", &["FR3"], Some(json!([P2]))),
    ]);
    assert!(gate::due_policies(&once, &strings(&["FR4"])).is_empty());
    let plain = graph_with(&[]);
    assert!(gate::due_policies(&plain, &strings(&["FR4"])).is_empty());
}

#[test]
fn fprem_new_policy_then_premises() {
    let plain = graph_with(&[]);
    assert_eq!(judge_meta(&meta_m(&plain), &plain), Gate::Allow);

    let g = graph_with(&[p1()]);
    let m = meta_m(&g);
    assert_eq!(m, meta_m(&plain), "方針は束の要約値を変えない");
    let refused = deny(Why::Premises, &[P1], None);
    assert_eq!(judge_meta(&m, &g), refused);
    for bad in [json!([]), json!(1)] {
        assert_eq!(judge_meta(&with_premises(m.clone(), bad.clone()), &g), refused, "{bad}");
    }
    for good in [json!([P1]), json!(P1)] {
        assert_eq!(
            judge_meta(&with_premises(m.clone(), good.clone()), &g),
            Gate::Allow,
            "{good}"
        );
    }

    let g = graph_with(&[p1(), question("fx-q.3", "closed", &["FR3"], Some(json!(P1)))]);
    let m = meta_m(&g);
    assert_eq!(judge_meta(&m, &g), Gate::Allow);
    assert_eq!(judge_meta(&with_premises(m, json!([P1])), &g), Gate::Allow);
}

#[test]
fn fprem_names_what_is_missing() {
    let g = graph_with(&ledger_f());
    let m = meta_m(&g);
    assert_eq!(judge_meta(&m, &g), deny(Why::Premises, &DUE_FR4, None));
    assert_eq!(
        judge_meta(&with_premises(m.clone(), json!([P1])), &g),
        deny(Why::Premises, &[BARE, P2], None)
    );
    for premises in [
        json!([P2, BARE, P1]),
        json!([BARE, P1, P2, OLD]),
        json!([BARE, P1, P2, P3]),
    ] {
        assert_eq!(
            judge_meta(&with_premises(m.clone(), premises.clone()), &g),
            Gate::Allow,
            "{premises}"
        );
    }
    assert_eq!(
        judge_meta(&with_premises(m.clone(), json!([P1, "FR4", "fx-q.1"])), &g),
        deny(Why::NotPolicy, &["FR4", "fx-q.1"], None)
    );
    let ruling = "fx-q.1:20260924T0200Z-1";
    assert_eq!(
        judge_meta(&with_premises(m, json!(ruling)), &g),
        deny(Why::NotPolicy, &[ruling], None)
    );

    let q9 = json!({
        "touches": ["fx-q.9"],
        "not-relevant": {},
        "digest": gate::bundle_digest(&g, &strings(&["fx-q.9"])),
    });
    assert_eq!(judge_meta(&q9, &g), deny(Why::Premises, &[BARE, P1, P3], None));
    assert_eq!(
        judge_meta(&with_premises(q9, json!([BARE, P1, P3])), &g),
        Gate::Allow
    );
}

#[test]
fn fprem_order_among_reasons() {
    let g = graph_with(&ledger_f());
    let d = gate::bundle_digest(&g, &strings(&["FR4"]));
    let three = json!(DUE_FR4);

    let mut m = meta_m(&g);
    m["touches"] = json!(["FR4", "NOPE-1"]);
    assert_eq!(judge_meta(&m, &g), deny(Why::UnknownId, &["NOPE-1"], None));

    let unread = graph_of(&read("index.tsv"), "");
    assert_eq!(
        judge_meta(&meta_m(&g), &unread),
        Gate::Unknown {
            why: Why::Unread,
            ids: strings(&["ledger"]),
            digest: None,
        }
    );

    let mut m = meta_m(&g);
    m["not-relevant"]
        .as_object_mut()
        .expect("object")
        .remove("ADR-2");
    assert_eq!(judge_meta(&m, &g), deny(Why::Premises, &DUE_FR4, None));
    assert_eq!(
        judge_meta(&with_premises(m, three.clone()), &g),
        deny(Why::Undisposed, &["ADR-2"], Some(&d))
    );

    let mut m = with_premises(meta_m(&g), three.clone());
    m["digest"] = json!("0000000000000000");
    assert_eq!(judge_meta(&m, &g), deny(Why::Stale, &[], Some(&d)));

    let two = format!(
        "{}; {}",
        command(&with_premises(meta_m(&g), three).to_string()),
        command(&meta_m(&g).to_string())
    );
    let drafts = gate::drafts(&payload(&two));
    assert_eq!(drafts.len(), 2);
    assert_eq!(gate::judge(&drafts, &g), deny(Why::Premises, &DUE_FR4, None));
}

/// 答えの permissionDecisionReason の字。
fn reason(gate: &Gate) -> String {
    let text = gate::output(gate).expect("止める答え");
    let v: Value = serde_json::from_str(&text).expect("答えは JSON");
    v["hookSpecificOutput"]["permissionDecisionReason"]
        .as_str()
        .expect("字")
        .to_string()
}

#[test]
fn fprem_answer_words() {
    let words: Vec<&str> = Why::ALL.iter().map(|w| w.word()).collect();
    assert_eq!(
        words,
        [
            "args",
            "metadata",
            "no-touches",
            "unread",
            "unknown-id",
            "not-policy",
            "premises",
            "too-many",
            "undisposed",
            "stale"
        ]
    );
    assert_eq!(
        reason(&deny(Why::Premises, &[BARE, P1], None)),
        "問いの起票の門は止める（premises） id = fx-bare:20260927T1035Z-1 fx-p.1:20260928T1049Z-1 次の一手 = 名指した方針を読み、その id を metadata の premises に足して置き直す。"
    );
    assert_eq!(
        reason(&deny(Why::NotPolicy, &["FR4"], None)),
        "問いの起票の門は止める（not-policy） id = FR4 次の一手 = metadata の premises から方針の id でない字を除いて置き直す。"
    );
}

#[test]
fn fprem_own_names_clean() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/teeth1/fprem.rs");
    let src = std::fs::read_to_string(&path).expect("tests/teeth1/fprem.rs を読む");
    let lines: Vec<&str> = src.lines().collect();
    let names: Vec<&str> = lines
        .windows(2)
        .filter(|w| w[0].trim() == "#[test]")
        .map(|w| {
            let rest = w[1].trim().strip_prefix("fn ").expect("test の属性の次は fn");
            rest.split('(').next().unwrap_or(rest)
        })
        .collect();
    assert_eq!(names.len(), 7, "歯の数");
    for name in names {
        assert!(name.starts_with("fprem_"), "{name} は fprem_ で始まらない");
        for word in FILTER_WORDS {
            assert!(!name.contains(word), "{name} が {word} を含む");
        }
    }
}
