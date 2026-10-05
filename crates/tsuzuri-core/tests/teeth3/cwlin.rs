//! 台帳の相談の行の歯（接頭辞 cwlin_・設計ノート surface-wave27a 行 cs-lines・判断の記録 ADR-29 決定 (4)(8)）。
//! 6 つの頭の行の字と読みの往復・自由な字の整え・器の引用の形と定型行の頭の断り・置き場（memo か根だけ）と
//! notes の上限の溢れ・未受けの計算・席への固定の 1 行の字を見る。台帳の bead は歯の中で組む。
#![cfg(test)]

use tsuzuri_contract::consult::{FindingId, Form, RequestId, Starter, Verdict, Via, WindowId};
use tsuzuri_contract::ledger::{BeadId, LedgerItem, LedgerRow, MEMO_LABEL, QUESTION_LABEL};
use tsuzuri_core::consult::lines::{
    By, Event, HEADS, Home, HomeError, Line, LineError, NOTES_MAX, Subject, cited, free, home_of,
    notice, read, render, scan, unreceived,
};
use tsuzuri_core::delivery::MARK_PREFIX;
use tsuzuri_core::graph::build::TYPED_LINES;

const AT: &str = "20261003T1530Z";

fn fid(s: &str) -> FindingId {
    FindingId::parse(s).expect("所見 id")
}

fn rq(s: &str) -> RequestId {
    RequestId::parse(s).expect("頼み id")
}

fn win(s: &str) -> WindowId {
    WindowId::parse(s).expect("窓 id")
}

fn open(by: By, opened: bool, again: bool) -> Line {
    Line::Open {
        window: win("cw3"),
        form: Form::Talk,
        topic: None,
        by,
        model: "fable".into(),
        effort: "xhigh".into(),
        opened,
        again,
        at: "20261003T1412Z".into(),
    }
}

/// 6 つの頭の見本の行と、その字。
fn samples() -> Vec<(Line, &'static str)> {
    vec![
        (
            Line::Request {
                id: rq("rq-20261003T1412Z-1"),
                topic: Some("t3-hub.75".into()),
                form: Form::Talk,
                model: "fable".into(),
                at: "20261003T1412Z".into(),
            },
            "相談の頼み = rq-20261003T1412Z-1・題 = t3-hub.75・形 = 話す・model = fable・起こし手 = 持ち主の button・時刻 = 20261003T1412Z",
        ),
        (
            open(
                By::Chat {
                    uttered: "20261003T1410Z".into(),
                },
                true,
                false,
            ),
            "相談の開き = cw3・形 = 話す・題 = 題なし・起こし手 = 持ち主のチャット・発話 = 20261003T1410Z・model = fable・念入りさ = xhigh・結果 = 開いた・時刻 = 20261003T1412Z",
        ),
        (
            Line::Finding {
                id: fid("cw3-1"),
                topic: Some("t3-hub.75".into()),
                at: AT.into(),
            },
            "相談の所見 = cw3-1・窓 = cw3・題 = t3-hub.75・時刻 = 20261003T1530Z",
        ),
        (
            Line::Receipt {
                subject: Subject::Finding(fid("cw3-1")),
                via: Via::Watch,
                at: AT.into(),
            },
            "相談の受け = cw3-1・経路 = 見張り・時刻 = 20261003T1530Z",
        ),
        (
            Line::Disposal {
                id: fid("cw3-1"),
                verdict: Verdict::Partial,
                reason: "前提の 1 つが古い".into(),
                keep: vec!["work/report.md".into()],
                drop: vec!["work/a.py".into(), "work/b.csv".into()],
                at: "20261003T1602Z".into(),
            },
            "相談の処分 = cw3-1・採否 = 一部採る・理由 = 前提の 1 つが古い・保存 = work/report.md・不保存 = work/a.py、work/b.csv・時刻 = 20261003T1602Z",
        ),
        (
            Line::Close {
                window: win("cw3"),
                by: Starter::Seat,
                findings: 2,
                at: "20261003T1800Z".into(),
            },
            "相談の閉じ = cw3・起こし手 = 席・所見 = 2・時刻 = 20261003T1800Z",
        ),
    ]
}

