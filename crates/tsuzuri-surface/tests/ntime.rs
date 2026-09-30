//! 便 g-node-timeline の歯（接頭辞 ntime_）: 節点の頁の 3 つ目の block「run の時間軸」の枠と口の path・走行の id の bead・
//! 中心の行から読む走行・段の chip と色・経験者の行・中身の 3 値・語の鍵と stylesheet・DOM の結び・自分の歯の名。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_contract::graph::{AroundDoc, NodeKind};
use tsuzuri_contract::runs::{RunLine, RunStep, RunsDoc};
use tsuzuri_contract::wire;
use tsuzuri_surface::account::home::EXPERT_CHARS;
use tsuzuri_surface::frame::{self, PageId};
use tsuzuri_surface::project::nodearound::{self, PageState};
use tsuzuri_surface::project::timeline::{
    self, BLOCK, Chip, NO_RUNS, REASON, RunRow, STAGES, UNREAD, Want, body, chips, expert,
    kept_want, request, run_bead, stage_col,
};
use tsuzuri_surface::project::{Body, Module, NO_CONTENT, NOT_READ};
use tsuzuri_surface::view::Fetched;
use tsuzuri_surface::vocab::vocab;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// 節の fixture の電文（行 e-runs の節の値の t3-hub.2）。
const RUNS: &str = r#"{"bead":"t3-hub.2","runs":{"known":[
{"run":"t3-hub.2-20260927T071348Z","started_at":null,"account":null,"steps":[
 {"at":null,"stage":"Intake","detail":null,"verdict":null,"verdict_kind":null},
 {"at":null,"stage":"Reviewed","detail":null,"verdict":"FAIL","verdict_kind":"other"}],
 "cost":{"events":1,"turns":1,"wall_ms":17235,"tokens_in":2,"tokens_out":1541,"cache_read":10896,"cache_create":8407}},
{"run":"t3-hub.2-20260927T071750Z","started_at":null,"account":"acct-1","steps":[
 {"at":null,"stage":"Intake","detail":null,"verdict":null,"verdict_kind":null},
 {"at":null,"stage":"Reviewed","detail":null,"verdict":"PASS","verdict_kind":null},
 {"at":null,"stage":"Spawned","detail":null,"verdict":null,"verdict_kind":null},
 {"at":null,"stage":"Implemented","detail":null,"verdict":null,"verdict_kind":null},
 {"at":null,"stage":"Gated","detail":null,"verdict":"FAIL","verdict_kind":null}],
 "cost":{"events":2,"turns":33,"wall_ms":155603,"tokens_in":32,"tokens_out":15638,"cache_read":503756,"cache_create":43725}},
{"run":"t3-hub.2-20260927T072139Z","started_at":null,"account":"acct-1","steps":[
 {"at":null,"stage":"Intake","detail":null,"verdict":null,"verdict_kind":null},
 {"at":null,"stage":"Reviewed","detail":null,"verdict":"PASS","verdict_kind":null},
 {"at":null,"stage":"Spawned","detail":null,"verdict":null,"verdict_kind":null},
 {"at":null,"stage":"Implemented","detail":null,"verdict":null,"verdict_kind":null},
 {"at":null,"stage":"Gated","detail":null,"verdict":"PASS","verdict_kind":null},
 {"at":null,"stage":"Gated","detail":null,"verdict":null,"verdict_kind":null},
 {"at":null,"stage":"Landed","detail":null,"verdict":null,"verdict_kind":null}],
 "cost":{"events":3,"turns":40,"wall_ms":212986,"tokens_in":38,"tokens_out":23140,"cache_read":706443,"cache_create":69522}}
]}}"#;

fn runs_doc() -> RunsDoc {
    wire::decode(RUNS).expect("fixture の電文が RunsDoc として読める")
}

fn lines() -> Vec<RunLine> {
    match runs_doc().runs {
        tsuzuri_contract::board::Reading::Known(v) => v,
        tsuzuri_contract::board::Reading::Unknown => panic!("fixture の runs が Unknown"),
    }
}

/// 段の列（段の字と verdict と verdict_kind の組・ほかの欄は null）。
fn steps(spec: &[(&str, Option<&str>, Option<&str>)]) -> Vec<RunStep> {
    let q = |v: Option<&str>| v.map_or("null".to_string(), |s| format!("\"{s}\""));
    let items: Vec<String> = spec
        .iter()
        .map(|(stage, v, k)| {
            format!(
                r#"{{"at":null,"stage":"{stage}","detail":null,"verdict":{},"verdict_kind":{}}}"#,
                q(*v),
                q(*k)
            )
        })
        .collect();
    wire::decode(&format!("[{}]", items.join(","))).expect("段の列の電文")
}

