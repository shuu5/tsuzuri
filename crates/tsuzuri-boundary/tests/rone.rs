//! 裁定の書き手を器の答えの口の 1 本に寄せる歯（接頭辞 rone_・設計ノート surface-v4b 行 t-ruling-one）。
//! board の答えと束の口が器の答えの口を撃ち、取り消しと番号と配達の読みが器の行も読む歯である。
#![cfg(test)]

use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use tsuzuri_boundary::server::batch::{self, next_batch_id};
use tsuzuri_boundary::server::ledger::Source;
use tsuzuri_boundary::server::ruling::{
    self, Outcome, Vessel, Writer, answer_argv, answered_id, next_id, revoke_line,
};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::{BeadId, LedgerItem, LedgerRow, QUESTION_LABEL};
use tsuzuri_contract::surface::{
    BatchItem, BatchItemResult, BatchRequest, BatchResponse, ItemOutcome, RulingId, RulingRequest,
    RulingResponse, latest_ruling, revocable,
};
use tsuzuri_contract::wire;
use tsuzuri_core::delivery::{Pending, Said, said, undelivered};
use tsuzuri_core::question::open_questions;

/// 受付の時刻（2026-09-28T04:41:30Z）と、その分。
const NOW: u64 = 1_790_570_490;
const MINUTE: &str = "20260928T0441Z";

/// 偽の器の行の発話の時刻の字。
const TS: &str = "2026-09-28T04:41:30.000Z";

/// 3 つの問い。
const QS: [&str; 3] = ["a1-q.1", "a1-q.2", "a1-q.3"];

fn bead(id: &str) -> BeadId {
    BeadId::new(id).expect("bead id")
}

fn rid(id: &str) -> RulingId {
    RulingId::new(id).expect("記帳 id")
}

/// 偽の器が問い id から出す裁定 id。
fn vessel_id(question: &str) -> String {
    format!("{question}:{MINUTE}-1")
}

fn script(path: &Path, body: &str) {
    fs::write(path, format!("#!/bin/sh\n{body}\n")).expect("偽の program");
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("偽の program の権限");
}

/// 器の結びの行の見本（裁定 id・問い id・発話の ts・経路 gui・逐語を二重引用符で囲んだ字を縦線の欄で繋いだ 5 欄）。
fn vessel_line(question: &str, verbatim: &str) -> String {
    format!(
        "{} | {question} | {TS} | gui | {}",
        vessel_id(question),
        wire::encode(&verbatim).expect("逐語の JSON の字")
    )
}

/// 器の束の行の見本（5 欄の経路の後に束の id の欄を挟んだ 6 欄）。
fn batch_line(question: &str, batch: &str, verbatim: &str) -> String {
    format!(
        "{} | {question} | {TS} | gui | {batch} | {}",
        vessel_id(question),
        wire::encode(&verbatim).expect("逐語の JSON の字")
    )
}

/// 台帳の JSON の字（根の epic と、status open で label intake:question の問い 3 本・notes は `notes` の組の字）。
fn ledger(notes: [&str; 3]) -> String {
    let s = |text: &str| wire::encode(&text).expect("字の電文");
    let mut lines = vec![
        "{\"id\":\"a1\",\"title\":\"根\",\"status\":\"open\",\"priority\":1,\"issue_type\":\"epic\",\"created_at\":\"2026-09-26T00:00:00Z\",\"updated_at\":\"2026-09-26T00:00:00Z\",\"labels\":[]}".to_string(),
    ];
    for (i, q) in QS.iter().enumerate() {
        let notes = if notes[i].is_empty() {
            String::new()
        } else {
            format!(",\"notes\":{}", s(notes[i]))
        };
        lines.push(format!(
            "{{\"id\":\"{q}\",\"title\":\"問い {q}\",\"description\":\"概要 = 問い {q} の概要\",\"status\":\"open\",\"priority\":1,\"issue_type\":\"task\",\"created_at\":\"2026-09-27T01:00:00Z\",\"updated_at\":\"2026-09-27T01:00:00Z\",\"labels\":[\"{QUESTION_LABEL}\"],\"parent\":\"a1\"{notes}}}"
        ));
    }
    format!("[\n{}\n]\n", lines.join(",\n"))
}

/// 歯ごとの作業場（repo・state dir・記録の置き場・偽の bd と bdw と器）。
struct Place {
    root: PathBuf,
    repo: PathBuf,
    state: PathBuf,
    log: PathBuf,
    program: PathBuf,
}

