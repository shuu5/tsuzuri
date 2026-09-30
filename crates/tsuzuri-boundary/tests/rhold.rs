//! 最後に読めた台帳の字の持ち回しの歯（接頭辞 rhold_・設計ノート surface-wave9 行 e-hold の完了の条件）。
//! 偽の bd は歯ごとの作業場の sh の script で、落とす印の file（down）が在れば何も出さずに rc 1、
//! 無ければ置いた字の file（out.json）を cat する（眠らない）。

use std::fs;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use tsuzuri_boundary::acct::Acct;
use tsuzuri_boundary::server::ledger::{Got, READ_HOLD, Source, parse_bd};
use tsuzuri_boundary::server::seat::HOLD;
use tsuzuri_boundary::server::{Config, Server};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::{READ_AGE_HEADER, READ_HOLD_S, READ_WARN_S};
use tsuzuri_contract::stats::LedgerStats;
use tsuzuri_contract::wire;

/// 歯が渡す持ち回しの上限。
const SHORT: Duration = Duration::from_secs(4);

/// 上限を 2 秒越える待ち（前の got が戻った時から）。
const PAST: Duration = Duration::from_secs(6);

/// 電文に渡す時刻。
const NOW: u64 = 1_790_500_000;

/// created_at と updated_at を持たない bead 3 つの台帳（歯 kcli の LEDGER と同じ形）。
const BARE: &str = r#"[
{"id": "k", "title": "根", "status": "open", "issue_type": "epic"},
{"id": "k.1", "title": "契約", "status": "open", "issue_type": "task",
 "dependencies": [{"issue_id": "k.1", "depends_on_id": "k", "type": "parent-child"}]},
{"id": "k.2", "title": "問い", "status": "open", "issue_type": "task", "labels": ["intake:question"],
 "dependencies": [{"issue_id": "k.2", "depends_on_id": "k", "type": "parent-child"}]}
]
"#;

/// 着地済みの行と第 3 波から第 9 波の行の verify の語（歯の名が含んではならない部分の字）。
const WORDS: [&str; 110] = [
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
    "qgate_",
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
    "qkey_",
    "wstrip_",
    "flight_",
];

fn manifest(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("{} を読めない: {e}", path.display()))
}

/// 読める台帳の字（bead 8 本・fx-hub.3 は問い）。
fn fixture() -> String {
    read(&manifest("../../tests/fixtures/ledger/bd-list-8.json"))
}

/// 書いてから名を移す（撃たれている script や読まれている file を書きかけで見せない）。
fn put(path: &Path, text: &str, mode: u32) {
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, text).expect("file を書く");
    fs::set_permissions(&tmp, fs::Permissions::from_mode(mode)).expect("file の権限");
    fs::rename(&tmp, path).expect("file を移す");
}

/// 歯ごとの作業場（repo・面の file・偽の bd と、その落とす印と置いた字）。
struct Place {
    root: PathBuf,
    repo: PathBuf,
    files: PathBuf,
}

impl Place {
    /// 偽の bd が `text` を出す作業場。
    fn new(name: &str, text: &str) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("rhold")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        let (repo, files) = (root.join("repo"), root.join("files"));
        fs::create_dir_all(repo.join(".beads")).expect("repo の置き場");
        fs::create_dir_all(&files).expect("面の file の置き場");
        fs::write(files.join("index.html"), "tz").expect("index.html");
        fs::write(repo.join(".beads/issues.jsonl"), "{}\n").expect("印の file");
        let place = Place { root, repo, files };
        place.bd_returns(text);
        place.bd_cats();
        place
    }

    fn bd(&self) -> PathBuf {
        self.root.join("bd")
    }

    /// 偽の bd を、落とす印が在れば rc 1・無ければ置いた字を出す script にする。
    fn bd_cats(&self) {
        let root = self.root.display();
        put(
            &self.bd(),
            &format!("#!/bin/sh\nif [ -e '{root}/down' ]; then exit 1; fi\nexec cat '{root}/out.json'\n"),
            0o755,
        );
    }

    /// 偽の bd を、rc 0 で JSON として読めない字（not json）を出す script にする。
    fn bd_garbles(&self) {
        put(&self.bd(), "#!/bin/sh\nprintf 'not json\\n'\n", 0o755);
    }

    fn bd_returns(&self, text: &str) {
        put(&self.root.join("out.json"), text, 0o644);
    }

    /// 偽の bd を落とす（落とす印の file を置く）。
    fn down(&self) {
        fs::write(self.root.join("down"), "").expect("落とす印");
    }

    /// 偽の bd を戻す（落とす印の file を消す）。
    fn up(&self) {
        fs::remove_file(self.root.join("down")).expect("落とす印を消す");
    }

    fn source(&self) -> Source {
        Source::new(self.repo.clone(), self.bd())
    }

    /// 印の file（issues.jsonl）に 1 行を足す（変化の見張りに読ませる）。
    fn touch(&self) {
        let path = self.repo.join(".beads/issues.jsonl");
        fs::write(&path, format!("{}{{}}\n", read(&path))).expect("印の file");
    }
}