#[test]
fn cwlin_six_heads_text_and_roundtrip() {
    assert_eq!(
        HEADS,
        [
            "相談の頼み",
            "相談の開き",
            "相談の所見",
            "相談の受け",
            "相談の処分",
            "相談の閉じ"
        ]
    );
    let all = samples();
    for (i, (line, want)) in all.iter().enumerate() {
        let text = render(line).expect("行の字");
        assert_eq!(text, *want);
        assert!(text.starts_with(&format!("{} = ", HEADS[i])));
        assert_eq!(read(&text).as_ref(), Some(line));
    }
    let notes = format!("前の行\n{}\n裁定 id = x\n{}\n", all[0].1, all[5].1);
    assert_eq!(scan(&notes), vec![all[0].0.clone(), all[5].0.clone()]);
    // 否定の見本は 6 つの見本の行（主の id もほかの欄も正しい）から 1 つだけを替える。
    // 頭だけを知らない頭に替えた行・閉じの見本から時刻の欄だけを除いた行・空の行は読まない。
    let mut bad: Vec<String> = all
        .iter()
        .map(|(_, want)| want.replacen("相談の", "相談の雑談の", 1))
        .collect();
    bad.push(all[5].1.replacen("・時刻 = 20261003T1800Z", "", 1));
    bad.push(String::new());
    for text in &bad {
        assert_eq!(read(text), None, "{text}");
    }
    // 欄の足りない行: 6 つの見本の行から欄を 1 つずつ除いた行（頭と主の id と残りの欄は正しいまま）を読まない。
    let mut short = Vec::new();
    for (_, want) in &all {
        let fields: Vec<&str> = want.split('・').collect();
        for skip in 1..fields.len() {
            let mut rest = fields.clone();
            rest.remove(skip);
            short.push(rest.join("・"));
        }
    }
    assert_eq!(short.len(), 26);
    for text in &short {
        assert_eq!(read(text), None, "{text}");
    }
}

#[test]
fn cwlin_open_starters_and_marks() {
    let cases = [
        (open(By::Seat, true, false), "起こし手 = 席・model"),
        (
            open(
                By::Chat {
                    uttered: "20261003T1410Z".into(),
                },
                true,
                false,
            ),
            "起こし手 = 持ち主のチャット・発話 = 20261003T1410Z・model",
        ),
        (
            open(
                By::Button {
                    request: rq("rq-20261003T1412Z-1"),
                },
                true,
                false,
            ),
            "起こし手 = 持ち主の button・頼み = rq-20261003T1412Z-1・model",
        ),
        (
            open(By::Seat, false, true),
            "結果 = 落ちた・撃ち直し = はい・時刻",
        ),
        (
            open(By::Seat, true, true),
            "結果 = 開いた・撃ち直し = はい・時刻",
        ),
    ];
    for (line, part) in cases {
        let text = render(&line).expect("行の字");
        assert!(text.contains(part), "{text}");
        assert_eq!(read(&text), Some(line));
    }
    let request = Line::Receipt {
        subject: Subject::Request(rq("rq-20261003T1412Z-2")),
        via: Via::Watch,
        at: AT.into(),
    };
    let text = render(&request).expect("行の字");
    assert_eq!(
        text,
        "相談の受け = rq-20261003T1412Z-2・経路 = 見張り・時刻 = 20261003T1530Z"
    );
    assert_eq!(read(&text), Some(request));
    for via in [Via::Done, Via::Hook, Via::List] {
        let line = Line::Receipt {
            subject: Subject::Finding(fid("cw1-2")),
            via,
            at: AT.into(),
        };
        assert_eq!(read(&render(&line).expect("行の字")), Some(line));
    }
}

#[test]
fn cwlin_free_text_cleaned() {
    assert_eq!(free(" a・b\nc\td ", 40), "a／b c d");
    assert_eq!(free(&"あ".repeat(50), 40).chars().count(), 40);
    let line = Line::Disposal {
        id: fid("cw3-1"),
        verdict: Verdict::Reject,
        reason: format!("x・y\n{}", "い".repeat(100)),
        keep: vec![],
        drop: vec![],
        at: AT.into(),
    };
    let text = render(&line).expect("行の字");
    let reason = text
        .split('・')
        .find_map(|f| f.strip_prefix("理由 = "))
        .expect("理由");
    assert!(reason.starts_with("x／y い"));
    assert_eq!(reason.chars().count(), 80);
    assert!(text.contains("・保存 = なし・不保存 = なし・"));
    let topic = Line::Finding {
        id: fid("cw3-1"),
        topic: Some(format!("題・{}", "う".repeat(60))),
        at: AT.into(),
    };
    let t = render(&topic).expect("行の字");
    let got = t
        .split('・')
        .find_map(|f| f.strip_prefix("題 = "))
        .expect("題");
    assert_eq!(got.chars().count(), 40);
    let blank = Line::Finding {
        id: fid("cw3-1"),
        topic: Some(" \n ".into()),
        at: AT.into(),
    };
    assert!(render(&blank).expect("行の字").contains("題 = 題なし"));
}

