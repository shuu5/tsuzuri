//! `folio check --emit-amends` と `--freeze-anchor`（便 9・docs/design/delivery-9.md §1 (b)(c)）。
//! day-1 の床 `scripts/check_draft.py` の 2 つの旗を同じ式で写す: 最新 anchor と現行の写しの欄単位の差分を
//! amends にそのまま貼れる形で印字する（読み取り専用）／全検査が 0 違反で測れないも無いときだけ現行の写しを
//! 新しい版の anchor として書き、索引に追記する。検査の式は便 7・便 8（`anchor.rs`・`lineage.rs`）のまま。
//! 書き手は `yaml::write`（外部 crate を足さない）。正規表現は使わない。

use std::fs;
use std::path::{Path, PathBuf};

use crate::adr::{self, Adr};
use crate::anchor;
use crate::ids;
use crate::lineage;
use crate::verdict::{Report, Verdict};
use crate::yaml::{self, Value};

/// `folio check` の旗（2 つ以上同時は引数の断り）。`FreezeIds` は便 88（`ids.rs`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flag {
    None,
    EmitAmends,
    FreezeAnchor,
    FreezeIds,
}

/// 便 8 までの検査が残した列の結果（`anchor::check_anchor` が返す）。
pub struct State {
    /// 現行の憲法（型付き）
    pub c: Value,
    /// 憲法 schema.amendment_scope
    pub scope: Vec<String>,
    /// 便 7 (e) の現行の写し
    pub cur_proj: Value,
    /// 現行 meta.version の `str(x)`
    pub cur_ver: String,
    /// 憲法 meta.approval（無ければ null）
    pub meta_approval: Value,
    /// 読めた索引
    pub index: Option<Value>,
    /// 索引の末尾の版
    pub newest: Option<String>,
    /// 最新の版の anchor（読めて列に載ったもの）
    pub newest_doc: Option<Value>,
    /// 版管理の HEAD か履歴に anchor が在った
    pub seen_in_git: bool,
    /// 改訂の記録（amended_by か発効した判断の amends）が在る
    pub records_exist: bool,
    /// `<dir>/anchors`
    pub anchors_dir: PathBuf,
}

/// 旗の後始末（標準出力・標準エラーへ書くもの）。
pub enum After {
    /// 旗なし
    Nothing,
    /// `--emit-amends` の標準出力の行
    Emit(Vec<String>),
    /// `--freeze-anchor` の (1)。他の出力をせず終了コード 1
    Refused(String),
    /// `--freeze-anchor` の結果の 1 行（標準エラー）
    Freeze(String),
}

fn floor(path: &[&str]) -> &'static str {
    adr::floor_val(path).unwrap_or_default()
}

fn s(x: &str) -> Value {
    Value::Str(x.to_string())
}

fn str_seq(items: &[impl AsRef<str>]) -> Value {
    Value::Seq(items.iter().map(|x| s(x.as_ref())).collect())
}

/// 床の ver_key: 字面の中の数字の列を数として並べる（「v」を除き「.」で分けた数の列）。
fn ver_key(v: &str) -> Vec<(usize, String)> {
    v.split(|c: char| !c.is_ascii_digit())
        .filter(|t| !t.is_empty())
        .map(|t| {
            let t = t.trim_start_matches('0');
            (t.len(), t.to_string())
        })
        .collect()
}

/// 旗の後始末。便 8 までの全検査の後に呼ぶ（`state` は列の結果・読めずに止まったなら None・`ids` は便 88 の id の一覧）。
pub fn after(
    dir: &Path,
    flag: Flag,
    state: Option<&State>,
    adr: Option<&Adr>,
    ids: Option<&ids::Current>,
    report: &mut Report,
) -> After {
    match (flag, state, adr) {
        (Flag::None, ..) => After::Nothing,
        (Flag::EmitAmends, Some(st), _) => After::Emit(emit_lines(st, report)),
        (Flag::EmitAmends, None, _) => After::Emit(Vec::new()),
        (Flag::FreezeAnchor, Some(st), Some(adr)) => freeze(dir, st, adr, report),
        (Flag::FreezeAnchor, ..) => After::Freeze(not_frozen(report)),
        (Flag::FreezeIds, ..) => match ids {
            Some(cur) => ids::freeze(cur, report),
            None => After::Freeze(not_frozen_by(report, "--freeze-ids")),
        },
    }
}

