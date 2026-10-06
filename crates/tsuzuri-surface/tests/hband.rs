//! 帯の host の負荷と書きの印と host の窓の歯（接頭辞 hband_）。
//! 電文 HostDoc を契約の型で組んで字にし、印の字と class・窓の段の行・読めない欄と口・窓の名と幅と語・帯と窓の配線の字を測る。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::host::{
    DeviceWrite, Gauge, HostDoc, Load, Memory, Pressure, ScopeMemory, Stall, Wear,
};
use tsuzuri_contract::wire;
use tsuzuri_surface::frame::Mode;
use tsuzuri_surface::hostwin::{
    BAD_BODY, HOST_KEY, Line, MARK_CLASSES, NO_DEVICES, NO_WEAR, PART_KEYS, Part, SCOPE_NAME,
    UNREAD, WIDTH, content, fixed2, gib, mark, mbps, parts, ratio, tb, write_total,
};
use tsuzuri_surface::project::{Body, NOT_READ};
use tsuzuri_surface::topbar::{TODOS, Win, win_href, win_of_href};
use tsuzuri_surface::view::Fetched;
use tsuzuri_surface::vocab::vocab;
use tsuzuri_surface::wins::frame_of;

const GIB: u64 = 1 << 30;

/// 摩耗の記録の時刻（日本時間 2026-10-05 18:00）と、1 時間後と 30 時間後の今。
const READ_AT: EpochSecs = 1_791_190_800;
const SOON: EpochSecs = READ_AT + 3_600;
const LATER: EpochSecs = READ_AT + 30 * 3_600;

fn read(rel: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn stall(avg10: u32, avg60: u32) -> Reading<Stall> {
    Reading::Known(Stall { avg10, avg60 })
}

fn device(name: &str, rate: Reading<u64>, wear: Option<Reading<Wear>>) -> DeviceWrite {
    DeviceWrite {
        name: name.to_string(),
        rate,
        wear,
    }
}

fn wear() -> Wear {
    Wear {
        read_at: READ_AT,
        used_pct: 6,
        written: 512_345_000_000_000,
        spare_pct: 100,
        media_errors: 0,
    }
}

/// 全部の欄が読めた電文（装置は 3 つ・摩耗は読めた・読めない・path なしの 3 通り・over は渡した物）。
fn full(over: Vec<Gauge>) -> HostDoc {
    HostDoc {
        at: READ_AT,
        load: Reading::Known(Load {
            one: 1344,
            five: 1210,
            fifteen: 980,
            cores: 32,
        }),
        pressure: Pressure {
            cpu_some: stall(1520, 1310),
            memory_full: stall(12, 7),
            io_full: stall(120, 80),
        },
        memory: Reading::Known(Memory {
            total: 64 * GIB,
            available: 22 * GIB + GIB / 2,
            swap_total: 8 * GIB,
            swap_free: 8 * GIB,
        }),
        scopes: Reading::Known(vec![
            ScopeMemory {
                name: "seat.scope".to_string(),
                current: 26 * GIB,
                max: 32 * GIB,
            },
            ScopeMemory {
                name: "run.scope".to_string(),
                current: 3 * GIB,
                max: 16 * GIB,
            },
        ]),
        devices: vec![
            device(
                "nvme0",
                Reading::Known(72_300_000),
                Some(Reading::Known(wear())),
            ),
            device("nvme1", Reading::Known(1_500_000), Some(Reading::Unknown)),
            device("sda", Reading::Known(0), None),
        ],
        over,
    }
}

/// 全部の欄が読めない電文（装置は 0）。
fn blank() -> HostDoc {
    HostDoc {
        at: READ_AT,
        load: Reading::Unknown,
        pressure: Pressure {
            cpu_some: Reading::Unknown,
            memory_full: Reading::Unknown,
            io_full: Reading::Unknown,
        },
        memory: Reading::Unknown,
        scopes: Reading::Unknown,
        devices: Vec::new(),
        over: Vec::new(),
    }
}

fn body(d: &HostDoc) -> Fetched {
    Fetched::Body(wire::encode(d).expect("電文"))
}

fn line(name: &str, value: &str, warn: bool) -> Line {
    Line {
        name: name.to_string(),
        value: value.to_string(),
        warn,
    }
}

fn find<'a>(ps: &'a [Part], key: &str) -> &'a Part {
    ps.iter()
        .find(|p| p.key == key)
        .unwrap_or_else(|| panic!("段 {key} が無い"))
}