#[test]
fn cwlin_refuses_cited_forms() {
    for bad in [
        "t3-hub.75:20261003T1412Z-1",
        "batch:20261003T1412Z-1",
        "policy:20261003T1412Z-2",
        "policy:x",
        "batch:",
        "user 2026-10-03T14:12Z",
        "user 20261003T1412Z",
        "裁定 id = a",
        "受け id = b",
        "方針 id = c",
        "配達 = d",
    ] {
        assert!(cited(bad), "{bad}");
        let line = Line::Finding {
            id: fid("cw3-1"),
            topic: Some(bad.into()),
            at: AT.into(),
        };
        assert_eq!(render(&line), Err(LineError::Cited), "{bad}");
    }
    for ok in [
        "t3-hub.75",
        "rq-20261003T1412Z-1",
        "時刻 = 20261003T1412Z",
        "user の言葉",
        "a:b",
        "t3-hub.75.1",
        "x:20261003T1412Z",
        // 裁定 id の形から分の字か字「-」か数字の 1 つだけを替えた字。
        "t3-hub.75:20261003X1412Z-1",
        "t3-hub.75:20261003T1412Z_1",
        "t3-hub.75:20261003T1412Z-x",
    ] {
        assert!(!cited(ok), "{ok}");
    }
    for (line, _) in samples() {
        let text = render(&line).expect("行の字");
        assert!(TYPED_LINES.iter().all(|(head, _)| !text.starts_with(head)));
        assert!(!text.starts_with(MARK_PREFIX));
    }
}

/// 行の時刻の欄だけを替える。
fn set_at(line: &mut Line, minute: &str) {
    match line {
        Line::Request { at, .. }
        | Line::Open { at, .. }
        | Line::Finding { at, .. }
        | Line::Receipt { at, .. }
        | Line::Disposal { at, .. }
        | Line::Close { at, .. } => *at = minute.into(),
    }
}

#[test]
fn cwlin_refuses_bad_fields() {
    // 6 つの見本の行の時刻の欄だけを分の形でない字に替える（ほかの欄は正しいまま）。
    for (mut line, _) in samples() {
        set_at(&mut line, "2026-10-03");
        assert_eq!(render(&line), Err(LineError::Minute), "{line:?}");
    }
    assert_eq!(
        render(&open(
            By::Chat {
                uttered: "昨日".into()
            },
            true,
            false
        )),
        Err(LineError::Minute)
    );
    let mut empty = open(By::Seat, true, false);
    if let Line::Open { model, .. } = &mut empty {
        *model = " ・ ".replace('・', "\n");
    }
    assert_eq!(render(&empty), Err(LineError::Empty));
    for path in ["a、b", "a・b", "", "a\nb"] {
        let line = Line::Disposal {
            id: fid("cw3-1"),
            verdict: Verdict::Adopt,
            reason: "r".into(),
            keep: vec![path.into()],
            drop: vec![],
            at: AT.into(),
        };
        assert_eq!(render(&line), Err(LineError::Path), "{path:?}");
    }
}

fn bead(id: &str, kind: &str, parent: Option<&str>, labels: &[&str], notes: usize) -> LedgerItem {
    LedgerItem {
        row: LedgerRow {
            id: BeadId::new(id).expect("bead id"),
            kind: kind.into(),
            title: id.into(),
            status: "open".into(),
            updated_at: 0,
            parent: parent.map(|p| BeadId::new(p).expect("親")),
            labels: labels.iter().map(|l| (*l).to_string()).collect(),
        },
        description: String::new(),
        notes: "x".repeat(notes),
    }
}

