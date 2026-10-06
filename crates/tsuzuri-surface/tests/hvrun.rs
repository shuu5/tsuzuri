//! 受入の runner の hover の歯（接頭辞 hvrun_）。
//! runner の hover の手順が規則の行 R-20 の値を、本物の面の関数（hover の place と keeps と定数）と同じ式で測る歯である。
#![cfg(test)]

use std::fs;
use std::path::Path;

use tsuzuri_boundary::audit::{
    Grace, HOVER_DX, HOVER_DY, HOVER_GRACE_MS, HoverCard, HoverTarget, MODES, Page, RULES, SCREENS,
    WIDTHS, case, count, exit_path, grace, spot, sweep,
};
use tsuzuri_boundary::stage::cdp::{CARD_EXPRESSION, Command, MEASURE_EXPRESSION, Step, moved};
use tsuzuri_surface::widgets::hover::{
    GRACE_MS, OFFSET_X, OFFSET_Y, Point, Rect, Size, keeps, place,
};

/// 撃った URL の board（clean.json の読み先と同じ origin）。
const BOARD: &str = "http://127.0.0.1:4801/";

/// 偽の頁の card の大きさと窓の大きさ。
const CARD: Size = Size {
    width: 200.0,
    height: 90.0,
};
const WINDOW: Size = Size {
    width: 1280.0,
    height: 800.0,
};

