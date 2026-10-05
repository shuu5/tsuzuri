//! 窓の一覧と閉じと退かせの歯（接頭辞 cwcls_・設計ノート surface-wave27a 行 cs-close）。
//! 歯ごとの置き場（CARGO_TARGET_TMPDIR の下）に、git init した repo と state dir と起草の置き場と、偽の bd・bdw と、
//! 受けた argv を記録して窓の名を tmux.name の字で答える偽の tmux を置き（本物の tmux は撃たない）、
//! 窓の作業場（控え・process の印・所見）を歯が直に書いて tz consult list と close を撃つ。
//! 一覧の口座（行 e-acct-consult）の歯は口を撃たず、起草の置き場だけを置いて一覧の組み（`list::board` と `list::text`）を直に撃つ。
#![cfg(test)]

use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use crate::common::{err, git, out, rc, script};
use tsuzuri_boundary::consult::Ctx;
use tsuzuri_boundary::consult::list::{NO_ACCOUNT, board, epoch_of, text};
use tsuzuri_boundary::server::{events, ruling};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::consult::{
    ConsultBoard, Finding, FindingId, FindingRow, Form, ProcMark, RequestId, RequestRow, SeatQuota,
    Starter, WindowFile, WindowId, WindowState,
};
use tsuzuri_contract::wire;
use tsuzuri_core::consult::finding::check;
use tsuzuri_core::consult::launch::private_tmp;

const LEDGER: &str = r#"[{"id":"fx-c","title":"根","status":"open","issue_type":"epic","updated_at":"2026-10-03T00:00:00Z","notes":"NOTES"}]"#;

const RQ: &str = "相談の頼み = rq-20261003T1412Z-1・題 = 題なし・形 = 話す・model = fable・起こし手 = 持ち主の button・時刻 = 20261003T1412Z";

fn w(n: u32) -> WindowId {
    WindowId::new(n).unwrap()
}

/// 歯ごとの置き場。
struct Fx {
    root: PathBuf,
    repo: PathBuf,
    drafts: PathBuf,
}

impl Fx {
    fn new(name: &str) -> Fx {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("cwcls")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        let (repo, state, drafts) = (root.join("repo"), root.join("state"), root.join("drafts"));
        for d in [&repo, &state, &drafts, &root.join("bin")] {
            fs::create_dir_all(d).unwrap();
        }
        git(&repo, &["init", "-q"]);
        git(
            &repo,
            &["config", "scribe2.statedir", &state.display().to_string()],
        );
        git(
            &repo,
            &["config", "tsuzuri.draftsdir", &drafts.display().to_string()],
        );
        let r = root.display().to_string();
        let record = |log: &str| {
            format!(
                "for a in \"$@\"; do printf '%s\\037' \"$a\"; done >> '{r}/{log}'\necho >> '{r}/{log}'"
            )
        };
        script(&root.join("bin/bd"), &format!("exec cat '{r}/ledger.json'"));
        script(&root.join("bin/bdw"), &record("bdw.log"));
        let tmux = format!(
            "{}\ncase \"$1\" in display-message) cat '{r}/tmux.name' ;; esac",
            record("tmux.log")
        );
        script(&root.join("bin/tmux"), &tmux);
        let fx = Fx { root, repo, drafts };
        fx.notes(&[]);
        fx
    }

    fn notes(&self, lines: &[&str]) {
        let text = LEDGER.replace("NOTES", &lines.join("\\n"));
        fs::write(self.root.join("ledger.json"), text).unwrap();
    }

    fn tz(&self, args: &[&str]) -> Output {
        let path = format!(
            "{}:{}",
            self.root.join("bin").display(),
            std::env::var("PATH").unwrap()
        );
        Command::new(env!("CARGO_BIN_EXE_tz"))
            .arg("consult")
            .args(args)
            .current_dir(&self.repo)
            .env("PATH", path)
            .output()
            .unwrap()
    }

    /// 窓の作業場（`dir` は作業場の dir の名・最後の process の印の pid と tmux の窓）。
    fn window(&self, dir: &str, n: u32, form: Form, proc_: Option<(u32, Option<&str>)>) -> PathBuf {
        let ws = self.drafts.join(dir);
        for d in [".consult", "findings"] {
            fs::create_dir_all(ws.join(d)).unwrap();
        }
        let file = WindowFile {
            id: w(n),
            form,
            topic: (n == 1).then(|| "fx-c.2".to_string()),
            model: "fable".into(),
            effort: "xhigh".into(),
            starter: Starter::Seat,
            uttered: None,
            request: None,
            made: "20261003T1400Z".into(),
        };
        fs::write(
            ws.join(".consult/window.json"),
            wire::encode(&file).unwrap(),
        )
        .unwrap();
        if let Some((pid, tmux)) = proc_ {
            mark(&ws, 1, form, pid, tmux);
        }
        ws
    }

