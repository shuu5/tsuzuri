//! 行 g-accept-runner の歯（接頭辞 bvsettle_）: 受入の runner の待ちと今の選び。
//! runner は頁を開いた後、面が描き、どの block も読みの印（面の kit の NOT_READ の字）を出さなくなるまで上限つきで待ち、
//! 上限で残る画面は違反 0 と数えずまだ分からないとする（憲法 P-7）。測りの式は切り替えの在りかに今の選びの印 now を
//! 付け、runner は今の選びを押さず、今でない選びは今までどおり押して URL を比べる。
//! 頁の口は歯の中の偽の頁（撃った命令と測りと URL の読みを順に記録する）で撃ち、本物の Chrome を起こさない。
#![cfg(test)]

use std::fs;
use std::path::Path;

use tsuzuri_boundary::audit::{
    MODES, NOT_READ, POLL_MS, Page, READ_POLLS, RULES, SCREENS, UNSETTLED, WIDTHS, case, count,
    details, facts, line, skipped_counts, sweep, unread,
};
use tsuzuri_boundary::stage::cdp::Command;

/// 撃った URL の board（clean.json の読み先と同じ origin）。
const BOARD: &str = "http://127.0.0.1:4801/";

/// clean.json の頁の見える字の欄。
const BODY: &str = r#""text": "pipeline dashboard\nすき間\n走行の今の段を 5 列で並べる。\n次の一手は user が決める（12:30Z）。","#;

