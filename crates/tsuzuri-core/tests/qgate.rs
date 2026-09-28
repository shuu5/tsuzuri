//! 問いの起票の門の歯（行 f-gate・要件 FR4・受入 AC6・接頭辞 qgate_）。
//! fixture: tests/fixtures/guard/question-4/ の index.tsv（設計の索引の形）・ledger.json（bd の一覧の形）・
//! cases.json（4 つの組）。グラフは index.tsv と ledger.json と空の event log の字で組む。

use std::path::Path;

use serde_json::{Value, json};
use tsuzuri_contract::graph::NodeKind;
use tsuzuri_contract::ledger::fnv1a64;
use tsuzuri_core::gate::{
    self, DIGEST, Draft, Gate, HUB_DEGREE, KINDS, NEEDS_CAP, NOT_RELEVANT, Why,
};
use tsuzuri_core::graph::{Graph, Inputs, build};

const DIR: &str = "tests/fixtures/guard/question-4";

/// 行 f-gate の外で決めた filter の語（この行の接頭辞 qgate_ は並べない）。
const FILTER_WORDS: [&str; 113] = [
    "accept_",
    "account_",
    "acctcore_",
    "acctdoc_",
    "acctframe_",
    "accthb_",
    "accthome_",
    "acctled_",
    "acctlook_",
    "acctpcore_",
    "acctproj_",
    "acctsess_",
    "acctwin_",
    "acctwire_",
    "askcard_",
    "batchpanel_",
    "board_min_",
    "contract_form_",
    "frame_",
    "gapspage_",
    "gquestion_",
    "graph_",
    "gview_",
    "hbconf_",
    "hbpost_",
    "hbproc_",
    "hbroute_",
    "hook_",
    "hsblock_",
    "hsderive_",
    "hspage_",
    "ledgerblock_",
    "mapgraph_",
    "mapview_",
    "nextstep_",
    "nodepage_",
    "parts_",
    "pipe_",
    "project_",
    "question_",
    "seatblock_",
    "seatcard_",
    "server_",
    "skeleton_",
    "stage_",
    "stats_",
    "steady_",
    "topbar_",
    "tz_",
    "mlink_",
    "gnav_",
    "mkeys_",
    "klink_",
    "ilink_",
    "afocus_",
    "nxact_",
    "urpanel_",
    "saxis_",
    "ticker_",
    "hfig_",
    "pmore_",
    "lspark_",
    "apop_",
    "brand_",
    "runsdoc_",
    "nbatch_",
    "kcli_",
    "nsum_",
    "hcard_",
    "ntime_",
    "ptitle_",
    "ncard_",
    "hsym_",
    "smore_",
    "lcard_",
    "aaround_",
    "hruling_",
    "sxaxis_",
    "plimit_",
    "bport_",
    "fstop_",
    "nsumw_",
    "fmark_",
    "fserve_",
    "cadopt_",
    "tipx_",
    "hcsess_",
    "hcproj_",
    "sesplit_",
    "mstore_",
    "athr_",
    "qblock_",
    "wsteady_",
    "gsum_",
    "nstall_",
    "pfold_",
    "uword_",
    "cround_",
    "cgdom_",
    "csled_",
    "lhome_",
    "shb_",
    "ghb_",
    "nact_",
    "aord_",
    "mtree_",
    "rhold_",
    "wstrip_",
    "flight_",
    "qkey_",
    "stage_term_",
    "stage_cdp_",
    "pwhole_",
];

type Segments = fn(&str) -> Vec<Vec<String>>;
type Drafts = fn(&str) -> Vec<Draft>;
type Needs = fn(&Graph, &[String]) -> Vec<String>;
type BundleDigest = fn(&Graph, &[String]) -> String;
type Judge = fn(&[Draft], &Graph) -> Gate;
type Output = fn(&Gate) -> Option<String>;

const SEGMENTS: Segments = gate::segments;
const DRAFTS: Drafts = gate::drafts;
const NEEDS: Needs = gate::needs;
const BUNDLE_DIGEST: BundleDigest = gate::bundle_digest;
const JUDGE: Judge = gate::judge;
const OUTPUT: Output = gate::output;

