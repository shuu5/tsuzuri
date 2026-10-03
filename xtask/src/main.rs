//! xtask: `cargo run -q -p xtask -- <task>` の形で起動する（alias の file は置かない）。
//! check は公開の走査（pub-scan）・workspace の build・歯の全部・clippy（host と面の wasm）・面の組み立て・入れ子の workspace の段を順に撃ち、最初に落ちた段の rc を返す。
//! surface-build は面の crate の dir で trunk を呼び、dist に index.html と wasm の file を出す（便 g-min）。
//! accept は受入 12 条を全画面 × 2 幅 × 2 mode で測り report を書く（行 j-runner・入口は accept の module）。
//! surface-build は dist が揃えば wasm・js・css の file ごとに隣へ gzip の写し（名に .gz）を書く（行 g-gz・gz の module）。
//! pub-scan は追跡される file の字と基準の commit より後の commit に tailnet の住所・名と一覧の語を探す（行 t-pub-scan・pubscan の module）。
//! check の size の段は 1 module の行数・中核の本体の総行数・歯と本体の行数比を規則の行 R-4 の上限と比べる（行 k-size-base・size の module）。
//! check の最後の段は根の直下の入れ子の workspace を数えて 1 行で出し、各々で build・歯・clippy・その workspace の xtask の check を撃つ（行 v-gate・nested の module）。
//! 割りの在る check は歯でない段を表の役だけで撃ち、歯だけを分ける（行 v-ci-split・spread の module）。
//! daily は host の timer が撃つ日に 1 度の全部の撃ちで、写しを origin の main に合わせて check を撃ち、記録と memo を書く（行 v-daily・daily の module）。

mod accept;
mod daily;
mod gz;
mod nested;
mod pubscan;
mod size;
mod spread;

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

/// check の段。順に撃ち、最初に落ちた段で止まる。
const CHECK_STEPS: &[&[&str]] = &[
    &["build", "--workspace"],
    &["nextest", "run", "--workspace"],
    &[
        "clippy",
        "--workspace",
        "--all-targets",
        "--",
        "-D",
        "warnings",
    ],
    // 面の DOM と通信（board と net）は wasm の target のときだけ組み立てるので、host の clippy に載らない分をここで見る。
    &[
        "clippy",
        "-p",
        "tsuzuri-surface",
        "--target",
        WASM_TARGET,
        "--",
        "-D",
        "warnings",
    ],
];

/// 面の crate の組み立て先（rust-toolchain.toml の targets と同じ）。
const WASM_TARGET: &str = "wasm32-unknown-unknown";

/// 面の crate の dir（workspace の root から）。trunk はここの Trunk.toml と index.html を読む。
const SURFACE_DIR: &str = "crates/tsuzuri-surface";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("check") if args.len() == 1 => exit_code(check(&workspace_root())),
        Some("surface-build") if args.len() == 1 => exit_code(surface_build(&workspace_root())),
        Some("pub-scan") if args.len() == 1 => exit_code(pubscan::run(&workspace_root())),
        Some("daily") => exit_code(daily_task(args.get(1..).unwrap_or_default())),
        Some("accept") => exit_code(accept::run(
            args.get(1..).unwrap_or_default(),
            &workspace_root(),
        )),
        _ => {
            emit_err(&format!(
                "usage: cargo run -q -p xtask -- <check|surface-build|pub-scan>\n{}\n{}",
                accept::USAGE,
                daily::USAGE
            ));
            ExitCode::from(2)
        }
    }
}

/// xtask の出力の手（行 k-lint-print）: 標準エラーへ 1 行を書く（字の後に改行）。xtask が書くのはこの関数だけ。
#[expect(
    clippy::print_stderr,
    reason = "xtask の標準エラーをこの 1 関数に閉じるための例外"
)]
fn emit_err(line: &str) {
    eprintln!("{line}");
}

