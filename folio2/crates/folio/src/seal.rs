//! 判断の記録の封（便 170・docs/design/delivery-170.md §1 (b)3・ADR-30 決定 (3)）。発効した判断の記録の本文を
//! `anchors/adr-seals.yaml`（封の一覧）の行で凍らせる。行 = id と要約値（退役で変えてよい欄〔床の定数 SEAL_OUTSIDE〕を
//! 除く記録の木の正規化〔凍結 anchor の digest と同じ `yaml::canonical`〕の sha256）で、足した順に並ぶ。
//! 要約値は置き場ごとの data で、source に焼かない（P-5.1）。封の一覧の中身は `folio check --freeze-adrs` の出力（P-6.2）。
//! 検査（`check_seals`）は口（`check.rs`）が id の一覧の検査の後に呼び、凍結（`freeze`）は後始末（`freeze.rs`）の腕 1 つが呼ぶ。

use std::fs;
use std::path::{Path, PathBuf};

use crate::adr;
use crate::anchor;
use crate::floor_adr::{EFFECTIVE_STATUS, SEAL_FILE, SEAL_KIND, SEAL_OUTSIDE};
use crate::phase::{self, After, Flag};
use crate::sha256;
use crate::verdict::{Report, Verdict};
use crate::yaml::{self, Node, Value};

const FLAG: &str = "--freeze-adrs";

/// 封の検査が残した凍結の材料。
pub(crate) struct Seals {
    /// `<dir>/anchors/adr-seals.yaml`
    path: PathBuf,
    /// `<dir>/anchors`
    anchors_dir: PathBuf,
    /// 封の一覧の在る行（file の順・file が無ければ空）
    rows: Vec<(String, String)>,
    /// 発効した記録の id と今の要約値（id の数の順）
    current: Vec<(String, String)>,
    /// 本文が行と違う発効した記録の id
    changed: Vec<String>,
    /// 置き場の名（頭の注に出す・便 154 と同じ導き方）
    name: Option<String>,
}

/// 記録の木から SEAL_OUTSIDE の欄を除いた表の正規化の sha256（封の要約値）。表でない・正規化できなければ Err。
pub(crate) fn body_sum(record: &Value) -> Result<String, String> {
    let body: Vec<(Value, Value)> = record
        .as_map()
        .ok_or_else(|| "判断の記録が欄の表でない".to_string())?
        .iter()
        .filter(|(k, _)| !k.as_str().is_some_and(|k| SEAL_OUTSIDE.contains(&k)))
        .cloned()
        .collect();
    yaml::canonical(&Value::Map(body)).map(|s| sha256::hex(s.as_bytes()))
}

/// id の並びの鍵（ADR- の後の数）。
fn id_key(id: &str) -> (u64, String) {
    let n = id
        .strip_prefix("ADR-")
        .and_then(|n| n.parse().ok())
        .unwrap_or(u64::MAX);
    (n, id.to_string())
}

/// 凍結の旗（行と file の欠けを前提の検査に数えない・FR24）。
fn freezing(flag: Flag) -> bool {
    matches!(
        flag,
        Flag::FreezeAnchor | Flag::FreezeIds | Flag::FreezeStart | Flag::FreezeAdrs
    )
}

/// 読めた封の一覧の行（id・要約値）。形が違えば None。
fn file_rows(doc: &Value) -> Option<Vec<(String, String)>> {
    if doc.get("kind").and_then(Value::as_str) != Some(SEAL_KIND)
        || doc.get("digest_algo").and_then(Value::as_str)
            != adr::floor_val(&["anchor", "digest_algo"])
        || !anchor::str_list_eq(doc.get("outside"), SEAL_OUTSIDE)
        || doc.get("digest").and_then(Value::as_str).is_none()
    {
        return None;
    }
    doc.get("rows")?
        .as_seq()?
        .iter()
        .map(|r| {
            let id = r.get("id")?.as_str()?;
            let sum = r.get("sum")?.as_str()?;
            Some((id.to_string(), sum.to_string()))
        })
        .collect()
}

