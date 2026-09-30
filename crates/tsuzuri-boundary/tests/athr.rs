//! 窓ごとの逼迫の閾値と群の知らせと断りの読みの歯（設計ノート surface-wave7 便 c-acct-thr・接頭辞 athr_）。
//! 偽の器は受けた argv を記録の置き場に 1 行ずつ足し、頭が rules なら `rules-<3 つ目の argv>`、頭が fleet なら
//! `usage` を作業場の out の下から出す（slow の下に同じ名の印が在れば 8 秒眠る・file が無ければ rc 1・ほかの頭は rc 1）。
//! 偽の git は -C の次の path の最後の区切りと鍵の組で作業場の git の下の file を出す（file が無ければ rc 1）。
//! 字の中の path は実行の時に組む（行 D-4）。今は 2026-09-27T12:00:00Z。
#![cfg(test)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use tsuzuri_boundary::acct::{Acct, CAP_ARGS};
use tsuzuri_contract::account::GroupNotice;
use tsuzuri_contract::board::Reading;
use tsuzuri_core::account::host::CAP_ROWS;

/// 読みの今の時刻（2026-09-27T12:00:00Z）。
const NOW: u64 = 1_790_510_400;

const EVENTS_A: &str = "{\"schema\":1,\"ts\":\"2026-09-27T11:00:00Z\",\"kind\":\"RunCreated\",\"run\":\"r.1-20260927T110000Z\",\"bead\":\"r.1\",\"actor\":\"machine\",\"stage\":\"Intake\"}\n\
{\"schema\":1,\"ts\":\"2026-09-27T11:30:00Z\",\"kind\":\"GroupPressureNotified\",\"account\":\"acct-2\",\"actor\":\"machine\",\"detail\":\"group=g-a window=5h used=90 cap=80 sent=1\"}\n\
{\"schema\":1,\"ts\":\"2026-09-27T11:40:00Z\",\"kind\":\"GroupMoveRefused\",\"account\":\"acct-3\",\"actor\":\"machine\",\"detail\":\"group=g-b reason=no-candidate\"}\n\
{\"schema\":1,\"ts\":\"2026-09-27T11:45:00Z\",\"kind\":\"GroupPressureNotified\",\"account\":\"acct-1\",\"actor\":\"machine\",\"detail\":\"group=g-z window=7d used=99 cap=90 sent=1\"}\n";
const EVENTS_C: &str = "{\"schema\":1,\"ts\":\"2026-09-27T11:50:00Z\",\"kind\":\"GroupMoveRefused\",\"account\":\"acct-1\",\"actor\":\"machine\",\"detail\":\"group=g-a reason=no-candidate\"}\n";

fn script(path: &Path, body: &str) {
    fs::write(path, format!("#!/bin/sh\n{body}\n")).expect("偽の program");
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("偽の program の権限");
}

/// 歯ごとの作業場。
struct Place {
    root: PathBuf,
}

