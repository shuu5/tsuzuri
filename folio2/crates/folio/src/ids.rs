//! 要件・判断・受入基準の id の消失と改番（便 88・docs/design/delivery-88.md §1 (b)(c)(d)）。
//! baseline は `anchors/` の直下の `ids-*.yaml`（id と規範文の要約値だけを持つ軽い凍結 anchor）の和。
//! baseline に在って現行に無い id は P-7（番号を消した）、その要約値が現行の別の id に在れば P-7.1（同じ本文を
//! 別の番号に付け替えた）。1 件の消えた id は 1 件だけ数える。足した id と本文だけの改訂は違反にしない。
//! anchor が 1 本も無ければ「まだ分からない」（P-4.2・P-10.3）。凍結は `folio check --freeze-ids`。

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::adr;
use crate::anchor;
use crate::phase::{self, After, Flag};
use crate::sha256;
use crate::verdict::{Report, Verdict};
use crate::yaml::{self, Node, Value};

/// file 名の頭と尻（`ids-<要件書の版>.yaml`）と kind の値。
const IDS_PREFIX: &str = "ids-";
const IDS_SUFFIX: &str = ".yaml";
const IDS_KIND: &str = "ids-anchor";

/// 節と、要約する欄（先に在る方）。adr は判断の記録（`adr/ADR-n.yaml`）。
const SECTIONS: [(&str, &[&str]); 4] = [
    ("requirements", &["shall", "title"]),
    ("nonfunctional", &["shall", "title"]),
    ("acceptance", &["title"]),
    ("adr", &["title"]),
];

/// id の一覧の 1 行。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Row {
    section: usize,
    key: (String, u64),
    id: String,
    sum: String,
}

/// 現行の id の一覧（凍結の材料）。
pub(crate) struct Current {
    rows: Vec<Row>,
    /// 要件書 meta.version（読めなければ None）
    version: Option<String>,
    anchors_dir: PathBuf,
}

/// 字面の UTF-8 の byte 列の sha256 の 16 進（前後の空白は落とさない）。
fn summary(text: &str) -> String {
    sha256::hex(text.as_bytes())
}

/// id の並びの鍵（数字でない頭と、末尾の数字列の値）。
fn sort_key(id: &str) -> (String, u64) {
    let head = id.trim_end_matches(|c: char| c.is_ascii_digit());
    (head.to_string(), id[head.len()..].parse().unwrap_or(0))
}

fn row(section: usize, id: &str, sum: String) -> Row {
    Row {
        section,
        key: sort_key(id),
        id: id.to_string(),
        sum,
    }
}

/// 行の要約値（要約する欄のうち先に在る字の欄・無ければ空の字面）。
fn row_summary(node: &Node, fields: &[&str]) -> String {
    let text = fields
        .iter()
        .find_map(|f| node.get(f).and_then(Node::as_str))
        .unwrap_or_default();
    summary(text)
}

/// 要件書の 3 節と判断の記録から現行の id の一覧を組む。
fn current_rows(srs: &Node, records: &[(String, Node)]) -> Vec<Row> {
    let mut rows = Vec::new();
    for (i, (section, fields)) in SECTIONS.iter().enumerate() {
        if *section == "adr" {
            for (id, rec) in records {
                rows.push(row(i, id, row_summary(rec, fields)));
            }
            continue;
        }
        for item in srs.get(section).and_then(Node::as_seq).unwrap_or_default() {
            if let Some(id) = item.get("id").and_then(Node::as_str) {
                rows.push(row(i, id, row_summary(item, fields)));
            }
        }
    }
    rows.sort();
    rows
}

