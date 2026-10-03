//! 受入 12 条の runner の歯（行 j-runner・接頭辞 jrun_）。
//! 頁の口は歯の中の偽の頁（撃った命令と測りと URL の読みを順に記録し、決まった事実の字と event を返す）で撃ち、
//! 本物の Chrome を起こさない。事実の字は行 j-count の fixture の clean.json と vocab.json を使う。
#![cfg(test)]

use std::fs;
use std::path::{Path, PathBuf};

use tsuzuri_boundary::audit::{
    MODES, Page, RULES, SCREENS, Switch, WIDTHS, case, count, errors, head, line, sweep,
};
use tsuzuri_boundary::stage::cdp::Command;

/// 撃った URL の board（clean.json の読み先と同じ origin）。
const BOARD: &str = "http://127.0.0.1:4801/";

/// 頁へ移る前の console の event（page error に数えない）。
const STALE: &str =
    r#"{"method":"Runtime.exceptionThrown","params":{"exceptionDetails":{"text":"Uncaught stale"}}}"#;

/// 読み込みの終わりの event。
const LOAD: &str = r#"{"method":"Page.loadEventFired","params":{"timestamp":1}}"#;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fixture(name: &str) -> String {
    let path = root().join("tests/fixtures/surface/accept-12").join(name);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn exception(text: &str) -> String {
    format!(
        r#"{{"method":"Runtime.exceptionThrown","params":{{"timestamp":2,"exceptionDetails":{{"exceptionId":1,"text":"{text}","lineNumber":0}}}}}}"#
    )
}

fn entry(level: &str, text: &str) -> String {
    format!(
        r#"{{"method":"Log.entryAdded","params":{{"entry":{{"source":"network","level":"{level}","text":"{text}"}}}}}}"#
    )
}

/// 頁へ移るたびに足す event（移った URL と何度目か）。
type Arrived = Box<dyn Fn(&str, usize) -> Vec<String>>;

/// 偽の頁（Navigate で今の URL を替え、Click は在りかの行き先へ URL を替える）。
struct Fake {
    log: Vec<String>,
    events: Vec<String>,
    current: String,
    navigations: usize,
    moves: Vec<(u32, u32, String)>,
    measured: Box<dyn Fn(&str) -> String>,
    arrived: Arrived,
}

impl Fake {
    fn new(
        measured: impl Fn(&str) -> String + 'static,
        arrived: impl Fn(&str, usize) -> Vec<String> + 'static,
    ) -> Fake {
        Fake {
            log: Vec::new(),
            events: Vec::new(),
            current: String::new(),
            navigations: 0,
            moves: Vec::new(),
            measured: Box::new(measured),
            arrived: Box::new(arrived),
        }
    }

    /// 撃った Navigate の URL（順に）。
    fn navigated(&self) -> Vec<String> {
        self.log
            .iter()
            .filter_map(|l| l.strip_prefix("Navigate "))
            .map(str::to_string)
            .collect()
    }
}

impl Page for Fake {
    fn run(&mut self, command: &Command) -> Result<Vec<String>, String> {
        match command {
            Command::Console => self.events.push(STALE.to_string()),
            Command::Navigate { url } => {
                self.log.push(format!("Navigate {url}"));
                self.current.clone_from(url);
                self.navigations += 1;
                let arrived = (self.arrived)(url, self.navigations);
                self.events.extend(arrived);
                return Ok(Vec::new());
            }
            Command::Click { x, y } => {
                if let Some((_, _, to)) = self.moves.iter().find(|(a, b, _)| (a, b) == (x, y)) {
                    self.current.clone_from(to);
                }
            }
            _ => {}
        }
        self.log.push(format!("{command:?}"));
        Ok(Vec::new())
    }

    fn measure(&mut self) -> Result<String, String> {
        self.log.push("measure".to_string());
        Ok((self.measured)(&self.current))
    }

    fn url(&mut self) -> Result<String, String> {
        self.log.push("url".to_string());
        Ok(self.current.clone())
    }

    fn events(&self) -> &[String] {
        &self.events
    }
}

/// 鍵 `key` の条だけが `n` の数の列を足した列。
fn plus(mut counts: [usize; 12], key: &str, n: usize) -> [usize; 12] {
    let at = RULES.iter().position(|(k, _)| *k == key).expect("条の鍵");
    counts[at] += n;
    counts
}

