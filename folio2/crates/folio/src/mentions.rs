//! 規則の表の行 R-17 の床の歯（便 93・docs/design/delivery-93.md §1）。設計文書の中で id を持つ行の散文の欄に現れた、
//! その行以外の id のうち、その行の型付きの欄に無いものを数える（値 0 件）。
//! 機械の読みは 5 つの閉じた一覧で閉じる＝対象の file（9 種）・id を持つ行（節点の索引の 5 種）・散文の欄
//! （型付きの欄 21 語と来歴 6 語と最上位 2 節の補集合）・受け皿の表・数えない言及の語形（10 語）。どれもこの file の定数（P-5.1）。
//! 対は無向で見る（どちらかの行の型付きの欄に相手が在れば満たす）。規範文の id は親の条へ丸めてから照合する。
//! 歯の入り口は rules.yaml の行 R-17 そのもので、行が無ければ数えずに判定の外の 1 行 `OFF` を出させ（便 156・FR5）、
//! 値が 0 件 でなければ「まだ分からない」（P-4.2）。
//! id を拾う口は refs.rs の `scan_ids` と link.rs の `scan_adr_ids`（正規表現は使わない）。

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use crate::adr::Adr;
use crate::link;
use crate::refs;
use crate::verdict::Report;
use crate::yaml::{self, Node};

/// 入り口の行と、その行が在る節と、歯が数える値。
const ROW_ID: &str = "R-17";
const RULE_SECTIONS: [&str; 2] = ["thresholds", "discipline"];
const VALUE: &str = "0 件";

/// 行 R-17 が無くて数えなかったときに folio check が標準エラーへ出す 1 行（便 156・床の判定の外）。
pub const OFF: &str = "# 行 R-17 が規則の表に無い＝散文の言及の歯は数えていない（床の判定の外・値 0 件 の行 R-17 を足して撃ち直すと数える）";

/// 対象の file（行 R-17 の母集団が名指す 9 つ・支度表は入れない）。判断の記録と設計ノートは dir の中の記録。
pub const TARGETS: [&str; 9] = [
    "constitution.yaml",
    "rules.yaml",
    "srs.yaml",
    "vocabulary.yaml",
    "ceiling.yaml",
    "index.yaml",
    "intake.yaml",
    "adr/",
    "design-note/",
];

/// 型付きの欄（21 語・便 90 の adrs・便 91 の refs・便 92 の produced を含む）。この欄の中の id は型付きの辺として読む。
const TYPED: [&str; 21] = [
    "article",
    "articles",
    "amended_by",
    "amends",
    "ac",
    "adrs",
    "basis",
    "figures",
    "goals",
    "produced",
    "reads",
    "ref",
    "refs",
    "relations",
    "req",
    "reqs",
    "rules",
    "sections",
    "target",
    "verifies",
    "verify",
];

/// 来歴と反対側からの確認の欄（6 つ）。散文にも型付きの辺にも数えない。
const PROVENANCE: [&str; 6] = ["approval", "date", "grill", "ruled_at", "ruling", "source"];

/// 最上位の 2 節（承認と版の来歴・生成区間）。
const TOP_SKIPPED: [&str; 2] = ["meta", "schema"];

/// 数えない言及の語形（10 語）。言及を含む 1 文（区切りは 。）にどれかが在れば数えない。
const EXCLUDED: [&str; 10] = [
    "対象外",
    "本判断の外",
    "同じ運び方",
    "同形",
    "と同じく",
    "と揃う",
    "審査の記帳",
    "根拠から",
    "へ移した",
    "と書いたら",
];

/// id を持つ行の種類（節点の索引の種類・受け皿の表の鍵）。
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Article,
    Statement,
    Rule,
    Adr,
    Requirement,
    Constraint,
    Acceptance,
    Goal,
    Actor,
    Output,
}

/// 要件書の 7 節と行の種類。
const SRS_SECTIONS: [(&str, Kind); 7] = [
    ("goals", Kind::Goal),
    ("actors", Kind::Actor),
    ("outputs", Kind::Output),
    ("requirements", Kind::Requirement),
    ("nonfunctional", Kind::Requirement),
    ("acceptance", Kind::Acceptance),
    ("constraints", Kind::Constraint),
];

