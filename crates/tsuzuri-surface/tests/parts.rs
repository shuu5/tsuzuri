//! 便 g-parts の歯: 読みの結果の 3 値・中身の無い 5 つの block は測れていない・口の path は block の module の
//! paths とグラフの口で互いに違う（行 hs-derived で block ごとの定数から導く形にした）・
//! 読み直しの合図の event の名は契約の型の crate の定数から引く・hover の card の置き場と猶予と行の切り方・
//! 定数が rules の file の行 R-20 と行 R-19 の字と同じ。
#![cfg(test)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::{LEDGER_CHANGED_EVENT, LedgerList};
use tsuzuri_contract::question::QuestionList;
use tsuzuri_contract::surface::BOARD_CHANGED_EVENT;
use tsuzuri_contract::wire;
use tsuzuri_surface::mapview::graph;
use tsuzuri_surface::project::{
    Body, Module, NO_CONTENT, NOT_READ, ask, askpage, ledger, map, next, pipeline, seat,
};
use tsuzuri_surface::view::{Fetched, RELOAD_EVENTS, Screen};
use tsuzuri_surface::widgets::hover::{
    Card, ELLIPSIS, GRACE_MS, OFFSET_X, OFFSET_Y, Point, ROW_CHARS, ROWS, Rect, Size, clip, keeps,
    place,
};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// src の下の .rs の file の全部（path の順）。
fn sources() -> Vec<(PathBuf, String)> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir).expect("src を読む").flatten() {
            let p = entry.path();
            if p.is_dir() {
                walk(&p, out);
            } else if p.extension().is_some_and(|x| x == "rs") {
                out.push(p);
            }
        }
    }
    let mut paths = Vec::new();
    walk(&crate_dir().join("src"), &mut paths);
    paths.sort();
    paths
        .into_iter()
        .map(|p| {
            let text = std::fs::read_to_string(&p).expect("src の file");
            (p, text)
        })
        .collect()
}

/// 3 値の全部（match に wildcard を置かないので、値が増減すれば組み立てで落ちる）。
fn three() -> [Fetched; 3] {
    let all = [
        Fetched::NotRead,
        Fetched::Body("{}".to_string()),
        Fetched::Failed,
    ];
    for f in &all {
        match f {
            Fetched::NotRead | Fetched::Body(_) | Fetched::Failed => {}
        }
    }
    all
}

/// 中身の関数がまだ無い 5 つ（block の id・中身の関数・口が読めないときの理由）。
type Pending = (&'static str, fn(&Fetched) -> Body<()>, &'static str);

fn pending_blocks() -> [Pending; 5] {
    [
        (next::BLOCK.id, next::body, next::REASON),
        (pipeline::BLOCK.id, pipeline::body, pipeline::REASON),
        (seat::BLOCK.id, seat::body, seat::REASON),
        (map::BLOCK.id, map::body, map::REASON),
        ("ledger の指標", ledger::metrics, ledger::METRICS_REASON),
    ]
}

/// 5 つの block は 3 値のどれを受けても 0 件でなく測れていないと空でない理由の 1 行を返す。
#[test]
fn parts_pending_blocks_unmeasured_for_all_three() {
    for (id, body, unread) in pending_blocks() {
        for fetched in three() {
            let got = body(&fetched);
            let Body::Unmeasured(reason) = got else {
                panic!("{id} が {fetched:?} で測れていないでない: {got:?}");
            };
            assert!(!reason.trim().is_empty(), "{id} の理由が空");
            assert!(!reason.contains('\n'), "{id} の理由が 1 行でない");
            let want = match fetched {
                Fetched::NotRead => NOT_READ,
                Fetched::Body(_) => NO_CONTENT,
                Fetched::Failed => unread,
            };
            assert_eq!(reason, want, "{id} が {fetched:?} で返す理由");
        }
    }
    assert_eq!(NO_CONTENT, "この block の中身はまだ無い");
}

/// ask・これまでの決定・ledger の一覧も同じ 3 値を受け、まだ読んでいないと読めないは 0 件でなく測れていない。
#[test]
fn parts_ledger_lists_read_three_values() {
    let empty = wire::encode(&LedgerList {
        rows: Reading::Known(vec![]),
    })
    .expect("電文");
    let no_cards = wire::encode(&QuestionList {
        cards: Reading::Known(vec![]),
        answerable: true,
    })
    .expect("電文");
    assert!(matches!(
        ask::body(&Fetched::Body(no_cards)),
        Body::Empty(_)
    ));
    assert!(matches!(
        askpage::body(&Fetched::Body(empty.clone())),
        Body::Empty(_)
    ));
    let known = Screen::initial().after_read(&Fetched::Body(empty), 5);
    for fetched in [Fetched::NotRead, Fetched::Failed] {
        assert!(matches!(ask::body(&fetched), Body::Unmeasured(r) if !r.is_empty()));
        assert!(matches!(askpage::body(&fetched), Body::Unmeasured(r) if !r.is_empty()));
        for screen in [Screen::initial(), known.clone()] {
            let s = screen.after_read(&fetched, 9);
            assert_eq!(s.board, Reading::Unknown, "{fetched:?}");
            assert!(matches!(ledger::body(&s), Body::Unmeasured(r) if !r.is_empty()));
            // 最終更新は読めた時刻のまま（読めない読みで進めない）。
            assert_eq!(s.updated_at, screen.updated_at);
        }
    }
}

