//! tz consult watch [--max <秒>]（行 cs-watch・席の背景の見張り・判断の記録 ADR-29 決定 (5)(8)・受入 AC16 の見張りの 1 行）。
//! 席が背景の命令として常に 1 本置く。`TICK` ごとに起草の置き場の窓（退いた窓を除く）を見て、次の事象の最初の 1 つで
//! 中核の `lines::notice` の固定の 1 行を標準出力に出して終わる（待ちの席を起こす）。順は (1) 受けの行の無い所見
//! （経路 見張り）、(2) 受けの行の無い頼み、(3) 所見の無いまま最後の process の無い問う窓（閉じの行の無い物）、
//! (4) 最後の pane の無い話す窓（閉じの行の無い物）。上限（既定 `MAX`・Bash の道具の背景の上限より短く）で
//! 上限の 1 行を出して終わる。台帳は始めに読み（読めなければ印を置かずに rc 2）、`Source::mark`（store の印）が
//! 変わった時だけ読み直す（読み直しが落ちれば前の読みで見る）。
//! 生きている印 `<起草の置き場>/consult-watch.alive`（pid と始まりの分）を置き、`TOUCH` ごとに書き直し、終わりで消す。
//! 印の更新時刻が `FRESH` の内なら 2 本目は「相談: 見張りはもう居る」の 1 行を出してすぐ終わる。

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

use tsuzuri_contract::consult::{Form, Via, WindowId};
use tsuzuri_core::consult::lines::{Event, Line, notice, unreceived};

use super::{
    COMMON, Ctx, FAIL, Refused, ctx, findings, flags, ledger, lines_of, minute_now, procs,
    read_window, refuse, tzw, windows, workspace,
};
use crate::out::{emit, emit_err};
use crate::server::ledger::{Mark, Source};

/// 生きている印の file の名（起草の置き場の直下）。
pub const ALIVE: &str = "consult-watch.alive";

/// 生きている印が新しいと見る間（印は `TOUCH` ごとに書き直す）。
pub const FRESH: Duration = Duration::from_secs(90);

/// 生きている印を書き直す間。
pub const TOUCH: Duration = Duration::from_secs(30);

/// 作業場と台帳の印を見る間。
pub const TICK: Duration = Duration::from_secs(5);

/// 既定の上限（秒・Bash の道具の背景の上限 7200 秒より 2 分短く）。
pub const MAX: u64 = 7080;

/// 2 本目の見張りが出す 1 行。
pub const ALREADY: &str = "相談: 見張りはもう居る";

/// tz consult watch の残りの引数を受けて終了 code を返す。
pub fn run(rest: &[&str]) -> u8 {
    let mut values = COMMON.to_vec();
    values.push("--max");
    let made = flags(rest, &values, &[], &[]).and_then(|f| {
        if !f.pos.is_empty() {
            return Err((FAIL, format!("知らない引数 {}", f.pos.join(" "))));
        }
        let max = match f.get("--max").map(str::parse::<u64>) {
            None => MAX,
            Some(Ok(s)) if s > 0 => s,
            Some(_) => return Err((FAIL, "--max は 1 以上の秒の数".to_string())),
        };
        let c = ctx(&f)?;
        watch(&c, Duration::from_secs(max))
    });
    match made {
        Ok(line) => {
            emit(&line);
            0
        }
        Err(e) => refuse("watch", e),
    }
}

/// 生きている印の path。
pub fn alive_path(drafts: &Path) -> PathBuf {
    drafts.join(ALIVE)
}

/// 生きている印が `FRESH` の内に書かれたか。
pub fn fresh(drafts: &Path) -> bool {
    fs::metadata(alive_path(drafts))
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| SystemTime::now().duration_since(t).ok())
        .is_some_and(|age| age < FRESH)
}

/// 生きている印の字。
fn mark_text(start: &str) -> String {
    format!("pid = {}・始まり = {start}\n", std::process::id())
}

/// 見張る（出す 1 行を返す）。
fn watch(c: &Ctx, max: Duration) -> Result<String, Refused> {
    if fresh(&c.drafts) {
        return Ok(ALREADY.to_string());
    }
    let source = Source::new(&c.repo, &c.bd);
    let (mut seen, mut lines) = (source.mark(), lines_of(&ledger(c)?.1));
    let path = alive_path(&c.drafts);
    let text = mark_text(&minute_now());
    let put =
        || fs::write(&path, &text).map_err(|e| (FAIL, format!("生きている印を書けない: {e}")));
    put()?;
    let (start, mut touched) = (Instant::now(), Instant::now());
    let found = loop {
        if let Some(event) = scan(c, &lines) {
            break notice(&event, &tzw());
        }
        if start.elapsed() >= max {
            break notice(&Event::Timeout, &tzw());
        }
        std::thread::sleep(TICK.min(max.saturating_sub(start.elapsed())));
        if touched.elapsed() >= TOUCH {
            put()?;
            touched = Instant::now();
        }
        let now: Mark = source.mark();
        if now != seen {
            match ledger(c) {
                Ok((_, items)) => lines = lines_of(&items),
                Err((_, why)) => emit_err(&format!("tz consult watch: {why}（前の読みで見る）")),
            }
            seen = now;
        }
    };
    if fs::read_to_string(&path).is_ok_and(|t| t == text) {
        let _ = fs::remove_file(&path);
    }
    Ok(found)
}

/// 窓が閉じの行を持つか。
fn closed(lines: &[Line], id: WindowId) -> bool {
    lines
        .iter()
        .any(|l| matches!(l, Line::Close { window, .. } if *window == id))
}

/// 見張る事象の最初の 1 つ（無ければ None）。
pub fn scan(c: &Ctx, lines: &[Line]) -> Option<Event> {
    let open: Vec<WindowId> = windows(&c.drafts)
        .into_iter()
        .filter(|(_, gone)| !gone)
        .map(|(id, _)| id)
        .collect();
    let found: Vec<_> = open
        .iter()
        .flat_map(|id| findings(&workspace(&c.drafts, *id), *id))
        .collect();
    let waiting = unreceived(lines, &found);
    if let Some(id) = waiting.findings.first() {
        return Some(Event::Finding {
            id: *id,
            via: Via::Watch,
        });
    }
    if let Some(id) = waiting.requests.first() {
        return Some(Event::Request(id.clone()));
    }
    let quiet = |id: &WindowId, form: Form| {
        let ws = workspace(&c.drafts, *id);
        !closed(lines, *id)
            && read_window(&ws).is_some_and(|w| w.form == form)
            && procs(&ws).last().is_some_and(|p| !super::alive(p.pid))
    };
    let stalled = open
        .iter()
        .find(|id| quiet(id, Form::Ask) && findings(&workspace(&c.drafts, **id), **id).is_empty());
    if let Some(id) = stalled {
        return Some(Event::Stalled(*id));
    }
    open.iter()
        .find(|id| quiet(id, Form::Talk))
        .map(|id| Event::Gone(*id))
}
