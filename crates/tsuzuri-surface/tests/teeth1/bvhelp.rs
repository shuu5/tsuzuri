//! 行 g-help-sweep の歯（接頭辞 bvhelp_）: 窓の題を語の鍵の見出しにし（題の横の数は見出しの外）、質問の窓の空の字を直し、
//! 最初の案内を 1 枚の画面の部品に合わせ（見える的・帯の下の字の帯・幅と scroll の後の置き直し）、
//! 記号の見方の窓に画面の見方（1 枚の画面・上の帯・窓・吹き出し・幅の 3 段・段の tile）の語と注釈と図を置く。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_surface::askwin::{Phase, TITLE_KEY, count_text, remaining};
use tsuzuri_surface::project::ask;
use tsuzuri_surface::project::legend::{HOW, HOW_KEY};
use tsuzuri_surface::topbar::Win;
use tsuzuri_surface::vocab::{label, vocab};
use tsuzuri_surface::widgets::coach::{Rect, STEPS, again, parts, visible};
use tsuzuri_surface::widgets::fig;
use tsuzuri_surface::widgets::modal::CLASSES;
use tsuzuri_surface::wins::frame_of;

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// wasm の枝（`mod dom {` から後）の字。
fn dom(rel: &str) -> String {
    let text = read(rel);
    let at = text
        .find("mod dom {")
        .unwrap_or_else(|| panic!("{rel} に mod dom"));
    text[at..].to_string()
}

/// `text` の中の `head` で始まる関数か型の字（`head` の行と同じ字下げの閉じ括弧の行の前まで）。
fn fn_of<'a>(text: &'a str, head: &str) -> &'a str {
    let at = text
        .find(head)
        .unwrap_or_else(|| panic!("{head} が無い"));
    let line = text[..at].rfind('\n').map_or(0, |n| n + 1);
    let indent = &text[line..at];
    assert!(indent.trim().is_empty(), "{head} が行の頭に無い");
    let rest = &text[at..];
    let close = format!("\n{indent}}}\n");
    &rest[..rest.find(&close).unwrap_or_else(|| panic!("{head} の閉じ"))]
}

/// 空白を除いた字（rustfmt の折り方によらず式の並びを照らす）。
fn squash(s: &str) -> String {
    s.split_whitespace().collect()
}

/// stylesheet の中で `rule` はちょうど 1 つで、注の外の media などの塊の外（波括弧の深さ 0）に在る。在りかを返す。
fn top(css: &str, rule: &str) -> usize {
    assert_eq!(css.matches(rule).count(), 1, "stylesheet の {rule} の数");
    let at = css.find(rule).unwrap_or_default();
    let mut plain = String::new();
    let mut rest = &css[..at];
    while let Some(i) = rest.find("/*") {
        plain.push_str(&rest[..i]);
        let end = rest[i..]
            .find("*/")
            .unwrap_or_else(|| panic!("{rule} が注の中"));
        rest = &rest[i + end + 2..];
    }
    plain.push_str(rest);
    assert_eq!(
        plain.matches('{').count(),
        plain.matches('}').count(),
        "{rule} が塊の中"
    );
    at
}

fn note(key: &str) -> String {
    vocab()
        .term(key)
        .unwrap_or_else(|| panic!("語彙に {key} が無い"))
        .note
        .clone()
}

const WINS: [Win; 8] = [
    Win::Ask,
    Win::Stalled,
    Win::Notices,
    Win::Seat,
    Win::Gaps,
    Win::Legend,
    Win::Dest,
    Win::Consult,
];

/// 質問の窓の題の横の残りの数（局面の残りを count_text で出す span）。
const NUM_SPAN: &str = "<span class=\"num\">{move || count_text(ids.with(|i| answered.with(|a| remaining(state.get(), i, a))))}</span>";

