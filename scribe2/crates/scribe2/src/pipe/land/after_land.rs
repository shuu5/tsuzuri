//! 着地の後の撃ち（宣言の鍵 `after-land`・設計 docs/design/contract-source.md §5・契約表の行 v-after-land）。
//!
//! 宣言が名指す命令の列を、着地のたびに anchor（`--repo` の checkout）で 1 行ずつ撃つ（行は repo が決める・器は撃つだけ）。
//! 起こす側（[`spawn`]・[`super::finish`] の終端の後）は子 process `pipe land --run <id> --after-land` を待たずに起こし、
//! 撃つ側（[`fire`]・子の口）は床の検査（`pipe::dispatch::floor`）と同じ組み——頭の語を PATH で解き・封じ込めの scope に包み・
//! rules 行 `floor.timeout_s` の上限で待つ——で行を順に撃つ。行はどれも `RunDone stage=Landed` の detail で、頭は
//! [`DETAIL_HEAD`]（終端の頭 `terminal:` と検出の頭 `detection:` とは別）。止めない線で、着地と終端の結末と rc は変えない。

use super::super::confine::{release_scope, unit_name, wrap_command, Caps, Limit, Wrap};
use super::super::declaration::{self, Ceiling, CEILING_ROW, DENIED_ROW};
use super::super::dispatch::floor::{resolve, run, Ran, ROW};
use super::super::dispatch::spawn_self;
use super::super::{emit, git_line, Emit};
use super::{AnchorSync, Land, MAIN_REF};
use crate::cli_outcome::{Outcome, RC_OK};
use crate::fleet::{EventKind, Stage};
use crate::invocation::Invocation;
use std::path::Path;
use std::process::Stdio;
use std::time::Duration;

/// 子の口の旗（値なし・`pipe land --run <id> --after-land`・`--detection-only` と同じ「着地をやり直さない口」）。
pub(in crate::pipe) const FLAG: &str = "--after-land";

/// `RunDone stage=Landed` の detail の前置き。
pub(super) const DETAIL_HEAD: &str = "after-land:";

/// 子を起こせた周の語（終えた語ではない＝子が後から行ごとの答えを記す）。
const SPAWNED: &str = "spawned";

/// 子を起こせなかった周の語。
const UNSPAWNED: &str = "unspawned";

/// anchor が揃わなかった周の語（撃たない）。
const ANCHOR_SKIPPED: &str = "anchor-skipped";

/// 子が宣言を読めなかった周の語。
const UNREADABLE: &str = "unreadable";

/// 撃つ行の無い周の stdout の語（行は書かない）。
const NONE: &str = "none";

/// 封じ込めの unit 名に載せる段の語。
const STAGE: &str = "after-land";

/// 着地の終端の後に子を起こす（待たない・返すのは stderr の行）。
///
/// 宣言に行が無い周と宣言を読めない周は何もせず行も書かない。着地した sha が anchor の main と違う周（列の先頭でない便と、着地の後に
/// main が進んだ周）は main の先端の便が撃つので重ねない。anchor が揃わなかった周は `anchor-skipped` の 1 行だけ。それ以外は
/// 着地後の検出と同じ argv の組みで子を起こし、起こせたかを `spawned` か `unspawned` で記す。
pub(super) fn spawn(entry: &Land<'_>, sha: &str, sync: &AnchorSync) -> Vec<String> {
    let Ok(facts) = declaration::terminal_facts(entry.repo) else {
        return Vec::new();
    };
    if facts.after_land.is_empty() || git_line(entry.repo, &["rev-parse", MAIN_REF]).as_deref() != Some(sha) {
        return Vec::new();
    }
    if !matches!(sync, AnchorSync::Synced) {
        return note(entry, ANCHOR_SKIPPED).into_iter().collect();
    }
    let mut argv: Vec<String> = ["land", "--run", entry.run, FLAG, "--state-dir"].into_iter().map(str::to_owned).collect();
    argv.extend([entry.state_dir.display().to_string(), "--repo".to_owned(), entry.repo.display().to_string()]);
    if let Some(rules) = entry.rules {
        argv.extend(["--rules".to_owned(), rules.display().to_string()]);
    }
    let word = if spawn_self(entry.state_dir, &argv) { SPAWNED } else { UNSPAWNED };
    note(entry, word).into_iter().collect()
}