/// 数の字: 100 倍は小数 2 桁、core 比は 1 分の負荷を core の数で切り捨てて割り（0 は 1 で割る）、MB/s は 10 未満だけ
/// 小数 1 桁、GiB と TB は小数 1 桁（どれも切り捨て）。
#[test]
fn hband_number_texts() {
    for (h, want) in [
        (42, "0.42"),
        (1344, "13.44"),
        (5, "0.05"),
        (100, "1.00"),
        (0, "0.00"),
    ] {
        assert_eq!(fixed2(h), want, "{h}");
    }
    let load = |one, cores| Load {
        one,
        five: 0,
        fifteen: 0,
        cores,
    };
    assert_eq!(ratio(&load(1344, 32)), 42);
    assert_eq!(ratio(&load(3300, 32)), 103);
    assert_eq!(ratio(&load(3200, 32)), 100);
    assert_eq!(ratio(&load(250, 0)), 250);
    for (b, want) in [
        (0, "0.0"),
        (3_456_789, "3.4"),
        (9_999_999, "9.9"),
        (10_000_000, "10"),
        (73_800_000, "73"),
    ] {
        assert_eq!(mbps(b), want, "{b}");
    }
    assert_eq!(gib(32 * GIB), "32.0");
    assert_eq!(gib(26 * GIB + GIB / 2), "26.5");
    assert_eq!(gib(GIB - 1), "0.9");
    assert_eq!(tb(512_345_000_000_000), "512.3");
    assert_eq!(tb(99_999_999_999), "0.0");
}

/// 印: 字は「負荷 <core 比> · 書き <装置の速さの和> MB/s」で、class は電文の over が空なら hostb・空でなければ
/// hostb w。和は装置が 0 か読めない速さが 1 つでも在れば「?」、負荷が読めなければ「?」。口が読めない・まだ読んでいない・
/// 電文でない本文は hostb u の「負荷 ? · 書き ?」。
#[test]
fn hband_mark_text_and_class() {
    let m = mark(&body(&full(Vec::new())));
    assert_eq!(
        (m.class, m.text.as_str()),
        ("hostb", "負荷 0.42 · 書き 73 MB/s")
    );
    let m = mark(&body(&full(vec![Gauge::Io])));
    assert_eq!(
        (m.class, m.text.as_str()),
        ("hostb w", "負荷 0.42 · 書き 73 MB/s")
    );
    assert_eq!(MARK_CLASSES, ["hostb", "hostb w", "hostb u"]);

    let mut d = full(Vec::new());
    d.devices[1].rate = Reading::Unknown;
    assert_eq!(write_total(&d.devices), None);
    assert_eq!(mark(&body(&d)).text, "負荷 0.42 · 書き ?");
    d.devices.clear();
    assert_eq!(write_total(&d.devices), None);
    assert_eq!(mark(&body(&d)).text, "負荷 0.42 · 書き ?");
    let mut d = full(Vec::new());
    d.devices = vec![device("nvme0", Reading::Known(3_456_789), None)];
    assert_eq!(write_total(&d.devices), Some(3_456_789));
    assert_eq!(mark(&body(&d)).text, "負荷 0.42 · 書き 3.4 MB/s");
    d.load = Reading::Unknown;
    let m = mark(&body(&d));
    assert_eq!(
        (m.class, m.text.as_str()),
        ("hostb", "負荷 ? · 書き 3.4 MB/s")
    );

    for f in [
        Fetched::NotRead,
        Fetched::Failed,
        Fetched::Body("{}".to_string()),
        Fetched::Body("not json".to_string()),
    ] {
        let m = mark(&f);
        assert_eq!(
            (m.class, m.text.as_str()),
            ("hostb u", "負荷 ? · 書き ?"),
            "{f:?}"
        );
    }
}

/// 窓の段は負荷・詰まり・メモリ・書きの速さ・摩耗の順で、行は電文の値の字（scope は電文の順・装置は表の順・
/// 摩耗は記録を持つ装置だけで読めない記録は「?」・記録の時刻は 20 時間を越えると月日を前に付ける）。
#[test]
fn hband_parts_lines() {
    let ps = parts(&full(Vec::new()), SOON);
    assert_eq!(ps.iter().map(|p| p.key).collect::<Vec<_>>(), PART_KEYS);
    assert_eq!(
        PART_KEYS,
        ["hw_load", "hw_stall", "hw_memory", "hw_write", "hw_wear"]
    );
    assert!(ps.iter().all(|p| p.none.is_none()));
    assert_eq!(
        find(&ps, "hw_load").lines,
        [
            line("core 比", "0.42", false),
            line("1 · 5 · 15 分", "13.44 · 12.10 · 9.80", false),
            line("core", "32", false),
        ]
    );
    assert_eq!(
        find(&ps, "hw_stall").lines,
        [
            line("cpu some", "avg10 15.20% · avg60 13.10%", false),
            line("memory full", "avg10 0.12% · avg60 0.07%", false),
            line("io full", "avg10 1.20% · avg60 0.80%", false),
        ]
    );
    assert_eq!(
        find(&ps, "hw_memory").lines,
        [
            line("使っている", "41.5 / 64.0 GiB", false),
            line("swap", "0.0 / 8.0 GiB", false),
            line("seat.scope", "26.0 / 32.0 GiB · 81%", false),
            line("run.scope", "3.0 / 16.0 GiB · 18%", false),
        ]
    );
    assert_eq!(
        find(&ps, "hw_write").lines,
        [
            line("nvme0", "72 MB/s", false),
            line("nvme1", "1.5 MB/s", false),
            line("sda", "0.0 MB/s", false),
        ]
    );
    assert_eq!(
        find(&ps, "hw_wear").lines,
        [
            line(
                "nvme0",
                "使った 6% · 書いた 512.3 TB · 予備 100% · 誤り 0 · 記録 18:00 JST",
                false
            ),
            line("nvme1", "?", false),
        ]
    );
    let later = parts(&full(Vec::new()), LATER);
    assert_eq!(
        find(&later, "hw_wear").lines[0].value,
        "使った 6% · 書いた 512.3 TB · 予備 100% · 誤り 0 · 記録 10-05 18:00 JST"
    );
}