/// 口の path は block の module の paths の全部とグラフの module の口で、互いに違う（数と字は持たない・
/// 値は歯 hsderive_ と mapgraph_ が見る・行 hs-derived）。
#[test]
fn parts_paths_distinct_in_block_modules() {
    let owned: Vec<(Module, &str)> = Module::ALL
        .into_iter()
        .flat_map(|m| m.paths().iter().map(move |p| (m, *p)))
        .collect();
    let mut paths: Vec<&str> = owned.iter().map(|(_, p)| *p).collect();
    paths.push(graph::PATH);
    let mut sorted = paths.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), paths.len(), "口の path が重なる: {paths:?}");

    // src の `"/api/…"` の字: block の口はその path を paths に持つ module の file に 1 度だけ・
    // グラフの口の字は mapview の下のグラフの module の file に 1 度だけ・ほかは変化の知らせの口だけ。
    let mut found: BTreeMap<String, Vec<PathBuf>> = BTreeMap::new();
    for (path, text) in sources() {
        let mut rest = text.as_str();
        while let Some(i) = rest.find("\"/api/") {
            rest = &rest[i + 1..];
            let end = rest.find('"').expect("字の終わり");
            found
                .entry(rest[..end].to_string())
                .or_default()
                .push(path.clone());
            rest = &rest[end..];
        }
    }
    for (m, p) in owned {
        let at = found.remove(p).unwrap_or_default();
        assert_eq!(at.len(), 1, "{p} の字が src に 1 度でない: {at:?}");
        let file = crate_dir()
            .join("src/project")
            .join(format!("{}.rs", m.name()));
        assert_eq!(at[0], file, "{p} が {} の module に無い: {at:?}", m.name());
    }
    let at = found.remove(graph::PATH).unwrap_or_default();
    assert_eq!(
        at,
        vec![crate_dir().join("src/mapview/graph.rs")],
        "{} の字がグラフの module に 1 度だけでない",
        graph::PATH
    );
    let rest: Vec<&String> = found.keys().collect();
    assert_eq!(rest, vec!["/api/surface/events"], "block の外の口の字");
}

/// 読み直しの合図の event の名は契約の型の crate の定数 2 つから引き、面の code は字を直に書かない。
/// 知らせの接続は頁に 1 本（src に EventSource を張る所が 1 つだけ）。
#[test]
fn parts_reload_events_from_contract_constants() {
    assert_eq!(LEDGER_CHANGED_EVENT, "ledger-changed");
    assert_eq!(BOARD_CHANGED_EVENT, "board-changed");
    assert_eq!(RELOAD_EVENTS, [LEDGER_CHANGED_EVENT, BOARD_CHANGED_EVENT]);
    let mut opens = 0;
    for (path, text) in sources() {
        for name in [LEDGER_CHANGED_EVENT, BOARD_CHANGED_EVENT] {
            assert!(
                !text.contains(name),
                "{} に event の名 {name} の字が直に在る",
                path.display()
            );
        }
        opens += text.matches("EventSource::new(").count();
    }
    assert_eq!(opens, 1, "知らせの接続を張る所が 1 つでない");
    let net = std::fs::read_to_string(crate_dir().join("src/net.rs")).expect("src/net.rs");
    assert!(
        net.contains("RELOAD_EVENTS"),
        "net が合図の名の一覧を使わない"
    );
}

const WINDOW: Size = Size {
    width: 1280.0,
    height: 800.0,
};

const CARD: Size = Size {
    width: 200.0,
    height: 100.0,
};

fn pt(x: f64, y: f64) -> Point {
    Point { x, y }
}

