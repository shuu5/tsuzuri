//! 外の読みの持ち回しの表と folio の読みの歯（接頭辞 eheld_・設計ノート surface-wave23b 行 e-held-design の完了の条件）。
//! 偽の folio は歯ごとの作業場の sh の script で、受けた引数を log に 1 行足して字を返す（眠らない）。

use std::cell::Cell;
use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use tsuzuri_boundary::server::design::{DESIGN_HOLD, Design};
use tsuzuri_boundary::server::held::{FAILED_HOLD, Held};
use tsuzuri_boundary::server::seat::{
    DOCTOR_ARGS, HOLD, Seat, SLOW_HOLD, TICK_ARGS, USAGE_ARGS, ceiling, input_marks, read_held,
};

/// 書いてから名を移す（撃たれている script を書きかけで見せない）。
fn put(path: &Path, text: &str, mode: u32) {
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, text).expect("file を書く");
    fs::set_permissions(&tmp, fs::Permissions::from_mode(mode)).expect("file の権限");
    fs::rename(&tmp, path).expect("file を移す");
}

/// 歯ごとの作業場（repo・偽の folio・その撃ちの log）。
struct Place {
    root: PathBuf,
    repo: PathBuf,
}

impl Place {
    fn new(name: &str) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("eheld")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        let repo = root.join("repo");
        fs::create_dir_all(repo.join("design-intent/adr")).expect("設計文書の dir");
        fs::create_dir_all(repo.join("contracts")).expect("contracts の dir");
        fs::create_dir_all(repo.join(".git/refs/heads")).expect(".git の refs");
        fs::write(repo.join("design-intent/rules.yaml"), "r").expect("file");
        fs::write(repo.join("design-intent/adr/ADR-1.yaml"), "a").expect("file");
        fs::write(repo.join("contracts/n.toml"), "c").expect("file");
        fs::write(repo.join(".git/HEAD"), "ref: refs/heads/main\n").expect("HEAD");
        fs::write(repo.join(".git/refs/heads/main"), "0000\n").expect("ref");
        let place = Place { root, repo };
        put(
            &place.folio(),
            &format!(
                "#!/bin/sh\necho \"$@\" >> '{}'\necho ok\n",
                place.log().display()
            ),
            0o755,
        );
        place
    }

    fn folio(&self) -> PathBuf {
        self.root.join("folio")
    }

    fn log(&self) -> PathBuf {
        self.root.join("calls.log")
    }

    fn design(&self) -> Design {
        Design::new(&self.repo, self.folio())
    }

    /// 偽の folio の撃ちの引数の行（撃ちの順）。
    fn calls(&self) -> Vec<String> {
        fs::read_to_string(self.log())
            .unwrap_or_default()
            .lines()
            .map(str::to_string)
            .collect()
    }

    /// 3 つの読みを 1 回ずつ読む（どれも読める）。
    fn read_all(&self, design: &Design) {
        assert_eq!(design.text().as_deref(), Some("ok\n"));
        assert_eq!(design.summary().as_deref(), Some("ok\n"));
        assert_eq!(design.rulings().as_deref(), Some("ok\n"));
    }
}

/// file を、前と長さが違う字で書き替える（更新時刻の粒度に頼らない）。
fn rewrite(path: &Path, text: &str) {
    fs::write(path, text).expect("file を書き替える");
}