/// 注意の色の行は電文の over の写し: Load は core 比の行・Memory は memory full の行・Io は io full の行だけで、
/// 値が線を越えていても over に無ければ色を付けず、値が小さくても over に在れば付ける（面は値と線を比べない）。
#[test]
fn hband_warn_copies_over() {
    let warned = |d: &HostDoc| -> Vec<String> {
        parts(d, SOON)
            .iter()
            .flat_map(|p| p.lines.iter().filter(|l| l.warn).map(|l| l.name.clone()))
            .collect()
    };
    assert!(warned(&full(Vec::new())).is_empty());
    assert_eq!(warned(&full(vec![Gauge::Load])), ["core 比"]);
    assert_eq!(warned(&full(vec![Gauge::Memory])), ["memory full"]);
    assert_eq!(warned(&full(vec![Gauge::Io])), ["io full"]);
    assert_eq!(
        warned(&full(Gauge::ALL.to_vec())),
        ["core 比", "memory full", "io full"]
    );
    let mut hot = full(Vec::new());
    hot.load = Reading::Known(Load {
        one: 9_000,
        five: 0,
        fifteen: 0,
        cores: 1,
    });
    hot.pressure.io_full = stall(9_000, 9_000);
    hot.pressure.memory_full = stall(9_000, 9_000);
    assert!(warned(&hot).is_empty(), "over の無い電文に色");
}

/// 口が読めなければ全体が測れていない（まだ読んでいない・読めない・電文でないの 3 つの理由）。電文の欄が読めなければ
/// その行だけ「?」・scope の列が読めなければ scope の 1 行が「?」・装置が 0 なら書きと摩耗の段は行の替わりの 1 行。
#[test]
fn hband_unread_and_unknown_fields() {
    for (f, want) in [
        (Fetched::NotRead, NOT_READ),
        (Fetched::Failed, UNREAD),
        (Fetched::Body("{}".to_string()), BAD_BODY),
        (Fetched::Body(String::new()), BAD_BODY),
    ] {
        assert_eq!(content(&f, SOON), Body::Unmeasured(want), "{f:?}");
    }
    let Body::Filled(ps) = content(&body(&blank()), SOON) else {
        panic!("読めた電文が中身でない");
    };
    for key in ["hw_load", "hw_stall"] {
        let p = find(&ps, key);
        assert_eq!(p.lines.len(), 3, "{key}");
        assert!(p.lines.iter().all(|l| l.value == "?"), "{key}");
    }
    assert_eq!(
        find(&ps, "hw_memory").lines,
        [
            line("使っている", "?", false),
            line("swap", "?", false),
            line(SCOPE_NAME, "?", false),
        ]
    );
    for (key, none) in [("hw_write", NO_DEVICES), ("hw_wear", NO_WEAR)] {
        let p = find(&ps, key);
        assert!(p.lines.is_empty(), "{key}");
        assert_eq!(p.none, Some(none), "{key}");
    }
    let mut no_wear = full(Vec::new());
    for d in &mut no_wear.devices {
        d.wear = None;
    }
    let ps = parts(&no_wear, SOON);
    assert_eq!(find(&ps, "hw_wear").none, Some(NO_WEAR));
    assert_eq!(find(&ps, "hw_write").none, None);
    let mut empty_scopes = full(Vec::new());
    empty_scopes.scopes = Reading::Known(Vec::new());
    assert_eq!(parts(&empty_scopes, SOON)[2].lines.len(), 2);
}