/// 子の口: 宣言の行を着地した sha の anchor で順に撃ち、行ごとの答えを記す（stdout は `run=<id> after-land=<最後の語>`・rc 0）。
///
/// 行は HEAD の宣言を上限と突き合わせて通った列（断られた周は `unreadable`）。行ごとに頭の語を PATH で解き（解けない周は
/// `<n>:unfireable`）、封じ込めの scope に包み（包めない周も同じ）、cwd は anchor・process group は新しく、rules 行 `floor.timeout_s` の
/// 秒を上限に撃つ（行が無い周は撃たず `1:unfireable`・埋め込みの値へ落とさない）。答えは `<n>:rc=<数>` か `<n>:timeout`。rc が 0 でない行と
/// timeout と unfireable の後は続く行を撃たない（後の行は前の行の組みに依る）。
pub(in crate::pipe) fn fire(entry: &Land<'_>, sha: &str) -> Outcome {
    let mut err = Vec::new();
    let mut say = |word: String| {
        err.extend(note(entry, &word));
        word
    };
    let last = fire_lines(entry, sha, &mut say);
    Outcome { out: vec![format!("run={} after-land={last}", entry.run)], err, rc: RC_OK }
}

/// [`fire`] の本体（`say` は答えを 1 件記して語を返す）。最後に記した語（撃つ行が無い周は [`NONE`]）を返す。
fn fire_lines(entry: &Land<'_>, sha: &str, say: &mut impl FnMut(String) -> String) -> String {
    let Ok(manifest) = crate::rules::read(entry.rules, Some(entry.state_dir)) else {
        return say(UNREADABLE.to_owned());
    };
    let (Ok(commands), Ok(denied)) = (crate::rules::list_row(&manifest, CEILING_ROW), crate::rules::list_row(&manifest, DENIED_ROW)) else {
        return say(UNREADABLE.to_owned());
    };
    let ceiling = Ceiling { row: CEILING_ROW, commands, denied, classes: &[] };
    let Ok(lines) = declaration::after_land_at(entry.repo, &ceiling) else {
        return say(UNREADABLE.to_owned());
    };
    if lines.is_empty() {
        return NONE.to_owned();
    }
    let Ok(secs) = crate::rules::int_row(&manifest, ROW) else {
        return say("1:unfireable".to_owned());
    };
    let mut last = String::new();
    for (index, row) in lines.iter().enumerate() {
        let n = index.saturating_add(1);
        let word = match shoot(row, (sha, n), entry.repo, secs) {
            None => format!("{n}:unfireable"),
            Some(Ran::Timeout) => format!("{n}:timeout"),
            Some(Ran::Done { rc, .. }) => format!("{n}:rc={rc}"),
        };
        let go_on = word.ends_with(":rc=0");
        last = say(word);
        if !go_on {
            break;
        }
    }
    last
}

/// 1 行を撃つ（撃てない周は `None`・scope は撃った後に片付ける）。床の検査の `judge` と同じ組みで、cwd だけが anchor である。
fn shoot(row: &str, (sha, n): (&str, usize), repo: &Path, secs: u64) -> Option<Ran> {
    let mut words = row.split_whitespace();
    let program = words.next().and_then(resolve)?;
    let mut cmd = Invocation::new(program);
    cmd.args(words);
    let unit = unit_name(sha, STAGE, n);
    let wrap = Wrap { unit: &unit, limit: Limit::HostReserve, caps: Caps::embedded(), width: None };
    let (mut cmd, confinement) = wrap_command(cmd, &wrap);
    if !confinement.confined() {
        return None;
    }
    cmd.current_dir(repo).process_group(0).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
    let ran = run(&mut cmd, Duration::from_secs(secs));
    let _ = release_scope(&confinement);
    ran
}

/// `RunDone stage=Landed` の detail に `after-land:<語>` を 1 件記す（記帳できない周は stderr の 1 行）。
fn note(entry: &Land<'_>, word: &str) -> Option<String> {
    let emitted = emit(
        entry.state_dir,
        &Emit {
            kind: EventKind::RunDone,
            run: entry.run,
            bead: entry.bead,
            stage: Some(Stage::Landed),
            seat: None,
            pid: None,
            detail: Some(format!("{DETAIL_HEAD}{word}")),
        },
        entry.policy,
    );
    emitted.err().map(|reason| format!("pipe: {reason}"))
}
