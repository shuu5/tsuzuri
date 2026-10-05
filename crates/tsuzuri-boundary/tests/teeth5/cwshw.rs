//! 席が所見を読む口と処分の口の歯（接頭辞 cwshw_・設計ノート surface-wave27a 行 cs-show・受入 AC16 の台帳の 2 行）。
//! 歯ごとの置き場（CARGO_TARGET_TMPDIR の下）に、git init した repo と state dir と起草の置き場と、偽の bd（置き場の
//! ledger.json を出す）と偽の bdw（argv を記録する）を置き、窓 cw3 の作業場に fixture finding-2 の complete.json を
//! 所見 cw3-1 として直に置いてから、tz consult show と dispose を撃つ。
#![cfg(test)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use crate::common::{err, git, out, rc, script};
use tsuzuri_boundary::server::{events, ruling};
use tsuzuri_contract::consult::{Finding, FindingId, Form, Starter, WindowFile, WindowId};
use tsuzuri_contract::wire;
use tsuzuri_core::consult::finding::check;
use tsuzuri_core::consult::lines::{Line, read};

/// 台帳の写し（根の epic・memo・memo の下の問い fx-hub.7・memo の notes は歯が足す）。
const LEDGER: &str = r#"[
{"id":"fx-hub","title":"根","status":"open","issue_type":"epic","updated_at":"2026-10-03T00:00:00Z"},
{"id":"fx-hub.5","title":"控え","status":"open","issue_type":"task","labels":["intake:memo"],"parent":"fx-hub","updated_at":"2026-10-03T00:00:00Z","notes":"NOTES"},
{"id":"fx-hub.7","title":"問い","status":"open","issue_type":"task","labels":["intake:question"],"parent":"fx-hub.5","updated_at":"2026-10-03T00:00:00Z"}
]"#;

/// 歯ごとの置き場（窓 cw3 の作業場と所見 cw3-1 を置いた後）。
struct Fx {
    root: PathBuf,
    repo: PathBuf,
    drafts: PathBuf,
    ws: PathBuf,
}

