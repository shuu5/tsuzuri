//! 行 g-top-fit の歯: 上端の帯は幅 1000 以下で 2 段の狭い形・1001 以上で 1 行の形。
//! 横 scroll そのものは browser でしか測れないので、幅を名指して、その幅で効く塊
//! （max-width の値がその幅以上）の規則の字を見る。
#![cfg(test)]

const BAND: [&str; 7] = [
    ".top { gap: var(--s2); padding: 0 var(--s3); height: auto; min-height: 56px; flex-wrap: wrap; padding-top: var(--s1); padding-bottom: var(--s1); }",
    ".nav { order: 5; width: 100%; margin: 0; justify-content: space-between; }",
    ".nav a { padding: 0 var(--s2); flex: 1; justify-content: center; }",
    ".nav a .lbl { display: none; }",
    ".brand .name { display: none; }",
    ".seatpill .who { display: none; }",
    ".seatpill { padding: 0 var(--s2); }",
];

fn read(rel: &str) -> String {
    let path = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn strip_comments(css: &str) -> String {
    let mut out = String::new();
    let mut rest = css;
    while let Some(i) = rest.find("/*") {
        out.push_str(&rest[..i]);
        rest = match rest[i + 2..].find("*/") {
            Some(j) => &rest[i + 2 + j + 2..],
            None => "",
        };
    }
    out.push_str(rest);
    out
}

/// `@media (max-width: Npx)` の塊を (N, 中身) で集める。中身は括弧の深さを数えて切り出す。
fn max_width_blocks(css: &str) -> Vec<(u32, String)> {
    const HEAD: &str = "@media (max-width: ";
    let css = strip_comments(css);
    let mut blocks = Vec::new();
    let mut rest = css.as_str();
    while let Some(i) = rest.find(HEAD) {
        rest = &rest[i + HEAD.len()..];
        let Some(px) = rest.find("px)") else { break };
        let Ok(n) = rest[..px].trim().parse::<u32>() else { continue };
        let Some(open) = rest.find('{') else { break };
        let body = &rest[open + 1..];
        let mut depth = 1usize;
        let mut end = body.len();
        for (k, c) in body.char_indices() {
            match c {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = k;
                        break;
                    }
                }
                _ => {}
            }
        }
        blocks.push((n, body[..end].to_string()));
        rest = &body[end..];
    }
    blocks
}

fn effective_at(blocks: &[(u32, String)], width: u32) -> impl Iterator<Item = &str> {
    blocks
        .iter()
        .filter(move |(n, _)| *n >= width)
        .map(|(_, b)| b.as_str())
}

#[test]
fn topfit_wraps_at_named_widths() {
    let blocks = max_width_blocks(&read("style.css"));
    for width in [390, 760, 768, 780, 820, 853, 900, 960, 1000] {
        for rule in BAND {
            assert!(
                effective_at(&blocks, width).any(|b| b.contains(rule)),
                "幅 {width}px で効く塊に帯の狭い形の規則が無い: {rule}"
            );
        }
    }
}

#[test]
fn topfit_one_row_above_break() {
    let blocks = max_width_blocks(&read("style.css"));
    for width in [1001, 1024, 1280] {
        assert!(
            !effective_at(&blocks, width).any(|b| b.contains(".top {")),
            "幅 {width}px で効く塊に .top の規則が在る（帯が 1 行の形でなくなる）"
        );
    }
}

#[test]
fn topfit_narrow_form_from_mock() {
    let blocks = max_width_blocks(&read("../../docs/design/mock3/ui.css"));
    for rule in BAND {
        assert!(
            blocks
                .iter()
                .any(|(n, b)| *n == 760 && b.contains(rule)),
            "見本の 760 の塊に帯の狭い形の規則が無い: {rule}"
        );
    }
}