/// 受け皿の表: 出所の行の種類が既に持つ型付きの欄が、指す先の種類を受けられるか。受けられない組は数えない。
fn receives(from: Kind, to: Kind) -> bool {
    use Kind::*;
    match from {
        Article | Rule | Adr => true,
        Requirement => matches!(to, Article | Statement | Goal | Rule | Acceptance | Adr),
        Constraint => matches!(to, Article | Statement | Rule),
        Acceptance => to == Requirement,
        Statement | Goal | Actor | Output => false,
    }
}

/// 規範文の id を親の条へ丸める（条 id だけが枝番「.数字」を持つ）。
fn round(id: &str) -> &str {
    id.split_once('.').map_or(id, |(head, _)| head)
}

fn ids_in(text: &str) -> Vec<String> {
    let mut out = refs::scan_ids(text);
    out.extend(link::scan_adr_ids(text));
    out
}

/// 行 R-17 の入り口。行が無ければ None。値が 0 件 でなければ「まだ分からない」を立てて Some(false)。数えるなら Some(true)。
fn switch(rules: &Node, report: &mut Report) -> Option<bool> {
    let row = RULE_SECTIONS
        .iter()
        .filter_map(|s| rules.get(s))
        .filter_map(Node::as_seq)
        .flatten()
        .find(|r| r.get("id").and_then(Node::as_str) == Some(ROW_ID))?;
    let value = row.get("value").and_then(Node::as_str);
    if value != Some(VALUE) {
        report.unknown(format!(
            "rules.yaml: {ROW_ID} の値「{}」が {VALUE} でない＝散文の言及の歯は数えられない",
            value.unwrap_or("?")
        ));
        return Some(false);
    }
    Some(true)
}

/// 節点の索引（id → 種類）: 条と規範文・規則の表の 2 節・要件書の 7 節・判断の記録。
fn index(constitution: &Node, rules: &Node, srs: &Node, adr: &Adr) -> HashMap<String, Kind> {
    let mut out = HashMap::new();
    let mut put = |node: &Node, kind: Kind| {
        if let Some(id) = node.get("id").and_then(Node::as_str) {
            out.entry(id.to_string()).or_insert(kind);
        }
    };
    for article in section(constitution, "articles") {
        put(article, Kind::Article);
        for st in section(article, "statements") {
            put(st, Kind::Statement);
        }
    }
    for name in RULE_SECTIONS {
        for row in section(rules, name) {
            put(row, Kind::Rule);
        }
    }
    for (name, kind) in SRS_SECTIONS {
        for row in section(srs, name) {
            put(row, kind);
        }
    }
    for (id, _) in &adr.records {
        out.entry(id.clone()).or_insert(Kind::Adr);
    }
    out
}

fn section<'a>(node: &'a Node, key: &str) -> &'a [Node] {
    node.get(key).and_then(Node::as_seq).unwrap_or_default()
}

/// 1 file を歩いて集めたもの。
#[derive(Default)]
struct Walk<'a> {
    /// 行 id → その行の型付きの欄に在る id（丸めた形）
    typed: HashMap<String, HashSet<String>>,
    /// （file・囲む行の id・散文の字）
    prose: Vec<(String, String, &'a str)>,
}

impl<'a> Walk<'a> {
    fn file(&mut self, file: &str, root: &'a Node, index: &HashMap<String, Kind>) {
        // 最上位そのものが id を持つ行になるのは判断の記録
        let row = row_of(root, index);
        for (key, value) in root.as_map().unwrap_or_default() {
            if !TOP_SKIPPED.contains(&key.as_str()) {
                self.entry(file, key, value, row, index);
            }
        }
    }

    fn entry(&mut self, file: &str, key: &str, value: &'a Node, row: Option<&'a str>, index: &HashMap<String, Kind>) {
        if PROVENANCE.contains(&key) {
            return;
        }
        if TYPED.contains(&key) {
            if let Some(id) = row {
                self.typed_ids(id, value);
            }
            return;
        }
        self.node(file, value, row, index);
    }

