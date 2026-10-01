//! 便 g-pipe の歯: 5 列の順と列の中の並び・「+n」と開いた列の query・札の題の引き方・0 件と測れていない・
//! 経過の字の境・hover の card の 4 行・段から列への対応は契約の型の関数・着地済みの外形と依存。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::{PipelineBoard, PipelineCard, PipelineColumn, Reading, Stage};
use tsuzuri_contract::ledger::{BeadId, LedgerList, LedgerRow};
use tsuzuri_contract::wire;
use tsuzuri_surface::project::pipeline::{
    self, Column, LANES, Lead, NO_AGE, RUN_KEY, SHOW, SOURCE, UNKNOWN_REASON, age, cards, columns,
    content, kcard, landed_today, open_columns, title_of, with_open,
};
use tsuzuri_surface::project::{Body, NO_CONTENT, NOT_READ};
use tsuzuri_surface::view::{Fetched, JST_OFFSET};
use tsuzuri_surface::vocab::vocab;
use tsuzuri_surface::widgets::hover::ROW_CHARS;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn fixture_text() -> String {
    read("../../tests/fixtures/surface/pipeline-board.json")
}

fn fixture_cards() -> Vec<PipelineCard> {
    let board: PipelineBoard =
        wire::decode(&fixture_text()).expect("fixture の板が電文として読める");
    let Reading::Known(cards) = board.cards else {
        panic!("fixture の札が Unknown");
    };
    cards
}

/// 長い題（空白を畳んで 36 字に切ると `LONG_CUT`）。
const LONG: &str = "0123456789  0123456789 0123456789 0123456789";
const LONG_CUT: &str = "0123456789 0123456789 0123456789 012";

fn row(id: &str, title: &str) -> LedgerRow {
    LedgerRow {
        id: BeadId::new(id).expect("id"),
        kind: "task".to_string(),
        title: title.to_string(),
        status: "open".to_string(),
        updated_at: 1,
        parent: None,
        labels: vec![],
    }
}

/// 台帳の一覧（px.3 は台帳に無い・px.2 は長い題）。
fn ledger_rows() -> Vec<LedgerRow> {
    let mut rows = vec![row("px.2", LONG)];
    for n in [1, 4, 5, 6, 7, 8, 9, 10, 11, 12] {
        rows.push(row(&format!("px.{n}"), &format!("題 {n}")));
    }
    rows
}

fn ledger_body() -> Fetched {
    Fetched::Body(
        wire::encode(&LedgerList {
            rows: Reading::Known(ledger_rows()),
        })
        .expect("電文"),
    )
}

/// 描く時の今（UTC の日の正午・日本時間の 21:00・fixture の経過の在る着地の札は全部が今日に入る）。
const NOW: EpochSecs = 1_790_510_400;

/// px.11 の札（経過の無い着地・今日の着地に入らないので板の列には出ない）。
fn px11() -> PipelineCard {
    fixture_cards()
        .into_iter()
        .find(|c| c.contract.as_str() == "px.11")
        .expect("fixture の px.11")
}

fn filled() -> Vec<Column> {
    match content(&Fetched::Body(fixture_text()), &ledger_body(), NOW) {
        Body::Filled(cols) => cols,
        other => panic!("fixture の板が中身を出さない: {other:?}"),
    }
}

fn ids(col: &Column, open: bool) -> Vec<&str> {
    col.shown(open).iter().map(|c| c.id.as_str()).collect()
}

