//! 書き込みの検出線の doctor の行の歯（接頭辞 `seat_doctor_write_budget_`・設計 write-budget.md §5・§6）。
//!
//! 置き場は歯ごとの tmp の根の下の state（`<根>/state`）で、host の面（`<state>/host.toml`）に表 `[[write-budget]]` を書き、host の根
//! （`<根>/<NAME>-host/write-budget/<name>/`）に設計 §4 の形の `open` と `days.log` を歯が組んで置く（日付は撃つ前の今から組み、
//! 器の判定の口を使わない）。値は 10^12 byte の単位（1 TB = 1000000000000）。`doctor --state-dir` を撃ち、`write-budget: ` の行を取る。

use super::*;

/// 1 TB（10^12 byte）。
const TB: u64 = 1_000_000_000_000;

/// 1 日の秒数。
const DAY: u64 = 86_400;

/// 埋め込みの 4 行の線の欄（平均の線 / 1 日の線・byte）。
const CAP: &str = "cap=1500000000000/3000000000000";

/// 行の頭。
const HEAD: &str = "write-budget: ";

/// 歯ごとの置き場（tmp の根は drop で消える）。
struct Wb {
    root: TmpDir,
    state: PathBuf,
    /// 撃つ前の今日の 0 時（UNIX 秒）。
    today0: u64,
}

/// 表の行 `names` を持つ host の面の字（`names` が空なら表の無い面）。
fn wb_host(root: &Path, names: &[&str]) -> String {
    let mut host = "schema = 1\n".to_owned();
    for name in names {
        host.push_str(&format!("\n[[write-budget]]\nname = \"{name}\"\nstat = \"{}\"\n", root.join(format!("stat-{name}")).display()));
    }
    host
}

/// 表の行 `names` の置き場を作る。
#[expect(
    clippy::expect_used,
    reason = "統合 test の helper。clippy の allow-expect-in-tests は #[test] 関数の中だけに効く"
)]
fn wb_place(names: &[&str]) -> Wb {
    let root = make_tmp_dir().expect("tmp の根を作れる");
    let state = root.join("state");
    fs::create_dir_all(&state).expect("置き場を作れる");
    fs::write(state.join(vessel::rules::HOST_MANIFEST), wb_host(&root, names)).expect("host の面を書ける");
    let now = unix_now();
    Wb { root, state, today0: now - now % DAY }
}

/// 記録の dir（契約の字面から組む）。
fn wb_dir(wb: &Wb, name: &str) -> PathBuf {
    wb.root.join(format!("{NAME}-host")).join("write-budget").join(name)
}

/// UNIX 秒の UTC の日付（`YYYY-MM-DD`）。
fn wb_date(secs: u64) -> String {
    vessel::fleet::cli::format_utc(secs).chars().take(10).collect()
}

/// `days.log` を書く（項は「何日前・written の byte（`None` は `-`）・state」）。
fn wb_days(wb: &Wb, name: &str, rows: &[(u64, Option<u64>, &str)]) {
    let text: String = rows
        .iter()
        .map(|(back, written, state)| {
            let word = written.map_or_else(|| "-".to_owned(), |found| found.to_string());
            format!("schema=1 date={} written={word} state={state} reboots=0 tail=-\n", wb_date(wb.today0 - back * DAY))
        })
        .collect();
    fs::create_dir_all(wb_dir(wb, name)).ok();
    fs::write(wb_dir(wb, name).join("days.log"), text).ok();
}

/// `back` 日前から `upto` 日前まで（両端を含む）を `written` の measured で埋めた項。
fn wb_flat(back: u64, upto: u64, written: u64) -> Vec<(u64, Option<u64>, &'static str)> {
    (back..=upto).map(|day| (day, Some(written), "measured")).collect()
}

/// `open` を書く（at の時刻・written・state）。
fn wb_open(wb: &Wb, name: &str, at: u64, written: u64, state: &str) {
    let stat = wb.root.join(format!("stat-{name}"));
    let line = format!(
        "schema=1 stat={} date={} sectors=0 at={at} written={written} state={state} reboots=0 probed={at} probe=ok\n",
        stat.display(),
        wb_date(at)
    );
    fs::create_dir_all(wb_dir(wb, name)).ok();
    fs::write(wb_dir(wb, name).join("open"), line).ok();
}

