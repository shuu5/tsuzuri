//! 束の書きが途中で落ちた 502 の歯（接頭辞 bhalf_・設計ノート surface-wave12e の行 e-batch-partial）。
//! server を立てず、束の受付 `batch::accept` を受付の時刻 1790570490（2026-09-28T04:41:30Z）で直に撃つ。
//! 偽の bd は作業場の out.json を標準出力へ出す script、偽の bdw と偽の器は受けた argv を記録の置き場に書き、
//! `<log>/<名>.fail` の回なら rc 1 で終わる script。台帳は歯の中で組む（fixture の file は使わない）。
#![cfg(test)]

use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use crate::common::{bead, now, script};
use tsuzuri_boundary::server::batch;
use tsuzuri_boundary::server::ledger::Source;
use tsuzuri_boundary::server::ruling::{self, Delivery, Writer};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::LedgerWrite;
use tsuzuri_contract::surface::{
    BatchItem, BatchItemResult, BatchRequest, BatchResponse, ItemOutcome, RulingId,
};
use tsuzuri_contract::wire;
use tsuzuri_core::delivery::{Route, mark_line};
use tsuzuri_core::question::open_questions;

/// 着地済みの filter の語を畳んだ語と、計画の後の行 g-gz の接頭辞（この行の歯の名はどれも部分の字として含まない）。
const FILTER_WORDS: [&str; 125] = [
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
    "gfresh_",
    "ghb_",
    "glabel_",
    "gnav_",
    "graph_",
    "gsum_",
    "gtuck_",
    "gview_",
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
    "lspark_",
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
    "pipe_",
    "plimit_",
    "pmore_",
    "pquest_",
    "project_",
    "ptitle_",
    "punmap_",
    "pwhole_",
    "qblock_",
    "qgate_",
    "qkey_",
    "question_",
    "rhold_",
    "runsdoc_",
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
    "steady_",
    "sxaxis_",
    "ticker_",
    "tipx_",
    "topbar_",
    "tz_",
    "urpanel_",
    "uword_",
    "wstrip_",
    "pgz_",
];

/// 受付の時刻（2026-09-28T04:41:30Z）と、その分。
const NOW: u64 = 1_790_570_490;
const MINUTE: &str = "20260928T0441Z";

/// 束の逐語。
const VERBATIM: &str = "束の答え";

/// 3 つの問い。
const QS: [&str; 3] = ["fx-h.1", "fx-h.2", "fx-h.3"];

/// 撃たれた回ごとに argv を `<log>/<name>.<回>.args` に書き、`<log>/<name>.fail` の回なら rc 1 で終わる script。
fn recorder(path: &Path, log: &Path, name: &str) {
    let log = log.display();
    script(
        path,
        &format!(
            "n=$(( $(cat '{log}/{name}.count' 2>/dev/null || echo 0) + 1 ))\n\
             echo \"$n\" > '{log}/{name}.count'\n\
             for a in \"$@\"; do printf '%s\\n' \"$a\"; done > '{log}/{name}.'\"$n\"'.args'\n\
             if [ \"$n\" = \"$(cat '{log}/{name}.fail' 2>/dev/null)\" ]; then echo 落ちた >&2; exit 1; fi\n\
             exit 0"
        ),
    );
}

/// `recorder` と同じ記録に加え、引数の頭の 3 語が seat と ruling と answer の回は標準入力を `<log>/<name>.<回>.stdin` に書き、
/// `--question` の値（9 番目の引数）に字 `:20260928T0441Z-1` を足した字を 1 行出す（`<log>/junk` が在れば字 not-an-id を出す）。
fn vessel(path: &Path, log: &Path, name: &str) {
    let log = log.display();
    script(
        path,
        &format!(
            "n=$(( $(cat '{log}/{name}.count' 2>/dev/null || echo 0) + 1 ))\n\
             echo \"$n\" > '{log}/{name}.count'\n\
             for a in \"$@\"; do printf '%s\\n' \"$a\"; done > '{log}/{name}.'\"$n\"'.args'\n\
             answer=no\n\
             if [ \"$1 $2 $3\" = 'seat ruling answer' ]; then answer=yes; cat > '{log}/{name}.'\"$n\"'.stdin'; fi\n\
             if [ \"$n\" = \"$(cat '{log}/{name}.fail' 2>/dev/null)\" ]; then echo 落ちた >&2; exit 1; fi\n\
             if [ \"$answer\" = yes ]; then\n\
               if [ -e '{log}/junk' ]; then echo not-an-id; else echo \"$9:{MINUTE}-1\"; fi\n\
             fi\n\
             exit 0"
        ),
    );
}