    fn node(&mut self, file: &str, node: &'a Node, row: Option<&'a str>, index: &HashMap<String, Kind>) {
        match node {
            Node::Null => {}
            Node::Scalar(text) => {
                if let Some(id) = row {
                    self.prose.push((file.to_string(), id.to_string(), text));
                }
            }
            Node::Seq(items) => {
                for item in items {
                    self.node(file, item, row, index);
                }
            }
            Node::Map(entries) => {
                let row = row_of(node, index).or(row);
                for (key, value) in entries {
                    self.entry(file, key, value, row, index);
                }
            }
        }
    }

    fn typed_ids(&mut self, row: &str, node: &Node) {
        let set = self.typed.entry(row.to_string()).or_default();
        let mut stack = vec![node];
        while let Some(n) = stack.pop() {
            match n {
                Node::Null => {}
                Node::Scalar(text) => set.extend(ids_in(text).iter().map(|id| round(id).to_string())),
                Node::Seq(items) => stack.extend(items),
                Node::Map(entries) => stack.extend(entries.iter().map(|(_, v)| v)),
            }
        }
    }
}

/// 表が id の欄を持ち、その値が節点の索引に在るならその id。
fn row_of<'a>(node: &'a Node, index: &HashMap<String, Kind>) -> Option<&'a str> {
    node.get("id")
        .and_then(Node::as_str)
        .filter(|id| index.contains_key(*id))
}

/// 設計ノートの正本（design-note/ の欄の決まり以外）。読めない file は note.rs が数えるのでここでは黙って外す。
fn design_notes(dir: &Path) -> Vec<(String, Node)> {
    let nd = dir.join("design-note");
    let Ok(entries) = fs::read_dir(&nd) else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".yaml") && n != "schema.yaml")
        .collect();
    names.sort();
    names
        .into_iter()
        .filter_map(|name| {
            let path = nd.join(&name);
            if path.is_symlink() || !path.is_file() {
                return None;
            }
            let doc = yaml::parse(&fs::read_to_string(&path).ok()?).ok()?;
            Some((format!("design-note/{name}"), doc.root))
        })
        .collect()
}

/// 行 R-17 を数える。`files` は正本 7 file（file 名と木）。行 R-17 が無くて数えなかったときだけ false（入口が `OFF` を出す）。
pub(crate) fn check_mentions(dir: &Path, files: &[(&str, &Node)], adr: &Adr, report: &mut Report) -> bool {
    let get = |name: &str| files.iter().find(|(n, _)| *n == name).map(|(_, node)| *node);
    let (Some(constitution), Some(rules), Some(srs)) =
        (get("constitution.yaml"), get("rules.yaml"), get("srs.yaml"))
    else {
        return true;
    };
    match switch(rules, report) {
        None => return false,
        Some(false) => return true,
        Some(true) => {}
    }
    let index = index(constitution, rules, srs, adr);
    let notes = design_notes(dir);
    let mut walk = Walk::default();
    for (name, root) in files.iter().filter(|(n, _)| TARGETS.contains(n)) {
        walk.file(name, root, &index);
    }
    for (id, root) in &adr.records {
        walk.file(&format!("adr/{id}.yaml"), root, &index);
    }
    for (name, root) in &notes {
        walk.file(name, root, &index);
    }
    let empty = HashSet::new();
    let typed = |id: &str| walk.typed.get(id).unwrap_or(&empty);
    let mut seen: HashSet<(String, String)> = HashSet::new();
    for (file, src, text) in &walk.prose {
        let Some(&from) = index.get(src) else {
            continue;
        };
        for sentence in text.split('。') {
            if EXCLUDED.iter().any(|w| sentence.contains(w)) {
                continue;
            }
            for id in ids_in(sentence) {
                let Some(&to) = index.get(&id) else {
                    continue;
                };
                let target = round(&id);
                if id == *src || target == round(src) || !receives(from, to) {
                    continue;
                }
                if typed(src).contains(target) || typed(target).contains(round(src)) {
                    continue;
                }
                if seen.insert((src.clone(), target.to_string())) {
                    report.violation(
                        ROW_ID,
                        format!(
                            "{file}: {src} の散文が {id} を指すのに、{src} の型付きの欄にも {target} の型付きの欄にも無い（型付きの欄へ書き写す）"
                        ),
                    );
                }
            }
        }
    }
    true
}
