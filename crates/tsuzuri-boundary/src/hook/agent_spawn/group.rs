//! 起こしの門の群の口（行 ag-gspawn・判断の記録 ADR-61 決定 (1)(2)(4)(5)(7)・要件 FR22）。
//! 席の Agent の呼びの頼みの頭に群の行が在れば、起草の置き場の群の id の dir の計画の file と一覧の file・ほかの群の係の札と群の席の札・
//! 群の係の測りの札（群の合計）・計画の裁定 id の問いの台帳の字を読み、中核の `judge` で判じる。群の行が無ければ、計画の file が
//! 割りに持つ名の起こしだけを断る（5 行目の欠け）。通す時は、同じ群の係の札を ADR-59 の対象の重なりの判じから外した生きた札を返す。
//! 台帳は `bd --readonly show <問い> --json`（cwd は repo・5 秒）で読み、起動できない・rc が 0 でない・JSON でない時は読めないと渡す。

use std::ffi::OsStr;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use tsuzuri_core::agent::meter::{METER, Meter};
use tsuzuri_core::agent::spec::group::judge::{Ask, Ledger, judge, reason, unmarked};
use tsuzuri_core::agent::spec::group::{
    LIST, Line, PLAN, Plan, SEAT, SHARED, Seat, group_line, ledger_of, model, question_of, shared,
    shared_line,
};
use tsuzuri_core::agent::spec::{Call, SPEC, Spec, head};

use super::specs;
use crate::server::events::now;
use crate::server::ledger::{BD, BD_TIMEOUT};
use crate::server::proc;

/// 群の口の判じの結果（ADR-59 の判じに渡す生きた札と、通した群の席の札と割りの読む path の重なり）。
pub struct Gate {
    pub live: Vec<Spec>,
    pub seat: Option<(Seat, Vec<String>)>,
}

/// 置き場の直下の dir の計画の file（dir の名と字の組）。
fn plans(drafts: &Path) -> Vec<(String, String)> {
    let Ok(dir) = fs::read_dir(drafts) else {
        return Vec::new();
    };
    dir.flatten()
        .filter_map(|e| {
            let text = fs::read_to_string(e.path().join(PLAN)).ok()?;
            Some((e.file_name().to_string_lossy().into_owned(), text))
        })
        .collect()
}

/// 置き場の直下の dir の、係の札と群の席の札の両方が読める群の係。
fn peers(drafts: &Path) -> Vec<(Spec, Seat)> {
    let Ok(dir) = fs::read_dir(drafts) else {
        return Vec::new();
    };
    dir.flatten()
        .filter_map(|e| {
            let spec = Spec::parse(&fs::read_to_string(e.path().join(SPEC)).ok()?)?;
            let seat = Seat::parse(&fs::read_to_string(e.path().join(SEAT)).ok()?)?;
            Some((spec, seat))
        })
        .collect()
}

/// 群 `group` の係の測りの札の新しい量と読み直しの和（札の無い係は 0・在って読めない札が 1 つでも在れば None）。
fn totals(drafts: &Path, peers: &[(Spec, Seat)], group: &str) -> Option<(u64, u64)> {
    let mut sum = (0, 0);
    for (spec, _) in peers.iter().filter(|(_, s)| s.group == group) {
        let Ok(text) = fs::read_to_string(drafts.join(&spec.name).join(METER)) else {
            continue;
        };
        let m = Meter::parse(&text)?;
        sum = (sum.0 + m.used, sum.1 + m.cache_read);
    }
    Some(sum)
}

/// 台帳の問い `q` の notes と本文（bd の読み取りの口・起動できないか rc が 0 でなければ読めない）。
fn ledger(repo: &Path, q: &str) -> Ledger {
    let args = ["--readonly", "show", q, "--json"];
    proc::capture(OsStr::new(BD), args, repo, BD_TIMEOUT)
        .map_or(Ledger::Unread, |out| ledger_of(&out))
}

/// 群の口の判じ（断る時は理由の字）。頭が読めないか名の無い呼びは、群の行が在っても ADR-59 の判じに任せる。
pub fn gate(drafts: &Path, repo: &Path, call: &Call, payload: &str) -> Result<Gate, String> {
    let live = specs(drafts);
    let name = call.name.as_deref().unwrap_or_default();
    let Some(line) = group_line(&call.prompt) else {
        return match unmarked(name, &plans(drafts)) {
            Some(r) => Err(reason(&r)),
            None => Ok(Gate { live, seat: None }),
        };
    };
    let (Ok(top), Some(_)) = (head(&call.prompt), call.name.as_deref()) else {
        return Ok(Gate { live, seat: None });
    };
    let peers = peers(drafts);
    let group = Line::parse(line).map(|l| l.group);
    let read = |f: &str| {
        group
            .as_ref()
            .and_then(|g| fs::read_to_string(drafts.join(g).join(f)).ok())
    };
    let (plan, list) = (read(PLAN), read(LIST));
    let parsed = plan.as_deref().and_then(Plan::parse);
    let ruling = parsed.as_ref().and_then(|p| p.ruling.clone());
    let book = ruling.map(|r| ledger(repo, question_of(&r)));
    let model = model(payload);
    let ask = Ask {
        kind: call.kind.as_deref(),
        name,
        line,
        head: &top,
        model: model.as_deref(),
        plan: plan.as_deref(),
        list: list.as_deref(),
        peers: &peers,
        totals: group.as_ref().and_then(|g| totals(drafts, &peers, g)),
        ledger: book.as_ref(),
        now: now(),
    };
    let seat = judge(&ask).map_err(|r| reason(&r))?;
    let mates: Vec<&str> = peers
        .iter()
        .filter(|(_, s)| s.group == seat.group)
        .map(|(p, _)| p.name.as_str())
        .collect();
    let live = live
        .into_iter()
        .filter(|s| !mates.contains(&s.name.as_str()))
        .collect();
    let paths = parsed.map(|p| shared(&p, name)).unwrap_or_default();
    Ok(Gate {
        live,
        seat: Some((seat, paths)),
    })
}

/// 通した群の係の dir に群の席の札を書き、割りの読む path の重なりが在れば群の id の dir に 1 行足す（記帳だけで断らない）。
pub fn place(drafts: &Path, name: &str, seat: &Seat, paths: &[String]) -> std::io::Result<()> {
    fs::write(drafts.join(name).join(SEAT), seat.render())?;
    if paths.is_empty() {
        return Ok(());
    }
    let line = shared_line(name, paths, now());
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(drafts.join(&seat.group).join(SHARED))?;
    writeln!(f, "{line}")
}