/// 封の検査（§1 (b)3）。`records` は読めた判断の記録。凍結の材料を返す。
pub(crate) fn check_seals(
    dir: &Path,
    records: &[(String, Node)],
    flag: Flag,
    report: &mut Report,
) -> Seals {
    let anchors_dir = dir.join(adr::floor_val(&["anchor", "dir"]).unwrap_or_default());
    let path = anchors_dir.join(SEAL_FILE);
    let file = format!("anchors/{SEAL_FILE}");
    let mut current: Vec<(String, String)> = Vec::new();
    for (id, d) in records {
        if !adr::in_enum(d.get("status"), EFFECTIVE_STATUS) {
            continue;
        }
        let at = format!("adr/{id}.yaml");
        let Some(tree) = anchor::read_typed(&dir.join(&at), &at, report) else {
            continue;
        };
        match body_sum(&tree) {
            Ok(sum) => current.push((id.clone(), sum)),
            Err(e) => report.unknown(format!("{at}: 封の要約値を計算できない（{e}）")),
        }
    }
    current.sort_by_key(|(id, _)| id_key(id));
    let mut seals = Seals {
        path,
        anchors_dir,
        rows: Vec::new(),
        current,
        changed: Vec::new(),
        name: adr::name_of(dir),
    };
    if seals.path.is_symlink() {
        report.violation("adr", format!("{file}: symlink は認めない"));
        return seals;
    }
    if !seals.path.exists() {
        if !seals.current.is_empty() && !freezing(flag) {
            report.pending(format!(
                "{file}（判断の記録の封の一覧）が無い＝発効した判断の記録 {} 本の本文の凍結を測れない（folio check --freeze-adrs で封を書き、commit する・P-10.3）",
                seals.current.len()
            ));
        }
        return seals;
    }
    let Some(doc) = anchor::read_typed(&seals.path, &file, report) else {
        return seals;
    };
    let Some(rows) = file_rows(&doc) else {
        report.pending(format!(
            "{file}: 封の一覧の形（kind {SEAL_KIND}・digest_algo・outside・rows の id と sum・digest）でない＝測れない"
        ));
        return seals;
    };
    match anchor::digest_of(&doc) {
        Ok(d) if doc.get("digest").and_then(Value::as_str) == Some(d.as_str()) => {}
        Ok(_) => {
            report.violation(
                "adr",
                format!("{file}: digest が中身と合わない（手で直した封・行を照らさない）"),
            );
            return seals;
        }
        Err(e) => {
            report.unknown(format!("{file}: digest を計算できない（{e}）"));
            return seals;
        }
    }
    let mut seen: Vec<&str> = Vec::new();
    for (id, _) in &rows {
        if seen.contains(&id.as_str()) {
            report.violation("adr", format!("{file}: {id} の行が 2 つ在る（行は id ごとに 1 つ）"));
        } else {
            seen.push(id);
        }
    }
    for (id, sum) in &seals.current {
        match rows.iter().find(|(r, _)| r == id) {
            Some((_, sealed)) if sealed != sum => {
                report.violation(
                    "adr",
                    format!(
                        "{id}: 発効した判断の記録の本文が封（{file}）の行と違う（発効した記録の本文は変えない・退役で変えてよいのは status と superseded_by だけ・判断を変えるなら新しい判断の記録を立てる）"
                    ),
                );
                seals.changed.push(id.clone());
            }
            Some(_) => {}
            None if freezing(flag) => {}
            None => report.link(
                "adr",
                format!(
                    "{id}: 発効しているのに封の行が無い（folio check --freeze-adrs で封を足し、commit する）"
                ),
            ),
        }
    }
    // 発効しているが読めずに要約値を持たない記録（まだ分からない が先に立つ）の行は数えない
    let effective = |id: &str| {
        records
            .iter()
            .any(|(r, d)| r == id && adr::in_enum(d.get("status"), EFFECTIVE_STATUS))
    };
    for (id, _) in &rows {
        if !effective(id) {
            report.violation(
                "adr",
                format!(
                    "{id}: 封（{file}）に行が在るのに発効した判断の記録が無い（発効した記録は消さない・proposed に戻さない・P-7.2）"
                ),
            );
        }
    }
    seals.rows = rows;
    seals
}