/// 根・memo・memo の下の問い・問いの下の task・根の下の契約と、根にならない 3 つ（閉じた epic・親の無い task・
/// 親の在る epic）。根にならない 3 つは id の自然な順で根より前に置く（根の 3 つの条件の 1 つを外すと根に選ばれる）。
fn ledger(memo_notes: usize, root_notes: usize) -> Vec<LedgerItem> {
    let mut old = bead("fx-a0", "epic", None, &[], 0);
    old.row.status = "closed".into();
    vec![
        old,
        bead("fx-hub", "epic", None, &[], root_notes),
        bead(
            "fx-hub.7",
            "task",
            Some("fx-hub"),
            &[MEMO_LABEL],
            memo_notes,
        ),
        bead("fx-hub.7.1", "task", Some("fx-hub.7"), &[QUESTION_LABEL], 0),
        bead("fx-hub.7.1.2", "task", Some("fx-hub.7.1"), &[], 0),
        bead("fx-hub.9", "task", Some("fx-hub"), &[], 0),
        bead("fx-a1", "task", None, &[], 0),
        bead("fx-a2", "epic", Some("fx-hub"), &[], 0),
    ]
}

fn home(id: &str) -> Home {
    Home {
        id: BeadId::new(id).expect("bead id"),
        full: None,
    }
}

#[test]
fn cwlin_home_memo_or_root() {
    let items = ledger(0, 0);
    let text = "相談の所見 = cw3-1・窓 = cw3・題 = fx-hub.7.1・時刻 = 20261003T1530Z";
    for (topic, want) in [
        (Some("fx-hub.7.1"), "fx-hub.7"),
        (Some("fx-hub.7.1.2"), "fx-hub.7"),
        (Some("fx-hub.7"), "fx-hub.7"),
        (Some("fx-hub.9"), "fx-hub"),
        (Some("fx-hub"), "fx-hub"),
        (Some("自由な題"), "fx-hub"),
        (None, "fx-hub"),
    ] {
        assert_eq!(home_of(&items, topic, text), Ok(home(want)), "{topic:?}");
    }
    for item in &items {
        let got = home_of(&items, Some(item.row.id.as_str()), text).expect("置き場");
        assert!(
            ["fx-hub", "fx-hub.7"].contains(&got.id.as_str()),
            "{} → {}",
            item.row.id,
            got.id
        );
    }
    assert_eq!(home_of(&items[2..], None, text), Err(HomeError::NoRoot));
    let mut ring = ledger(0, 0);
    ring[4].row.parent = Some(BeadId::new("fx-hub.7.1.2").expect("親"));
    ring[3].row.parent = Some(BeadId::new("fx-hub.7.1.2").expect("親"));
    assert_eq!(home_of(&ring, Some("fx-hub.7.1"), text), Ok(home("fx-hub")));
    // memo の下の memo の下の題は近い方の memo に置く。
    let mut nested = ledger(0, 0);
    nested.extend([
        bead("fx-hub.7.5", "task", Some("fx-hub.7"), &[MEMO_LABEL], 0),
        bead("fx-hub.7.5.1", "task", Some("fx-hub.7.5"), &[], 0),
    ]);
    assert_eq!(
        home_of(&nested, Some("fx-hub.7.5.1"), text),
        Ok(home("fx-hub.7.5"))
    );
    // 根になれる epic が 2 つなら id の自然な順で最初（台帳の順・字の順・逆の順なら fx-r10）。
    let roots = [
        bead("fx-r10", "epic", None, &[], 0),
        bead("fx-r9", "epic", None, &[], 0),
    ];
    assert_eq!(home_of(&roots, None, text), Ok(home("fx-r9")));
}

#[test]
fn cwlin_full_memo_goes_to_root() {
    assert_eq!(NOTES_MAX, 8192);
    let text = "相談の受け = cw3-1・経路 = 見張り・時刻 = 20261003T1530Z";
    let room = NOTES_MAX - text.len() - 1;
    let fits = ledger(room, 0);
    assert_eq!(
        home_of(&fits, Some("fx-hub.7.1"), text),
        Ok(home("fx-hub.7"))
    );
    let full = ledger(room + 1, 0);
    let got = home_of(&full, Some("fx-hub.7.1"), text).expect("根");
    assert_eq!(got.id.as_str(), "fx-hub");
    assert_eq!(got.full.as_ref().map(BeadId::as_str), Some("fx-hub.7"));
    assert_eq!(
        got.line(text),
        format!("{text}・memo = fx-hub.7（notes が満ちた）")
    );
    let both = ledger(room + 1, NOTES_MAX * 4);
    assert_eq!(home_of(&both, Some("fx-hub.7.1"), text), Ok(got));
    let root_big = ledger(0, NOTES_MAX * 4);
    assert_eq!(home_of(&root_big, None, text), Ok(home("fx-hub")));
    assert_eq!(home_of(&ledger(0, room), None, text), Ok(home("fx-hub")));
}