/// 日に 1 度の全部の撃ち（行 v-daily）: 引数を読み、撃ち、記録の 1 行を出して rc を返す（引数を読めなければ rc 2）。
fn daily_task(args: &[String]) -> i32 {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let d = match daily::parse(args, &workspace_root(), &cargo) {
        Ok(d) => d,
        Err(e) => {
            emit_err(&format!("xtask daily: {e}\n{}", daily::USAGE));
            return 2;
        }
    };
    let outcome = daily::run(&d, daily::now());
    emit_err(&format!("xtask daily: {}", outcome.line));
    outcome.rc
}

fn exit_code(rc: i32) -> ExitCode {
    ExitCode::from(u8::try_from(rc).unwrap_or(1))
}

/// workspace の根（xtask の manifest の dir の親・親の無い path なら字 .. を足した path）。
fn workspace_root() -> PathBuf {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .map_or_else(|| manifest.join(".."), Path::to_path_buf)
}

/// 歯の段を分ける変数（CI の matrix が job ごとに `count:K/N` を渡す）。無ければ歯の全部を撃つ。
const PARTITION_ENV: &str = "TSUZURI_CHECK_PARTITION";

/// nextest の `--partition` に渡してよい字は `count:K/N`（K と N は 1 以上の十進で K ≤ N）だけ。
fn partition(s: &str) -> Option<&str> {
    let (k, n) = s.strip_prefix("count:")?.split_once('/')?;
    let decimal = |t: &str| {
        (!t.is_empty() && t.bytes().all(|b| b.is_ascii_digit()))
            .then(|| t.parse::<u64>().ok())
            .flatten()
            .filter(|v| *v >= 1)
    };
    (decimal(k)? <= decimal(n)?).then_some(s)
}

/// 段の引数。頭の語が nextest の段にだけ、partition が在れば `--partition` と字を足す。
fn step_args(step: &[&str], partition: Option<&str>) -> Vec<String> {
    let mut args: Vec<String> = step.iter().map(|a| (*a).to_string()).collect();
    if let Some(p) = partition
        && step.first() == Some(&"nextest")
    {
        args.push("--partition".to_string());
        args.push(p.to_string());
    }
    args
}

/// 公開の走査を撃ち、段を順に撃ち、最初に落ちた段の rc を返す（全部通れば 0）。
/// 走査が落ちれば後の build・歯・clippy・面の組み立てを撃たない。
/// 走査の後、cargo の段の前に大きさの数え（size）を撃ち、上限を越えれば違反を出して rc 1 を返し、後の段を撃たない。
/// 変数 PARTITION_ENV が在れば nextest の段だけを分け、読めない字なら段を撃たずに rc 2 を返す。
/// 面の組み立ての後に入れ子の workspace の段（nested の module）を撃ち、その rc を返す（入れ子の歯も同じ partition で分ける）。
/// partition が在れば、歯でない段（根の clippy 2 つ・面の組み立て・入れ子の build と clippy と xtask の check）は spread の表の役だけが撃つ。
fn check(root: &Path) -> i32 {
    let raw = std::env::var(PARTITION_ENV);
    let part = match &raw {
        Ok(s) => match partition(s) {
            Some(p) => Some(p),
            None => {
                emit_err(&format!(
                    "xtask check: {PARTITION_ENV} は count:K/N（1 ≤ K ≤ N）の字だけ: {s:?}"
                ));
                return 2;
            }
        },
        Err(std::env::VarError::NotPresent) => None,
        Err(e) => {
            emit_err(&format!("xtask check: {PARTITION_ENV} を読めない: {e}"));
            return 2;
        }
    };
    emit_err("xtask check: pub-scan");
    let rc = pubscan::run(root);
    if rc != 0 {
        emit_err(&format!("xtask check: 落ちた段 pub-scan (rc {rc})"));
        return rc;
    }
    emit_err("xtask check: size");
    match size::measure(root) {
        Ok(facts) => emit_err(&format!("xtask check: size {facts}")),
        Err(e) => {
            emit_err(&format!("xtask check: size: {e}"));
            emit_err("xtask check: 落ちた段 size (rc 1)");
            return 1;
        }
    }
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    for step in CHECK_STEPS {
        let rc = cargo_step(root, &cargo, step, part);
        if rc != 0 {
            return rc;
        }
    }
    let rc = match turn(spread::SURFACE, part) {
        Ok(true) => {
            emit_err("xtask check: surface-build");
            surface_build(root)
        }
        Ok(false) => 0,
        Err(rc) => rc,
    };
    if rc != 0 {
        emit_err(&format!("xtask check: 落ちた段 surface-build (rc {rc})"));
        return rc;
    }
    nested::run(root, &cargo, part)
}