/// (b) amends にそのまま貼れる差分の行。
fn emit_lines(st: &State, report: &mut Report) -> Vec<String> {
    let (Some(newest), Some(doc)) = (&st.newest, &st.newest_doc) else {
        return vec!["# 比較元の anchor が無い（最初の版か、列が切れている）".to_string()];
    };
    let mut lines = vec![format!(
        "# 比較元 anchor {newest} → 現行（版 {}）の欄単位の差分。amends にそのまま貼る（version は現行 meta.version の値＝上げる版）",
        st.cur_ver
    )];
    let Some(changed) = lineage::diff_with(doc, &st.cur_proj, &st.scope, report) else {
        return lines;
    };
    for (t, ch) in &changed {
        for (k, (p, c)) in ch {
            let mut line = String::from("- {");
            for (i, (key, val)) in [
                ("target", t.as_str()),
                ("field", k.as_str()),
                ("version", st.cur_ver.as_str()),
                ("previous_text", p.as_str()),
                ("new_text", c.as_str()),
            ]
            .iter()
            .enumerate()
            {
                if i > 0 {
                    line.push_str(", ");
                }
                yaml::json_str(key, &mut line);
                line.push_str(": ");
                yaml::json_str(val, &mut line);
            }
            line.push('}');
            lines.push(line);
        }
    }
    lines
}

fn not_frozen(report: &Report) -> String {
    not_frozen_by(report, "--freeze-anchor")
}

/// 凍結しないときの 1 行（`flag` は旗の綴り）。
pub(crate) fn not_frozen_by(report: &Report, flag: &str) -> String {
    format!(
        "凍結しない — 違反 {} 件・まだ分からない {} 件を直してから {flag}",
        report.violations.len(),
        report.unknowns.len() + report.pendings.len()
    )
}

