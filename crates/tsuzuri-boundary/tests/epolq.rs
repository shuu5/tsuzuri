//! 方針の問いの受付の歯（接頭辞 epolq_・設計ノート surface-wave12h 行 e-policy-q の完了の条件）。
//! 偽の bd は作業場の out.json を標準出力へ出す script、偽の bdw と偽の器は撃たれた回ごとの argv
//! （語ごとに NUL で終える・本文の改行を語の中に保つ）と cwd を記録の置き場に書き、落とさない回で
//! 最初の引数が `create` なら `<log>/<名>.created` の字（無ければ `fx-p.7` と改行）を出す script。
//! 受付は `policy::accept` を受付の時刻 `NOW` で直に撃つ（口の本文を見る歯だけが server を立てる）。
#![cfg(test)]

use std::fs;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tsuzuri_boundary::server::ledger::Source;
use tsuzuri_boundary::server::policy::{self, Outcome};
use tsuzuri_boundary::server::ruling::{Delivery, Writer, minute};
use tsuzuri_boundary::server::{Config, Server};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::graph::{EdgeType, NodeKind};
use tsuzuri_contract::ledger::{
    BeadId, ChildType, Effect, LedgerRow, LedgerWrite, POLICY_SCOPE_LABEL, QUESTION_LABEL,
};
use tsuzuri_contract::surface::{PolicyRequest, PolicyResponse, Refusal, RulingId};
use tsuzuri_contract::wire;
use tsuzuri_core::delivery::{Pending, undelivered};
use tsuzuri_core::graph::build::add_summary;
use tsuzuri_core::graph::check::unsummarized;
use tsuzuri_core::graph::{Inputs, build};
use tsuzuri_core::question::{
    ENG_PREFIX, PLAIN_PREFIX, REASON_PREFIX, RECOMMEND_PREFIX, open_questions, typed,
};