/// `from` から `wait` の後まで眠る。
fn sleep_until(from: Instant, wait: Duration) {
    thread::sleep((from + wait).saturating_duration_since(Instant::now()));
}

#[test]
fn rhold_values_and_header_name() {
    assert_eq!(READ_AGE_HEADER, "X-Tz-Read-Age");
    assert_eq!(READ_HOLD_S, 60);
    assert_eq!(READ_WARN_S, 15);
    assert_eq!(READ_HOLD, Duration::from_secs(60));
    let source = Source::new("/r", "bd");
    assert_eq!(source.hold(), READ_HOLD);
    assert_eq!(source.with_hold(SHORT).hold(), SHORT);
}

#[test]
fn rhold_source_holds_then_gives_up() {
    let place = Place::new("holds", &fixture());
    let source = place.source().with_hold(SHORT);
    let before = Instant::now();
    let good = source.got();
    let after = Instant::now();
    assert_eq!(
        good,
        Got {
            text: Some(fixture().into()),
            stale: None
        }
    );
    // 落とした直後は前の字と、前に読めた時刻。
    place.down();
    let held = source.got();
    assert_eq!(held.text, good.text);
    let at = held.stale.expect("落ちた読みの時刻");
    assert!(before <= at && at <= after, "{at:?} not in {before:?}..={after:?}");
    assert_eq!(source.text().as_deref(), good.text.as_deref());
    assert_eq!(source.read(), Reading::Unknown);
    // 上限を 2 秒越えれば字は無い（時刻は同じ）。
    sleep_until(after, PAST);
    assert_eq!(
        source.got(),
        Got {
            text: None,
            stale: Some(at)
        }
    );
    assert_eq!(source.read(), Reading::Unknown);
    // 戻せば新しい読み。
    place.up();
    assert_eq!(
        source.got(),
        Got {
            text: Some(fixture().into()),
            stale: None
        }
    );
}

#[test]
fn rhold_never_read_has_no_text() {
    let place = Place::new("never", &fixture());
    place.down();
    let before = Instant::now();
    let source = place.source();
    let got = source.got();
    let after = Instant::now();
    assert_eq!(got.text, None);
    let at = got.stale.expect("落ちた読みの時刻");
    assert!(before <= at && at <= after, "{at:?} not in {before:?}..={after:?}");
    assert_eq!(source.text(), None);
    assert_eq!(source.read(), Reading::Unknown);
    // rc 0 の読めない字も、一度も読めていなければ字は無い。
    let place = Place::new("never-garbled", &fixture());
    place.bd_garbles();
    let source = place.source();
    assert_eq!(source.got().text, None);
    assert_eq!(source.text(), None);
    assert_eq!(source.read(), Reading::Unknown);
}

#[test]
fn rhold_unparsed_is_a_failure() {
    assert_eq!(parse_bd(BARE), Reading::Unknown);
    assert!(matches!(
        tsuzuri_core::ledger::stats(BARE, 0),
        Reading::Known(_)
    ));
    let place = Place::new("unparsed", BARE);
    let source = place.source().with_hold(SHORT);
    let before = Instant::now();
    let good = source.got();
    let after = Instant::now();
    assert_eq!(
        good,
        Got {
            text: Some(BARE.into()),
            stale: None
        }
    );
    // rc 0 の読めない字は落ちた読み（前の字を返し、読めない字は返さない）。
    place.bd_garbles();
    let held = source.got();
    assert_eq!(held.text, good.text);
    let at = held.stale.expect("落ちた読みの時刻");
    assert!(before <= at && at <= after, "{at:?} not in {before:?}..={after:?}");
    assert_eq!(source.read(), Reading::Unknown);
    sleep_until(after, PAST);
    assert_eq!(
        source.got(),
        Got {
            text: None,
            stale: Some(at)
        }
    );
}

