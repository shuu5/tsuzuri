//! 行 c-acct-consult の境の歯（接頭辞 acctcwr_・判断の記録 ADR-55 決定 (4)）: account board の読みが anchor ごとに git で
//! 起草の置き場を引き、退いていない相談の窓の作業場の控えと最後の process の印を読み、session の行に窓を出す。
//! 偽の git は鍵 tsuzuri.draftsdir にだけ起草の置き場を返し、ほかの鍵と偽の器と偽の bd は落ちる（rc 1）。
#![cfg(test)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use tsuzuri_boundary::acct::{Acct, consult_windows};
use tsuzuri_contract::consult::{Form, ProcMark, Starter, WindowFile, WindowId};
use tsuzuri_contract::seat::SeatState;
use tsuzuri_contract::surface::SeatRole;
use tsuzuri_contract::wire;
use tsuzuri_core::account::project::ConsultWindow;

/// 今（2026-10-05T03:13:20Z）。
const NOW: u64 = 1_791_170_000;

/// 在り得ない pid（`/proc` に無い）。
const GONE: u32 = u32::MAX;

fn root(name: &str) -> PathBuf {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("acctcwr")
        .join(name);
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("歯の作業場");
    root
}

fn control(n: &str) -> WindowFile {
    WindowFile {
        id: WindowId::parse(n).expect("窓の id"),
        form: Form::Talk,
        topic: None,
        model: "opus".into(),
        effort: "high".into(),
        starter: Starter::Seat,
        uttered: None,
        request: None,
        made: "20261005T0100Z".into(),
    }
}

fn mark(k: u32, pid: u32, account: Option<&str>) -> ProcMark {
    ProcMark {
        k,
        form: Form::Talk,
        pid,
        at: "20261005T0100Z".into(),
        again: k > 1,
        tmux_window: Some(format!("@{k}")),
        account: account.map(str::to_string),
    }
}

/// 起草の置き場の下に作業場を置く（`dir` は dir の名・控えが偽なら .consult の dir だけ）。
fn put(drafts: &Path, dir: &str, n: &str, control_file: bool, marks: &[ProcMark]) {
    let dot = drafts.join(dir).join(".consult");
    fs::create_dir_all(&dot).expect(".consult");
    if control_file {
        let text = wire::encode(&control(n)).expect("控えの字");
        fs::write(dot.join("window.json"), text).expect("控えを書く");
    }
    for m in marks {
        let text = wire::encode(m).expect("印の字");
        fs::write(dot.join(format!("proc-{}.json", m.k)), text).expect("印を書く");
    }
}

/// 窓 cw1（印 2 つ・最後は生きた pid と口座 acct-new）・cw2（印なし）・cw5（最後の印の pid が無い）と、
/// 読まない dir（退いた cw3・同じ id の退いた cw2・控えの無い cw4・窓の名でない dir）。
fn drafts_of(root: &Path) -> PathBuf {
    let drafts = root.join("drafts");
    let me = std::process::id();
    put(
        &drafts,
        "consult-cw1",
        "cw1",
        true,
        &[mark(1, GONE, Some("/s/accounts/acct-old")), mark(2, me, Some("/s/accounts/acct-new"))],
    );
    put(&drafts, "consult-cw2", "cw2", true, &[]);
    put(&drafts, "retired-consult-cw2", "cw2", true, &[mark(1, me, Some("/s/accounts/acct-old"))]);
    put(&drafts, "retired-consult-cw3", "cw3", true, &[mark(1, me, None)]);
    put(&drafts, "consult-cw4", "cw4", false, &[mark(1, me, None)]);
    put(&drafts, "consult-cw5", "cw5", true, &[mark(1, GONE, Some("/s/accounts/acct-new"))]);
    put(&drafts, "notes", "cw6", true, &[]);
    drafts
}

/// (1) 退いていない作業場ごとに控えと最後の印（k の最も大きい物）と process の在る無し（窓の id の順）。
/// 退いた作業場（同じ id の作業場が在っても）・控えの無い作業場・窓の名でない dir は読まない。
#[test]
fn acctcwr_reads_open_workspaces() {
    let root = root("reads");
    let drafts = drafts_of(&root);
    let me = std::process::id();
    let want = vec![
        ConsultWindow {
            window: control("cw1"),
            last: Some(mark(2, me, Some("/s/accounts/acct-new"))),
            alive: true,
        },
        ConsultWindow {
            window: control("cw2"),
            last: None,
            alive: false,
        },
        ConsultWindow {
            window: control("cw5"),
            last: Some(mark(1, GONE, Some("/s/accounts/acct-new"))),
            alive: false,
        },
    ];
    assert_eq!(consult_windows(&drafts), want);
    assert_eq!(consult_windows(&root.join("no-such")), Vec::new());
}

fn script(path: &Path, body: &str) {
    fs::write(path, format!("#!/bin/sh\n{body}\n")).expect("偽の program");
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("偽の program の権限");
}

/// 偽の git（鍵 tsuzuri.draftsdir の時だけ `answer` を返す）と落ちる器と bd と、anchor 1 つの群の宣言。
fn world(root: &Path, answer: &str) -> Acct {
    let bin = root.join("bin");
    let state = root.join("state");
    fs::create_dir_all(&bin).expect("bin");
    fs::create_dir_all(&state).expect("state dir");
    script(
        &bin.join("git"),
        &format!("case \"$*\" in *tsuzuri.draftsdir*) printf '%s\\n' '{answer}';; *) exit 1;; esac"),
    );
    script(&bin.join("scribe2"), "exit 1");
    script(&bin.join("bd"), "exit 1");
    let anchor = root.join("work/proj-w");
    fs::create_dir_all(&anchor).expect("anchor");
    let toml = format!(
        "[[account]]\nlabel = \"acct-new\"\n\n[[account-group]]\nname = \"g-w\"\nanchors = [\"{}\"]\naccounts = [\"acct-new\"]\n",
        anchor.display()
    );
    fs::write(state.join("host.toml"), toml).expect("群の宣言");
    Acct::new(bin.join("scribe2"), bin.join("git"), bin.join("bd"), state, root)
}

/// (2) 起草の置き場の引けた anchor の project は、state dir が引けなくても席なしの行の直後に窓の行を出す
/// （口座は最後の印の口座の置き場の末の名・生きた process は run・無ければ wait）。置き場が引けなければ出さない。
#[test]
fn acctcwr_doc_lists_windows_from_git_drafts() {
    let root = root("doc");
    let drafts = drafts_of(&root);
    let acct = world(&root, &drafts.display().to_string());
    assert_eq!(acct.drafts(&root.join("work/proj-w")), Some(drafts.clone()));
    let got: Vec<(SeatRole, String, Option<String>, SeatState)> = acct
        .doc(NOW)
        .sessions
        .into_iter()
        .map(|l| (l.role, l.name, l.account, l.state))
        .collect();
    let consult = |n: &str, a: Option<&str>, s| (SeatRole::Consult, n.to_string(), a.map(str::to_string), s);
    assert_eq!(
        got,
        vec![
            (SeatRole::Orchestrator, String::new(), None, SeatState::Unknown),
            consult("cw1", Some("acct-new"), SeatState::Run),
            consult("cw2", None, SeatState::Wait),
            consult("cw5", Some("acct-new"), SeatState::Wait),
        ]
    );
    let blank = root.join("blank");
    let none = world(&blank, "");
    assert_eq!(none.drafts(&blank.join("work/proj-w")), None);
    assert!(none.doc(NOW).sessions.iter().all(|l| l.role != SeatRole::Consult));
}
