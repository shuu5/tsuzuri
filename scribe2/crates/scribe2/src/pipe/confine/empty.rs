//! 作り手の死んだ空の器の scope を止める（判断の記録 ADR-45 の門 H6・管理 tick の周の片付け）。
//!
//! 作り手（`systemd-run --scope` を撃った process）が scope の起動の途中で殺されると、中の task が 0 のまま active の scope が
//! 残る。`--collect` は終わった unit だけを畳み、止める口の片付け（[`super::reap_orphan_scopes`]）は作り手の死んだ scope に
//! kill と reset-failed を撃つだけで、空の scope の段を替えない。ここは止める口と同じ選び（[`reap_targets`]＝器の名・段の語
//! seat の外し・作り手の生死）に、中の task の数がちょうど 0 の照らしを足し、選んだ unit に `systemctl --user stop --no-block`
//! を 1 回ずつ撃つ。kill は撃たない（中に process の在る scope は選ばない）。値を持たない（閾値も rules の行も無い）。

use super::{alive_creators, list_scopes, reap_targets, SYSTEMCTL};
use crate::invocation::Invocation;
use std::collections::BTreeSet;
use std::io;
use std::process::{Output, Stdio};

/// 中の task の数の property（`systemctl show -p`）。
const TASKS: &str = "TasksCurrent";

/// `systemctl --user show <unit>.scope -p TasksCurrent --value` の結果が空の scope か（pure）: rc 0 で、stdout の前後の空白を
/// 除いた字がちょうど `0`。起動できない周・rc 非 0・数でない字（`[not set]`・空）・1 以上は空でない側（止めない）。
pub fn empty_from(out: io::Result<Output>) -> bool {
    out.is_ok_and(|found| found.status.success() && String::from_utf8_lossy(&found.stdout).trim() == "0")
}

/// 止める相手を選ぶ（`show` は unit 名〔`.scope` なし〕ごとに task の数を引く口）: [`reap_targets`] が残す unit のうち、
/// `show` の結果を [`empty_from`] が空と読む unit だけ（`.scope` を剥がした名・一覧の順）。`show` は [`reap_targets`] が残す
/// unit にだけ撃つ。
pub fn empty_targets(rows: &[String], alive: &BTreeSet<u32>, mut show: impl FnMut(&str) -> io::Result<Output>) -> Vec<String> {
    reap_targets(rows, alive).into_iter().filter(|unit| empty_from(show(unit))).collect()
}

/// unit の中の task の数を 1 回引く（PATH 解決・[`SYSTEMCTL`]）。
fn tasks_of(unit: &str) -> io::Result<Output> {
    Invocation::new(SYSTEMCTL)
        .args(["--user", "show"])
        .arg(format!("{unit}.scope"))
        .args(["-p", TASKS, "--value"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
}

/// unit を止める job を 1 つ積む（待たない・rc 0 の周だけ `true`）。
fn stop(unit: &str) -> bool {
    Invocation::new(SYSTEMCTL)
        .args(["--user", "stop", "--no-block"])
        .arg(format!("{unit}.scope"))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

/// 作り手の死んだ空の器の scope を一覧から選び、1 本ずつ止める。返りは止める job を積めた数で、一覧を測れない周（道具が無い・
/// rc 非 0）は `None`＝何も撃たない（呼び手の管理 tick は返りを捨てる）。
pub fn stop_empty_scopes() -> Option<usize> {
    let rows = list_scopes()?;
    Some(empty_targets(&rows, &alive_creators(&rows), tasks_of).iter().filter(|unit| stop(unit)).count())
}

#[cfg(test)]
mod tests {
    use super::empty_targets;
    use std::collections::BTreeSet;
    use std::os::unix::process::ExitStatusExt;
    use std::process::{ExitStatus, Output};

    /// 選びは、正しい見本（器の名・段 common・作り手の pid 100 が死に・show が rc 0 で字 0）から 1 句ずつ外した unit を返さず、
    /// show は器の名で始まり作り手が死に段の語が seat でない unit にだけ撃つ。
    #[test]
    fn vscpe_picks_only_empty_scopes_of_dead_creators_outside_seats() {
        let name = crate::name::NAME;
        let unit = |tail: &str| format!("{name}-{tail}");
        let good = unit("s2-kill-1-common-2-100-2");
        let cases = [
            (good.clone(), Some((0, "0\n"))),
            (unit("s2-kill-2-common-2-200-2"), Some((0, "0\n"))),
            (unit("tk-seat-0-100-0"), Some((0, "0\n"))),
            (unit("s2-kill-3-common-2-100-3"), Some((0, "2\n"))),
            (unit("s2-kill-4-common-2-100-4"), Some((0, "[not set]\n"))),
            (unit("s2-kill-5-common-2-100-5"), Some((0, "\n"))),
            (unit("s2-kill-6-common-2-100-6"), Some((1 << 8, "0\n"))),
            (unit("s2-kill-7-common-2-100-7"), None),
            ("other-s2-kill-8-common-2-100-8".to_owned(), Some((0, "0\n"))),
            (unit("s2-kill-9-common-2-x-9"), Some((0, "0\n"))),
        ];
        let rows: Vec<String> = cases.iter().map(|(unit, _)| format!("{unit}.scope")).collect();
        let alive: BTreeSet<u32> = [200].into_iter().collect();
        let mut shown = Vec::new();
        let show = |asked: &str| {
            shown.push(asked.to_owned());
            match cases.iter().find(|(unit, _)| unit == asked).and_then(|(_, answer)| *answer) {
                Some((raw, stdout)) => Ok(Output { status: ExitStatus::from_raw(raw), stdout: stdout.as_bytes().to_vec(), stderr: Vec::new() }),
                None => Err(std::io::Error::new(std::io::ErrorKind::NotFound, "no systemctl")),
            }
        };
        assert_eq!(empty_targets(&rows, &alive, show), vec![good], "正しい見本の 1 本だけ（母集団 {} 行）", rows.len());
        let asked: Vec<String> = [0, 3, 4, 5, 6, 7].iter().filter_map(|at| cases.get(*at).map(|(unit, _)| unit.clone())).collect();
        assert_eq!(shown, asked, "show は器の名で始まり作り手が死に段の語が seat でない unit にだけ");
    }
}