struct Reply {
    status: u16,
    head: String,
    body: String,
}

impl Reply {
    /// 頭 X-Tz-Read-Age の値（名の大小は問わない・無ければ None）。
    fn age(&self) -> Option<String> {
        let name = format!("{}:", READ_AGE_HEADER.to_ascii_lowercase());
        self.head.lines().find_map(|l| {
            l.to_ascii_lowercase()
                .starts_with(&name)
                .then(|| l[name.len()..].trim().to_string())
        })
    }
}

fn get(addr: SocketAddr, path: &str) -> Reply {
    let mut s = TcpStream::connect(addr).expect("接続");
    s.set_read_timeout(Some(Duration::from_secs(20)))
        .expect("timeout");
    s.write_all(format!("GET {path} HTTP/1.1\r\nHost: x\r\n\r\n").as_bytes())
        .expect("要求を書く");
    let mut out = Vec::new();
    s.read_to_end(&mut out).expect("応答を読む");
    let text = String::from_utf8(out).expect("応答の字");
    let (head, body) = text.split_once("\r\n\r\n").expect("頭と本文");
    let status = head
        .split(' ')
        .nth(1)
        .and_then(|c| c.parse().ok())
        .expect("状態の code");
    Reply {
        status,
        head: head.to_string(),
        body: body.to_string(),
    }
}

/// 台帳を読む 10 の GET の口。
const ROUTES: [&str; 10] = [
    "/api/ledger",
    "/api/ledger/fx-hub.3",
    "/api/metrics",
    "/api/questions",
    "/api/pipeline",
    "/api/next",
    "/api/graph",
    "/api/graph/view",
    "/api/around?id=fx-hub.3",
    "/api/unreflected",
];

#[test]
fn rhold_routes_hold_and_mark() {
    let place = Place::new("routes", &fixture());
    let config = Config {
        bd: place.bd().into(),
        folio: place.root.join("no-such-folio").into(),
        scribe2: place.root.join("no-such-scribe2").into(),
        state_dir: None,
        ..Config::new(
            place.repo.clone(),
            "127.0.0.1:0".parse().expect("bind 先"),
            place.files.clone(),
        )
    };
    let server = Server::bind(&config).expect("起動");
    let addr = server.local_addr().expect("口の住所");
    thread::spawn(move || server.run());

    let mut before = Vec::new();
    for route in ROUTES {
        let reply = get(addr, route);
        assert_eq!(reply.status, 200, "{route}: {}", reply.body);
        assert_eq!(reply.age(), None, "{route}: {}", reply.head);
        before.push(reply.body);
    }
    // 口は見張りの読みの字を返すので、印を動かして見張りの落ちた読みを待つ（行 e-snap）。
    place.down();
    place.touch();
    let until = Instant::now() + Duration::from_millis(2500);
    while get(addr, "/api/ledger").age().is_none() {
        assert!(Instant::now() < until, "落として 2.5 秒の後も頭が無い");
        thread::sleep(Duration::from_millis(50));
    }
    for (route, was) in ROUTES.iter().zip(&before) {
        let reply = get(addr, route);
        assert_eq!(reply.status, 200, "{route}: {}", reply.body);
        let age = reply
            .age()
            .unwrap_or_else(|| panic!("{route} に頭が無い: {}", reply.head));
        let secs: u64 = age
            .parse()
            .unwrap_or_else(|_| panic!("{route} の頭の値が整数でない: {age}"));
        assert!(secs <= 59, "{route}: {secs}");
        if matches!(*route, "/api/ledger" | "/api/questions") {
            assert_eq!(&reply.body, was, "{route}");
        }
        if *route == "/api/metrics" {
            let stats: Reading<LedgerStats> = wire::decode(&reply.body).expect("指標の形");
            assert!(matches!(stats, Reading::Known(_)), "{}", reply.body);
        }
    }
    // 台帳を読まない口は頭を付けない。
    let project = get(addr, "/api/project");
    assert_eq!(project.status, 200, "{}", project.body);
    assert_eq!(project.age(), None, "{}", project.head);
    // 戻して印を動かせば 5 秒以内に頭の無い 200。
    place.up();
    place.touch();
    let until = Instant::now() + Duration::from_secs(5);
    loop {
        let reply = get(addr, "/api/ledger");
        if reply.status == 200 && reply.age().is_none() {
            break;
        }
        assert!(Instant::now() < until, "戻して 5 秒の後も頭が在る: {}", reply.head);
        thread::sleep(Duration::from_millis(100));
    }
}