/// 置き場: 左上を pointer の右 16 px・上 20 px・右端を越えれば pointer の左・上下は窓に収まるまで寄せる。
#[test]
fn parts_hover_place() {
    assert_eq!(place(pt(100.0, 200.0), CARD, WINDOW), pt(116.0, 180.0));
    // 右端にちょうど収まる（1080 + 200 = 1280）ならそのまま。
    assert_eq!(place(pt(1064.0, 200.0), CARD, WINDOW), pt(1080.0, 180.0));
    // 1 px でも越えれば pointer の左 16 px に card の右端を置く（1065 − 16 − 200）。
    assert_eq!(place(pt(1065.0, 200.0), CARD, WINDOW), pt(849.0, 180.0));
    assert_eq!(place(pt(1200.0, 200.0), CARD, WINDOW), pt(984.0, 180.0));
    // 上端を越える: 上端に寄せる。
    assert_eq!(place(pt(100.0, 10.0), CARD, WINDOW), pt(116.0, 0.0));
    assert_eq!(place(pt(100.0, 20.0), CARD, WINDOW), pt(116.0, 0.0));
    // 下端を越える: 下端に収まるまで上へ寄せる（800 − 100）。
    assert_eq!(place(pt(100.0, 790.0), CARD, WINDOW), pt(116.0, 700.0));
    assert_eq!(place(pt(100.0, 820.0), CARD, WINDOW), pt(116.0, 700.0));
    // 下端にちょうど収まるならそのまま（700 + 100 = 800）。
    assert_eq!(place(pt(100.0, 720.0), CARD, WINDOW), pt(116.0, 700.0));
    // 右と下を同時に越える。
    assert_eq!(place(pt(1270.0, 795.0), CARD, WINDOW), pt(1054.0, 700.0));
    // 窓より高い card は上端に揃える・左へ返しても左端を越えるなら左端に寄せる。
    let tall = Size {
        width: 250.0,
        height: 900.0,
    };
    let narrow = Size {
        width: 300.0,
        height: 800.0,
    };
    assert_eq!(place(pt(150.0, 400.0), tall, narrow), pt(0.0, 0.0));
}

/// 猶予: 出てから 150 ms 以内で、card の矩形までの距離が出た点より縮んだときだけ残す。
#[test]
fn parts_hover_grace() {
    let card = Rect {
        left: 116.0,
        top: 180.0,
        width: 200.0,
        height: 100.0,
    };
    let exit = pt(100.0, 200.0);
    assert_eq!(card.distance(exit), 16.0);
    assert_eq!(card.distance(pt(200.0, 200.0)), 0.0);
    assert_eq!(card.distance(pt(113.0, 176.0)), 5.0);
    // 近づいた・150 ms 以内。
    assert!(keeps(exit, pt(108.0, 200.0), card, 0.0));
    assert!(keeps(exit, pt(108.0, 200.0), card, 100.0));
    assert!(keeps(exit, pt(108.0, 200.0), card, f64::from(GRACE_MS)));
    // card の中に入った。
    assert!(keeps(exit, pt(120.0, 220.0), card, 60.0));
    // 150 ms を過ぎた。
    assert!(!keeps(
        exit,
        pt(108.0, 200.0),
        card,
        f64::from(GRACE_MS) + 1.0
    ));
    assert!(!keeps(exit, pt(120.0, 220.0), card, 400.0));
    // 縮んでいない（同じ点・遠ざかった・横へずれて距離が延びた）。
    assert!(!keeps(exit, exit, card, 10.0));
    assert!(!keeps(exit, pt(90.0, 200.0), card, 10.0));
    assert!(!keeps(exit, pt(100.0, 300.0), card, 10.0));
    // 斜めに近づく（右上へ）。
    assert!(keeps(pt(100.0, 300.0), pt(110.0, 290.0), card, 50.0));
}

/// card の行は 4 行で、36 字を超える行は 35 字と「…」に切る。
#[test]
fn parts_hover_rows_clip() {
    let fits = "あ".repeat(ROW_CHARS);
    assert_eq!(clip(&fits), fits);
    let over = "い".repeat(ROW_CHARS + 1);
    let cut = clip(&over);
    assert_eq!(cut.chars().count(), ROW_CHARS);
    assert_eq!(cut, format!("{}{ELLIPSIS}", "い".repeat(ROW_CHARS - 1)));
    assert_eq!(clip(""), "");
    assert_eq!(clip("abc"), "abc");
    let mixed: String = "a字".repeat(30);
    assert_eq!(clip(&mixed).chars().count(), ROW_CHARS);
    assert!(clip(&mixed).ends_with(ELLIPSIS));

    let card = Card {
        title: "題".repeat(50),
        kind: "task".to_string(),
        value: "v".repeat(ROW_CHARS),
        src: "crates/tsuzuri-surface/src/widgets/hover.rs:1".repeat(2),
        more: vec!["詳しく".repeat(30)],
    };
    let rows = card.rows();
    assert_eq!(rows.len(), ROWS);
    let classes: Vec<&str> = rows.iter().map(|(c, _)| *c).collect();
    assert_eq!(classes, vec!["r ti", "r k", "r v", "r src"]);
    for (class, text) in &rows {
        assert!(
            text.chars().count() <= ROW_CHARS,
            "{class} が 36 字を超える"
        );
    }
    assert_eq!(rows[1].1, "task");
    assert_eq!(rows[2].1, card.value);
    assert!(rows[0].1.ends_with(ELLIPSIS));
    assert!(rows[3].1.ends_with(ELLIPSIS));
    // 「詳しく」は行の予算の外（切らない）。
    assert_eq!(card.more[0].chars().count(), 90);
}

