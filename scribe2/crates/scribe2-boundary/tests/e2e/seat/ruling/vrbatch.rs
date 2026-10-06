//! 器の答えと結びの口の束の欄（旗 `--batch`）と、裁定の行の読みと、結びの番号と、答えの口の途中の止まりの束つきの結び直しの歯
//! （接頭辞 `vrbatch_`・設計 docs/design/dialogue-surface.md §11 / fleet-event-log.md §14）。

use super::*;
use vessel::ledger::close_reason::{ruling_row, RulingRow};

/// 束の id（旗 `--batch` の値）。
const BATCH: &str = "batch:20261006T0100Z-1";

/// 問いの起票の時刻（開いた問いの fixture）。
const CREATED: &str = "2026-09-30T06:00:00Z";

/// 偽の bd の書きの動詞の列（撃った順）。
fn verbs(fake: &Fake) -> Vec<String> {
    fake.writes().iter().filter_map(|call| call.first().cloned()).collect()
}

/// notes の裁定の行を持つ問い 1 本の show（status と notes を選ぶ）。
fn with_notes(status: &str, notes: &str) -> String {
    show_json(status, &["intake:question"], CREATED, None, notes)
}

/// 偽の bd の close の理由（書きのうち動詞が close の撃ちの引数 3 つ目）。
fn close_reason(fake: &Fake) -> Option<String> {
    fake.writes().iter().find(|call| call.first().is_some_and(|verb| verb == "close")).and_then(|call| call.get(3).cloned())
}

/// 答えの口の途中の止まりの 1 行（束の周）。
fn partial_line(ts: &str) -> String {
    format!("seat ruling: partial utterance={ts} batch={BATCH} question=s2-q1\n")
}

/// (1) 束つきの答え: rc 0・stdout は裁定 id の 1 行だけ・notes の書きは 1 回で 6 欄（束の欄は経路と逐語の間）・close の理由は `裁定 <id>`。
#[test]
fn vrbatch_answer_writes_the_batch_row() {
    let fake = Fake::new();
    fake.show("s2-q1", &open_question(CREATED, None));
    let out = fake.answer_with("s2-q1", ANSWER_WORDS, &["--batch", BATCH]);
    assert_eq!(rc_of(&out), i32::from(RC_OK), "stderr={}", stderr_of(&out));
    let said = fake.utterances();
    let ts = said.first().map(|found| found.ts.clone()).unwrap_or_default();
    let id = answered_id("s2-q1", &ts);
    assert_eq!(stdout_of(&out), format!("{id}\n"), "stdout は裁定 id と改行だけ");
    let row = [id.as_str(), "s2-q1", ts.as_str(), "gui", BATCH, json_lite::quote(ANSWER_WORDS).as_str()].join(" | ");
    let expected = [
        vec!["update".to_owned(), "s2-q1".to_owned(), "--append-notes".to_owned(), row],
        vec!["close".to_owned(), "s2-q1".to_owned(), "--reason".to_owned(), format!("裁定 {id}")],
    ];
    assert_eq!(fake.writes(), expected, "notes は 1 回の 6 欄の行・close の理由は 裁定 と id");
}

/// (2) 悪い束の語（`x:1`・`batch:`・`batch:a b`）は答えの口も結びの口も使い方の 1 行で断り、偽の bd を撃たず、event log は不変。
#[test]
fn vrbatch_both_mouths_refuse_a_bad_batch_word() {
    let usage = vessel::seat::cli::usage();
    for word in ["x:1", "batch:", "batch:a b"] {
        let answer = Fake::new();
        answer.show("s2-q1", &open_question(CREATED, None));
        let before = answer.log();
        let out = answer.answer_with("s2-q1", ANSWER_WORDS, &["--batch", word]);
        assert_eq!(rc_of(&out), i32::from(RC_REFUSED), "answer {word:?}: {}", stderr_of(&out));
        assert_eq!(stderr_of(&out), format!("{usage}\n"), "answer {word:?}: 使い方の 1 行");
        assert!(answer.calls().is_empty(), "answer {word:?}: 偽の bd を撃たない");
        assert_eq!(answer.log(), before, "answer {word:?}: event log は不変");

        let bind = Fake::new();
        bind.say(TS_A, Channel::Gui, WORDS);
        bind.show("s2-q1", &open_question(CREATED, None));
        let before = bind.log();
        let out = bind.bind_with("s2-q1", TS_A, &["--batch", word]);
        assert_eq!(rc_of(&out), i32::from(RC_REFUSED), "bind {word:?}: {}", stderr_of(&out));
        assert_eq!(stderr_of(&out), format!("{usage}\n"), "bind {word:?}: 使い方の 1 行");
        assert!(bind.calls().is_empty(), "bind {word:?}: 偽の bd を撃たない");
        assert_eq!(bind.log(), before, "bind {word:?}: event log は不変");
    }
}