/// 歯ごとの作業場（repo・state dir・記録の置き場・偽の program）。
struct Place {
    root: PathBuf,
    repo: PathBuf,
    state: PathBuf,
    log: PathBuf,
}

impl Place {
    fn new(name: &str) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("bhalf")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        let (repo, state, log) = (root.join("repo"), root.join("state"), root.join("log"));
        for dir in [&repo, &state, &log] {
            fs::create_dir_all(dir).expect("作業場の dir");
        }
        script(
            &root.join("bd"),
            &format!("exec cat '{}'", root.join("out.json").display()),
        );
        recorder(&root.join("bdw"), &log, "bdw");
        vessel(&root.join("scribe2"), &log, "scribe2");
        Place {
            root,
            repo,
            state,
            log,
        }
    }

    /// 偽の bd が返す字を置く。
    fn bd_returns(&self, text: &str) {
        fs::write(self.root.join("out.json"), text).expect("偽の bd の出力");
    }

    /// 偽の program を `n` 回目で落とす。
    fn fail_at(&self, name: &str, n: usize) {
        fs::write(self.log.join(format!("{name}.fail")), n.to_string()).expect("落とす回");
    }

    /// 落とす回を消す。
    fn fail_never(&self, name: &str) {
        fs::remove_file(self.log.join(format!("{name}.fail"))).expect("落とす回を消す");
    }

    /// 偽の program が撃たれた回ごとの argv。
    fn argvs(&self, name: &str) -> Vec<Vec<String>> {
        let count: usize = fs::read_to_string(self.log.join(format!("{name}.count")))
            .map_or(0, |c| c.trim().parse().expect("回の数"));
        (1..=count)
            .map(|n| {
                fs::read_to_string(self.log.join(format!("{name}.{n}.args")))
                    .expect("記録")
                    .lines()
                    .map(str::to_string)
                    .collect()
            })
            .collect()
    }

    /// 偽の program の `n` 回目の argv の記録が 3 行になるまで 10 秒まで待つ（配達は受付の後の thread）。
    fn wait(&self, name: &str, n: usize) {
        let path = self.log.join(format!("{name}.{n}.args"));
        let until = Instant::now() + Duration::from_secs(10);
        while !fs::read_to_string(&path).is_ok_and(|t| t.lines().count() == 3)
            && Instant::now() < until
        {
            thread::sleep(Duration::from_millis(20));
        }
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
                target: "tsuzuri:0.1".to_string(),
            }),
        }
    }

    fn accept(&self, req: &BatchRequest) -> batch::Outcome {
        let program = self.root.join("scribe2");
        let vessel = (program.as_os_str(), Some(self.state.as_path()));
        batch::accept(req, &self.source(), &self.writer(), vessel, NOW)
    }

    /// 偽の器の答えの口の 1 回の argv（束の旗つきの 11 語）。
    fn answer_words(&self, q: &str, batch_id: &RulingId) -> Vec<String> {
        let (repo, state) = (self.repo.display().to_string(), self.state.display().to_string());
        [
            "seat",
            "ruling",
            "answer",
            "--repo",
            repo.as_str(),
            "--state-dir",
            state.as_str(),
            "--question",
            q,
            "--batch",
            batch_id.as_str(),
        ]
        .map(str::to_string)
        .to_vec()
    }
}

