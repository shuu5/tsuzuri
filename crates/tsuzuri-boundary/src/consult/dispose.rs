//! tz consult dispose <所見 id> --verdict 採る|一部採る|採らない --reason <字> [--keep <path>]… [--drop <path>]… [--into <木の根>]
//! （行 cs-show・判断の記録 ADR-10 決定 (5) と ADR-29 決定 (1)(オ)・(9)）。所見ごとに採否と理由と、添え物ごとの保存する
//! （--keep）・しない（--drop）を宣言する。添え物の全部がちょうど 1 度ずつ keep か drop に在ること（欠け・知らない path・
//! 重なりを断る）。受けの行の無い所見（先に show で受ける）と処分した所見を断る。保存する添え物は写し先の木の根
//! （--into・省けば repo）の `kept_path`（docs/consult/kept/<所見 id>/<path>）へ写す（在れば上書きせずに断り、
//! どれも写さない・席が commit する）。台帳の所見の題の置き場に相談の処分の行を書き、標準出力に「処分を記した」と
//! 写した path（木の根からの相対）の列を出す。窓が閉じていて所見の全部が処分されたら作業場を退かせる（`retire`）。

use std::fs;
use std::path::{Path, PathBuf};

use tsuzuri_contract::consult::{FindingId, Verdict, WindowId, Word, kept_path};
use tsuzuri_core::consult::lines::{Line, Subject, render};

use super::show::{read_finding, topic_of};
use super::{
    COMMON, Ctx, FAIL, Refused, UNKNOWN, append, ctx, findings, flags, ledger, lines_of,
    minute_now, refuse, retired, workspace,
};
use crate::out::emit;

/// tz consult dispose の残りの引数を受けて終了 code を返す。
pub fn run(rest: &[&str]) -> u8 {
    let mut values = COMMON.to_vec();
    values.extend(["--verdict", "--reason", "--into"]);
    let made = flags(rest, &values, &[], &["--keep", "--drop"]).and_then(|f| {
        let [id] = f.pos.as_slice() else {
            return Err((FAIL, "所見の id を 1 つ渡す".to_string()));
        };
        let id = FindingId::parse(id).map_err(|_| (FAIL, format!("{id} は所見の id でない")))?;
        let verdict = f.get("--verdict").and_then(Verdict::from_word).ok_or((
            FAIL,
            "--verdict は 採る・一部採る・採らない のどれか".to_string(),
        ))?;
        let reason = f
            .get("--reason")
            .ok_or((FAIL, "--reason が要る".to_string()))?;
        let c = ctx(&f)?;
        let into = f
            .get("--into")
            .map_or_else(|| c.repo.clone(), PathBuf::from);
        let keep: Vec<String> = f.all("--keep").into_iter().map(str::to_string).collect();
        let drop: Vec<String> = f.all("--drop").into_iter().map(str::to_string).collect();
        let line = Line::Disposal {
            id,
            verdict,
            reason: reason.to_string(),
            keep,
            drop,
            at: minute_now(),
        };
        dispose(&c, &into, line)
    });
    match made {
        Ok(out) => {
            out.iter().for_each(|l| emit(l));
            0
        }
        Err(e) => refuse("dispose", e),
    }
}

/// 添え物の選びの欠け・知らない path・重なり（無ければ空）。
fn choice_gaps(attached: &[String], keep: &[String], drop: &[String]) -> Vec<String> {
    let mut gaps: Vec<String> = attached
        .iter()
        .filter(|a| keep.contains(a) == drop.contains(a))
        .map(|a| format!("選びの無いか重なった添え物 {a}"))
        .collect();
    gaps.extend(
        keep.iter()
            .chain(drop)
            .filter(|p| !attached.contains(p))
            .map(|p| format!("所見に無い添え物 {p}")),
    );
    gaps
}