/// cargo の段 1 つ: ほかの役が撃つ段は 1 行を出して 0、撃つ段は撃って rc を返す（落ちたら落ちた段の 1 行）。
fn cargo_step(root: &Path, cargo: &str, step: &[&str], part: Option<&str>) -> i32 {
    let args = step_args(step, part);
    match turn(&spread::key("cargo", &args), part) {
        Ok(true) => {}
        Ok(false) => return 0,
        Err(rc) => return rc,
    }
    emit_err(&format!("xtask check: cargo {}", args.join(" ")));
    let status = Command::new(cargo).args(&args).current_dir(root).status();
    let rc = match status {
        Ok(s) if s.success() => return 0,
        Ok(s) => s.code().unwrap_or(1),
        Err(e) => {
            emit_err(&format!("xtask check: cargo を起動できない: {e}"));
            1
        }
    };
    emit_err(&format!(
        "xtask check: 落ちた段 cargo {} (rc {rc})",
        args.join(" ")
    ));
    rc
}

/// 段の振り分けの判じ: 撃つ段は Ok(true)、ほかの役が撃つ段は 1 行を出して Ok(false)、表に無い段か読めない割りは 1 行を出して Err(2)。
fn turn(key: &str, part: Option<&str>) -> Result<bool, i32> {
    match spread::turn(key, part) {
        Ok(spread::Turn::Run) => Ok(true),
        Ok(spread::Turn::Elsewhere(owner, n)) => {
            emit_err(&format!("xtask check: {}", spread::line(key, owner, n)));
            Ok(false)
        }
        Err(e) => {
            emit_err(&format!("xtask check: {e}"));
            Err(2)
        }
    }
}

/// 面の crate の dir で `trunk build` を撃ち（設定は Trunk.toml）、dist に index.html と wasm の file が在るかを見て、
/// 揃っていれば gzip の写しを書く（書けなければ 1）。
fn surface_build(root: &Path) -> i32 {
    let dir = root.join(SURFACE_DIR);
    emit_err(&format!("xtask surface-build: trunk build ({SURFACE_DIR})"));
    let rc = match Command::new("trunk")
        .arg("build")
        .current_dir(&dir)
        .status()
    {
        Ok(s) if s.success() => 0,
        Ok(s) => s.code().unwrap_or(1),
        Err(e) => {
            emit_err(&format!("xtask surface-build: trunk を起動できない: {e}"));
            1
        }
    };
    if rc != 0 {
        return rc;
    }
    let dist = dir.join("dist");
    match dist_missing(&dist) {
        None => match gz::write_copies(&dist) {
            Ok(copies) => {
                emit_err(&format!(
                    "xtask surface-build: gzip の写しを {} 個書いた",
                    copies.len()
                ));
                0
            }
            Err(e) => {
                emit_err(&format!("xtask surface-build: gzip の写し: {e}"));
                1
            }
        },
        Some(what) => {
            emit_err(&format!(
                "xtask surface-build: {} に {what} が無い",
                dist.display()
            ));
            1
        }
    }
}