/// 行の value の字。
fn value_of(row: &str) -> Option<&str> {
    Some(row.split_once("value: \"")?.1.split_once('"')?.0)
}

/// `marker` の直後の「数 unit」の数（marker が 1 度だけ在るときだけ）。
fn number_after(text: &str, marker: &str, unit: &str) -> Option<u64> {
    let mut parts = text.split(marker);
    parts.next()?;
    let rest = parts.next()?;
    if parts.next().is_some() {
        return None;
    }
    rest.trim_start().split_once(unit)?.0.trim().parse().ok()
}

/// 行 R-20 の字の値（右・上・猶予）。
fn r20(row: &str) -> Option<(u64, u64, u64)> {
    let v = value_of(row)?;
    Some((
        number_after(v, "pointer の右", "px")?,
        number_after(v, "px と上", "px")?,
        number_after(v, "猶予", "ms")?,
    ))
}

/// 行 R-19 の字の hover の card の値（行の数・1 行の字数）。
fn r19(row: &str) -> Option<(u64, u64)> {
    let v = value_of(row)?;
    let mut parts = v.split("hover の card は");
    parts.next()?;
    let card = parts.next()?.split('・').next()?;
    if parts.next().is_some() {
        return None;
    }
    let rows = card.trim_start().split_once('行')?.0.trim().parse().ok()?;
    Some((rows, number_after(card, "（1 行", "字")?))
}

fn rule_row(id: &str) -> String {
    let rules = std::fs::read_to_string(crate_dir().join("../../design-intent/rules.yaml"))
        .expect("design-intent/rules.yaml");
    rules
        .lines()
        .find(|l| l.contains(&format!("id: {id},")))
        .unwrap_or_else(|| panic!("rules の file に行 {id} が在る"))
        .to_string()
}

/// 置き場と猶予と行の字数の定数（16・20・150・4・36）が rules の file の行 R-20 と行 R-19 の字と同じ。
#[test]
fn parts_constants_match_rules_r20_r19() {
    let row20 = rule_row("R-20");
    let want20 = (
        u64::try_from(OFFSET_X).expect("正の数"),
        u64::try_from(OFFSET_Y).expect("正の数"),
        u64::from(GRACE_MS),
    );
    assert_eq!(r20(&row20), Some(want20), "{row20}");
    let row19 = rule_row("R-19");
    let want19 = (
        u64::try_from(ROWS).expect("数"),
        u64::try_from(ROW_CHARS).expect("数"),
    );
    assert_eq!(r19(&row19), Some(want19), "{row19}");
    assert_eq!(want20, (16, 20, 150));
    assert_eq!(want19, (4, 36));
}

/// 読みの関数は字が変われば違う値を返す（定数と食い違えば上の歯が落ちる）。
#[test]
fn parts_rule_readers_follow_the_text() {
    let row20 = r#"  - {id: R-20, what: "x", value: "card は pointer の右 16 px と上 20 px・要素を出てからの猶予 150 ms・光る節点 20 以下", note: "猶予 9 ms"}"#;
    assert_eq!(r20(row20), Some((16, 20, 150)));
    assert_eq!(
        r20(&row20.replace("右 16 px", "右 18 px")),
        Some((18, 20, 150))
    );
    assert_eq!(
        r20(&row20.replace("上 20 px", "上 24 px")),
        Some((16, 24, 150))
    );
    assert_eq!(r20(&row20.replace("150 ms", "200 ms")), Some((16, 20, 200)));
    assert_eq!(r20(&row20.replace("150 ms", "百 ms")), None);
    assert_eq!(r20(&row20.replace("光る", "猶予 3 ms・光る")), None);

    let row19 = r#"  - {id: R-19, what: "x", value: "「?」は 1 行目 40 字 以下と箇条書き 4 項 以下（1 項 20 字 以下）・hover の card は 4 行 以下（1 行 36 字 以下）・経験者向けの 1 行は 60 字 以下", note: "y"}"#;
    assert_eq!(r19(row19), Some((4, 36)));
    assert_eq!(r19(&row19.replace("4 行", "5 行")), Some((5, 36)));
    assert_eq!(r19(&row19.replace("36 字", "40 字")), Some((4, 40)));
    assert_eq!(r19(&row19.replace("hover の card は", "card は")), None);
    // 置き場と猶予の定数を使う関数の値も同じ数に乗る（右 16・上 20）。
    assert_eq!(
        place(pt(0.0, 100.0), CARD, WINDOW),
        pt(f64::from(OFFSET_X), 100.0 - f64::from(OFFSET_Y))
    );
}