fn chip(text: &str, class: &str, term: Option<&'static str>) -> Chip {
    Chip {
        text: text.to_string(),
        class: class.to_string(),
        term,
    }
}

fn strings(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| (*s).to_string()).collect()
}

fn around() -> AroundDoc {
    wire::decode(&read("../../tests/fixtures/surface/around-doc.json"))
        .expect("fixture が AroundDoc として読める")
}

/// 中心の行（列 0）の種類と id を書き換えた近傍。
fn around_as(kind: NodeKind, id: &str) -> AroundDoc {
    let mut doc = around();
    let row = doc
        .rows
        .iter_mut()
        .find(|r| r.col == 0)
        .expect("fixture に中心の行");
    row.node.kind = kind;
    row.node.id = id.to_string();
    doc
}

fn want(path: &str, only: Option<&str>) -> Want {
    Want {
        path: path.to_string(),
        only: only.map(str::to_string),
    }
}

/// (1) block の枠・Module の ALL に在る・節点の頁の block の並び・file の字・口の path。
#[test]
fn ntime_block_and_page() {
    assert_eq!(BLOCK.id, "timeline");
    assert_eq!(BLOCK.heading, "timeline");
    assert_eq!(BLOCK.class, "panel");
    assert!(timeline::PATHS.is_empty());
    assert!(timeline::FOLDS.is_empty());
    let m = Module::ALL
        .into_iter()
        .find(|m| m.name() == "timeline")
        .expect("Module の ALL に timeline");
    assert_eq!(m.block(), BLOCK);
    assert_eq!(
        frame::page(PageId::Node).block_ids(),
        vec!["node", "around", "timeline"]
    );
    let text = read("src/project/timeline.rs");
    assert!(text.contains("id: \"timeline\""));
    assert!(!text.contains("\"/api/"), "timeline.rs が /api/ の字を持つ");
    assert_eq!(timeline::path("t3-hub.2"), "/api/runs?bead=t3-hub.2");
    assert_eq!(timeline::path("a b"), "/api/runs?bead=a%20b");
}

/// (2) 走行の id の bead（最後のハイフンの後ろが器の時刻の形で前が空でない）。
#[test]
fn ntime_run_bead_shape() {
    assert_eq!(run_bead("t3-hub.2-20260927T071750Z"), Some("t3-hub.2"));
    assert_eq!(run_bead("a-b-20260927T000000Z"), Some("a-b"));
    for bad in [
        "t3-hub.2",
        "-20260927T000000Z",
        "x-20260927T0000Z",
        "x-2026092xT000000Z",
        "x-20260927X000000Z",
    ] {
        assert_eq!(run_bead(bad), None, "{bad}");
    }
}

/// (3) 中心の行から読む走行と、近傍の読みの状態で保つ値。
#[test]
fn ntime_request_by_kind() {
    let center = |doc: &AroundDoc| {
        doc.rows
            .iter()
            .find(|r| r.col == 0)
            .cloned()
            .expect("中心の行")
    };
    let task = want("/api/runs?bead=t3-hub.2", None);
    assert_eq!(
        request(&center(&around_as(NodeKind::Task, "t3-hub.2"))),
        Some(task.clone())
    );
    assert_eq!(
        request(&center(&around_as(NodeKind::Epic, "t3-hub"))),
        Some(want("/api/runs?bead=t3-hub", None))
    );
    assert_eq!(
        request(&center(&around_as(
            NodeKind::Run,
            "t3-hub.2-20260927T071750Z"
        ))),
        Some(want(
            "/api/runs?bead=t3-hub.2",
            Some("t3-hub.2-20260927T071750Z")
        ))
    );
    assert_eq!(request(&center(&around_as(NodeKind::Run, "odd-run"))), None);
    let fr1 = around();
    assert_eq!(center(&fr1).node.id, "FR1");
    assert_eq!(request(&center(&fr1)), None);
    assert_eq!(
        request(&center(&around_as(NodeKind::Question, "t3-hub.9"))),
        None
    );
    assert_eq!(request(&center(&around_as(NodeKind::Memo, "t3-hub.8"))), None);

    let task_doc = PageState::Doc(around_as(NodeKind::Task, "t3-hub.2"));
    assert_eq!(kept_want(None, &task_doc), Some(task.clone()));
    let via_state = nodearound::state(
        &Fetched::Body(wire::encode(&around_as(NodeKind::Task, "t3-hub.2")).expect("電文")),
        Some(200),
    );
    assert_eq!(kept_want(None, &via_state), Some(task.clone()));
    assert_eq!(
        kept_want(Some(task.clone()), &PageState::NotRead),
        Some(task.clone())
    );
    assert_eq!(
        kept_want(
            Some(task.clone()),
            &PageState::Unread(nodearound::REASON)
        ),
        Some(task.clone())
    );
    assert_eq!(kept_want(Some(task.clone()), &PageState::NotFound), None);
    assert_eq!(
        kept_want(Some(task.clone()), &PageState::Doc(around())),
        None
    );
    let mut no_center = around_as(NodeKind::Task, "t3-hub.2");
    no_center.rows.retain(|r| r.col != 0);
    assert_eq!(kept_want(Some(task), &PageState::Doc(no_center)), None);
}