    /// 所見を置く（fixture complete.json の草稿・時刻 20261003T1500Z）。
    fn finding(&self, ws: &Path, n: u32, k: u32) -> Finding {
        let text = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/consult/finding-2/complete.json");
        let draft = check(&fs::read_to_string(text).unwrap()).unwrap();
        let f = Finding::new(FindingId::new(w(n), k).unwrap(), "20261003T1500Z", draft);
        fs::write(
            ws.join(format!("findings/{}.json", f.id)),
            wire::encode(&f).unwrap(),
        )
        .unwrap();
        f
    }

    fn calls(&self, log: &str) -> Vec<Vec<String>> {
        fs::read_to_string(self.root.join(log))
            .unwrap_or_default()
            .lines()
            .map(|l| {
                l.split('\x1f')
                    .filter(|a| !a.is_empty())
                    .map(str::to_string)
                    .collect()
            })
            .collect()
    }
}

/// 作業場に k 番目の process の印を置く（pid と tmux の窓）。
fn mark(ws: &Path, k: u32, form: Form, pid: u32, tmux: Option<&str>) {
    let m = ProcMark {
        k,
        form,
        pid,
        at: "20261003T1401Z".into(),
        again: false,
        tmux_window: tmux.map(str::to_string),
        account: None,
    };
    fs::write(
        ws.join(format!(".consult/proc-{k}.json")),
        wire::encode(&m).unwrap(),
    )
    .unwrap();
}

/// 一覧の歯の置き場（cw1 生きている問う窓・cw2 止まった問う窓・cw3 process の印が生きている閉じた窓・cw4 閉じの行の在る退いた窓・cw5 起こしていない窓）。
fn listed(fx: &Fx, today: &str) -> Vec<String> {
    let me = std::process::id();
    let ws1 = fx.window("consult-cw1", 1, Form::Ask, Some((me, None)));
    fx.finding(&ws1, 1, 1);
    fx.window("consult-cw2", 2, Form::Ask, Some((0, None)));
    let ws3 = fx.window("consult-cw3", 3, Form::Talk, Some((me, Some("@3"))));
    fx.finding(&ws3, 3, 1);
    fx.window("retired-consult-cw4", 4, Form::Talk, None);
    fx.window("consult-cw5", 5, Form::Talk, None);
    let open = |n: u32| {
        format!(
            "相談の開き = cw{n}・形 = 問う・題 = 題なし・起こし手 = 席・model = fable・念入りさ = xhigh・結果 = 開いた・時刻 = {today}"
        )
    };
    vec![
        "相談の開き = cw1・形 = 問う・題 = 題なし・起こし手 = 席・model = fable・念入りさ = xhigh・結果 = 開いた・時刻 = 20261002T1400Z".to_string(),
        open(1),
        open(2),
        "相談の開き = cw1・形 = 問う・題 = 題なし・起こし手 = 席・model = fable・念入りさ = xhigh・結果 = 落ちた・時刻 = 20261002T1500Z".to_string(),
        "相談の受け = cw1-1・経路 = 見張り・時刻 = 20261003T1501Z".to_string(),
        "相談の閉じ = cw3・起こし手 = 席・所見 = 1・時刻 = 20261003T1600Z".to_string(),
        "相談の閉じ = cw4・起こし手 = 席・所見 = 0・時刻 = 20261003T1600Z".to_string(),
        "相談の処分 = cw3-1・採否 = 採らない・理由 = 古い・保存 = なし・不保存 = work/report.md、work/probe.py・時刻 = 20261003T1601Z".to_string(),
        RQ.to_string(),
        "相談の頼み = rq-20261003T1412Z-2・題 = 題なし・形 = 話す・model = fable・起こし手 = 持ち主の button・時刻 = 20261003T1412Z".to_string(),
        "相談の受け = rq-20261003T1412Z-2・経路 = hook・時刻 = 20261003T1413Z".to_string(),
        RQ.to_string(),
    ]
}