/// verify の filter の語（main 67b74e3 の contracts の語を畳んだ 135 語と、並行と後の行の接頭辞 4 語）。
const FILTER_WORDS: [&str; 139] = [
    "aaround_",
    "accept_",
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
    "cgdom_",
    "cmark_",
    "contract_form_",
    "cround_",
    "csled_",
    "denv_",
    "ecache_",
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
    "glabel_",
    "gnav_",
    "gpill_",
    "gpulse_",
    "graph_",
    "gsum_",
    "gtuck_",
    "gview_",
    "hacols_",
    "hbconf_",
    "hbpost_",
    "hbproc_",
    "hbroute_",
    "hcard_",
    "hcled_",
    "hcnx_",
    "hcproj_",
    "hcsess_",
    "hfig_",
    "hnunk_",
    "hook_",
    "hruling_",
    "hsblock_",
    "hsderive_",
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

/// 受付の時刻（2026-09-28T08:41:30Z・分は `MINUTE`）。
const NOW: u64 = 1_790_584_890;
const MINUTE: &str = "20260928T0841Z";

/// 偽の bdw が子を作る書きの回に既定で出す id と、それから発行する方針の id。
const CREATED: &str = "fx-p.7";
const POLICY_ID: &str = "fx-p.7:20260928T0841Z-1";

/// 2 行の逐語（項 2 と項 3）。
const TWO_LINES: &str = "全体に急がない\n2 行目";

const SEAT: &str = "tsuzuri:0.1";
const STAMP: &str = "2026-09-27T01:00:00Z";

/// 台帳の 1 本（歯の中で JSON の配列に組む）。
#[derive(Clone)]
struct Bead {
    id: String,
    title: String,
    kind: String,
    status: String,
    labels: Vec<String>,
    parent: Option<String>,
    description: String,
    notes: String,
}

fn bead(id: &str, title: &str, kind: &str, status: &str, labels: &[&str]) -> Bead {
    Bead {
        id: id.to_string(),
        title: title.to_string(),
        kind: kind.to_string(),
        status: status.to_string(),
        labels: labels.iter().map(|l| l.to_string()).collect(),
        parent: Some("fx-p".to_string()),
        description: String::new(),
        notes: String::new(),
    }
}

fn s(text: &str) -> String {
    wire::encode(&text).expect("字の電文")
}

impl Bead {
    fn json(&self) -> String {
        let labels: Vec<String> = self.labels.iter().map(|l| s(l)).collect();
        let mut out = format!(
            "{{\"id\":{},\"title\":{},\"status\":{},\"priority\":2,\"issue_type\":{},\"created_at\":\"{STAMP}\",\"updated_at\":\"{STAMP}\",\"labels\":[{}]",
            s(&self.id),
            s(&self.title),
            s(&self.status),
            s(&self.kind),
            labels.join(",")
        );
        if let Some(p) = &self.parent {
            out.push_str(&format!(",\"parent\":{}", s(p)));
        }
        if !self.description.is_empty() {
            out.push_str(&format!(",\"description\":{}", s(&self.description)));
        }
        if !self.notes.is_empty() {
            out.push_str(&format!(",\"notes\":{}", s(&self.notes)));
        }
        out.push('}');
        out
    }
}

/// 根 fx-p・方針の memo fx-p.1・open の問い fx-p.2・closed の問い fx-p.3。
fn base() -> Vec<Bead> {
    let mut root = bead("fx-p", "根", "epic", "open", &[]);
    root.parent = None;
    let mut memo = bead("fx-p.1", "方針", "task", "open", &["intake:memo"]);
    memo.description = "全体への指示を受ける memo".into();
    memo.notes = "方針 id = policy:20260927T1034Z-1・範囲 = all・逐語 = 前".into();
    let mut open = bead("fx-p.2", "問い — 色の数", "task", "open", &[QUESTION_LABEL]);
    open.description = "概要 = 画面の色を 2 つに減らしてよいか".into();
    let mut closed = bead(
        "fx-p.3",
        "問い — 答えの済んだ問い",
        "task",
        "closed",
        &[QUESTION_LABEL],
    );
    closed.description = "概要 = 閉じた問い".into();
    closed.notes = "裁定 id = fx-p.3:20260924T0200Z-1・問い = fx-p.3・逐語 = はい".into();
    vec![root, memo, open, closed]
}

fn ledger(beads: &[Bead]) -> String {
    let lines: Vec<String> = beads.iter().map(Bead::json).collect();
    format!("[\n{}\n]\n", lines.join(",\n"))
}

fn script(path: &Path, body: &str) {
    fs::write(path, format!("#!/bin/sh\n{body}\n")).expect("偽の program");
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("偽の program の権限");
}

/// 撃たれた回ごとに argv（語ごとに NUL で終える）と cwd を書き、`<log>/<name>.fail` の回なら rc 1 で終わる script。
/// 落とさない回で最初の引数が `create` なら `<log>/<name>.created` の字（無ければ `fx-p.7` と改行）を出す。
fn recorder(path: &Path, log: &Path, name: &str) {
    let log = log.display();
    script(
        path,
        &format!(
            "n=$(( $(cat '{log}/{name}.count' 2>/dev/null || echo 0) + 1 ))\n\
             echo \"$n\" > '{log}/{name}.count'\n\
             for a in \"$@\"; do printf '%s\\000' \"$a\"; done > '{log}/{name}.'\"$n\"'.args'\n\
             pwd -P > '{log}/{name}.'\"$n\"'.cwd'\n\
             if [ \"$n\" = \"$(cat '{log}/{name}.fail' 2>/dev/null)\" ]; then echo 落ちた >&2; exit 1; fi\n\
             if [ \"$1\" = create ]; then\n\
               if [ -f '{log}/{name}.created' ]; then cat '{log}/{name}.created'; else printf 'fx-p.7\\n'; fi\n\
             fi\n\
             exit 0"
        ),
    );
}

/// 歯ごとの作業場（repo・面の file・state dir・偽の program・記録の置き場）。
struct Place {
    root: PathBuf,
    repo: PathBuf,
    files: PathBuf,
    state: PathBuf,
    log: PathBuf,
}

impl Place {
    fn new(name: &str) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("epolq")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        let (repo, files, state, log) = (
            root.join("repo"),
            root.join("files"),
            root.join("state"),
            root.join("log"),
        );
        fs::create_dir_all(repo.join(".beads")).expect("repo の置き場");
        fs::create_dir_all(&files).expect("面の file の置き場");
        fs::create_dir_all(&state).expect("state dir");
        fs::create_dir_all(&log).expect("記録の置き場");
        fs::write(files.join("index.html"), "tz").expect("index.html");
        script(
            &root.join("bd"),
            &format!("exec cat '{}'", root.join("out.json").display()),
        );
        recorder(&root.join("bdw"), &log, "bdw");
        recorder(&root.join("scribe2"), &log, "scribe2");
        let place = Place {
            root,
            repo,
            files,
            state,
            log,
        };
        place.bd_returns(&ledger(&base()));
        place
    }

    fn bd_returns(&self, text: &str) {
        fs::write(self.root.join("out.json"), text).expect("偽の bd の出力");
    }

    /// 偽の bd を落とす（出す file が無いので rc 1）。
    fn bd_fails(&self) {
        fs::remove_file(self.root.join("out.json")).expect("偽の bd の出力を消す");
    }

    /// 偽の bdw を `n` 回目で落とす。
    fn fail_at(&self, n: u32) {
        fs::write(self.log.join("bdw.fail"), n.to_string()).expect("落とす回");
    }

    /// 偽の bdw が子を作る書きの回に出す字。
    fn created(&self, out: &str) {
        fs::write(self.log.join("bdw.created"), out).expect("作った id の字");
    }

    fn calls(&self, name: &str) -> Vec<(Vec<String>, String)> {
        let count: u32 = fs::read_to_string(self.log.join(format!("{name}.count")))
            .map_or(0, |c| c.trim().parse().expect("回の数"));
        (1..=count)
            .map(|n| {
                let read = |ext: &str| {
                    fs::read_to_string(self.log.join(format!("{name}.{n}.{ext}"))).expect("記録")
                };
                (
                    read("args")
                        .split_terminator('\0')
                        .map(str::to_string)
                        .collect(),
                    read("cwd"),
                )
            })
            .collect()
    }

    fn argvs(&self) -> Vec<Vec<String>> {
        self.calls("bdw").into_iter().map(|(argv, _)| argv).collect()
    }

    fn repo_real(&self) -> String {
        format!(
            "{}\n",
            self.repo.canonicalize().expect("repo の実体").display()
        )
    }

    fn source(&self) -> Source {
        Source::new(self.repo.clone(), self.root.join("bd"))
    }

    fn writer(&self) -> Writer {
        Writer {
            repo: self.repo.clone(),
            bdw: self.root.join("bdw").into(),
            delivery: Some(Delivery {
                program: self.root.join("scribe2").into(),
                state_dir: self.state.clone(),
                target: SEAT.to_string(),
            }),
        }
    }

    fn accept(&self, scope: &str, verbatim: &str) -> Outcome {
        policy::accept(&request(scope, verbatim), &self.source(), &self.writer(), NOW)
    }

    fn serve(&self) -> SocketAddr {
        let config = Config {
            bd: self.root.join("bd").into(),
            state_dir: Some(self.state.clone()),
            bdw: self.root.join("bdw").into(),
            seat: Some(SEAT.to_string()),
            scribe2: self.root.join("scribe2").into(),
            ..Config::new(
                self.repo.clone(),
                "127.0.0.1:0".parse().expect("bind 先"),
                self.files.clone(),
            )
        };
        let server = Server::bind(&config).expect("起動");
        let addr = server.local_addr().expect("口の住所");
        thread::spawn(move || server.run());
        addr
    }
}