/// (3)(4) `ruling_row` は 5 つ目の欄が `batch:` で始まる 6 欄の行を 5 欄の `RulingRow` として読み、始まらない 6 欄の行は `None`。
#[test]
fn vrbatch_ruling_row_reads_the_batch_row() {
    let line = format!("s2-q1:20261006T0100Z-1 | s2-q1 | 2026-10-06T01:00:00.000Z | gui | {BATCH} | \"よい\"");
    let expected = RulingRow {
        id: "s2-q1:20261006T0100Z-1".to_owned(),
        question: "s2-q1".to_owned(),
        ts: "2026-10-06T01:00:00.000Z".to_owned(),
        route: "gui".to_owned(),
        verbatim: "\"よい\"".to_owned(),
    };
    assert_eq!(ruling_row(&line, Some("s2")), Some(expected), "束の欄を飛ばして読む");
    let other = "s2-q1:20261006T0100Z-1 | s2-q1 | 2026-10-06T01:00:00.000Z | gui | \"a\" | \"よい\"";
    assert_eq!(ruling_row(other, Some("s2")), None, "5 つ目の欄が batch: でない 6 欄の行は読まない");
}

/// (5)(6) 取られた番号の次へ進む: notes の 1 行に番号 1 の id が在れば番号 2・同じ発話の番号 2 の 5 欄の行が在れば書かずに番号 2 で閉じる。
#[test]
fn vrbatch_bind_numbers_past_a_taken_id() {
    let taken = "裁定 id = s2-q1:20260930T0705Z-1・問い = s2-q1・逐語 = 前";
    let fake = Fake::new();
    fake.say(TS_A, Channel::Gui, WORDS);
    fake.show("s2-q1", &with_notes("open", taken));
    let out = fake.bind("s2-q1", TS_A);
    assert_eq!(rc_of(&out), i32::from(RC_OK), "stderr={}", stderr_of(&out));
    assert_eq!(verbs(&fake), ["update", "close"], "notes を 1 回足して close");
    let id = "s2-q1:20260930T0705Z-2";
    assert_eq!(fake.notes().split(" | ").next(), Some(id), "足す行の 1 つ目の欄は番号 2");
    assert_eq!(close_reason(&fake), Some(format!("裁定 {id}")), "close の理由は番号 2");

    let fake = Fake::new();
    fake.say(TS_A, Channel::Gui, WORDS);
    let written = format!("{id} | s2-q1 | {TS_A} | gui | {}", json_lite::quote(WORDS));
    fake.show("s2-q1", &with_notes("open", &format!("{taken}\n{written}")));
    let out = fake.bind("s2-q1", TS_A);
    assert_eq!(rc_of(&out), i32::from(RC_OK), "stderr={}", stderr_of(&out));
    assert_eq!(verbs(&fake), ["close"], "notes は書かない");
    assert_eq!(close_reason(&fake), Some(format!("裁定 {id}")), "close の理由は書き済みの番号 2");
}

/// (7) 1 行に番号 1 と 2 と 3 の id が在る notes は番号 4 へ進む（notes の行の数を上限にする実装を落とす）。
#[test]
fn vrbatch_bind_numbers_past_every_id_of_one_line() {
    let line = "s2-q1:20260930T0705Z-1 s2-q1:20260930T0705Z-2 s2-q1:20260930T0705Z-3";
    let fake = Fake::new();
    fake.say(TS_A, Channel::Gui, WORDS);
    fake.show("s2-q1", &with_notes("open", line));
    let out = fake.bind("s2-q1", TS_A);
    assert_eq!(rc_of(&out), i32::from(RC_OK), "stderr={}", stderr_of(&out));
    let id = "s2-q1:20260930T0705Z-4";
    assert_eq!(fake.notes().split(" | ").next(), Some(id), "足す行の 1 つ目の欄は番号 4");
    assert_eq!(close_reason(&fake), Some(format!("裁定 {id}")), "close の理由は番号 4");
}