impl Place {
    fn new(name: &str, text: &str) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("rone")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        let (repo, state, log) = (root.join("repo"), root.join("state"), root.join("log"));
        for dir in [&repo, &state, &log] {
            fs::create_dir_all(dir).expect("作業場の dir");
        }
        fs::write(root.join("out.json"), text).expect("偽の bd の出力");
        script(
            &root.join("bd"),
            &format!("exec cat '{}'", root.join("out.json").display()),
        );
        let (r, l) = (root.display(), log.display());
        let record = |name: &str| {
            format!(
                "n=$(( $(cat '{l}/{name}.count' 2>/dev/null || echo 0) + 1 ))\n\
                 echo \"$n\" > '{l}/{name}.count'\n\
                 for a in \"$@\"; do printf '%s\\000' \"$a\"; done > '{l}/{name}.'\"$n\"'.args'\n"
            )
        };
        script(&root.join("bdw"), &format!("{}exit 0", record("bdw")));
        script(
            &root.join("scribe2"),
            &format!(
                "{}if [ \"$1 $2 $3\" = 'seat ruling answer' ]; then\n\
                   cat > '{l}/scribe2.'\"$n\"'.stdin'\n\
                   if [ -e '{r}/fail.'\"$9\" ]; then echo 落ちた >&2; exit 1; fi\n\
                   if [ -e '{l}/junk' ]; then echo not-an-id; exit 0; fi\n\
                   echo \"$9:{MINUTE}-1\"\n\
                 fi\n\
                 exit 0",
                record("scribe2")
            ),
        );
        let program = root.join("scribe2");
        Place {
            root,
            repo,
            state,
            log,
            program,
        }
    }

    fn source(&self) -> Source {
        Source::new(self.repo.clone(), self.root.join("bd"))
    }

    /// 配達の先は無し。
    fn writer(&self) -> Writer {
        Writer {
            repo: self.repo.clone(),
            bdw: self.root.join("bdw").into(),
            delivery: None,
        }
    }

    /// 答えの口の撃ち先（偽の器の program と置き場）。
    fn vessel(&self) -> Vessel<'_> {
        (self.program.as_os_str(), Some(self.state.as_path()))
    }

    /// 偽の器が問いで落ちる（何も出さず rc 1）。
    fn fail_on(&self, question: &str) {
        fs::write(self.root.join(format!("fail.{question}")), "").expect("落とす問い");
    }

    /// 偽の器が字 not-an-id を出す。
    fn junk(&self) {
        fs::write(self.log.join("junk"), "").expect("junk の印");
    }

    /// 偽の program（bdw か scribe2）が撃たれた回ごとの argv（NUL で区切った記録）。
    fn argvs(&self, name: &str) -> Vec<Vec<String>> {
        let count: u32 = fs::read_to_string(self.log.join(format!("{name}.count")))
            .map_or(0, |c| c.trim().parse().expect("回の数"));
        (1..=count)
            .map(|n| {
                fs::read_to_string(self.log.join(format!("{name}.{n}.args")))
                    .expect("記録")
                    .split_terminator('\0')
                    .map(str::to_string)
                    .collect()
            })
            .collect()
    }

    /// 偽の器の `n` 回目の標準入力の byte。
    fn stdin(&self, n: u32) -> Vec<u8> {
        fs::read(self.log.join(format!("scribe2.{n}.stdin"))).expect("標準入力の記録")
    }

    /// 偽の器の答えの口の argv の見本（束の id が在れば末に旗つき）。
    fn words(&self, question: &str, batch: Option<&str>) -> Vec<String> {
        let mut words: Vec<String> = [
            "seat",
            "ruling",
            "answer",
            "--repo",
            &self.repo.display().to_string(),
            "--state-dir",
            &self.state.display().to_string(),
            "--question",
            question,
        ]
        .map(str::to_string)
        .to_vec();
        if let Some(batch) = batch {
            words.extend(["--batch".to_string(), batch.to_string()]);
        }
        words
    }
}

/// 中核の問いの一覧で読んだ card の digest（見た版の要約値）。
fn digest(text: &str, id: &str) -> String {
    let Reading::Known(open) = open_questions(text) else {
        panic!("見本の台帳が Unknown");
    };
    open.into_iter()
        .find(|q| q.card.id.as_str() == id)
        .unwrap_or_else(|| panic!("{id} が open の問いでない"))
        .card
        .digest
}