fn request(scope: &str, verbatim: &str) -> PolicyRequest {
    PolicyRequest {
        scope: scope.to_string(),
        verbatim: verbatim.to_string(),
    }
}

fn id(s: &str) -> BeadId {
    BeadId::new(s).expect("bead id")
}

fn rid(s: &str) -> RulingId {
    RulingId::new(s).expect("記帳 id")
}

fn recorded(policy: &str) -> Outcome {
    Outcome::Recorded(PolicyResponse {
        policy: rid(policy),
        recorded_at: NOW,
    })
}

/// 項 3 の 3 つの書き（作る・足す・閉じる）の argv。
fn three(scope: &str, verbatim: &str) -> Vec<Vec<String>> {
    let policy_id = rid(POLICY_ID);
    vec![
        policy::create_write(&id("fx-p"), scope, verbatim).argv(),
        LedgerWrite::AppendNotes {
            id: id(CREATED),
            line: policy::line(&policy_id, scope, verbatim),
        }
        .argv(),
        LedgerWrite::CloseItem {
            id: id(CREATED),
            reason: format!("裁定 policy:{POLICY_ID}"),
        }
        .argv(),
    ]
}

fn words(list: &[&str]) -> Vec<String> {
    list.iter().map(|w| w.to_string()).collect()
}