/// done (3) で除く辺の行。
const CUT_EDGE: &str = "ADR-7\tR-30\trules\n";

fn read(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(DIR)
        .join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{name} を読む: {e}"))
}

fn graph_of(index: &str, ledger: &str) -> Graph {
    build(&Inputs {
        design_index: index,
        ledger,
        events: "",
    })
}

fn fixture_graph() -> Graph {
    graph_of(&read("index.tsv"), &read("ledger.json"))
}

/// index.tsv から ADR-7 と R-30 の rules の辺の行だけを除いた字。
fn cut_index() -> String {
    let index = read("index.tsv");
    assert_eq!(index.matches(CUT_EDGE).count(), 1, "除く行は 1 つ");
    index.replacen(CUT_EDGE, "", 1)
}

fn strings(ids: &[&str]) -> Vec<String> {
    ids.iter().map(|s| s.to_string()).collect()
}

/// R-a から R-b の id。
fn rules(a: u32, b: u32) -> Vec<String> {
    (a..=b).map(|n| format!("R-{n}")).collect()
}

fn cases() -> Vec<Value> {
    let v: Value = serde_json::from_str(&read("cases.json")).expect("cases.json は JSON");
    v["cases"].as_array().expect("cases は配列").clone()
}

fn case(name: &str) -> Value {
    cases()
        .into_iter()
        .find(|c| c["name"] == name)
        .unwrap_or_else(|| panic!("組 {name} が無い"))
}

fn case_touches(c: &Value) -> Vec<String> {
    c["touches"]
        .as_array()
        .expect("touches は配列")
        .iter()
        .map(|t| t.as_str().expect("touches の id は字").to_string())
        .collect()
}

/// 組の metadata の object（digest の current は `graph` の束の要約値に替える）。
fn case_meta(c: &Value, graph: &Graph) -> Value {
    let digest = match c["digest"].as_str().expect("digest は字") {
        "current" => BUNDLE_DIGEST(graph, &case_touches(c)),
        other => other.to_string(),
    };
    json!({
        "touches": c["touches"],
        "not-relevant": c["not-relevant"],
        "digest": digest,
    })
}