/// (1) 5 列の順・class・見出しの語の鍵と、列ごとの札の id の並び（経過の短い順・同じなら id の順・経過の無い札は後）。
#[test]
fn pipe_columns_order_and_cards_sorted_by_elapsed() {
    let cards = fixture_cards();
    assert_eq!(cards.len(), 12);
    for stage in Stage::ALL {
        let n = cards.iter().filter(|c| c.stage == stage).count();
        let want = if stage == Stage::Landed { 5 } else { 1 };
        assert_eq!(n, want, "{stage:?} の札の数");
    }
    let cols = filled();
    let lanes: Vec<(PipelineColumn, &str, &str)> = cols
        .iter()
        .map(|c| (c.lane.column, c.class.as_str(), c.lane.key))
        .collect();
    assert_eq!(
        lanes,
        vec![
            (PipelineColumn::Blocked, "col c-block", "col_block"),
            (PipelineColumn::Queued, "col c-queue", "col_queue"),
            (PipelineColumn::RunningGated, "col c-run", "col_run"),
            (
                PipelineColumn::QuestionedFailedStopped,
                "col c-stop",
                "col_stop"
            ),
            (PipelineColumn::Landed, "col c-land", "col_land"),
        ]
    );
    let all: Vec<Vec<&str>> = cols.iter().map(|c| ids(c, true)).collect();
    assert_eq!(
        all,
        vec![
            vec!["px.2"],
            vec!["px.1"],
            vec!["px.4", "px.3"],
            vec!["px.7", "px.5", "px.6"],
            vec!["px.12", "px.9", "px.10", "px.8"],
        ]
    );
    // 見出しの横の数は列の札の全部（Landed の列は今日の着地だけ・経過の無い px.11 は入らない）。
    let counts: Vec<usize> = cols.iter().map(|c| c.cards.len()).collect();
    assert_eq!(counts, vec![1, 1, 2, 3, 4]);
    // 状態の記号: 待ち（Blocked と Queued）・動いている・止まった（待ちの記号）・取り込み（記号でなく取り込みの印）。
    let states: Vec<Option<&str>> = cols.iter().map(|c| c.cards[0].state).collect();
    assert_eq!(
        states,
        vec![Some("wait"), Some("wait"), Some("run"), Some("wait"), None]
    );
}

/// (2) 3 枚を超える列は 3 枚と「+n」・開いた列は全部・開いた列の名は URL の query（鍵 col）に残る。
#[test]
fn pipe_more_button_and_open_column_in_url() {
    assert_eq!(SHOW, 3);
    let cols = filled();
    let land = &cols[4];
    assert_eq!(ids(land, false), vec!["px.12", "px.9", "px.10"]);
    assert_eq!(land.more(false), Some(1));
    assert_eq!(ids(land, true).len(), 4);
    assert_eq!(land.more(true), None);
    // ちょうど 3 枚の列は「+n」を出さない。
    let stop = &cols[3];
    assert_eq!(ids(stop, false).len(), 3);
    assert_eq!(stop.more(false), None);
    open_in_url(&cols);
}

/// 開いた列の名は URL の query（鍵 col）に残り、頁を開き直しても開いたまま。
fn open_in_url(cols: &[Column]) {
    assert_eq!(open_columns(""), vec![]);
    let url = with_open("?mode=expert", PipelineColumn::Landed);
    assert_eq!(url, "?mode=expert&col=land");
    assert_eq!(open_columns(&url), vec![PipelineColumn::Landed]);
    let two = with_open(&url, PipelineColumn::Queued);
    assert_eq!(two, "?mode=expert&col=queue,land");
    assert_eq!(
        open_columns(&two),
        vec![PipelineColumn::Queued, PipelineColumn::Landed]
    );
    assert_eq!(with_open(&two, PipelineColumn::Landed), two);
    assert_eq!(
        open_columns("?col=bogus,stop&page=home"),
        vec![PipelineColumn::QuestionedFailedStopped]
    );
    // 頁を開き直したとき（同じ query）も開いたまま。
    let reopened: Vec<bool> = cols
        .iter()
        .map(|c| open_columns(&two).contains(&c.lane.column))
        .collect();
    assert_eq!(reopened, vec![false, true, false, false, true]);
}

