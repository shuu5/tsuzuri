//! 吹き出しの概要の歯（行 g-pop-sum・接頭辞 ppsum_・判断の記録 ADR-30 決定 (2)(3)・規則の行 R-19）。
//! 開いた bead の 1 本の引きの電文を with_sum に通し、2 つの表示の型で概要の選びと印と、読めない間のまだ分からないを断言し、
//! 畳みの字と口の語の鍵と、層の DOM（wasm の枝）の 1 本の引きの読みと台帳の字の印の字を断言する。
#![cfg(test)]

use crate::common::{E, P, X, read};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::{BeadId, LedgerItem, LedgerRow};
use tsuzuri_contract::wire;
use tsuzuri_surface::frame::Mode;
use tsuzuri_surface::view::Fetched;
use tsuzuri_surface::vocab::vocab;
use tsuzuri_surface::widgets::pop::{
    Pop, SUM_CLASS, SUM_KEYS, Src, Sum, item_path, pop, sum_text, with_sum,
};
use tsuzuri_surface::widgets::sumpick::{BODY_KEY, ENG_KEY, PLAIN_KEY, Picked};

/// 1 本の引きの電文（id と本文だけを替える）。
fn item(id: &str, description: &str) -> Fetched {
    let row = LedgerRow {
        id: BeadId::new(id).expect("bead の id"),
        kind: "task".to_string(),
        title: format!("{id} の題"),
        status: "open".to_string(),
        updated_at: 1_790_000_000,
        parent: None,
        labels: Vec::new(),
    };
    let it = LedgerItem {
        row,
        description: description.to_string(),
        notes: String::new(),
    };
    Fetched::Body(wire::encode(&it).expect("電文に書ける"))
}

/// 口が全部読めない吹き出し（概要の欄だけを見る）。
fn base(id: &str) -> Pop {
    let src = Src {
        facts: Reading::Unknown,
        rows: Reading::Unknown,
        cards: &[],
        parts: Reading::Unknown,
        graph: None,
    };
    pop(id, &src)
}

fn sum(item: &Fetched, mode: Mode) -> Sum {
    with_sum(base("t-1"), item, mode).summary
}

fn picked(key: &'static str, text: &str, marked: bool) -> Sum {
    Sum::Picked(Picked {
        key,
        text: text.to_string(),
        marked,
    })
}

/// 2 つの概要の字が違う本文（定型行の形と見出しの形）で、初心者は非エンジニア向けだけ・経験者はエンジニア向けだけを
/// 印なしで切らずに置く（200 字を越える字も切らない）。
#[test]
fn ppsum_item_mode() {
    let long = format!("{P}{}", "あ".repeat(240));
    let typed = format!("# 題\n\n概要 = {long}\n技術 = {E}\n\n本文の行\n");
    let heads = format!("## 概要\n{long}\n\n## 技術\n{E}\n");
    for desc in [typed, heads] {
        let it = item("t-1", &desc);
        assert_eq!(sum(&it, Mode::Beginner), picked(PLAIN_KEY, &long, false));
        assert_eq!(sum(&it, Mode::Expert), picked(ENG_KEY, E, false));
    }
}

/// 表示の型の側の概要が無ければもう一方を印つきで、2 つとも無ければ本文の頭の 1 行を印 sum_body つきで置き、
/// それも無ければ Empty（要約なし）。
#[test]
fn ppsum_fallback_marks() {
    let only_plain = item("t-1", &format!("概要 = {P}\n\n{X}\n"));
    assert_eq!(sum(&only_plain, Mode::Expert), picked(PLAIN_KEY, P, true));
    let only_eng = item("t-1", &format!("技術 = {E}\n\n{X}\n"));
    assert_eq!(sum(&only_eng, Mode::Beginner), picked(ENG_KEY, E, true));
    let body = item("t-1", &format!("# 題\n\n{X}\n"));
    for mode in [Mode::Beginner, Mode::Expert] {
        assert_eq!(sum(&body, mode), picked(BODY_KEY, X, true));
    }
    assert_eq!(BODY_KEY, "sum_body");
    let empty = item("t-1", "# 題\n\n");
    assert_eq!(sum(&empty, Mode::Beginner), Sum::Empty);
}

