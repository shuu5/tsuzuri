//! `folio check --emit-amends` と `--freeze-anchor`（便 9・docs/design/delivery-9.md §1 (b)(c)）。
//! day-1 の床 `scripts/check_draft.py` の 2 つの旗を同じ式で写す: 最新 anchor と現行の写しの欄単位の差分を
//! amends にそのまま貼れる形で印字する（読み取り専用）／全検査が 0 違反で測れないも無いときだけ現行の写しを
//! 新しい版の anchor として書き、索引に追記する。検査の式は便 7・便 8（`anchor.rs`・`lineage.rs`）のまま。
//! 書き手は `yaml::write`（外部 crate を足さない）。正規表現は使わない。
//! 旗・列の結果・旗の後始末の型 3 つと凍結しないときの 1 行は便 111 で `phase.rs` へ降ろした（ADR-15・層 1 読む）。
//! 便 121（ADR-16 決定 (2)）: 列の根は憲法の名で列の根の表を引いて照らす（`check_root`）。凍結の木は組む（`build`・`texts`）と
//! 書く（`write_chain`）に分け、始まりの凍結 `--freeze-start` が `ids.rs` の組む・書くと合わせて 2 つを同時に書く。

use std::fs;
use std::path::{Path, PathBuf};

use crate::adr::{self, Adr};
use crate::anchor;
use crate::ids;
use crate::lineage;
use crate::phase::{After, Flag, State, not_frozen_by};
use crate::verdict::{Report, Verdict};
use crate::yaml::{self, Value};

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
        (Flag::FreezeStart, ..) => freeze_start(dir, state, ids, report),
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

/// (2) 列の始め直し（`--freeze-anchor` と `--freeze-start` が共有）。
fn restart(st: &State, report: &mut Report) {
    let first_ver = floor(&["anchor", "first_version"]);
    let cur_ver = st.cur_ver.as_str();
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
}

/// 列の根の凍結の照らし（便 121・ADR-16 決定 (2)(ア)・`--freeze-anchor` と `--freeze-start` が共有）。
/// 組んだ木の digest を、憲法の名で引いた列の根の表の行と比べる。表に無い名は digest の全桁（表に足す値）を出す違反。
fn check_root(flag: &str, st: &State, digest: &str, report: &mut Report) {
    let name = st.name.as_deref();
    let shown = name.unwrap_or("（meta.id が無い）");
    match adr::root_digest(name) {
        None => report.violation(
            "anchor",
            format!(
                "列の根の凍結だが憲法の名 {shown} が列の根の表に無い（床の定数 root_digests に行が無い）＝{flag} は凍結しない。組んだ木の digest {digest}（表に行を足す値・行を足すのは folio2 の便で持ち主の承認の裁定 id を名指す）"
            ),
        ),
        Some(want) if want != digest => report.violation(
            "anchor",
            format!(
                "列の根の凍結だが digest {} が床の定数（列の根の表の {shown} の行）{} と違う（根は 1 度きり・作り直せない。別の中身で列を始めるのは移行＝床の外の手順）",
                &digest[..12],
                &want[..12]
            ),
        ),
        Some(_) => {}
    }
}

/// 組んだ凍結の木（digest の欄まで）。
struct Built {
    tree: Vec<(Value, Value)>,
    digest: String,
    previous: Value,
    n_approvals: usize,
}

/// (4) 凍結する木を組む（`ver_adrs` は現行の版を名指す発効した判断・`newest_doc` は最新 anchor）。
/// 組めなければ「まだ分からない」を積んで None。
fn build(
    dir: &Path,
    st: &State,
    ver_adrs: &[&str],
    newest_doc: Option<(&String, &Value)>,
    report: &mut Report,
) -> Option<Built> {
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
        for id in ver_adrs {
            let file = format!("adr/{id}.yaml");
            let record = anchor::read_typed(&dir.join(&file), &file, report)?;
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
        (s("version"), s(&st.cur_ver)),
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
            return None;
        }
    };
    tree.push((s("digest"), s(&digest)));
    Some(Built {
        tree,
        digest,
        previous,
        n_approvals,
    })
}

/// 凍結 anchor の file 名と path（`constitution-<版>.yaml`）。
fn target(st: &State) -> (String, PathBuf) {
    let name = floor(&["anchor", "file_name"]).replace("<version>", &st.cur_ver);
    let path = st.anchors_dir.join(&name);
    (name, path)
}

