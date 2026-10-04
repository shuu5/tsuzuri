//! 席の手元の写し（行 t-seatcopy・判断の記録 ADR-38 決定 (3)(4)・条 P-13・P-2）。憲法の正本から、要の写し（順位・段
//! 「絶対にやらない」と段「確認してから」の規範文の全文・段「いつも守る」の条の id と題・全文の写しの在りか）と、全文の写し
//! （順位と全部の規範文の 1 行 1 文）の 2 つの file の字を導く。`derive.rs` が導出物の置き場の下の dir `seat` に書き・比べる
//! （置き場の直下に置かないのは、器の契約表の宣言が直下の .md と .toml を契約表と読むため）。
//! 形は中身を削らない最短の形: 頭の 1 行・順位の行・段の名だけの行・「id 字」の行・「条の id 題」の行・在りかの行を改行 1 つで
//! 区切り、区切りに「: 」を使わず、字は空白を 1 つに畳むだけで逐語。規則の表に欄 key が seat-bytes と seat-role-bytes の行が
//! 2 本とも無い置き場は写しを導かない。強さが規範の値でない文・「。」で終わらない文・文 0 本は Err（まだ分からない・退いた
//! folio inject の導き方を引き継ぐ）。

use std::fs;
use std::path::{Path, PathBuf};

use crate::constitution_enums::{Strength, Tier};
use crate::note;
use crate::plan;
use crate::rules::{self, SEAT_BYTES, SEAT_ROLE_BYTES};
use crate::yaml::{self, Node};

/// 導出物の置き場の下の写しの dir。
pub(crate) const DIR: &str = "seat";

/// 要の写しの file の名。
pub(crate) const BRIEF: &str = "brief.txt";

/// 全文の写しの file の名。
pub(crate) const FULL: &str = "constitution.txt";

/// 頭の 1 行の頭の字（続けて憲法の正本の版管理の根からの path と空白 1 つと版の字）。
const HEAD: &str = "生成物・手で直さない・";

/// 順位の行の頭の語（続けて空白 1 つと順位の字）。
const PRECEDENCE: &str = "順位";

/// 在りかの行の頭の語（続けて空白 1 つと全文の写しの版管理の根からの path）。
const LOCATION: &str = "全文";

/// 要の写しの段の順と段の名の行（条 P-13.1 の字）。
const TIERS: [(Tier, &str); 3] = [
    (Tier::Never, "絶対にやらない"),
    (Tier::AskFirst, "確認してから"),
    (Tier::Always, "いつも守る"),
];

/// 憲法の正本の file の名（正本の置き場の直下）。
const CONSTITUTION: &str = "constitution.yaml";

/// 導いた 2 つの写しの字と、要の写しの file 全体の byte の上限。
pub(crate) struct Copies {
    pub(crate) brief: String,
    pub(crate) full: String,
    /// 欄 key が seat-bytes の行の値から seat-role-bytes の行の値を引いた数（規則の行 R-1 − R-41）。
    pub(crate) cap: usize,
}

/// 条 1 つ（段・題と、規範文の「id 字」の行）。
struct Article<'a> {
    id: &'a str,
    tier: Tier,
    title: &'a str,
    lines: Vec<String>,
}

/// 2 つの写しを導く。欄 key の行が 2 本とも無い（規則の表が無いを含む）置き場は None。`out_dir` は導出物の置き場（無くてもよい）。
pub(crate) fn derive(dir: &Path, out_dir: &Path) -> Result<Option<Copies>, String> {
    let Some(rules) = plan::load_rules(dir)? else {
        return Ok(None);
    };
    let Some(cap) = cap(&rules)? else {
        return Ok(None);
    };
    let path = dir.join(CONSTITUTION);
    let text = fs::read_to_string(&path).map_err(|e| format!("{CONSTITUTION}: 読めない: {e}"))?;
    let c = yaml::parse(&text)
        .map_err(|e| format!("{CONSTITUTION}: parse できない: {e}"))?
        .root;
    let version = c
        .get("meta")
        .and_then(|m| m.get("version"))
        .and_then(Node::as_str)
        .ok_or_else(|| format!("{CONSTITUTION}: meta.version が字でない"))?;
    let pre = c
        .get("precedence")
        .and_then(|p| p.get("text"))
        .and_then(Node::as_str)
        .map(squash)
        .filter(|p| !p.is_empty())
        .ok_or_else(|| format!("{CONSTITUTION}: precedence.text が無いか空"))?;
    let root = place_root(dir)?;
    let head = format!("{HEAD}{} {version}", rel(&root, &path)?);
    let full_at = under(rel(&root, out_dir)?, &format!("{DIR}/{FULL}"));
    let arts = articles(&c)?;
    let pre = format!("{PRECEDENCE} {pre}");
    Ok(Some(Copies {
        brief: brief(&[&head, &pre], &arts, &full_at),
        full: full(&[&head, &pre], &arts),
        cap,
    }))
}

/// 要の写しの上限（欄 key の 2 本の値の差）。2 本とも無ければ None。片方だけ・差が 0 以下は Err。
fn cap(rules: &Node) -> Result<Option<usize>, String> {
    match (rules::bytes(rules, SEAT_BYTES)?, rules::bytes(rules, SEAT_ROLE_BYTES)?) {
        (None, None) => Ok(None),
        (Some(total), Some(role)) => total
            .checked_sub(role)
            .filter(|n| *n > 0)
            .map(Some)
            .ok_or_else(|| format!("欄 key が {SEAT_ROLE_BYTES} の値 {role} byte が {SEAT_BYTES} の値 {total} byte 以上（要の写しの上限が 0 以下）")),
        (Some(_), None) => Err(format!("欄 key が {SEAT_ROLE_BYTES} の閾値の行が無い（{SEAT_BYTES} の行は在る）")),
        (None, Some(_)) => Err(format!("欄 key が {SEAT_BYTES} の閾値の行が無い（{SEAT_ROLE_BYTES} の行は在る）")),
    }
}