/// (3) 札の題は台帳の一覧から bead の id で引いて 36 字に切り、引けない札は題を出さず id だけ。
#[test]
fn pipe_titles_from_ledger_cut_to_36() {
    let rows = ledger_rows();
    assert_eq!(title_of(&rows, "px.2").as_deref(), Some(LONG_CUT));
    assert_eq!(LONG_CUT.chars().count(), 36);
    assert_eq!(title_of(&rows, "px.1").as_deref(), Some("題 1"));
    assert_eq!(title_of(&rows, "px.3"), None);

    let cols = filled();
    let find = |id: &str| {
        cols.iter()
            .flat_map(|c| c.cards.iter())
            .find(|k| k.id == id)
            .unwrap_or_else(|| panic!("札 {id}"))
            .clone()
    };
    assert_eq!(find("px.2").title.as_deref(), Some(LONG_CUT));
    assert_eq!(find("px.3").title, None);
    assert_eq!(find("px.3").id, "px.3");

    // 台帳の一覧の口が読めなければ、札は全部 id だけ（板は測れていないにしない）。
    for ledger in [Fetched::NotRead, Fetched::Failed] {
        let Body::Filled(cols) = content(&Fetched::Body(fixture_text()), &ledger, NOW) else {
            panic!("台帳が読めないと板が中身を出さない");
        };
        assert!(
            cols.iter()
                .flat_map(|c| c.cards.iter())
                .all(|k| k.title.is_none())
        );
    }
    leads_and_classes(find, rows);
}

/// 止まった列の札は段の理由・ほかの列の札は回数と、札の class と経過の無い札の経過の字。
fn leads_and_classes(find: impl Fn(&str) -> pipeline::Kcard, rows: Vec<LedgerRow>) {
    // 止まった列は回数の代わりに段の理由（理由が空なら段の名）・ほかの列は回数。
    assert_eq!(find("px.5").lead, Lead::Why("about:write-set".to_string()));
    assert_eq!(find("px.6").lead, Lead::Why("verify が赤".to_string()));
    assert_eq!(find("px.7").lead, Lead::Why("Stopped".to_string()));
    assert_eq!(find("px.4").lead, Lead::Runs(3));
    assert_eq!(find("px.9").lead, Lead::Runs(2));
    assert_eq!(find("px.5").class, "kcard why-stop");
    assert_eq!(find("px.1").class, "kcard");
    assert_eq!(kcard(&px11(), &rows, NOW).age, NO_AGE);
}

/// (4) 読めて 0 枚なら 0 件の帯（run の語と 0）と空の 5 列・まだ分からない・読めない・まだ読んでいないは測れていない。
#[test]
fn pipe_empty_band_and_unmeasured() {
    let empty = wire::encode(&PipelineBoard {
        cards: Reading::Known(vec![]),
        misfits: Reading::Known(vec![]),
    })
    .expect("電文");
    let unknown = wire::encode(&PipelineBoard {
        cards: Reading::Unknown,
        misfits: Reading::Unknown,
    })
    .expect("電文");
    let ledger = ledger_body();
    assert_eq!(
        pipeline::body(&Fetched::Body(empty.clone())),
        Body::Empty(RUN_KEY)
    );
    assert!(
        matches!(content(&Fetched::Body(empty), &ledger, NOW), Body::Empty(k) if k == RUN_KEY)
    );
    assert_eq!(vocab().label(RUN_KEY), "run");
    let blank = columns(&[], &[], NOW);
    assert!(
        blank
            .iter()
            .all(|c| c.cards.is_empty() && c.more(false).is_none())
    );
    let classes: Vec<&str> = blank.iter().map(|c| c.class.as_str()).collect();
    assert_eq!(
        classes,
        vec![
            "col c-block is-empty",
            "col c-queue is-empty",
            "col c-run is-empty",
            "col c-stop is-empty",
            "col c-land is-empty"
        ]
    );

    for (fetched, want) in [
        (Fetched::Body(unknown), UNKNOWN_REASON),
        (Fetched::Failed, pipeline::REASON),
        (Fetched::NotRead, NOT_READ),
        (Fetched::Body("{}".to_string()), NO_CONTENT),
        (Fetched::Body("not json".to_string()), NO_CONTENT),
    ] {
        assert_eq!(
            pipeline::body(&fetched),
            Body::Unmeasured(want),
            "{fetched:?}"
        );
        assert_eq!(
            content(&fetched, &ledger, NOW),
            Body::Unmeasured(want),
            "{fetched:?}"
        );
        assert_eq!(cards(&fetched), Err(want));
        assert!(!want.trim().is_empty() && !want.contains('\n'));
    }
    assert_eq!(
        pipeline::body(&Fetched::Body(fixture_text())),
        Body::Filled(())
    );
}