/// 凍結 anchor と索引（既存の entries に 1 項を追記）の本文を組む。
fn texts(st: &State, b: &Built) -> Result<(String, String), String> {
    let cur_ver = st.cur_ver.as_str();
    let article_fields = adr::floor_strs(&["anchor", "projection_article_fields"]);
    let statement_fields = adr::floor_strs(&["anchor", "statement_fields"]);
    let mut entries: Vec<Value> = st
        .index
        .as_ref()
        .and_then(|ix| ix.get("entries"))
        .and_then(Value::as_seq)
        .unwrap_or_default()
        .to_vec();
    entries.push(Value::Map(vec![
        (s("version"), s(cur_ver)),
        (s("previous"), b.previous.clone()),
        (s("digest"), s(&b.digest)),
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
    yaml::write(&Value::Map(b.tree.clone()), &anchor_header)
        .and_then(|a| yaml::write(&index, index_header).map(|i| (a, i)))
}

/// 凍結 anchor と索引を書く（anchors/ が無ければ作る）。
fn write_chain(st: &State, path: &Path, anchor_text: String, index_text: String) -> std::io::Result<()> {
    let index_file = floor(&["anchor", "index_file"]);
    fs::create_dir_all(&st.anchors_dir)
        .and_then(|()| fs::write(path, anchor_text))
        .and_then(|()| fs::write(st.anchors_dir.join(index_file), index_text))
}

/// (c) 凍結。
fn freeze(dir: &Path, st: &State, adr: &Adr, report: &mut Report) -> After {
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
    restart(st, report);
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
    let Some(built) = build(dir, st, &ver_adrs, newest_doc, report) else {
        return After::Freeze(not_frozen(report));
    };
    if newest_doc.is_none() && st.newest.is_none() {
        check_root("--freeze-anchor", st, &built.digest, report);
    }
    // (5) 0 違反で測れないも無いときだけ書く
    if report.verdict() != Verdict::Pass {
        return After::Freeze(not_frozen(report));
    }
    let index_file = floor(&["anchor", "index_file"]);
    let (name, path) = target(st);
    if path.exists() || path.is_symlink() {
        report.violation(
            "anchor",
            format!("anchors/{name} が既に在る（同じ版は上書きしない・N-1.1）"),
        );
        return After::Freeze(not_frozen(report));
    }
    let (anchor_text, index_text) = match texts(st, &built) {
        Ok(t) => t,
        Err(e) => {
            report.pending(format!("凍結する木を書けない（{e}）"));
            return After::Freeze(not_frozen(report));
        }
    };
    if let Err(e) = write_chain(st, &path, anchor_text, index_text) {
        report.unknown(format!("anchors/ に書けない: {e}"));
        return After::Freeze(not_frozen(report));
    }
    let n_articles = anchor::value_rows(Some(&st.cur_proj), "articles").count();
    After::Freeze(format!(
        "凍結した: {}（条 {n_articles}・previous {}・承認 {} 件）・索引 {index_file} に追記",
        path.display(),
        built.previous.py_str(),
        built.n_approvals
    ))
}

/// 始まりの凍結 `--freeze-start`（便 121・ADR-16 決定 (2)(イ)・FR24）。憲法の列と id の一覧がどちらも 0 本の置き場で、
/// 最初の版の anchor と索引と id の一覧を既存の 2 つの旗と同じ関数で組み、断りを全部見てから同時に書く。
/// どちらか 1 本でも在れば何も書かずに断る（終了 1）。数えから外すのは 2 つの不在の「まだ分からない」だけ
/// （`anchor.rs` の (i) と `ids.rs` の不在）で、列の根の照らしを含むほかの検査は全部数える。
fn freeze_start(
    dir: &Path,
    state: Option<&State>,
    ids: Option<&ids::Current>,
    report: &mut Report,
) -> After {
    const FLAG: &str = "--freeze-start";
    let chain = state.is_some_and(|st| st.chain_exists);
    let listed = ids.is_some_and(|cur| cur.exists);
    if chain || listed {
        let what = match (chain, listed) {
            (true, true) => "憲法の列と id の一覧",
            (true, false) => "憲法の列",
            _ => "id の一覧",
        };
        return After::Refused(format!(
            "{FLAG}: anchors/ に{what}が既に在る（始まりの凍結は憲法の列と id の一覧がどちらも無い置き場でだけ・何も書かない。在る列は --freeze-anchor と --freeze-ids で続ける）"
        ));
    }
    let (Some(st), Some(cur)) = (state, ids) else {
        return After::Freeze(not_frozen_by(report, FLAG));
    };
    // 列の始め直しと列の根の照らしは --freeze-anchor と同じ（最初の版・previous は空・承認一覧は憲法 meta.approval の写し）
    restart(st, report);
    let Some(built) = build(dir, st, &[], None, report) else {
        return After::Freeze(not_frozen_by(report, FLAG));
    };
    check_root(FLAG, st, &built.digest, report);
    let Some((ids_name, ids_path)) = ids::target(cur, report) else {
        return After::Freeze(not_frozen_by(report, FLAG));
    };
    if report.verdict() != Verdict::Pass {
        return After::Freeze(not_frozen_by(report, FLAG));
    }
    // 書く前に 3 つの path がどれも無いことを確かめる（上書きしない・N-1.1）
    let index_file = floor(&["anchor", "index_file"]);
    let (name, path) = target(st);
    let mut present = false;
    for (file, p) in [
        (name.as_str(), path.clone()),
        (index_file, st.anchors_dir.join(index_file)),
        (ids_name.as_str(), ids_path.clone()),
    ] {
        if p.exists() || p.is_symlink() {
            report.violation(
                "anchor",
                format!("anchors/{file} が既に在る（上書きしない・N-1.1）"),
            );
            present = true;
        }
    }
    if present {
        return After::Freeze(not_frozen_by(report, FLAG));
    }
    let texts = texts(st, &built).and_then(|(a, i)| ids::build(cur).map(|t| (a, i, t)));
    let (anchor_text, index_text, ids_text) = match texts {
        Ok(t) => t,
        Err(e) => {
            report.pending(format!("凍結する木を書けない（{e}）"));
            return After::Freeze(not_frozen_by(report, FLAG));
        }
    };
    // 書けた file は消さない（N-1.1）＝途中で書けなければ「まだ分からない」
    let written = write_chain(st, &path, anchor_text, index_text)
        .and_then(|()| ids::write(cur, &ids_path, ids_text));
    if let Err(e) = written {
        report.unknown(format!("anchors/ に書けない: {e}"));
        return After::Freeze(not_frozen_by(report, FLAG));
    }
    let n_articles = anchor::value_rows(Some(&st.cur_proj), "articles").count();
    After::Freeze(format!(
        "始まりの凍結をした: {}・{}（条 {n_articles}・id {} 本）・索引 {index_file} を作った・3 file を版管理に commit する（commit するまで素の床は未追跡の anchor で 1）",
        path.display(),
        ids_path.display(),
        ids::count(cur)
    ))
}
