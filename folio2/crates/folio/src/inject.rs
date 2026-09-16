//! `folio inject`（便 2・docs/design/delivery-2.md §1）。憲法の前文と規範文を CLAUDE.md の生成区間へ書く・検査する。
//! day-1 の script `scripts/inject_check.py` の写し。--write・--check・--print は同じ導出関数 `derive` を使う。
//! 終了コードは便 0 と同じ 3 値（R-2 の超過は script の 3 ではなく 違反 = 1）。

use std::fs;
use std::path::Path;

use crate::verdict::Verdict;
use crate::yaml::{self, Node};

pub const BEGIN: &str = "<!-- constitution:begin -->";
pub const END: &str = "<!-- constitution:end -->";

/// 規範文の判定は正本の strength 欄で行う（文末の語で判定しない）。
const STRENGTHS: [&str; 3] = ["must", "must-not", "should"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Write,
    Check,
    Print,
}

/// 1 回の実行の結果。`stdout` は --print の本文、`messages` は標準エラーへ出す所見。
#[derive(Debug)]
pub struct Outcome {
    pub verdict: Verdict,
    pub stdout: Option<String>,
    pub messages: Vec<String>,
}

impl Outcome {
    fn new(verdict: Verdict, msg: impl Into<String>) -> Self {
        Outcome {
            verdict,
            stdout: None,
            messages: vec![msg.into()],
        }
    }
}