impl Fx {
    fn new(name: &str) -> Fx {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("cwshw")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        let (repo, state, drafts) = (root.join("repo"), root.join("state"), root.join("drafts"));
        for d in [
            &repo,
            &state,
            &drafts,
            &root.join("bin"),
            &root.join("into"),
        ] {
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
        script(&root.join("bin/bd"), &format!("exec cat '{r}/ledger.json'"));
        script(
            &root.join("bin/bdw"),
            &format!(
                "for a in \"$@\"; do printf '%s\\037' \"$a\"; done >> '{r}/bdw.log'\necho >> '{r}/bdw.log'"
            ),
        );
        let ws = drafts.join("consult-cw3");
        let fx = Fx {
            root,
            repo,
            drafts,
            ws,
        };
        fx.notes(&[]);
        fx.answer();
        fx
    }

    /// 窓 cw3 の作業場を置き、complete.json の草稿に id と時刻 20261003T1500Z を足した所見 cw3-1 を置く。
    fn answer(&self) {
        for d in [".consult", "findings", "work"] {
            fs::create_dir_all(self.ws.join(d)).unwrap();
        }
        let w = WindowFile {
            id: WindowId::new(3).unwrap(),
            form: Form::Talk,
            topic: None,
            model: "fable".into(),
            effort: "xhigh".into(),
            starter: Starter::Seat,
            uttered: None,
            request: None,
            made: "20261003T1400Z".into(),
        };
        fs::write(
            self.ws.join(".consult/window.json"),
            wire::encode(&w).unwrap(),
        )
        .unwrap();
        fs::write(self.ws.join("work/report.md"), "調べのレポート\n").unwrap();
        fs::write(self.ws.join("work/probe.py"), "print(1)\n").unwrap();
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/consult/finding-2/complete.json");
        let draft = check(&fs::read_to_string(fixture).unwrap()).unwrap();
        let id = FindingId::new(WindowId::new(3).unwrap(), 1).unwrap();
        let f = Finding::new(id, "20261003T1500Z", draft);
        fs::write(
            self.ws.join("findings/cw3-1.json"),
            wire::encode(&f).unwrap(),
        )
        .unwrap();
    }

    /// memo の notes を替える（行の列は notes.txt にも置く）。
    fn notes(&self, lines: &[String]) {
        let text = LEDGER.replace("NOTES", &lines.join("\\n"));
        fs::write(self.root.join("ledger.json"), text).unwrap();
        fs::write(self.root.join("notes.txt"), lines.join("\n")).unwrap();
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
            .env("TZ_WRAPPER", "/x/tzw")
            .output()
            .unwrap()
    }

    /// 偽の bdw が受けた（置き場・行の字）の列。
    fn written(&self) -> Vec<(String, String)> {
        fs::read_to_string(self.root.join("bdw.log"))
            .unwrap_or_default()
            .lines()
            .map(|l| {
                let a: Vec<&str> = l.split('\x1f').collect();
                assert_eq!(a[0], "update");
                (
                    a[1].to_string(),
                    a[2].strip_prefix("--append-notes=").unwrap().to_string(),
                )
            })
            .collect()
    }

    /// 書いた行を台帳の写しの memo の notes に足す（席が書いた後の台帳の見立て）。
    fn settle(&self, extra: &[String]) {
        let before = fs::read_to_string(self.root.join("notes.txt")).unwrap();
        let mut lines: Vec<String> = before.lines().map(str::to_string).collect();
        lines.extend(self.written().into_iter().map(|(_, t)| t));
        lines.extend(extra.iter().cloned());
        self.notes(&lines);
        let _ = fs::remove_file(self.root.join("bdw.log"));
    }
}

fn time_of(text: &str) -> String {
    text.rsplit_once("時刻 = ").unwrap().1.to_string()
}

const DISPOSE: [&str; 7] = [
    "dispose",
    "cw3-1",
    "--verdict",
    "一部採る",
    "--reason",
    "前提の 1 つが古い",
    "--keep",
];

#[test]
fn cwshw_finding_two_lines_once() {
    let fx = Fx::new("twice");
    let date = fs::read_to_string(fx.ws.join("findings/cw3-1.json")).unwrap();
    let date = date
        .split("\"date\":\"")
        .nth(1)
        .unwrap()
        .get(..14)
        .unwrap()
        .to_string();
    // ほかの所見 cw3-2 の 2 行だけが在る台帳（同じ id の行だけを重なりと見る）。
    fx.notes(&[
        "相談の所見 = cw3-2・窓 = cw3・題 = fx-hub.7・時刻 = 20261003T1500Z".to_string(),
        "相談の受け = cw3-2・経路 = 一覧・時刻 = 20261003T1500Z".to_string(),
    ]);
    let before = ruling::minute(events::now());
    let o = fx.tz(&["show", "cw3-1", "--via", "見張り"]);
    let after = ruling::minute(events::now());
    assert_eq!(rc(&o), 0, "{}", err(&o));
    let w = fx.written();
    assert_eq!(w.len(), 2, "{w:?}");
    assert!(w.iter().all(|(home, _)| home == "fx-hub.5"), "{w:?}");
    assert_eq!(
        w[0].1,
        format!("相談の所見 = cw3-1・窓 = cw3・題 = fx-hub.7・時刻 = {date}")
    );
    let at = time_of(&w[1].1);
    assert!(before <= at && at <= after, "{at}");
    assert_eq!(
        w[1].1,
        format!("相談の受け = cw3-1・経路 = 見張り・時刻 = {at}")
    );
    assert!(matches!(read(&w[1].1), Some(Line::Receipt { .. })));
    fx.settle(&[]);
    let o = fx.tz(&["show", "cw3-1", "--via", "一覧"]);
    assert_eq!(rc(&o), 0);
    assert!(fx.written().is_empty(), "2 度目は書かない");
    fx.notes(&[format!(
        "相談の所見 = cw3-1・窓 = cw3・題 = fx-hub.7・時刻 = {date}"
    )]);
    let o = fx.tz(&["show", "cw3-1", "--via", "hook"]);
    assert_eq!(rc(&o), 0);
    let w = fx.written();
    assert_eq!(w.len(), 1);
    assert!(
        w[0].1.starts_with("相談の受け = cw3-1・経路 = hook・"),
        "{w:?}"
    );
}

#[test]
fn cwshw_show_text_and_request() {
    let fx = Fx::new("text");
    let o = fx.tz(&["show", "cw3-1", "--via", "完了"]);
    let text = out(&o);
    let ws = fx.ws.display().to_string();
    for want in [
        "所見 cw3-1（窓 cw3・題 fx-hub.7・時刻 ".to_string(),
        "結論: 読む根は repo と器の出力の 2 つに絞る\n".to_string(),
        "- a 絞る（採る）: repo と fleet と pipe だけを読む根にする・理由: 資格の file が読めない\n".to_string(),
        "- b 丸ごと（採らない）: ".to_string(),
        "- [verified] 囲いの中から accounts の file が読める（作業場の probe で撃った）\n".to_string(),
        "- [inferred] 対話の窓でも同じ設定が効く".to_string(),
        "根拠: ADR-29, FR19\n持ち主に問う: なし\n触る節点: FR19\n束の要約値: 0123456789abcdef\n囲いの断り: なし\n".to_string(),
        format!("- report {ws}/work/report.md（調べのレポート）\n- program {ws}/work/probe.py（"),
    ] {
        assert!(text.contains(&want), "{want}\n{text}");
    }
    let rq = "相談の頼み = rq-20261003T1412Z-1・題 = fx-hub.7・形 = 問う・model = opus・起こし手 = 持ち主の button・時刻 = 20261003T1412Z";
    fx.settle(&[rq.to_string()]);
    let o = fx.tz(&["show", "rq-20261003T1412Z-1", "--via", "見張り"]);
    assert_eq!(rc(&o), 0, "{}", err(&o));
    let want = "頼み rq-20261003T1412Z-1（題 fx-hub.7・形 問う・model opus・時刻 20261003T1412Z）\n開く: /x/tzw consult open --request rq-20261003T1412Z-1 --by button\n";
    assert_eq!(out(&o), want);
    assert!(fx.written().is_empty());
    for bad in [
        &["show", "rq-20261003T1412Z-2", "--via", "見張り"][..],
        &["show", "cw3-9", "--via", "見張り"],
        &["show", "cw3-1"],
        &["show", "cw3-1", "--via", "完"],
        &["show", "x", "--via", "hook"],
    ] {
        assert_eq!(rc(&fx.tz(bad)), 1, "{bad:?}");
    }
    fs::rename(&fx.ws, fx.drafts.join("retired-consult-cw3")).unwrap();
    assert_eq!(
        rc(&fx.tz(&["show", "cw3-1", "--via", "一覧"])),
        0,
        "退いた作業場の所見も読む"
    );
}

#[test]
fn cwshw_dispose_keep_drop() {
    let fx = Fx::new("dispose");
    assert_eq!(rc(&fx.tz(&["show", "cw3-1", "--via", "見張り"])), 0);
    fx.settle(&[]);
    let into = fx.root.join("into");
    let into_s = into.display().to_string();
    let before = ruling::minute(events::now());
    let o = fx.tz(&[
        &DISPOSE[..],
        &[
            "work/report.md",
            "--drop",
            "work/probe.py",
            "--into",
            &into_s,
        ],
    ]
    .concat());
    assert_eq!(rc(&o), 0, "{}", err(&o));
    assert_eq!(
        out(&o),
        "処分を記した\ndocs/consult/kept/cw3-1/work/report.md\n"
    );
    let copied = fs::read_to_string(into.join("docs/consult/kept/cw3-1/work/report.md")).unwrap();
    assert_eq!(copied, "調べのレポート\n");
    assert!(!into.join("docs/consult/kept/cw3-1/work/probe.py").exists());
    let after = ruling::minute(events::now());
    let w = fx.written();
    let at = time_of(&w[0].1);
    assert!(before <= at && at <= after, "{at}");
    let want = format!(
        "相談の処分 = cw3-1・採否 = 一部採る・理由 = 前提の 1 つが古い・保存 = work/report.md・不保存 = work/probe.py・時刻 = {at}"
    );
    assert_eq!(w, [("fx-hub.5".to_string(), want)]);
    assert!(fx.ws.is_dir(), "閉じていない窓は退かせない");
    fx.settle(&[]);
    let again = fx.tz(&[&DISPOSE[..], &["work/report.md", "--drop", "work/probe.py"]].concat());
    assert_eq!(rc(&again), 1);
    assert!(err(&again).contains("もう処分した"), "{}", err(&again));
}

/// 整えて空になる理由と、写し先が在る時の断り（どれも写さず書かない）。
fn refusals_reason_and_kept(fx: &Fx) {
    let base = ["dispose", "cw3-1", "--verdict", "採る", "--reason", "理由"];
    let bad_reason = [
        "dispose",
        "cw3-1",
        "--verdict",
        "採る",
        "--reason",
        " ",
        "--keep",
        "work/report.md",
        "--drop",
        "work/probe.py",
    ];
    assert_eq!(rc(&fx.tz(&bad_reason)), 1);
    let kept = fx.repo.join("docs/consult/kept/cw3-1/work/report.md");
    fs::create_dir_all(kept.parent().unwrap()).unwrap();
    fs::write(&kept, "前の写し\n").unwrap();
    let o = fx.tz(&[
        &base[..],
        &["--keep", "work/report.md", "--drop", "work/probe.py"],
    ]
    .concat());
    assert_eq!(rc(&o), 1);
    assert!(err(&o).contains("もう在る"), "{}", err(&o));
    assert_eq!(fs::read_to_string(&kept).unwrap(), "前の写し\n");
    assert!(fx.written().is_empty());
}

#[test]
fn cwshw_dispose_refusals() {
    let fx = Fx::new("refuse");
    // ほかの所見 cw3-2 の受けの行だけが在る台帳（cw3-1 の受けの欠けだけの見本）。
    fx.notes(&["相談の受け = cw3-2・経路 = 一覧・時刻 = 20261003T1500Z".to_string()]);
    let unreceived =
        fx.tz(&[&DISPOSE[..], &["work/report.md", "--drop", "work/probe.py"]].concat());
    assert!(
        err(&unreceived).contains("まだ受けていない"),
        "{}",
        err(&unreceived)
    );
    assert_eq!(rc(&fx.tz(&["show", "cw3-1", "--via", "見張り"])), 0);
    fx.settle(&[]);
    let base = ["dispose", "cw3-1", "--verdict", "採る", "--reason", "理由"];
    let kd = "--keep work/report.md --drop work/probe.py";
    let file = fx.root.join("ledger.json").display().to_string();
    let cases = [
        (
            "--keep work/report.md".to_string(),
            "選びの無いか重なった添え物 work/probe.py",
        ),
        (
            format!("{kd} --drop work/x.md"),
            "所見に無い添え物 work/x.md",
        ),
        (
            format!("{kd} --drop work/report.md"),
            "重なった添え物 work/report.md",
        ),
        (format!("{kd} --into /nonexistent-into"), "dir でない"),
        (format!("{kd} --into {file}"), "dir でない"),
        (format!("{kd} --verdict x"), "2 度ある"),
        (format!("{kd} --reason y"), "2 度ある"),
        (format!("{kd} --x 1"), "知らない引数"),
    ];
    for (extra, want) in cases {
        let extra: Vec<&str> = extra.split_whitespace().collect();
        let o = fx.tz(&[&base[..], &extra].concat());
        assert_eq!(rc(&o), 1, "{extra:?}");
        assert!(err(&o).contains(want), "{want}: {}", err(&o));
    }
    refusals_reason_and_kept(&fx);
}

/// 所見を、添え物の全部を不保存にして処分する（採らない・理由 古い）。
fn drop_all(fx: &Fx, id: &str) -> Output {
    fx.tz(&[
        "dispose",
        id,
        "--verdict",
        "採らない",
        "--reason",
        "古い",
        "--drop",
        "work/report.md",
        "--drop",
        "work/probe.py",
    ])
}

#[test]
fn cwshw_retire_after_last() {
    let fx = Fx::new("retire");
    let first = fs::read_to_string(fx.ws.join("findings/cw3-1.json")).unwrap();
    for id in ["cw3-2", "cw3-3"] {
        let path = fx.ws.join(format!("findings/{id}.json"));
        fs::write(path, first.replace("cw3-1", id)).unwrap();
    }
    for id in ["cw3-1", "cw3-3"] {
        assert_eq!(rc(&fx.tz(&["show", id, "--via", "見張り"])), 0);
    }
    let closed = "相談の閉じ = cw3・起こし手 = 席・所見 = 3・時刻 = 20261003T1800Z".to_string();
    fx.settle(std::slice::from_ref(&closed));
    let args = [
        "--keep",
        "work/report.md",
        "--drop",
        "work/probe.py",
        "--into",
    ];
    let into = fx.root.join("into").display().to_string();
    let o = fx.tz(&[&DISPOSE[..], &args[1..], &[into.as_str()]].concat());
    assert_eq!(rc(&o), 0, "{}", err(&o));
    assert!(fx.ws.is_dir(), "処分の無い所見 cw3-2 が残る間は退かせない");
    fx.settle(&[]);
    let o = drop_all(&fx, "cw3-3");
    assert_eq!(rc(&o), 0, "{}", err(&o));
    assert!(
        fx.ws.is_dir(),
        "id の最後の cw3-3 を処分しても cw3-2 が残る間は退かせない"
    );
    assert_eq!(rc(&fx.tz(&["show", "cw3-2", "--via", "一覧"])), 0);
    fx.settle(&[]);
    let o = drop_all(&fx, "cw3-2");
    assert_eq!(rc(&o), 0, "{}", err(&o));
    let gone = fx.drafts.join("retired-consult-cw3");
    assert_eq!(
        out(&o),
        format!("処分を記した\n退いた: {}\n", gone.display())
    );
    assert!(!fx.ws.exists() && gone.join("findings/cw3-2.json").is_file());
}