/// 行の中の `"…"` の字の列。
fn quoted(text: &str) -> Vec<&str> {
    text.split('"').skip(1).step_by(2).collect()
}

/// fn の頭の行から、その fn の腕の `=> "…"` の字（fn の閉じまで）。
fn arms(text: &str, head: &str) -> Vec<String> {
    let body = &text[text.find(head).unwrap_or_else(|| panic!("{head} が無い"))..];
    let end = body.find("\n    }\n").expect("fn の閉じ");
    body[..end]
        .lines()
        .filter_map(|l| l.split_once("=> ").map(|(_, r)| r))
        .filter_map(|r| quoted(r).first().map(ToString::to_string))
        .collect()
}

#[test]
fn jrun_screens_cover_surface() {
    assert_eq!(WIDTHS, [1280, 960, 700, 390]);
    assert_eq!(MODES, ["beginner", "expert"]);
    let src = root().join("crates/tsuzuri-surface/src");
    let mut want = vec!["?".to_string()];
    let pages: Vec<String> = fs::read_dir(src.join("pages"))
        .expect("面の頁の dir を読む")
        .map(|e| e.expect("dir の項").file_name().to_string_lossy().into_owned())
        .filter_map(|n| n.strip_suffix(".rs").map(str::to_string))
        .filter(|n| !["mod", "home", "node"].contains(&n.as_str()))
        .collect();
    // home と節点の頁のほかの頁の file は無い（質問の頁と抜けの検査の頁は行 g-one-screen-a で消した）。
    assert!(pages.is_empty(), "頁の file {pages:?}");
    // 帯の印が開く窓は home の頁の query の win で開く（行 g-accept）。
    let topbar = fs::read_to_string(src.join("topbar.rs")).expect("topbar.rs を読む");
    let wins = arms(&topbar, "pub fn key(self) -> &'static str {");
    assert_eq!(wins.len(), 7, "{wins:?}");
    want.extend(wins.iter().map(|w| format!("?win={w}&")));
    let account = fs::read_to_string(src.join("account/mod.rs")).expect("account/mod.rs を読む");
    let tabs = arms(&account, "pub fn id(self) -> &'static str {");
    assert_eq!(tabs, ["home", "session", "projects"]);
    want.extend(tabs.iter().map(|t| format!("?board=account&tab={t}&")));
    let mut got: Vec<String> = SCREENS.iter().map(ToString::to_string).collect();
    got.sort();
    got.dedup();
    assert_eq!(got.len(), 11, "SCREENS に重なりが無い");
    want.sort();
    assert_eq!(got, want);
}