fn read(rel: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// `head` で始まり、その後の最初の `close` の手前で終わる切り（`head` は字の中に 1 つだけ）を、改行と続く字下げを
/// 除いてつないだ字（JS の鎖を頭から尾まで、関数や腕を頭から閉じまで照らす）。
fn cut(text: &str, head: &str, close: &str) -> String {
    assert_eq!(text.matches(head).count(), 1, "{head}");
    let from = text.find(head).expect("切りの頭");
    let to = from + text[from..].find(close).expect("切りの閉じ");
    text[from..to].split('\n').map(str::trim_start).collect()
}

fn clean() -> String {
    read("tests/fixtures/surface/accept-12/clean.json")
}

fn vocab() -> String {
    read("tests/fixtures/surface/accept-12/vocab.json")
}

/// clean.json の見える字の欄を `body` の字に替えた事実の字。
fn with_body(body: &str) -> String {
    let text = clean();
    assert_eq!(text.matches(BODY).count(), 1, "clean.json の見える字の欄");
    text.replace(BODY, &format!(r#""text": "{body}","#))
}

/// どれかの block が読みの印を出す頁の事実の字。
fn pending() -> String {
    with_body(&format!("pipeline dashboard\\n測れていない {NOT_READ}"))
}

/// clean.json の切り替えの在りかを `spots` の字に替えた事実の字。
fn with_spots(spots: &str) -> String {
    clean().replace(r#""switch_at": []"#, &format!(r#""switch_at": [{spots}]"#))
}

/// 偽の頁（Navigate で今の URL を替え、Click は在りかの行き先へ URL を替える）。測りの字は
/// 今の URL と、何度目に移った頁かと、移ってから何度目の測りかで決まる。
struct Fake {
    log: Vec<String>,
    current: String,
    navigations: usize,
    polls: usize,
    moves: Vec<(u32, u32, String)>,
    measured: Measured,
}

/// 測りの字を返す口（今の URL・何度目に移った頁か・移ってから何度目の測りか）。
type Measured = Box<dyn Fn(&str, usize, usize) -> String>;

impl Fake {
    fn new(measured: impl Fn(&str, usize, usize) -> String + 'static) -> Fake {
        let (log, current, moves) = (Vec::new(), String::new(), Vec::new());
        let measured = Box::new(measured);
        Fake {
            log,
            current,
            navigations: 0,
            polls: 0,
            moves,
            measured,
        }
    }

    /// 撃った Navigate の URL（順に）。
    fn navigated(&self) -> Vec<&str> {
        self.log
            .iter()
            .filter_map(|l| l.strip_prefix("Navigate "))
            .collect()
    }

    /// 記録の中の `entry` の数。
    fn times(&self, entry: &str) -> usize {
        self.log.iter().filter(|l| *l == entry).count()
    }
}

impl Page for Fake {
    fn run(&mut self, command: &Command) -> Result<Vec<String>, String> {
        match command {
            Command::Navigate { url } => {
                self.log.push(format!("Navigate {url}"));
                self.current.clone_from(url);
                (self.navigations, self.polls) = (self.navigations + 1, 0);
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
        self.polls += 1;
        Ok((self.measured)(&self.current, self.navigations, self.polls))
    }

    fn url(&mut self) -> Result<String, String> {
        self.log.push("url".to_string());
        Ok(self.current.clone())
    }

    fn events(&self) -> &[String] {
        &[]
    }
}

fn wait(ms: u64) -> String {
    format!("{:?}", Command::Wait { ms })
}

/// URL `target` の頁だけ読みの印が残り、ほかは読めた頁。
fn stuck_at(target: String) -> Fake {
    Fake::new(move |url, _, _| if url == target { pending() } else { clean() })
}

/// (2) 読みの印の判じと待ちの間: 見える字が空か読みの印を含めば unread。頁を開いて 1000 ms 待って測り、
/// unread の間は 300 ms ごとに測り直し、印の消えた測りの事実で数える（印の消える偽の頁は上限の前に測りに進む）。
#[test]
fn bvsettle_waits_until_read() {
    assert_eq!((POLL_MS, READ_POLLS), (300, 100));
    assert!(unread("") && unread(" \n ") && unread(&format!("pipeline\n{NOT_READ}\n")));
    assert!(!unread("pipeline dashboard") && !unread("この block の口を読んだ"));
    let url = format!("{BOARD}?win=gaps&mode=beginner");
    for first in [pending(), with_body(""), with_body(" ")] {
        let mut page = Fake::new(move |_, _, poll| if poll <= 3 { first.clone() } else { clean() });
        let facts = case(&mut page, &url, 1280).expect("印の消える頁");
        let mut want = vec![format!("Navigate {url}"), wait(1000), "measure".to_string()];
        for _ in 0..3 {
            want.extend([wait(POLL_MS), "measure".to_string()]);
        }
        assert_eq!(page.log.get(2..), Some(want.as_slice()));
        assert!(!unread(&facts.text) && facts.text.starts_with("pipeline"));
        assert_eq!(count(&facts, &vocab()), [0; 12]);
    }
    let mut ready = Fake::new(|_, _, _| clean());
    case(&mut ready, &url, 390).expect("読めた頁");
    assert_eq!(
        (ready.times("measure"), ready.times(&wait(POLL_MS))),
        (1, 0)
    );
}

/// (3) 印が上限まで残る頁: 測りは READ_POLLS 回で止まり（間の待ちは 1 つ少ない）、case は URL と UNSETTLED を含む Err。
/// 切り替えを押して戻った頁で印が上限まで残るのも Err。
#[test]
fn bvsettle_limit_is_unknown() {
    let url = format!("{BOARD}?win=stalled&mode=beginner");
    assert!(UNSETTLED.starts_with("まだ分からない"), "{UNSETTLED}");
    for stuck in [pending(), with_body("")] {
        let mut page = Fake::new(move |_, _, _| stuck.clone());
        let err = case(&mut page, &url, 700).expect_err("印の残る頁");
        assert!(err.contains(&url) && err.contains(UNSETTLED), "{err}");
        assert_eq!(page.times("measure"), READ_POLLS);
        assert_eq!(page.times(&wait(POLL_MS)), READ_POLLS - 1);
        assert_eq!(page.times(&wait(1000)), 1);
    }
    let spots = with_spots(r#"{"label": "expert", "now": false, "x": 5, "y": 6}"#);
    let mut back = Fake::new(move |_, nav, _| if nav == 1 { spots.clone() } else { pending() });
    let err = case(&mut back, &url, 700).expect_err("戻った頁で印が残る");
    assert!(err.contains(UNSETTLED), "{err}");
    assert_eq!(back.navigated(), [url.as_str(), url.as_str()]);
    assert_eq!(back.times("measure"), 1 + READ_POLLS);
}

/// (4) sweep は印が上限まで残る画面を違反に数えず、幅と mode と URL と UNSETTLED の行にして続け、まだ分からない 計 に
/// 数える。home が残る組は節点の頁を開かず、id の無い節点の頁の URL をまだ分からないの行にする。xtask の accept は
/// まだ分からない画面が在れば違反の数に依らず rc 2。
#[test]
fn bvsettle_sweep_counts_unknown() {
    let stalled = format!("{BOARD}?win=stalled&mode=beginner");
    let mut page = stuck_at(stalled.clone());
    let (report, total, unknown) = sweep(&mut page, BOARD, &vocab()).expect("sweep");
    assert_eq!((total, unknown), (0, 4));
    for width in WIDTHS {
        let gone = format!("{width} beginner {stalled} {UNSETTLED}");
        assert_eq!(
            report.lines().filter(|l| **l == gone).count(),
            1,
            "{report}"
        );
        let zero = line(width, "beginner", &stalled, &[0; 12]);
        assert!(!report.lines().any(|l| l == zero), "{report}");
    }
    assert!(
        report.contains("\n違反 計 0\nまだ分からない 計 4\n"),
        "{report}"
    );
    assert_eq!(report.lines().count(), 1 + 112 + 3);

    assert_eq!(SCREENS[0], "?");
    let mut page = stuck_at(format!("{BOARD}?mode=expert"));
    let (report, total, unknown) = sweep(&mut page, BOARD, &vocab()).expect("sweep");
    assert_eq!((total, unknown), (0, 8));
    let nodes: Vec<&str> = page
        .navigated()
        .into_iter()
        .filter(|u| u.contains("page=node"))
        .collect();
    assert_eq!(nodes.len(), WIDTHS.len() * MODES.len() - 4);
    assert!(
        nodes.iter().all(|u| u.ends_with("&mode=beginner")),
        "{nodes:?}"
    );
    let blind = format!(" expert {BOARD}?page=node&mode=expert {UNSETTLED}");
    assert_eq!(report.lines().filter(|l| l.ends_with(&blind)).count(), 4);
    assert_eq!(page.navigated().len(), 112 - 4);
    accept_unknown_rc();
}

/// xtask の accept: fn sweep は audit の sweep のまだ分からない画面の数をそのまま返し、fn run の match は 0 の腕の後の
/// まだ分からない腕で、標準誤りの 1 文（違反の計とまだ分からない画面の数）の他に枝を持たず、違反の数に依らず 2 を返す。
fn accept_unknown_rc() {
    let accept = read("xtask/src/accept.rs");
    let sweep = cut(
        &accept,
        "fn sweep(flags: &Flags, root: &Path) -> Result<(usize, usize), String> {\n",
        "\n}\n",
    );
    assert!(
        sweep
            .contains("let (report, total, unknown) = audit::sweep(&mut session, board, &vocab)?;")
            && sweep.ends_with("Ok((total, unknown))")
            && sweep.matches("unknown").count() == 2,
        "{sweep}"
    );
    let run = cut(
        &accept,
        "pub fn run(args: &[String], root: &Path) -> i32 {\n",
        "\n}\n",
    );
    let arms = cut(&run, "match sweep(&flags, root) {", "}Err(e) => {");
    let (zero, unknown) = arms
        .strip_prefix("match sweep(&flags, root) {Ok((total, 0)) => {")
        .and_then(|a| a.split_once("}Ok((total, unknown)) => {"))
        .expect("0 の腕のすぐ次にまだ分からない腕");
    let say = unknown
        .strip_prefix("emit_err(")
        .and_then(|a| a.strip_suffix(");2"))
        .expect("まだ分からない腕は 1 文の後に 2");
    assert!(
        !zero.contains("Ok((")
            && !say.contains(';')
            && say.contains("まだ分からない画面 {unknown}"),
        "{arms}"
    );
}

/// (5) 今の選び（在りかの now が真）は押さず、今でない選びは押して URL を比べる。URL の替わらない今でない選びは、
/// 今の選びと同じく条 10 の違反に数える。now の欠けと真偽でない値は Err（字は switch_at と URL を含む）。
#[test]
fn bvsettle_current_not_pressed() {
    let url = format!("{BOARD}?board=account&tab=session&mode=beginner");
    let moved = format!("{BOARD}?board=account&tab=home&mode=beginner");
    let url_rule = RULES.iter().position(|(k, _)| *k == "url").expect("条 url");
    let mut counts = Vec::new();
    for now in ["true", "false"] {
        let text = with_spots(&format!(
            r#"{{"label": "session", "now": {now}, "x": 1, "y": 1}}, {{"label": "7d", "now": false, "x": 2, "y": 2}}, {{"label": "home", "now": false, "x": 3, "y": 3}}"#
        ));
        let mut page = Fake::new(move |_, _, _| text.clone());
        page.moves.push((3, 3, moved.clone()));
        let facts = case(&mut page, &url, 960).expect("case");
        let pressed: Vec<&str> = facts.switches.iter().map(|s| s.label.as_str()).collect();
        let clicked = page.times(&format!("{:?}", Command::Click { x: 1, y: 1 }));
        let want: (&[&str], usize) = if now == "true" {
            (&["7d", "home"], 0)
        } else {
            (&["session", "7d", "home"], 1)
        };
        assert_eq!((pressed.as_slice(), clicked), want, "now {now}");
        let still = facts.switches.iter().find(|s| s.label == "7d").expect("7d");
        assert_eq!(
            [still.before.as_str(), still.after.as_str()],
            [url.as_str(); 2]
        );
        counts.push(count(&facts, &vocab())[url_rule]);
    }
    assert_eq!(counts, [1, 2]);
    for odd in [
        r#"{"label": "a", "x": 1, "y": 1}"#,
        r#"{"label": "a", "now": "true", "x": 1, "y": 1}"#,
        r#"{"label": "a", "now": 1, "x": 1, "y": 1}"#,
        r#"{"label": "a", "now": null, "x": 1, "y": 1}"#,
    ] {
        let text = with_spots(odd);
        let mut page = Fake::new(move |_, _, _| text.clone());
        let err = case(&mut page, &url, 960).expect_err("now の形の違い");
        assert!(
            err.contains("switch_at") && err.contains(&url),
            "{odd}: {err}"
        );
    }
}

/// (6) 測りの式は切り替えの在りかに今の選びの印 now を返し、印は aria-selected と aria-pressed の真・aria-current
/// （false でない）・class on。面の src の今の印はこの一覧の中だけ（aria-pressed は 6 file の 7 か所で、値は比べの真偽の
/// 字・aria-current と class on は account board の tab の link だけ・aria-selected と role tab は無い）で、面の
/// 読みの印の字は audit の NOT_READ と同じ字を見える span に出し、面の body は描く前に空。
#[test]
fn bvsettle_probe_marks_current() {
    let probe = read("crates/tsuzuri-boundary/src/stage/measure.js");
    assert_eq!(
        cut(&probe, "  const current = ", ";\n"),
        "const current = \"[aria-selected=true], [aria-pressed=true], [aria-current]:not([aria-current=false]), .on\""
    );
    // 在りかの鎖は頭（seen の切り替え）から尾（窓の中の点）まで、now を e.matches(current) の 1 所だけで決める。
    assert_eq!(
        cut(&probe, "  const switch_at = ", ";\n"),
        concat!(
            "const switch_at = seen",
            ".filter((e) => e.matches(\"button.seg, .seg button, [role=tab], [aria-pressed], [data-tab]\"))",
            ".map((e) => ({ label: words(e.textContent), now: e.matches(current), r: e.getBoundingClientRect() }))",
            ".map(({ label, now, r }) => ({ label, now, x: Math.round(r.left + r.width / 2), y: Math.round(r.top + r.height / 2) }))",
            ".filter(({ x, y }) => x >= 0 && y >= 0 && x < vw && y < vh)",
        )
    );
    let pressed = surface_pressed();
    assert_eq!(
        pressed.join(" "),
        "account/board.rs:1 account/ledger.rs:1 account/projects.rs:1 account/session.rs:2 \
         project/nodearound.rs:1 project_dom/seat.rs:1"
    );
    let board = read("crates/tsuzuri-surface/src/account/board.rs");
    assert!(board.contains("let current = move || (tab.get() == l.tab).then_some(\"page\");"));
    assert!(board.contains("class=class aria-current=current on:click=press data-tab=l.tab.id()"));
    let account = read("crates/tsuzuri-surface/src/account/mod.rs");
    assert!(account.contains("class: if tab == current { \"on\" } else { \"\" },"));
    surface_unread_shown();
}

/// 面の kit: 口をまだ読んでいない block の中身は NOT_READ の理由で、測れていないの 1 行は理由を見える span に出す
/// （包みの div も span も隠す属性を持たない）。面の index.html は head の後に空の body だけを持つ（描く前に見える字は無い）。
fn surface_unread_shown() {
    let kit = read("crates/tsuzuri-surface/src/kit.rs");
    assert!(kit.contains(&format!("pub const NOT_READ: &str = \"{NOT_READ}\";")));
    let pending = cut(
        &kit,
        "pub fn pending(fetched: &Fetched, unread: &'static str) -> Body<()> {\n",
        "\n}\n",
    );
    assert!(
        pending.contains("Fetched::NotRead => NOT_READ,"),
        "{pending}"
    );
    let body = cut(
        &kit,
        "    pub fn body_view(body: Body<()>) -> AnyView {\n",
        "\n    }\n",
    );
    assert!(
        body.contains("Body::Unmeasured(reason) => unmeasured(reason),"),
        "{body}"
    );
    assert_eq!(
        cut(
            &kit,
            "    pub fn unmeasured(reason: &'static str) -> AnyView {\n",
            "\n    }\n"
        ),
        concat!(
            "pub fn unmeasured(reason: &'static str) -> AnyView {view! {<div class=\"empty\">",
            "{state_icon(UNKNOWN)}<span>{label(state_key(UNKNOWN))}</span>",
            "<span class=\"small muted\">{reason}</span></div>}.into_any()",
        )
    );
    let index = read("crates/tsuzuri-surface/index.html");
    let after = &index[index.find("</head>").expect("head の閉じ")..];
    assert_eq!(
        after.split_whitespace().collect::<String>(),
        "</head><body></body></html>",
        "描く前の body は空"
    );
}

/// 面の crate の src の .rs の今の印を照らし、aria-pressed を持つ file と数の字（file:数 を名の順に）を返す。
fn surface_pressed() -> Vec<String> {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("../tsuzuri-surface/src");
    let (mut dirs, mut pressed) = (vec![src.clone()], Vec::new());
    while let Some(dir) = dirs.pop() {
        for path in fs::read_dir(&dir)
            .expect("src の dir")
            .map(|e| e.expect("dir の項").path())
        {
            if path.is_dir() {
                dirs.push(path);
            } else if path.extension().is_some_and(|x| x == "rs") {
                let rel = path
                    .strip_prefix(&src)
                    .expect("src の下")
                    .to_string_lossy()
                    .into_owned();
                let n = file_marks(&rel, &fs::read_to_string(&path).expect("src を読む"));
                if n > 0 {
                    pressed.push(format!("{rel}:{n}"));
                }
            }
        }
    }
    pressed.sort();
    pressed
}

/// 1 つの file の今の印（aria-selected と role tab は無い・aria-pressed はどれも値の pressed を .to_string() で
/// 終わる閉包で組む・aria-current と data-tab は account/board.rs だけ）を照らし、aria-pressed の数を返す。
fn file_marks(rel: &str, text: &str) -> usize {
    for marker in ["aria-selected", "role=\"tab\"", "role=tab"] {
        assert!(!text.contains(marker), "{rel} に {marker}");
    }
    let n = text.matches("aria-pressed=").count();
    let lets: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with("let pressed = move || "))
        .collect();
    assert_eq!(
        (text.matches("aria-pressed=pressed ").count(), lets.len()),
        (n, n),
        "{rel}"
    );
    // 閉包の中身は、信号の今の値（get か with の読み）と項の値の比べの真偽の 1 つ（決まった字でない）。
    let compares = |l: &&str| {
        l.strip_prefix("let pressed = move || ")
            .and_then(|c| c.strip_suffix(".to_string();"))
            .is_some_and(|c| {
                c.matches(" == ").count() == 1 && (c.contains(".get()") || c.contains(".with("))
            })
    };
    assert!(lets.iter().all(compares), "{rel}: {lets:?}");
    let tabs = text.matches("aria-current=").count() + text.matches("data-tab=").count();
    assert_eq!(tabs > 0, rel == "account/board.rs", "{rel}");
    n
}

/// (8) 測りの式は台帳の字の印（data-ledger-text）の下の字を散文と文の予算に、表の行を開いた中（class c-more）の字を
/// 散文に数えず、外した置き場を skipped に名指す。印の無い同じ字は今までどおり数え、外した置き場の数は report の
/// 画面の行の末と 外した置き場 計 の行に出る。skipped の項の頭が ledger か pair でなければ Err。
#[test]
fn bvsettle_skips_named_places() {
    probe_skips_named(&read("crates/tsuzuri-boundary/src/stage/measure.js"));
    let long = "台帳の字・長い問い・続き".repeat(31);
    let plain = clean().replace("\"P-1 は 12:30Z に決めた。\"", &format!("\"{long}\""));
    let marked = clean().replace(
        r#""skipped": []"#,
        r#""skipped": ["ledger div.qb", "pair div.c-more", "pair div.c-more"]"#,
    );
    let vocab = vocab();
    let got = facts(&plain).expect("印の無い字");
    let at = |key: &str| RULES.iter().position(|(k, _)| *k == key).expect("条");
    let counts = count(&got, &vocab);
    assert_eq!((counts[at("prose")], counts[at("budget")]), (1, 1));
    let got = facts(&marked).expect("印の在る字");
    assert_eq!(
        (count(&got, &vocab), skipped_counts(&got)),
        ([0; 12], [1, 2, 0])
    );
    for odd in [r#"["other x"]"#, r#"["ledger"]"#, "[1]"] {
        let text = clean().replace(r#""skipped": []"#, &format!(r#""skipped": {odd}"#));
        let err = facts(&text).expect_err("skipped の形の違い");
        assert!(err.contains("skipped"), "{odd}: {err}");
    }
    let stalled = format!("{BOARD}?win=stalled&mode=beginner");
    let ask = format!("{BOARD}?win=ask&mode=beginner");
    let (a, b) = (stalled.clone(), ask.clone());
    let mut page = Fake::new(move |url, _, _| match url {
        u if u == a => marked.clone(),
        u if u == b => plain.clone(),
        _ => clean(),
    });
    let (report, total, _) = sweep(&mut page, BOARD, &vocab).expect("sweep");
    assert_eq!(total, 8);
    for width in WIDTHS {
        let skip = format!(
            "{} 台帳の字 1 語と値の対 2 見えない要素 0",
            line(width, "beginner", &stalled, &[0; 12])
        );
        assert_eq!(report.lines().filter(|l| *l == skip).count(), 1, "{report}");
        let mut two = [0; 12];
        (two[at("prose")], two[at("budget")]) = (1, 1);
        assert!(
            report
                .lines()
                .any(|l| l == line(width, "beginner", &ask, &two))
        );
    }
    assert!(
        report.ends_with("外した置き場 計 台帳の字 4 語と値の対 8 見えない要素 0\n"),
        "{report}"
    );
}

/// 測りの式の外しの字: 印の選びの 2 つ・題の外し・返す欄、skipped の鎖は頭（seen の印の要素）から尾まで、散文の walk は
/// 字の直の親の要素で外しを判じる。
fn probe_skips_named(probe: &str) {
    for want in [
        "  const ledger = \"[data-ledger-text]\";\n  const pairs = \".c-more\";\n",
        "e.matches(\"[data-t]\") && !e.closest(ledger))",
        "    reach,\n    skipped,\n",
    ] {
        assert_eq!(probe.matches(want).count(), 1, "measure.js の {want}");
    }
    assert_eq!(
        cut(probe, "  const skipped = ", ";\n"),
        concat!(
            "const skipped = seen.filter((e) => e.matches(ledger)).map((e) => \"ledger \" + name(e))",
            ".concat(seen.filter((e) => e.matches(pairs)).map((e) => \"pair \" + name(e)))",
            ".concat(unseen.map((e) => \"hidden \" + name(e)))",
        )
    );
    assert_eq!(
        cut(
            probe,
            "  for (let t = walk.nextNode(); t; t = walk.nextNode()) {\n",
            "\n  }\n"
        ),
        concat!(
            "for (let t = walk.nextNode(); t; t = walk.nextNode()) {",
            "const p = t.parentElement;const piece = words(t.nodeValue);",
            "if (!piece || !p || !shown(p) || p.closest(skip) || p.closest(ledger) || p.closest(pairs)) continue;",
            "if (p.getBoundingClientRect().top < vh) first.push(piece);",
        )
    );
}

/// (9) 測りの式は、箱を持っても描かれない要素（checkVisibility が偽・閉じた details の中・祖先の display none・
/// visibility hidden）をどの条の要素の列（seen と散文の片）にも入れず、skipped に hidden の名で返す。hidden の項は
/// どの条の数も替えず（見える同じ要素は今までどおり数える）、report の画面の行の末と 外した置き場 計 の行に出る。
#[test]
fn bvsettle_skips_unseen_elements() {
    let probe = read("crates/tsuzuri-boundary/src/stage/measure.js");
    for want in [
        "  const drawn = (e) => e.checkVisibility({ visibilityProperty: true });\n  const shown = (e) => boxed(e) && drawn(e);\n",
        "  const unseen = Array.from(root.querySelectorAll(\"*\")).filter((e) => boxed(e) && !drawn(e));\n",
        "    .concat(unseen.map((e) => \"hidden \" + name(e)));\n",
        "if (!piece || !p || !shown(p) ||",
    ] {
        assert_eq!(probe.matches(want).count(), 1, "measure.js の {want}");
    }
    assert_eq!(
        probe.matches("filter(shown)").count(),
        1,
        "要素の列は seen の 1 つ"
    );
    probe_lists_from_seen(&probe);
    let vocab = vocab();
    let at = |key: &str| RULES.iter().position(|(k, _)| *k == key).expect("条");
    let pair = r#""overlap": ["button.btn × span.ttl"]"#;
    let hidden = r#""skipped": ["hidden button.btn", "hidden span.ttl", "hidden a.lk"]"#;
    let seen = clean().replace(r#""overlap": []"#, pair);
    let both = seen.replace(r#""skipped": []"#, hidden);
    let only = clean().replace(r#""skipped": []"#, hidden);
    let mut one = [0; 12];
    one[at("overlap")] = 1;
    for (text, counts, skipped) in [
        (&seen, one, [0, 0, 0]),
        (&both, one, [0, 0, 3]),
        (&only, [0; 12], [0, 0, 3]),
    ] {
        let got = facts(text).expect("事実");
        assert_eq!(
            (count(&got, &vocab), skipped_counts(&got)),
            (counts, skipped)
        );
    }
    let ask = format!("{BOARD}?win=ask&mode=beginner");
    let a = ask.clone();
    let mut page = Fake::new(move |url, _, _| if url == a { only.clone() } else { clean() });
    let (report, total, _) = sweep(&mut page, BOARD, &vocab).expect("sweep");
    assert_eq!(total, 0, "{report}");
    for width in WIDTHS {
        let want = format!(
            "{} 台帳の字 0 語と値の対 0 見えない要素 3",
            line(width, "beginner", &ask, &[0; 12])
        );
        assert_eq!(report.lines().filter(|l| *l == want).count(), 1, "{report}");
    }
    assert!(
        report.ends_with("外した置き場 計 台帳の字 0 語と値の対 0 見えない要素 12\n"),
        "{report}"
    );
}

/// 測りの式の条ごとの要素の列（はみ出し・重なりの部品・吹き出しの口・節点の受け手・見出し・題・節点の id・切り替えの
/// 在りか・外した置き場）はどれも seen から作り、要素を全部拾う querySelectorAll の * は seen と unseen の 2 所だけ。
fn probe_lists_from_seen(probe: &str) {
    for list in [
        "overflow",
        "parts",
        "nopop",
        "reach",
        "headings",
        "titles",
        "nodes",
        "switch_at",
        "skipped",
    ] {
        let head = format!("  const {list} = ");
        let chain = cut(probe, &head, ";\n");
        assert!(
            chain.starts_with(&format!("const {list} = seen.")),
            "{chain}"
        );
    }
    assert_eq!(
        cut(probe, "  const seen = ", ";\n"),
        "const seen = Array.from(root.querySelectorAll(\"*\")).filter(shown)"
    );
    assert_eq!(probe.matches("querySelectorAll(\"*\")").count(), 2);
}

/// (10) report は違反の在る画面の行の下に、数えた違反ごとに 2 つの空白と条の鍵と空白と要素の名か字の 1 行を持つ
/// （details・行の数は count の和と同じ・error は字のまま・違反の無い画面は行を足さない）。
#[test]
fn bvsettle_report_names_items() {
    let vocab = vocab();
    for (i, (key, _)) in RULES.iter().enumerate() {
        let text = read(&format!(
            "tests/fixtures/surface/accept-12/{:02}-{key}.json",
            i + 1
        ));
        let got = facts(&text).expect("事実");
        let (lines, counts) = (details(&got, &vocab), count(&got, &vocab));
        assert_eq!(
            lines.len(),
            counts.iter().sum::<usize>(),
            "{key}: {lines:?}"
        );
        let head = format!("{key} ");
        assert_eq!(
            lines.iter().filter(|l| l.starts_with(&head)).count(),
            counts[i],
            "{key}"
        );
        assert!(counts[i] > 0, "{key}");
    }
    assert!(details(&facts(&clean()).expect("clean"), &vocab).is_empty());
    let reach = clean().replace(
        r#""reach": []"#,
        r#""reach": [{"name": "a.on", "up": ["pointerenter"]}, {"name": "a.off", "up": [""]}]"#,
    );
    let got = facts(&reach).expect("reach");
    assert_eq!(details(&got, &vocab), ["hover a.off"]);
    let text = clean().replace(r#""overflow": []"#, r#""overflow": ["div.saxis", "pre.x"]"#);
    let mut got = facts(&text).expect("事実");
    got.errors = vec!["Failed to load resource\nline 2".to_string()];
    let want = [
        "overflow div.saxis",
        "overflow pre.x",
        "error Failed to load resource line 2",
    ];
    assert_eq!(details(&got, &vocab), want);
    let legend = format!("{BOARD}?win=legend&mode=beginner");
    let a = legend.clone();
    let mut page = Fake::new(move |url, _, _| if url == a { text.clone() } else { clean() });
    let (report, total, _) = sweep(&mut page, BOARD, &vocab).expect("sweep");
    assert_eq!(total, 8, "{report}");
    let mut two = [0; 12];
    two[0] = 2;
    for width in WIDTHS {
        let block = format!(
            "{}\n  overflow div.saxis\n  overflow pre.x\n",
            line(width, "beginner", &legend, &two)
        );
        assert_eq!(report.matches(&block).count(), 1, "{report}");
    }
    assert_eq!(report.lines().filter(|l| l.starts_with("  ")).count(), 8);
}
