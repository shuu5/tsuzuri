//! 手順と流れの図（見本の ui.js の FIGS・figFs・figSVG の写し）: 注釈の行 `{fig:名}` を 3〜5 の箱と矢印の SVG の字にする。
//! 見本と違い、svg の class は fig だけ・無印の箱の g は fb だけにする（style.css に規則の在る class だけを使う）。

use crate::vocab::label;

/// 図の名（見本の FIGS の鍵の順）。
pub const NAMES: [&str; 7] = ["back", "move", "next", "reserve", "promo", "tick", "hover"];

/// 図の幅と箱の高さ。
const WIDTH: i32 = 320;
const BOX_H: i32 = 30;

/// 箱と線の字（直の字か、見出しの語と同じ字は語彙の鍵から引く）。
#[derive(Debug, Clone, Copy)]
enum Txt {
    Lit(&'static str),
    Term(&'static str),
}

impl Txt {
    fn text(self) -> String {
        match self {
            Txt::Lit(s) => s.to_string(),
            Txt::Term(k) => label(k),
        }
    }
}

/// 箱（x・y・幅・字・class・無印は空）。
struct FigBox {
    x: i32,
    y: i32,
    w: i32,
    t: Txt,
    class: &'static str,
}

/// 線（元の箱・先の箱・字・点線か）。
struct Edge {
    from: usize,
    to: usize,
    t: Txt,
    dash: bool,
}

struct Fig {
    h: i32,
    boxes: &'static [FigBox],
    edges: &'static [Edge],
    cap: &'static str,
}

const fn b(x: i32, y: i32, w: i32, t: &'static str, class: &'static str) -> FigBox {
    FigBox {
        x,
        y,
        w,
        t: Txt::Lit(t),
        class,
    }
}

const fn e(from: usize, to: usize) -> Edge {
    Edge {
        from,
        to,
        t: Txt::Lit(""),
        dash: false,
    }
}

const fn dash(from: usize, to: usize) -> Edge {
    Edge {
        dash: true,
        ..e(from, to)
    }
}

const BACK: Fig = Fig {
    h: 104,
    boxes: &[
        b(4, 38, 70, "戻る を押す", ""),
        b(110, 6, 92, "窓がある → 前面へ", "ok"),
        b(110, 70, 92, "無い → 新しく開く", ""),
        b(236, 38, 80, "この窓を閉じる", "end"),
    ],
    edges: &[e(0, 1), e(0, 2), e(1, 3), e(2, 3)],
    cap: "",
};

const MOVE: Fig = Fig {
    h: 92,
    boxes: &[
        b(4, 8, 92, "① 移動の承認", "ok"),
        b(116, 8, 92, "② /exit 待ち", ""),
        b(228, 8, 88, "③ 起こし直し", "ok"),
        b(116, 60, 92, "戻らない → 保留", "ng"),
    ],
    edges: &[e(0, 1), e(1, 2), dash(1, 3)],
    cap: "",
};

const NEXT: Fig = Fig {
    h: 58,
    boxes: &[
        b(2, 14, 56, "a 限度", ""),
        b(66, 14, 56, "b 応答なし", ""),
        b(130, 14, 56, "c run", ""),
        b(194, 14, 64, "d〜f 質問", ""),
        b(266, 14, 52, "g なし", "off"),
    ],
    edges: &[e(0, 1), e(1, 2), e(2, 3), e(3, 4)],
    cap: "← 左ほど優先 · 先頭の 1 つが大きく出る",
};

const RESERVE: Fig = Fig {
    h: 100,
    boxes: &[
        b(2, 8, 68, "門で絞る", ""),
        b(80, 8, 70, "鍵で並べる", ""),
        b(160, 8, 76, "Tier1 が取る", "ok"),
        b(244, 8, 76, "Tier2 は残り", "ok"),
        b(160, 62, 160, "残りなし → 移り先なし", "ng"),
    ],
    edges: &[e(0, 1), e(1, 2), e(2, 3), dash(3, 4)],
    cap: "",
};

const PROMO: Fig = Fig {
    h: 64,
    boxes: &[
        b(4, 18, 80, "memo", ""),
        b(120, 4, 92, "task", "ok"),
        b(120, 36, 92, "ADR・rule 等", "ok"),
        b(240, 18, 76, "閉じる", "off"),
    ],
    edges: &[
        Edge {
            t: Txt::Term("nx_promote"),
            ..e(0, 1)
        },
        e(0, 2),
        dash(1, 3),
    ],
    cap: "",
};

const TICK: Fig = Fig {
    h: 50,
    boxes: &[
        b(2, 10, 84, "最後の tick", ""),
        b(100, 10, 106, "≤ 2×周期 healthy", "ok"),
        b(220, 10, 98, "> 2×周期 stale", "ng"),
    ],
    edges: &[e(0, 1), e(1, 2)],
    cap: "",
};

const HOVER: Fig = Fig {
    h: 56,
    boxes: &[
        b(4, 12, 90, "← 根拠 2 段", "up"),
        FigBox {
            t: Txt::Term("nb_self"),
            ..b(114, 12, 90, "", "self")
        },
        b(224, 12, 92, "影響 2 段 →", "down"),
    ],
    edges: &[e(1, 0), e(1, 2)],
    cap: "",
};

fn fig(name: &str) -> Option<&'static Fig> {
    Some(match name {
        "back" => &BACK,
        "move" => &MOVE,
        "next" => &NEXT,
        "reserve" => &RESERVE,
        "promo" => &PROMO,
        "tick" => &TICK,
        "hover" => &HOVER,
        _ => return None,
    })
}

/// 箱の字の大きさ（見本の figFs: ASCII の印字できる字は 0.62・ほかは 1.02 の幅として 8〜10.5 に収める）。
pub fn font_size(s: &str, width: u32) -> String {
    let units: f64 = s
        .chars()
        .map(|c| if (' '..='~').contains(&c) { 0.62 } else { 1.02 })
        .sum();
    format!("{:.1}", ((f64::from(width) - 8.0) / units).clamp(8.0, 10.5))
}

/// 線の端（見本の figSVG の 3 つの場合: 横に並ぶ・右下か右上へ・縦に並ぶ）。
fn ends(a: &FigBox, b: &FigBox) -> (i32, i32, i32, i32) {
    if (a.y - b.y).abs() < 4 {
        let (x1, x2) = if a.x < b.x {
            (a.x + a.w, b.x)
        } else {
            (a.x, b.x + b.w)
        };
        (x1, a.y + BOX_H / 2, x2, a.y + BOX_H / 2)
    } else if a.x + a.w <= b.x {
        (a.x + a.w, a.y + BOX_H / 2, b.x, b.y + BOX_H / 2)
    } else {
        let x = a.x.max(b.x) + 20;
        if a.y < b.y {
            (x, a.y + BOX_H, x, b.y)
        } else {
            (x, a.y, x, b.y + BOX_H)
        }
    }
}

fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

/// 図の SVG の字（名が NAMES に無ければ None）。
pub fn svg(name: &str) -> Option<String> {
    let f = fig(name)?;
    let h = f.h;
    let mut s = format!(
        "<svg class=\"fig\" viewBox=\"0 0 {WIDTH} {h}\" width=\"{WIDTH}\" height=\"{h}\" role=\"img\" aria-label=\"図: {}\">",
        esc(name)
    );
    s.push_str(&format!(
        "<defs><marker id=\"fa-{name}\" viewBox=\"0 0 8 8\" refX=\"7\" refY=\"4\" markerWidth=\"7\" markerHeight=\"7\" orient=\"auto\"><path d=\"M0 0L8 4L0 8z\" fill=\"var(--ink-3)\"/></marker></defs>"
    ));
    for l in f.edges {
        let (Some(from), Some(to)) = (f.boxes.get(l.from), f.boxes.get(l.to)) else {
            continue;
        };
        let (x1, y1, x2, y2) = ends(from, to);
        let dash = if l.dash {
            " stroke-dasharray=\"3 3\""
        } else {
            ""
        };
        s.push_str(&format!(
            "<path d=\"M{x1} {y1} L{x2} {y2}\" stroke=\"var(--ink-3)\" stroke-width=\"1.4\" fill=\"none\"{dash} marker-end=\"url(#fa-{name})\"/>"
        ));
        let t = l.t.text();
        if !t.is_empty() {
            s.push_str(&format!(
                "<text x=\"{}\" y=\"{}\" font-size=\"9\" text-anchor=\"middle\" fill=\"var(--ink-3)\">{}</text>",
                (x1 + x2) / 2,
                y1.min(y2) - 3,
                esc(&t)
            ));
        }
    }
    for bx in f.boxes {
        let class = if bx.class.is_empty() {
            "fb".to_string()
        } else {
            format!("fb fb-{}", bx.class)
        };
        let t = bx.t.text();
        s.push_str(&format!(
            "<g class=\"{class}\"><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{BOX_H}\" rx=\"5\"/><text x=\"{}\" y=\"{}\" font-size=\"{}\" text-anchor=\"middle\">{}</text></g>",
            bx.x,
            bx.y,
            bx.w,
            bx.x + bx.w / 2,
            bx.y + 19,
            font_size(&t, bx.w.unsigned_abs()),
            esc(&t)
        ));
    }
    if !f.cap.is_empty() {
        s.push_str(&format!(
            "<text x=\"160\" y=\"{}\" font-size=\"9.5\" text-anchor=\"middle\" fill=\"var(--ink-3)\">{}</text>",
            h - 4,
            esc(f.cap)
        ));
    }
    s.push_str("</svg>");
    Some(s)
}
