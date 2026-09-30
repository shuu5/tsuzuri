//! 便 g-seat の歯: fixture の 5 組の状態の記号と語と「から」・稼働の記録の 3 つの幅の矩形と縦線・幅の query・
//! 窓ごとの割合・状態の帯・口座の履歴・「まだ分からない」の欄だけ測れていない・口が読めない・着地済みの外形と依存。
#![cfg(test)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use tsuzuri_contract::board::{GroupRow, Reading};
use tsuzuri_contract::seat::{AccountMove, QuotaUsed, SeatCard, SeatSpan, SeatState};
use tsuzuri_contract::wire;
use tsuzuri_surface::project::seat::{
    self, Band, HistRow, LIMIT_LINE, MORE, MOVE_WAIT, NEXT_TARGET, NG, OK, Rect, Seat, Span,
    SHORT, WINDOWS, content, span_of, strip_svg, until, with_span,
};
use tsuzuri_surface::project::{Body, NO_CONTENT, NOT_READ};
use tsuzuri_surface::view::Fetched;
use tsuzuri_surface::vocab::vocab;

/// fixture の組の時点（2026-09-27 12:00Z・日本時間の 21:00）。
const AT: u64 = 1_790_510_400;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// fixture の 5 組（組の名 → 電文の型）。
fn fixture() -> BTreeMap<String, SeatCard> {
    wire::decode(&read("../../tests/fixtures/surface/seat-card.json"))
        .expect("fixture の組が電文として読める")
}

fn card(name: &str) -> SeatCard {
    fixture()
        .remove(name)
        .unwrap_or_else(|| panic!("fixture の組 {name}"))
}

/// 組の電文の字を口の本文として読んだ中身。
fn filled(name: &str) -> Seat {
    let text = wire::encode(&card(name)).expect("電文");
    match content(&Fetched::Body(text), card(name).at) {
        Body::Filled(s) => s,
        other => panic!("組 {name} が中身を出さない: {other:?}"),
    }
}

/// (1) fixture の 5 組で、状態の記号の class と語の鍵と語の class と「から」の時刻の字が期待と一致する。
#[test]
fn seatblock_state_icon_key_and_since_per_set() {
    let names: Vec<String> = fixture().into_keys().collect();
    assert_eq!(names, vec!["limit", "run", "silent", "unknown", "wait"]);
    let cases: [(&str, &str, &str, &str, Option<&str>); 5] = [
        (
            "run",
            "st st-run lg",
            "st_run",
            "state stlabel st-run",
            Some("18:50 JST"),
        ),
        (
            "wait",
            "st st-wait lg",
            "st_wait",
            "state stlabel st-wait",
            Some("20:35 JST"),
        ),
        (
            "limit",
            "st st-limit lg",
            "st_limit",
            "state stlabel st-limit",
            Some("19:00 JST"),
        ),
        (
            "silent",
            "st st-silent lg",
            "st_silent",
            "state stlabel st-silent",
            None,
        ),
        (
            "unknown",
            "st st-unknown lg",
            "st_unknown",
            "state stlabel st-unknown",
            Some("20:59 JST"),
        ),
    ];
    for (name, icon, key, label_class, since) in cases {
        let top = filled(name).top;
        assert_eq!(top.state, name, "{name} の値");
        assert_eq!(top.icon_class, icon, "{name} の記号の class");
        assert_eq!(top.key, key, "{name} の語の鍵");
        assert_eq!(top.label_class, label_class, "{name} の語の class");
        assert_eq!(top.since.as_deref(), since, "{name} の「から」");
    }
    // 状態は電文の値を写すだけ（5 値が 5 つの値に 1 対 1）。
    let values: Vec<&str> = SeatState::ALL.into_iter().map(seat::state_value).collect();
    assert_eq!(values, vec!["run", "wait", "limit", "silent", "unknown"]);
    assert_eq!(seat::hm(AT), "21:00 JST");
    // 合図の行: 健康なら ✓・健康でなければ !・heartbeat は on か off。
    let run = filled("run").top;
    assert_eq!(run.tick, Reading::Known(OK));
    assert_eq!(run.heartbeat, Reading::Known("on"));
    let wait = filled("wait").top;
    assert_eq!(wait.tick, Reading::Known(NG));
    assert_eq!(wait.heartbeat, Reading::Known("off"));
    assert_eq!((OK.glyph, NG.glyph), ("✓", "!"));
}

