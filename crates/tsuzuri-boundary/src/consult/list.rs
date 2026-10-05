//! tz consult list [--json]（窓の一覧・判断の記録 ADR-29 決定 (5)(9)(10)・hook の代わりの拾い）。
//! 起草の置き場の窓（退いた窓も）ごとに id・形・題・状態・所見の数・未処分の数を、続けて受けの無い頼みと、
//! 席が自分で開いた問う窓の今日の数と今の数（規則の行 R-38）を 1 行ずつ出す。--json は電文 `ConsultBoard`。
//! 状態は、退いた作業場なら 退いた、閉じの行が在れば 閉じた、最後の process が在れば 生きている、ほかは 止まった。
//! 一覧の材料は作業場の file と台帳の相談の行だけ（読むだけ・台帳は書かない・席は show --via 一覧 で受ける）。
//! 時刻は行の分の字を epoch 秒にする（読めない字は None）。
//! 口座は最後の process の印の口座の置き場の末の名で、印が無いか口座の欄が無ければ 分からない（判断の記録 ADR-55 決定 (4)）。

use std::path::Path;

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::consult::{
    ConsultBoard, Finding, FindingRow, RequestRow, SeatQuota, WindowId, WindowRow, WindowState,
    Word,
};
use tsuzuri_contract::wire;
use tsuzuri_core::account::project::account_label;
use tsuzuri_core::consult::lines::{Line, Subject, unreceived};
use tsuzuri_core::consult::quota::{DAY_MAX, LIVE_MAX, count};

use super::{
    COMMON, Ctx, FAIL, ctx, findings, flags, ledger, lines_of, live, minute_now, procs,
    read_window, refuse, retired, windows, workspace,
};
use crate::out::emit;
use crate::server::clock::epoch_secs;

/// tz consult list の残りの引数を受けて終了 code を返す。
pub fn run(rest: &[&str]) -> u8 {
    let made = flags(rest, &COMMON, &["--json"], &[]).and_then(|f| {
        if !f.pos.is_empty() {
            return Err((FAIL, format!("知らない引数 {}", f.pos.join(" "))));
        }
        let c = ctx(&f)?;
        let (_, items) = ledger(&c)?;
        let b = board(&c, &lines_of(&items), &minute_now());
        if f.has("--json") {
            return wire::encode(&b)
                .map(|t| vec![t])
                .map_err(|e| (FAIL, e.to_string()));
        }
        Ok(text(&b))
    });
    match made {
        Ok(lines) => {
            lines.iter().for_each(|l| emit(l));
            0
        }
        Err(e) => refuse("list", e),
    }
}

/// UTC の分の字（`20261003T1412Z`）の epoch 秒。
pub fn epoch_of(minute: &str) -> Option<EpochSecs> {
    let d = |a: usize, b: usize| minute.get(a..b);
    let rfc = format!(
        "{}-{}-{}T{}:{}:00Z",
        d(0, 4)?,
        d(4, 6)?,
        d(6, 8)?,
        d(9, 11)?,
        d(11, 13)?
    );
    tsuzuri_contract::consult::minute_ok(minute)
        .then(|| epoch_secs(&rfc))
        .flatten()
}

/// 口座の無い窓の口座の字。
pub const NO_ACCOUNT: &str = "分からない";

/// 窓の状態。
pub fn state_of(c: &Ctx, id: WindowId, gone: bool, lines: &[Line]) -> WindowState {
    if gone {
        WindowState::Retired
    } else if lines
        .iter()
        .any(|l| matches!(l, Line::Close { window, .. } if *window == id))
    {
        WindowState::Closed
    } else if live(&workspace(&c.drafts, id)) {
        WindowState::Live
    } else {
        WindowState::Stalled
    }
}

/// 所見が処分されたか。
fn disposed(lines: &[Line], id: tsuzuri_contract::consult::FindingId) -> bool {
    lines
        .iter()
        .any(|l| matches!(l, Line::Disposal { id: x, .. } if *x == id))
}

/// 受けの行の時刻。
fn received(lines: &[Line], subject: &Subject) -> Option<EpochSecs> {
    lines.iter().find_map(|l| match l {
        Line::Receipt { subject: s, at, .. } if s == subject => epoch_of(at),
        _ => None,
    })
}