#[test]
fn rhold_acct_ledger_held() {
    let place = Place::new("acct", &fixture());
    let anchor = place.root.join("work/proj-x");
    fs::create_dir_all(&anchor).expect("anchor");
    let state = place.root.join("state-h");
    fs::create_dir_all(&state).expect("引数の state dir");
    fs::write(
        state.join("host.toml"),
        format!(
            "[[account-group]]\nname = \"g-x\"\nanchors = [\"{}\"]\n",
            anchor.display()
        ),
    )
    .expect("host.toml");
    let git = place.root.join("git");
    put(
        &git,
        &format!(
            "#!/bin/sh\nif [ \"$5\" = scribe2.statedir ]; then echo '{}'; exit 0; fi\nexit 1\n",
            place.root.join("state-x").display()
        ),
        0o755,
    );
    let scribe2 = place.root.join("scribe2");
    put(&scribe2, "#!/bin/sh\nexit 1\n", 0o755);
    let acct = Acct::new(scribe2, git, place.bd(), state, place.repo.clone());

    let (doc, stale) = acct.doc_read(NOW);
    assert_eq!(stale, None);
    assert_eq!(doc.projects.len(), 1, "{doc:?}");
    assert!(
        matches!(doc.projects[0].ledger, Reading::Known(_)),
        "{doc:?}"
    );
    place.down();
    // 台帳の印が動かなければ前の読みを使い bd を撃たないので、anchor の印の file を置いて読み直させる（行 a-lean）。
    fs::create_dir_all(anchor.join(".beads")).expect("anchor の .beads");
    fs::write(anchor.join(".beads/issues.jsonl"), "").expect("anchor の印の file");
    thread::sleep(HOLD + Duration::from_secs(2));
    let (again, stale) = acct.doc_read(NOW);
    assert_eq!(again, doc);
    let age = stale.expect("落ちた読みの時刻").elapsed();
    assert!(
        age >= Duration::from_secs(5) && age < Duration::from_secs(60),
        "{age:?}"
    );
    assert_eq!(acct.doc(NOW), again);
}

#[test]
fn rhold_wiring_text() {
    for name in [
        "ledger",
        "item",
        "metrics",
        "questions",
        "pipeline",
        "next",
        "graph",
        "graphview",
        "around",
        "unreflected",
        "account",
    ] {
        let text = read(&manifest(&format!("src/server/routes/{name}.rs")));
        assert!(text.contains("aged("), "{name}.rs に aged( が無い");
    }
    let account = read(&manifest("src/server/routes/account.rs"));
    assert!(account.contains("doc_read("), "account.rs に doc_read( が無い");
    let module = read(&manifest("src/server/mod.rs"));
    assert!(module.contains("fn aged("), "mod.rs に fn aged( が無い");
    assert!(module.contains("READ_AGE_HEADER"), "mod.rs に READ_AGE_HEADER が無い");
    let events = read(&manifest("src/server/events.rs"));
    let start = events.find("pub fn start<").expect("pub fn start<");
    let rest = &events[start + "pub fn start<".len()..];
    let body = rest.find("pub fn ").map_or(rest, |end| &rest[..end]);
    assert!(body.contains("source.read()"), "見張りが持ち回さない読みを使わない");
}

#[test]
fn rhold_own_names_clean() {
    let text = read(&manifest("tests/rhold.rs"));
    let mut names = Vec::new();
    let mut lines = text.lines();
    while let Some(line) = lines.next() {
        if line.trim() != "#[test]" {
            continue;
        }
        let decl = lines.next().expect("test の属性の次の行");
        let name = decl
            .trim()
            .strip_prefix("fn ")
            .and_then(|r| r.split('(').next())
            .unwrap_or_else(|| panic!("fn の宣言でない: {decl}"));
        names.push(name.to_string());
    }
    assert_eq!(names.len(), 8, "{names:?}");
    for name in &names {
        let rest = name
            .strip_prefix("rhold_")
            .unwrap_or_else(|| panic!("{name} が rhold_ で始まらない"));
        for word in WORDS {
            assert!(!rest.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