/// `doctor --state-dir`（`rules` は `--rules` の path）を撃つ。
#[expect(
    clippy::expect_used,
    reason = "統合 test の helper。clippy の allow-expect-in-tests は #[test] 関数の中だけに効く"
)]
fn wb_doctor(wb: &Wb, rules: Option<&Path>) -> Output {
    let mut command = Command::new(bin());
    command.args(["doctor", "--state-dir", &wb.state.display().to_string()]);
    if let Some(found) = rules {
        command.args(["--rules", &found.display().to_string()]);
    }
    command.output().expect("binary を起動できる")
}

/// doctor の `write-budget: ` の行（rc 0 を要求）。
fn wb_lines(wb: &Wb, rules: Option<&Path>) -> Vec<String> {
    let out = wb_doctor(wb, rules);
    assert_eq!(rc_of(&out), i32::from(RC_OK), "doctor は rc 0: {}", stderr_of(&out));
    stdout_of(&out).lines().filter(|line| line.starts_with(HEAD)).map(str::to_owned).collect()
}

/// 線の欄が値の行（today と yesterday の後ろの 5 欄・sampled と probe は記録の無い形）。
fn wb_line(name: &str, head: (&str, &str), rest: &str) -> String {
    format!("{HEAD}name={name} today={} yesterday={} {rest} {CAP} sampled=- probe=-", head.0, head.1)
}

/// (a) 表 2 行。nvme-a は昨日まで 7 日が 1 TB の measured・今日の open が measured の 0.5 TB、nvme-b は記録が無い。drafts-cap の
/// 記録を置き `--repo` を渡さずに撃つと、doctor の最後の 3 行が drafts-cap の行・nvme-a の行・nvme-b の行の順で、表の無い同じ置き場は
/// `write-budget: ` の行を出さない。
#[test]
fn seat_doctor_write_budget_under_the_lines_is_quiet() {
    let wb = wb_place(&["nvme-a", "nvme-b"]);
    wb_days(&wb, "nvme-a", &wb_flat(1, 7, TB));
    wb_open(&wb, "nvme-a", wb.today0, TB / 2, "measured");
    fs::create_dir_all(vessel::seat::seats_root(&wb.state)).ok();
    vessel::seat::write_drafts_cap(&wb.state, None);
    let out = wb_doctor(&wb, None);
    assert_eq!(rc_of(&out), i32::from(RC_OK), "doctor は rc 0: {}", stderr_of(&out));
    let text = stdout_of(&out);
    let lines: Vec<&str> = text.lines().collect();
    let sampled = vessel::fleet::cli::format_utc(wb.today0);
    let want = [
        "drafts-cap=no-rule".to_owned(),
        format!(
            "{HEAD}name=nvme-a today=500000000000 yesterday=1000000000000 avg=1000000000000 days=7/7 over=- streak=0 owner=no {CAP} sampled={sampled} probe=ok"
        ),
        wb_line("nvme-b", ("unmeasured", "unmeasured"), "avg=unmeasured days=0/7 over=- streak=0 owner=no"),
    ];
    assert_eq!(lines[lines.len().saturating_sub(3)..].to_vec(), want.iter().map(String::as_str).collect::<Vec<_>>(), "最後の 3 行: {text}");
    fs::write(wb.state.join(vessel::rules::HOST_MANIFEST), wb_host(&wb.root, &[])).ok();
    assert_eq!(wb_lines(&wb, None), Vec::<String>::new(), "表の無い置き場は 0 行");
}

/// (b) 1 日の線: 昨日 3.5 TB・ほかの 6 日 1 TB は over=day streak=1、7 日とも 1 TB で今日の open が 3.1 TB は over=day streak=0。
#[test]
fn seat_doctor_write_budget_names_the_day_line() {
    let wb = wb_place(&["yday", "today"]);
    let mut rows = wb_flat(2, 7, TB);
    rows.push((1, Some(35 * TB / 10), "measured"));
    wb_days(&wb, "yday", &rows);
    wb_days(&wb, "today", &wb_flat(1, 7, TB));
    wb_open(&wb, "today", wb.today0, 31 * TB / 10, "measured");
    let sampled = vessel::fleet::cli::format_utc(wb.today0);
    assert_eq!(
        wb_lines(&wb, None),
        [
            wb_line("yday", ("unmeasured", "3500000000000"), "avg=1357142857142 days=7/7 over=day streak=1 owner=no"),
            format!(
                "{HEAD}name=today today=3100000000000 yesterday=1000000000000 avg=1000000000000 days=7/7 over=day streak=0 owner=no {CAP} sampled={sampled} probe=ok"
            ),
        ]
    );
}