/// 処分する（出す行の列を返す）。
fn dispose(c: &Ctx, into: &Path, line: Line) -> Result<Vec<String>, Refused> {
    let Line::Disposal { id, keep, drop, .. } = &line else {
        return Err((FAIL, "処分の行でない".to_string()));
    };
    let (path, f) = read_finding(&c.drafts, *id)?;
    let ws = path
        .parent()
        .and_then(Path::parent)
        .ok_or((FAIL, "作業場".to_string()))?;
    let attached: Vec<String> = f.attachments.iter().map(|a| a.path.clone()).collect();
    let gaps = choice_gaps(&attached, keep, drop);
    if !gaps.is_empty() {
        return Err((FAIL, gaps.join("・")));
    }
    let (_, items) = ledger(c)?;
    let mut lines = lines_of(&items);
    let subject = Subject::Finding(*id);
    if !lines
        .iter()
        .any(|l| matches!(l, Line::Receipt { subject: s, .. } if *s == subject))
    {
        return Err((
            FAIL,
            format!("所見 {id} はまだ受けていない（先に show で受ける）"),
        ));
    }
    if lines
        .iter()
        .any(|l| matches!(l, Line::Disposal { id: x, .. } if x == id))
    {
        return Err((FAIL, format!("所見 {id} はもう処分した")));
    }
    render(&line).map_err(|e| (FAIL, format!("処分の行を書けない: {e:?}")))?;
    let copies = copy_kept(ws, into, *id, keep)?;
    append(c, &items, topic_of(&f).as_deref(), &line)?;
    lines.push(line.clone());
    let mut out = vec!["処分を記した".to_string()];
    out.extend(copies);
    if let Some(gone) = retire(c, id.window(), &lines)? {
        out.push(format!("退いた: {}", gone.display()));
    }
    Ok(out)
}

/// 保存する添え物を写し先の木の根へ写す（写し先が 1 つでも在ればどれも写さずに断る・木の根からの相対の path の列）。
fn copy_kept(
    ws: &Path,
    into: &Path,
    id: FindingId,
    keep: &[String],
) -> Result<Vec<String>, Refused> {
    if !into.is_dir() {
        return Err((
            FAIL,
            format!("写し先の木の根 {} が dir でない", into.display()),
        ));
    }
    let rels: Vec<String> = keep.iter().map(|p| kept_path(id, p)).collect();
    if let Some(there) = rels.iter().find(|r| into.join(r).exists()) {
        return Err((FAIL, format!("写し先 {there} がもう在る（上書きしない）")));
    }
    for (p, rel) in keep.iter().zip(&rels) {
        let to = into.join(rel);
        if let Some(dir) = to.parent() {
            fs::create_dir_all(dir)
                .map_err(|e| (UNKNOWN, format!("{rel} の dir を作れない: {e}")))?;
        }
        fs::copy(ws.join(p), &to).map_err(|e| (UNKNOWN, format!("{p} を写せない: {e}")))?;
    }
    Ok(rels)
}

/// 窓が閉じていて作業場の所見の全部が処分されていれば、作業場を退いた作業場の名へ rename する（消さない・
/// 退かせた path を返す・作業場が無いか条件に当たらなければ None）。
pub fn retire(c: &Ctx, window: WindowId, lines: &[Line]) -> Result<Option<PathBuf>, Refused> {
    let ws = workspace(&c.drafts, window);
    if !ws.is_dir() {
        return Ok(None);
    }
    let closed = lines
        .iter()
        .any(|l| matches!(l, Line::Close { window: w, .. } if *w == window));
    let disposed = |id: &FindingId| {
        lines
            .iter()
            .any(|l| matches!(l, Line::Disposal { id: x, .. } if x == id))
    };
    if !closed || !findings(&ws, window).iter().all(disposed) {
        return Ok(None);
    }
    let to = retired(&c.drafts, window);
    if to.exists() {
        return Err((FAIL, format!("退きの置き場 {} がもう在る", to.display())));
    }
    fs::rename(&ws, &to).map_err(|e| (UNKNOWN, format!("作業場を退かせられない: {e}")))?;
    Ok(Some(to))
}
