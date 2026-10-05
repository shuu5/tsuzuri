//! account board の host の側の部分の歯（便 e-acct-host・接頭辞 acctcore_）。
//! fixture: tests/fixtures/account/acct-inputs.json（host の側の字と、期待の口座の列・群の枠の列・移動の列）。
#![cfg(test)]

use std::path::{Path, PathBuf};

use crate::common::{FIXTURE, Inputs, known};
use tsuzuri_contract::account::{AccountRow, GroupMember, MoveRow};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::seat::QuotaUsed;
use tsuzuri_core::account::host::{HostTexts, accounts, declaration, groups, moves};
use tsuzuri_core::account::project_name;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn root() -> PathBuf {
    crate_dir().join("../..")
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{} を読む: {e}", path.display()))
}

fn inputs() -> Inputs {
    serde_json::from_str(&read(&root().join(FIXTURE))).expect("fixture の形")
}

fn texts() -> HostTexts {
    inputs().texts
}

fn used(window: &str, pct: u8, resets: Option<u64>) -> QuotaUsed {
    QuotaUsed {
        window: window.into(),
        used_pct: pct,
        resets_at: resets,
        counted: true,
    }
}

fn mv(at: u64, group: &str, from: Option<&str>, to: &str) -> MoveRow {
    MoveRow {
        at,
        group: group.into(),
        from: from.map(str::to_string),
        to: to.into(),
    }
}

/// 字の一部を置き換えた字。
fn edit(text: &Option<String>, from: &str, to: &str) -> Option<String> {
    let t = text.as_deref().expect("字");
    assert!(t.contains(from), "{from} が字に無い");
    Some(t.replace(from, to))
}

#[test]
fn acctcore_fixture_builds_expected() {
    let i = inputs();
    assert_eq!(accounts(&i.texts), i.accounts);
    assert_eq!(groups(&i.texts), i.groups);
    assert_eq!(moves(&i.texts), i.moves);
    // 期待の値は 3 つとも読める値で、どれも空でない。
    assert!(!known(i.accounts).is_empty());
    assert!(!known(i.groups).is_empty());
    assert!(!known(i.moves).is_empty());
}

#[test]
fn acctcore_declaration_reads_two_tables() {
    let t = texts();
    let d = declaration(t.host_toml.as_deref().expect("宣言"));
    // ほかの表（[defaults]・[seat]）の欄は読み捨てる。
    assert_eq!(d.accounts, ["acct-1", "acct-2", "acct-3", "acct-4"]);
    let names: Vec<&str> = d.groups.iter().map(|g| g.name.as_str()).collect();
    assert_eq!(names, ["main", "aux"]);
    assert_eq!(
        d.groups[0].anchors.as_deref(),
        Some(&["/work/proj-a".to_string(), "/work/proj-c".to_string()][..])
    );
    // 行をまたぐ配列と注釈も読む。
    assert_eq!(
        d.groups[1].anchors.as_deref(),
        Some(&["/work/proj-b".to_string()][..])
    );
    assert_eq!(
        d.groups[1].accounts.as_deref(),
        Some(&["acct-2".to_string(), "acct-3".to_string()][..])
    );
    // 読めない配列は None・欄が無ければ空の列・name の無い群と label の無い口座は数えない。
    let odd = "[[account]]\nplan = \"max\"\n[[account-group]]\nname = \"g\"\nanchors = [/work/proj-a,\n  \"/work/proj-b\"]\n[[account-group]]\nanchors = [\"/work/proj-c\"]\n[[account-group]]\nname = \"h\"\n";
    let d = declaration(odd);
    assert!(d.accounts.is_empty());
    assert_eq!(d.groups.len(), 2);
    assert_eq!(
        (d.groups[0].name.as_str(), &d.groups[0].anchors),
        ("g", &None)
    );
    assert_eq!(
        (d.groups[1].name.as_str(), d.groups[1].anchors.as_deref()),
        ("h", Some(&[][..]))
    );
    // 閉じの括弧の無い配列は読めない。
    let open = "[[account-group]]\nname = \"g\"\nanchors = [\"/work/proj-a\",\n";
    assert_eq!(declaration(open).groups[0].anchors, None);
    assert_eq!(project_name("/work/proj-b/"), "proj-b");
}