/// (5) 経過の字の 4 つの境（端数は切り捨て）。
#[test]
fn pipe_age_boundaries() {
    let cases = [
        (0, "0s"),
        (59, "59s"),
        (60, "1m"),
        (119, "1m"),
        (3_599, "59m"),
        (3_600, "1h"),
        (86_399, "23h"),
        (86_400, "1d"),
        (172_799, "1d"),
        (172_800, "2d"),
    ];
    for (secs, want) in cases {
        assert_eq!(age(secs), want, "{secs} 秒");
    }
}

/// (6) 札の hover の card の 4 行（題・run と段の名・回数と口座と経過・出所）が期待と一致し、1 行は 36 字以下。
#[test]
fn pipe_hover_card_rows() {
    let cols = filled();
    let hover = |id: &str| {
        cols.iter()
            .flat_map(|c| c.cards.iter())
            .find(|k| k.id == id)
            .unwrap_or_else(|| panic!("札 {id}"))
            .hover
            .rows()
            .map(|(_, t)| t)
    };
    assert_eq!(
        hover("px.5"),
        ["題 5", "run · Questioned", "↻2 · acct-4 · 1h", SOURCE].map(str::to_string)
    );
    assert_eq!(
        hover("px.3"),
        ["px.3", "run · Running", "↻1 · acct-4 · 2h", SOURCE].map(str::to_string)
    );
    assert_eq!(
        kcard(&px11(), &ledger_rows(), NOW).hover.rows().map(|(_, t)| t),
        ["題 11", "run · Landed", "↻1 · ― · ―", SOURCE].map(str::to_string)
    );
    assert_eq!(
        hover("px.2"),
        [LONG_CUT, "run · Blocked", "↻2 · ― · 45s", SOURCE].map(str::to_string)
    );
    assert_eq!(SOURCE, "fleet/events.jsonl");
    for card in cols.iter().flat_map(|c| c.cards.iter()) {
        for (class, text) in card.hover.rows() {
            assert!(
                text.chars().count() <= ROW_CHARS,
                "{} の {class} が 36 字を超える",
                card.id
            );
        }
    }
}

/// (7) 段から列への対応は契約の型の関数（Stage::column）を呼び、面の code に段の名の対応の表を書かない。
#[test]
fn pipe_stage_column_from_contract() {
    let cols = filled();
    let cards = fixture_cards();
    for col in &cols {
        for k in &col.cards {
            let card = cards
                .iter()
                .find(|c| c.contract.as_str() == k.id)
                .expect("fixture の札");
            assert_eq!(card.stage.column(), col.lane.column, "{}", k.id);
        }
    }
    let src = read("src/project/pipeline.rs");
    assert!(src.contains(".column()"), "契約の型の関数を呼ばない");
    for stage in Stage::ALL {
        let name = format!("Stage::{stage:?}");
        assert!(!src.contains(&name), "pipeline.rs に段の名 {name} が在る");
    }
    // 5 列の表は列の全部を 1 度ずつ持つ。
    let mut lanes: Vec<PipelineColumn> = LANES.iter().map(|l| l.column).collect();
    lanes.dedup();
    assert_eq!(lanes.len(), 5);
    for stage in Stage::ALL {
        assert_eq!(pipeline::lane(stage.column()).column, stage.column());
    }
}

/// 語の鍵は語の辞書に、class は stylesheet に在る。
#[test]
fn pipe_keys_in_vocab_and_classes_in_stylesheet() {
    for key in [
        "col_block",
        "col_queue",
        "col_run",
        "col_stop",
        "col_land",
        "pipeline",
        "st_unknown",
        RUN_KEY,
    ] {
        assert!(vocab().term(key).is_some(), "鍵 {key} が vocab に無い");
    }
    let css = read("style.css");
    let has = |name: &str| {
        let dot = format!(".{name}");
        css.match_indices(&dot).any(|(i, _)| {
            css[i + dot.len()..]
                .chars()
                .next()
                .is_none_or(|c| !(c.is_ascii_alphanumeric() || c == '-' || c == '_'))
        })
    };
    let mut used: Vec<String> = Vec::new();
    for col in filled().iter().chain(columns(&[], &[], NOW).iter()) {
        used.extend(col.class.split_whitespace().map(str::to_string));
        for k in &col.cards {
            used.extend(k.class.split_whitespace().map(str::to_string));
        }
    }
    used.extend(
        [
            "board", "cnt", "cards", "more", "kid", "why", "empty", "dot", "num", "t", "tt", "m",
            "st",
        ]
        .map(str::to_string),
    );
    for c in used {
        assert!(has(&c), "stylesheet に class {c} が無い");
    }
}