/// 窓の層は題の帯（class mh）の中で、戻る口・h3 の語の鍵・題の横の物・× の順に置き、本文（class mb）には本文だけを置く。
fn layer_head_order() {
    let modal = dom("src/widgets/modal.rs");
    let layer = squash(fn_of(&modal, "pub fn layer<"));
    for w in [
        "let[boxed,head,main,back_class,x]=CLASSES;",
        "<divclass=head>{back}{h3(f.key)}{f.side}<buttontype=\"button\"class=x",
        "<divclass=main>{f.body}</div>",
    ] {
        assert!(layer.contains(w), "窓の層に {w} が無い");
    }
    assert_eq!(modal.matches("{f.side}").count(), 1, "題の横の物は 1 所");
    assert_eq!(CLASSES[1], "mh");
    assert_eq!(CLASSES[2], "mb");
}

/// 質問の窓の関数 frame は鍵 TITLE_KEY（ask_open）と関数 head の題の横を渡し、残りの数の span は head の中に 1 つだけ。
fn ask_title_side() {
    let ask = dom("src/askwin.rs");
    let frame = fn_of(&ask, "pub fn frame(");
    for w in ["key: TITLE_KEY,", "side: Some(head(w)),"] {
        assert!(frame.contains(w), "askwin.rs の frame に {w} が無い");
    }
    let head = fn_of(&ask, "fn head(w: Walk)");
    assert!(head.contains(NUM_SPAN), "題の横の数が head の中に無い");
    assert!(
        squash(head).contains(&squash(&format!(
            "{NUM_SPAN} <span class=\"qdots\">{{dots}}</span>"
        ))),
        "題の横は数の後に点"
    );
    assert_eq!(ask.matches("<span class=\"num\">").count(), 1, "数の span は 1 つ");
    assert_eq!(TITLE_KEY, "ask_open");
}

/// 窓の題は語の鍵の見出し: 層は題を h3 の語の鍵で描き、題の横の物を見出しの外に置く。7 つの窓の鍵は語彙の label と注釈を持つ。
#[test]
fn bvhelp_win_titles_keyed() {
    let modal = dom("src/widgets/modal.rs");
    let frame = fn_of(&modal, "pub struct Frame {");
    for w in ["pub key: &'static str,", "pub side: Option<AnyView>,"] {
        assert!(frame.contains(w), "modal.rs の Frame に {w} が無い");
    }
    for w in ["<h3>", "f.title", "pub title"] {
        assert!(!modal.contains(w), "modal.rs の wasm の枝に {w} が在る");
    }
    layer_head_order();
    let help = dom("src/widgets/help.rs");
    assert!(
        help.contains("view! { <h3 data-v=key>{label_with_q(key)}</h3> }.into_any()"),
        "help.rs に窓の題の見出しが無い"
    );
    let wins = dom("src/wins.rs");
    assert!(
        wins.contains("            key,\n            side: None,\n"),
        "wins.rs の Frame に鍵が無い"
    );
    ask_title_side();
    assert!(
        !read("src/askwin.rs").contains("pub fn title("),
        "askwin.rs に題の字の関数が残る"
    );
    assert_eq!(frame_of(Win::Ask).1, TITLE_KEY);
    for w in WINS {
        let key = frame_of(w).1;
        assert!(
            vocab().term(key).is_some(),
            "窓 {w:?} の鍵 {key} が語彙に無い"
        );
        assert!(!note(key).is_empty(), "窓 {w:?} の鍵 {key} の注釈が空");
    }
    top(&read("style.css"), ".mh > .num { flex: none;");
}

/// 質問の窓の題の横の残りの数は局面の残り（一覧が読めない局面 Unmeasured は「?」・空は 0）で、見出しの外に出す（憲法 P-7）。
#[test]
fn bvhelp_ask_count_unknown() {
    let ask = dom("src/askwin.rs");
    assert!(
        fn_of(&ask, "fn head(w: Walk)").contains(NUM_SPAN),
        "題の横の数が局面の残りでないか、題の横の関数 head の外"
    );
    assert_eq!(ask.matches(NUM_SPAN).count(), 1, "題の横の数は 1 所");
    let all = ["q.1".to_string()];
    let none = std::collections::BTreeSet::new();
    assert_eq!(count_text(remaining(Phase::Unmeasured("r"), &all, &none)), "?");
    assert_eq!(count_text(remaining(Phase::Empty, &[], &none)), "0");
    assert_eq!(count_text(remaining(Phase::Walk, &all, &none)), "1");
}