/// 問いの起票の command（metadata の字を一重引用符で囲む）。
fn command(meta: &str) -> String {
    format!("bdw create --parent=fx-q --labels=intake:question --metadata='{meta}' 門の歯の問い")
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

/// metadata の字 1 つの下書きを判じる（下書きは 1 つ拾える）。
fn judge_meta(meta: &str, graph: &Graph) -> Gate {
    let drafts = DRAFTS(&payload(&command(meta)));
    assert_eq!(drafts.len(), 1, "{meta}");
    assert_eq!(drafts[0].metadata.as_deref(), Some(meta));
    JUDGE(&drafts, graph)
}

fn deny(why: Why, ids: &[&str], digest: Option<&str>) -> Gate {
    Gate::Deny {
        why,
        ids: strings(ids),
        digest: digest.map(str::to_string),
    }
}

#[test]
fn qgate_pure_module() {
    assert_eq!(HUB_DEGREE, 30);
    assert_eq!(NEEDS_CAP, 30);
    assert_eq!(
        KINDS,
        [NodeKind::Article, NodeKind::Rule, NodeKind::Adr, NodeKind::Req]
    );
    assert_eq!(NOT_RELEVANT, "not-relevant");
    assert_eq!(DIGEST, "digest");
    let g = Graph::default();
    assert!(SEGMENTS("").is_empty());
    assert!(DRAFTS("").is_empty());
    assert!(NEEDS(&g, &[]).is_empty());
    assert_eq!(BUNDLE_DIGEST(&g, &[]).len(), 16);
    assert_eq!(JUDGE(&[], &g), Gate::Allow);
    assert_eq!(OUTPUT(&Gate::Allow), None);
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/gate.rs");
    let src = std::fs::read_to_string(&path).expect("src/gate.rs を読む");
    for word in [
        "std::fs",
        "std::process",
        "std::net",
        "SystemTime",
        "Instant",
    ] {
        assert!(!src.contains(word), "src/gate.rs に {word}");
    }
}

#[test]
fn qgate_fixture_four_cases() {
    let g = fixture_graph();
    assert!(g.unread.iter().all(|s| *s == tsuzuri_core::graph::Source::Runs));
    let all = cases();
    assert_eq!(all.len(), 4);
    for c in &all {
        let name = c["name"].as_str().expect("name");
        let meta = case_meta(c, &g).to_string();
        let got = judge_meta(&meta, &g);
        let current = BUNDLE_DIGEST(&g, &case_touches(c));
        let want_ids: Vec<String> = c["ids"]
            .as_array()
            .expect("ids")
            .iter()
            .map(|v| v.as_str().expect("id").to_string())
            .collect();
        let (kind, why, ids, digest) = match &got {
            Gate::Allow => ("allow", None, Vec::new(), None),
            Gate::Deny { why, ids, digest } => ("deny", Some(why.word()), ids.clone(), digest.clone()),
            Gate::Unknown { why, ids, digest } => {
                ("unknown", Some(why.word()), ids.clone(), digest.clone())
            }
        };
        assert_eq!(kind, c["want"], "{name}: {got:?}");
        assert_eq!(why, c["why"].as_str(), "{name}: {got:?}");
        assert_eq!(ids, want_ids, "{name}: {got:?}");
        if kind != "allow" {
            assert_eq!(digest.as_deref(), Some(current.as_str()), "{name}");
        }
        for hidden in ["ADR-7", "AC6", "GOAL2", "NFR2", "fx-q.1"] {
            assert!(!ids.iter().any(|i| i == hidden), "{name} が {hidden} を名指す");
        }
    }
    let one = case("one-undisposed");
    let d = BUNDLE_DIGEST(&g, &case_touches(&one));
    assert_eq!(
        judge_meta(&case_meta(&one, &g).to_string(), &g),
        deny(Why::Undisposed, &["ADR-2"], Some(&d))
    );
    let all_disposed = case("all-disposed");
    assert_eq!(
        judge_meta(&case_meta(&all_disposed, &g).to_string(), &g),
        Gate::Allow
    );
    let stale = case("stale");
    assert_eq!(
        judge_meta(&case_meta(&stale, &g).to_string(), &g),
        deny(Why::Stale, &[], Some(&d))
    );
    let many = case("too-many");
    assert_eq!(
        judge_meta(&case_meta(&many, &g).to_string(), &g),
        Gate::Unknown {
            why: Why::TooMany,
            ids: rules(1, 30),
            digest: Some(BUNDLE_DIGEST(&g, &strings(&["ADR-7"]))),
        }
    );
}

#[test]
fn qgate_hub_and_cap_edges() {
    let g = fixture_graph();
    assert_eq!(g.degrees().get("ADR-7"), Some(&31));
    assert_eq!(
        NEEDS(&g, &strings(&["FR4"])),
        strings(&["ADR-2", "FR3", "P-14", "R-1"])
    );
    let mut want = strings(&["FR4"]);
    want.extend(rules(1, 30));
    assert_eq!(NEEDS(&g, &strings(&["ADR-7"])), want);

    let cut = graph_of(&cut_index(), &read("ledger.json"));
    assert_eq!(cut.degrees().get("ADR-7"), Some(&30));
    assert_eq!(
        NEEDS(&cut, &strings(&["FR4"])),
        strings(&["ADR-2", "ADR-7", "FR3", "P-14", "R-1"])
    );
    let one = case("one-undisposed");
    let d = BUNDLE_DIGEST(&cut, &case_touches(&one));
    assert_eq!(
        judge_meta(&case_meta(&one, &cut).to_string(), &cut),
        deny(Why::Undisposed, &["ADR-2", "ADR-7"], Some(&d))
    );
    let many = case("too-many");
    assert_eq!(
        judge_meta(&case_meta(&many, &cut).to_string(), &cut),
        Gate::Deny {
            why: Why::Undisposed,
            ids: rules(1, 29),
            digest: Some(BUNDLE_DIGEST(&cut, &strings(&["ADR-7"]))),
        }
    );
}

/// 組 all-disposed の metadata の object（digest は `graph` の束の要約値）。
fn base(graph: &Graph) -> Value {
    case_meta(&case("all-disposed"), graph)
}

#[test]
fn qgate_reasons_in_order() {
    let g = fixture_graph();
    let d = BUNDLE_DIGEST(&g, &strings(&["FR4"]));

    // --metadata の無い下書き・空の object・touches が空の配列。
    let bare = DRAFTS(&payload(
        "bdw create --parent=fx-q --labels=intake:question 門の歯の問い",
    ));
    assert_eq!(bare, vec![Draft { metadata: None }]);
    assert_eq!(JUDGE(&bare, &g), deny(Why::NoTouches, &[], None));
    assert_eq!(judge_meta("{}", &g), deny(Why::NoTouches, &[], None));
    let mut m = base(&g);
    m["touches"] = json!([]);
    assert_eq!(judge_meta(&m.to_string(), &g), deny(Why::NoTouches, &[], None));

    // JSON の object として読めない metadata。
    for text in ["@q.json", "[1]", ""] {
        assert_eq!(judge_meta(text, &g), deny(Why::Metadata, &[], None), "{text:?}");
    }

    // グラフに無い id。
    let mut m = base(&g);
    m["touches"] = json!(["FR4", "NOPE-1"]);
    assert_eq!(
        judge_meta(&m.to_string(), &g),
        deny(Why::UnknownId, &["NOPE-1"], None)
    );

    // 読めない出所。
    let index = read("index.tsv");
    let ledger = read("ledger.json");
    let meta = base(&g).to_string();
    for (idx, led, ids) in [
        (index.as_str(), "", &["ledger"][..]),
        ("", ledger.as_str(), &["design"][..]),
        ("", "", &["design", "ledger"][..]),
    ] {
        assert_eq!(
            judge_meta(&meta, &graph_of(idx, led)),
            Gate::Unknown {
                why: Why::Unread,
                ids: strings(ids),
                digest: None,
            },
            "{ids:?}"
        );
    }
    assert_eq!(
        JUDGE(&bare, &graph_of(&index, "")),
        deny(Why::NoTouches, &[], None)
    );

    // 処分にならない not-relevant の値。
    for v in [json!(""), json!("  "), json!(1)] {
        let mut m = base(&g);
        m[NOT_RELEVANT]["ADR-2"] = v.clone();
        assert_eq!(
            judge_meta(&m.to_string(), &g),
            deny(Why::Undisposed, &["ADR-2"], Some(&d)),
            "{v}"
        );
    }
    let mut m = base(&g);
    m[NOT_RELEVANT] = json!(["P-14", "R-1", "FR3", "ADR-2"]);
    assert_eq!(
        judge_meta(&m.to_string(), &g),
        deny(Why::Undisposed, &["ADR-2", "FR3", "P-14", "R-1"], Some(&d))
    );

    // digest の鍵が無い・値が数。
    let mut m = base(&g);
    m.as_object_mut().expect("object").remove(DIGEST);
    assert_eq!(judge_meta(&m.to_string(), &g), deny(Why::Stale, &[], Some(&d)));
    let mut m = base(&g);
    m[DIGEST] = json!(1);
    assert_eq!(judge_meta(&m.to_string(), &g), deny(Why::Stale, &[], Some(&d)));

    // touches が字 1 つ。
    let mut m = base(&g);
    m["touches"] = json!("FR4");
    assert_eq!(judge_meta(&m.to_string(), &g), Gate::Allow);

    // 1 つの command の 2 つの下書き。
    let one = case_meta(&case("one-undisposed"), &g).to_string();
    let two = format!("{}; {}", command(&base(&g).to_string()), command(&one));
    let drafts = DRAFTS(&payload(&two));
    assert_eq!(drafts.len(), 2);
    assert_eq!(
        JUDGE(&drafts, &g),
        deny(Why::Undisposed, &["ADR-2"], Some(&d))
    );
    let two = format!("{}; {}", command("{}"), command(&one));
    let drafts = DRAFTS(&payload(&two));
    assert_eq!(drafts.len(), 2);
    assert_eq!(JUDGE(&drafts, &g), deny(Why::NoTouches, &[], None));
    assert_eq!(JUDGE(&[], &g), Gate::Allow);
}

#[test]
fn qgate_digest_definition() {
    let g = fixture_graph();
    let fr4 = strings(&["FR4"]);
    let got = BUNDLE_DIGEST(&g, &fr4);
    assert_eq!(got.len(), 16);
    assert!(
        got.chars()
            .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)),
        "{got}"
    );
    let mut bundle = NEEDS(&g, &fr4);
    bundle.push("FR4".to_string());
    bundle.sort_by(|a, b| tsuzuri_contract::graph::natural_cmp(a, b));
    assert_eq!(bundle, strings(&["ADR-2", "FR3", "FR4", "P-14", "R-1"]));
    let lines = [
        "ADR-2\taaaa0202\t\t面の技術を今決めない",
        "FR3\taaaa0003\t\t不変条件の数え方",
        "FR4\taaaa0004\t\t問いの起票の門",
        "P-14\taaaa0014\t\t判断を代行しない",
        "R-1\tbbbb0001\t\t規則の行 1",
    ];
    let want = format!("{:016x}", fnv1a64(lines.join("\n").as_bytes()));
    assert_eq!(got, want);

    let index = read("index.tsv");
    let ledger = read("ledger.json");
    let once = |from: &str, to: &str| {
        assert_eq!(index.matches(from).count(), 1, "{from}");
        index.replacen(from, to, 1)
    };
    let changed = [
        once("\t面の技術を今決めない", "\t面の技術を今は決めない"),
        once("\taaaa0202\t", "\taaaa9202\t"),
        once("FR4\tR-1\trules\n", "FR4\tR-1\trules\nFR4\tR-2\trules\n"),
    ];
    for idx in &changed {
        assert_ne!(BUNDLE_DIGEST(&graph_of(idx, &ledger), &fr4), got, "{idx}");
    }
    let same = [
        once("\t面の模型", "\t面の模型の改め"),
        once("\t問いの起票 4 通り", "\t問いの起票の 4 通り"),
    ];
    for idx in &same {
        assert_eq!(BUNDLE_DIGEST(&graph_of(idx, &ledger), &fr4), got, "{idx}");
    }
    let mut beads: Value = serde_json::from_str(&ledger).expect("ledger.json は JSON");
    let q = beads
        .as_array_mut()
        .expect("配列")
        .iter_mut()
        .find(|b| b["id"] == "fx-q.1")
        .expect("fx-q.1");
    q["status"] = json!("closed");
    assert_eq!(
        BUNDLE_DIGEST(&graph_of(&index, &beads.to_string()), &fr4),
        got
    );
}