/// 窓の名は 9 つ目に host を足し、URL の query に読み戻り、幅は 600 で、題の語は語彙の「host の負荷」と空でない注釈で、
/// 段の見出しの語は全部が語彙に在り、帯のやる事の数には入らない。
#[test]
fn hband_window_named_host() {
    assert_eq!(Win::ALL.len(), 9);
    assert_eq!(Win::ALL.last(), Some(&Win::Host));
    assert_eq!(Win::Host.key(), "host");
    for m in Mode::ALL {
        assert_eq!(
            win_of_href(&win_href(Win::Host, m)),
            Some((Win::Host, None))
        );
    }
    assert_eq!((WIDTH, HOST_KEY), (600, "host_load"));
    assert_eq!(frame_of(Win::Host), (600, HOST_KEY));
    let term = vocab().term(HOST_KEY).expect("語彙の host_load");
    assert_eq!(term.label, "host の負荷");
    assert!(!term.note.is_empty(), "注釈が空");
    for (key, want) in
        PART_KEYS
            .into_iter()
            .zip(["負荷", "詰まり", "メモリ", "書きの速さ", "SSD の摩耗"])
    {
        assert_eq!(
            vocab().term(key).map(|t| t.label.as_str()),
            Some(want),
            "{key}"
        );
    }
    assert!(
        TODOS.iter().all(|(_, _, w)| *w != Win::Host),
        "やる事の数に host が在る"
    );
}

/// 関数 `name` の本体（頭の行から、同じ字下げの閉じの行まで）。
fn fn_body<'a>(src: &'a str, head: &str) -> &'a str {
    let at = src.find(head).unwrap_or_else(|| panic!("{head} が無い"));
    let indent = src[..at].rsplit('\n').next().map_or(0, str::len);
    let close = format!("\n{}}}\n", " ".repeat(indent));
    let end = src[at..]
        .find(&close)
        .map_or(src.len(), |e| at + e + close.len());
    &src[at..end]
}

/// 帯と窓の DOM（wasm の枝・host で組めないので字で測る）: 帯は host の口を読み、host の印は席と口座の印の直後で抜けの
/// 検査の印の前に 1 つだけ在り、押すと host の窓を開く。窓は host の窓の名で hostwin の本文を描き、本文は host の口を読む。
#[test]
fn hband_band_and_window_wiring() {
    let bar = read("src/topbar.rs");
    let dom = &bar[bar.find("mod dom {").expect("wasm の枝")..];
    let view = fn_body(dom, "pub fn view(bar: Bar)");
    assert!(view.contains("let host_doc = crate::net::read(hostwin::PATH);"));
    assert!(
        view.contains(
            "let host_part = move || host_doc.with(|h| host_view(hostwin::mark(h), wins));"
        )
    );
    let seat = view.find("{seat_part}").expect("席と口座の印");
    let host = view.find("{host_part}").expect("host の印");
    let gp = view.find("{gp}").expect("抜けの検査の印");
    assert!(
        seat < host && host < gp,
        "host の印は席と口座と抜けの検査の間"
    );
    assert_eq!(view.matches("{host_part}").count(), 1);
    let host_view = fn_body(dom, "fn host_view(m: HostMark, wins: WinCtx<Win>)");
    assert!(host_view.contains("class=m.class aria-label=label(HOST_KEY) on:click=move |_| wins.open(Win::Host, false)>{m.text}</button>"));
    assert_eq!(
        dom.matches("Win::Host").count(),
        1,
        "帯の wasm の枝の host の窓の口"
    );

    let wins = read("src/wins.rs");
    let draw = fn_body(&wins, "pub fn draw(win: Win, ctx: WinCtx<Win>)");
    assert!(draw.contains("Win::Host => crate::hostwin::body(),"));
    let host = read("src/hostwin.rs");
    let body = fn_body(&host, "pub fn body() -> AnyView");
    assert!(body.contains("crate::net::read(PATH)"));
    assert!(body.contains("content(f, crate::net::now())"));
    assert!(host.contains("pub use tsuzuri_contract::host::PATH;"));
    assert!(read("src/lib.rs").contains("\npub mod hostwin;\n"));
}

/// 印と段の class は stylesheet（基と部品）に在り、印の注意の色は帯の規則の中に在る。
#[test]
fn hband_classes_in_stylesheet() {
    let css = read("style.css") + &tsuzuri_surface::style::joined();
    for rule in [
        "#bar .hostb { height: 28px;",
        "#bar .hostb.w {",
        "#bar .hostb.u {",
        ".hw {",
        ".hw-p h4 {",
        ".hw-l {",
        ".hw-n {",
        ".hw-v {",
        ".hw-l.w {",
    ] {
        assert_eq!(css.matches(rule).count(), 1, "stylesheet の {rule}");
    }
    assert!(css.contains(".num {") && css.contains(".muted {"));
}