/// 質問の窓の空の字は日本語だけ（英語の語を混ぜない）。
#[test]
fn bvhelp_ask_empty_words() {
    assert_eq!(ask::EMPTY, "答えを待つ質問は無い");
    assert!(!ask::EMPTY.chars().any(|c| c.is_ascii_alphabetic()));
}

/// 的の DOM: 関数 seen は selector の片を , で分けた順に見て、片ごとの最初の要素の枠のうち見える最初の枠を返し、
/// 見えるかの関数 found と輪の置き場の関数 place は seen を使う。
fn coach_target_text() {
    let d = dom("src/widgets/coach.rs");
    let seen = squash(fn_of(&d, "fn seen("));
    assert!(
        seen.ends_with("{parts(sel).filter_map(|p|document().query_selector(p).ok().flatten()).map(|el|rect_of(&el)).find(|r|visible(*r))"),
        "的の鎖: {seen}"
    );
    for (head, want) in [
        ("fn found(", "seen(sel).is_some()"),
        ("fn place(", "letel=seen(STEPS.get(step)?.sel)?;"),
    ] {
        assert!(squash(fn_of(&d, head)).contains(want), "{head} に {want} が無い");
    }
}

/// 案内の 5 段の字・selector の片・見える的・置き直しの段。
#[test]
fn bvhelp_coach_steps_and_again() {
    let texts: Vec<&str> = STEPS.iter().map(|s| s.text).collect();
    assert_eq!(
        texts,
        [
            "まずここ。次の一手を押すと進む。",
            "帯の印を押すと、その窓が開く。",
            "段の tile を押すと、その段の札が下に開く。",
            "札や一覧の行を押すと、吹き出しが開く。",
            "⚙ の記号の見方に、画面の見方がある。",
        ]
    );
    let card = STEPS[3].sel;
    assert_eq!(
        parts(card).collect::<Vec<_>>(),
        [".board .kcard", ".ptlist .kcard"]
    );
    assert_eq!(parts(" a ,, b ").collect::<Vec<_>>(), ["a", "b"]);
    let r = |w: f64, h: f64| Rect {
        left: 10.0,
        top: 10.0,
        width: w,
        height: h,
    };
    assert!(visible(r(20.0, 8.0)));
    assert!(!visible(r(0.0, 8.0)));
    assert!(!visible(r(20.0, 0.0)));
    coach_target_text();
    // 幅 600 以下: tile が見え、板の札は隠れる。広い幅: tile が隠れる。
    let narrow = |s: &str| s != "#bar .gearb";
    let wide = |s: &str| s != ".ptile";
    assert_eq!(again(2, narrow), Some(2));
    assert_eq!(again(2, wide), Some(3));
    assert_eq!(again(4, narrow), Some(0));
    assert_eq!(again(1, |s: &str| s == "#bar .pill"), Some(0));
    assert_eq!(again(0, |_: &str| false), None);
}

/// 輪は窓に固定した座標: 要素の枠は get_bounding_client_rect の 4 つの値のまま（scroll を足さない）で輪の置き場の関数 ring に渡し、
/// 輪の style はその 4 つを left・top・width・height の順に置く。
fn ring_text() {
    let d = dom("src/widgets/coach.rs");
    assert!(
        squash(fn_of(&d, "fn rect_of(")).contains(
            "letr=el.get_bounding_client_rect();Rect{left:r.left(),top:r.top(),width:r.width(),height:r.height()"
        ),
        "要素の枠に scroll などを足す"
    );
    assert!(squash(fn_of(&d, "fn place(")).contains("ring:ring(el)"));
    let view = squash(fn_of(&d, "fn coach_view("));
    for w in [
        "format!(\"left:{}px;top:{}px;width:{}px;height:{}px\",s.ring.left,s.ring.top,s.ring.width,s.ring.height",
        "<divclass=\"coach-ring\"style=ring_at></div>",
    ] {
        assert!(view.contains(w), "coach_view に {w} が無い");
    }
}