#[test]
fn eheld_held_marks_and_ceiling() {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join("eheld").join("held");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("作業場");
    let mark = root.join("mark");
    fs::write(&mark, "1").expect("印");
    let marks = [mark.clone(), root.join("absent")];
    let key: Vec<OsString> = vec!["k".into()];
    let held = Held::new();
    let shots = Cell::new(0);
    let read = |text: Option<&'static str>| {
        let shots = &shots;
        move || {
            shots.set(shots.get() + 1);
            text.map(str::to_string)
        }
    };
    let ceiling = Duration::from_millis(200);

    assert_eq!(held.get(&key, &marks, ceiling, read(Some("a"))).as_deref(), Some("a"));
    assert_eq!(shots.get(), 1);
    // 印が同じで上限の内なら撃たず、1 回目の字を返す。
    assert_eq!(held.get(&key, &marks, ceiling, read(Some("b"))).as_deref(), Some("a"));
    assert_eq!(shots.get(), 1);
    // 印の file を書き替えると撃ち直す。
    rewrite(&mark, "22");
    assert_eq!(held.get(&key, &marks, ceiling, read(Some("c"))).as_deref(), Some("c"));
    assert_eq!(shots.get(), 2);
    // clone は同じ表を分け合う。
    assert_eq!(held.clone().get(&key, &marks, ceiling, read(Some("d"))).as_deref(), Some("c"));
    assert_eq!(shots.get(), 2);
    // 別の鍵は別の持ち分。
    let other: Vec<OsString> = vec!["other".into()];
    assert_eq!(held.get(&other, &marks, ceiling, read(Some("e"))).as_deref(), Some("e"));
    assert_eq!(shots.get(), 3);
    // 上限を過ぎると撃ち直す。
    thread::sleep(ceiling + Duration::from_millis(100));
    assert_eq!(held.get(&key, &marks, ceiling, read(Some("f"))).as_deref(), Some("f"));
    assert_eq!(shots.get(), 4);

    // 読めなかった読みは、上限が 60 秒でも FAILED_HOLD の内は撃ち直さず None を返し、過ぎれば撃ち直す。
    assert_eq!(FAILED_HOLD, Duration::from_secs(5));
    let long = Duration::from_secs(60);
    let bad: Vec<OsString> = vec!["bad".into()];
    assert_eq!(held.get(&bad, &marks, long, read(None)), None);
    assert_eq!(shots.get(), 5);
    assert_eq!(held.get(&bad, &marks, long, read(Some("g"))), None);
    assert_eq!(shots.get(), 5);
    // 印が動けば FAILED_HOLD の内でも撃ち直す。
    rewrite(&mark, "333");
    assert_eq!(held.get(&bad, &marks, long, read(Some("h"))).as_deref(), Some("h"));
    assert_eq!(shots.get(), 6);
    assert_eq!(held.get(&bad, &marks, long, read(Some("i"))).as_deref(), Some("h"));
    assert_eq!(shots.get(), 6);
    let bad2: Vec<OsString> = vec!["bad2".into()];
    assert_eq!(held.get(&bad2, &marks, long, read(None)), None);
    assert_eq!(shots.get(), 7);
    thread::sleep(FAILED_HOLD + Duration::from_millis(300));
    assert_eq!(held.get(&bad2, &marks, long, read(Some("j"))).as_deref(), Some("j"));
    assert_eq!(shots.get(), 8);
    // 読めた字は FAILED_HOLD を過ぎても上限の内なら持つ。
    assert_eq!(held.get(&bad2, &marks, long, read(Some("k"))).as_deref(), Some("j"));
    assert_eq!(shots.get(), 8);
}

#[test]
fn eheld_design_reads_follow_marks() {
    let place = Place::new("follow");
    let design = place.design();
    place.read_all(&design);
    assert_eq!(place.calls().len(), 3, "{:?}", place.calls());
    place.read_all(&design);
    place.read_all(&design.clone());
    assert_eq!(place.calls().len(), 3, "{:?}", place.calls());
    // 設計文書の dir の file を 1 つ書き替えた後の 3 つの読みで 6 回になる。
    rewrite(&place.repo.join("design-intent/adr/ADR-1.yaml"), "aa");
    place.read_all(&design);
    assert_eq!(place.calls().len(), 6, "{:?}", place.calls());
    place.read_all(&design);
    assert_eq!(place.calls().len(), 6, "{:?}", place.calls());
    // 3 つの引数の列は別の鍵で、それぞれ 2 回ずつ撃たれた。
    let calls = place.calls();
    for head in ["graph --print --dir", "graph --print --summary --dir", "check --emit-rulings --dir"] {
        let n = calls.iter().filter(|c| c.starts_with(head)).count();
        assert_eq!(n, 2, "{head}: {calls:?}");
    }
}

#[test]
fn eheld_design_rulings_marks() {
    assert_eq!(DESIGN_HOLD, Duration::from_secs(300));
    let place = Place::new("rulings");
    let design = place.design();
    place.read_all(&design);
    assert_eq!(place.calls().len(), 3);
    let mut seen = 3;
    let mut bump = |what: &str| {
        place.read_all(&design);
        let calls = place.calls();
        seen += 1;
        assert_eq!(calls.len(), seen, "{what}: 裁定の書き出しだけ 1 回: {calls:?}");
        assert!(calls[seen - 1].starts_with("check --emit-rulings"), "{what}: {calls:?}");
        place.read_all(&design);
        assert_eq!(place.calls().len(), seen, "{what}: 続けて読んでも撃たない");
    };
    rewrite(&place.repo.join("contracts/n.toml"), "cc");
    bump("contracts の file");
    fs::write(place.repo.join("contracts/new.toml"), "x").expect("新しい契約");
    bump("contracts の新しい file");
    rewrite(&place.repo.join(".git/refs/heads/main"), "11111\n");
    bump(".git/refs/heads の file");
    rewrite(&place.repo.join(".git/HEAD"), "ref: refs/heads/other\n");
    bump(".git/HEAD");
    rewrite(&place.repo.join(".git/packed-refs"), "# pack\n");
    bump(".git/packed-refs");
}