/// 前後の空白を落とし、連続する空白を半角空白 1 つに畳む。
fn squash(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// 正本 → 生成区間の本文（決定的）。返り値 = (本文, 行の数)。規範でない strength・「。」で終わらない文・0 本は Err。
pub fn derive(constitution: &Node) -> Result<(String, usize), String> {
    let mut lines = Vec::new();
    let articles = match constitution.get("articles") {
        None => &[][..],
        Some(a) => a.as_seq().ok_or("articles が一覧でない")?,
    };
    for art in articles {
        let statements = match art.get("statements") {
            None => &[][..],
            Some(s) => s.as_seq().ok_or("statements が一覧でない")?,
        };
        for s in statements {
            let id = s
                .get("id")
                .and_then(Node::as_str)
                .ok_or("規範文に id が無い")?;
            let strength = s.get("strength").and_then(Node::as_str);
            if !strength.is_some_and(|v| STRENGTHS.contains(&v)) {
                return Err(format!("{id}: strength が規範の値でない: {strength:?}"));
            }
            let text = s
                .get("text")
                .and_then(Node::as_str)
                .ok_or_else(|| format!("{id}: text が無い"))?;
            let text = squash(text);
            if !text.ends_with('。') {
                return Err(format!("{id}: 規範文が「。」で終わらない"));
            }
            lines.push(format!("{id}: {text}"));
        }
    }
    if lines.is_empty() {
        return Err("規範文が 0 本（母集団が空 = 正本が壊れている）".to_string());
    }
    let pre = constitution
        .get("precedence")
        .and_then(|p| p.get("text"))
        .and_then(Node::as_str);
    if let Some(pre) = pre.filter(|p| !p.is_empty()) {
        lines.insert(0, format!("順位: {}", squash(pre)));
    }
    Ok((lines.join("\n\n"), lines.len()))
}

/// 区間（begin の直後から end の直前まで）。begin と end がそれぞれ 1 本ずつで begin が先のときだけ定まる。
pub fn region_of(md: &str) -> Option<(usize, usize)> {
    let b: Vec<_> = md.match_indices(BEGIN).map(|(i, _)| i).collect();
    let e: Vec<_> = md.match_indices(END).map(|(i, _)| i).collect();
    match (b.as_slice(), e.as_slice()) {
        ([b], [e]) if b <= e => Some((b + BEGIN.len(), *e)),
        _ => None,
    }
}

/// rules の thresholds の R-2 の value 欄から「byte」の直前の数（桁区切りの「,」を除く）を読む。
pub fn r2_limit(rules: &Node) -> Option<usize> {
    let row = rules
        .get("thresholds")?
        .as_seq()?
        .iter()
        .find(|row| row.get("id").and_then(Node::as_str) == Some("R-2"))?;
    let value = row.get("value").and_then(Node::as_str)?;
    value.match_indices("byte").find_map(|(at, _)| {
        let head = value[..at].trim_end();
        let start = head
            .trim_end_matches(|c: char| c.is_ascii_digit() || c == ',')
            .len();
        let n: String = head[start..].chars().filter(|c| *c != ',').collect();
        n.parse().ok()
    })
}

/// 区間の外の行のうち、前後の空白を落として「する。」か「ない。」で終わる行（`<!--` で始まる行は除く）。
fn normative_outside(md: &str, region: (usize, usize)) -> Vec<String> {
    let outside = format!("{}{}", &md[..region.0], &md[region.1..]);
    outside
        .split(['\n', '\r'])
        .map(str::trim)
        .filter(|l| (l.ends_with("する。") || l.ends_with("ない。")) && !l.starts_with("<!--"))
        .map(str::to_string)
        .collect()
}

fn load_yaml(path: &Path) -> Result<Node, String> {
    let text =
        fs::read_to_string(path).map_err(|e| format!("{}: 読めない: {e}", path.display()))?;
    yaml::parse(&text)
        .map(|doc| doc.root)
        .map_err(|e| format!("{}: parse できない: {e}", path.display()))
}

pub fn run(dir: &Path, claude_md: &Path, mode: Mode) -> Outcome {
    let constitution = match load_yaml(&dir.join("constitution.yaml")) {
        Ok(n) => n,
        Err(e) => return Outcome::new(Verdict::Unknown, e),
    };
    let (body, n) = match derive(&constitution) {
        Ok(d) => d,
        Err(e) => return Outcome::new(Verdict::Unknown, format!("正本を導出できない: {e}")),
    };
    let size = body.len();
    let limit = match load_yaml(&dir.join("rules.yaml")).map(|r| r2_limit(&r)) {
        Ok(Some(l)) => l,
        Ok(None) => {
            return Outcome::new(
                Verdict::Unknown,
                "rules の R-2 が読めない（上限検査を skip しない）",
            );
        }
        Err(e) => return Outcome::new(Verdict::Unknown, e),
    };
    if size > limit {
        return Outcome::new(
            Verdict::Fail,
            format!("生成区間 {size} byte が R-2 上限 {limit} byte を超える"),
        );
    }
    if mode == Mode::Print {
        return Outcome {
            verdict: Verdict::Pass,
            stdout: Some(format!("{body}\n")),
            messages: vec![format!("{n} 行 / {size} byte / R-2 上限 {limit}")],
        };
    }
    let md = match fs::read_to_string(claude_md) {
        Ok(md) => md,
        Err(e) => {
            return Outcome::new(
                Verdict::Unknown,
                format!("{}: 読めない: {e}", claude_md.display()),
            );
        }
    };
    let Some(region) = region_of(&md) else {
        return Outcome::new(
            Verdict::Unknown,
            "marker が 1 対でない（0 本・2 本以上・逆順）",
        );
    };
    let want = format!("\n{body}\n");
    if mode == Mode::Write {
        let new = format!("{}{want}{}", &md[..region.0], &md[region.1..]);
        if new != md {
            if let Err(e) = fs::write(claude_md, new) {
                return Outcome::new(
                    Verdict::Unknown,
                    format!("{}: 書けない: {e}", claude_md.display()),
                );
            }
            return Outcome::new(Verdict::Pass, format!("wrote {n} 行 / {size} byte"));
        }
        return Outcome::new(Verdict::Pass, format!("差が無い（{n} 行 / {size} byte）"));
    }
    let outside = normative_outside(&md, region);
    if let Some(first) = outside.first() {
        return Outcome::new(
            Verdict::Fail,
            format!("区間の外に規範語で終わる行が {} 行: {first}", outside.len()),
        );
    }
    let cur = &md[region.0..region.1];
    if cur.trim().is_empty() {
        return Outcome::new(Verdict::Unknown, "区間が空（未注入）");
    }
    if cur != want {
        return Outcome::new(
            Verdict::Fail,
            format!("区間 {} byte ≠ 導出 {} byte", cur.len(), want.len()),
        );
    }
    Outcome::new(Verdict::Pass, format!("{n} 行 / {size} byte 一致"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inject_region_needs_one_pair_in_order() {
        assert_eq!(
            region_of("a<!-- constitution:begin -->x<!-- constitution:end -->"),
            Some((28, 29))
        );
        assert_eq!(
            region_of("<!-- constitution:end --><!-- constitution:begin -->"),
            None
        );
        assert_eq!(region_of("<!-- constitution:begin -->"), None);
    }

    #[test]
    fn inject_r2_limit_reads_number_before_byte() {
        let rules =
            yaml::parse("thresholds:\n  - {id: R-2, value: \"8,000 byte 以下\"}\n").unwrap();
        assert_eq!(r2_limit(&rules.root), Some(8000));
        let rules = yaml::parse("thresholds:\n  - {id: R-2, value: 上限なし}\n").unwrap();
        assert_eq!(r2_limit(&rules.root), None);
    }

    #[test]
    fn inject_derive_squashes_and_rejects_unknown_strength() {
        let c = yaml::parse(
            "precedence: {text: \" 甲  乙 \"}\narticles:\n  - statements:\n      - {id: X-1.1, strength: must, text: \"a  b する。\"}\n",
        )
        .unwrap();
        assert_eq!(
            derive(&c.root).unwrap(),
            ("順位: 甲 乙\n\nX-1.1: a b する。".to_string(), 2)
        );
        let c = yaml::parse(
            "articles:\n  - statements:\n      - {id: X-1.1, strength: maybe, text: する。}\n",
        )
        .unwrap();
        assert!(derive(&c.root).is_err());
    }
}