#[test]
fn cwcls_list_states_and_text() {
    let fx = Fx::new("text");
    let today = ruling::minute(events::now());
    let lines = listed(&fx, &today);
    fx.notes(&lines.iter().map(String::as_str).collect::<Vec<_>>());
    let o = fx.tz(&["list"]);
    assert_eq!(rc(&o), 0, "{}", err(&o));
    let want = [
        "窓 cw1・形 問う・題 fx-c.2・状態 生きている・所見 1・未処分 1・口座 分からない",
        "窓 cw2・形 問う・題 題なし・状態 止まった・所見 0・未処分 0・口座 分からない",
        "窓 cw3・形 話す・題 題なし・状態 閉じた・所見 1・未処分 0・口座 分からない",
        "窓 cw4・形 話す・題 題なし・状態 退いた・所見 0・未処分 0・口座 分からない",
        "窓 cw5・形 話す・題 題なし・状態 止まった・所見 0・未処分 0・口座 分からない",
        "受けの無い頼み rq-20261003T1412Z-1・題 題なし",
        "席の問う窓: 今日 2・今 1（規則の行 R-38 の上限 同時 1・1 日 3）",
    ];
    assert_eq!(out(&o).lines().collect::<Vec<_>>(), want);
    assert!(fx.calls("bdw.log").is_empty(), "一覧は台帳を書かない");
    fx.window("consult-cw10", 10, Form::Talk, None);
    let text = out(&fx.tz(&["list"]));
    let ids: Vec<&str> = text
        .lines()
        .filter_map(|l| l.strip_prefix("窓 ")?.split('・').next())
        .collect();
    assert_eq!(ids, ["cw1", "cw2", "cw3", "cw4", "cw5", "cw10"]);
    assert_eq!(rc(&fx.tz(&["list", "x"])), 1);
    fs::remove_file(fx.root.join("ledger.json")).unwrap();
    assert_eq!(rc(&fx.tz(&["list"])), 2);
}

#[test]
fn cwcls_list_json_board() {
    let fx = Fx::new("json");
    let today = ruling::minute(events::now());
    let lines = listed(&fx, &today);
    fx.notes(&lines.iter().map(String::as_str).collect::<Vec<_>>());
    let o = fx.tz(&["list", "--json"]);
    assert_eq!(rc(&o), 0, "{}", err(&o));
    let b: ConsultBoard = wire::decode(out(&o).trim()).unwrap();
    let Reading::Known(wins) = &b.windows else {
        panic!("{b:?}")
    };
    let states: Vec<(WindowId, WindowState)> = wins.iter().map(|r| (r.id, r.state)).collect();
    let want = [
        (w(1), WindowState::Live),
        (w(2), WindowState::Stalled),
        (w(3), WindowState::Closed),
        (w(4), WindowState::Retired),
        (w(5), WindowState::Stalled),
    ];
    assert_eq!(states, want);
    assert_eq!((wins[0].opened, wins[2].opened), (epoch_of(&today), None));
    let row = FindingRow {
        id: FindingId::new(w(1), 1).unwrap(),
        topic: Some("fx-hub.7".into()),
        arrived: Some(1_791_039_600),
        received: Some(1_791_039_660),
    };
    assert_eq!(b.findings, Reading::Known(vec![row]));
    let rq = RequestRow {
        id: RequestId::parse("rq-20261003T1412Z-1").unwrap(),
        topic: None,
        at: 1_791_036_720,
    };
    assert_eq!(b.requests, Reading::Known(vec![rq]));
    assert_eq!(b.quota, Reading::Known(SeatQuota { today: 2, live: 1 }));
    assert_eq!(epoch_of("20261003T1412Z"), Some(1_791_036_720));
    assert_eq!(
        (epoch_of("20261003T1412"), epoch_of("2026-10-03")),
        (None, None)
    );
}