#[test]
fn acctcore_accounts_order_usage_and_model() {
    let t = texts();
    let rows = known(accounts(&t));
    let labels: Vec<&str> = rows.iter().map(|r| r.label.as_str()).collect();
    assert_eq!(labels, ["acct-1", "acct-2", "acct-3", "acct-4"]);
    assert_eq!(
        rows[0].usage,
        Reading::Known(vec![
            used("five_hour", 83, Some(1_790_521_200)),
            used("seven_day", 40, Some(1_790_812_800)),
            used("seven_day_model", 12, None),
        ])
    );
    assert_eq!(rows[0].model.as_deref(), Some("opus"));
    assert_eq!(rows[1].model.as_deref(), Some("sonnet"));
    // unmeasured の行と行の無い口座は usage だけが Unknown（ほかの欄は組む）。
    assert_eq!((&rows[2].usage, rows[2].retired), (&Reading::Unknown, true));
    assert_eq!(rows[3].usage, Reading::Unknown);
    // 宣言の順を入れ替えれば行の順も入れ替わる。
    let mut swapped = t.clone();
    swapped.host_toml = edit(&t.host_toml, "label = \"acct-1\"", "label = \"acct-0\"");
    swapped.host_toml = edit(
        &swapped.host_toml,
        "label = \"acct-4\"",
        "label = \"acct-1\"",
    );
    let labels: Vec<String> = known(accounts(&swapped))
        .into_iter()
        .map(|r| r.label)
        .collect();
    assert_eq!(labels, ["acct-0", "acct-2", "acct-3", "acct-1"]);
    // 形の読めない窓の行も usage だけが Unknown。
    let mut bad = t.clone();
    bad.usage = edit(&t.usage, "resets=2026-09-27T15:00:00Z", "resets=soon");
    let rows = known(accounts(&bad));
    assert_eq!(
        (&rows[0].usage, rows[0].occupant.as_deref()),
        (&Reading::Unknown, Some("main"))
    );
}

#[test]
fn acctcore_accounts_occupant_and_retired() {
    let t = texts();
    let rows = known(accounts(&t));
    let occ: Vec<Option<&str>> = rows.iter().map(|r| r.occupant.as_deref()).collect();
    assert_eq!(occ, [Some("main"), Some("aux"), None, None]);
    let retired: Vec<bool> = rows.iter().map(|r| r.retired).collect();
    assert_eq!(retired, [false, false, true, false]);
    // 今の口座が替われば占有も替わり、retired=yes の行が替われば退役も替わる。
    let mut moved = t.clone();
    moved.doctor = edit(&t.doctor, "current=acct-1", "current=acct-4");
    moved.doctor = edit(&moved.doctor, "acct-4 retired=no", "acct-4 retired=yes");
    let rows = known(accounts(&moved));
    let occ: Vec<Option<&str>> = rows.iter().map(|r| r.occupant.as_deref()).collect();
    assert_eq!(occ, [None, Some("aux"), None, Some("main")]);
    let retired: Vec<bool> = rows.iter().map(|r| r.retired).collect();
    assert_eq!(retired, [false, false, true, true]);
}