/// (8) close の途中の止まり: 束の周の partial の 1 行と 6 欄の notes の 1 行が残り、同じ ts と束の結び直しは close だけを撃って閉じる。
#[test]
fn vrbatch_close_stop_is_finished_by_a_batch_bind() {
    let fake = Fake::new();
    fake.show("s2-q1", &open_question(CREATED, None));
    fixture(&fake.dir, "close.fail", "");
    let out = fake.answer_with("s2-q1", ANSWER_WORDS, &["--batch", BATCH]);
    let ts = fake.utterances().first().map(|found| found.ts.clone()).unwrap_or_default();
    assert_eq!(rc_of(&out), i32::from(RC_REFUSED), "stderr={}", stderr_of(&out));
    assert_eq!(stderr_of(&out), partial_line(&ts), "束の周の partial の 1 行");
    assert_eq!(fake.notes().lines().count(), 1, "notes の行は 1 行");
    let fields: Vec<String> = fake.notes().trim_end().splitn(6, " | ").map(str::to_owned).collect();
    assert_eq!(fields.get(4).map(String::as_str), Some(BATCH), "5 つ目の欄は束の id: {fields:?}");
    assert_eq!(fake.rulings().len(), 0, "裁定 event は 0 件");
    fake.show("s2-q1", &with_notes("open", &fake.notes()));
    fake.forget();
    let again = fake.bind_with("s2-q1", &ts, &["--batch", BATCH]);
    assert_eq!(rc_of(&again), i32::from(RC_OK), "stderr={}", stderr_of(&again));
    assert_eq!(verbs(&fake), ["close"], "書きは close の 1 回だけ");
    assert_eq!(close_reason(&fake), Some(format!("裁定 {}", answered_id("s2-q1", &ts))), "理由は番号 1 の裁定 id");
    assert_eq!(fake.notes().lines().count(), 1, "notes は 1 行のまま");
    assert_eq!(fake.rulings().len(), 1, "裁定 event は 1 件");
}

/// (9) 裁定 event の途中の止まり: 脇へ移した log の発話の ts で束の周の partial の 1 行が出て、log を戻した束つきの結び直しは台帳へ書かず裁定 event を 1 件書く。
#[test]
fn vrbatch_event_stop_is_finished_by_a_batch_bind() {
    let fake = Fake::new();
    fake.show("s2-q1", &open_question(CREATED, None));
    let path = store::events_path(&fake.state);
    fixture(&fake.dir, "close.swap", &path.display().to_string());
    let out = fake.answer_with("s2-q1", ANSWER_WORDS, &["--batch", BATCH]);
    let aside = PathBuf::from(format!("{}.aside", path.display()));
    let moved: Vec<Event> = fs::read_to_string(&aside).unwrap_or_default().lines().filter_map(|line| Event::from_line(line).ok()).collect();
    let ts = moved.iter().find(|found| found.kind == EventKind::UtteranceReceived).map(|found| found.ts.clone()).unwrap_or_default();
    assert_eq!(rc_of(&out), i32::from(RC_REFUSED), "stderr={}", stderr_of(&out));
    assert_eq!(stderr_of(&out), partial_line(&ts), "束の周の partial の 1 行");
    assert_eq!(verbs(&fake), ["update", "close"], "update と close は残る");
    fs::remove_dir(&path).ok();
    fs::rename(&aside, &path).ok();
    fake.show("s2-q1", &with_notes("closed", &fake.notes()));
    fake.forget();
    let again = fake.bind_with("s2-q1", &ts, &["--batch", BATCH]);
    assert_eq!(rc_of(&again), i32::from(RC_OK), "stderr={}", stderr_of(&again));
    assert!(fake.writes().is_empty(), "結び直しは台帳へ書かない: {:?}", fake.writes());
    assert_eq!(fake.rulings().len(), 1, "裁定 event は 1 件");
    assert_eq!(fake.utterances().len(), 1, "発話 event は 1 件のまま");
}

/// (10) notes の追記の止まり: 束の周の partial の 1 行・書きは update だけ。追記の落ちを除いた束つきの結び直しは update と close を 1 回ずつ撃ち、
/// update の行は番号 1 の裁定 id と束の欄の 6 欄。
#[test]
fn vrbatch_notes_stop_is_finished_by_a_batch_bind() {
    let fake = Fake::new();
    fake.show("s2-q1", &open_question(CREATED, None));
    fixture(&fake.dir, "notes.fail", "");
    let out = fake.answer_with("s2-q1", ANSWER_WORDS, &["--batch", BATCH]);
    let ts = fake.utterances().first().map(|found| found.ts.clone()).unwrap_or_default();
    assert_eq!(rc_of(&out), i32::from(RC_REFUSED), "stderr={}", stderr_of(&out));
    assert_eq!(stderr_of(&out), partial_line(&ts), "束の周の partial の 1 行");
    assert_eq!(verbs(&fake), ["update"], "書きは update だけ");
    fs::remove_file(fake.dir.join("notes.fail")).ok();
    fake.forget();
    let again = fake.bind_with("s2-q1", &ts, &["--batch", BATCH]);
    assert_eq!(rc_of(&again), i32::from(RC_OK), "stderr={}", stderr_of(&again));
    assert_eq!(verbs(&fake), ["update", "close"], "update と close の 1 回ずつ");
    let fields: Vec<String> = fake.notes().trim_end().splitn(6, " | ").map(str::to_owned).collect();
    assert_eq!(fields.first().cloned(), Some(answered_id("s2-q1", &ts)), "1 つ目の欄は番号 1 の裁定 id");
    assert_eq!((fields.len(), fields.get(4).map(String::as_str)), (6, Some(BATCH)), "6 欄・5 つ目の欄は束の id: {fields:?}");
    assert_eq!(fake.rulings().len(), 1, "裁定 event は 1 件");
}