fn read(rel: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn fixture(name: &str) -> String {
    read(&format!("tests/fixtures/surface/accept-12/{name}"))
}

/// 札の在る事実の字（clean.json の hover_at の空の列を札 1 つの列に替える）。
fn with_target() -> String {
    let clean = fixture("clean.json");
    let empty = r#""hover_at": []"#;
    assert_eq!(clean.matches(empty).count(), 1, "clean.json の hover_at");
    clean.replace(
        empty,
        r#""hover_at": [{"name": "a.node", "x": 100, "y": 200, "bottom": 210}]"#,
    )
}

fn target() -> HoverTarget {
    HoverTarget {
        name: "a.node".to_string(),
        x: 100,
        y: 200,
        bottom: 210,
    }
}

fn pt(x: f64, y: f64) -> Point {
    Point { x, y }
}

/// 偽の頁の崩し方（None は面と同じ振る舞い）。
#[derive(Clone, Copy, PartialEq, Eq)]
enum Fault {
    None,
    NoCard,
    Shifted,
    DropOnExit,
    StayAfterGrace,
}

/// 偽の頁: 札（x 60 から 140・y 190 から 210）の中へ指を動かすと面の place の左上に card を出し、札を出た後は
/// 面の keeps で残すか消し、猶予（GRACE_MS）の過ぎた card の読みは null を返す。時計は Wait だけが進める。
struct Hovering {
    log: Vec<String>,
    fault: Fault,
    clock: u64,
    shown: Option<Point>,
    leaving: Option<(Point, u64)>,
    text: String,
}

impl Hovering {
    fn new(fault: Fault) -> Hovering {
        Hovering {
            log: Vec::new(),
            fault,
            clock: 0,
            shown: None,
            leaving: None,
            text: with_target(),
        }
    }

    /// 記録の中の頭が `head` の項の数。
    fn times(&self, head: &str) -> usize {
        self.log.iter().filter(|l| l.starts_with(head)).count()
    }
}

impl Page for Hovering {
    fn run(&mut self, command: &Command) -> Result<Vec<String>, String> {
        match command {
            Command::Wait { ms } => self.clock += ms,
            Command::Navigate { .. } => (self.shown, self.leaving) = (None, None),
            _ => {}
        }
        self.log.push(format!("{command:?}"));
        Ok(Vec::new())
    }

    fn measure(&mut self) -> Result<String, String> {
        self.log.push("measure".to_string());
        Ok(self.text.clone())
    }

    fn url(&mut self) -> Result<String, String> {
        self.log.push("url".to_string());
        Ok(String::new())
    }

    fn events(&self) -> &[String] {
        &[]
    }

    fn point(&mut self, x: u32, y: u32) -> Result<(), String> {
        self.log.push(format!("Point {x} {y}"));
        let now = pt(f64::from(x), f64::from(y));
        if (60..=140).contains(&x) && (190..=210).contains(&y) {
            self.leaving = None;
            if self.fault != Fault::NoCard {
                let at = place(now, CARD, WINDOW);
                let shift = if self.fault == Fault::Shifted { 2.0 } else { 0.0 };
                self.shown = Some(pt(at.x + shift, at.y));
            }
            return Ok(());
        }
        let Some(at) = self.shown else {
            return Ok(());
        };
        let rect = Rect {
            left: at.x,
            top: at.y,
            width: CARD.width,
            height: CARD.height,
        };
        if rect.distance(now) == 0.0 {
            self.leaving = None;
            return Ok(());
        }
        match self.leaving {
            None if self.fault == Fault::DropOnExit => self.shown = None,
            None => self.leaving = Some((now, self.clock)),
            Some((from, since)) => {
                if !keeps(from, now, rect, (self.clock - since) as f64) {
                    (self.shown, self.leaving) = (None, None);
                }
            }
        }
        Ok(())
    }

    fn card(&mut self) -> Result<String, String> {
        self.log.push("card".to_string());
        if let Some((_, since)) = self.leaving
            && self.fault != Fault::StayAfterGrace
            && self.clock - since > u64::from(GRACE_MS)
        {
            (self.shown, self.leaving) = (None, None);
        }
        Ok(match self.shown {
            Some(at) => format!(
                r#"{{"left":{},"top":{},"width":{},"height":{},"vw":{},"vh":{}}}"#,
                at.x, at.y, CARD.width, CARD.height, WINDOW.width, WINDOW.height
            ),
            None => "null".to_string(),
        })
    }
}

/// hover の条の違反の数だけが `n` でほかは 0 の数の列。
fn hover_only(n: usize) -> [usize; 12] {
    let at = RULES.iter().position(|(k, _)| *k == "hover").expect("条");
    let mut want = [0; 12];
    want[at] = n;
    want
}

fn card_at(left: f64, top: f64) -> HoverCard {
    HoverCard {
        left,
        top,
        width: 200.0,
        height: 90.0,
        vw: 1280.0,
        vh: 800.0,
    }
}

#[test]
fn hvrun_values_match() {
    assert_eq!(HOVER_DX, f64::from(OFFSET_X));
    assert_eq!(HOVER_DY, f64::from(OFFSET_Y));
    assert_eq!(HOVER_GRACE_MS, u64::from(GRACE_MS));
    let card = Size {
        width: 200.0,
        height: 100.0,
    };
    let window = Size {
        width: 1280.0,
        height: 800.0,
    };
    let mut points: Vec<(f64, f64, Size, Size)> = [
        (100.0, 200.0),
        (1064.0, 200.0),
        (1065.0, 200.0),
        (1200.0, 200.0),
        (100.0, 10.0),
        (100.0, 20.0),
        (100.0, 790.0),
        (100.0, 820.0),
        (100.0, 720.0),
        (1270.0, 795.0),
    ]
    .iter()
    .map(|&(x, y)| (x, y, card, window))
    .collect();
    let tall = Size {
        width: 250.0,
        height: 900.0,
    };
    let narrow = Size {
        width: 300.0,
        height: 800.0,
    };
    points.push((150.0, 400.0, tall, narrow));
    assert_eq!(points.len(), 11);
    for (x, y, card, window) in points {
        let want = place(pt(x, y), card, window);
        let got = spot(
            (x, y),
            (card.width, card.height),
            (window.width, window.height),
        );
        assert_eq!(got, (want.x, want.y), "pointer ({x}, {y})");
    }
}

#[test]
fn hvrun_move_step() {
    assert_eq!(
        moved(10, 20),
        Step::Call {
            method: "Input.dispatchMouseEvent",
            params: r#"{"type":"mouseMoved","x":10,"y":20}"#.to_string(),
        }
    );
    assert!(CARD_EXPRESSION.contains(".hcard.on"), "{CARD_EXPRESSION}");
    assert!(
        MEASURE_EXPRESSION.contains("const hover_at = seen."),
        "測りの式の hover_at の鎖"
    );
    let start = MEASURE_EXPRESSION
        .find("return JSON.stringify({")
        .expect("返す object");
    let body = &MEASURE_EXPRESSION[start..];
    let body = &body[..body.find("});").expect("object の閉じ")];
    let last = body.lines().map(str::trim).rfind(|l| !l.is_empty());
    assert_eq!(last, Some("hover_at,"));
}

#[test]
fn hvrun_probe_pass() {
    let url = format!("{BOARD}?mode=beginner");
    let mut page = Hovering::new(Fault::None);
    let facts = case(&mut page, &url, 1280).expect("case");
    assert!(facts.hovered.is_empty(), "{:?}", facts.hovered);
    assert_eq!(facts.probes, ["測った hover a.node"]);
    let at = page.log.iter().position(|l| l == "measure").expect("測り");
    let tail = &page.log[at + 1..];
    let (last, head) = tail.split_last().expect("probe の記録");
    let (wait, head) = head.split_last().expect("最後の Wait");
    let want = [
        "Point 100 200",
        "Wait { ms: 100 }",
        "card",
        "Point 100 212",
        "Point 108 212",
        "card",
    ];
    assert_eq!(head, want);
    assert_eq!(last, "card");
    let ms: u64 = wait
        .strip_prefix("Wait { ms: ")
        .and_then(|w| w.strip_suffix(" }"))
        .and_then(|w| w.parse().ok())
        .unwrap_or_else(|| panic!("{wait}"));
    assert!((290..=300).contains(&ms), "{wait}");
}

#[test]
fn hvrun_probe_faults() {
    let vocab = fixture("vocab.json");
    let url = format!("{BOARD}?mode=beginner");
    for (fault, word) in [
        (Fault::NoCard, "show"),
        (Fault::Shifted, "place"),
        (Fault::DropOnExit, "grace"),
        (Fault::StayAfterGrace, "gone"),
    ] {
        let mut page = Hovering::new(fault);
        let facts = case(&mut page, &url, 1280).expect("case");
        assert_eq!(facts.hovered.len(), 1, "{word}: {:?}", facts.hovered);
        assert!(
            facts.hovered[0].starts_with(&format!("{word} a.node")),
            "{word}: {:?}",
            facts.hovered
        );
        assert_eq!(count(&facts, &vocab), hover_only(1), "{word}");
    }
}

#[test]
fn hvrun_grace_verdict() {
    let first = card_at(116.0, 180.0);
    let same = card_at(116.0, 180.0);
    assert_eq!(grace(&first, Some(&same), 0), Grace::Kept);
    assert_eq!(grace(&first, Some(&same), 149), Grace::Kept);
    assert_eq!(grace(&first, None, 0), Grace::Lost);
    assert_eq!(grace(&first, Some(&card_at(118.0, 180.0)), 0), Grace::Lost);
    assert_eq!(grace(&first, Some(&card_at(116.0, 182.0)), 0), Grace::Lost);
    assert_eq!(grace(&first, Some(&same), 150), Grace::Late);
    assert_eq!(grace(&first, None, 150), Grace::Late);
}

#[test]
fn hvrun_exit_path() {
    let target = target();
    assert_eq!(
        exit_path(&target, &card_at(116.0, 180.0)),
        Some(((100, 212), (108, 212)))
    );
    let left = HoverCard {
        width: 60.0,
        ..card_at(20.0, 180.0)
    };
    assert_eq!(exit_path(&target, &left), Some(((100, 212), (92, 212))));
    let around = HoverCard {
        width: 100.0,
        height: 100.0,
        ..card_at(50.0, 190.0)
    };
    assert_eq!(exit_path(&target, &around), None);
    let short = HoverCard {
        vh: 212.0,
        ..card_at(116.0, 180.0)
    };
    assert_eq!(exit_path(&target, &short), None);
}

#[test]
fn hvrun_narrow_skip() {
    let url = format!("{BOARD}?mode=beginner");
    let mut page = Hovering::new(Fault::None);
    let facts = case(&mut page, &url, 390).expect("case");
    assert_eq!(page.times("Point"), 0, "{:?}", page.log);
    assert!(facts.probes.is_empty() && facts.hovered.is_empty());
}

#[test]
fn hvrun_sweep_report() {
    let mut page = Hovering::new(Fault::None);
    let (report, total, unknown) = sweep(&mut page, BOARD, &fixture("vocab.json")).expect("sweep");
    assert_eq!((total, unknown), (0, 0), "{report}");
    let wide = WIDTHS.iter().filter(|w| **w > 390).count();
    let screens = wide * MODES.len() * (SCREENS.len() + 1);
    let measured = report.lines().filter(|l| l.contains("測った hover")).count();
    assert_eq!(measured, screens, "{report}");
    assert!(report.contains("\n違反 計 0\n"), "{report}");
}
