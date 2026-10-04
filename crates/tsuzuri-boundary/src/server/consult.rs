//! 相談の窓の board の口の材料（設計ノート surface-wave27b 行 cs-server・判断の記録 ADR-29 決定 (4)(8)(9)）。
//! 一覧の口（GET /api/consult）と未受けの口（GET /api/consult/unreceived）は、repo の git config の鍵
//! `tsuzuri.draftsdir` の起草の置き場の作業場の file と、台帳の見張りの最後の読みの相談の行を読む（読むだけ）。
//! state dir が無い server・鍵が無いか dir でない置き場・読めない台帳は、一覧の 4 つの段と未受けを Unknown にする。
//! 頼みの口（POST /api/consult/request）は台帳を新しい子 process で読み直し、題の置き場（中核の `home_of`・memo か根）に
//! 相談の頼みの行を bdw で 1 行足すだけで、窓を開かず、裁定の配達を撃たず、承認として数えない。
//! 頼みの id の数は、同じ分の字の頼みの行の数の最大 + 1。

use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::consult::{ConsultBoard, ConsultRequest, ConsultUnreceived, RequestId};
use tsuzuri_contract::ledger::{BeadId, LedgerWrite};
use tsuzuri_core::consult::lines::{Line, home_of, render, unreceived};

use crate::consult::list::board as list_board;
use crate::consult::{Ctx, DRAFTS_ARGS, GIT_TIMEOUT, findings, lines_of, windows, workspace};
use crate::server::Config;
use crate::server::ledger::{Source, capture, parse_bd};
use crate::server::ruling::{WRITE_TIMEOUT, Writer, minute};

/// 口の置き場の材料（repo・state dir・台帳の読みと書きの program・置き場を引く git の program）。
#[derive(Debug, Clone)]
pub struct Consult {
    repo: PathBuf,
    state: Option<PathBuf>,
    bd: OsString,
    bdw: OsString,
    git: OsString,
}

impl Consult {
    /// 起動の引数の repo と state dir と bd と bdw の program と、置き場を引く git の program から作る。
    pub fn new(config: &Config, git: &OsStr) -> Consult {
        Consult {
            repo: config.repo.clone(),
            state: config.state_dir.clone(),
            bd: config.bd.clone(),
            bdw: config.bdw.clone(),
            git: git.to_os_string(),
        }
    }

    /// 口の置き場（state dir が無いか、鍵 `tsuzuri.draftsdir` が引けないか、その値が dir でなければ None）。
    /// 鍵は要求ごとに引く（server を起こした後に置いた鍵も効く）。
    pub fn ctx(&self) -> Option<Ctx> {
        let state = self.state.clone()?;
        let mut args: Vec<&OsStr> = vec![OsStr::new("-C"), self.repo.as_os_str()];
        args.extend(DRAFTS_ARGS.iter().map(OsStr::new));
        let out = capture(&self.git, args, &self.repo, GIT_TIMEOUT)?;
        let drafts = PathBuf::from(String::from_utf8(out).ok()?.trim());
        drafts.is_dir().then(|| Ctx {
            repo: self.repo.clone(),
            state,
            drafts,
            bd: self.bd.clone(),
            bdw: self.bdw.clone(),
        })
    }
}

/// 台帳の字の相談の行（読めない字は None）。
fn lines(text: Option<&str>) -> Option<Vec<Line>> {
    match parse_bd(text?) {
        Reading::Known(items) => Some(lines_of(&items)),
        Reading::Unknown => None,
    }
}

/// 4 つの段が全部 Unknown の一覧。
pub fn unknown_board() -> ConsultBoard {
    ConsultBoard {
        windows: Reading::Unknown,
        findings: Reading::Unknown,
        requests: Reading::Unknown,
        quota: Reading::Unknown,
    }
}

/// 一覧の電文（置き場か台帳が読めなければ全部の段が Unknown・ほかは tz consult list と同じ組み）。
pub fn board(c: Option<&Ctx>, text: Option<&str>, now: &str) -> ConsultBoard {
    match (c, lines(text)) {
        (Some(c), Some(lines)) => list_board(c, &lines, now),
        _ => unknown_board(),
    }
}

/// 未受けの所見（退いていない作業場の所見で受けの行の無い物）と頼み（受けの行の無い頼みの行）。
pub fn waiting(c: Option<&Ctx>, text: Option<&str>) -> Reading<ConsultUnreceived> {
    let (Some(c), Some(lines)) = (c, lines(text)) else {
        return Reading::Unknown;
    };
    let found: Vec<_> = windows(&c.drafts)
        .into_iter()
        .filter(|(_, gone)| !gone)
        .flat_map(|(id, _)| findings(&workspace(&c.drafts, id), id))
        .collect();
    Reading::Known(unreceived(&lines, &found))
}

/// 次の頼みの id（分の字 `minute` の頼みの行の数の最大 + 1・分の字が形でなければ None）。
pub fn next_request(lines: &[Line], minute: &str) -> Option<RequestId> {
    let last = lines
        .iter()
        .filter_map(|l| match l {
            Line::Request { id, .. } if id.minute() == minute => Some(id.n()),
            _ => None,
        })
        .max()
        .unwrap_or(0);
    RequestId::new(minute, last.checked_add(1)?)
}

/// 頼みの受付の結果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// 頼みの行を置いた（頼みの id と置いた bead の id）。
    Recorded(RequestId, BeadId),
    /// 台帳が読めない（何も書いていない）。
    LedgerUnknown,
    /// 行の字が書けない（model が空・題に器の引用の形が在るなど・何も書いていない）。
    BadLine,
    /// 根の epic が無い（何も書いていない）。
    NoRoot,
    /// bdw の書きが落ちた（置き場の bead の id）。
    AppendFailed(BeadId),
}

/// 頼みを受ける（台帳を新しい子 process で読み直し、置き場に相談の頼みの行を 1 行足す・配達は撃たない）。
pub fn request(req: &ConsultRequest, ledger: &Source, writer: &Writer, now: EpochSecs) -> Outcome {
    let Some(Reading::Known(items)) = ledger.text_alone().map(|t| parse_bd(&t)) else {
        return Outcome::LedgerUnknown;
    };
    let at = minute(now);
    let Some(id) = next_request(&lines_of(&items), &at) else {
        return Outcome::BadLine;
    };
    let line = Line::Request {
        id: id.clone(),
        topic: req.topic.clone(),
        form: req.form,
        model: req.model.clone(),
        at,
    };
    let Ok(text) = render(&line) else {
        return Outcome::BadLine;
    };
    let Ok(home) = home_of(&items, req.topic.as_deref(), &text) else {
        return Outcome::NoRoot;
    };
    let write = LedgerWrite::AppendNotes {
        id: home.id.clone(),
        line: home.line(&text),
    };
    match capture(&writer.bdw, write.argv(), &writer.repo, WRITE_TIMEOUT) {
        Some(_) => Outcome::Recorded(id, home.id),
        None => Outcome::AppendFailed(home.id),
    }
}
