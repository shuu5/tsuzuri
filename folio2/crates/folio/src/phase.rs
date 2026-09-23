//! 凍結の実行の様態の型と凍結しないときの 1 行の置き場（便 111・docs/design/delivery-111.md §1 (b)・ADR-15・層 1 読む）。
//! `folio check` の旗・列の結果・旗の後始末の型 3 つと凍結しないときの 1 行は `freeze.rs` から字を変えずに降ろした
//! （移した注の中の file 名は移す前の置き場を指す）。読むのは凍結 anchor の列の検査（`anchor.rs`）・id の凍結（`ids.rs`）・
//! 口（`check.rs`）・凍結の書き手（`freeze.rs`）・配信（`site.rs`）・入口。

use std::path::PathBuf;

use crate::verdict::Report;
use crate::yaml::Value;

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

/// 凍結しないときの 1 行（`flag` は旗の綴り）。
pub(crate) fn not_frozen_by(report: &Report, flag: &str) -> String {
    format!(
        "凍結しない — 違反 {} 件・まだ分からない {} 件を直してから {flag}",
        report.violations.len(),
        report.unknowns.len() + report.pendings.len()
    )
}
