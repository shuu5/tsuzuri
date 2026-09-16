//! 正本の読み手。yaml-rust2 の低水準の event から木を組み、同じ表に同じキーを 2 度書いた箇所を拾う
//! （高水準の読み手は後のキーで黙って上書きする＝人と機械の読みが割れる）。

use std::collections::HashMap;

use yaml_rust2::parser::{Event, MarkedEventReceiver, Parser};
use yaml_rust2::scanner::{Marker, TScalarStyle};

/// 読んだ木。表は書かれた順に持つ（重複キーは最初の値を残し、`Doc::duplicates` に記録する）。
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    /// 値の無い欄（`~`・`null`・空）
    Null,
    Scalar(String),
    Seq(Vec<Node>),
    Map(Vec<(String, Node)>),
}

impl Node {
    pub fn as_map(&self) -> Option<&[(String, Node)]> {
        match self {
            Node::Map(m) => Some(m),
            _ => None,
        }
    }

    pub fn as_seq(&self) -> Option<&[Node]> {
        match self {
            Node::Seq(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Node::Scalar(s) => Some(s),
            _ => None,
        }
    }

    pub fn get(&self, key: &str) -> Option<&Node> {
        self.as_map()?
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v)
    }

    /// 空の欄か（無い・null・空白だけの文字列・空の一覧・空の表）。
    pub fn is_blank(&self) -> bool {
        match self {
            Node::Null => true,
            Node::Scalar(s) => s.trim().is_empty(),
            Node::Seq(s) => s.is_empty(),
            Node::Map(m) => m.is_empty(),
        }
    }
}

/// 同じ表に 2 度書かれたキー（1 始まりの行番号は 2 度目の位置）。
#[derive(Debug, Clone, PartialEq)]
pub struct Duplicate {
    pub key: String,
    pub line: usize,
}

#[derive(Debug)]
pub struct Doc {
    pub root: Node,
    pub duplicates: Vec<Duplicate>,
}

enum Frame {
    Seq(Vec<Node>, usize),
    /// 表・読みかけのキー・表の anchor
    Map(Vec<(String, Node)>, Option<String>, usize),
}

#[derive(Default)]
struct Builder {
    stack: Vec<Frame>,
    root: Option<Node>,
    anchors: HashMap<usize, Node>,
    duplicates: Vec<Duplicate>,
    error: Option<String>,
    docs: usize,
}

impl Builder {
    fn push_value(&mut self, node: Node, anchor: usize, mark: Marker) {
        if anchor > 0 {
            self.anchors.insert(anchor, node.clone());
        }
        match self.stack.last_mut() {
            None => self.root = Some(node),
            Some(Frame::Seq(items, _)) => items.push(node),
            Some(Frame::Map(entries, pending, _)) => match pending.take() {
                None => match node {
                    Node::Scalar(k) => *pending = Some(k),
                    Node::Null => *pending = Some(String::new()),
                    _ => {
                        self.error.get_or_insert_with(|| {
                            format!("{} 行: 表のキーが文字列でない", mark.line())
                        });
                        *pending = Some(String::new());
                    }
                },
                Some(key) => {
                    if entries.iter().any(|(k, _)| *k == key) {
                        self.duplicates.push(Duplicate {
                            key,
                            line: mark.line(),
                        });
                    } else {
                        entries.push((key, node));
                    }
                }
            },
        }
    }
}

impl MarkedEventReceiver for Builder {
    fn on_event(&mut self, ev: Event, mark: Marker) {
        match ev {
            Event::DocumentStart => {
                self.docs += 1;
                if self.docs > 1 {
                    self.error
                        .get_or_insert_with(|| "文書が 2 つ以上ある".to_string());
                }
            }
            Event::Scalar(value, style, anchor, tag) => {
                let null = tag.is_none()
                    && matches!(style, TScalarStyle::Plain)
                    && matches!(value.as_str(), "" | "~" | "null" | "Null" | "NULL");
                let node = if null {
                    Node::Null
                } else {
                    Node::Scalar(value)
                };
                self.push_value(node, anchor, mark);
            }
            Event::SequenceStart(anchor, _) => self.stack.push(Frame::Seq(Vec::new(), anchor)),
            Event::MappingStart(anchor, _) => self.stack.push(Frame::Map(Vec::new(), None, anchor)),
            Event::SequenceEnd => {
                if let Some(Frame::Seq(items, anchor)) = self.stack.pop() {
                    self.push_value(Node::Seq(items), anchor, mark);
                }
            }
            Event::MappingEnd => {
                if let Some(Frame::Map(entries, _, anchor)) = self.stack.pop() {
                    self.push_value(Node::Map(entries), anchor, mark);
                }
            }
            Event::Alias(id) => match self.anchors.get(&id).cloned() {
                Some(node) => self.push_value(node, 0, mark),
                None => {
                    self.error.get_or_insert_with(|| {
                        format!("{} 行: 未定義の別名（alias）", mark.line())
                    });
                }
            },
            _ => {}
        }
    }
}

/// YAML の文字列を読む。構文が壊れている・文書が 1 つでない・キーが文字列でない、は Err（読めない）。
/// 重複キーは読めたうえでの違反として `Doc::duplicates` に返す。
pub fn parse(text: &str) -> Result<Doc, String> {
    let mut builder = Builder::default();
    Parser::new_from_str(text)
        .load(&mut builder, true)
        .map_err(|e| e.to_string())?;
    if let Some(e) = builder.error {
        return Err(e);
    }
    let root = builder.root.ok_or_else(|| "文書が空".to_string())?;
    Ok(Doc {
        root,
        duplicates: builder.duplicates,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplicate_keys_are_found_in_nested_and_flow_maps() {
        let doc = parse("a: 1\nb:\n  c: 2\n  c: 3\nd: [{e: 1, e: 2}]\n").unwrap();
        let keys: Vec<_> = doc.duplicates.iter().map(|d| d.key.as_str()).collect();
        assert_eq!(keys, ["c", "e"]);
        assert_eq!(
            doc.root.get("b").unwrap().get("c"),
            Some(&Node::Scalar("2".into()))
        );
    }

    #[test]
    fn blank_and_broken() {
        let doc = parse("a: \"\"\nb:\nc: ~\nd: x\n").unwrap();
        for k in ["a", "b", "c"] {
            assert!(doc.root.get(k).unwrap().is_blank(), "{k}");
        }
        assert!(!doc.root.get("d").unwrap().is_blank());
        assert!(parse("a: [1, 2\n").is_err());
        assert!(parse("").is_err());
    }
}
