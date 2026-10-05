//! 係の床の記録と終える前の門の撃ち直しの判じ（判断の記録 ADR-63 決定 (4)・要件 FR21）。
//! 出す物が差の file（名の末が `.patch`）を名指す係は、出力の dir に床の記録 `floor.tsv` を置く。記録は差の file ごとに 1 行で、
//! 欄はタブで区切った 6 つ（行 id・preflight・tz check・tz derive --check・器の表の検査の rc・verify の 2 行目から最後の行の rc を `,` で
//! 繋いだ字）。`#` で始まる行は頭として読まない。門は安い 4 本を床の写しで撃ち直し、行ごとに、記録が無い・行が欠ける・
//! 撃ち直せない（まだ分からない）・撃ち直しと違う・rc が 0 でない・合う、の 1 つの型と詳細と欠けの字に判じる。
//! 判じは門の記録 `floor-gate.tsv` に 1 行ずつ足す。記録が在る事も撃ち直しの合格も、完成とも審査の合格とも扱わない（条 P-6.3）。

use std::collections::BTreeMap;

/// 係の床の記録の file（出力の dir の下）。
pub const RECORD: &str = "floor.tsv";

/// 門の判じの記録の file（出力の dir の下・判じるごとに 1 行ずつ足す）。
pub const LOG: &str = "floor-gate.tsv";

/// 門が preflight に渡す空の state dir（係の dir の下）。
pub const STATE: &str = "floor-state";

/// 撃ち直す 4 本の、門の記録の詳細の名と欠けの字の名（記録の 2〜5 欄の順）。
pub const CHEAP: [(&str, &str); 4] = [
    ("preflight", "preflight"),
    ("check", "tz check"),
    ("derive", "tz derive --check"),
    ("table", "器の表の検査"),
];

/// 床の写しの dir の名（起草の置き場の直下）。
pub fn copy_name(name: &str) -> String {
    format!("try-{name}-fl")
}

/// 判じの型（門の記録の 4 欄目の字）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    NoRecord,
    NoRow,
    Unknown,
    Mismatch,
    Rc,
    Ok,
}

impl Kind {
    /// 門の記録の字。
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::NoRecord => "no-record",
            Kind::NoRow => "no-row",
            Kind::Unknown => "unknown",
            Kind::Mismatch => "mismatch",
            Kind::Rc => "rc",
            Kind::Ok => "ok",
        }
    }
}

/// 床の記録の 1 行（安い 4 本の rc と、verify の 2 行目から最後の行の rc）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rec {
    pub rcs: [i32; 4],
    pub verify: Vec<i32>,
}

/// 門の撃ち直し（安い 4 本の rc と、記録に要る verify の rc の数）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rerun {
    pub rcs: [i32; 4],
    pub verify: usize,
}

/// 行ごとの判じ（行 id・型・詳細・欠けの字の列）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verdict {
    pub row: String,
    pub kind: Kind,
    pub detail: String,
    pub lacks: Vec<String>,
}

/// 出す物のうち差の file の行 id（file の名から `.patch` を除いた空でない字・出す物の順・重ねない）。
pub fn rows(outputs: &[String]) -> Vec<String> {
    let mut ids: Vec<String> = Vec::new();
    for o in outputs {
        let file = o.rsplit('/').next().unwrap_or(o);
        if let Some(id) = file.strip_suffix(".patch").filter(|s| !s.is_empty())
            && !ids.iter().any(|x| x == id)
        {
            ids.push(id.to_string());
        }
    }
    ids
}

/// 床の記録の字の行（`#` で始まる行を除き、タブで区切った 6 欄で、2〜5 欄が整数・6 欄が空か整数を `,` で繋いだ字の行だけ・同じ行 id は後の行）。
pub fn record(text: &str) -> BTreeMap<String, Rec> {
    let mut out = BTreeMap::new();
    for line in text.lines().filter(|l| !l.starts_with('#')) {
        let f: Vec<&str> = line.split('\t').collect();
        let [id, a, b, c, d, v] = f.as_slice() else {
            continue;
        };
        let rcs: Option<Vec<i32>> = [a, b, c, d].iter().map(|x| x.trim().parse().ok()).collect();
        let verify: Option<Vec<i32>> = if v.trim().is_empty() {
            Some(Vec::new())
        } else {
            v.split(',').map(|x| x.trim().parse().ok()).collect()
        };
        if let (Some([a, b, c, d]), Some(verify)) = (rcs.as_deref(), verify)
            && !id.is_empty()
        {
            out.insert(
                (*id).to_string(),
                Rec {
                    rcs: [*a, *b, *c, *d],
                    verify,
                },
            );
        }
    }
    out
}

/// 契約表の file の字から、行 `row` の頭（`[[contract]]` の行・1 から数える）と verify の行の数（無いか読めなければ None）。
pub fn locate(toml: &str, row: &str) -> Option<(usize, usize)> {
    let lines: Vec<&str> = toml.lines().collect();
    let id = format!("id = {}", serde_json::to_string(row).ok()?);
    let at = lines.iter().position(|l| *l == id)?;
    let head = lines
        .get(..at)?
        .iter()
        .rposition(|l| *l == "[[contract]]")?;
    let verify = lines
        .get(at..)?
        .iter()
        .take_while(|l| **l != "[[contract]]")
        .find_map(|l| l.strip_prefix("verify = "))?;
    let list: Vec<String> = serde_json::from_str(verify).ok()?;
    Some((head + 1, list.len()))
}