#[test]
fn eheld_design_rulings_marks_follow_worktree_git() {
    let place = Place::new("worktree");
    // .git が file の repo（worktree）: gitdir に HEAD、commondir の指す dir に refs。
    let common = place.root.join("common");
    let gitdir = common.join("worktrees/w");
    fs::create_dir_all(common.join("refs/heads")).expect("共有の refs");
    fs::create_dir_all(&gitdir).expect("gitdir");
    fs::write(gitdir.join("HEAD"), "ref: refs/heads/w\n").expect("HEAD");
    fs::write(gitdir.join("commondir"), "../..\n").expect("commondir");
    fs::write(common.join("refs/heads/w"), "0000\n").expect("ref");
    fs::remove_dir_all(place.repo.join(".git")).expect(".git を file にする");
    fs::write(
        place.repo.join(".git"),
        format!("gitdir: {}\n", gitdir.display()),
    )
    .expect(".git の file");
    let design = place.design();
    place.read_all(&design);
    assert_eq!(place.calls().len(), 3);
    for (path, text) in [
        (gitdir.join("HEAD"), "ref: refs/heads/x\n"),
        (common.join("refs/heads/w"), "11111\n"),
    ] {
        let before = place.calls().len();
        rewrite(&path, text);
        place.read_all(&design);
        assert_eq!(place.calls().len(), before + 1, "{}", path.display());
    }
}

const TARGET: &str = "proj-1:0.1";

/// 席の読みの出所（偽の器・state dir `name`・席の dir を作る）。
fn seat_in(place: &Place, name: &str) -> Seat {
    let state_dir = place.root.join(name).join("state");
    let seat = Seat {
        program: place.folio().into(),
        state_dir,
        target: TARGET.to_string(),
        cwd: place.root.clone(),
    };
    fs::create_dir_all(seat.seat_dir().expect("席の dir")).expect("席の dir を作る");
    seat
}

#[test]
fn eheld_vessel_reads_per_state_dir() {
    let place = Place::new("vessel-per-dir");
    let (a, b) = (seat_in(&place, "a"), seat_in(&place, "b"));
    let held = Held::new();
    let read_four = || {
        for seat in [&a, &b] {
            for head in [&TICK_ARGS[..], &DOCTOR_ARGS[..]] {
                assert_eq!(read_held(&held, seat, head).as_deref(), Some("ok\n"));
            }
        }
    };
    read_four();
    assert_eq!(place.calls().len(), 4, "{:?}", place.calls());
    read_four();
    assert_eq!(place.calls().len(), 4, "{:?}", place.calls());
    // A の席の dir に停止の記録を置くと、A の 2 つだけ撃ち直す。
    fs::write(a.seat_dir().expect("席の dir").join("heartbeat-off"), "").expect("停止の記録");
    read_four();
    let calls = place.calls();
    assert_eq!(calls.len(), 6, "{calls:?}");
    let a_dir = a.state_dir.display().to_string();
    for call in &calls[4..] {
        assert!(call.ends_with(&format!("--state-dir {a_dir}")), "{call}");
    }
}

#[test]
fn eheld_vessel_marks_by_head() {
    assert_eq!(HOLD, Duration::from_secs(5));
    assert_eq!(SLOW_HOLD, Duration::from_secs(30));
    assert_eq!(ceiling(&TICK_ARGS), HOLD);
    assert_eq!(ceiling(&DOCTOR_ARGS), SLOW_HOLD);
    assert_eq!(ceiling(&USAGE_ARGS), SLOW_HOLD);

    let place = Place::new("vessel-marks");
    let seat = seat_in(&place, "m");
    let groups = seat.groups_dir().expect("群の記録の dir");
    fs::create_dir_all(&groups).expect("群の記録の dir を作る");
    for name in ["g-1.account", "g-1.refused", "g-1.judged", "g-1.lock", "lock"] {
        fs::write(groups.join(name), "x").expect("群の記録");
    }
    let dir = seat.seat_dir().expect("席の dir");

    assert_eq!(
        input_marks(&seat, &USAGE_ARGS),
        [
            seat.state_dir.join("host.toml"),
            seat.state_dir.join("fleet/events.jsonl")
        ]
    );
    let doctor = input_marks(&seat, &DOCTOR_ARGS);
    let tick = input_marks(&seat, &TICK_ARGS);
    for (name, marks) in [("doctor", &doctor), ("tick", &tick)] {
        for want in [
            dir.join("heartbeat-off"),
            dir.join("heartbeat-on"),
            dir.join("account"),
            groups.join("g-1.account"),
            groups.join("g-1.refused"),
        ] {
            assert!(marks.contains(&want), "{name}: {want:?} {marks:?}");
        }
        for not in [groups.join("g-1.judged"), groups.join("g-1.lock"), groups.join("lock")] {
            assert!(!marks.contains(&not), "{name}: {not:?} {marks:?}");
        }
    }
    assert!(!doctor.contains(&dir.join("tick-last")), "{doctor:?}");
    assert!(tick.contains(&dir.join("tick-last")), "{tick:?}");
    assert!(input_marks(&seat, &["other"]).is_empty());
}