/// (c) 凍結。
fn freeze(dir: &Path, st: &State, adr: &Adr, report: &mut Report) -> After {
    let first_ver = floor(&["anchor", "first_version"]);
    let cur_ver = st.cur_ver.as_str();
    let newest_doc = st.newest.as_ref().zip(st.newest_doc.as_ref());
    // (1) 版が最新 anchor より新しくなければ、他の検査より先に断る
    if let Some((newest, _)) = newest_doc
        && ver_key(newest) >= ver_key(cur_ver)
    {
        return After::Refused(format!(
            "版 {cur_ver} は最新 anchor {newest} より新しくない（同じ版は上書きしない・版を上げてから）"
        ));
    }
    // (2) 列の始め直し
    if st.newest.is_none() && cur_ver != first_ver {
        report.violation(
            "anchor",
            format!(
                "最初の anchor は版 {first_ver}（床の定数）でなければならない＝{cur_ver} で列を始め直すことはできない"
            ),
        );
    }
    if st.newest.is_none() && (st.seen_in_git || st.records_exist) {
        report.violation(
            "anchor",
            "anchor が版管理（HEAD か履歴）か記録の上では存在した（消された）ので、列を始め直す凍結は認めない",
        );
    }
    // (3) 最新 anchor → 現行の差分を、現行の版を名指す発効した判断と 1:1 に消し込む
    let mut ver_adrs: Vec<&str> = adr
        .records
        .iter()
        .filter(|(_, d)| {
            anchor::is_effective(d)
                && anchor::amends_list(d).any(|e| anchor::node_str(e.get("version")) == cur_ver)
        })
        .map(|(id, _)| id.as_str())
        .collect();
    ver_adrs.sort_unstable();
    if let Some((newest, doc)) = newest_doc {
        let changed = lineage::verify_pair(
            doc,
            &st.cur_proj,
            &st.scope,
            cur_ver,
            "現行",
            &st.c,
            adr,
            report,
        );
        if changed.is_some_and(|ch| ch.is_empty()) {
            report.violation(
                "A-2",
                format!(
                    "版を {newest} → {cur_ver} に上げたが条文（{}）に差分が無い（意味の無い版上げは凍結しない）",
                    st.scope.join("・")
                ),
            );
        }
        if ver_adrs.is_empty() {
            report.violation(
                "N-4",
                format!(
                    "版 {cur_ver} を凍結するには、その版を amends に持つ発効した判断の記録（持ち主の承認）が要る"
                ),
            );
        }
    }
    // (4) 凍結する木
    let approval_fields = adr::floor_strs(&["approval", "required"]);
    let pick = |ap: Option<&Value>, id: Value| {
        let mut row = vec![(s("adr"), id)];
        for f in approval_fields {
            let v = ap.and_then(|a| a.get(f)).cloned().unwrap_or(Value::Null);
            row.push((s(f), v));
        }
        Value::Map(row)
    };
    let approvals: Vec<Value> = if newest_doc.is_some() {
        let mut rows = Vec::new();
        for id in &ver_adrs {
            let file = format!("adr/{id}.yaml");
            let Some(record) = anchor::read_typed(&dir.join(&file), &file, report) else {
                return After::Freeze(not_frozen(report));
            };
            rows.push(pick(record.get("approval"), s(id)));
        }
        rows
    } else {
        vec![pick(Some(&st.meta_approval), Value::Null)]
    };
    let article_fields = adr::floor_strs(&["anchor", "projection_article_fields"]);
    let statement_fields = adr::floor_strs(&["anchor", "statement_fields"]);
    let previous = newest_doc.map_or(Value::Null, |(v, _)| s(v));
    let n_approvals = approvals.len();
    let mut tree = vec![
        (s("kind"), s("constitution-anchor")),
        (s("digest_algo"), s(floor(&["anchor", "digest_algo"]))),
        (s("version"), s(cur_ver)),
        (s("previous"), previous.clone()),
        (
            s("projection"),
            Value::Map(vec![
                (s("scope"), str_seq(&st.scope)),
                (s("article_fields"), str_seq(article_fields)),
                (s("statement_fields"), str_seq(statement_fields)),
            ]),
        ),
        (s("meta_approval"), st.meta_approval.clone()),
        (s("approvals"), Value::Seq(approvals)),
        (s("content"), st.cur_proj.clone()),
    ];
    let digest = match anchor::digest_of(&Value::Map(tree.clone())) {
        Ok(d) => d,
        Err(e) => {
            report.pending(format!("凍結する木の digest を計算できない（{e}）"));
            return After::Freeze(not_frozen(report));
        }
    };
    tree.push((s("digest"), s(&digest)));
    let root_digest = floor(&["anchor", "root_digest"]);
    if newest_doc.is_none() && st.newest.is_none() && digest != root_digest {
        report.violation(
            "anchor",
            format!(
                "列の根の凍結だが digest {} が床の定数 {} と違う（根は 1 度きり・作り直せない。別の中身で列を始めるのは移行＝床の外の手順）",
                &digest[..12],
                &root_digest[..12]
            ),
        );
    }
    // (5) 0 違反で測れないも無いときだけ書く
    if report.verdict() != Verdict::Pass {
        return After::Freeze(not_frozen(report));
    }
    let name = floor(&["anchor", "file_name"]).replace("<version>", cur_ver);
    let index_file = floor(&["anchor", "index_file"]);
    let path = st.anchors_dir.join(&name);
    if path.exists() || path.is_symlink() {
        report.violation(
            "anchor",
            format!("anchors/{name} が既に在る（同じ版は上書きしない・N-1.1）"),
        );
        return After::Freeze(not_frozen(report));
    }
    let mut entries: Vec<Value> = st
        .index
        .as_ref()
        .and_then(|ix| ix.get("entries"))
        .and_then(Value::as_seq)
        .unwrap_or_default()
        .to_vec();
    entries.push(Value::Map(vec![
        (s("version"), s(cur_ver)),
        (s("previous"), previous.clone()),
        (s("digest"), s(&digest)),
    ]));
    let index = Value::Map(vec![
        (s("kind"), s("constitution-anchor-index")),
        (s("entries"), Value::Seq(entries)),
    ]);
    let anchor_header = format!(
        "# folio2 憲法 {cur_ver} の凍結 anchor（ADR-2）。写し（範囲 {}・条は {}・規範文は id・{}）+ 発効の承認の写し + この版の承認一覧 + digest。手で直さない・消さない・同じ版は上書きしない（folio check --freeze-anchor が全検査 0 違反のときだけ作る）。",
        st.scope.join("・"),
        article_fields.join("・"),
        statement_fields
            .iter()
            .filter(|f| **f != "id")
            .copied()
            .collect::<Vec<_>>()
            .join("・")
    );
    let index_header = "# folio2 凍結 anchor の索引（追記のみ・ADR-2）。列 = entries の順。手で直さない・消さない・空にしない。";
    let texts = yaml::write(&Value::Map(tree), &anchor_header)
        .and_then(|a| yaml::write(&index, index_header).map(|i| (a, i)));
    let (anchor_text, index_text) = match texts {
        Ok(t) => t,
        Err(e) => {
            report.pending(format!("凍結する木を書けない（{e}）"));
            return After::Freeze(not_frozen(report));
        }
    };
    let written = fs::create_dir_all(&st.anchors_dir)
        .and_then(|()| fs::write(&path, anchor_text))
        .and_then(|()| fs::write(st.anchors_dir.join(index_file), index_text));
    if let Err(e) = written {
        report.unknown(format!("anchors/ に書けない: {e}"));
        return After::Freeze(not_frozen(report));
    }
    let n_articles = anchor::value_rows(Some(&st.cur_proj), "articles").count();
    After::Freeze(format!(
        "凍結した: {}（条 {n_articles}・previous {}・承認 {n_approvals} 件）・索引 {index_file} に追記",
        path.display(),
        previous.py_str()
    ))
}
