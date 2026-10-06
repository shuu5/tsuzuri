//! tz consult watch [--max <秒>]（席の背景の見張り・判断の記録 ADR-29 決定 (5)(8)・受入 AC16 の見張りの 1 行）。
//! 席が背景の命令として常に 1 本置く。`TICK` ごとに起草の置き場の窓（退いた窓を除く）を見て、次の事象の最初の 1 つで
//! 中核の `lines::notice` の固定の 1 行を標準出力に出して終わる（待ちの席を起こす）。順は (1) 受けの行の無い所見
//! （経路 見張り）、(2) 受けの行の無い頼み、(3) 所見の無いまま最後の process の無い問う窓（閉じの行の無い物）、
//! (4) 最後の pane の無い話す窓（閉じの行の無い物）、(5) 最後の process の印の口座が見張りの環境の口座と違い、
//! 会話の印が手すきで、付いてこなかった印（`follow::held_rel`）の無い、生きている話す窓（閉じの行の無い物）。
//! 上限（既定 `MAX`・Bash の道具の背景の上限より短く）で上限の 1 行を出して終わる。台帳は始めに読み（読めなければ印を置かずに rc 2）、`Source::mark`（store の印）が
//! 変わった時だけ読み直す（読み直しが落ちれば前の読みで見る）。
//! 生きている印 `<起草の置き場>/consult-watch.alive`（pid と始まりの分）を置き、`TOUCH` ごとに書き直し、終わりで消す。
//! 印の更新時刻が `FRESH` の内なら 2 本目は「相談: 見張りはもう居る」の 1 行を出してすぐ終わる。
//! (3) と (4) は窓ごとに同じ process（最後の process の印の番号と pid）について 1 度だけ出す（判断の記録 ADR-55・
//! 止まった窓は閉じずに開き直しを待つ）。出す時に知らせ済みの印 `<起草の置き場>/consult-watch.told-cw<n>` を置き、印が最後の
//! process の印と同じ窓は飛ばして見張りを続ける。新しい process が起きてまた止まれば、また 1 度出す。

use std::fs;
use std::io::{Read, Write};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

use tsuzuri_contract::consult::{Form, Via, WindowId};
use tsuzuri_core::consult::follow::drifted;
use tsuzuri_core::consult::lines::{Event, Line, notice, unreceived};

use super::follow::{held_rel, stamped_idle};
use super::launch::ACCOUNT_ENV;
use super::plain::plain_file;
use super::{
    COMMON, Ctx, FAIL, Refused, alive, ctx, findings, flags, ledger, lines_of, minute_now,
    odd_lines, procs, read_window, refuse, tzw, windows, workspace,
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

/// 知らせ済みの印の file の名の頭（起草の置き場の直下・窓の名を足す・窓の作業場の外）。
pub const TOLD: &str = "consult-watch.told-";

/// 知らせ済みの印として読む字の上限（byte）。
const TOLD_MAX: u64 = 256;

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
    let mut named = Vec::new();
    let found = loop {
        for line in odd_lines(&c.drafts) {
            if !named.contains(&line) {
                emit_err(&format!("tz consult watch: {line}"));
                named.push(line);
            }
        }
        if let Some(event) = scan(c, &lines) {
            tell(&c.drafts, &event);
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

/// 窓の知らせ済みの印の path。
pub fn told_path(drafts: &Path, id: WindowId) -> PathBuf {
    drafts.join(format!("{TOLD}{id}"))
}

/// 知らせ済みの印の字（最後の process の印の番号と pid）。
pub fn told_text(k: u32, pid: u32) -> String {
    format!("k = {k}・pid = {pid}\n")
}

/// 窓の知らせ済みの印が最後の process の印の番号 `k` と `pid` を持つか（symlink を辿らない＝lstat で普通の file と見て開き、
/// 開いた後の fstat で dev と ino が同じかを照らす・普通の file でない印・読めない印・字の違う印は持たないとする＝知らせが多い側）。
fn told(drafts: &Path, id: WindowId, k: u32, pid: u32) -> bool {
    let path = told_path(drafts, id);
    let Some(before) = fs::symlink_metadata(&path)
        .ok()
        .filter(fs::Metadata::is_file)
    else {
        return false;
    };
    let Ok(file) = fs::File::open(&path) else {
        return false;
    };
    let same = file
        .metadata()
        .is_ok_and(|m| m.dev() == before.dev() && m.ino() == before.ino());
    let mut text = String::new();
    same && file.take(TOLD_MAX).read_to_string(&mut text).is_ok() && text == told_text(k, pid)
}

/// 止まった窓の事象（問う窓の `Stalled`・話す窓の `Gone`）なら、最後の process がまだ無い時だけ知らせ済みの印を置く
/// （在る印は消してから新しく作る＝symlink を辿らない・書けなければ置かずに標準エラーへ 1 行＝次の見張りがまた知らせる）。
fn tell(drafts: &Path, event: &Event) {
    let (Event::Stalled(id) | Event::Gone(id)) = event else {
        return;
    };
    let Some(last) = procs(&workspace(drafts, *id)).pop() else {
        return;
    };
    if super::alive(last.pid) {
        return;
    }
    let path = told_path(drafts, *id);
    let _ = fs::remove_file(&path);
    let put = fs::File::options()
        .write(true)
        .create_new(true)
        .open(&path)
        .and_then(|mut f| f.write_all(told_text(last.k, last.pid).as_bytes()));
    if let Err(e) = put {
        emit_err(&format!(
            "tz consult watch: 窓 {id} の知らせ済みの印を書けない: {e}（次の見張りがまた知らせる）"
        ));
    }
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
            && procs(&ws)
                .last()
                .is_some_and(|p| !super::alive(p.pid) && !told(&c.drafts, *id, p.k, p.pid))
    };
    let stalled = open
        .iter()
        .find(|id| quiet(id, Form::Ask) && findings(&workspace(&c.drafts, **id), **id).is_empty());
    if let Some(id) = stalled {
        return Some(Event::Stalled(*id));
    }
    if let Some(id) = open.iter().find(|id| quiet(id, Form::Talk)) {
        return Some(Event::Gone(*id));
    }
    let now = std::env::var(ACCOUNT_ENV).ok();
    open.iter()
        .find(|id| moved(c, lines, **id, now.as_deref()))
        .map(|id| Event::Drift(*id))
}

/// 生きている話す窓（閉じの行の無い物）の最後の process の印の口座が `now` と違い、会話の印が手すきで、付いてこなかった印が無いか。
fn moved(c: &Ctx, lines: &[Line], id: WindowId, now: Option<&str>) -> bool {
    let ws = workspace(&c.drafts, id);
    let Some(mark) = procs(&ws).pop() else {
        return false;
    };
    !closed(lines, id)
        && read_window(&ws).is_some_and(|w| w.form == Form::Talk)
        && alive(mark.pid)
        && drifted(mark.account.as_deref(), now)
        && stamped_idle(&ws)
        && !plain_file(&ws, &held_rel(mark.k))
}