#[test]
fn acctcore_groups_from_doctor_and_record() {
    let t = texts();
    let cards = known(groups(&t));
    let main = &cards[0];
    assert_eq!(main.row.group, "main");
    assert_eq!(main.row.account, "acct-1");
    assert_eq!(main.row.candidates, ["acct-1", "acct-2", "acct-4"]);
    assert_eq!(main.row.next_account.as_deref(), Some("acct-2"));
    assert!(main.row.remaining.is_empty());
    assert_eq!(main.refused, None);
    assert_eq!(
        (main.since, main.previous.as_deref(), main.recorded),
        (Some(1_790_510_400), Some("acct-2"), true)
    );
    let aux = &cards[1];
    assert_eq!(aux.row.next_account, None);
    assert_eq!(aux.refused.as_deref(), Some("acct-3:retired"));
    // 今の記録が無い群は記録なし（history の記録は今の記録にしない）。
    assert_eq!(
        (aux.since, &aux.previous, aux.recorded),
        (None, &None, false)
    );
    // 宣言の順を入れ替えれば枠の順も入れ替わる。
    let mut swapped = t.clone();
    swapped.host_toml = edit(&t.host_toml, "name = \"main\"", "name = \"tmp\"");
    swapped.host_toml = edit(&swapped.host_toml, "name = \"aux\"", "name = \"main\"");
    swapped.host_toml = edit(&swapped.host_toml, "name = \"tmp\"", "name = \"aux\"");
    let names: Vec<String> = known(groups(&swapped))
        .into_iter()
        .map(|c| c.row.group)
        .collect();
    assert_eq!(names, ["aux", "main"]);
    // 記録が在っても previous が空なら前の口座は無し。
    let mut blank = t.clone();
    blank.records.insert(
        "aux.account".into(),
        "account=acct-2\nts=2026-09-27T10:00:00Z\nreason=manual\nprevious=\n".into(),
    );
    let aux = &known(groups(&blank))[1];
    assert_eq!(
        (aux.since, &aux.previous, aux.recorded),
        (Some(1_790_503_200), &None, true)
    );
    // 宣言の群の行が doctor に無ければ群の列は Unknown。
    let mut gone = t.clone();
    gone.doctor = edit(&t.doctor, "group=aux ", "group=other ");
    assert_eq!(groups(&gone), Reading::Unknown);
}

fn member(project: &str, seat: Option<&str>, matches: Reading<bool>) -> GroupMember {
    GroupMember {
        project: project.into(),
        seat_account: seat.map(str::to_string),
        matches,
    }
}

#[test]
fn acctcore_group_members_seat_account() {
    let t = texts();
    let cards = known(groups(&t));
    // anchors の配列の順。pipeline の席と別の anchor の席は数えない。末尾の「/」は同じ path。
    assert_eq!(
        cards[0].members,
        [
            member("proj-a", Some("acct-1"), Reading::Known(true)),
            member("proj-c", None, Reading::Unknown),
        ]
    );
    assert_eq!(
        cards[1].members,
        [member("proj-b", Some("acct-1"), Reading::Known(false))]
    );
    // anchors の順を入れ替えれば列の順も入れ替わる。
    let mut swapped = t.clone();
    swapped.host_toml = edit(
        &t.host_toml,
        "[\"/work/proj-a\", \"/work/proj-c\"]",
        "[\"/work/proj-c\", \"/work/proj-a\"]",
    );
    let projects: Vec<String> = known(groups(&swapped))[0]
        .members
        .iter()
        .map(|m| m.project.clone())
        .collect();
    assert_eq!(projects, ["proj-c", "proj-a"]);
    // 席の行が pipeline だけなら Unknown・orchestrator の行が複数なら最初の行。
    let mut pipe = t.clone();
    pipe.seat_doctors.insert(
        "/work/proj-b".into(),
        "seat: role=pipeline anchor=/work/proj-b target=proj-b:1.1 account=acct-2 model=opus\n"
            .into(),
    );
    assert_eq!(
        known(groups(&pipe))[1].members,
        [member("proj-b", None, Reading::Unknown)]
    );
    let mut two = t.clone();
    two.seat_doctors.insert(
        "/work/proj-b".into(),
        "seat: role=orchestrator anchor=/work/proj-b target=proj-b:0.1 account=acct-2 model=opus\nseat: role=orchestrator anchor=/work/proj-b target=proj-b:0.2 account=acct-1 model=opus\n"
            .into(),
    );
    assert_eq!(
        known(groups(&two))[1].members,
        [member("proj-b", Some("acct-2"), Reading::Known(true))]
    );
    // 今の口座が替われば一致も替わる。
    let mut moved = t.clone();
    moved.doctor = edit(&t.doctor, "current=acct-1", "current=acct-2");
    assert_eq!(
        known(groups(&moved))[0].members[0].matches,
        Reading::Known(false)
    );
}