/// 矩形の左端・幅・class。
type Shape = (f64, f64, &'static str);

fn rect(x: f64, width: f64, class: &'static str) -> Shape {
    (x, width, class)
}

fn shape(rects: &[Rect]) -> Vec<Shape> {
    rects.iter().map(|r| (r.x, r.width, r.class)).collect()
}

/// (2) 稼働の記録は 3 つの幅で、区間の矩形の位置と幅（窓の端で切る・1 未満は 1）と口座の移動の縦線の位置が期待と一致する。
#[test]
fn seatblock_strip_rects_and_marks_per_span() {
    let s = filled("run");
    let want: [(Span, Vec<Shape>, Vec<f64>); 3] = [
        (
            Span::H24,
            vec![
                rect(0.0, 144.0, "sg-wait"),
                rect(144.0, 24.0, "sg-limit"),
                rect(168.0, 102.0, "sg-wait"),
                rect(270.0, 1.0, "sg-silent"),
                rect(270.2, 17.8, "sg-run"),
            ],
            vec![121.0, 264.0],
        ),
        (
            Span::H6,
            vec![
                rect(0.0, 216.0, "sg-wait"),
                rect(216.0, 1.0, "sg-silent"),
                rect(216.8, 71.2, "sg-run"),
            ],
            vec![192.0],
        ),
        (
            Span::H3,
            vec![
                rect(0.0, 144.0, "sg-wait"),
                rect(144.0, 1.6, "sg-silent"),
                rect(145.6, 142.4, "sg-run"),
            ],
            vec![96.0],
        ),
    ];
    for (span, rects, marks) in want {
        let strip = s.strip(span);
        let Reading::Known(got) = &strip.rects else {
            panic!("{span:?} の矩形が測れていない");
        };
        assert_eq!(shape(got), rects, "{span:?} の矩形");
        assert_eq!(strip.marks, Reading::Known(marks), "{span:?} の縦線");
        // 矩形は 288 の幅に収まる。
        assert!(
            got.iter()
                .all(|r| r.x >= 0.0 && r.x + r.width <= 288.0 + 1e-9)
        );
    }
    let spans: Vec<Span> = s.strips.iter().map(|x| x.span).collect();
    assert_eq!(spans, Span::ALL.to_vec());
    // 待っている区間は低く、動いている区間は高さの全部。
    let Reading::Known(r24) = &s.strip(Span::H24).rects else {
        panic!()
    };
    assert!(r24[0].height < r24[4].height);
    assert_eq!(r24[4].height, seat::HEIGHT);
    assert_eq!(r24[4].y + r24[4].height, seat::HEIGHT);

    // 窓の右端は電文の at（右端を越える区間は at で切る・窓の外の移動は線を置かない）。
    let spans = [SeatSpan {
        from: AT - 600,
        to: AT + 600,
        state: SeatState::Run,
    }];
    let cut = seat::rects(&spans, AT, Span::H24);
    assert_eq!(shape(&cut), vec![rect(286.0, 2.0, "sg-run")]);
    let moves = [
        AccountMove {
            at: AT + 1,
            from: None,
            to: "b".to_string(),
        },
        AccountMove {
            at: AT - 86_401,
            from: None,
            to: "a".to_string(),
        },
        AccountMove {
            at: AT,
            from: None,
            to: "c".to_string(),
        },
    ];
    assert_eq!(seat::marks(&moves, AT, Span::H24), vec![288.0]);
}

/// (2) query の字から幅を読む関数と幅を query の字に写す関数（3 つの幅・知らない値・値なし）。
#[test]
fn seatblock_span_query_read_and_write() {
    for (search, want) in [
        ("", Span::H24),
        ("?span=24h", Span::H24),
        ("?span=6h", Span::H6),
        ("?span=3h", Span::H3),
        ("?span=12h", Span::H24),
        ("?span=", Span::H24),
        ("?span", Span::H24),
        ("?mode=expert&span=3h", Span::H3),
    ] {
        assert_eq!(span_of(search), want, "{search}");
    }
    assert_eq!(with_span("", Span::H6), "?span=6h");
    assert_eq!(with_span("?span=6h", Span::H3), "?span=3h");
    assert_eq!(
        with_span("?mode=expert&span=3h&page=home", Span::H24),
        "?mode=expert&span=24h&page=home"
    );
    assert_eq!(with_span("?mode=expert", Span::H3), "?mode=expert&span=3h");
    for span in Span::ALL {
        assert_eq!(span_of(&with_span("?mode=beginner", span)), span);
    }
    let keys: Vec<&str> = Span::ALL.into_iter().map(Span::key).collect();
    assert_eq!(keys, vec!["24h", "6h", "3h"]);
}

/// (3) 窓ごとの割合: 使った割合と戻るまでの字・数えない窓は薄く・100 以上と 80 を超える棒は別の class。
#[test]
fn seatblock_usage_rows_and_bar_classes() {
    let rows = |name: &str| {
        let Reading::Known(rows) = filled(name).low.usage else {
            panic!("{name} の割合が測れていない");
        };
        rows.into_iter()
            .map(|r| (r.key, r.class, r.used, r.width, r.bar_class, r.reset))
            .collect::<Vec<_>>()
    };
    let s = |x: &str| x.to_string();
    assert_eq!(
        rows("run"),
        vec![
            (Some("five_hour"), "wrow", s("42%"), 42, "", s("30m")),
            (Some("seven_day"), "wrow", s("85%"), 85, "w80", s("3h")),
            (
                Some("seven_day_model"),
                "wrow muted",
                s("100%"),
                100,
                "w100",
                s("2d")
            ),
        ]
    );
    assert_eq!(
        rows("limit"),
        vec![
            (Some("five_hour"), "wrow", s("100%"), 100, "w100", s("45m")),
            (Some("seven_day"), "wrow", s("81%"), 81, "w80", s("―")),
            (Some("seven_day_model"), "wrow", s("80%"), 80, "", s("1d")),
        ]
    );
    assert_eq!(
        rows("silent"),
        vec![(Some("seven_day"), "wrow muted", s("12%"), 12, "", s("0m"))]
    );
    // 戻るまでの字の境目（切り捨て）と、戻る時刻が過ぎた窓。
    for (secs, want) in [
        (0, "0m"),
        (59, "0m"),
        (3_599, "59m"),
        (3_600, "1h"),
        (86_399, "23h"),
        (86_400, "1d"),
        (3 * 86_400 + 5, "3d"),
    ] {
        assert_eq!(until(AT, Some(AT + secs)), want, "{secs}");
    }
    assert_eq!(until(AT, Some(AT - 10)), "0m");
    assert_eq!(until(AT, None), "―");
    // 棒の class の境目と、100 を超える割合の棒の長さ。
    for (pct, want) in [
        (0, ""),
        (80, ""),
        (81, "w80"),
        (99, "w80"),
        (100, "w100"),
        (140, "w100"),
    ] {
        assert_eq!(seat::bar_class(pct), want, "{pct}");
    }
    let over = seat::window_row(
        &QuotaUsed {
            window: "five_hour".to_string(),
            used_pct: 140,
            resets_at: None,
            counted: true,
        },
        AT,
    );
    assert_eq!((over.used.as_str(), over.width), ("140%", 100));
    // 知らない窓の名は語の鍵を持たず、字のまま出す。
    let other = seat::window_row(
        &QuotaUsed {
            window: "one_hour".to_string(),
            used_pct: 1,
            resets_at: None,
            counted: true,
        },
        AT,
    );
    assert_eq!((other.key, other.window.as_str()), (None, "one_hour"));
}

/// 便 g-seat-fix: 窓の名の欄は短い字（5h・7d・model・知らない窓は電文の字のまま）で、表は 5 字以下の定数。
#[test]
fn seatblock_window_short_names() {
    assert_eq!(
        SHORT,
        [
            ("five_hour", "5h"),
            ("seven_day", "7d"),
            ("seven_day_model", "model")
        ]
    );
    assert_eq!(SHORT.map(|(w, _)| w), WINDOWS);
    for (_, s) in SHORT {
        assert!(!s.is_empty() && s.chars().count() <= 5, "短い字 {s}");
    }
    let row = |window: &str| {
        seat::window_row(
            &QuotaUsed {
                window: window.to_string(),
                used_pct: 9,
                resets_at: None,
                counted: true,
            },
            AT,
        )
    };
    for (window, want, key) in [
        ("five_hour", "5h", Some("five_hour")),
        ("seven_day", "7d", Some("seven_day")),
        ("seven_day_model", "model", Some("seven_day_model")),
        ("one_hour", "one_hour", None),
    ] {
        let r = row(window);
        assert_eq!((r.short.as_str(), r.key), (want, key), "{window}");
        assert_eq!(r.window, window);
        assert_eq!(seat::short(window), want);
    }
    // fixture の窓の行も短い字を持つ。
    let Reading::Known(rows) = filled("run").low.usage else {
        panic!("run の割合が測れていない");
    };
    assert!(!rows.is_empty());
    for r in rows {
        assert_eq!(r.short, seat::short(&r.window));
    }
}

/// (4) 状態の帯は、限度のときと登録の口座が群の今の口座と違うときだけ出し、平時は出さない。
#[test]
fn seatblock_band_only_on_limit_or_move_wait() {
    assert_eq!(
        filled("run").band,
        None,
        "平時（次の移り先が在っても出さない）"
    );
    assert_eq!(filled("silent").band, None);
    assert_eq!(filled("unknown").band, None);
    assert_eq!(
        filled("wait").band,
        Some(Band {
            l1: vec![format!("{MOVE_WAIT} acct-4 → acct-5")],
            l2: None,
        })
    );
    assert_eq!(
        filled("limit").band,
        Some(Band {
            l1: vec![LIMIT_LINE.to_string()],
            l2: Some(format!("{NEXT_TARGET} acct-6")),
        })
    );
    // 限度で、しかも移動待ち（2 つとも出し、次の移り先も出す）。
    let mut both = card("limit");
    both.account = Some("acct-9".to_string());
    assert_eq!(
        seat::band(&both),
        Some(Band {
            l1: vec![
                LIMIT_LINE.to_string(),
                format!("{MOVE_WAIT} acct-9 → acct-4")
            ],
            l2: Some(format!("{NEXT_TARGET} acct-6")),
        })
    );
    // 群が分からないときは移動待ちと判じない（限度でなければ帯を出さない）。
    let mut lost = card("wait");
    lost.group = Reading::Unknown;
    assert_eq!(seat::band(&lost), None);
    // 帯の字（本文の字）は見本の語を持つ。
    assert!(LIMIT_LINE.starts_with("限度で止まっている"));
    assert!(MOVE_WAIT.starts_with("移動待ち"));
    assert!(NEXT_TARGET.starts_with("次の移り先"));
    assert!(MORE.starts_with("詳しく"));
}

/// (5) 口座の履歴は新しい順で、件数を持つ（日本の日が別の日の移動は月日を前に付ける）。
#[test]
fn seatblock_history_newest_first_with_count() {
    let Reading::Known(rows) = filled("run").hist else {
        panic!("run の履歴が測れていない");
    };
    assert_eq!(
        rows,
        vec![
            HistRow {
                at: "19:00 JST".to_string(),
                from: Some("acct-3".to_string()),
                to: "acct-4".to_string(),
            },
            HistRow {
                at: "07:05 JST".to_string(),
                from: None,
                to: "acct-3".to_string(),
            },
        ]
    );
    assert_eq!(filled("wait").hist, Reading::Known(vec![]));
    let Reading::Known(limit) = filled("limit").hist else {
        panic!()
    };
    assert_eq!(limit.len(), 1);
    assert_eq!(limit[0].at, "09-26 17:13 JST");
    // 電文の順が古い順でも新しい順でも、出すのは新しい順。
    let mut rev = card("run");
    if let Reading::Known(m) = &mut rev.moves {
        m.reverse();
    }
    assert_eq!(seat::hist(&rev), filled("run").hist);
}

/// (6) 「まだ分からない」の欄はその欄だけ測れていない（0 や空で埋めない）・読めた欄は出す。
#[test]
fn seatblock_unknown_parts_only_unmeasured() {
    let s = filled("silent");
    assert_eq!(s.top.tick, Reading::Unknown);
    assert_eq!(s.top.heartbeat, Reading::Unknown);
    assert_eq!(s.low.account, None);
    assert_eq!(s.low.group, Reading::Unknown);
    assert_eq!(s.hist, Reading::Unknown);
    assert_eq!(s.more.current, Reading::Unknown);
    assert_eq!(s.more.same, Some(Reading::Unknown));
    // 読めた欄は出す（状態・割合・0 本の区間は 0 本の矩形）。
    assert_eq!(s.top.key, "st_silent");
    assert!(matches!(&s.low.usage, Reading::Known(r) if r.len() == 1));
    for span in Span::ALL {
        assert_eq!(s.strip(span).rects, Reading::Known(vec![]));
        assert_eq!(s.strip(span).marks, Reading::Unknown);
    }

    let u = filled("unknown");
    assert_eq!(u.low.account.as_deref(), Some("acct-4"));
    assert_eq!(u.low.usage, Reading::Unknown);
    assert_eq!(u.more.target, "tsuzuri-orch");
    for span in Span::ALL {
        assert_eq!(u.strip(span).rects, Reading::Unknown);
    }

    let w = filled("wait");
    assert_eq!(w.low.usage, Reading::Unknown);
    assert_eq!(w.strip(Span::H24).rects, Reading::Unknown);
    assert_eq!(w.strip(Span::H24).marks, Reading::Known(vec![]));
    assert_eq!(w.low.group, Reading::Known("tsuzuri-g".to_string()));
    assert_eq!(w.more.current, Reading::Known("acct-5".to_string()));
    assert_eq!(w.more.same, Some(Reading::Known(NG)));
    assert_eq!(filled("run").more.same, Some(Reading::Known(OK)));
}

/// (7)(8) 口が読めない・まだ読んでいない・電文として読めない本文は、block の全体が測れていないと理由の 1 行。
#[test]
fn seatblock_unreadable_is_unmeasured() {
    for (fetched, want) in [
        (Fetched::NotRead, NOT_READ),
        (Fetched::Failed, seat::REASON),
        (Fetched::Body("{}".to_string()), NO_CONTENT),
        (Fetched::Body("not json".to_string()), NO_CONTENT),
    ] {
        assert_eq!(seat::body(&fetched), Body::Unmeasured(want), "{fetched:?}");
        assert_eq!(content(&fetched, 0), Body::Unmeasured(want), "{fetched:?}");
        assert_eq!(seat::card(&fetched), Err(want));
        assert!(!want.trim().is_empty() && !want.contains('\n'));
    }
    for c in fixture().into_values() {
        let text = wire::encode(&c).expect("電文");
        assert_eq!(seat::body(&Fetched::Body(text)), Body::Filled(()));
    }
}

/// strip の SVG の字: 横の幅 288・矩形・口座の移動の縦線・右端の今の線。
#[test]
fn seatblock_strip_svg_text() {
    let rects = [Rect {
        x: 0.0,
        y: 0.0,
        width: 144.5,
        height: 28.0,
        class: "sg-run",
    }];
    let svg = strip_svg(&rects, &[96.0]);
    assert!(svg.starts_with("<svg viewBox=\"0 0 288 28\""), "{svg}");
    assert!(svg.contains("<rect class=\"sg-run\" x=\"0\" y=\"0\" width=\"144.5\" height=\"28\"/>"));
    assert!(svg.contains("<line class=\"mk mk-acct\" x1=\"96\" x2=\"96\" y1=\"0\" y2=\"28\"/>"));
    assert!(svg.contains("<line class=\"mk mk-now\" x1=\"287\""));
    assert!(svg.ends_with("</svg>"));
}

/// 語の鍵（19 個）は語の辞書に在り、組む class は stylesheet に在る。
#[test]
fn seatblock_keys_in_vocab_and_classes_in_stylesheet() {
    let keys = [
        "orch_acct",
        "since",
        "tick_health",
        "heartbeat",
        "history",
        "span",
        "reg_account",
        "group",
        "current_account",
        "acct_hist",
        "allowance",
        "five_hour",
        "seven_day",
        "seven_day_model",
        "st_run",
        "st_wait",
        "st_limit",
        "st_silent",
        "st_unknown",
    ];
    for k in keys {
        assert!(vocab().term(k).is_some(), "鍵 {k} が vocab に無い");
    }
    assert_eq!(WINDOWS.to_vec(), keys[11..14].to_vec());
    assert_eq!(seat::BLOCK.heading, "orch_acct");

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
    for s in fixture().keys().map(|n| filled(n)) {
        used.extend(s.top.icon_class.split_whitespace().map(str::to_string));
        used.extend(s.top.label_class.split_whitespace().map(str::to_string));
        if let Reading::Known(rows) = &s.low.usage {
            for r in rows {
                used.extend(r.class.split_whitespace().map(str::to_string));
                used.extend(r.bar_class.split_whitespace().map(str::to_string));
            }
        }
        for strip in &s.strips {
            if let Reading::Known(rects) = &strip.rects {
                for r in rects {
                    used.extend(r.class.split_whitespace().map(str::to_string));
                }
            }
        }
    }
    used.extend(
        [
            "olow", "big", "tkrow", "tkhb", "strip", "usage", "wrow", "meter", "fold", "ahistd",
            "gmore", "sband", "mk", "mk-acct", "mk-now", "gi", "ok", "ng",
        ]
        .map(str::to_string),
    );
    // 上段の class は見本の字のまま（見本の ui.css にも規則が無い）。
    assert_eq!(seat::OROW, "orow");
    for c in ["w80", "w100", "muted", "sg-limit", "stlabel", "lg"] {
        assert!(
            used.iter().any(|u| u == c),
            "組の中身が class {c} を使わない"
        );
    }
    for c in used {
        assert!(has(&c), "stylesheet に class {c} が無い");
    }
}

/// (8) 着地済みの外形（BLOCK・口の path・body の名と引数と返りの型）と、面の crate の直接依存が増えない。
#[test]
fn seatblock_landed_shape_and_no_new_deps() {
    assert_eq!(seat::BLOCK.id, "orch");
    assert_eq!(seat::BLOCK.heading, "orch_acct");
    assert_eq!(seat::BLOCK.class, "panel seatcard");
    assert_eq!(seat::PATH, "/api/seat");
    let body: fn(&Fetched) -> Body<()> = seat::body;
    assert!(matches!(
        body(&Fetched::Body("{}".to_string())),
        Body::Unmeasured(_)
    ));
    // 群の型は契約の型のまま読む（面は群の行を組み直さない）。
    let g: GroupRow = match card("run").group {
        Reading::Known(g) => g,
        Reading::Unknown => panic!("run の群が測れていない"),
    };
    assert_eq!(g.next_account.as_deref(), Some("acct-5"));

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