/// (c) 平均の線: 7 日とも 1.6 TB は over=avg で、2 日前で終わる窓は 6 日の下限で線の下なので streak=1。
#[test]
fn seat_doctor_write_budget_names_the_average_line() {
    let wb = wb_place(&["avg"]);
    wb_days(&wb, "avg", &wb_flat(1, 7, 16 * TB / 10));
    assert_eq!(
        wb_lines(&wb, None),
        [wb_line("avg", ("unmeasured", "1600000000000"), "avg=1600000000000 days=7/7 over=avg streak=1 owner=no")]
    );
}

/// (d) 下限と欠け: 3 日前だけ partial・2 日前の行が無く 5 日前が unmeasured・昨日から 5 日前まで 2.2 TB で 6 日前と 7 日前の行が
/// 無い（下限で越える）・昨日が partial・open の date が昨日（at は昨日の 12 時）の 5 形。
#[test]
fn seat_doctor_write_budget_reads_partial_and_missing_days_as_lower_bounds() {
    let wb = wb_place(&["p3", "gap", "low", "ypart", "stale"]);
    let mut rows = wb_flat(1, 7, TB);
    rows[2] = (3, Some(TB), "partial");
    wb_days(&wb, "p3", &rows);
    let gap: Vec<_> = [1, 3, 4, 6, 7].iter().map(|day| (*day, Some(TB), "measured")).chain([(5, None, "unmeasured")]).collect();
    wb_days(&wb, "gap", &gap);
    wb_days(&wb, "low", &wb_flat(1, 5, 22 * TB / 10));
    let mut rows = wb_flat(2, 7, TB);
    rows.push((1, Some(TB), "partial"));
    wb_days(&wb, "ypart", &rows);
    let noon = wb.today0 - DAY / 2;
    wb_open(&wb, "stale", noon, TB / 2, "measured");
    let sampled = vessel::fleet::cli::format_utc(noon);
    assert_eq!(
        wb_lines(&wb, None),
        [
            wb_line("p3", ("unmeasured", "1000000000000"), "avg=1000000000000:partial days=7/7 over=- streak=0 owner=no"),
            wb_line("gap", ("unmeasured", "1000000000000"), "avg=714285714285:partial days=5/7 over=- streak=0 owner=no"),
            wb_line("low", ("unmeasured", "2200000000000"), "avg=1571428571428:partial days=5/7 over=avg streak=1 owner=no"),
            wb_line("ypart", ("unmeasured", "1000000000000:partial"), "avg=1000000000000:partial days=7/7 over=- streak=0 owner=no"),
            format!(
                "{HEAD}name=stale today=unmeasured yesterday=unmeasured avg=unmeasured days=0/7 over=- streak=0 owner=no {CAP} sampled={sampled} probe=ok"
            ),
        ]
    );
}

/// (e) 続いた越え: 昨日から 3 日前まで 3.5 TB は streak=3 owner=yes、昨日と 2 日前だけ 3.5 TB は streak=2 owner=no、昨日・3 日前・
/// 4 日前が 3.5 TB で 2 日前の行が無い形は streak=1 owner=no。
#[test]
fn seat_doctor_write_budget_counts_the_streak_to_the_owner() {
    let wb = wb_place(&["three", "two", "cut"]);
    let high = 35 * TB / 10;
    let mut three = wb_flat(1, 3, high);
    three.extend(wb_flat(4, 7, TB));
    wb_days(&wb, "three", &three);
    let mut two = wb_flat(1, 2, high);
    two.extend(wb_flat(3, 7, TB));
    wb_days(&wb, "two", &two);
    let cut: Vec<_> = [(1, high), (3, high), (4, high), (5, TB), (6, TB), (7, TB)].iter().map(|(day, bytes)| (*day, Some(*bytes), "measured")).collect();
    wb_days(&wb, "cut", &cut);
    assert_eq!(
        wb_lines(&wb, None),
        [
            wb_line("three", ("unmeasured", "3500000000000"), "avg=2071428571428 days=7/7 over=day,avg streak=3 owner=yes"),
            wb_line("two", ("unmeasured", "3500000000000"), "avg=1714285714285 days=7/7 over=day,avg streak=2 owner=no"),
            wb_line("cut", ("unmeasured", "3500000000000"), "avg=1928571428571:partial days=6/7 over=day,avg streak=1 owner=no"),
        ]
    );
}