/// dist に足りない物（index.html か wasm の file）の名。揃っていれば None。
fn dist_missing(dist: &Path) -> Option<&'static str> {
    if !dist.join("index.html").is_file() {
        return Some("index.html");
    }
    let has_wasm = std::fs::read_dir(dist).is_ok_and(|entries| {
        entries
            .flatten()
            .any(|e| e.path().extension().is_some_and(|x| x == "wasm") && e.path().is_file())
    });
    (!has_wasm).then_some("wasm の file")
}

/// 歯が読む file の簡易な読み（依存を serde と serde_json に限るので TOML の crate を使わない）。
#[cfg(test)]
mod read {
    use std::path::Path;

    /// 行の中の `"…"` を順に取り出す（TOML の基本文字列の簡易な読み・escape は扱わない）。
    fn quoted(s: &str) -> Vec<String> {
        s.split('"')
            .skip(1)
            .step_by(2)
            .map(str::to_string)
            .collect()
    }

    /// `key = [ … ]` の配列の文字列（複数行にまたがってよい）。
    pub(super) fn string_array(text: &str, key: &str) -> Option<Vec<String>> {
        let start = text.lines().position(|l| {
            l.split_once('=')
                .is_some_and(|(k, v)| k.trim() == key && v.trim_start().starts_with('['))
        })?;
        let mut body = String::new();
        for line in text.lines().skip(start) {
            body.push_str(line);
            body.push('\n');
            if line.contains(']') {
                let (_, rest) = body.split_once('=')?;
                let inner = rest.split_once('[')?.1.rsplit_once(']')?.0;
                return Some(quoted(inner));
            }
        }
        None
    }

    /// `[section]` の中の key の名の一覧（`key = …` の行の左辺）。
    pub(super) fn section_keys(text: &str, section: &str) -> Vec<String> {
        let header = format!("[{section}]");
        let mut inside = false;
        let mut keys = Vec::new();
        for line in text.lines() {
            let line = line.trim();
            if line.starts_with('[') {
                inside = line == header;
            } else if inside
                && !line.starts_with('#')
                && let Some((k, _)) = line.split_once('=')
            {
                keys.push(k.trim().trim_matches('"').to_string());
            }
        }
        keys
    }

    /// `[section]` の中の `key = "…"` の値。
    pub(super) fn string_value(text: &str, section: &str, key: &str) -> Option<String> {
        let header = format!("[{section}]");
        let mut inside = false;
        for line in text.lines() {
            let line = line.trim();
            if line.starts_with('[') {
                inside = line == header;
            } else if inside
                && let Some((k, v)) = line.split_once('=')
                && k.trim() == key
            {
                return quoted(v).into_iter().next();
            }
        }
        None
    }

    /// 依存の節か（`dependencies`・`dev-dependencies`・`build-dependencies` と、その target ごとの節）。
    fn is_dependency_table(table: &str) -> bool {
        const KINDS: [&str; 3] = ["dependencies", "dev-dependencies", "build-dependencies"];
        KINDS.contains(&table)
            || (table.starts_with("target.")
                && KINDS.iter().any(|k| table.ends_with(&format!(".{k}"))))
    }

    /// manifest の直接依存の名（どの依存の節か・target ごとの節も含めて集める・重複なし・名の順）。
    /// `name.workspace = true` の形は `name`、`[dependencies.name]` の節は `name` と読む。
    pub(super) fn dependency_names(text: &str) -> Vec<String> {
        let mut inside = false;
        let mut names = Vec::new();
        for line in text.lines() {
            let line = line.trim();
            if let Some(table) = line.strip_prefix('[') {
                let table = table.trim_start_matches('[').trim_end_matches(']').trim();
                inside = is_dependency_table(table);
                if !inside
                    && let Some((head, name)) = table.rsplit_once('.')
                    && is_dependency_table(head)
                {
                    names.push(name.trim().trim_matches('"').to_string());
                }
            } else if inside
                && !line.starts_with('#')
                && let Some((k, _)) = line.split_once('=')
            {
                let key = k.trim().trim_matches('"');
                names.push(key.split('.').next().unwrap_or(key).to_string());
            }
        }
        names.sort();
        names.dedup();
        names
    }