#[test]
fn acctcore_moves_newest_first() {
    let t = texts();
    // request・refused・宣言に無い群・ts の無い記録は読まない。同じ ts は宣言の順（main が先）。
    assert_eq!(
        known(moves(&t)),
        [
            mv(1_790_510_400, "main", Some("acct-2"), "acct-1"),
            mv(1_790_503_200, "main", Some("acct-1"), "acct-2"),
            mv(1_790_503_200, "aux", None, "acct-2"),
            mv(1_790_409_600, "main", None, "acct-1"),
        ]
    );
    // 宣言の順を入れ替えれば同じ ts の順も入れ替わる。
    let mut swapped = t.clone();
    swapped.host_toml = edit(&t.host_toml, "name = \"main\"", "name = \"tmp\"");
    swapped.host_toml = edit(&swapped.host_toml, "name = \"aux\"", "name = \"main\"");
    swapped.host_toml = edit(&swapped.host_toml, "name = \"tmp\"", "name = \"aux\"");
    let tie: Vec<String> = known(moves(&swapped))[1..3]
        .iter()
        .map(|m| m.group.clone())
        .collect();
    assert_eq!(tie, ["aux", "main"]);
    // 時刻の窓で絞らない（ずっと前の記録も入る）。account の無い記録と読めない ts の記録は入らない。
    let mut more = t.clone();
    more.history.insert(
        "aux.account.20200101T000000Z.1".into(),
        "account=acct-3\nts=2020-01-01T00:00:00Z\nreason=initial\n".into(),
    );
    more.history.insert(
        "aux.account.20200102T000000Z.1".into(),
        "ts=2020-01-02T00:00:00Z\nreason=broken\nprevious=acct-3\n".into(),
    );
    more.history.insert(
        "aux.account.20200103T000000Z.1".into(),
        "account=acct-2\nts=yesterday\nreason=broken\n".into(),
    );
    let all = known(moves(&more));
    assert_eq!(all.len(), 5);
    assert_eq!(all[4], mv(1_577_836_800, "aux", None, "acct-3"));
}

#[test]
fn acctcore_unread_texts_touch_only_their_parts() {
    let full = inputs();
    let t = &full.texts;
    // host の doctor の字が無ければ口座の列と群の列が Unknown・移動の列は組む。
    let mut no_doctor = t.clone();
    no_doctor.doctor = None;
    assert_eq!(accounts(&no_doctor), Reading::Unknown);
    assert_eq!(groups(&no_doctor), Reading::Unknown);
    assert_eq!(moves(&no_doctor), full.moves);
    // usage の字が無ければ口座の列の usage と model だけが無い。
    let mut no_usage = t.clone();
    no_usage.usage = None;
    let rows = known(accounts(&no_usage));
    let want: Vec<AccountRow> = known(full.accounts.clone())
        .into_iter()
        .map(|mut r| {
            r.usage = Reading::Unknown;
            r.model = None;
            r
        })
        .collect();
    assert_eq!(rows, want);
    assert_eq!(groups(&no_usage), full.groups);
    assert_eq!(moves(&no_usage), full.moves);
    // 群の宣言の字が無ければ 3 つとも Unknown。
    let mut no_host = t.clone();
    no_host.host_toml = None;
    assert_eq!(accounts(&no_host), Reading::Unknown);
    assert_eq!(groups(&no_host), Reading::Unknown);
    assert_eq!(moves(&no_host), Reading::Unknown);
    unread_seat(&full, t);
}

/// anchor の doctor の字と記録が無い時の口座・移動・群の列。
fn unread_seat(full: &Inputs, t: &HostTexts) {
    // anchor の doctor の字と記録が無ければ、その部分だけが替わる。
    let mut no_seat = t.clone();
    no_seat.seat_doctors.clear();
    no_seat.records.clear();
    no_seat.history.clear();
    assert_eq!(accounts(&no_seat), full.accounts);
    assert_eq!(moves(&no_seat), Reading::Known(vec![]));
    for card in known(groups(&no_seat)) {
        assert_eq!(
            (card.since, &card.previous, card.recorded),
            (None, &None, false)
        );
        for m in card.members {
            assert_eq!((m.seat_account, m.matches), (None, Reading::Unknown));
        }
    }
}