fn rid(id: &str) -> RulingId {
    RulingId::new(id).expect("記帳 id")
}

/// 束の id（`batch:<分>-<n>`）と問いの裁定の id（`<問い>:<分>-<n>`）。
fn batch_id(n: u32) -> RulingId {
    rid(&format!("batch:{MINUTE}-{n}"))
}

fn ruling_of(q: &str, n: u32) -> RulingId {
    rid(&format!("{q}:{MINUTE}-{n}"))
}

/// 問いの 1 本の状態（status と notes の末に足す行）。
struct Q<'a> {
    status: &'a str,
    extra: Option<String>,
}

const OPEN: Q<'static> = Q {
    status: "open",
    extra: None,
};

/// 根の epic と 3 つの問いの台帳（JSON の配列・notes の改行は JSON の字の中で逃がす）。
fn ledger(qs: [Q; 3]) -> String {
    let mut lines = vec![
        "{\"id\":\"fx-h\",\"title\":\"根\",\"status\":\"open\",\"priority\":1,\"issue_type\":\"epic\",\"created_at\":\"2026-09-26T00:00:00Z\",\"updated_at\":\"2026-09-26T00:00:00Z\",\"labels\":[]}".to_string(),
    ];
    for (i, q) in qs.iter().enumerate() {
        let n = i + 1;
        let notes = match &q.extra {
            Some(line) => format!("fx-h.{n} の notes\\n{line}"),
            None => format!("fx-h.{n} の notes"),
        };
        lines.push(format!(
            "{{\"id\":\"fx-h.{n}\",\"title\":\"問い {n}\",\"description\":\"概要 = 問い {n} の概要\",\"status\":\"{}\",\"priority\":1,\"issue_type\":\"task\",\"created_at\":\"2026-09-27T0{n}:00:00Z\",\"updated_at\":\"2026-09-27T0{n}:00:00Z\",\"notes\":\"{notes}\",\"labels\":[\"intake:question\"],\"parent\":\"fx-h\"}}",
            q.status
        ));
    }
    format!("[\n{}\n]\n", lines.join(",\n"))
}

/// 台帳を中核の問いの一覧に通した card の digest を見た版の要約値にした要求（行の逐語は無し）。
fn request(text: &str, questions: &[&str]) -> BatchRequest {
    let Reading::Known(open) = open_questions(text) else {
        panic!("歯の台帳が読めない");
    };
    let items = questions
        .iter()
        .map(|q| BatchItem {
            question: bead(q),
            seen_digest: open
                .iter()
                .find(|o| o.card.id.as_str() == *q)
                .unwrap_or_else(|| panic!("{q} が open の問いでない"))
                .card
                .digest
                .clone(),
            verbatim: None,
        })
        .collect();
    BatchRequest {
        items,
        verbatim: VERBATIM.to_string(),
    }
}

/// 器の束の行（裁定 id・問い id・発話の ts・経路 gui・束の id・逐語の JSON の字を縦線の欄で繋いだ 6 欄）。
/// 台帳の JSON の字の中に置くので、逐語の JSON の字の二重引用符は逆斜線で逃がす。
fn vessel_row(q: &str, batch_id: &RulingId) -> String {
    let verbatim = wire::encode(&VERBATIM)
        .expect("逐語の JSON の字")
        .replace('"', "\\\"");
    format!(
        "{} | {q} | 2026-09-28T04:41:30.000Z | gui | {batch_id} | {verbatim}",
        ruling_of(q, 1)
    )
}

fn mark_argv(q: &str, ruling: &RulingId, minute: &str) -> Vec<String> {
    LedgerWrite::AppendNotes {
        id: bead(q),
        line: mark_line(ruling, Route::Deliver, minute),
    }
    .argv()
}

fn result(q: &str, outcome: ItemOutcome) -> BatchItemResult {
    BatchItemResult {
        question: bead(q),
        outcome,
    }
}

