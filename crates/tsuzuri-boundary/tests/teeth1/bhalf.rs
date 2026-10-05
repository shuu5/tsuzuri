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
        recorder(&root.join("scribe2"), &log, "scribe2");
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
        batch::accept(req, &self.source(), &self.writer(), NOW)
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

/// 束の 1 行の 2 回の書き（notes への追記と閉じる）の argv。
fn row_argvs(q: &str, ruling: &RulingId, batch_id: &RulingId) -> [Vec<String>; 2] {
    [
        LedgerWrite::AppendNotes {
            id: bead(q),
            line: batch::line(ruling, &bead(q), batch_id, VERBATIM),
        }
        .argv(),
        LedgerWrite::CloseItem {
            id: bead(q),
            reason: format!("裁定 {ruling} 束 {batch_id}"),
        }
        .argv(),
    ]
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

/// (1) 閉じていないと書いていないの電文の字（書いたは今のまま・知らない outcome の字は読めない）。
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
            result(
                "fx-h.2",
                ItemOutcome::Unclosed {
                    ruling: ruling_of("fx-h.2", 1),
                },
            ),
            "{\"question\":\"fx-h.2\",\"outcome\":\"unclosed\",\"ruling\":\"fx-h.2:20260928T0441Z-1\"}",
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
    assert!(
        wire::decode::<BatchItemResult>("{\"question\":\"fx-h.3\",\"outcome\":\"unsent\"}")
            .is_err()
    );
}

/// (2) 6 回の書きの s 回目が落ちると、502 の本文は要求の 3 行を要求の順に持ち、後の書きも配達も撃たない。
#[test]
fn bhalf_step_k_of_n() {
    let text = ledger([OPEN, OPEN, OPEN]);
    let req = request(&text, &QS);
    let b1 = batch_id(1);
    let full: Vec<Vec<String>> = QS
        .iter()
        .flat_map(|q| row_argvs(q, &ruling_of(q, 1), &b1))
        .collect();
    assert_eq!(full.len(), 6);
    for s in 1..=6usize {
        let place = Place::new(&format!("step-{s}"));
        place.bd_returns(&text);
        place.fail_at("bdw", s);
        let fell = (s - 1) / 2;
        let items: Vec<BatchItemResult> = QS
            .iter()
            .enumerate()
            .map(|(i, q)| {
                let outcome = if i < fell {
                    ItemOutcome::Written {
                        ruling: ruling_of(q, 1),
                    }
                } else if i == fell && s % 2 == 0 {
                    ItemOutcome::Unclosed {
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
        assert_eq!(place.argvs("bdw"), full[..s], "{s} 回目で止める");
        assert!(place.argvs("scribe2").is_empty(), "{s}: 落ちた束を配達する");
    }
}

/// (3) 2 行目の閉じる書きで落ちた束の後に、残りの 2 行を送り直す（新しい裁定の id で足して閉じ、何も消さない）。
#[test]
fn bhalf_resend_rest() {
    let place = Place::new("resend");
    let text = ledger([OPEN, OPEN, OPEN]);
    place.bd_returns(&text);
    place.fail_at("bdw", 4);
    let b1 = batch_id(1);
    let old = ruling_of("fx-h.2", 1);
    assert!(matches!(
        place.accept(&request(&text, &QS)),
        batch::Outcome::WriteFailed(_)
    ));
    assert_eq!(place.argvs("bdw").len(), 4);
    // 1 つ目の束の書きを台帳に映す（fx-h.1 は閉じ、fx-h.2 は open のまま裁定の行が残る）。
    let kept = |q: &str| Some(batch::line(&ruling_of(q, 1), &bead(q), &b1, VERBATIM));
    let after = ledger([
        Q {
            status: "closed",
            extra: kept("fx-h.1"),
        },
        Q {
            status: "open",
            extra: kept("fx-h.2"),
        },
        OPEN,
    ]);
    place.bd_returns(&after);
    place.fail_never("bdw");
    let from = now();
    let got = place.accept(&request(&after, &QS[1..]));
    let to = now();
    let b2 = batch_id(2);
    let (r2, r3) = (ruling_of("fx-h.2", 2), ruling_of("fx-h.3", 1));
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
    place.wait("bdw", 10);
    let argvs = place.argvs("bdw");
    assert_eq!(argvs.len(), 10, "{argvs:?}");
    let [a2, c2] = row_argvs("fx-h.2", &r2, &b2);
    let [a3, c3] = row_argvs("fx-h.3", &r3, &b2);
    assert_eq!(argvs[4..8], [a2, c2, a3, c3], "追記・閉じる・追記・閉じる");
    let minutes = [ruling::minute(from), ruling::minute(to)];
    for (at, q, r) in [(8, "fx-h.2", &r2), (9, "fx-h.3", &r3)] {
        let marks: Vec<Vec<String>> = minutes.iter().map(|m| mark_argv(q, r, m)).collect();
        assert!(
            marks.contains(&argvs[at]),
            "{} 回目は {q} の配達の口の印: {:?}",
            at + 1,
            argvs[at]
        );
    }
    deliver_and_writes(place, b2, argvs, minutes, old);
}

/// 送り直しの束の配達が 1 度だけで、書きが追記か閉じるだけで前の裁定に触れないことを見る。
fn deliver_and_writes(
    place: Place,
    b2: RulingId,
    argvs: Vec<Vec<String>>,
    minutes: [String; 2],
    old: RulingId,
) {
    let state = place.state.display().to_string();
    assert_eq!(
        place.argvs("scribe2"),
        [[
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
        .to_vec()],
        "送り直しの束の id で 1 度だけ"
    );
    // 書きは追記か閉じるだけ（notes の置き換えも開き直しも無い）。
    for argv in &argvs {
        assert_eq!(argv.len(), 3, "{argv:?}");
        let append = argv[0] == "update" && argv[2].starts_with("--append-notes=");
        let close = argv[0] == "close" && argv[2].starts_with("--reason=");
        assert!(append || close, "{argv:?}");
    }
    // 前の裁定の行を消す書きも、その裁定の印を置く書きも無い。
    for argv in &argvs {
        for route in [Route::Deliver, Route::Stop] {
            for m in &minutes {
                let mark = mark_line(&old, route, m);
                assert!(!argv[2].contains(&mark), "{argv:?}");
            }
        }
        assert!(!argv[2].contains(&format!("配達 = {old}")), "{argv:?}");
    }
    for argv in &argvs[4..] {
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