impl Place {
    /// 作業場 W（`slow` なら model の行の出力を足し、残量と 5h と 7d の行を眠らせる）。
    fn new(name: &str, slow: bool) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join("athr").join(name);
        let _ = fs::remove_dir_all(&root);
        for dir in ["bin", "out", "git", "slow", "log", "repo", "work/proj-a", "work/proj-b", "work/proj-c"] {
            fs::create_dir_all(root.join(dir)).expect("置き場");
        }
        let place = Place { root };
        let root = place.root.display().to_string();
        script(
            &place.root.join("bin/scribe2"),
            &"printf '%s\\n' \"$*\" >> 'ROOT/log/scribe2'\n\
              case \"$1\" in rules) f=\"rules-$3\" ;; fleet) f=usage ;; *) exit 1 ;; esac\n\
              if [ -e 'ROOT/slow/'\"$f\" ]; then exec sleep 8; fi\n\
              exec cat 'ROOT/out/'\"$f\""
                .replace("ROOT", &root),
        );
        script(
            &place.root.join("bin/git"),
            &"printf '%s\\n' \"$*\" >> 'ROOT/log/git'\n\
              f=\"${2%/}\"; f=\"${f##*/}\"\n\
              exec cat 'ROOT/git/'\"$f.$5\""
                .replace("ROOT", &root),
        );
        let [a, b, c] = ["proj-a", "proj-b", "proj-c"].map(|p| place.root.join("work").join(p).display().to_string());
        place.put(
            "states/state-h/host.toml",
            &format!(
                "[[account-group]]\nname = \"g-a\"\nanchors = [\"{a}\", \"{b}\"]\n\n\
                 [[account-group]]\nname = \"g-b\"\nanchors = [\"{c}\"]\n"
            ),
        );
        let (sa, sc) = (place.root.join("states/state-a"), place.root.join("states/state-c"));
        place.put("git/proj-a.scribe2.statedir", &format!("{}\n", sa.display()));
        place.put("git/proj-b.scribe2.statedir", &format!("{}\n", sa.display()));
        place.put("git/proj-c.scribe2.statedir", &format!("{}\n", sc.display()));
        place.put("states/state-a/fleet/events.jsonl", EVENTS_A);
        place.put("states/state-c/fleet/events.jsonl", EVENTS_C);
        place.put("out/rules-fleet.group_pressure_5h_pct", "80\n");
        place.put("out/rules-fleet.group_pressure_7d_pct", "  90  \n");
        if slow {
            place.put("out/rules-fleet.group_pressure_model_pct", "95\n");
            for mark in ["usage", "rules-fleet.group_pressure_5h_pct", "rules-fleet.group_pressure_7d_pct"] {
                place.put(&format!("slow/{mark}"), "");
            }
        }
        place
    }

    fn put(&self, rel: &str, text: &str) {
        let path = self.root.join(rel);
        fs::create_dir_all(path.parent().expect("親")).expect("親の dir");
        fs::write(&path, text).expect("file を置く");
    }

    fn acct(&self) -> Acct {
        Acct::new(
            self.root.join("bin/scribe2"),
            self.root.join("bin/git"),
            "/nonexistent/tz-no-such-bd",
            self.root.join("states/state-h"),
            self.root.join("repo"),
        )
    }

    /// 偽の器が受けた argv（1 回 1 行・名の順）。
    fn calls(&self) -> Vec<String> {
        let mut calls: Vec<String> = fs::read_to_string(self.root.join("log/scribe2"))
            .map(|t| t.lines().map(str::to_string).collect())
            .unwrap_or_default();
        calls.sort();
        calls
    }
}

/// 閾値の列の値だけ（CAP_ROWS の順の窓と行の id を見たうえで）。
fn cap_values(doc: &tsuzuri_contract::account::AccountDoc) -> Vec<Reading<u64>> {
    let rows: Vec<(&str, &str)> = doc
        .caps
        .iter()
        .map(|c| (c.window.as_str(), c.rule.as_str()))
        .collect();
    assert_eq!(rows, CAP_ROWS);
    doc.caps.iter().map(|c| c.cap.clone()).collect()
}

#[test]
fn athr_reads_rows_once_first_round() {
    let place = Place::new("once", false);
    let acct = place.acct();
    let first = acct.doc(NOW);
    let second = acct.doc(NOW);
    assert_eq!(first, second);
    assert_eq!(CAP_ARGS, ["rules", "get"]);
    let rules: Vec<String> = place
        .calls()
        .into_iter()
        .filter(|c| c.starts_with("rules"))
        .collect();
    // 猶予の rules 行は撃たない（残り秒は席の card の器の欄の写し・行 c-grace-acct）。
    let mut want: Vec<String> = CAP_ROWS
        .iter()
        .map(|(_, rule)| format!("{} {rule}", CAP_ARGS.join(" ")))
        .collect();
    want.sort();
    assert_eq!(rules, want, "行ごとに 1 回・--state-dir を付けない・5 秒の内に増えない");
}

#[test]
fn athr_caps_from_vessel_output() {
    let place = Place::new("caps", false);
    let doc = place.acct().doc(NOW);
    assert_eq!(
        cap_values(&doc),
        [Reading::Known(80), Reading::Known(90), Reading::Unknown]
    );
}