#[test]
fn cwcls_close_named_window_and_retire() {
    let fx = Fx::new("close");
    let ws1 = fx.window("consult-cw1", 1, Form::Talk, Some((0, Some("@6"))));
    mark(&ws1, 2, Form::Talk, 0, Some("@7"));
    let tmp = private_tmp(&fs::canonicalize(&ws1).unwrap().display().to_string());
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir(&tmp).unwrap();
    fs::write(Path::new(&tmp).join("left"), "x").unwrap();
    fs::write(fx.root.join("tmux.name"), "consult-cw1\n").unwrap();
    let before = ruling::minute(events::now());
    let o = fx.tz(&["close", "cw1", "--by", "chat"]);
    let after = ruling::minute(events::now());
    assert_eq!(rc(&o), 0, "{}", err(&o));
    let gone = fx.drafts.join("retired-consult-cw1");
    assert_eq!(
        out(&o),
        format!("窓 cw1 を閉じた\n退いた: {}\n", gone.display())
    );
    assert!(gone.join(".consult/window.json").is_file() && !ws1.exists());
    assert!(!Path::new(&tmp).exists());
    let tmux = fx.calls("tmux.log");
    assert_eq!(
        tmux,
        [
            vec!["display-message", "-p", "-t", "@7", "#{window_name}"],
            vec!["kill-window", "-t", "@7"]
        ]
    );
    let bdw = fx.calls("bdw.log");
    let line = bdw[0][2].strip_prefix("--append-notes=").unwrap();
    let at = line.rsplit_once("時刻 = ").unwrap().1;
    assert!(before.as_str() <= at && at <= after.as_str(), "{at}");
    assert_eq!((bdw.len(), bdw[0][1].as_str()), (1, "fx-c"));
    assert_eq!(
        line,
        format!("相談の閉じ = cw1・起こし手 = 持ち主のチャット・所見 = 0・時刻 = {at}")
    );
}

#[test]
fn cwcls_close_other_name_and_keep() {
    let fx = Fx::new("other");
    let ws2 = fx.window("consult-cw2", 2, Form::Talk, Some((0, Some("@8"))));
    fx.finding(&ws2, 2, 1);
    fx.finding(&ws2, 2, 2);
    fx.notes(&[
        "相談の処分 = cw2-2・採否 = 採らない・理由 = 古い・保存 = なし・不保存 = work/report.md、work/probe.py・時刻 = 20261003T1601Z",
    ]);
    let tmp = private_tmp(&fs::canonicalize(&ws2).unwrap().display().to_string());
    let _ = fs::remove_file(&tmp);
    let _ = fs::remove_dir_all(&tmp);
    let target = fx.root.join("elsewhere");
    fs::create_dir_all(&target).unwrap();
    std::os::unix::fs::symlink(&target, &tmp).unwrap();
    fs::write(fx.root.join("tmux.name"), "consult-cw20\n").unwrap();
    let o = fx.tz(&["close", "cw2", "--by", "seat"]);
    assert_eq!(rc(&o), 0, "{}", err(&o));
    assert_eq!(out(&o), "窓 cw2 を閉じた\n");
    assert!(err(&o).contains("@8"), "{}", err(&o));
    assert_eq!(
        fx.calls("tmux.log").len(),
        1,
        "名が違えば kill-window を撃たない"
    );
    assert!(ws2.is_dir(), "処分の無い所見が在る間は退かせない");
    assert!(fs::symlink_metadata(&tmp).unwrap().is_symlink() && target.is_dir());
    let bdw = fx.calls("bdw.log");
    assert!(bdw[0][2].starts_with("--append-notes=相談の閉じ = cw2・起こし手 = 席・所見 = 2・"));
    fs::remove_file(&tmp).unwrap();
}

#[test]
fn cwcls_close_refusals() {
    let fx = Fx::new("refuse");
    fx.window("consult-cw1", 1, Form::Ask, None);
    fx.window("retired-consult-cw4", 4, Form::Talk, None);
    // tmux の窓を持つ話す窓 cw2（2 つの位置の引数の見本の 2 つ目も在る窓にする・閉じた窓の見本にも使う）。
    fx.window("consult-cw2", 2, Form::Talk, Some((0, Some("@9"))));
    fs::write(fx.root.join("tmux.name"), "consult-cw2\n").unwrap();
    for bad in [
        &["close", "cw9", "--by", "seat"][..],
        &["close", "cw4", "--by", "seat"],
        &["close", "cw1"],
        &["close", "cw1", "--by", "owner"],
        &["close", "cw1", "cw2", "--by", "seat"],
        &["close", "x", "--by", "seat"],
    ] {
        assert_eq!(rc(&fx.tz(bad)), 1, "{bad:?}");
    }
    fx.notes(&[
        "相談の閉じ = cw1・起こし手 = 席・所見 = 0・時刻 = 20261003T1600Z",
        "相談の閉じ = cw2・起こし手 = 席・所見 = 0・時刻 = 20261003T1600Z",
    ]);
    for id in ["cw1", "cw2"] {
        let o = fx.tz(&["close", id, "--by", "seat"]);
        assert_eq!(rc(&o), 1);
        assert!(err(&o).contains("もう閉じた"), "{id} {}", err(&o));
    }
    assert!(fx.calls("bdw.log").is_empty() && fx.calls("tmux.log").is_empty());
    fx.notes(&["相談の閉じ = cw2・起こし手 = 席・所見 = 0・時刻 = 20261003T1600Z"]);
    let o = fx.tz(&["close", "cw1", "--by", "button"]);
    assert_eq!(rc(&o), 0, "問う窓は tmux を撃たずに閉じる: {}", err(&o));
    assert!(fx.calls("tmux.log").is_empty());
    let bdw = fx.calls("bdw.log");
    assert!(
        bdw[0][2].contains("起こし手 = 持ち主の button・"),
        "{bdw:?}"
    );
}

