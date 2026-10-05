//! project の名の電文（header の題が読む）。
//! 名は起動の引数 --repo の dir の名から server が実行の時に取る（tracked な file に名を書かない）。

use serde::{Deserialize, Serialize};

/// project の名の口の path。
pub const PATH: &str = "/api/project";

/// board の project の名（--repo の dir の最後の名）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectName {
    pub name: String,
}