/// 窓の行と処分の無い所見の行。
fn rows(c: &Ctx, lines: &[Line]) -> (Vec<WindowRow>, Vec<FindingRow>) {
    let (mut wins, mut open) = (Vec::new(), Vec::new());
    for (id, gone) in windows(&c.drafts) {
        let ws = if gone {
            retired(&c.drafts, id)
        } else {
            workspace(&c.drafts, id)
        };
        let Some(w) = read_window(&ws) else { continue };
        let found = findings(&ws, id);
        let left: Vec<_> = found
            .iter()
            .copied()
            .filter(|f| !disposed(lines, *f))
            .collect();
        let opened = lines.iter().rev().find_map(|l| match l {
            Line::Open {
                window,
                opened: true,
                at,
                ..
            } if *window == id => epoch_of(at),
            _ => None,
        });
        wins.push(WindowRow {
            id,
            form: w.form,
            topic: w.topic.clone(),
            state: state_of(c, id, gone, lines),
            opened,
            findings: u32::try_from(found.len()).unwrap_or(u32::MAX),
            undisposed: u32::try_from(left.len()).unwrap_or(u32::MAX),
            account: procs(&ws)
                .pop()
                .and_then(|p| p.account)
                .and_then(|a| account_label(&a)),
        });
        open.extend(left.into_iter().map(|f| finding_row(&ws, f, lines)));
    }
    (wins, open)
}

/// 処分の無い所見の 1 行（所見の file が読めなければ題と届いた時刻は None）。
fn finding_row(ws: &Path, id: tsuzuri_contract::consult::FindingId, lines: &[Line]) -> FindingRow {
    let read: Option<Finding> = std::fs::read_to_string(ws.join(format!("findings/{id}.json")))
        .ok()
        .and_then(|t| wire::decode(&t).ok());
    FindingRow {
        id,
        topic: read.as_ref().map(|f| f.topic.clone()),
        arrived: read.as_ref().and_then(|f| epoch_of(&f.date)),
        received: received(lines, &Subject::Finding(id)),
    }
}

/// 一覧の電文（`now` は今の UTC の分の字）。
pub fn board(c: &Ctx, lines: &[Line], now: &str) -> ConsultBoard {
    let (wins, open) = rows(c, lines);
    let requests = unreceived(lines, &[])
        .requests
        .into_iter()
        .filter_map(|want| {
            lines.iter().find_map(|l| match l {
                Line::Request { id, topic, at, .. } if *id == want => Some(RequestRow {
                    id: id.clone(),
                    topic: topic.clone(),
                    at: epoch_of(at).unwrap_or_default(),
                }),
                _ => None,
            })
        })
        .collect();
    let alive: Vec<WindowId> = wins
        .iter()
        .filter(|w| w.state == WindowState::Live)
        .map(|w| w.id)
        .collect();
    ConsultBoard {
        windows: Reading::Known(wins),
        findings: Reading::Known(open),
        requests: Reading::Known(requests),
        quota: Reading::Known(count(lines, now, &alive)),
    }
}

/// 一覧の人の読む行。
pub fn text(b: &ConsultBoard) -> Vec<String> {
    let mut out = Vec::new();
    if let Reading::Known(wins) = &b.windows {
        for w in wins {
            out.push(format!(
                "窓 {}・形 {}・題 {}・状態 {}・所見 {}・未処分 {}・口座 {}",
                w.id,
                w.form.word(),
                w.topic.as_deref().unwrap_or("題なし"),
                w.state.word(),
                w.findings,
                w.undisposed,
                w.account.as_deref().unwrap_or(NO_ACCOUNT)
            ));
        }
    }
    if let Reading::Known(rqs) = &b.requests {
        for r in rqs {
            let topic = r.topic.as_deref().unwrap_or("題なし");
            out.push(format!("受けの無い頼み {}・題 {topic}", r.id));
        }
    }
    let q = match &b.quota {
        Reading::Known(q) => *q,
        Reading::Unknown => SeatQuota::default(),
    };
    out.push(format!(
        "席の問う窓: 今日 {}・今 {}（規則の行 R-38 の上限 同時 {LIVE_MAX}・1 日 {DAY_MAX}）",
        q.today, q.live
    ));
    out
}