#[test]
fn qgate_segments_and_drafts() {
    let seg = |words: &[&[&str]]| -> Vec<Vec<String>> { words.iter().map(|w| strings(w)).collect() };
    assert_eq!(
        SEGMENTS(r#"a 'b c' "d\"e" f\ g;h|i&&j"#),
        seg(&[&["a", "b c", "d\"e", "f g"], &["h"], &["i"], &["j"]])
    );
    assert_eq!(SEGMENTS("x ''"), seg(&[&["x", ""]]));
    assert_eq!(SEGMENTS("p 'q r"), seg(&[&["p", "q r"]]));

    let g = fixture_graph();
    let j = case_meta(&case("one-undisposed"), &g).to_string();
    let a = format!("bdw create --parent=fx-q --labels=intake:question --metadata='{j}' 門の歯の問い");
    let e = "bdw create --labels intake:question --metadata=a --metadata=b";
    let some = |s: &str| Draft {
        metadata: Some(s.to_string()),
    };
    let forms: Vec<(String, Vec<Draft>)> = vec![
        (a.clone(), vec![some(&j)]),
        (
            format!("/usr/local/bin/bdw create --parent fx-q -l intake:question --metadata '{j}'"),
            vec![some(&j)],
        ),
        (
            format!(
                "BEADS_ACTOR=seat bd create --label a,intake:question --metadata=\"{}\"",
                j.replace('"', "\\\"")
            ),
            vec![some(&j)],
        ),
        (
            "bdw create -lintake:question --parent fx-q 門の歯の問い".to_string(),
            vec![Draft { metadata: None }],
        ),
        (e.to_string(), vec![some("b")]),
        (format!("cd /r && {a}; {e}"), vec![some(&j), some("b")]),
    ];
    for (cmd, want) in &forms {
        assert_eq!(&DRAFTS(&payload(cmd)), want, "{cmd}");
    }

    let memo = format!("bdw create --parent=fx-q --labels=intake:memo --metadata='{j}' 門の歯の問い");
    let empties = [
        json!({"tool_name": "Edit", "tool_input": {"command": a}}).to_string(),
        "not json".to_string(),
        payload("ls -la"),
        payload(&memo),
        payload("bdw update fx-q.1 --add-label=intake:question"),
        payload("echo bdw create --labels=intake:question"),
        json!({"tool_name": "Bash", "tool_input": {"command": 1}}).to_string(),
    ];
    for p in &empties {
        assert!(DRAFTS(p).is_empty(), "{p}");
    }
}

#[test]
fn qgate_output_shape() {
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
    assert_eq!(OUTPUT(&Gate::Allow), None);
    let digest = "0123456789abcdef";
    for why in Why::ALL {
        for ids in [Vec::new(), strings(&["ADR-2", "R-1"])] {
            for d in [None, Some(digest.to_string())] {
                let gates = [
                    (
                        "問いの起票の門は止める（",
                        Gate::Deny {
                            why,
                            ids: ids.clone(),
                            digest: d.clone(),
                        },
                    ),
                    (
                        "問いの起票の門は まだ分からない（",
                        Gate::Unknown {
                            why,
                            ids: ids.clone(),
                            digest: d.clone(),
                        },
                    ),
                ];
                for (head, gate) in gates {
                    let text = OUTPUT(&gate).unwrap_or_else(|| panic!("{gate:?} の答え"));
                    let v: Value = serde_json::from_str(&text).expect("答えは JSON");
                    let o = v.as_object().expect("object");
                    assert_eq!(o.keys().collect::<Vec<_>>(), ["hookSpecificOutput"]);
                    let inner = o["hookSpecificOutput"].as_object().expect("object");
                    let mut keys: Vec<&str> = inner.keys().map(String::as_str).collect();
                    keys.sort_unstable();
                    assert_eq!(
                        keys,
                        ["hookEventName", "permissionDecision", "permissionDecisionReason"]
                    );
                    assert_eq!(inner["hookEventName"], "PreToolUse");
                    assert_eq!(inner["permissionDecision"], "deny");
                    let reason = inner["permissionDecisionReason"].as_str().expect("字");
                    let mut want = format!("{head}{}）", why.word());
                    if !ids.is_empty() {
                        want.push_str(" id =");
                        for id in &ids {
                            want.push(' ');
                            want.push_str(id);
                        }
                    }
                    if let Some(d) = &d {
                        want.push_str(" 要約値 = ");
                        want.push_str(d);
                    }
                    want.push_str(" 次の一手 = ");
                    let rest = reason
                        .strip_prefix(want.as_str())
                        .unwrap_or_else(|| panic!("{reason} は {want} で始まらない"));
                    assert!(!rest.is_empty(), "{reason}");
                    assert!(!rest.contains("id =") && !rest.contains("要約値 ="), "{reason}");
                    if ids.is_empty() {
                        assert!(!reason.contains("id ="), "{reason}");
                    }
                }
            }
        }
    }
}

#[test]
fn qgate_own_names_clean() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/qgate.rs");
    let src = std::fs::read_to_string(&path).expect("tests/qgate.rs を読む");
    let lines: Vec<&str> = src.lines().collect();
    let names: Vec<&str> = lines
        .windows(2)
        .filter(|w| w[0].trim() == "#[test]")
        .map(|w| {
            let rest = w[1].trim().strip_prefix("fn ").expect("test の属性の次は fn");
            rest.split('(').next().unwrap_or(rest)
        })
        .collect();
    assert_eq!(names.len(), 8, "歯の数");
    for name in names {
        let rest = name
            .strip_prefix("qgate_")
            .unwrap_or_else(|| panic!("{name} は qgate_ で始まらない"));
        for word in FILTER_WORDS {
            assert!(!rest.contains(word), "{name} が {word} を含む");
        }
    }
}