/// 案内は帯の下に固定した字の帯で、頁の上の余白をその高さだけ広げる（札と行に重ならない）・輪も窓に固定。
/// 規則はどれも media などの塊の外に 1 つずつ在り、同じ詳しさの規則は後が勝つので、基礎の規則は幅の規則より前に置く。
#[test]
fn bvhelp_coach_strip_css() {
    let css = read("style.css");
    let rules = [
        ":root { --coach-h: 44px; }",
        ".coach { position: fixed; top: 52px; left: 0; right: 0; z-index: 25; height: var(--coach-h);",
        ".coach-ring { position: fixed;",
        "body:has(#bar):has(.coach) { padding-top: calc(52px + var(--coach-h)); }",
        "@media (min-width: 601px) { body:has(.coach) .one { height: calc(100vh - 52px - var(--coach-h)); } }",
        "@media (max-width: 760px) { .coach { top: 96px; } body:has(#bar):has(.coach) { padding-top: calc(96px + var(--coach-h)); } }",
        "@media (min-width: 601px) and (max-width: 760px) { body:has(.coach) .one { height: calc(100vh - 96px - var(--coach-h)); } }",
        "@media (max-width: 600px) { :root { --coach-h: 64px; } }",
    ];
    let at = rules.map(|r| top(&css, r));
    for (base, wide) in [(0, 7), (1, 5), (3, 5), (4, 6)] {
        assert!(
            at[base] < at[wide],
            "{} が {} より後",
            rules[base],
            rules[wide]
        );
    }
    assert_eq!(css.matches("--coach-h:").count(), 2, "字の帯の高さの規則");
    assert_eq!(css.matches(".coach {").count(), 2, "字の帯の上の規則");
    for w in [
        ".coach::before",
        ".coach { width: 220px; }",
        ".coach-ring { position: absolute;",
    ] {
        assert!(!css.contains(w), "stylesheet に {w} が在る");
    }
    ring_text();
}

/// 案内の DOM: 幅の替わりと scroll（捕らえの段）と段を出した直後に今の段から置き直す。
/// scroll の受け手は捕らえの段（3 つ目の引数 true）で頁の一生の間持ち、段を出した直後の置き直しは段が替わった時だけ、
/// 幅の替わりの受け手は関数 follow が返し、層の片付け（on_cleanup）の中でだけ外す。
#[test]
fn bvhelp_coach_follow_text() {
    let d = dom("src/widgets/coach.rs");
    assert!(
        squash(fn_of(&d, "fn replace(")).contains(
            "ifletSome(s)=shown.get_untracked(){shown.set(again(s.step,found).and_then(place));}"
        ),
        "置き直しが今の段からでない"
    );
    let follow = squash(fn_of(&d, "fn follow("));
    for w in [
        "letscrolled=Closure::<dynFnMut(web_sys::Event)>::new(move|_:web_sys::Event|replace(shown));",
        "Effect::new(move|prev:Option<Option<usize>>|{letstep=shown.with(|s|s.map(|s|s.step));ifstep.is_some()&&prev!=Some(step){request_animation_frame(move||replace(shown));}step});",
    ] {
        assert!(follow.contains(w), "follow に {w} が無い");
    }
    let call = "add_event_listener_with_callback_and_bool(";
    let at = follow.find(call).expect("scroll の受け手");
    let args = &follow[at + call.len()..];
    let args = &args[..args.find(");").expect("受け手の引数の閉じ")];
    let args: Vec<&str> = args.trim_end_matches(',').split(',').collect();
    assert_eq!(args, ["\"scroll\"", "scrolled.as_ref().unchecked_ref()", "true"]);
    assert!(follow[at..].contains("scrolled.forget();"), "scroll の受け手を持たない");
    assert!(
        follow.ends_with("window_event_listener(ev::resize,move|_|replace(shown))"),
        "幅の替わりの受け手を返さない"
    );
    let layer = squash(fn_of(&d, "pub fn CoachLayer("));
    let at = layer.find("on_cleanup(move||{").expect("層の片付け");
    let cleanup = &layer[at..at + layer[at..].find("});").expect("片付けの閉じ")];
    assert!(cleanup.contains("resize.remove();") && cleanup.contains("esc.remove();"));
    assert!(layer[..at].contains("letresize=follow(shown);"));
    assert_eq!(d.matches("resize.remove()").count(), 1, "幅の受け手を外す所");
    assert!(d.contains("<div class=\"ct\">{text}</div>"));
    for w in ["scroll_x", "scroll_y", "box_at", "BOX_W", "style=box_at"] {
        assert!(
            !read("src/widgets/coach.rs").contains(w),
            "coach.rs に {w} が在る"
        );
    }
}

