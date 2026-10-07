// flip-check: moved t3-hub.92.10.5
//! 起こす面（`start`・`launched`・`spawn_self`・`launch_log`・`myself`）。親の `dispatch.rs` から純移動した。

use super::{Input, Launch, MARK, PIPE, SPAWN};
use crate::fleet::store;
use crate::fleet::{Event, EventKind, Mark, SCHEMA};
use crate::invocation::Invocation;
use std::path::Path;
use std::process::Stdio;

/// 子の stderr を append する診断 file の置き場（`<state_dir>/pipe/launch.log`・設計 §17・機械は読まない）。
const LAUNCH_LOG: [&str; 2] = ["pipe", "launch.log"];

/// 起こす。
/// 書けない周は起こさない。
/// 出所: 設計 §3・§5 §5 §17
pub(super) fn start(input: &Input<'_>, launch: &Launch) -> Result<(), &'static str> {
    if !launched(input, launch) {
        return Err(MARK);
    }
    if !spawn_self(input.state_dir, &launch.argv) {
        return Err(SPAWN);
    }
    Ok(())
}

/// 起こした事実の印を 1 件記帳する（`DispatchMark` mark = `launched`・detail = 起こした argv の subcommand 1 語）。
///
/// 追記は fleet の 1 本（[`store::append`]・C6.3）で、lock の待ち方は列と同じ manifest から読む。読めない周も
/// 書けない周と同じ `false`（記帳できない起動を数えない）。
fn launched(input: &Input<'_>, launch: &Launch) -> bool {
    let Ok(policy) = store::LockPolicy::from_rules(input.manifest) else {
        return false;
    };
    let event = Event {
        schema: SCHEMA,
        ts: crate::fleet::cli::now_utc(),
        kind: EventKind::DispatchMark,
        run: String::new(),
        bead: launch.bead.clone(),
        host: crate::fleet::cli::host(),
        actor: EventKind::DispatchMark.default_actor().to_owned(),
        stage: None,
        seat: None,
        pid: None,
        detail: launch.argv.first().cloned(),
        allowance: None,
        registration: None,
        mark: Some(Mark::Launched),
        account: None,
        cost: None,
        rule: None,
        case: None,
    };
    store::append(input.state_dir, &event, policy).is_ok()
}

/// 自分自身を `pipe <argv>` で起こす（**起こす側と起こし直す側の 1 実装**・C2）。
///
/// 子の stderr は `<state_dir>/pipe/launch.log` に append する（設計 §17・受付で落ちた子の死因を席が読める
/// 場所に残す・C10）。file を開けない周は stderr を捨てて**起こす**（起動を記録の失敗で止めない）。
/// land の着地後の検出（設計 gate-cost.md §44 形 (11)）も同じ 1 本で起こす。argv に `--run` を持つ子（起こし直しと着地後の
/// 検出）は便の留めを撃つ（[`super::pin::program`]・行 v-pin）。
pub(in crate::pipe) fn spawn_self(state_dir: &Path, argv: &[String]) -> bool {
    let program = super::pin::run_of(argv).map_or_else(myself, |run| super::pin::program(state_dir, run));
    Invocation::new(program)
        .arg(PIPE)
        .args(argv)
        .process_group(0)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(launch_log(state_dir))
        .spawn()
        .is_ok()
}

/// 子の stderr の行き先（`launch.log` を append で開く・開けない周は [`Stdio::null`]）。
fn launch_log(state_dir: &Path) -> Stdio {
    let path = LAUNCH_LOG.iter().fold(state_dir.to_path_buf(), |dir, part| dir.join(part));
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    match std::fs::OpenOptions::new().create(true).append(true).open(&path) {
        Ok(file) => Stdio::from(file),
        Err(_) => Stdio::null(),
    }
}

/// 自分の binary（`argv[0]`）。PATH で呼ばれた周は同じ名で子も PATH から解ける。
///
/// **`current_exe` は使わない**——`/proc/self/exe` を読むのは「器は env も HOME も読まない」（C2.2）の
/// 外側で、xtask の門が違反として数える。`argv[0]` は**呼ばれ方そのもの**なので、同じ呼ばれ方で子を起こす。
///
/// 席の hook が鮮度の外の口座を子で測る口（`hook::group`・設計 account-lifecycle.md §19 形 5）も同じ 1 本を読む。
/// 呼ばれ方が便の留めの形の周は器の名に戻す（新しい便と便に結ばれない子は PATH の器で起こす・[`super::pin::launcher`]）。
pub(crate) fn myself() -> String {
    super::pin::launcher(std::env::args().next())
}
