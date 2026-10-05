//! 表示先の設定の電文（要件 FR16・判断の記録 ADR-15 の決定 (2)(6)）。
//! 読み（GET `PATH`）は tz stage target show --json の 1 行を `StageTargets` として返す。
//! 書き（POST）は `OwnTarget`（project board・自分の project だけ）と `Targets`（account board・一括か project ごと）で、
//! 窓を開く頼み（POST `OPEN_PATH`）は `OpenRequest`。server は設定の file を書かず、tz の口を撃つだけにする。
//! 要求の 3 つは、ほかの鍵を持つ本文を読まず、Option の欄の鍵を省いた本文（空の本文 {} を既定へ戻す頼みに読むこと）も読まない。

use serde::{Deserialize, Serialize};

/// 表示先の読みと project board の書きの口の path（GET が読み・POST が書き）。
pub const PATH: &str = "/api/stage/target";

/// account board の書きの口の path（一括と project ごと）。
pub const ALL_PATH: &str = "/api/stage/targets";

/// 窓を開く button の口の path。
pub const OPEN_PATH: &str = "/api/stage/open";

/// 効く値の出所。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Origin {
    /// 全体の既定。
    Default,
    /// project ごとの上書き。
    Override,
}

/// project に効く端末の名と出所（`listed` は層 A の名に在るか）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Effective {
    pub name: String,
    pub origin: Origin,
    pub listed: bool,
}

/// project の行（効く値が無ければ `effective` は null）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectTarget {
    pub project: String,
    pub effective: Option<Effective>,
}

/// 設定の表 project の 1 行（上書き）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Override {
    pub project: String,
    pub name: String,
}

/// 表示先の設定の読み（GET `PATH` の 200 の本文）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StageTargets {
    /// --repo の project の名。
    pub project: String,
    /// 全体の既定（無ければ null）。
    pub default: Option<String>,
    /// 設定の表 project の全部（名の順）。
    pub overrides: Vec<Override>,
    /// 群の宣言の anchor の project と --repo の project（tz stage target show の行の順）。
    pub projects: Vec<ProjectTarget>,
    /// 層 A の名（host の面の [[device]] の順）。
    pub names: Vec<String>,
}

/// project board の書きの要求（端末の名か、null で上書きを外す）。鍵 to を省いた本文は読まない。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnTarget {
    #[serde(deserialize_with = "Option::deserialize")]
    pub to: Option<String>,
}

/// account board の書きの要求（外の札は all か project）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub enum Targets {
    /// 全体の既定を置き、project ごとの上書きを全部外す。
    All { to: String },
    /// project の上書きを置くか（to が端末の名）外す（to が null）。
    Project {
        project: String,
        #[serde(deserialize_with = "Option::deserialize")]
        to: Option<String>,
    },
}

/// 窓を開く要求（project が null なら --repo の project）。鍵 project を省いた本文は読まない。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OpenRequest {
    #[serde(deserialize_with = "Option::deserialize")]
    pub project: Option<String>,
}