/// (4) 段の表・色・続く同じ段をまとめた chip。
#[test]
fn ntime_chips_group_and_colour() {
    let want_stages: [(&str, &str, &str); 11] = [
        ("Intake", "stage:Intake", "wait"),
        ("Queued", "stage:Queued", "wait"),
        ("Blocked", "stage:Blocked", "wait"),
        ("Spawned", "stage:Spawned", "run"),
        ("Implemented", "stage:Implemented", "run"),
        ("Gated", "stage:Gated", "run"),
        ("Reviewed", "stage:Reviewed", "wait"),
        ("Questioned", "stage:Questioned", "stop"),
        ("Failed", "stage:Failed", "stop"),
        ("Stopped", "stage:Stopped", "stop"),
        ("Landed", "stage:Landed", "land"),
    ];
    assert_eq!(STAGES, want_stages);

    assert_eq!(stage_col("Reviewed", Some("PASS")), "wait");
    assert_eq!(stage_col("Reviewed", Some("FAIL")), "stop");
    assert_eq!(stage_col("Landed", Some("FAIL")), "stop");
    assert_eq!(stage_col("Questioned", None), "stop");
    assert_eq!(stage_col("Spawned", None), "run");
    assert_eq!(stage_col("Landed", None), "land");
    assert_eq!(stage_col("Weird", None), "wait");

    assert_eq!(
        chips(&steps(&[("Intake", None, None), ("Intake", None, None)])),
        vec![chip("Intake", "stg c-wait", Some("stage:Intake"))]
    );
    assert_eq!(
        chips(&steps(&[("Landed", None, None); 4])),
        vec![chip("Landed", "stg c-land", Some("stage:Landed"))]
    );
    assert_eq!(
        chips(&steps(&[("Gated", Some("PASS"), None), ("Gated", None, None)])),
        vec![chip("Gated PASS", "stg c-run", Some("stage:Gated"))]
    );
    assert_eq!(
        chips(&steps(&[("Gated", None, None), ("Gated", Some("FAIL"), None)])),
        vec![chip("Gated FAIL", "stg c-stop", Some("stage:Gated"))]
    );
    assert_eq!(
        chips(&steps(&[
            ("Gated", None, None),
            ("Implemented", None, None),
            ("Gated", None, None)
        ])),
        vec![
            chip("Gated", "stg c-run", Some("stage:Gated")),
            chip("Implemented", "stg c-run", Some("stage:Implemented")),
            chip("Gated", "stg c-run", Some("stage:Gated")),
        ]
    );
    assert_eq!(
        chips(&steps(&[("Reviewed", Some("INCONCLUSIVE"), None)])),
        vec![chip(
            "Reviewed INCONCLUSIVE",
            "stg c-stop",
            Some("stage:Reviewed")
        )]
    );
    assert_eq!(
        chips(&steps(&[("Weird", None, None)])),
        vec![chip("Weird", "stg c-wait", None)]
    );
    assert_eq!(chips(&[]), Vec::<Chip>::new());
}

/// 走行の電文（段と費用を書き換える元・account は null・cost は全部 0）。
fn bare_run(account: Option<&str>, spec: &[(&str, Option<&str>, Option<&str>)]) -> RunLine {
    let mut run: RunLine = wire::decode(
        r#"{"run":"x-20260927T000000Z","started_at":null,"account":null,"steps":[],
        "cost":{"events":0,"turns":0,"wall_ms":0,"tokens_in":0,"tokens_out":0,"cache_read":0,"cache_create":0}}"#,
    )
    .expect("走行の電文");
    run.account = account.map(str::to_string);
    run.steps = steps(spec);
    run
}