/// 自分の file の test の属性の付いた fn の名。
fn test_names(text: &str) -> Vec<String> {
    let lines: Vec<&str> = text.lines().map(str::trim).collect();
    lines
        .windows(2)
        .filter(|w| w[0] == "#[test]")
        .filter_map(|w| w[1].strip_prefix("fn "))
        .map(|rest| rest.split('(').next().unwrap_or(rest).to_string())
        .collect()
}

/// (1) 書いたと書いていないの電文の字（知らない outcome の字は読めない）。
#[test]
fn bhalf_wire_words() {
    for (value, text) in [
        (
            result(
                "fx-h.1",
                ItemOutcome::Written {
                    ruling: ruling_of("fx-h.1", 1),
                },
            ),
            "{\"question\":\"fx-h.1\",\"outcome\":\"written\",\"ruling\":\"fx-h.1:20260928T0441Z-1\"}",
        ),
        (
            result("fx-h.3", ItemOutcome::Unwritten),
            "{\"question\":\"fx-h.3\",\"outcome\":\"unwritten\"}",
        ),
    ] {
        assert_eq!(wire::encode(&value).expect("電文"), text);
        assert_eq!(
            wire::decode::<BatchItemResult>(text).expect("読める"),
            value,
            "{text}"
        );
    }
    for text in [
        "{\"question\":\"fx-h.2\",\"outcome\":\"unclosed\",\"ruling\":\"fx-h.2:20260928T0441Z-1\"}",
        "{\"question\":\"fx-h.2\",\"outcome\":\"skipped\",\"ruling\":\"fx-h.2:20260928T0441Z-1\"}",
        "{\"question\":\"fx-h.2\",\"outcome\":\"refused\",\"reason\":\"stale-version\"}",
        "{\"question\":\"fx-h.3\",\"outcome\":\"unsent\"}",
    ] {
        assert!(wire::decode::<BatchItemResult>(text).is_err(), "{text}");
    }
}

/// (2) 3 回の答えの口の s 回目が落ちると、502 の本文は要求の 3 行を要求の順に持ち、後の答えも配達も印も撃たない。
#[test]
fn bhalf_step_k_of_n() {
    let text = ledger([OPEN, OPEN, OPEN]);
    let req = request(&text, &QS);
    let b1 = batch_id(1);
    for s in 1..=3usize {
        let place = Place::new(&format!("step-{s}"));
        place.bd_returns(&text);
        place.fail_at("scribe2", s);
        let items: Vec<BatchItemResult> = QS
            .iter()
            .enumerate()
            .map(|(i, q)| {
                let outcome = if i < s - 1 {
                    ItemOutcome::Written {
                        ruling: ruling_of(q, 1),
                    }
                } else {
                    ItemOutcome::Unwritten
                };
                result(q, outcome)
            })
            .collect();
        assert_eq!(
            place.accept(&req),
            batch::Outcome::WriteFailed(BatchResponse {
                batch: b1.clone(),
                items,
            }),
            "{s} 回目で落ちた"
        );
        let want: Vec<Vec<String>> = QS[..s].iter().map(|q| place.answer_words(q, &b1)).collect();
        assert_eq!(place.argvs("scribe2"), want, "{s} 回目で止める");
        assert!(place.argvs("bdw").is_empty(), "{s}: 落ちた束に偽の bdw を撃つ");
    }
}