/// 今 NOW から経過を引いた時刻に段を決めた札。
fn card(id: &str, stage: Stage, elapsed: Option<u64>) -> PipelineCard {
    card_at(id, stage, elapsed, NOW)
}

/// 今 now から経過を引いた時刻（今より前に届かなければ 0）に段を決めた札。
fn card_at(id: &str, stage: Stage, elapsed: Option<u64>, now: EpochSecs) -> PipelineCard {
    PipelineCard {
        contract: BeadId::new(id).expect("id"),
        runs: 1,
        stage,
        reason: None,
        account: None,
        since: elapsed.map(|e| now.saturating_sub(e)),
        ci: None,
    }
}

fn board_body(cards: Vec<PipelineCard>) -> Fetched {
    Fetched::Body(
        wire::encode(&PipelineBoard {
            cards: Reading::Known(cards),
            misfits: Reading::Known(vec![]),
        })
        .expect("電文"),
    )
}

/// 便 g-pipe-today (1)(2): 今日（日本の日）の着地は、段が Landed で経過が在り、今から経過を引いた時刻が今日の始まり以上。
#[test]
fn pipe_landed_today_boundaries() {
    assert_eq!(NOW % 86_400, 43_200, "NOW は UTC の日の正午");
    assert_eq!((NOW + JST_OFFSET) % 86_400, 75_600, "NOW は日本時間の 21:00");
    let cases = [
        (Stage::Landed, Some(3_600), true),
        (Stage::Landed, Some(75_600), true),
        (Stage::Landed, Some(75_601), false),
        (Stage::Landed, None, false),
        (Stage::Running, Some(60), false),
        (Stage::Landed, Some(0), true),
        (Stage::Landed, Some(NOW + 1), false),
    ];
    for (stage, elapsed, want) in cases {
        assert_eq!(
            landed_today(&card("px.1", stage, elapsed), NOW),
            want,
            "{stage:?} {elapsed:?}"
        );
    }
    // 段が Landed でない札は、経過が今日に入っても偽。
    for stage in Stage::ALL.into_iter().filter(|s| *s != Stage::Landed) {
        assert!(!landed_today(&card("px.1", stage, Some(60)), NOW), "{stage:?}");
    }
    // 日の始まりちょうどの今は、経過 0 だけが今日。
    let midnight = NOW - 75_600;
    assert!(landed_today(&card_at("px.1", Stage::Landed, Some(0), midnight), midnight));
    assert!(!landed_today(&card_at("px.1", Stage::Landed, Some(1), midnight), midnight));
    // 今の関数の型（札と今の時刻を受けて真偽）。
    let f: fn(&PipelineCard, EpochSecs) -> bool = landed_today;
    assert!(f(&card("px.1", Stage::Landed, Some(1)), NOW));
}

