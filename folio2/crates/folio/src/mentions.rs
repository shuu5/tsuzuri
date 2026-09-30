//! 規則の表の行 R-17 の床の歯（便 93・docs/design/delivery-93.md §1）。設計文書の中で id を持つ行の散文の欄に現れた、
//! その行以外の id のうち、その行の型付きの欄に無いものを数える（値 0 件）。
//! 機械の読みは 5 つの閉じた一覧で閉じる＝対象の file（9 種）・id を持つ行（節点の索引の 5 種）・散文の欄
//! （型付きの欄 21 語と来歴 6 語と最上位 2 節の補集合）・受け皿の表・数えない言及の語形（10 語）。どれもこの file の定数（P-5.1）で、
//! 索引の欄の決まり graph.yaml の生成区間の mentions はその写し（P-5.6・行 D-11・便 195）。
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

/// 入り口の行と、歯が数える値（行が在る節は refs.rs の RULE_SECTIONS）。
const ROW_ID: &str = "R-17";
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
pub(crate) const TYPED: [&str; 21] = [
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
pub(crate) const PROVENANCE: [&str; 6] = ["approval", "date", "grill", "ruled_at", "ruling", "source"];

/// 最上位の 2 節（承認と版の来歴・生成区間）。
pub(crate) const TOP_SKIPPED: [&str; 2] = ["meta", "schema"];

/// 数えない言及の語形（10 語）。言及を含む 1 文（区切りは 。）にどれかが在れば数えない。
pub(crate) const EXCLUDED: [&str; 10] = [
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
pub(crate) enum Kind {
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

impl Kind {
    /// 種類の名（受け皿の表の鍵と値・索引の欄の決まりの写しの字）。要件は要件書の requirements と nonfunctional の両方の行。
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Kind::Article => "条",
            Kind::Statement => "規範文",
            Kind::Rule => "規則行",
            Kind::Adr => "判断の記録",
            Kind::Requirement => "要件",
            Kind::Constraint => "制約",
            Kind::Acceptance => "受入基準",
            Kind::Goal => "目的",
            Kind::Actor => "登場人物",
            Kind::Output => "出力",
        }
    }
}

/// 要件書の 7 節と行の種類。
pub(crate) const SRS_SECTIONS: [(&str, Kind); 7] = [
    ("goals", Kind::Goal),
    ("actors", Kind::Actor),
    ("outputs", Kind::Output),
    ("requirements", Kind::Requirement),
    ("nonfunctional", Kind::Requirement),
    ("acceptance", Kind::Acceptance),
    ("constraints", Kind::Constraint),
];

/// 受け皿の表（種類の名で引く）: 鍵 = 出所の行の種類・値 = その種類の行が既に持つ型付きの欄が受けられる指す先の種類。
/// 値に無い組（空の一覧の行は全部）は数えない。種類の名は `Kind::name`。
pub(crate) const RECEIVES: [(&str, &[&str]); 10] = [
    ("条", &["条", "規範文", "規則行", "判断の記録", "要件", "制約", "受入基準", "目的", "登場人物", "出力"]),
    ("規範文", &[]),
    ("規則行", &["条", "規範文", "規則行", "判断の記録", "要件", "制約", "受入基準", "目的", "登場人物", "出力"]),
    ("判断の記録", &["条", "規範文", "規則行", "判断の記録", "要件", "制約", "受入基準", "目的", "登場人物", "出力"]),
    ("要件", &["条", "規範文", "規則行", "判断の記録", "受入基準", "目的"]),
    ("制約", &["条", "規範文", "規則行"]),
    ("受入基準", &["要件"]),
    ("目的", &[]),
    ("登場人物", &[]),
    ("出力", &[]),
];

/// 出所の行の種類が、指す先の種類を受けられるか（受け皿の表を種類の名で引く・表に無い鍵は受けない）。
fn receives(from: Kind, to: Kind) -> bool {
    RECEIVES
        .iter()
        .find(|(key, _)| *key == from.name())
        .is_some_and(|(_, kinds)| kinds.contains(&to.name()))
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
    let row = refs::RULE_SECTIONS
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
    for name in refs::RULE_SECTIONS {
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

    #[expect(
        clippy::too_many_arguments,
        reason = "引数が規則の行 R-4 の 5 を越える・行 r4-folio-src-b が直してこの属性を外す"
    )]
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

#[cfg(test)]
mod tests {
    use super::*;

    /// 受け皿の表の鍵と行列の順（Kind の宣言の順）。
    const ALL: [Kind; 10] = [
        Kind::Article,
        Kind::Statement,
        Kind::Rule,
        Kind::Adr,
        Kind::Requirement,
        Kind::Constraint,
        Kind::Acceptance,
        Kind::Goal,
        Kind::Actor,
        Kind::Output,
    ];

    /// 便 195 の歯 3: 受け皿の表の答えの全数（10 × 10）。行 = 出所・列 = 指す先（ALL の順）・1 = 受ける。
    /// 期待の行列は便 93 の match の式（base 7528256）を手で写した字で、表を型付きの定数に置き換えても答えは 1 つも変わらない。
    #[test]
    fn f195_receives_answers_every_pair_as_before() {
        const WANT: [&str; 10] = [
            "1111111111", // 条
            "0000000000", // 規範文
            "1111111111", // 規則行
            "1111111111", // 判断の記録
            "1111001100", // 要件（要件書の requirements と nonfunctional の行）
            "1110000000", // 制約
            "0000100000", // 受入基準
            "0000000000", // 目的
            "0000000000", // 登場人物
            "0000000000", // 出力
        ];
        let mut ones = 0;
        for (i, (from, row)) in ALL.iter().zip(WANT).enumerate() {
            for (j, (to, want)) in ALL.iter().zip(row.chars()).enumerate() {
                assert_eq!(receives(*from, *to), want == '1', "行 {i} → 列 {j}");
                ones += usize::from(want == '1');
            }
        }
        assert_eq!(ones, 40, "受ける組の数");
    }

    /// 便 195 の歯 4: 受け皿の表の鍵は 10 の種類の名が宣言の順にちょうど 1 つずつで、値はどれも種類の名（綴りの誤りは受けない組を黙って増やす）。
    /// 要件書の 7 節の種類の名も種類の名。
    #[test]
    fn f195_receives_table_is_keyed_by_the_kind_names() {
        let names: Vec<&str> = ALL.iter().map(|k| k.name()).collect();
        let keys: Vec<&str> = RECEIVES.iter().map(|(k, _)| *k).collect();
        assert_eq!(keys, names);
        for (key, kinds) in RECEIVES {
            for to in kinds {
                assert!(names.contains(to), "{key} の値 {to} が種類の名でない");
            }
        }
        let srs: Vec<(&str, &str)> = SRS_SECTIONS.iter().map(|(s, k)| (*s, k.name())).collect();
        assert_eq!(
            srs,
            [
                ("goals", "目的"),
                ("actors", "登場人物"),
                ("outputs", "出力"),
                ("requirements", "要件"),
                ("nonfunctional", "要件"),
                ("acceptance", "受入基準"),
                ("constraints", "制約"),
            ]
        );
    }
}
