//! 本文の書式（Markdown）の一部を描く部品（判断の記録 ADR-30 決定 (4)・規則の行 R-25）。
//! 台帳の本文と記録が使う形（見出し・箇条・番号つきの箇条・段落・等幅の字）だけを読み、描けない形（表の行・囲いの code）は
//! 字のままの段にする。字は HTML として読まず、DOM は字の node で組む。読み手は純な関数で host でも組み、外の部品を足さない。

/// 見出しの段の class（段の深さ 1〜4 の順）。
pub const HEAD_CLASSES: [&str; 4] = ["mdh1", "mdh2", "mdh3", "mdh4"];

/// 箇条の段・字のままの段・描いた本文の箱の class。
pub const CLASSES: [&str; 3] = ["mdli", "mdpre", "md"];

/// 箇条の印（番号の無い箇条）。
pub const BULLET: &str = "・";

/// 囲いの code の行の頭。
pub const FENCE: &str = "```";

/// 行の中の片。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Inline {
    /// 字のまま（山括弧の字も字のまま）。
    Text(String),
    /// 逆引用符の対の中（等幅の字）。
    Code(String),
}

/// 段。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    /// 見出し（行頭の # の数 1〜4 と、残りの片）。
    Head(u8, Vec<Inline>),
    /// 段落（続く行は段落の中の改行として行ごとに残す）。
    Para(Vec<Vec<Inline>>),
    /// 箇条（深さは字下げ 2 字ごとに 1・番号つきなら番号の字）。
    Item {
        depth: u8,
        ordered: Option<String>,
        body: Vec<Inline>,
    },
    /// 字のままの段（表の行・囲いの code）。
    Pre(String),
}

/// 行の中の逆引用符の対を等幅の片にする（対にならない逆引用符は字のまま）。
pub fn inline(line: &str) -> Vec<Inline> {
    let mut out = Vec::new();
    let mut rest = line;
    while let Some(open) = rest.find('`') {
        let Some(len) = rest[open + 1..].find('`') else {
            break;
        };
        if open > 0 {
            out.push(Inline::Text(rest[..open].to_string()));
        }
        out.push(Inline::Code(rest[open + 1..open + 1 + len].to_string()));
        rest = &rest[open + len + 2..];
    }
    if !rest.is_empty() {
        out.push(Inline::Text(rest.to_string()));
    }
    out
}

/// 見出しの行（行頭の # が 1〜4 個と空白）なら深さと残り。
fn head(line: &str) -> Option<(u8, &str)> {
    let n = line.len() - line.trim_start_matches('#').len();
    let rest = line[n..].strip_prefix(' ')?;
    u8::try_from(n)
        .ok()
        .filter(|n| (1..=4).contains(n))
        .map(|n| (n, rest))
}

/// 箇条の行（字下げと「- 」「* 」か、数字と「. 」「) 」）なら段。
fn item(line: &str) -> Option<Block> {
    let body = line.trim_start_matches(' ');
    let depth = u8::try_from((line.len() - body.len()) / 2).unwrap_or(u8::MAX);
    if let Some(rest) = body.strip_prefix("- ").or_else(|| body.strip_prefix("* ")) {
        return Some(Block::Item {
            depth,
            ordered: None,
            body: inline(rest),
        });
    }
    let digits = body.len() - body.trim_start_matches(|c: char| c.is_ascii_digit()).len();
    let rest = body.get(digits..).filter(|_| digits > 0)?;
    let rest = rest
        .strip_prefix(". ")
        .or_else(|| rest.strip_prefix(") "))?;
    Some(Block::Item {
        depth,
        ordered: Some(body[..=digits].to_string()),
        body: inline(rest),
    })
}

/// 1 行（囲いの code なら閉じる行まで）を段にする（空の行は None）。
fn block<'a>(line: &'a str, lines: &mut impl Iterator<Item = &'a str>) -> Option<Block> {
    if line.trim_start().starts_with(FENCE) {
        let mut kept = vec![line];
        for l in lines.by_ref() {
            kept.push(l);
            if l.trim_start().starts_with(FENCE) {
                break;
            }
        }
        return Some(Block::Pre(kept.join("\n")));
    }
    if line.trim().is_empty() {
        return None;
    }
    if let Some((n, rest)) = head(line) {
        return Some(Block::Head(n, inline(rest)));
    }
    item(line).or_else(|| {
        Some(if line.starts_with('|') {
            Block::Pre(line.to_string())
        } else {
            Block::Para(vec![inline(line)])
        })
    })
}

/// 本文を段の列に読む。空の行で段落を分け、続く段落の行と続く表の行は 1 つの段に足す。
pub fn parse(text: &str) -> Vec<Block> {
    let mut out: Vec<Block> = Vec::new();
    // 前の行が続きを受ける段（段落の行か表の行）か（空の行とほかの段で切れる）。
    let mut cont = false;
    let mut lines = text.lines().map(|l| l.trim_end_matches('\r'));
    while let Some(line) = lines.next() {
        let b = block(line, &mut lines);
        let joined = match (&b, out.last_mut()) {
            (Some(Block::Pre(row)), Some(Block::Pre(at))) if cont && line.starts_with('|') => {
                at.push('\n');
                at.push_str(row);
                true
            }
            (Some(Block::Para(row)), Some(Block::Para(at))) if cont => {
                at.extend(row.iter().cloned());
                true
            }
            _ => false,
        };
        cont = match &b {
            Some(Block::Para(_)) => true,
            Some(Block::Pre(_)) => line.starts_with('|'),
            _ => false,
        };
        if !joined {
            out.extend(b);
        }
    }
    out
}

#[cfg(target_arch = "wasm32")]
pub use dom::view;

#[cfg(target_arch = "wasm32")]
mod dom {
    use leptos::prelude::*;

    use super::{BULLET, Block, CLASSES, HEAD_CLASSES, Inline};

    fn inline_view(xs: Vec<Inline>) -> AnyView {
        xs.into_iter()
            .map(|x| match x {
                Inline::Text(t) => view! { <span>{t}</span> }.into_any(),
                Inline::Code(t) => view! { <code>{t}</code> }.into_any(),
            })
            .collect_view()
            .into_any()
    }

    fn block_view(b: Block) -> AnyView {
        let [li, pre, _] = CLASSES;
        match b {
            Block::Head(n, xs) => {
                let at = usize::from(n).saturating_sub(1);
                let class = HEAD_CLASSES.get(at).copied().unwrap_or_default();
                view! { <div class=class>{inline_view(xs)}</div> }.into_any()
            }
            Block::Para(lines) => {
                let lines = lines
                    .into_iter()
                    .enumerate()
                    .map(|(i, xs)| view! { {(i > 0).then(|| view! { <br/> })}{inline_view(xs)} })
                    .collect_view();
                view! { <p>{lines}</p> }.into_any()
            }
            Block::Item {
                depth,
                ordered,
                body,
            } => {
                let style = format!("margin-left:{}em", f32::from(depth) * 1.2);
                let mark = ordered.unwrap_or_else(|| BULLET.to_string());
                view! { <div class=li style=style><span>{mark}</span><span>{inline_view(body)}</span></div> }
                    .into_any()
            }
            Block::Pre(t) => view! { <pre class=pre>{t}</pre> }.into_any(),
        }
    }

    /// 段の列を描く（字は字の node で出し、HTML として読まない）。
    pub fn view(blocks: Vec<Block>) -> AnyView {
        let items = blocks.into_iter().map(block_view).collect_view();
        view! { <div class=CLASSES[2]>{items}</div> }.into_any()
    }
}