/// (5) 経験者の行（口座・費用・審査の結びと種類・60 字で畳む）。
#[test]
fn ntime_expert_lines_wrap() {
    let all = lines();
    let want = [
        strings(&[
            "account=― turns=1 wall=17s in=2 out=1541 cache=10896/8407",
            "Reviewed=FAIL/other",
        ]),
        strings(&[
            "account=acct-1 turns=33 wall=155s in=32 out=15638",
            "cache=503756/43725 Reviewed=PASS Gated=FAIL",
        ]),
        strings(&[
            "account=acct-1 turns=40 wall=212s in=38 out=23140",
            "cache=706443/69522 Reviewed=PASS Gated=PASS",
        ]),
    ];
    assert_eq!(all.len(), want.len());
    for (run, w) in all.iter().zip(want) {
        assert_eq!(expert(run), w, "{}", run.run);
    }
    let odd = bare_run(
        None,
        &[
            ("Intake", None, None),
            ("Reviewed", Some("INCONCLUSIVE"), Some("section-material-missing")),
        ],
    );
    assert_eq!(
        expert(&odd),
        strings(&[
            "account=― cost=―",
            "Reviewed=INCONCLUSIVE/section-material-missing"
        ])
    );
    let long = bare_run(Some(&"a".repeat(70)), &[]);
    let got = expert(&long);
    assert_eq!(
        got,
        vec![
            format!("account={}", "a".repeat(52)),
            format!("{} cost=―", "a".repeat(18)),
        ]
    );
    for run in all.iter().chain([&odd, &long]) {
        for line in expert(run) {
            assert!(line.chars().count() <= EXPERT_CHARS, "{line}");
        }
    }
}

/// (6) fixture の 3 つの走行の行・走行の頁で絞った行・電文に無い走行。
#[test]
fn ntime_rows_on_fixture() {
    let fetched = Fetched::Body(RUNS.to_string());
    let all = lines();
    let Body::Filled(rows) = body(&fetched, None) else {
        panic!("fixture が Filled でない");
    };
    let ids: Vec<&str> = rows.iter().map(|r| r.run.as_str()).collect();
    assert_eq!(
        ids,
        vec![
            "t3-hub.2-20260927T071348Z",
            "t3-hub.2-20260927T071750Z",
            "t3-hub.2-20260927T072139Z"
        ]
    );
    let nths: Vec<&str> = rows.iter().map(|r| r.nth.as_str()).collect();
    assert_eq!(nths, vec!["1", "2", "3"]);
    for (row, run) in rows.iter().zip(&all) {
        assert_eq!(row.chips, chips(&run.steps));
        assert_eq!(row.expert, expert(run));
    }
    let i = |s: &'static str| Some(s);
    assert_eq!(
        rows[0].chips,
        vec![
            chip("Intake", "stg c-wait", i("stage:Intake")),
            chip("Reviewed FAIL", "stg c-stop", i("stage:Reviewed")),
        ]
    );
    let second = vec![
        chip("Intake", "stg c-wait", i("stage:Intake")),
        chip("Reviewed PASS", "stg c-wait", i("stage:Reviewed")),
        chip("Spawned", "stg c-run", i("stage:Spawned")),
        chip("Implemented", "stg c-run", i("stage:Implemented")),
        chip("Gated FAIL", "stg c-stop", i("stage:Gated")),
    ];
    assert_eq!(rows[1].chips, second);
    assert_eq!(
        rows[2].chips,
        vec![
            chip("Intake", "stg c-wait", i("stage:Intake")),
            chip("Reviewed PASS", "stg c-wait", i("stage:Reviewed")),
            chip("Spawned", "stg c-run", i("stage:Spawned")),
            chip("Implemented", "stg c-run", i("stage:Implemented")),
            chip("Gated PASS", "stg c-run", i("stage:Gated")),
            chip("Landed", "stg c-land", i("stage:Landed")),
        ]
    );
    assert_eq!(
        body(&fetched, Some("t3-hub.2-20260927T071750Z")),
        Body::Filled(vec![RunRow {
            run: "t3-hub.2-20260927T071750Z".to_string(),
            nth: String::new(),
            chips: second,
            expert: rows[1].expert.clone(),
        }])
    );
    assert_eq!(
        body(&fetched, Some("t3-hub.2-20260927T000000Z")),
        Body::Empty(NO_RUNS)
    );
}