/// 引きをまだ読めない・口が読めない・電文が読めない・ほかの bead の電文の間はまだ分からない（Unread）。
/// pop は事実の口から概要を読まず、with_sum の前は Unread。
#[test]
fn ppsum_unread_unknown() {
    let desc = format!("概要 = {P}\n技術 = {E}\n");
    assert_eq!(base("t-1").summary, Sum::Unread);
    assert_eq!(
        sum(&item("t-1", &desc), Mode::Beginner),
        picked(PLAIN_KEY, P, false)
    );
    for f in [
        Fetched::NotRead,
        Fetched::Failed,
        Fetched::Body("{".to_string()),
        item("t-2", &desc),
    ] {
        assert_eq!(sum(&f, Mode::Beginner), Sum::Unread, "{f:?}");
    }
}

/// 200 字以下は全文で口なし。201 字は畳んだ形が頭の 199 字と … で口は開く口の鍵、開いた形は全文で口は畳む口の鍵。
/// 口の語は語の辞書に在り、口の class は stylesheet に在る。
#[test]
fn ppsum_fold_open() {
    let at = "あ".repeat(200);
    assert_eq!(sum_text(&at, false), (at.clone(), None));
    assert_eq!(sum_text(&at, true), (at.clone(), None));
    let over = format!("{}い", "あ".repeat(200));
    let (shut, key) = sum_text(&over, false);
    assert_eq!(shut, format!("{}…", "あ".repeat(199)));
    assert_eq!(shut.chars().count(), 200);
    assert_eq!(key, Some(SUM_KEYS[0]));
    assert_eq!(sum_text(&over, true), (over.clone(), Some(SUM_KEYS[1])));
    assert_eq!(SUM_KEYS, ["psum_more", "psum_less"]);
    for (k, want) in SUM_KEYS.iter().zip(["続きを読む", "短く畳む"]) {
        let t = vocab()
            .term(k)
            .unwrap_or_else(|| panic!("{k} が語の辞書に無い"));
        assert_eq!(t.label, want, "{k}");
        assert!(!t.note.is_empty(), "{k}");
    }
    assert_eq!(SUM_CLASS, "pmore");
    assert!(read("style.css").contains(".pmore { "));
}

/// 吹き出しの層は開いた bead の 1 本の引きの口を path の signal で読み（閉じていれば空の path で読まない）、
/// with_why の後に with_sum で概要を置き、口を押すたびに概要を畳んだ形に戻す。
#[test]
fn ppsum_reads_item_on_open() {
    assert_eq!(item_path("t-1.2"), "/api/ledger/t-1.2");
    let src = read("src/widgets/pop.rs");
    let layer = &src[src.find("pub fn PopLayer(").expect("層")..];
    for s in [
        "Signal::derive(move || ctx.shown().map(|id| item_path(&id)).unwrap_or_default());",
        "let item = crate::net::read_path(item_at);",
        "let p = with_why(p, &src);\n            let p = item.with(|(f, _)| with_sum(p, f, mode));",
    ] {
        assert_eq!(layer.matches(s).count(), 1, "{s}");
    }
    let press = &src[src.find("pub fn press(").expect("口の押し")..];
    let press = &press[..press.find("pub fn close(").expect("押しの終わり")];
    assert!(press.contains("self.whole.set(false);"));
}

/// 概要の字の置き場は台帳の字の印（data-ledger-text）を持ち、畳む字の口は押すと開き閉じを替える。吹き出しは概要を
/// sum_view で 1 つだけ描く。
#[test]
fn ppsum_ledger_text_mark() {
    let src = read("src/widgets/pop.rs");
    let at = src.find("fn sum_view(").expect("概要の描き");
    let view = &src[at..at + src[at..].find("fn val_view(").expect("描きの終わり")];
    assert_eq!(
        view.matches("<span data-ledger-text=\"\">{move || shown().0}</span>")
            .count(),
        1
    );
    assert!(view.contains("class=SUM_CLASS on:click=move |_| ctx.whole.update(|w| *w = !*w)"));
    let dom = &src[src.find("fn pop_view(").expect("吹き出しの描き")..];
    assert_eq!(dom.matches("{sum_view(ctx, p.summary)}").count(), 1);
    assert!(!src.contains("<div class=sum>"));
}