/// 本文の 4 行の頭（最初の字 `=` の前の字）。
fn heads(description: &str) -> Vec<String> {
    description
        .lines()
        .map(|l| l.split_once('=').expect("字 =").0.trim().to_string())
        .collect()
}

fn description(w: &LedgerWrite) -> &str {
    match w {
        LedgerWrite::CreateChild { description, .. } => description,
        other => panic!("子を作る書きでない: {other:?}"),
    }
}

fn labels(w: &LedgerWrite) -> Vec<String> {
    match w {
        LedgerWrite::CreateChild { labels, .. } => labels.clone(),
        other => panic!("子を作る書きでない: {other:?}"),
    }
}

#[test]
fn epolq_create_argv_words() {
    let w = LedgerWrite::CreateChild {
        parent: id("fx-p"),
        title: "方針（範囲 = all）".into(),
        child_type: ChildType::Task,
        description: "d\ne".into(),
        labels: vec![QUESTION_LABEL.into(), "policy-scope:all".into()],
        effect: Some(Effect::Operation),
    };
    let argv = w.argv();
    assert_eq!(
        argv,
        words(&[
            "create",
            "--parent=fx-p",
            "--type=task",
            "--labels=intake:question,policy-scope:all",
            r#"--metadata={"effect":"operation"}"#,
            "--no-inherit-labels=true",
            "--silent=true",
            "--description=d\ne",
            "--",
            "方針（範囲 = all）",
        ])
    );
    // metadata は鍵 effect の 1 対だけの、空白の無い 1 行の JSON の object。
    let meta = argv[4].strip_prefix("--metadata=").expect("metadata の旗");
    assert!(meta.starts_with('{') && !meta.contains(char::is_whitespace), "{meta}");
    let pairs: std::collections::BTreeMap<String, String> =
        wire::decode(meta).expect("metadata の JSON");
    assert_eq!(
        pairs.into_iter().collect::<Vec<_>>(),
        [("effect".to_string(), "operation".to_string())]
    );

    let bare = LedgerWrite::CreateChild {
        parent: id("fx-p"),
        title: "t".into(),
        child_type: ChildType::Epic,
        description: "d".into(),
        labels: vec![],
        effect: None,
    };
    let argv = bare.argv();
    assert_eq!(
        argv,
        words(&[
            "create",
            "--parent=fx-p",
            "--type=epic",
            "--no-inherit-labels=true",
            "--silent=true",
            "--description=d",
            "--",
            "t",
        ])
    );
    assert!(
        !argv
            .iter()
            .any(|a| a.starts_with("--labels=") || a.starts_with("--metadata=")),
        "{argv:?}"
    );
    let LedgerWrite::CreateChild {
        parent,
        title,
        child_type,
        description,
        labels,
        ..
    } = bare
    else {
        panic!("子を足す書きでない: {bare:?}")
    };
    let doc = LedgerWrite::CreateChild {
        parent,
        title,
        child_type,
        description,
        labels,
        effect: Some(Effect::Document),
    };
    let argv = doc.argv();
    assert_eq!(argv.len(), 9, "{argv:?}");
    assert_eq!(argv[3], r#"--metadata={"effect":"document"}"#);
    assert_eq!(
        (Effect::Document.as_str(), Effect::Operation.as_str()),
        ("document", "operation")
    );
}

#[test]
fn epolq_create_form() {
    let w = policy::create_write(&id("fx-p"), "all", TWO_LINES);
    let body = "概要 = 全体に急がない\n技術 = 範囲 = 全体\n理由 = 持ち主の方針（方針の欄）\n推奨 = なし";
    assert_eq!(
        w,
        LedgerWrite::CreateChild {
            parent: id("fx-p"),
            title: "方針（範囲 = all）".into(),
            child_type: ChildType::Task,
            description: body.into(),
            labels: vec![QUESTION_LABEL.into(), "policy-scope:all".into()],
            effect: Some(Effect::Operation),
        }
    );
    assert_eq!(
        w.argv(),
        words(&[
            "create",
            "--parent=fx-p",
            "--type=task",
            "--labels=intake:question,policy-scope:all",
            r#"--metadata={"effect":"operation"}"#,
            "--no-inherit-labels=true",
            "--silent=true",
            &format!("--description={body}"),
            "--",
            "方針（範囲 = all）",
        ])
    );

    let one = policy::create_write(
        &id("fx-p"),
        "fx-p.2",
        "\n \t　\n  この問いには急がない  \r\n次の行",
    );
    let text = description(&one);
    assert_eq!(text.lines().next(), Some("概要 = この問いには急がない"));
    assert_eq!(text.lines().nth(1), Some("技術 = 範囲 = fx-p.2"));
    assert_eq!(labels(&one)[1], "policy-scope:fx-p.2");
    let LedgerWrite::CreateChild { title, .. } = &one else {
        panic!("子を足す書きでない: {one:?}")
    };
    assert_eq!(title, "方針（範囲 = fx-p.2）");

    let raw = "a\\b 裁定 id = x";
    let odd = policy::create_write(&id("fx-p"), "all", raw);
    assert_eq!(
        description(&odd).lines().next(),
        Some(format!("概要 = {raw}").as_str())
    );

    for (w, plain, eng) in [
        (&w, "全体に急がない", "範囲 = 全体"),
        (&one, "この問いには急がない", "範囲 = fx-p.2"),
        (&odd, raw, "範囲 = 全体"),
    ] {
        let text = description(w);
        assert_eq!(heads(text), ["概要", "技術", "理由", "推奨"], "{text}");
        assert_eq!(typed(text, PLAIN_PREFIX).as_deref(), Some(plain));
        assert_eq!(typed(text, ENG_PREFIX).as_deref(), Some(eng));
        assert_eq!(
            typed(text, REASON_PREFIX).as_deref(),
            Some("持ち主の方針（方針の欄）")
        );
        assert_eq!(typed(text, RECOMMEND_PREFIX).as_deref(), Some("なし"));
    }
}

#[test]
fn epolq_all_writes_three() {
    assert_eq!(minute(NOW), MINUTE);
    assert_eq!(
        RulingId::for_question(&id(CREATED), MINUTE, 1).expect("id"),
        rid(POLICY_ID)
    );
    let place = Place::new("all");
    assert_eq!(place.accept("all", TWO_LINES), recorded(POLICY_ID));
    let calls = place.calls("bdw");
    assert_eq!(calls.len(), 3, "{calls:?}");
    let argvs: Vec<Vec<String>> = calls.iter().map(|(a, _)| a.clone()).collect();
    assert_eq!(argvs, three("all", TWO_LINES));
    assert_eq!(
        argvs[1][2],
        format!("--append-notes=方針 id = {POLICY_ID}・範囲 = all・逐語 = 全体に急がない\\n2 行目")
    );
    assert_eq!(argvs[2][2], format!("--reason=裁定 policy:{POLICY_ID}"));
    assert!(
        argvs[0][7].contains('\n'),
        "本文の改行は語の中: {:?}",
        argvs[0][7]
    );
    for (argv, cwd) in &calls {
        assert_eq!(cwd, &place.repo_real(), "cwd は repo の置き場");
        assert!(
            !["fx-p.1", "fx-p.2", "fx-p.3"].contains(&argv[1].as_str()),
            "{argv:?}"
        );
    }
    assert!(place.calls("scribe2").is_empty(), "方針は配達しない");
}

#[test]
fn epolq_scope_one() {
    let place = Place::new("scope");
    let verbatim = "この問いには急がない";
    assert_eq!(place.accept("fx-p.2", verbatim), recorded(POLICY_ID));
    let argvs = place.argvs();
    assert_eq!(argvs, three("fx-p.2", verbatim));
    assert_eq!(argvs[0][3], "--labels=intake:question,policy-scope:fx-p.2");
    assert_eq!(argvs[0][9], "方針（範囲 = fx-p.2）");
    assert_eq!(
        argvs[1][2],
        format!("--append-notes=方針 id = {POLICY_ID}・範囲 = fx-p.2・逐語 = この問いには急がない")
    );
    assert!(argvs.iter().all(|a| a[1] != "fx-p.2"), "{argvs:?}");
    assert!(place.calls("scribe2").is_empty());
}

#[test]
fn epolq_refusals_write_nothing() {
    let place = Place::new("refuse");
    for scope in ["fx-p.3", "fx-p.9", "fx-p.1", "", "ALL", " all"] {
        assert_eq!(place.accept(scope, "はい"), Outcome::BadScope, "{scope:?}");
    }
    for verbatim in ["", " \n\t　"] {
        assert_eq!(
            place.accept("all", verbatim),
            Outcome::Refused(Refusal::EmptyVerbatim),
            "{verbatim:?}"
        );
    }
    let rootless: Vec<Bead> = base().into_iter().filter(|b| b.id != "fx-p").collect();
    let mut closed = base();
    closed[0].status = "closed".into();
    let mut task = base();
    task[0].kind = "task".into();
    let mut child = base();
    child[0].parent = Some("fx-q".into());
    for (label, beads) in [
        ("根が無い", rootless),
        ("根が closed", closed),
        ("根が task", task),
        ("根に親", child),
    ] {
        place.bd_returns(&ledger(&beads));
        assert_eq!(place.accept("all", "はい"), Outcome::NoRoot, "{label}");
    }
    place.bd_returns("not json");
    assert_eq!(place.accept("all", "はい"), Outcome::LedgerUnknown);
    place.bd_fails();
    assert_eq!(place.accept("all", "はい"), Outcome::LedgerUnknown);
    assert!(place.calls("bdw").is_empty(), "断りで偽の bdw を撃つ");
    assert!(place.calls("scribe2").is_empty(), "断りで偽の器を撃つ");

    // 方針の memo が無くても受ける（memo は探さない）。
    let place = Place::new("memoless");
    let memoless: Vec<Bead> = base().into_iter().filter(|b| b.id != "fx-p.1").collect();
    place.bd_returns(&ledger(&memoless));
    assert_eq!(place.accept("all", "はい"), recorded(POLICY_ID));
    assert_eq!(place.argvs(), three("all", "はい"));
}

#[test]
fn epolq_root_pick() {
    let place = Place::new("root");
    let mut beads = base();
    let mut a = bead("fx-a", "閉じた根", "epic", "closed", &[]);
    a.parent = None;
    let mut p10 = bead("fx-p10", "もう 1 つの根", "epic", "open", &[]);
    p10.parent = None;
    beads.extend([a, p10]);
    place.bd_returns(&ledger(&beads));
    assert_eq!(place.accept("all", "はい"), recorded(POLICY_ID));
    let argvs = place.argvs();
    assert_eq!(argvs[0][1], "--parent=fx-p");
    assert_eq!(argvs, three("all", "はい"));
}

#[test]
fn epolq_step_k_of_3() {
    let full = three("all", TWO_LINES);
    for step in 1..=3u32 {
        let place = Place::new(&format!("step-{step}"));
        place.fail_at(step);
        let want = match step {
            1 => Outcome::CreateFailed,
            2 => Outcome::AppendFailed(id(CREATED)),
            _ => Outcome::CloseFailed(rid(POLICY_ID)),
        };
        assert_eq!(place.accept("all", TWO_LINES), want, "{step}");
        let argvs = place.argvs();
        assert_eq!(argvs.len(), step as usize, "{step}: {argvs:?}");
        assert_eq!(argvs[..], full[..step as usize], "{step}");
        assert!(place.calls("scribe2").is_empty(), "{step}");
    }
}

#[test]
fn epolq_created_id_shapes() {
    for (n, out) in [
        "",
        "\n",
        "fx-p.7 fx-p.8\n",
        "-x\n",
        "fx:p\n",
        "方針\n",
    ]
    .into_iter()
    .enumerate()
    {
        let place = Place::new(&format!("created-bad-{n}"));
        place.created(out);
        assert_eq!(
            place.accept("all", "はい"),
            Outcome::CreateFailed,
            "{out:?}"
        );
        assert_eq!(place.calls("bdw").len(), 1, "{out:?}");
    }
    let place = Place::new("created-spaced");
    place.created("  fx-p.7\n\n");
    assert_eq!(place.accept("all", "はい"), recorded(POLICY_ID));

    let long47 = format!("fx-p.{}", "7".repeat(42));
    assert_eq!(long47.len(), 47);
    let place = Place::new("created-47");
    place.created(&format!("{long47}\n"));
    let Outcome::Recorded(got) = place.accept("all", "はい") else {
        panic!("47 byte の id で書かない");
    };
    assert_eq!(got.policy.as_str().len(), 64, "{}", got.policy);
    assert_eq!(got.policy.as_str(), format!("{long47}:{MINUTE}-1"));

    let long48 = format!("fx-p.{}", "7".repeat(43));
    assert_eq!(long48.len(), 48);
    let place = Place::new("created-48");
    place.created(&format!("{long48}\n"));
    assert_eq!(
        place.accept("all", "はい"),
        Outcome::IdShape(id(&long48))
    );
    assert_eq!(place.calls("bdw").len(), 1);
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("時計")
        .as_secs()
}

/// POST /api/policy を Origin の頭を持たずに撃ち、接続が閉じるまで応答を読む（状態の code と本文）。
fn post(addr: SocketAddr, scope: &str, verbatim: &str) -> (u16, String) {
    let body = wire::encode(&request(scope, verbatim)).expect("要求の電文");
    let mut s = TcpStream::connect(addr).expect("接続");
    s.set_read_timeout(Some(Duration::from_secs(20)))
        .expect("timeout");
    s.write_all(
        format!(
            "POST {} HTTP/1.1\r\nHost: {addr}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
            policy::PATH,
            body.len()
        )
        .as_bytes(),
    )
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
    (status, body.to_string())
}

/// 撃つ前と後の時刻の分で発行しうる方針の id。
fn policy_ids(created: &str, from: u64, to: u64) -> [String; 2] {
    [from, to].map(|t| {
        RulingId::for_question(&id(created), &minute(t), 1)
            .expect("id")
            .to_string()
    })
}

#[test]
fn epolq_route_bodies() {
    let place = Place::new("route");
    let addr = place.serve();
    let from = now();
    let (status, body) = post(addr, "all", "はい");
    let to = now();
    assert_eq!(status, 200, "{body}");
    let got: PolicyResponse = wire::decode(&body).expect("方針の応答の形");
    assert!(
        policy_ids(CREATED, from, to).contains(&got.policy.to_string()),
        "{}",
        got.policy
    );
    assert!((from..=to).contains(&got.recorded_at), "{}", got.recorded_at);
    assert_eq!(place.calls("bdw").len(), 3);
    assert!(place.calls("scribe2").is_empty());

    for step in 1..=3u32 {
        let place = Place::new(&format!("route-fail-{step}"));
        place.fail_at(step);
        let addr = place.serve();
        let from = now();
        let (status, body) = post(addr, "all", "はい");
        let to = now();
        assert_eq!(status, 502, "{step}: {body}");
        match step {
            1 => assert_eq!(body, "ledger-create"),
            2 => assert_eq!(body, format!("ledger-append {CREATED}")),
            _ => assert!(
                policy_ids(CREATED, from, to)
                    .iter()
                    .any(|p| body == format!("ledger-close {p}")),
                "{body}"
            ),
        }
    }

    let long48 = format!("fx-p.{}", "7".repeat(43));
    let place = Place::new("route-shape");
    place.created(&format!("{long48}\n"));
    let addr = place.serve();
    assert_eq!(
        post(addr, "all", "はい"),
        (500, format!("policy-id-shape {long48}"))
    );

    let place = Place::new("route-root");
    let mut task = base();
    task[0].kind = "task".into();
    place.bd_returns(&ledger(&task));
    let addr = place.serve();
    assert_eq!(post(addr, "all", "はい"), (503, "no-root".to_string()));
    assert!(place.calls("bdw").is_empty());
}

#[test]
fn epolq_derived_reads_both() {
    let mut beads = base();
    beads[1].status = "closed".into();
    let w = policy::create_write(&id("fx-p"), "all", TWO_LINES);
    let mut made = bead(
        CREATED,
        "方針（範囲 = all）",
        "task",
        "closed",
        &[QUESTION_LABEL, "policy-scope:all"],
    );
    made.description = description(&w).to_string();
    made.notes = policy::line(&rid(POLICY_ID), "all", TWO_LINES);
    beads.push(made);
    let text = ledger(&beads);

    let mut g = build(&Inputs {
        design_index: "FR8\t要件\tsrs.yaml\t00000000\t方針の欄",
        ledger: &text,
        events: "",
    });
    assert!(add_summary(
        &mut g,
        r#"{"id":"FR8","file":"srs.yaml","line":1,"plain":"方針の欄","eng":null}"#
    ));
    let of_kind = |kind: NodeKind| {
        let mut ids: Vec<&str> = g
            .nodes
            .iter()
            .filter(|n| n.kind == kind)
            .map(|n| n.id.as_str())
            .collect();
        ids.sort();
        ids
    };
    assert_eq!(
        of_kind(NodeKind::Policy),
        [POLICY_ID, "policy:20260927T1034Z-1"]
    );
    let node = g.node(CREATED).expect("fx-p.7 の節点");
    assert_eq!(node.kind, NodeKind::Question);
    assert_eq!(node.plain.as_deref(), Some("全体に急がない"));
    assert_eq!(node.eng.as_deref(), Some("範囲 = 全体"));
    assert_eq!(
        unsummarized(&g, true),
        Reading::Known(vec!["fx-p".to_string(), "fx-p.1".to_string()])
    );
    assert_eq!(of_kind(NodeKind::Ruling), ["fx-p.3:20260924T0200Z-1"]);
    let answers: Vec<(&str, &str)> = g
        .edges
        .iter()
        .filter(|e| e.edge_type == EdgeType::Answers)
        .map(|e| (e.from.as_str(), e.to.as_str()))
        .collect();
    assert_eq!(answers, [("fx-p.3:20260924T0200Z-1", "fx-p.3")]);
    assert!(
        !answers.iter().any(|(f, t)| *t == CREATED || *f == POLICY_ID),
        "{answers:?}"
    );

    assert_eq!(
        undelivered(&text),
        Reading::Known(vec![Pending {
            question: id("fx-p.3"),
            ruling: rid("fx-p.3:20260924T0200Z-1"),
        }])
    );
    let Reading::Known(open) = open_questions(&text) else {
        panic!("読める台帳が Unknown");
    };
    let ids: Vec<&str> = open.iter().map(|q| q.card.id.as_str()).collect();
    assert_eq!(ids, ["fx-p.2"]);
}

fn row(labels: &[&str]) -> LedgerRow {
    LedgerRow {
        id: id("fx-p.7"),
        kind: "task".into(),
        title: "方針（範囲 = all）".into(),
        status: "closed".into(),
        updated_at: NOW,
        parent: Some(id("fx-p")),
        labels: labels.iter().map(|l| l.to_string()).collect(),
    }
}

#[test]
fn epolq_policy_label_shape() {
    assert_eq!(POLICY_SCOPE_LABEL, "policy-scope:");
    for yes in [
        &["policy-scope:all"][..],
        &["policy-scope:"],
        &[QUESTION_LABEL, "policy-scope:fx-p.2"],
    ] {
        assert!(row(yes).is_policy(), "{yes:?}");
    }
    for no in [
        &["policy-scope"][..],
        &["xpolicy-scope:all"],
        &[QUESTION_LABEL],
        &[],
    ] {
        assert!(!row(no).is_policy(), "{no:?}");
    }
    assert_eq!(
        labels(&policy::create_write(&id("fx-p"), "all", "急がない")),
        ["intake:question", "policy-scope:all"]
    );
    assert_eq!(
        labels(&policy::create_write(&id("fx-p"), "fx-p.2", "急がない")),
        ["intake:question", "policy-scope:fx-p.2"]
    );
    assert_eq!(policy::scope_label("fx-p.2"), "policy-scope:fx-p.2");
}

/// test の属性の付いた fn の名（属性の次の行の `fn <名>(`）。
fn test_names(text: &str) -> Vec<String> {
    let lines: Vec<&str> = text.lines().map(str::trim).collect();
    lines
        .windows(2)
        .filter(|w| w[0] == "#[test]")
        .filter_map(|w| w[1].strip_prefix("fn "))
        .map(|rest| rest.split('(').next().unwrap_or(rest).to_string())
        .collect()
}

#[test]
fn epolq_own_names_clean() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mine = fs::read_to_string(dir.join("tests/epolq.rs")).expect("この file");
    let hist = fs::read_to_string(dir.join("../tsuzuri-surface/tests/epolqhist.rs"))
        .expect("面の歯の file");
    let names: Vec<String> = [mine, hist].iter().flat_map(|t| test_names(t)).collect();
    assert_eq!(names.len(), 13, "{names:?}");
    let mut sorted = names.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(sorted.len(), 13, "名が重なる: {names:?}");
    for name in &names {
        assert!(name.starts_with("epolq_"), "{name}");
        for word in FILTER_WORDS {
            assert!(!name.contains(word), "{name} が {word} を含む");
        }
    }
    let mut words = FILTER_WORDS.to_vec();
    words.sort();
    words.dedup();
    assert_eq!(words.len(), 139, "filter の語が重なる");
}