fn ruling_request(text: &str, question: &str, verbatim: &str) -> RulingRequest {
    RulingRequest {
        question: bead(question),
        seen_digest: digest(text, question),
        verbatim: verbatim.to_string(),
    }
}

fn batch_request(text: &str, questions: &[&str], verbatim: &str) -> BatchRequest {
    BatchRequest {
        items: questions
            .iter()
            .map(|q| BatchItem {
                question: bead(q),
                seen_digest: digest(text, q),
                verbatim: None,
            })
            .collect(),
        verbatim: verbatim.to_string(),
    }
}

/// (1) 答えの口の引数の列は 9 語で、束の id を渡すと末に旗と束の id が付く。
#[test]
fn rone_answer_argv_words() {
    let (state, repo, question) = (Path::new("/st"), Path::new("/r"), bead("a1-q.1"));
    let words = |argv: Vec<OsString>| -> Vec<String> {
        argv.into_iter()
            .map(|a| a.into_string().expect("argv の字"))
            .collect()
    };
    assert_eq!(
        words(answer_argv(state, repo, &question, None)),
        [
            "seat",
            "ruling",
            "answer",
            "--repo",
            "/r",
            "--state-dir",
            "/st",
            "--question",
            "a1-q.1"
        ]
    );
    let batch = rid("batch:20260928T0441Z-1");
    assert_eq!(
        words(answer_argv(state, repo, &question, Some(&batch))),
        [
            "seat",
            "ruling",
            "answer",
            "--repo",
            "/r",
            "--state-dir",
            "/st",
            "--question",
            "a1-q.1",
            "--batch",
            "batch:20260928T0441Z-1"
        ]
    );
}

/// (2) 答えは器の答えの口を 1 回撃ち、裁定 id は器が出した字で、偽の bdw は撃たない。
#[test]
fn rone_ruling_answers_through_the_vessel() {
    let text = ledger(["", "", ""]);
    let place = Place::new("answers", &text);
    let verbatim = "はい\n二行目 \\ 逆斜線\n";
    let got = ruling::accept(
        &ruling_request(&text, "a1-q.1", verbatim),
        &place.source(),
        &place.writer(),
        place.vessel(),
        NOW,
    );
    assert_eq!(
        got,
        Outcome::Recorded(RulingResponse {
            ruling: rid(&vessel_id("a1-q.1")),
            recorded_at: NOW,
        })
    );
    let argvs = place.argvs("scribe2");
    assert_eq!(argvs, [place.words("a1-q.1", None)], "器は 1 回");
    let direct: Vec<String> = answer_argv(&place.state, &place.repo, &bead("a1-q.1"), None)
        .into_iter()
        .map(|a| a.into_string().expect("argv の字"))
        .collect();
    assert_eq!(argvs[0], direct, "argv は answer_argv の列");
    assert_eq!(place.stdin(1), verbatim.as_bytes(), "標準入力は逐語の byte");
    assert!(place.argvs("bdw").is_empty(), "偽の bdw を撃つ");
}

/// (3) 器の答えの口が落ちれば AnswerFailed で、偽の bdw は撃たない。
#[test]
fn rone_ruling_answer_failure_writes_nothing() {
    let text = ledger(["", "", ""]);
    let place = Place::new("failure", &text);
    place.fail_on("a1-q.1");
    let got = ruling::accept(
        &ruling_request(&text, "a1-q.1", "はい"),
        &place.source(),
        &place.writer(),
        place.vessel(),
        NOW,
    );
    assert_eq!(got, Outcome::AnswerFailed);
    assert_eq!(place.argvs("scribe2").len(), 1);
    assert!(place.argvs("bdw").is_empty(), "偽の bdw を撃つ");
}

/// (4) rc 0 でも標準出力が答えた問いの id の形でなければ IdShape。
#[test]
fn rone_ruling_answer_stdout_must_be_an_id() {
    let text = ledger(["", "", ""]);
    let place = Place::new("not-an-id", &text);
    place.junk();
    let got = ruling::accept(
        &ruling_request(&text, "a1-q.1", "はい"),
        &place.source(),
        &place.writer(),
        place.vessel(),
        NOW,
    );
    assert_eq!(got, Outcome::IdShape);
    assert!(place.argvs("bdw").is_empty(), "偽の bdw を撃つ");
    // 形の照らしは問いの id と分の字と数字の 3 つを見る。
    let q = bead("a1-q.1");
    let shaped = |out: &str| answered_id(&q, out.as_bytes()).map(|id| id.as_str().to_string());
    assert_eq!(
        shaped("a1-q.1:20260928T0441Z-1\n").as_deref(),
        Some("a1-q.1:20260928T0441Z-1")
    );
    assert_eq!(
        shaped(" a1-q.1:20260928T0441Z-12 ").as_deref(),
        Some("a1-q.1:20260928T0441Z-12")
    );
    for bad in [
        "not-an-id",
        "",
        "a1-q.2:20260928T0441Z-1",
        "a1-q.1:20260928T0441Z-",
        "a1-q.1:20260928T0441Z-x",
        "a1-q.1:20260928T0441-1",
        "a1-q.1:2026-09-28-1",
        "a1-q.1:20260928T0441Z-1\nおまけ",
    ] {
        assert_eq!(shaped(bad), None, "{bad:?}");
    }
}