/// 口座の歯の在り得ない pid（`/proc` に無い）。
const ACC_GONE: u32 = u32::MAX;

fn acc_put(drafts: &Path, dir: &str, n: &str, accounts: &[Option<&str>]) {
    let dot = drafts.join(dir).join(".consult");
    fs::create_dir_all(&dot).expect(".consult");
    let control = WindowFile {
        id: WindowId::parse(n).expect("窓の id"),
        form: Form::Talk,
        topic: None,
        model: "opus".into(),
        effort: "high".into(),
        starter: Starter::Seat,
        uttered: None,
        request: None,
        made: "20261005T0100Z".into(),
    };
    fs::write(dot.join("window.json"), wire::encode(&control).expect("控えの字")).expect("控え");
    for (i, a) in accounts.iter().enumerate() {
        let k = u32::try_from(i).expect("k") + 1;
        let m = ProcMark {
            k,
            form: Form::Talk,
            pid: ACC_GONE,
            at: "20261005T0100Z".into(),
            again: k > 1,
            tmux_window: Some(format!("@{k}")),
            account: a.map(str::to_string),
        };
        let text = wire::encode(&m).expect("印の字");
        fs::write(dot.join(format!("proc-{k}.json")), text).expect("印");
    }
}

fn acc_ctx(name: &str) -> Ctx {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("cwcls-acc")
        .join(name);
    let _ = fs::remove_dir_all(&root);
    let drafts = root.join("drafts");
    acc_put(&drafts, "consult-cw1", "cw1", &[Some("/s/accounts/acct-old"), Some("/s/accounts/acct-new/")]);
    acc_put(&drafts, "consult-cw2", "cw2", &[]);
    acc_put(&drafts, "consult-cw3", "cw3", &[None]);
    acc_put(&drafts, "consult-cw4", "cw4", &[Some("/s/accounts/acct-old"), None]);
    acc_put(&drafts, "retired-consult-cw5", "cw5", &[Some("/s/accounts/acct-new")]);
    Ctx {
        repo: root.clone(),
        state: root,
        drafts,
        bd: OsString::from("bd"),
        bdw: OsString::from("bdw"),
    }
}

/// 行 e-acct-consult（判断の記録 ADR-55 決定 (4)）: 電文の窓の行の口座（窓の id の順・退いた窓も同じ決まり）と、tz consult list の行の末の「口座 <名>」。
#[test]
fn cwcls_list_rows_hold_the_account() {
    let c = acc_ctx("rows");
    let b = board(&c, &[], "20261005T0300Z");
    let Reading::Known(wins) = &b.windows else {
        panic!("窓の段が読めない: {b:?}")
    };
    let got: Vec<(String, Option<&str>)> = wins
        .iter()
        .map(|w| (w.id.to_string(), w.account.as_deref()))
        .collect();
    assert_eq!(
        got,
        [
            ("cw1".to_string(), Some("acct-new")),
            ("cw2".to_string(), None),
            ("cw3".to_string(), None),
            ("cw4".to_string(), None),
            ("cw5".to_string(), Some("acct-new")),
        ]
    );
    let tails: Vec<String> = text(&b)
        .into_iter()
        .filter(|l| l.starts_with("窓 "))
        .filter_map(|l| l.rsplit('・').next().map(str::to_string))
        .collect();
    let none = format!("口座 {NO_ACCOUNT}");
    assert_eq!(
        tails,
        ["口座 acct-new", none.as_str(), none.as_str(), none.as_str(), "口座 acct-new"]
    );
    assert_eq!(NO_ACCOUNT, "分からない");
}