#[test]
fn jrun_case_order_and_readings() {
    let url = format!("{BOARD}?win=gaps&mode=beginner");
    let pressed = format!("{BOARD}?win=gaps&mode=expert");
    let spots = r#""switch_at": [{"label": "expert", "now": false, "x": 10, "y": 20}, {"label": "hold", "now": false, "x": 30, "y": 40}]"#;
    let text = fixture("clean.json").replace(r#""switch_at": []"#, spots);
    assert!(text.contains("hold"), "switch_at を仕込む");
    let mut page = Fake::new(
        move |_| text.clone(),
        |_, n| {
            if n == 1 {
                vec![
                    LOAD.to_string(),
                    exception("Uncaught boom"),
                    entry("warning", "slow"),
                ]
            } else {
                vec![LOAD.to_string(), exception("Uncaught later")]
            }
        },
    );
    page.moves.push((10, 20, pressed.clone()));
    let facts = case(&mut page, &url, 390).expect("case");
    let wait = format!("{:?}", Command::Wait { ms: 1000 });
    let navigate = format!("Navigate {url}");
    let want: Vec<String> = [
        format!(
            "{:?}",
            Command::Viewport {
                width: 390,
                height: 844,
                scale: 1,
                mobile: true,
            }
        ),
        format!("{:?}", Command::Console),
        navigate.clone(),
        wait.clone(),
        "measure".to_string(),
        "url".to_string(),
        format!("{:?}", Command::Click { x: 10, y: 20 }),
        "url".to_string(),
        navigate.clone(),
        wait.clone(),
        "measure".to_string(),
        "url".to_string(),
        format!("{:?}", Command::Click { x: 30, y: 40 }),
        "url".to_string(),
        navigate,
        wait,
        "measure".to_string(),
    ]
    .to_vec();
    assert_eq!(page.log, want);
    assert_eq!(facts.url, url);
    assert_eq!(facts.errors, ["Uncaught boom"]);
    switches_and_widths(url, pressed, facts);
}

/// 押した切り替えの読みと数え、広い幅の case と事実の形の違い。
fn switches_and_widths(url: String, pressed: String, facts: tsuzuri_boundary::audit::Facts) {
    assert_eq!(
        facts.switches,
        [
            Switch {
                label: "expert".to_string(),
                before: url.clone(),
                after: pressed,
            },
            Switch {
                label: "hold".to_string(),
                before: url.clone(),
                after: url.clone(),
            },
        ]
    );
    assert_eq!(
        count(&facts, &fixture("vocab.json")),
        plus(plus([0; 12], "error", 1), "url", 1)
    );

    let mut wide = Fake::new(|_| fixture("clean.json"), |_, _| vec![LOAD.to_string()]);
    let facts = case(&mut wide, &url, 1280).expect("広い幅の case");
    assert_eq!(
        wide.log.first().cloned(),
        Some(format!(
            "{:?}",
            Command::Viewport {
                width: 1280,
                height: 800,
                scale: 1,
                mobile: false,
            }
        ))
    );
    assert_eq!(wide.log.len(), 5, "{:?}", wide.log);
    assert!(facts.switches.is_empty() && facts.errors.is_empty());
    assert_eq!(count(&facts, &fixture("vocab.json")), [0; 12]);

    let mut odd = Fake::new(
        |_| fixture("clean.json").replace(r#""switch_at": []"#, r#""switch_at": [{"x": 1}]"#),
        |_, _| Vec::new(),
    );
    let err = case(&mut odd, &url, 390).expect_err("switch_at の形の違い");
    assert!(err.contains("switch_at") && err.contains(&url), "{err}");
    let mut bare = Fake::new(|_| r#"{"url": ""}"#.to_string(), |_, _| Vec::new());
    let err = case(&mut bare, &url, 390).expect_err("鍵の欠け");
    assert!(err.contains("hscroll"), "{err}");
}

#[test]
fn jrun_errors_from_events() {
    let events = [
        LOAD.to_string(),
        exception("Uncaught a"),
        entry("warning", "w"),
        entry("error", "e"),
        r#"{"method":"Runtime.consoleAPICalled","params":{"type":"error","args":[]}}"#.to_string(),
        entry("info", "i"),
        exception("Uncaught b"),
        r#"{"id":3,"result":{}}"#.to_string(),
        "not json".to_string(),
    ];
    assert_eq!(errors(&events), ["Uncaught a", "e", "Uncaught b"]);
    assert!(errors(&[]).is_empty());
    assert!(errors(&[LOAD.to_string(), entry("warning", "w")]).is_empty());
}

/// 違反もまだ分からない画面も外した置き場も無い sweep の report の末の 3 行。
const TAIL: &str =
    "違反 計 0\nまだ分からない 計 0\n外した置き場 計 台帳の字 0 語と値の対 0 見えない要素 0";

#[test]
fn jrun_sweep_all_cases() {
    const ID: &str = "t3-hub.5_2~#é";
    let clean = fixture("clean.json");
    assert_eq!(clean.matches("t3-hub.52").count(), 2, "札の id と字");
    let first = clean.replace("t3-hub.52", ID);
    let vocab = fixture("vocab.json");
    let measured = move |url: &str| {
        if url.starts_with(&format!("{BOARD}?mode=")) {
            first.clone()
        } else {
            clean.clone()
        }
    };
    let mut page = Fake::new(measured.clone(), |_, _| vec![LOAD.to_string()]);
    let (report, total, unknown) = sweep(&mut page, BOARD, &vocab).expect("sweep");
    let mut urls = Vec::new();
    for width in [1280, 960, 700, 390] {
        for mode in ["beginner", "expert"] {
            let mut group: Vec<String> = SCREENS
                .iter()
                .map(|s| format!("{BOARD}{s}mode={mode}"))
                .collect();
            group.push(format!(
                "{BOARD}?page=node&id=t3-hub.5_2~%23%C3%A9&mode={mode}"
            ));
            urls.extend(group.into_iter().map(|u| (width, mode, u)));
        }
    }
    assert_eq!(urls.len(), 96);
    let navigated: Vec<String> = urls.iter().map(|(_, _, u)| u.clone()).collect();
    assert_eq!(page.navigated(), navigated);
    let mut want = vec![head()];
    want.extend(
        urls.iter()
            .map(|(width, mode, url)| line(*width, mode, url, &[0; 12])),
    );
    want.push(TAIL.to_string());
    assert_eq!(report, format!("{}\n", want.join("\n")));
    assert_eq!((total, unknown), (0, 0));

    let mut broken = Fake::new(measured, |url, _| {
        let mut events = vec![LOAD.to_string()];
        if url.contains("page=node") {
            events.push(exception("Uncaught node"));
        }
        events
    });
    let (report, total, unknown) = sweep(&mut broken, BOARD, &vocab).expect("page error");
    assert_eq!((total, unknown), (8, 0));
    assert!(
        report.contains("\n違反 計 8\nまだ分からない 計 0\n"),
        "{report}"
    );
    let node_line = line(
        390,
        "expert",
        &format!("{BOARD}?page=node&id=t3-hub.5_2~%23%C3%A9&mode=expert"),
        &plus([0; 12], "error", 1),
    );
    assert!(report.lines().any(|l| l == node_line), "{report}");
    sweep_without_nodes(vocab);
}

/// 節点の無い home では、節点の頁を開かずに止まる。
fn sweep_without_nodes(vocab: String) {
    let clean = fixture("clean.json");
    let start = clean.find(r#""nodes": ["#).expect("札の列");
    let end = start + clean[start..].find(']').expect("札の列の閉じ");
    let bare = format!(r#"{}"nodes": []{}"#, &clean[..start], &clean[end + 1..]);
    let mut empty = Fake::new(move |_| bare.clone(), |_, _| vec![LOAD.to_string()]);
    let err = sweep(&mut empty, BOARD, &vocab).expect_err("節点の無い home");
    assert!(err.contains("節点が無い"), "{err}");
    assert_eq!(
        empty.navigated().len(),
        SCREENS.len(),
        "節点の頁を開かずに止まる"
    );
}

#[test]
fn jrun_measure_expression_text() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/stage/measure.js");
    let text = fs::read_to_string(&path).expect("measure.js を読む");
    assert!(text.starts_with("// 受入 12 条の測りの式"), "頭の字");
    for word in [
        "innerHTML",
        "fetch(",
        "XMLHttpRequest",
        "sendBeacon",
        "WebSocket",
        "location.",
        "localStorage",
        "document.write",
        "eval(",
        "Function(",
        "click(",
        "dispatchEvent",
    ] {
        assert!(!text.contains(word), "measure.js が {word} を含む");
    }
    let removes: Vec<&str> = text.lines().filter(|l| l.contains(".remove()")).collect();
    assert_eq!(text.matches(".remove()").count(), 1, "{removes:?}");
    assert!(
        removes.len() == 1 && removes[0].contains(r#"querySelectorAll(".q")"#),
        "{removes:?}"
    );
    assert!(text.contains("getEventListeners"), "hover の受け手の見方");
    returned_object_keys(text);
}

/// 測りの式の返す object の鍵の並びと、決まった値の欄。
fn returned_object_keys(text: String) {
    let start = text.find("return JSON.stringify({").expect("返す object");
    let body = &text[start..];
    let body = &body[..body.find("});").expect("object の閉じ")];
    let entries: Vec<&str> = body
        .lines()
        .skip(1)
        .map(str::trim)
        .filter(|e| !e.is_empty())
        .collect();
    let keys: Vec<&str> = entries
        .iter()
        .map(|e| e.split([':', ',']).next().unwrap_or(e))
        .collect();
    assert_eq!(
        keys,
        [
            "url",
            "hscroll",
            "overflow",
            "overlap",
            "nocard",
            "reach",
            "skipped",
            "headings",
            "first",
            "errors",
            "titles",
            "nodes",
            "switches",
            "text",
            "libraries",
            "switch_at",
        ]
    );
    for fixed in [
        r#"url: "","#,
        "errors: [],",
        "switches: [],",
        "text: document.body.innerText,",
    ] {
        assert!(entries.contains(&fixed), "{fixed}: {entries:?}");
    }
}