/// (3) 2 行目の答えで落ちた束の後に、残りの 2 行を送り直す（束の id は次の番号・行は偽の器の出した裁定 id・何も消さない）。
#[test]
fn bhalf_resend_rest() {
    let place = Place::new("resend");
    let text = ledger([OPEN, OPEN, OPEN]);
    place.bd_returns(&text);
    place.fail_at("scribe2", 2);
    let b1 = batch_id(1);
    assert!(matches!(
        place.accept(&request(&text, &QS)),
        batch::Outcome::WriteFailed(_)
    ));
    assert_eq!(place.argvs("scribe2").len(), 2);
    // 1 つ目の束の器の書きを台帳に映す（fx-h.1 は閉じて器の束の行を持ち、fx-h.2 と fx-h.3 は open のまま）。
    let after = ledger([
        Q {
            status: "closed",
            extra: Some(vessel_row("fx-h.1", &b1)),
        },
        OPEN,
        OPEN,
    ]);
    place.bd_returns(&after);
    place.fail_never("scribe2");
    let from = now();
    let got = place.accept(&request(&after, &QS[1..]));
    let to = now();
    let b2 = batch_id(2);
    let (r2, r3) = (ruling_of("fx-h.2", 1), ruling_of("fx-h.3", 1));
    assert_eq!(
        got,
        batch::Outcome::Recorded(BatchResponse {
            batch: b2.clone(),
            items: vec![
                result("fx-h.2", ItemOutcome::Written { ruling: r2.clone() }),
                result("fx-h.3", ItemOutcome::Written { ruling: r3.clone() }),
            ],
        })
    );
    place.wait("bdw", 2);
    let argvs = place.argvs("bdw");
    assert_eq!(argvs.len(), 2, "{argvs:?}");
    let minutes = [ruling::minute(from), ruling::minute(to)];
    for (at, q, r) in [(0, "fx-h.2", &r2), (1, "fx-h.3", &r3)] {
        let marks: Vec<Vec<String>> = minutes.iter().map(|m| mark_argv(q, r, m)).collect();
        assert!(
            marks.contains(&argvs[at]),
            "{} 回目は {q} の配達の口の印: {:?}",
            at + 1,
            argvs[at]
        );
    }
    deliver_and_writes(&place, &b1, &b2, &argvs, &minutes);
}

/// 送り直しの束の答えが残りの行だけで、配達が 1 度だけで、偽の bdw の書きが印だけで前の裁定に触れないことを見る。
fn deliver_and_writes(
    place: &Place,
    b1: &RulingId,
    b2: &RulingId,
    argvs: &[Vec<String>],
    minutes: &[String; 2],
) {
    let state = place.state.display().to_string();
    assert_eq!(
        place.argvs("scribe2"),
        [
            place.answer_words("fx-h.1", b1),
            place.answer_words("fx-h.2", b1),
            place.answer_words("fx-h.2", b2),
            place.answer_words("fx-h.3", b2),
            [
                "seat",
                "deliver",
                "--state-dir",
                state.as_str(),
                "--target",
                "tsuzuri:0.1",
                "--ruling",
                b2.as_str(),
            ]
            .map(str::to_string)
            .to_vec()
        ],
        "落ちた束の 2 回・送り直しの 2 行の答え・送り直しの束の id の配達 1 度だけ"
    );
    // 書きは印の追記だけ（notes の置き換えも閉じも開き直しも無い）。
    for argv in argvs {
        assert_eq!(argv.len(), 3, "{argv:?}");
        assert!(
            argv[0] == "update" && argv[2].starts_with("--append-notes=配達 = "),
            "{argv:?}"
        );
    }
    // 前の束で書いた裁定の行を消す書きも、その裁定の印を置く書きも無い。
    let old = ruling_of("fx-h.1", 1);
    for argv in argvs {
        for route in [Route::Deliver, Route::Stop] {
            for m in minutes {
                let mark = mark_line(&old, route, m);
                assert!(!argv[2].contains(&mark), "{argv:?}");
            }
        }
        assert!(!argv.join(" ").contains(old.as_str()), "{argv:?}");
    }
}

/// (9) この file の歯の名はどれも bhalf_ で始まり、並べた filter の語を部分の字として含まない。
#[test]
fn bhalf_own_names_clean() {
    let text = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/teeth1/bhalf.rs"))
        .expect("自分の file");
    let names = test_names(&text);
    assert_eq!(names.len(), 4, "{names:?}");
    for name in &names {
        assert!(name.starts_with("bhalf_"), "{name}");
        for word in FILTER_WORDS {
            assert!(!name.contains(word), "{name} が {word} を含む");
        }
    }
}