    /// workspace の member の package の名（root の Cargo.toml の members から各 member の Cargo.toml を読む）。
    pub(super) fn member_names(root: &Path) -> Vec<String> {
        let manifest = std::fs::read_to_string(root.join("Cargo.toml")).expect("Cargo.toml を読む");
        string_array(&manifest, "members")
            .expect("workspace の members")
            .iter()
            .map(|dir| {
                let m = std::fs::read_to_string(root.join(dir).join("Cargo.toml"))
                    .unwrap_or_else(|e| panic!("{dir}/Cargo.toml を読む: {e}"));
                string_value(&m, "package", "name")
                    .unwrap_or_else(|| panic!("{dir}/Cargo.toml の package の名"))
            })
            .collect()
    }

    /// Cargo.lock の package の (名, workspace の外か)。source の在る package が外。
    pub(super) fn lock_packages(lock: &str) -> Vec<(String, bool)> {
        lock.split("[[package]]")
            .skip(1)
            .map(|block| {
                let mut name = None;
                let mut external = false;
                for line in block.lines() {
                    if let Some((k, v)) = line.split_once('=') {
                        match k.trim() {
                            "name" => name = quoted(v).into_iter().next(),
                            "source" => external = true,
                            _ => {}
                        }
                    }
                }
                (name.expect("package の name"), external)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::read::{
        dependency_names, lock_packages, member_names, section_keys, string_array, string_value,
    };
    use super::{CHECK_STEPS, SURFACE_DIR, partition, step_args, workspace_root};

    /// partition は nextest の count の形だけを通し、通した字はそのまま返す。
    #[test]
    fn cishard_partition_reads_count_only() {
        for ok in ["count:1/8", "count:8/8", "count:1/2", "count:2/2"] {
            assert_eq!(partition(ok), Some(ok), "{ok}");
        }
        for bad in [
            "",
            "count:0/8",
            "count:9/8",
            "count:1/0",
            "hash:1/2",
            "slice:1/2",
            "count:1",
            "count:a/2",
            "count:+1/2",
            "count: 1/2",
            "count:1/ 2",
            "count:1/2 ",
            " count:1/2",
            "COUNT:1/2",
        ] {
            assert_eq!(partition(bad), None, "{bad:?}");
        }
    }

    /// --partition を持つ段は nextest の 1 段だけ。ほかの段と partition が無い時は段の字のまま。
    #[test]
    fn cishard_partition_only_on_nextest() {
        let mut with = 0;
        for step in CHECK_STEPS {
            let plain: Vec<String> = step.iter().map(|a| (*a).to_string()).collect();
            assert_eq!(step_args(step, None), plain, "{step:?}");
            let args = step_args(step, Some("count:3/8"));
            if step.first() == Some(&"nextest") {
                let mut want = plain;
                want.extend(["--partition".to_string(), "count:3/8".to_string()]);
                assert_eq!(args, want);
                with += 1;
            } else {
                assert_eq!(args, plain, "{step:?}");
                assert!(!args.iter().any(|a| a == "--partition"), "{step:?}");
            }
        }
        assert_eq!(with, 1);
    }

    const MEMBERS: [&str; 6] = [
        "tsuzuri-contract",
        "tsuzuri-core",
        "tsuzuri-boundary",
        "tsuzuri-surface",
        "folio",
        "xtask",
    ];

    /// crate ごとの直接依存の名の一覧（member の dir と、依存の全部の節（target ごとの節も）の名・名の順）。
    /// 外の部品は便ごとに足した名だけ: serde（便 a）・serde_json（便 b・中核は便 c）・Leptos の一式（便 g-min・規則の行 R-25）・
    /// miniz_oxide（行 g-gz・xtask だけ・裁定 t3-hub.52.40:20260928T0156Z-1）・
    /// clap と yaml-rust2（行 k-join-folio・持ち込んだ folio だけ・判断の記録 ADR-21 の承認）。
    /// 境界の crate の folio は workspace の member（行 k-tz-entry・folio の lib の入口を tz の口が撃つ）。
    /// xtask の tsuzuri-contract は行 v-daily（日に 1 度の撃ちが memo の状態を台帳の読みの型 BdLine と wire で読む・外の部品は増えない）。
    /// 名を足す便はこの一覧を直す。一覧に無い名が manifest に在れば落ちる。
    const DIRECT_DEPS: [(&str, &[&str]); 6] = [
        ("crates/tsuzuri-contract", &["serde", "serde_json"]),
        (
            "crates/tsuzuri-core",
            &["serde", "serde_json", "tsuzuri-contract"],
        ),
        (
            "crates/tsuzuri-boundary",
            &["folio", "tsuzuri-contract", "tsuzuri-core"],
        ),
        (
            SURFACE_DIR,
            &[
                "leptos",
                "tsuzuri-boundary",
                "tsuzuri-contract",
                "wasm-bindgen-futures",
                "web-sys",
            ],
        ),
        ("folio2/crates/folio", &["clap", "yaml-rust2"]),
        (
            "xtask",
            &["miniz_oxide", "tsuzuri-boundary", "tsuzuri-contract"],
        ),
    ];

    /// 面の crate の直接依存の上限（規則の行 R-25 の値・要件 NFR3）。rules の file の行 R-25 の字と照らす。
    const SURFACE_DIRECT_MAX: usize = 8;

    #[test]
    fn skeleton_workspace_has_six_members() {
        let mut names = member_names(&workspace_root());
        names.sort();
        let mut want = MEMBERS.map(str::to_string).to_vec();
        want.sort();
        assert_eq!(names, want);
    }

    #[test]
    fn skeleton_common_verify_is_xtask_check() {
        let vessel =
            std::fs::read_to_string(workspace_root().join(".vessel.toml")).expect(".vessel.toml");
        assert_eq!(
            string_array(&vessel, "common-verify"),
            Some(vec!["cargo run -q -p xtask -- check".to_string()])
        );
    }

    /// 各 member の manifest の直接依存の名が一覧と同じ（面の crate の外の crate の名は増えない）。
    #[test]
    fn skeleton_direct_deps_match_list() {
        let root = workspace_root();
        let mut dirs: Vec<&str> = DIRECT_DEPS.iter().map(|(dir, _)| *dir).collect();
        dirs.sort_unstable();
        let manifest = std::fs::read_to_string(root.join("Cargo.toml")).expect("Cargo.toml");
        let mut members = string_array(&manifest, "members").expect("workspace の members");
        members.sort();
        assert_eq!(members, dirs, "一覧は workspace の member の全部を持つ");
        for (dir, want) in DIRECT_DEPS {
            let text = std::fs::read_to_string(root.join(dir).join("Cargo.toml"))
                .unwrap_or_else(|e| panic!("{dir}/Cargo.toml を読む: {e}"));
            assert_eq!(dependency_names(&text), want.to_vec(), "{dir} の直接依存");
        }
    }

    /// 面の crate の直接依存は規則の行 R-25 の上限（8 本）以下。
    #[test]
    fn skeleton_surface_direct_deps_within_budget() {
        let (_, surface) = DIRECT_DEPS
            .iter()
            .find(|(dir, _)| *dir == SURFACE_DIR)
            .expect("面の crate の行");
        assert!(
            surface.len() <= SURFACE_DIRECT_MAX,
            "面の crate の直接依存 {} 本が上限 {SURFACE_DIRECT_MAX} 本を越える: {surface:?}",
            surface.len()
        );
    }

    /// 歯の上限の定数が rules の file の行 R-25 の字（直接依存 N 本）と同じ。
    #[test]
    fn skeleton_surface_budget_matches_rule_r25() {
        let rules = std::fs::read_to_string(workspace_root().join("design-intent/rules.yaml"))
            .expect("design-intent/rules.yaml");
        let row = rules
            .lines()
            .find(|l| l.contains("id: R-25,"))
            .expect("rules の file に行 R-25 が在る");
        assert_eq!(rule_direct_max(row), Some(SURFACE_DIRECT_MAX), "{row}");
    }

    /// 行の value の字の「直接依存 N 本」の N（無いか 2 つ以上なら None）。
    fn rule_direct_max(row: &str) -> Option<usize> {
        let value = row.split_once("value: \"")?.1.split_once('"')?.0;
        let mut found = value.split("直接依存").skip(1).filter_map(|rest| {
            let n = rest.trim_start().split_once('本')?.0.trim();
            n.parse::<usize>().ok()
        });
        let n = found.next()?;
        found.next().is_none().then_some(n)
    }

    #[test]
    fn skeleton_rule_direct_max_reads_value() {
        let row = r#"  - {id: R-25, what: "x", value: "直接依存 8 本 以下（面の crate）・以後は 1 便 1 本", note: "直接依存 3 本"}"#;
        assert_eq!(rule_direct_max(row), Some(8));
        assert_eq!(rule_direct_max(&row.replace("8 本", "9 本")), Some(9));
        assert_eq!(rule_direct_max(&row.replace("8 本", "八本")), None);
        assert_eq!(
            rule_direct_max(&row.replace("8 本 以下", "8 本 以下・直接依存 7 本")),
            None
        );
    }

    #[test]
    fn skeleton_dependency_names_reads_every_table() {
        let text = r#"
[package]
name = "x"

[dependencies]
a = { path = "../a" }
b.workspace = true
# c = "1"

[target.'cfg(target_arch = "wasm32")'.dependencies]
d = { version = "0.3", features = ["E"] }

[dev-dependencies]
a = { path = "../a" }

[build-dependencies.e]
version = "1"

[[bin]]
name = "y"
"#;
        assert_eq!(dependency_names(text), vec!["a", "b", "d", "e"]);
    }

    /// Cargo.lock の workspace の中の package は member だけ。
    #[test]
    fn skeleton_lock_inside_is_members() {
        let lock =
            std::fs::read_to_string(workspace_root().join("Cargo.lock")).expect("Cargo.lock");
        let packages = lock_packages(&lock);
        let mut inside: Vec<&str> = packages
            .iter()
            .filter(|(_, ext)| !*ext)
            .map(|(n, _)| n.as_str())
            .collect();
        inside.sort_unstable();
        let mut want = MEMBERS.to_vec();
        want.sort_unstable();
        assert_eq!(inside, want);
    }

    /// 境界の crate（最小の server と tz の入口・便 e-min）は外の依存を足さない（外の部品の名は無い）。
    /// 直接依存は workspace の member だけ（folio の crate・契約の型の crate・中核の crate）で、binary の名は tz。
    #[test]
    fn skeleton_boundary_std_only_and_bin_tz() {
        let manifest =
            std::fs::read_to_string(workspace_root().join("crates/tsuzuri-boundary/Cargo.toml"))
                .expect("境界の crate の Cargo.toml");
        for section in ["dependencies", "dev-dependencies", "build-dependencies"] {
            let stray: Vec<String> = section_keys(&manifest, section)
                .into_iter()
                .filter(|k| !MEMBERS.contains(&k.as_str()))
                .collect();
            assert!(stray.is_empty(), "[{section}] に外の依存: {stray:?}");
        }
        assert!(
            !manifest.contains("[target."),
            "target ごとの依存の節を持たない"
        );
        assert_eq!(
            section_keys(&manifest, "dependencies"),
            vec![
                "folio".to_string(),
                "tsuzuri-contract".to_string(),
                "tsuzuri-core".to_string()
            ]
        );
        assert_eq!(
            string_value(&manifest, "[bin]", "name").as_deref(),
            Some("tz")
        );
    }
}