/// `anchors/` の直下の `ids-*.yaml` の名前（名前順）。
fn ids_names(anchors: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(anchors)
        .map(|rd| {
            rd.filter_map(Result::ok)
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .filter(|n| {
                    n.len() > IDS_PREFIX.len() + IDS_SUFFIX.len()
                        && n.starts_with(IDS_PREFIX)
                        && n.ends_with(IDS_SUFFIX)
                })
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    names
}

/// 読めた anchor の行（id・要約値）。形が違えば None。
fn anchor_rows(doc: &Value) -> Option<Vec<(String, String)>> {
    if doc.get("kind").and_then(Value::as_str) != Some(IDS_KIND)
        || doc.get("digest_algo").and_then(Value::as_str)
            != adr::floor_val(&["anchor", "digest_algo"])
    {
        return None;
    }
    doc.get("ids")?
        .as_seq()?
        .iter()
        .map(|r| {
            let id = r.get("id")?.as_str()?;
            let sum = r.get("sum")?.as_str()?;
            Some((id.to_string(), sum.to_string()))
        })
        .collect()
}

/// id の一覧の検査（§1 (c)）。凍結の材料を返す。
pub(crate) fn check_ids(
    dir: &Path,
    srs: &Node,
    records: &[(String, Node)],
    flag: Flag,
    report: &mut Report,
) -> Current {
    let anchors_dir = dir.join(adr::floor_val(&["anchor", "dir"]).unwrap_or_default());
    let rows = current_rows(srs, records);
    let version = srs
        .get("meta")
        .and_then(|m| m.get("version"))
        .and_then(Node::as_str)
        .map(str::to_string);
    let names = if anchors_dir.is_dir() {
        ids_names(&anchors_dir)
    } else {
        Vec::new()
    };
    // 読めた anchor の ids の和（id → 記録された要約値）
    let mut baseline: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for name in &names {
        let file = format!("anchors/{name}");
        let Some(doc) = anchor::read_typed(&anchors_dir.join(name), &file, report) else {
            continue;
        };
        let Some(ids) = anchor_rows(&doc) else {
            report.unknown(format!(
                "{file}: id の一覧の anchor の形（kind {IDS_KIND}・digest_algo・ids の id と sum）でない"
            ));
            continue;
        };
        let stored = doc.get("digest").and_then(Value::as_str);
        match anchor::digest_of(&doc) {
            Ok(d) if stored == Some(d.as_str()) => {}
            Ok(_) => {
                report.violation(
                    "anchor",
                    format!("{file}: digest が中身と合わない（手で直した anchor・baseline に混ぜない）"),
                );
                continue;
            }
            Err(e) => {
                report.unknown(format!("{file}: digest を計算できない（{e}）"));
                continue;
            }
        }
        for (id, sum) in ids {
            let sums = baseline.entry(id).or_default();
            if !sums.contains(&sum) {
                sums.push(sum);
            }
        }
    }
    // 読めないのではなく測れない（違反が在れば不合格が先に立つ）
    if names.is_empty() && flag != Flag::FreezeIds {
        report.pending(
            "要件・判断・受入基準の id の消失と改番は baseline（anchors/ids-*.yaml）が無いので測れない（folio check --freeze-ids で凍結できる）",
        );
    }
    for (id, sums) in &baseline {
        if rows.iter().any(|r| &r.id == id) {
            continue;
        }
        match rows.iter().find(|r| sums.contains(&r.sum)) {
            Some(r) => report.violation(
                "P-7.1",
                format!(
                    "{id} の本文が {} へ付け替えられた（要約値が同じ・id は改番しない＝{id} を戻し、新しい本文には新しい番号を振る）",
                    r.id
                ),
            ),
            None => report.violation(
                "P-7",
                format!(
                    "{id} が消えた（baseline の anchors/ids-*.yaml に在る・番号は消さず、廃止は状態で表す・P-7.2）"
                ),
            ),
        }
    }
    Current {
        rows,
        version,
        anchors_dir,
    }
}

/// `--freeze-ids`（§1 (d)）。全検査が 0 違反で「まだ分からない」も無いときだけ書く。
pub(crate) fn freeze(cur: &Current, report: &mut Report) -> After {
    let Some(version) = &cur.version else {
        report.unknown("srs.yaml: meta.version が読めない（id の一覧の anchor の file 名を決められない）");
        return After::Freeze(phase::not_frozen_by(report, "--freeze-ids"));
    };
    let name = format!("{IDS_PREFIX}{version}{IDS_SUFFIX}");
    let path = cur.anchors_dir.join(&name);
    if path.exists() || path.is_symlink() {
        return After::Refused(format!(
            "anchors/{name} が既に在る（同じ版は上書きしない・N-1.1）"
        ));
    }
    if report.verdict() != Verdict::Pass {
        return After::Freeze(phase::not_frozen_by(report, "--freeze-ids"));
    }
    let s = |x: &str| Value::Str(x.to_string());
    let fields = SECTIONS
        .iter()
        .map(|(sec, f)| (s(sec), Value::Seq(f.iter().map(|x| s(x)).collect())))
        .collect();
    let ids = cur
        .rows
        .iter()
        .map(|r| {
            Value::Map(vec![
                (s("id"), s(&r.id)),
                (s("section"), s(SECTIONS[r.section].0)),
                (s("sum"), s(&r.sum)),
            ])
        })
        .collect();
    let mut tree = vec![
        (s("kind"), s(IDS_KIND)),
        (
            s("digest_algo"),
            s(adr::floor_val(&["anchor", "digest_algo"]).unwrap_or_default()),
        ),
        (s("version"), s(version)),
        (
            s("projection"),
            Value::Map(vec![
                (
                    s("sections"),
                    Value::Seq(SECTIONS.iter().map(|(sec, _)| s(sec)).collect()),
                ),
                (s("fields"), Value::Map(fields)),
            ]),
        ),
        (s("ids"), Value::Seq(ids)),
    ];
    let written = anchor::digest_of(&Value::Map(tree.clone())).and_then(|d| {
        tree.push((s("digest"), s(&d)));
        let header = format!(
            "# folio2 要件・判断・受入基準の id の一覧の凍結 anchor（要件書 {version} の時点・P-7.1）。行 = id・節・要約値（欄の字面の UTF-8 の sha256・欄は projection.fields の先に在る方）。手で直さない・消さない・同じ版は上書きしない（folio check --freeze-ids が全検査 0 違反のときだけ作る）。"
        );
        yaml::write(&Value::Map(tree), &header)
    });
    let text = match written {
        Ok(t) => t,
        Err(e) => {
            report.pending(format!("id の一覧の木を書けない（{e}）"));
            return After::Freeze(phase::not_frozen_by(report, "--freeze-ids"));
        }
    };
    if let Err(e) = fs::create_dir_all(&cur.anchors_dir).and_then(|()| fs::write(&path, text)) {
        report.unknown(format!("anchors/ に書けない: {e}"));
        return After::Freeze(phase::not_frozen_by(report, "--freeze-ids"));
    }
    After::Freeze(format!(
        "凍結した: {}（id {} 本）・版管理に commit する（commit するまで素の床は未追跡の anchor で 1）",
        path.display(),
        cur.rows.len()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_sort_key_orders_numbers_as_numbers() {
        assert!(sort_key("FR2") < sort_key("FR10"));
        assert!(sort_key("ADR-9") < sort_key("ADR-12"));
    }

    #[test]
    fn ids_summary_is_the_plain_sha256_of_the_text() {
        assert_eq!(
            summary("abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