/// 出所の帯の記号は画面の見方の 6 つの語の注釈に無く、記号の見方の注釈は形と色の 3 行を 1 行ずつ持つ（要件 FR5）。
fn band_only_in_status() {
    for k in HOW {
        assert!(
            !note(k).contains("{band:"),
            "{k} の注釈に出所の帯の記号が在る"
        );
    }
    let status = note("status");
    for line in [
        "\n{band:constitution} 四角 = file に書かれた行\n",
        "\n{band:beads} 丸 = 台帳と器の動くもの\n",
        "\n・ 色 = 出所の帯ごとに 1 色\n",
    ] {
        assert!(status.contains(line), "記号の見方の注釈に {line} が無い");
    }
}

/// 記号の見方の窓の画面の見方: 6 つの部品の語と注釈（図 one と lay）・地図と tab の説明は無く、出所の帯の記号は画面の見方の
/// 語に無い。記号の見方の注釈は形と色の 3 行を持つ（要件 FR5・判断の記録 ADR-27 決定 (1)(ア)）。
/// 画面の見方の段は記号の見方の窓の中身（関数 inner）の末に在り、その規則は media などの塊の外に在る。
#[test]
fn bvhelp_how_notes() {
    assert_eq!(HOW_KEY, "how");
    assert_eq!(
        HOW,
        [
            "how_one", "how_bar", "how_win", "how_pop", "how_lay", "how_tile"
        ]
    );
    let labels: Vec<String> = HOW.iter().map(|k| label(k)).collect();
    assert_eq!(
        labels,
        [
            "1 枚の画面",
            "上の帯",
            "窓",
            "吹き出し",
            "幅の 3 段",
            "段の tile"
        ]
    );
    assert_eq!(label(HOW_KEY), "画面の見方");
    for k in HOW.iter().chain([&HOW_KEY]) {
        assert!(!note(k).is_empty(), "{k} の注釈が空");
    }
    assert!(note("how_one").contains("{fig:one}"));
    assert!(note("how_lay").contains("{fig:lay}"));
    for k in HOW.iter().chain(["status", "ask_open"].iter()) {
        let n = note(k);
        for w in ["地図", "tab", "6 つ"] {
            assert!(!n.contains(w), "{k} の注釈に {w} が在る: {n}");
        }
    }
    band_only_in_status();
    let legend = read("src/project/legend.rs");
    assert!(
        squash(fn_of(&legend, "pub fn inner()")).ends_with(&squash(
            "<div class=\"lghow\">{hs(HOW_KEY)}{HOW.map(hs).into_iter().collect_view()}</div> } .into_any()"
        )),
        "記号の見方の窓の末に画面の見方の段が無い"
    );
    top(&read("style.css"), ".lghow { display: flex;");
}

/// 図 one（帯の印 → 窓・札か行 → 吹き出し）と図 lay（幅の 3 段）の字。
#[test]
fn bvhelp_figs_one_and_lay() {
    let one = fig::svg("one").expect("図 one");
    for w in [
        format!(">{}</text>", label("how_win")),
        format!(">{}</text>", label("how_pop")),
        ">帯の印を押す</text>".to_string(),
        ">札か行を押す</text>".to_string(),
        ">個別の頁へ</text>".to_string(),
    ] {
        assert!(one.contains(&w), "図 one に {w} が無い");
    }
    let lay = fig::svg("lay").expect("図 lay");
    for w in [
        ">1200 以上 横並び</text>",
        ">601〜1199 縦積み</text>",
        ">600 以下 tile</text>",
        ">← 広い · 窓の幅 · 狭い →</text>",
    ] {
        assert!(lay.contains(w), "図 lay に {w} が無い");
    }
    let src = read("src/widgets/fig.rs");
    for w in [
        "tb(124, 8, 80, \"how_win\", \"ok\"),",
        "tb(124, 56, 80, \"how_pop\", \"ok\"),",
    ] {
        assert_eq!(src.matches(w).count(), 1, "図 one の箱 {w}");
    }
}
