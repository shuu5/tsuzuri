//! `folio hello`（便 21・docs/design/delivery-21.md §1）。AI のセッションが始まったときに 1 行だけ
//! 「設計文書がまだ無い・最初の文書一式は folio init から」と知らせる（FR3 の型 state・行き先は FR22 の骨格の命令・
//! 便 125・docs/design/delivery-125.md §1 (b)）。出すのは design-intent が
//! 未整備のあいだだけで、止める設定 1 つで切れ、同じプロジェクトには 1 回しか出さない。整備済みなら設計文書の索引の
//! 数と全体像の口 `folio graph --digest` を 1 行で毎回出す（便 96・docs/design/delivery-96.md §1 (d)）。
//! repo には何も書かない——印は `--state` の下だけ（N-1）。正規表現も外部 crate も使わない。

use std::fs;
use std::path::{Path, PathBuf};

use crate::graph;
use crate::sha256;
use crate::verdict::Verdict;

/// 未整備のときに出す 1 行（これだけを標準出力へ書く）。
const GREETING: &str = "folio: この project には設計文書（design-intent）がまだ無い。folio init --dir <置き場> で最初の文書一式（骨格）を書く（止めるには .folio-quiet を置く）";

/// 整備済みだが索引を組めないときに出す 1 行（便 96）。
const NO_INDEX: &str = "folio: 設計文書はあるが索引を組めない（まだ分からない）";

/// 唯一の止める設定（プロジェクトの根の直下・中身は読まない）。
const QUIET: &str = ".folio-quiet";

/// 整備済みの目印（正本の置き場の直下）。
const CONSTITUTION: &str = "constitution.yaml";

/// 出した印の置き場（`--state` の下）。
const GREETED: &str = "greeted";

/// 1 回の結果。標準出力は出す 1 行、標準エラーは印を書けなかった理由。
pub struct Outcome {
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub verdict: Verdict,
}

/// 何も出さずに終わる（合格 0）。
fn silent() -> Outcome {
    Outcome {
        stdout: None,
        stderr: None,
        verdict: Verdict::Pass,
    }
}

/// 印の置き場の既定: 環境変数 XDG_STATE_HOME の下の folio・無ければ HOME の下の .local/state/folio。
/// どちらも無ければ置き場が決まらない（印を書けない）。
fn default_state() -> Option<PathBuf> {
    if let Some(xdg) = std::env::var_os("XDG_STATE_HOME").filter(|v| !v.is_empty()) {
        return Some(PathBuf::from(xdg).join("folio"));
    }
    let home = std::env::var_os("HOME").filter(|v| !v.is_empty())?;
    Some(PathBuf::from(home).join(".local/state/folio"))
}

/// プロジェクトの根 = 正本の置き場の親（親が無い＝置き場が根なら置き場そのもの）。絶対 path にする
/// （印の名は根の絶対 path で決まる＝どこから撃っても同じプロジェクトなら同じ名）。
fn project_root(dir: &Path) -> PathBuf {
    let abs = std::path::absolute(dir).unwrap_or_else(|_| dir.to_path_buf());
    match abs.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
        _ => abs,
    }
}

/// 印を作る（dir が無ければ作る・中身は空）。
fn write_mark(mark: &Path) -> std::io::Result<()> {
    if let Some(parent) = mark.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(mark, b"")
}

/// 判定の順に見て、最初に当たったもので終わる。
pub fn run(dir: &Path, state: Option<&Path>) -> Outcome {
    let root = project_root(dir);
    // 1. 止める設定（何より先に効く）
    if root.join(QUIET).is_file() {
        return silent();
    }
    // 2. 整備済み。索引の数を持つ 1 行を毎回出す（印は付けない・何も書かない・便 96）。組めなければ 1 行を出したまま
    //    「まだ分からない」で終わる（黙って 0 を返して設計文書が無いと読ませない・P-4.1）。
    if dir.is_dir() && dir.join(CONSTITUTION).is_file() {
        return match graph::counts(dir) {
            Ok((nodes, edges)) => Outcome {
                stdout: Some(format!(
                    "folio: 設計文書 {nodes} 節点・{edges} 辺。全体像は folio graph --digest"
                )),
                stderr: None,
                verdict: Verdict::Pass,
            },
            Err(why) => Outcome {
                stdout: Some(NO_INDEX.to_string()),
                stderr: Some(format!("folio hello: まだ分からない: 索引を組めない: {why}")),
                verdict: Verdict::Unknown,
            },
        };
    }
    // 3. 出した印（プロジェクト 1 つにつき 1 回）
    let state = state.map(Path::to_path_buf).or_else(default_state);
    let mark = state.map(|s| {
        s.join(GREETED)
            .join(sha256::hex(root.as_os_str().as_encoded_bytes()))
    });
    if let Some(mark) = &mark
        && mark.exists()
    {
        return silent();
    }
    // 4. 未整備。1 行を出し、印を作る。印を書けなくても 1 行は出したまま「まだ分からない」で終わる
    //    （次回も出る＝黙って 1 回きりにしない・P-4.1）。
    let stderr = match &mark {
        Some(mark) => write_mark(mark).err().map(|e| e.to_string()),
        None => Some("印の置き場が分からない（XDG_STATE_HOME も HOME も無い）".to_string()),
    };
    match stderr {
        None => Outcome {
            stdout: Some(GREETING.to_string()),
            stderr: None,
            verdict: Verdict::Pass,
        },
        Some(why) => Outcome {
            stdout: Some(GREETING.to_string()),
            stderr: Some(format!("folio hello: まだ分からない: 印を書けない: {why}")),
            verdict: Verdict::Unknown,
        },
    }
}