/// (7) 中身の 3 値（まだ読んでいない・読めない・読めない本文・log が読めない・0 件）。
#[test]
fn ntime_body_three_values() {
    assert_eq!(body(&Fetched::NotRead, None), Body::Unmeasured(NOT_READ));
    assert_eq!(body(&Fetched::Failed, None), Body::Unmeasured(REASON));
    for text in ["{}", "[]"] {
        assert_eq!(
            body(&Fetched::Body(text.to_string()), None),
            Body::Unmeasured(NO_CONTENT),
            "{text}"
        );
    }
    let mut doc = runs_doc();
    doc.runs = tsuzuri_contract::board::Reading::Unknown;
    let unknown = wire::encode(&doc).expect("電文");
    assert!(unknown.contains("\"unknown\""), "{unknown}");
    assert_eq!(
        body(&Fetched::Body(unknown), None),
        Body::Unmeasured(UNREAD)
    );
    doc.runs = tsuzuri_contract::board::Reading::Known(vec![]);
    let empty = wire::encode(&doc).expect("電文");
    assert_eq!(body(&Fetched::Body(empty), None), Body::Empty(NO_RUNS));
    for s in [REASON, UNREAD, NO_RUNS] {
        assert!(!s.trim().is_empty());
        assert!(!s.contains('\n'), "{s}");
    }
}

/// style.css の規則の名（`.名` の名・comment を除く）。
fn stylesheet_classes(css: &str) -> Vec<String> {
    let mut text = String::new();
    let mut rest = css;
    while let Some(i) = rest.find("/*") {
        text.push_str(&rest[..i]);
        rest = rest[i..].find("*/").map_or("", |j| &rest[i + j + 2..]);
    }
    text.push_str(rest);
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    for i in 0..chars.len() {
        let starts = chars
            .get(i + 1)
            .is_some_and(|c| c.is_ascii_alphabetic() || *c == '_' || *c == '-');
        if chars[i] == '.' && starts {
            out.push(
                chars[i + 1..]
                    .iter()
                    .take_while(|c| c.is_ascii_alphanumeric() || **c == '-' || **c == '_')
                    .collect(),
            );
        }
    }
    out
}

/// (8) 見出しと段の語の鍵は語の辞書に在り、段の色の class と時間軸の規則は style.css に在る。
#[test]
fn ntime_vocab_and_css() {
    let v = vocab();
    for key in std::iter::once(BLOCK.heading).chain(STAGES.iter().map(|(_, k, _)| *k)) {
        let term = v.term(key).unwrap_or_else(|| panic!("語の辞書に {key} が無い"));
        assert!(!term.label.trim().is_empty(), "{key} の語が空");
    }
    let css = read("style.css");
    let names = stylesheet_classes(&css);
    let mut want: Vec<String> = vec!["stg".to_string()];
    for (stage, _, _) in STAGES {
        for verdict in [None, Some("FAIL")] {
            want.push(format!("c-{}", stage_col(stage, verdict)));
        }
    }
    for w in want {
        assert!(names.contains(&w), "style.css に規則の名 {w} が無い");
    }
    for rule in [
        ".runs",
        ".runrow",
        ".runrow .nth",
        ".stages",
        ".stg.c-wait",
        ".stg.c-run",
        ".stg.c-stop",
        ".stg.c-land",
        ".arrow",
    ] {
        assert!(css.contains(rule), "style.css に {rule} が無い");
    }
}

/// (9) DOM の部分（wasm の target のときだけ）は近傍の読み・走行の読み・中身・注釈・link を結ぶ。
#[test]
fn ntime_dom_wiring() {
    let text = read("src/project/timeline.rs");
    let (_, dom) = text.split_once("mod dom {").expect("timeline.rs に mod dom");
    for word in [
        "source()",
        "read_path(",
        "kept_want(",
        "body(",
        "shows_internal(",
        "node_href(",
        "term(",
        "section(BLOCK",
        "class=\"runs\"",
        "class=\"runrow\"",
        "class=\"nth num\"",
        "class=\"stages\"",
        "class=\"chip num\"",
        "class=\"small muted mono\"",
    ] {
        assert!(dom.contains(word), "DOM の部分に {word} が無い");
    }
}

/// 着地済みの行と起草中の行の verify の filter の語（この行の接頭辞は並べない）。
const FILTERS: [&str; 80] = [
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
    "hcard_",
    "qgate_",
    "nsum_",
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
];

/// (11) この file の歯の名はどれも ntime_ で始まり、残りの字は filter の語を含まない。
#[test]
fn ntime_own_names_clean() {
    let text = read("tests/ntime.rs");
    let lines: Vec<&str> = text.lines().map(str::trim).collect();
    let names: Vec<&str> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| **l == "#[test]")
        .map(|(i, _)| {
            lines[i + 1..]
                .iter()
                .find_map(|l| l.strip_prefix("fn "))
                .and_then(|rest| rest.split('(').next())
                .expect("test の属性の後の fn")
        })
        .collect();
    assert!(names.len() >= 10, "歯の数 {}", names.len());
    for name in names {
        let rest = name
            .strip_prefix("ntime_")
            .unwrap_or_else(|| panic!("{name} が ntime_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