/// 埋め込み manifest の写しの字を `edit` で替えて置き場の dir に書き、path を返す（替わらない写しは落とす）。
#[expect(
    clippy::expect_used,
    reason = "統合 test の helper。clippy の allow-expect-in-tests は #[test] 関数の中だけに効く"
)]
fn wb_rules(wb: &Wb, file: &str, edit: (&str, &str)) -> PathBuf {
    let embedded = include_str!("../../../../../rules/manifest.toml");
    let copy = embedded.replace(edit.0, edit.1);
    assert_ne!(copy, embedded, "写しの字が替わる: {}", edit.0);
    let path = wb.root.join(file);
    fs::write(&path, copy).expect("rules の写しを書ける");
    path
}

/// stdout から `write-budget: ` の行を除いた字（rc 0 を要求）。
fn wb_rest(wb: &Wb, rules: Option<&Path>) -> String {
    let out = wb_doctor(wb, rules);
    assert_eq!(rc_of(&out), i32::from(RC_OK), "doctor は rc 0: {}", stderr_of(&out));
    stdout_of(&out).lines().filter(|line| !line.starts_with(HEAD)).map(|line| format!("{line}\n")).collect()
}

/// (f) open を dir にした置き場は today=unreadable で sampled と probe が `-`、形でない行を持つ days.log と今日 3.1 TB の open は
/// days.log に依る欄が unreadable で over は今日だけで判じ、owner の行を欠く写しと窓を 0 にした写しは線に依る 6 欄が no-rule。
/// どの周も rc 0 で、`write-budget: ` の行を除いた stdout は表だけを除いた同じ置き場（host-manifest=present のまま）と等しい。
#[test]
fn seat_doctor_write_budget_says_unreadable_and_no_rule_without_changing_rc() {
    let wb = wb_place(&["odir", "bad"]);
    fs::create_dir_all(wb_dir(&wb, "odir").join("open")).ok();
    fs::create_dir_all(wb_dir(&wb, "bad")).ok();
    fs::write(wb_dir(&wb, "bad").join("days.log"), format!("schema=1 date={} written=1 state=measured reboots=0\n", wb_date(wb.today0 - DAY))).ok();
    wb_open(&wb, "bad", wb.today0, 31 * TB / 10, "measured");
    let sampled = vessel::fleet::cli::format_utc(wb.today0);
    let unreadable = format!(
        "{HEAD}name=bad today=3100000000000 yesterday=unreadable avg=unreadable days=unreadable over=day streak=unreadable owner=unreadable {CAP} sampled={sampled} probe=ok"
    );
    let odir = wb_line("odir", ("unreadable", "unmeasured"), "avg=unmeasured days=0/7 over=- streak=0 owner=no");
    assert_eq!(wb_lines(&wb, None), [odir, unreadable]);
    let owner = wb_rules(&wb, "no-owner.toml", ("id = \"host.write_owner_days\"", "id = \"host.write_owner_dayz\""));
    let window = wb_rules(&wb, "zero-window.toml", ("id = \"host.write_avg_days\"\nkind = \"HostWriteAvgDays\"\nvalue = 7", "id = \"host.write_avg_days\"\nkind = \"HostWriteAvgDays\"\nvalue = 0"));
    let no_rule = |name: &str, head: &str, tail: &str| {
        format!("{HEAD}name={name} {head} avg=no-rule days=no-rule over=no-rule streak=no-rule owner=no-rule cap=no-rule {tail}")
    };
    let want = [
        no_rule("odir", "today=unreadable yesterday=unmeasured", "sampled=- probe=-"),
        no_rule("bad", "today=3100000000000 yesterday=unreadable", &format!("sampled={sampled} probe=ok")),
    ];
    assert_eq!(wb_lines(&wb, Some(&owner)), want, "owner の行を欠く写し");
    assert_eq!(wb_lines(&wb, Some(&window)), want, "窓を 0 にした写し");
    let rests = [wb_rest(&wb, None), wb_rest(&wb, Some(&owner)), wb_rest(&wb, Some(&window))];
    fs::write(wb.state.join(vessel::rules::HOST_MANIFEST), wb_host(&wb.root, &[])).ok();
    assert_eq!(rests, [wb_rest(&wb, None), wb_rest(&wb, Some(&owner)), wb_rest(&wb, Some(&window))], "write-budget の行を除いた stdout は表の無い置き場と等しい");
    assert!(rests[0].contains("host-manifest=present"), "表を除いた置き場も host の面は在る: {}", rests[0]);
}