/// 器の表の検査の出力 `out` と rc から、契約表の file `note`（`contracts/<ノート>.toml`）の行の頭 `head` の行の rc
/// （検査の rc が 0 と 1 のどちらでもなければその rc・その行を名指す断りの行が在れば 1・無ければ 0）。
pub fn table_rc(out: &str, rc: i32, note: &str, head: usize) -> i32 {
    if rc != 0 && rc != 1 {
        return rc;
    }
    let named = format!("contracts: {note}:{head} ");
    i32::from(out.lines().any(|l| l.starts_with(&named)))
}

/// 記録の行と撃ち直しを比べた (型・詳細の項・欠けの字) の列（違う rc・0 でない rc・verify の rc の数の違い）。
fn compare(row: &str, rec: &Rec, re: &Rerun) -> Vec<(Kind, String, String)> {
    let mut out = Vec::new();
    for (((key, shown), had), got) in CHEAP.iter().zip(rec.rcs).zip(re.rcs) {
        if had != got {
            out.push((
                Kind::Mismatch,
                format!("{key}={had}/{got}"),
                format!("床の記録の行 {row} の {shown} の rc {had} が撃ち直しの rc {got} と違う"),
            ));
        } else if had != 0 {
            out.push((
                Kind::Rc,
                format!("{key}={had}"),
                format!("床の記録の行 {row} の {shown} の rc が {had}（0 でない）"),
            ));
        }
    }
    if rec.verify.len() != re.verify {
        out.push((
            Kind::NoRow,
            format!("verify={}/{}", rec.verify.len(), re.verify),
            format!(
                "床の記録の行 {row} の verify の rc が {} 個（verify の 2 行目から最後の行の {} 個が要る）",
                rec.verify.len(),
                re.verify
            ),
        ));
    }
    for (k, rc) in rec.verify.iter().enumerate().filter(|(_, rc)| **rc != 0) {
        out.push((
            Kind::Rc,
            format!("verify{}={rc}", k + 2),
            format!(
                "床の記録の行 {row} の verify の {} 行目の rc が {rc}（0 でない）",
                k + 2
            ),
        ));
    }
    out
}

/// 1 行の判じ（記録 `rec` の行・撃ち直し `rerun`）。
fn one(row: &str, rec: Option<&Rec>, rerun: &impl Fn(&str) -> Result<Rerun, String>) -> Verdict {
    let verdict = |kind: Kind, detail: String, lacks: Vec<String>| Verdict {
        row: row.to_string(),
        kind,
        detail,
        lacks,
    };
    let Some(rec) = rec else {
        let lack = format!(
            "床の記録 floor.tsv に行 {row} の行が無い（欄はタブで区切った 6 つ・rc は整数）"
        );
        return verdict(Kind::NoRow, "-".into(), vec![lack]);
    };
    let re = match rerun(row) {
        Ok(re) => re,
        Err(what) => {
            let lack = format!("床を撃ち直せない（まだ分からない）: {what}");
            return verdict(Kind::Unknown, what, vec![lack]);
        }
    };
    let found = compare(row, rec, &re);
    let Some(kind) = found.iter().map(|(k, _, _)| *k).min() else {
        return verdict(Kind::Ok, "-".into(), Vec::new());
    };
    let detail = found
        .iter()
        .map(|(_, d, _)| d.as_str())
        .collect::<Vec<_>>()
        .join(",");
    verdict(kind, detail, found.into_iter().map(|(_, _, l)| l).collect())
}

/// 差の file の行 `rows` の判じ（床の記録の字 `record`・無ければ None）と、重ねない欠けの字の列。
/// `rerun` は記録に行の在る行だけを撃ち直し、撃ち直せない時は見つからない物の字を返す。
pub fn judge(
    rows: &[String],
    record: Option<&str>,
    rerun: impl Fn(&str) -> Result<Rerun, String>,
) -> (Vec<Verdict>, Vec<String>) {
    let Some(text) = record else {
        let lack = format!(
            "床の記録 floor.tsv が出力の dir に無い（差の file の行 {} ごとに 1 行）",
            rows.join("・")
        );
        let all = rows
            .iter()
            .map(|r| Verdict {
                row: r.clone(),
                kind: Kind::NoRecord,
                detail: "-".into(),
                lacks: vec![lack.clone()],
            })
            .collect();
        return (
            all,
            if rows.is_empty() {
                Vec::new()
            } else {
                vec![lack]
            },
        );
    };
    let recs = self::record(text);
    let all: Vec<Verdict> = rows.iter().map(|r| one(r, recs.get(r), &rerun)).collect();
    let mut lacks: Vec<String> = Vec::new();
    for l in all.iter().flat_map(|v| v.lacks.iter()) {
        if !lacks.contains(l) {
            lacks.push(l.clone());
        }
    }
    (all, lacks)
}

/// 門の記録の 1 行（時刻・終わりの回 1 か 2・行 id・型・詳細をタブで区切り、改行で終える）。
pub fn log_line(at: &str, again: bool, v: &Verdict) -> String {
    let round = if again { 2 } else { 1 };
    format!(
        "{at}\t{round}\t{}\t{}\t{}\n",
        v.row,
        v.kind.as_str(),
        v.detail
    )
}
