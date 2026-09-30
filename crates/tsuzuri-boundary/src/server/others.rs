//! ほかの project の問いの読み（起動の引数 --project の置き場ごと・行 e-multi-ask）。
//! 札は置き場の dir の名（`tsuzuri_core::account::project_name`・account board の project の名と同じ読み）で、
//! 台帳は読み取りの bd（`ledger::BD_ARGS`・--bd と同じ program）だけで撃つ。見張りは server の Hub に足し
//! （`Hub::watch_ledger_into`）、印が動いた周と知らせの接続が居る間の読み直しの周だけ読み、open の問いの card が
//! 前と変わった周だけ ledger-changed を送る。口の読みは、印が見張りの最後の読みの前と同じなら bd を撃たず
//! その字を返し、違えば自分で読む（`Source::watched`・行 e-ledger-lazy）。
//! 答えは受けない（札の組の answerable は偽・ほかの repo の台帳へ書かない）。
//! 導出グラフの口 /api/graph も同じ最後の読みの字を、要求のたびに `outside` で読んで g-3 に渡す（行 c-g3-extern）。

use std::ffi::OsStr;
use std::path::PathBuf;
use std::sync::Arc;

use tsuzuri_contract::board::Reading;
use tsuzuri_contract::question::{ProjectQuestions, QuestionCard};
use tsuzuri_core::account::project_name;
use tsuzuri_core::graph::{Outside, build};

use super::events::{Hub, TIMING};
use super::ledger::{Got, Source};

/// ほかの project の札と台帳の読みの組の列（引数の順）。
pub(super) struct Others {
    projects: Vec<(String, Source)>,
}

impl Others {
    /// 置き場ごとに札と見張りの Source を作る（bd は撃たない・置き場が dir でなくても断らない）。
    pub(super) fn new(projects: &[PathBuf], bd: &OsStr) -> Others {
        Others {
            projects: projects
                .iter()
                .map(|dir| {
                    let label = project_name(&dir.to_string_lossy());
                    (label, Source::new(dir, bd).watched())
                })
                .collect(),
        }
    }

    /// 組ごとの見張りの Source（引数の順・account board が同じ置き場の anchor の台帳を分け合う・行 e-ledger-lazy）。
    pub(super) fn sources(&self) -> Vec<Source> {
        self.projects.iter().map(|(_, s)| s.clone()).collect()
    }

    /// 組ごとに台帳の見張りを Hub に足す（最初の印と読みは戻る前に取る）。
    pub(super) fn watch(&self, hub: &Arc<Hub>) {
        for (_, source) in &self.projects {
            let (marks, reads) = (source.clone(), source.clone());
            Hub::watch_ledger_into(
                hub,
                move || marks.mark(),
                move || {
                    reads.read();
                    cards(&reads.got())
                },
                TIMING,
            );
        }
    }

    /// 組ごとの問いの一覧（引数の順・答えを受けない・字が無ければ card は Unknown）。
    pub(super) fn questions(&self) -> Vec<ProjectQuestions> {
        self.projects
            .iter()
            .map(|(label, source)| ProjectQuestions {
                project: label.clone(),
                answerable: false,
                cards: cards(&source.got()),
            })
            .collect()
    }

    /// 組ごとの台帳の読み（引数の順・`Source::got` の字を `build::outside` で読む・
    /// 字が無いか読めなければ None・bd は印が見張りの最後の読みの前と違う時だけ撃つ・行 c-g3-extern）。
    pub(super) fn outside(&self) -> Vec<Option<Outside>> {
        self.projects
            .iter()
            .map(|(_, source)| source.got().text.as_deref().and_then(build::outside))
            .collect()
    }
}

/// 読みの字の open の問いの card の列（字が無ければ Unknown）。
fn cards(got: &Got) -> Reading<Vec<QuestionCard>> {
    got.text
        .as_deref()
        .map_or(Reading::Unknown, |text| tsuzuri_core::question::list(text).cards)
}