/// (5) 置き場が無ければ答えも束も NoStateDir で、偽の器も偽の bdw も撃たない。
#[test]
fn rone_ruling_needs_a_state_dir() {
    let text = ledger(["", "", ""]);
    let place = Place::new("no-state", &text);
    let vessel: Vessel<'_> = (place.program.as_os_str(), None);
    let got = ruling::accept(
        &ruling_request(&text, "a1-q.1", "はい"),
        &place.source(),
        &place.writer(),
        vessel,
        NOW,
    );
    assert_eq!(got, Outcome::NoStateDir);
    let got = batch::accept(
        &batch_request(&text, &["a1-q.1", "a1-q.2"], "はい"),
        &place.source(),
        &place.writer(),
        vessel,
        NOW,
    );
    assert_eq!(got, batch::Outcome::NoStateDir);
    assert!(place.argvs("scribe2").is_empty(), "偽の器を撃つ");
    assert!(place.argvs("bdw").is_empty(), "偽の bdw を撃つ");
}

/// (6) 束は行を要求の順に束の旗つきで 1 行ずつ答え、行の結果は偽の器が出した裁定 id の Written。
#[test]
fn rone_batch_answers_rows_in_order() {
    let text = ledger(["", "", ""]);
    let place = Place::new("batch-rows", &text);
    let got = batch::accept(
        &batch_request(&text, &["a1-q.1", "a1-q.2"], "束の答え"),
        &place.source(),
        &place.writer(),
        place.vessel(),
        NOW,
    );
    let batch_id = format!("batch:{MINUTE}-1");
    assert_eq!(
        got,
        batch::Outcome::Recorded(BatchResponse {
            batch: rid(&batch_id),
            items: ["a1-q.1", "a1-q.2"]
                .map(|q| BatchItemResult {
                    question: bead(q),
                    outcome: ItemOutcome::Written {
                        ruling: rid(&vessel_id(q)),
                    },
                })
                .to_vec(),
        })
    );
    assert_eq!(
        place.argvs("scribe2"),
        [
            place.words("a1-q.1", Some(&batch_id)),
            place.words("a1-q.2", Some(&batch_id))
        ],
        "要求の順に 2 回"
    );
    assert_eq!(place.stdin(1), "束の答え".as_bytes());
    assert_eq!(place.stdin(2), "束の答え".as_bytes());
    assert!(place.argvs("bdw").is_empty(), "偽の bdw を撃つ");
}

/// (7) 束は落ちた行で止め、その行とそれより後の行は Unwritten で、後の行は撃たない。
#[test]
fn rone_batch_stops_at_the_failed_row() {
    let text = ledger(["", "", ""]);
    let place = Place::new("batch-stop", &text);
    place.fail_on("a1-q.2");
    let got = batch::accept(
        &batch_request(&text, &QS, "束の答え"),
        &place.source(),
        &place.writer(),
        place.vessel(),
        NOW,
    );
    let batch_id = format!("batch:{MINUTE}-1");
    let outcomes = [
        ItemOutcome::Written {
            ruling: rid(&vessel_id("a1-q.1")),
        },
        ItemOutcome::Unwritten,
        ItemOutcome::Unwritten,
    ];
    assert_eq!(
        got,
        batch::Outcome::WriteFailed(BatchResponse {
            batch: rid(&batch_id),
            items: QS
                .iter()
                .zip(outcomes)
                .map(|(q, outcome)| BatchItemResult {
                    question: bead(q),
                    outcome,
                })
                .collect(),
        })
    );
    assert_eq!(
        place.argvs("scribe2"),
        [
            place.words("a1-q.1", Some(&batch_id)),
            place.words("a1-q.2", Some(&batch_id))
        ],
        "落ちた行で止める"
    );
    assert!(place.argvs("bdw").is_empty(), "偽の bdw を撃つ");
}