/// 憲法の条の全部（正本の順）。規範文が 1 本も無ければ Err。
fn articles(c: &Node) -> Result<Vec<Article<'_>>, String> {
    let list = c
        .get("articles")
        .and_then(Node::as_seq)
        .ok_or("articles が一覧でない")?;
    let mut out = Vec::with_capacity(list.len());
    for a in list {
        let id = a.get("id").and_then(Node::as_str).ok_or("条に id が無い")?;
        let tier = a
            .get("tier")
            .and_then(Node::as_str)
            .and_then(Tier::from_name)
            .ok_or_else(|| format!("{id}: tier が段の値でない"))?;
        let title = a
            .get("title")
            .and_then(Node::as_str)
            .ok_or_else(|| format!("{id}: title が無い"))?;
        let statements = match a.get("statements") {
            None => &[][..],
            Some(s) => s
                .as_seq()
                .ok_or_else(|| format!("{id}: statements が一覧でない"))?,
        };
        let lines = statements
            .iter()
            .map(statement)
            .collect::<Result<Vec<_>, _>>()?;
        out.push(Article {
            id,
            tier,
            title,
            lines,
        });
    }
    if out.iter().all(|a| a.lines.is_empty()) {
        return Err("規範文が 0 本（母集団が空 = 正本が壊れている）".to_string());
    }
    Ok(out)
}

/// 規範文 1 つの「id 字」の行。強さが規範の値でない・「。」で終わらないは Err（判定は正本の strength 欄で行い文末の語で判じない）。
fn statement(s: &Node) -> Result<String, String> {
    let id = s
        .get("id")
        .and_then(Node::as_str)
        .ok_or("規範文に id が無い")?;
    let strength = s.get("strength").and_then(Node::as_str);
    if !strength.is_some_and(|v| Strength::from_name(v).is_some()) {
        return Err(format!("{id}: strength が規範の値でない: {strength:?}"));
    }
    let text = s
        .get("text")
        .and_then(Node::as_str)
        .map(squash)
        .ok_or_else(|| format!("{id}: text が無い"))?;
    if !text.ends_with('。') {
        return Err(format!("{id}: 規範文が「。」で終わらない"));
    }
    Ok(format!("{id} {text}"))
}

/// 要の写し: 頭の行と順位の行・段ごとに段の名の行と、段「いつも守る」は条の「id 題」・ほかの段は規範文の行・在りかの行。
fn brief(top: &[&str], arts: &[Article], full_at: &str) -> String {
    let mut lines: Vec<String> = top.iter().map(|s| (*s).to_string()).collect();
    for (tier, name) in TIERS {
        lines.push(name.to_string());
        for a in arts.iter().filter(|a| a.tier == tier) {
            if tier == Tier::Always {
                lines.push(format!("{} {}", a.id, squash(a.title)));
            } else {
                lines.extend(a.lines.iter().cloned());
            }
        }
    }
    lines.push(format!("{LOCATION} {full_at}"));
    lines.join("\n") + "\n"
}

/// 全文の写し: 頭の行と順位の行と、全部の規範文の行（正本の順）。
fn full(top: &[&str], arts: &[Article]) -> String {
    let mut lines: Vec<String> = top.iter().map(|s| (*s).to_string()).collect();
    lines.extend(arts.iter().flat_map(|a| a.lines.iter().cloned()));
    lines.join("\n") + "\n"
}

/// 前後の空白を落とし、続く空白を半角空白 1 つに畳む。
fn squash(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// 置き場を含む版管理の根（`note::root_of` を実の path で解く）。
fn place_root(dir: &Path) -> Result<PathBuf, String> {
    let here = fs::canonicalize(dir).map_err(|e| format!("{}: 解けない: {e}", dir.display()))?;
    note::root_of(&here)
}

/// 根からの path の字の下の字（根そのものなら下の字だけ）。
fn under(base: String, tail: &str) -> String {
    if base.is_empty() {
        tail.to_string()
    } else {
        format!("{base}/{tail}")
    }
}

/// 根からの path の字（`/` で区切る・根そのものは空の字）。path が無ければ親を実の path で解いて名を足す。根の外は Err。
fn rel(root: &Path, path: &Path) -> Result<String, String> {
    let real = match fs::canonicalize(path) {
        Ok(p) => p,
        Err(_) => {
            let parent = path
                .parent()
                .ok_or_else(|| format!("{}: 親 dir が無い", path.display()))?;
            let name = path
                .file_name()
                .ok_or_else(|| format!("{}: 名が無い", path.display()))?;
            fs::canonicalize(parent)
                .map_err(|e| format!("{}: 解けない: {e}", parent.display()))?
                .join(name)
        }
    };
    let tail = real.strip_prefix(root).map_err(|_| {
        format!(
            "{} が版管理の根 {} の下に無い",
            real.display(),
            root.display()
        )
    })?;
    Ok(tail
        .iter()
        .map(|c| c.to_string_lossy())
        .collect::<Vec<_>>()
        .join("/"))
}