#[test]
fn athr_notices_from_state_logs() {
    let place = Place::new("notices", false);
    let doc = place.acct().doc(NOW);
    let want = vec![
        GroupNotice::Refused {
            at: 1_790_509_800,
            group: "g-a".into(),
            account: "acct-1".into(),
            reason: "no-candidate".into(),
        },
        GroupNotice::Refused {
            at: 1_790_509_200,
            group: "g-b".into(),
            account: "acct-3".into(),
            reason: "no-candidate".into(),
        },
        GroupNotice::Pressure {
            at: 1_790_508_600,
            group: "g-a".into(),
            account: "acct-2".into(),
            window: "five_hour".into(),
            used: 90,
            cap: 80,
            sent: 1,
        },
    ];
    assert_eq!(doc.notices, Reading::Known(want));
}

#[test]
fn athr_slow_row_is_unknown() {
    let place = Place::new("slow", true);
    let from = Instant::now();
    let doc = place.acct().doc(NOW);
    let took = from.elapsed();
    assert!(took >= Duration::from_secs(4), "5 秒待たずに返る: {took:?}");
    assert!(took < Duration::from_secs(8), "1 段目に並ばない: {took:?}");
    assert_eq!(
        cap_values(&doc),
        [Reading::Unknown, Reading::Unknown, Reading::Known(95)]
    );
}

/// 着地済みの filter の語（歯の名から先頭の athr_ を除いた字はどれも含まない）。
const WORDS: [&str; 92] = [
    "accept_", "account_", "acctcore_", "acctdoc_", "acctframe_", "accthb_", "accthome_",
    "acctled_", "acctlook_", "acctpcore_", "acctproj_", "acctsess_", "acctwin_", "acctwire_",
    "askcard_", "batchpanel_", "board_min_", "contract_form_", "frame_", "gapspage_",
    "gquestion_", "graph_", "gview_", "hbconf_", "hbpost_", "hbproc_", "hbroute_", "hook_",
    "hsblock_", "hsderive_", "hspage_", "ledgerblock_", "mapgraph_", "mapview_", "nextstep_",
    "nodepage_", "parts_", "pipe_", "project_", "question_", "seatblock_", "seatcard_",
    "server_", "skeleton_", "stage_", "stats_", "steady_", "topbar_", "tz_", "mlink_", "gnav_",
    "mkeys_", "klink_", "ilink_", "afocus_", "nxact_", "urpanel_", "saxis_", "ticker_", "hfig_",
    "pmore_", "lspark_", "apop_", "brand_", "runsdoc_", "nbatch_", "kcli_", "hcard_", "qgate_",
    "nsum_", "ntime_", "ptitle_", "ncard_", "hsym_", "smore_", "lcard_", "aaround_", "hruling_",
    "sxaxis_", "plimit_", "bport_", "fmark_", "fstop_", "fserve_", "nsumw_", "cadopt_", "tipx_",
    "hcsess_", "hcproj_", "sesplit_", "mstore_", "qblock_",
];

#[test]
fn athr_own_names_clean() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/athr.rs");
    let text = fs::read_to_string(&path).expect("自分の file");
    let mut lines = text.lines().map(str::trim);
    let mut names = Vec::new();
    while let Some(line) = lines.next() {
        if line != "#[test]" {
            continue;
        }
        let f = lines.next().expect("属性の次の行");
        let name = f
            .strip_prefix("fn ")
            .and_then(|r| r.split_once('('))
            .map(|(n, _)| n)
            .unwrap_or_else(|| panic!("属性の次が fn でない: {f}"));
        names.push(name.to_string());
    }
    assert_eq!(names.len(), 5, "{names:?}");
    for name in &names {
        let rest = name
            .strip_prefix("athr_")
            .unwrap_or_else(|| panic!("接頭辞: {name}"));
        for w in WORDS {
            assert!(!rest.contains(w), "{name} が {w} を含む");
        }
    }
}