/// 字の中の口座の名（`鍵=値` の値をコンマで割ったもの・none と - と空は除く）。
fn account_names(text: &str, out: &mut Vec<String>) {
    for token in text.split_whitespace() {
        let Some((key, value)) = token.split_once('=') else {
            continue;
        };
        if matches!(
            key,
            "account" | "accounts" | "seat-accounts" | "current" | "next" | "previous"
        ) {
            out.extend(
                value
                    .split(',')
                    .filter(|v| !matches!(*v, "" | "none" | "-"))
                    .map(str::to_string),
            );
        }
    }
}

#[test]
fn acctcore_fixture_names_and_size() {
    let text = read(&root().join(FIXTURE));
    assert!(text.len() <= 20_000, "fixture は {} byte", text.len());
    let t = texts();
    let host = t.host_toml.as_deref().expect("宣言");
    let d = declaration(host);
    let mut names: Vec<String> = d.accounts.clone();
    let mut anchors: Vec<String> = Vec::new();
    for g in &d.groups {
        names.extend(g.accounts.clone().expect("accounts"));
        anchors.extend(g.anchors.clone().expect("anchors"));
    }
    // 宣言の中の label の行は全部読めている。
    let label_lines = host
        .lines()
        .filter(|l| l.trim_start().starts_with("label"))
        .count();
    assert_eq!(label_lines, d.accounts.len());
    let texts = [&t.usage, &t.doctor]
        .into_iter()
        .flatten()
        .chain(t.seat_doctors.values())
        .chain(t.records.values())
        .chain(t.history.values());
    for x in texts {
        account_names(x, &mut names);
        for token in x.split_whitespace() {
            if let Some(a) = token.strip_prefix("anchor=") {
                anchors.push(a.to_string());
            }
        }
    }
    anchors.extend(t.seat_doctors.keys().cloned());
    assert!(names.len() > 10 && anchors.len() > 3);
    for n in &names {
        let digits = n
            .strip_prefix("acct-")
            .unwrap_or_else(|| panic!("口座の名 {n}"));
        assert!(
            !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()),
            "口座の名 {n}"
        );
    }
    for a in &anchors {
        assert!(a.starts_with("/work/"), "anchor の path {a}");
    }
    assert!(!text.contains("/home/"), "home の path");
}

/// dir の下の全部の `.rs` の file（path の順）。
fn rs_files(dir: &Path) -> Vec<PathBuf> {
    let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)
        .expect("dir を読む")
        .map(|e| e.expect("dir の中身").path())
        .collect();
    entries.sort();
    let mut out = Vec::new();
    for p in entries {
        if p.is_dir() {
            out.extend(rs_files(&p));
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
    out
}

#[test]
fn acctcore_pure_and_no_new_dependencies() {
    let files = rs_files(&crate_dir().join("src/account"));
    assert!(files.iter().any(|f| f.ends_with("host.rs")));
    assert!(files.iter().any(|f| f.ends_with("mod.rs")));
    for f in &files {
        let text = read(f);
        for word in ["std::fs", "std::process", "SystemTime"] {
            assert!(!text.contains(word), "{} に {word}", f.display());
        }
    }
    let manifest = read(&crate_dir().join("Cargo.toml"));
    let deps: Vec<&str> = manifest
        .split("[dependencies]")
        .nth(1)
        .expect("[dependencies] の節")
        .lines()
        .map(str::trim)
        .take_while(|l| !l.starts_with('['))
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .filter_map(|l| l.split(['.', '=', ' ']).next())
        .collect();
    assert_eq!(deps, ["serde", "serde_json", "tsuzuri-contract"]);
    assert!(!manifest.contains("[dev-dependencies]"));
}