/// `--freeze-adrs`（§1 (b)3）。本文が行と違う記録が在れば断る。全検査が 0 違反で「まだ分からない」も無いときだけ、
/// 在る行を字のまま残し、欠けた行を id の数の順で末尾に足して書く。足す行が無ければ書かない。
pub(crate) fn freeze(seals: &Seals, report: &mut Report) -> After {
    if let Some(id) = seals.changed.first() {
        return After::Refused(format!(
            "{FLAG}: anchors/{SEAL_FILE} の在る行（{id}）は書き換えない（封は足すだけ・N-1.1。発効した記録の本文を戻すか、判断を変えるなら新しい判断の記録を立てる）"
        ));
    }
    if report.verdict() != Verdict::Pass {
        return After::Freeze(phase::not_frozen_by(report, FLAG));
    }
    let added: Vec<&(String, String)> = seals
        .current
        .iter()
        .filter(|(id, _)| !seals.rows.iter().any(|(r, _)| r == id))
        .collect();
    if added.is_empty() {
        return After::Freeze(format!(
            "足す封の行は無い（{} の行 {} はどれも発効した記録と同じ・何も書かない）",
            seals.path.display(),
            seals.rows.len()
        ));
    }
    let rows: Vec<(String, String)> = seals
        .rows
        .iter()
        .chain(added.iter().copied())
        .cloned()
        .collect();
    let text = match build(&rows, seals.name.as_deref()) {
        Ok(t) => t,
        Err(e) => {
            report.pending(format!("封の一覧の木を書けない（{e}）"));
            return After::Freeze(phase::not_frozen_by(report, FLAG));
        }
    };
    if let Err(e) = fs::create_dir_all(&seals.anchors_dir).and_then(|()| fs::write(&seals.path, text)) {
        report.unknown(format!("anchors/ に書けない: {e}"));
        return After::Freeze(phase::not_frozen_by(report, FLAG));
    }
    let ids: Vec<&str> = added.iter().map(|(id, _)| id.as_str()).collect();
    After::Freeze(format!(
        "封を足した: {}（足した行 {}・在る行 {} は変えない）・版管理に commit する",
        seals.path.display(),
        ids.join("・"),
        seals.rows.len()
    ))
}

/// 封の一覧の本文を組む（digest を足した木を書き手で字にする）。
fn build(rows: &[(String, String)], name: Option<&str>) -> Result<String, String> {
    let s = |x: &str| Value::Str(x.to_string());
    let rows = rows
        .iter()
        .map(|(id, sum)| Value::Map(vec![(s("id"), s(id)), (s("sum"), s(sum))]))
        .collect();
    let mut tree = vec![
        (s("kind"), s(SEAL_KIND)),
        (
            s("digest_algo"),
            s(adr::floor_val(&["anchor", "digest_algo"]).unwrap_or_default()),
        ),
        (
            s("outside"),
            Value::Seq(SEAL_OUTSIDE.iter().map(|x| s(x)).collect()),
        ),
        (s("rows"), Value::Seq(rows)),
    ];
    let digest = anchor::digest_of(&Value::Map(tree.clone()))?;
    tree.push((s("digest"), s(&digest)));
    let header = format!(
        "# {}",
        adr::named(
            name,
            " ",
            "判断の記録の封の一覧（ADR-30 決定 (3)）。行 = 発効した判断の記録の id と要約値（outside の欄を除く記録の木の json の sha256）。足すだけ・手で直さない・消さない（folio check --freeze-adrs が全検査 0 違反のときだけ欠けた行を末尾に足す）。"
        )
    );
    yaml::write(&Value::Map(tree), &header)
}

#[cfg(test)]
mod tests {
    use super::*;

    const RECORD: &str = "id: ADR-1\ntitle: 封\nstatus: accepted\ndate: 2026-09-27\nsuperseded_by: ADR-2\nbasis: [P-1]\n";

    /// 要約値は status と superseded_by だけを除いた木の正規化の sha256（便 170 §1 (c)8）。字の json は手で写した凍結の字
    /// （Python の json.dumps(sort_keys・区切り「,」「:」・ensure_ascii なし) と同じ形）で、要約値はその字の sha256 の全桁。
    #[test]
    fn f170_the_body_sum_skips_only_status_and_superseded_by() {
        assert_eq!(SEAL_OUTSIDE, ["status", "superseded_by"]);
        let v = |t: &str| yaml::parse_typed(t).unwrap();
        let sum = body_sum(&v(RECORD)).unwrap();
        let json = "{\"basis\":[\"P-1\"],\"date\":\"2026-09-27\",\"id\":\"ADR-1\",\"title\":\"封\"}";
        assert_eq!(sum, sha256::hex(json.as_bytes()));
        assert_eq!(
            sum,
            "78ab3bb977dfaa34806a4d070662d8ea625677c95186ae537c63013118227db2"
        );
        let retired = RECORD
            .replace("status: accepted", "status: retired")
            .replace("superseded_by: ADR-2", "superseded_by: ADR-3");
        assert_eq!(body_sum(&v(&retired)).unwrap(), sum);
        let without = RECORD.replace("superseded_by: ADR-2\n", "");
        assert_eq!(body_sum(&v(&without)).unwrap(), sum);
        assert_ne!(body_sum(&v(&RECORD.replace("title: 封", "title: 封。"))).unwrap(), sum);
        assert_ne!(body_sum(&v(&RECORD.replace("date: 2026-09-27", "date: 2026-09-28"))).unwrap(), sum);
    }
}