/// 便 g-pipe-today (3)(4): Landed の列は今日の着地だけ・数の字は今日の着地の数・ほかの 4 列は今の時刻に依らない。
#[test]
fn pipe_landed_column_only_today() {
    let mut cs = fixture_cards();
    cs.push(card("px.20", Stage::Landed, Some(75_601)));
    cs.push(card("px.21", Stage::Landed, Some(17 * 86_400)));
    cs.push(card("px.22", Stage::Landed, Some(75_600)));
    let rows = ledger_rows();
    let cols = columns(&cs, &rows, NOW);
    assert_eq!(
        ids(&cols[4], true),
        vec!["px.12", "px.9", "px.10", "px.8", "px.22"]
    );
    assert_eq!(cols[4].cards.len(), 5, "数の字は今日の着地の数");
    assert!(
        cols[4]
            .cards
            .iter()
            .all(|k| landed_today(cs.iter().find(|c| c.contract.as_str() == k.id).unwrap(), NOW))
    );
    // content も同じ列を組む。
    let Body::Filled(via) = content(&board_body(cs.clone()), &ledger_body(), NOW) else {
        panic!("板が中身を出さない");
    };
    assert_eq!(via, cols);

    // ほかの 4 列の札の入り方と並びは今の時刻に依らない（fixture の並びのまま）。
    // 翌日の始まりの 10 秒後に描くと、いちばん新しい 30 秒前の着地も昨日。
    let later = columns(&cs, &rows, NOW + 86_400 - 75_600 + 10);
    let later_ids: Vec<Vec<&str>> = later[..4].iter().map(|c| ids(c, true)).collect();
    let others: Vec<Vec<&str>> = cols[..4].iter().map(|c| ids(c, true)).collect();
    assert_eq!(others, later_ids);
    assert_eq!(
        others,
        vec![
            vec!["px.2"],
            vec!["px.1"],
            vec!["px.4", "px.3"],
            vec!["px.7", "px.5", "px.6"],
        ]
    );
    // そのときの Landed の列は空。
    assert!(later[4].cards.is_empty());
    assert_eq!(later[4].class, "col c-land is-empty");
}

/// 便 g-pipe-today (5): 今日の着地が 0 本でほかの列に札が在れば、Landed の列は空の列で、板は測れていないにも 0 件にもならない。
#[test]
fn pipe_no_landed_today_is_empty_column() {
    let cs = vec![
        card("px.1", Stage::Queued, Some(300)),
        card("px.8", Stage::Landed, Some(17 * 86_400)),
        card("px.11", Stage::Landed, None),
    ];
    let got = content(&board_body(cs), &ledger_body(), NOW);
    let Body::Filled(cols) = got else {
        panic!("板が中身を出さない: {got:?}");
    };
    assert_eq!(cols.len(), 5);
    assert_eq!(ids(&cols[1], true), vec!["px.1"]);
    assert_eq!(cols[1].class, "col c-queue");
    assert!(cols[4].cards.is_empty());
    assert_eq!(cols[4].class, "col c-land is-empty");
    assert_eq!(cols[4].more(false), None);
}

/// 便 g-pipe-today (6): 描く所は content に描く時の net の now を渡す。
#[test]
fn pipe_view_passes_net_now() {
    let src = read("src/project/pipeline.rs");
    let dom = &src[src.find("mod dom").expect("dom の module")..];
    assert!(dom.contains("crate::net::now()"), "view が net の now を呼ばない");
    assert!(dom.contains("content(p, l, now)"), "view が content に now を渡さない");
}

/// (8) 着地済みの外形（BLOCK・口の path・body の名と引数と返りの型）と、面の crate の直接依存が増えない。
#[test]
fn pipe_landed_shape_and_no_new_deps() {
    assert_eq!(pipeline::BLOCK.id, "pipe");
    assert_eq!(pipeline::BLOCK.heading, "pipeline");
    assert_eq!(pipeline::BLOCK.class, "panel");
    assert_eq!(pipeline::PATH, "/api/pipeline");
    let body: fn(&Fetched) -> Body<()> = pipeline::body;
    assert!(matches!(
        body(&Fetched::Body("{}".to_string())),
        Body::Unmeasured(_)
    ));

    let manifest = read("Cargo.toml");
    let mut names = Vec::new();
    let mut inside = false;
    for line in manifest.lines().map(str::trim) {
        if line.starts_with('[') {
            inside = line.ends_with("dependencies]");
        } else if inside
            && !line.starts_with('#')
            && let Some((k, _)) = line.split_once('=')
            && !k.trim().starts_with('"')
        {
            names.push(k.trim().to_string());
        }
    }
    names.sort();
    assert_eq!(
        names,
        vec![
            "leptos",
            "tsuzuri-boundary",
            "tsuzuri-contract",
            "wasm-bindgen-futures",
            "web-sys"
        ]
    );
}