/// (8) 束の id の番号は、台帳の字が器の束の行の縦線の欄を持つ時も、定型行の束の欄を持つ時も増える。
#[test]
fn rone_batch_id_sees_vessel_rows() {
    let first = format!("batch:{MINUTE}-1");
    let second = format!("batch:{MINUTE}-2");
    let next = |text: &str| {
        next_batch_id(text, MINUTE)
            .expect("束の id")
            .as_str()
            .to_string()
    };
    assert_eq!(next("束の行は無い"), first);
    let vessel = format!("見本\\n{}", batch_line("a1-q.1", &first, "はい"));
    assert!(vessel.contains(&format!(" | {first} | ")));
    assert_eq!(next(&vessel), second);
    assert_eq!(next(&format!("束 = {first}・")), second);
    // 両方の字が在れば、両方の番号を越える。
    assert_eq!(
        next(&format!("{vessel}\\n束 = {second}・逐語 = はい")),
        format!("batch:{MINUTE}-3")
    );
}

/// (9) 裁定 id の番号は、notes の器の結びの行（5 欄）の id も取られた id に数える。
#[test]
fn rone_next_id_sees_vessel_ids() {
    let q = bead("a1-q.1");
    let first = format!("a1-q.1:{MINUTE}-1");
    let second = format!("a1-q.1:{MINUTE}-2");
    let id = |notes: &str| {
        next_id(&q, notes, MINUTE)
            .expect("裁定 id")
            .as_str()
            .to_string()
    };
    assert_eq!(id(""), first);
    let vessel = vessel_line("a1-q.1", "よい");
    assert!(vessel.starts_with(&format!("{first} | a1-q.1 | {TS} | gui | \"")));
    assert_eq!(id(&vessel), second);
    assert_eq!(id(&format!("前置き\n{vessel}\r\n")), second);
    // 問いが違う器の行は数えない。
    assert_eq!(id(&vessel_line("a1-q.2", "よい")), first);
}

/// (10) 器の結びの行だけを notes に持つ閉じた問いは取り消せ、取り消しの行を足すと効いている裁定が無い。
#[test]
fn rone_revocable_reads_vessel_rows() {
    let first = format!("a1-q.1:{MINUTE}-1");
    let second = format!("a1-q.1:{MINUTE}-2");
    let item = |notes: String| LedgerItem {
        row: LedgerRow {
            id: bead("a1-q.1"),
            kind: "task".to_string(),
            title: "問い".to_string(),
            status: "closed".to_string(),
            updated_at: NOW,
            parent: Some(bead("a1")),
            labels: vec![QUESTION_LABEL.to_string()],
        },
        description: String::new(),
        notes,
    };
    let notes = vessel_line("a1-q.1", "はい");
    let closed = item(notes.clone());
    assert_eq!(latest_ruling(&closed.notes), Some(first.as_str()));
    assert!(revocable(&closed, &rid(&first)));
    let revoked = format!(
        "{notes}\n{}",
        revoke_line(&rid(&second), &bead("a1-q.1"), &rid(&first), "待つ")
    );
    assert_eq!(latest_ruling(&revoked), None);
}

/// (11) 束の旗つきの 6 欄の行は、束の id でも裁定 id でも名指せて逐語を戻し、印の無い裁定として出る。
#[test]
fn rone_said_reads_the_batch_row() {
    let batch_id = rid(&format!("batch:{MINUTE}-1"));
    let ruling = rid(&vessel_id("a1-q.3"));
    let verbatim = "はい\n二行目 \\ 逆斜線 | 縦線";
    let text = ledger([
        "",
        "",
        &batch_line("a1-q.3", batch_id.as_str(), verbatim),
    ]);
    let want = Reading::Known(vec![Said {
        question: bead("a1-q.3"),
        ruling: ruling.clone(),
        verbatim: verbatim.to_string(),
    }]);
    assert_eq!(said(&text, std::slice::from_ref(&batch_id)), want);
    assert_eq!(said(&text, std::slice::from_ref(&ruling)), want);
    assert_eq!(
        said(&text, &[rid(&format!("batch:{MINUTE}-2"))]),
        Reading::Known(Vec::new()),
        "ほかの束の id では名指せない"
    );
    assert_eq!(
        undelivered(&text),
        Reading::Known(vec![Pending {
            question: bead("a1-q.3"),
            ruling,
        }])
    );
}