/// 頼みの行（id は分の字 20261003T1412Z と番 n）。
fn request_line(n: u32) -> Line {
    Line::Request {
        id: rq(&format!("rq-20261003T1412Z-{n}")),
        topic: None,
        form: Form::Ask,
        model: "fable".into(),
        at: AT.into(),
    }
}

/// 受けの行。
fn receipt_line(subject: Subject) -> Line {
    Line::Receipt {
        subject,
        via: Via::Watch,
        at: AT.into(),
    }
}

/// 所見の行。
fn finding_line(id: &str) -> Line {
    Line::Finding {
        id: fid(id),
        topic: None,
        at: AT.into(),
    }
}

#[test]
fn cwlin_unreceived_findings_and_requests() {
    // 台帳は頼み 2・3・10・頼み 3 の受け・頼み 1・頼み 2 の 2 度目・所見 cw1-1 の受け・所見の行 cw1-2 と cw3-1。
    let lines = vec![
        request_line(2),
        request_line(3),
        request_line(10),
        receipt_line(Subject::Request(rq("rq-20261003T1412Z-3"))),
        request_line(1),
        request_line(2),
        receipt_line(Subject::Finding(fid("cw1-1"))),
        finding_line("cw1-2"),
        finding_line("cw3-1"),
    ];
    // 作業場の所見は入力の順で cw2-1・cw10-1・cw1-1・cw1-2・cw2-1（2 度目）。
    let found = [
        fid("cw2-1"),
        fid("cw10-1"),
        fid("cw1-1"),
        fid("cw1-2"),
        fid("cw2-1"),
    ];
    let got = unreceived(&lines, &found);
    // 所見は重ねずに id の順（窓の数の順・字の順なら cw10-1 が cw2-1 の前・入力の順とその逆とも違う）。
    // 受けの行の在る cw1-1 と、台帳に所見の行だけが在って作業場に無い cw3-1 は返さない。
    assert_eq!(
        got.findings,
        vec![fid("cw1-2"), fid("cw2-1"), fid("cw10-1")]
    );
    // 頼みは重ねずに台帳の順で 2・10・1（id の数の順 1・2・10 と、id の字の順と逆の順の 1・10・2 のどれとも違う）。
    // 受けの行の在る頼み 3 は返さず、2 度在る頼み 2 は最初の行の位置。
    assert_eq!(
        got.requests,
        vec![
            rq("rq-20261003T1412Z-2"),
            rq("rq-20261003T1412Z-10"),
            rq("rq-20261003T1412Z-1")
        ]
    );
    assert_eq!(unreceived(&[], &[]).findings, vec![]);
}

#[test]
fn cwlin_notice_fixed_lines() {
    let t = "/x/tzw";
    let tail = "・「/x/tzw consult watch」を背景で置き直す";
    let cases = [
        (
            Event::Finding {
                id: fid("cw3-1"),
                via: Via::Watch,
            },
            format!(
                "相談: 窓 cw3 の所見 cw3-1 が届いた（/x/tzw consult show cw3-1 --via 見張り）{tail}"
            ),
        ),
        (
            Event::Finding {
                id: fid("cw4-2"),
                via: Via::Done,
            },
            "相談: 窓 cw4 の所見 cw4-2 が届いた（/x/tzw consult show cw4-2 --via 完了）"
                .to_string(),
        ),
        (
            Event::Request(rq("rq-20261003T1412Z-1")),
            format!(
                "相談: 頼み rq-20261003T1412Z-1 が届いた（/x/tzw consult open --request rq-20261003T1412Z-1 --by button）{tail}"
            ),
        ),
        (
            Event::Stalled(win("cw4")),
            format!(
                "相談: 問う窓 cw4 が所見なしで止まった（/x/tzw consult launch cw4 --again）{tail}"
            ),
        ),
        (
            Event::Gone(win("cw3")),
            format!(
                "相談: 話す窓 cw3 が止まった（開き直しを待つ・持ち主が閉じると言えば /x/tzw consult close cw3 --by chat）{tail}"
            ),
        ),
        (
            Event::Timeout,
            format!("相談: 見張りが上限で終わった{tail}"),
        ),
    ];
    for (event, want) in cases {
        assert_eq!(notice(&event, t), want);
    }
    // 経路 完了 でない所見の行（hook・一覧）は末に置き直しの命令を持つ。
    for via in [Via::Hook, Via::List] {
        let event = Event::Finding {
            id: fid("cw3-1"),
            via,
        };
        assert!(notice(&event, t).ends_with(tail), "{via:?}");
    }
}
