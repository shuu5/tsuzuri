//! memo の起票の門の歯（接頭辞 mlnk_・設計ノート surface-v4a 行 t-memo-link-gate・判断の記録 ADR-72 の決定 (5)）。
//! 純関数の歯は fixture の字（索引は FR9・FR8・FR7・P-9・ADR-9・ADR-8・NFR9・nx#r1・nx#r2・R-1 から R-31、台帳は fx-m の木）から
//! 中核の `graph::build` でグラフを組んで `memo_gate` の関数を直に撃つ。口の歯は tz hook question-gate を偽の bd と偽の設計の道具
//! （撃たれた引数を記録の file に 1 行足してから作業場の字を出す script）で撃つ。作業場は CARGO_TARGET_TMPDIR の下の mlnk/<歯の名>。
#![cfg(test)]

use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use tsuzuri_boundary::server::design::{DESIGN_DIR, FOLIO_ARGS};
use tsuzuri_boundary::server::ledger::BD_ARGS;
use tsuzuri_core::gate;
use tsuzuri_core::graph::{Graph, Inputs, build};
use tsuzuri_core::memo_gate::{
    self, CANDIDATE_KINDS, MemoDraft, MemoGate, MemoWhy, memo_digest, memo_needs,
};

/// 索引の字（節点は id・種類・file・要約値・題、辺は端・端・型のタブ区切り）。
/// FR9 の隣は P-9・ADR-9・ADR-8・NFR9・nx#r1、ADR-8 の隣は FR9 と R-1 から R-31 の 32 で hub。
fn index() -> String {
    let mut lines: Vec<String> = Vec::new();
    let mut nodes: Vec<(String, &str, &str)> = vec![
        ("FR9".into(), "要件", "srs.yaml"),
        ("FR8".into(), "要件", "srs.yaml"),
        ("FR7".into(), "要件", "srs.yaml"),
        ("P-9".into(), "条", "constitution.yaml"),
        ("ADR-9".into(), "判断の記録", "adr/ADR-9.yaml"),
        ("ADR-8".into(), "判断の記録", "adr/ADR-8.yaml"),
        ("NFR9".into(), "非機能要件", "srs.yaml"),
        ("nx#r1".into(), "設計ノートの行", "design-note/nx.yaml"),
        ("nx#r2".into(), "設計ノートの行", "design-note/nx.yaml"),
    ];
    nodes.extend((1..=31).map(|n| (format!("R-{n}"), "規則行", "rules.yaml")));
    for (id, kind, file) in nodes {
        lines.push(format!("{id}\t{kind}\t{file}\td-{id}\t節点 {id}"));
    }
    let mut edges: Vec<(String, String, &str)> = vec![
        ("FR9".into(), "P-9".into(), "basis"),
        ("FR9".into(), "ADR-9".into(), "adrs"),
        ("FR9".into(), "ADR-8".into(), "adrs"),
        ("FR9".into(), "NFR9".into(), "refs"),
        ("nx#r1".into(), "FR9".into(), "req"),
        ("nx#r2".into(), "FR8".into(), "req"),
    ];
    edges.extend((1..=31).map(|n| ("ADR-8".to_string(), format!("R-{n}"), "rules")));
    for (from, to, kind) in edges {
        lines.push(format!("{from}\t{to}\t{kind}"));
    }
    lines.join("\n") + "\n"
}

/// 台帳の bead の 1 本の JSON の字（種類と状態の組・label と touches の組・acceptance の行 design は `design` が在れば 1 行）。
fn bead(
    id: &str,
    (kind, status): (&str, &str),
    (labels, touches): (&[&str], &[&str]),
    design: Option<&str>,
    title: &str,
) -> String {
    let quoted = |items: &[&str]| {
        items
            .iter()
            .map(|i| format!("\"{i}\""))
            .collect::<Vec<_>>()
            .join(",")
    };
    let (labels, touches) = (quoted(labels), quoted(touches));
    let acceptance = design
        .map(|d| format!(r#","acceptance_criteria":"design = {d}""#))
        .unwrap_or_default();
    format!(
        r#"{{"id":"{id}","title":"{title}","status":"{status}","issue_type":"{kind}","labels":[{labels}],"metadata":{{"touches":[{touches}]}}{acceptance}}}"#
    )
}

/// 台帳の字（bd の一覧の JSON の配列）。fx-m.3 と fx-m.4 の題と、FR7 を触る開いた memo fx-t.1 から fx-t.`crowd` の本数は引数。
fn ledger(m3_title: &str, m4_title: &str, crowd: usize) -> String {
    let (open, closed) = (("task", "open"), ("task", "closed"));
    let memo = |touches: &'static [&'static str]| (&["intake:memo"][..], touches);
    let (question, none) = ((&["intake:question"][..], &["FR9"][..]), (&[][..], &[][..]));
    let mut beads = vec![
        bead("fx-m", ("epic", "open"), none, None, "memo の木"),
        bead("fx-m.1", open, memo(&["FR9"]), None, "memo 1"),
        bead("fx-m.2", closed, memo(&["FR9"]), None, "memo 2"),
        bead("fx-m.3", open, memo(&["FR8"]), None, m3_title),
        bead("fx-m.4", open, question, None, m4_title),
        bead("fx-m.5", open, none, Some("contracts/nx.toml#r1"), "契約 5"),
        bead("fx-m.6", open, none, Some("contracts/nx.toml#r2"), "契約 6"),
        bead("fx-m.7", closed, none, Some("contracts/nx.toml#r1"), "契約 7"),
    ];
    beads.extend((1..=crowd).map(|n| {
        bead(&format!("fx-t.{n}"), open, memo(&["FR7"]), None, "群れの memo")
    }));
    format!("[{}]", beads.join(","))
}

fn graph_of(index: &str, ledger: &str) -> Graph {
    build(&Inputs {
        design_index: index,
        ledger,
        events: "",
    })
}

fn graph() -> Graph {
    graph_of(&index(), &ledger("題 3", "題 4", 0))
}

fn ids(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

/// metadata の JSON の字（not-relevant は id と理由の組・digest は在れば）。
fn meta(touches: &[&str], not_relevant: &[(&str, &str)], digest: Option<&str>) -> String {
    let touches = touches
        .iter()
        .map(|t| format!("\"{t}\""))
        .collect::<Vec<_>>()
        .join(",");
    let not_relevant = not_relevant
        .iter()
        .map(|(id, why)| format!("\"{id}\":\"{why}\""))
        .collect::<Vec<_>>()
        .join(",");
    let digest = digest
        .map(|d| format!(r#","digest":"{d}""#))
        .unwrap_or_default();
    format!(r#"{{"short":"門の歯","touches":[{touches}],"not-relevant":{{{not_relevant}}}{digest}}}"#)
}

fn draft(metadata: &str, links: &[&str]) -> MemoDraft {
    MemoDraft {
        metadata: Some(metadata.to_string()),
        links: ids(links),
    }
}

/// JSON の字の中身（二重引用符と逆斜線に逆斜線を付ける）。
fn escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Bash の PreToolUse の hook の入力の字。
fn payload(command: &str) -> String {
    format!(
        r#"{{"session_id":"s-1","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{{"command":"{}"}}}}"#,
        escape(command)
    )
}

/// memo の起票の command（--deps の値と metadata）。
fn memo_command(deps: &str, metadata: &str) -> String {
    format!(
        "bdw create --parent=fx-m --labels=intake:memo --deps {deps} --metadata='{metadata}' 門の歯の memo"
    )
}

#[test]
fn mlnk_drafts_pick_memo_creates() {
    let m = meta(&["FR9"], &[], None);
    let command = format!(
        "bdw create --parent=fx-m --labels=intake:memo --deps relates-to:fx-m.1,blocks:fx-m.5 --deps=discovered-from:fx-m.4 --metadata='{m}' 題"
    );
    assert_eq!(
        memo_gate::drafts(&payload(&command)),
        vec![MemoDraft {
            metadata: Some(m.clone()),
            links: ids(&["fx-m.1", "fx-m.4"]),
        }]
    );
    // 最後の --metadata の値・前後の空白と型の無い項と空の id は数えない・label はコンマ区切りと短い旗も読む。
    let two = "FOO=1 /usr/bin/bd create -lx,intake:memo --metadata '{\"a\":1}' --metadata='{\"b\":2}' --deps ' discovered-from: fx-m.9 ,fx-m.8,relates-to:,Relates-to:fx-m.7'";
    assert_eq!(
        memo_gate::drafts(&payload(two)),
        vec![MemoDraft {
            metadata: Some("{\"b\":2}".to_string()),
            links: ids(&["fx-m.9"]),
        }]
    );
    // 2 つの create は command の順に拾い、deps の無い memo は links が空。
    let pair = format!("{}; bd create --label intake:memo 別の memo", memo_command("relates-to:fx-m.1", &m));
    let found = memo_gate::drafts(&payload(&pair));
    assert_eq!(found.len(), 2);
    assert_eq!(found[0].links, ids(&["fx-m.1"]));
    assert_eq!(found[1], MemoDraft { metadata: None, links: Vec::new() });

    let edit = format!(
        r#"{{"tool_name":"Edit","tool_input":{{"command":"{}"}}}}"#,
        escape(&command)
    );
    for (name, input) in [
        ("問いの create", payload("bdw create --labels=intake:question --deps relates-to:fx-m.1")),
        ("update", payload("bdw update fx-m.1 --add-label=intake:memo --deps relates-to:fx-m.2")),
        ("echo の後の create", payload("echo bdw create --labels=intake:memo")),
        ("label の無い create", payload("bdw create --deps relates-to:fx-m.1 題")),
        ("Edit", edit),
        ("JSON でない", "not json".to_string()),
    ] {
        assert!(memo_gate::drafts(&input).is_empty(), "{name}: {input}");
    }
}

#[test]
fn mlnk_needs_name_the_neighbours() {
    let g = graph();
    let want = ids(&["ADR-9", "P-9", "fx-m.1", "fx-m.2", "fx-m.4", "fx-m.5", "nx#r1"]);
    assert_eq!(memo_needs(&g, &ids(&["FR9"])), want);
    // 種類の外（NFR9）・hub（ADR-8）・辺の無い memo と別の行の契約・閉じた契約は入らない。
    for out in ["NFR9", "ADR-8", "fx-m.3", "fx-m.6", "fx-m.7", "FR9"] {
        assert!(!want.contains(&out.to_string()), "{out}");
    }
    assert_eq!(CANDIDATE_KINDS.len(), 8);
    assert!(memo_needs(&g, &ids(&["FR77"])).is_empty());
}

#[test]
fn mlnk_digest_follows_the_candidates() {
    let touches = ids(&["FR9"]);
    let base = memo_digest(&graph(), &touches);
    assert_eq!(base.len(), 16, "{base}");
    assert!(base.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f')), "{base}");
    assert_eq!(base, memo_digest(&graph(), &touches), "同じ入力は同じ値");
    let candidate = graph_of(&index(), &ledger("題 3", "別の題 4", 0));
    assert_ne!(memo_digest(&candidate, &touches), base, "候補 fx-m.4 の題");
    let other = graph_of(&index(), &ledger("別の題 3", "題 4", 0));
    assert_eq!(memo_digest(&other, &touches), base, "候補でない fx-m.3 の題");
}

#[test]
fn mlnk_judge_refuses_bad_drafts() {
    let g = graph();
    let deny = |why: MemoWhy, found: &[&str]| MemoGate::Deny {
        why,
        ids: ids(found),
        digest: None,
    };
    for text in ["[1]", "not json", "\"s\""] {
        assert_eq!(
            memo_gate::judge(&[draft(text, &[])], &g),
            deny(MemoWhy::Metadata, &[]),
            "{text}"
        );
    }
    for text in [r#"{"short":"門の歯"}"#, r#"{"touches":[]}"#] {
        assert_eq!(
            memo_gate::judge(&[draft(text, &[])], &g),
            deny(MemoWhy::NoTouches, &[]),
            "{text}"
        );
    }
    assert_eq!(
        memo_gate::judge(&[MemoDraft { metadata: None, links: Vec::new() }], &g),
        deny(MemoWhy::NoTouches, &[])
    );
    let unknown = draft(&meta(&["FR9", "FR77"], &[], None), &[]);
    assert_eq!(memo_gate::judge(&[unknown], &g), deny(MemoWhy::UnknownId, &["FR77"]));

    // 読めない出所は まだ分からない（台帳が空・索引が空）。
    let fr9 = draft(&meta(&["FR9"], &[], None), &[]);
    for (g, id) in [
        (graph_of(&index(), ""), "ledger"),
        (graph_of("", &ledger("題 3", "題 4", 0)), "design"),
    ] {
        assert_eq!(
            memo_gate::judge(std::slice::from_ref(&fr9), &g),
            MemoGate::Unknown { why: MemoWhy::Unread, ids: ids(&[id]), digest: None },
            "{id}"
        );
    }
    // 31 本の開いた memo が触る FR7 だけの下書きは まだ分からない。
    let crowded = graph_of(&index(), &ledger("題 3", "題 4", 31));
    let fr7 = ids(&["FR7"]);
    let want: Vec<String> = {
        let mut all: Vec<String> = (1..=31).map(|n| format!("fx-t.{n}")).collect();
        all.sort_by(|a, b| tsuzuri_contract::graph::natural_cmp(a, b));
        all
    };
    assert_eq!(memo_needs(&crowded, &fr7), want);
    assert_eq!(
        memo_gate::judge(&[draft(&meta(&["FR7"], &[], None), &[])], &crowded),
        MemoGate::Unknown {
            why: MemoWhy::TooMany,
            ids: want,
            digest: Some(memo_digest(&crowded, &fr7)),
        }
    );
    // 30 本ちょうどは数の上限を越えない。
    let edge = graph_of(&index(), &ledger("題 3", "題 4", 30));
    assert!(matches!(
        memo_gate::judge(&[draft(&meta(&["FR7"], &[], None), &[])], &edge),
        MemoGate::Deny { why: MemoWhy::Undisposed, .. }
    ));
}

#[test]
fn mlnk_judge_names_the_undisposed() {
    let g = graph();
    let touches = ids(&["FR9"]);
    let digest = memo_digest(&g, &touches);
    let links = ["fx-m.1", "fx-m.4"];
    let reasons = [("P-9", "条は関わらない"), ("ADR-9", "決めは関わらない")];
    let undisposed = |found: &[&str]| MemoGate::Deny {
        why: MemoWhy::Undisposed,
        ids: ids(found),
        digest: Some(digest.clone()),
    };
    let first = draft(&meta(&["FR9"], &reasons, None), &links);
    assert_eq!(
        memo_gate::judge(&[first], &g),
        undisposed(&["fx-m.2", "fx-m.5", "nx#r1"])
    );
    // 理由が空白だけの not-relevant は処分でない。
    let blank = draft(&meta(&["FR9"], &[("P-9", "  "), reasons[1]], None), &links);
    assert_eq!(
        memo_gate::judge(&[blank], &g),
        undisposed(&["P-9", "fx-m.2", "fx-m.5", "nx#r1"])
    );
    // relates-to と discovered-from のほかの型の辺は処分でない（型の無い項と同じく links に入らない）。
    let blocks = MemoDraft {
        metadata: Some(meta(&["FR9"], &reasons, None)),
        links: ids(&["fx-m.4"]),
    };
    assert_eq!(
        memo_gate::judge(&[blocks], &g),
        undisposed(&["fx-m.1", "fx-m.2", "fx-m.5", "nx#r1"])
    );

    // 7 つの候補を全部処分した下書きは、digest が今の値なら通り、違えば stale。
    let all = [
        ("P-9", "条は関わらない"),
        ("ADR-9", "決めは関わらない"),
        ("fx-m.2", "直った問題の memo"),
        ("fx-m.5", "別の契約"),
        ("nx#r1", "別の行"),
    ];
    let allow = draft(&meta(&["FR9"], &all, Some(&digest)), &links);
    assert_eq!(memo_gate::judge(std::slice::from_ref(&allow), &g), MemoGate::Allow);
    for stale in [Some("0000000000000000"), None] {
        assert_eq!(
            memo_gate::judge(&[draft(&meta(&["FR9"], &all, stale), &links)], &g),
            MemoGate::Deny {
                why: MemoWhy::Stale,
                ids: Vec::new(),
                digest: Some(digest.clone()),
            }
        );
    }
    // 最初に通さなかった下書きの答えを返し、下書きが 0 なら通す。
    let bad = draft("[1]", &[]);
    assert_eq!(
        memo_gate::judge(&[allow.clone(), bad, draft(&meta(&[], &[], None), &[])], &g),
        MemoGate::Deny { why: MemoWhy::Metadata, ids: Vec::new(), digest: None }
    );
    assert_eq!(memo_gate::judge(&[], &g), MemoGate::Allow);
    // 候補の題が変われば、写した digest は古くなる。
    let moved = graph_of(&index(), &ledger("題 3", "別の題 4", 0));
    assert!(matches!(
        memo_gate::judge(&[allow], &moved),
        MemoGate::Deny { why: MemoWhy::Stale, .. }
    ));
}

#[test]
fn mlnk_output_words() {
    let words: Vec<&str> = MemoWhy::ALL.iter().map(|w| w.word()).collect();
    assert_eq!(
        words,
        ["args", "metadata", "no-touches", "unread", "unknown-id", "too-many", "undisposed", "stale"]
    );
    assert_eq!(memo_gate::output(&MemoGate::Allow), None);
    let deny = memo_gate::output(&MemoGate::Deny {
        why: MemoWhy::Undisposed,
        ids: ids(&["P-9", "fx-m.2"]),
        digest: Some("0123456789abcdef".to_string()),
    })
    .expect("止める答え");
    assert!(deny.contains(r#""permissionDecision":"deny""#), "{deny}");
    assert!(deny.contains(r#""hookEventName":"PreToolUse""#), "{deny}");
    assert!(
        deny.contains("memo の起票の門は止める（undisposed） id = P-9 fx-m.2 要約値 = 0123456789abcdef 次の一手 = "),
        "{deny}"
    );
    assert!(deny.contains("[再発]") && deny.contains("--deps"), "{deny}");
    let unknown = memo_gate::output(&MemoGate::Unknown {
        why: MemoWhy::Unread,
        ids: ids(&["ledger"]),
        digest: None,
    })
    .expect("まだ分からない答え");
    assert!(unknown.contains("memo の起票の門は まだ分からない（unread） id = ledger 次の一手 = "), "{unknown}");
    assert!(!unknown.contains("要約値"), "{unknown}");
    // 問いの門の答えの字とは頭の字が違う。
    let question = gate::output(&gate::Gate::Unknown {
        why: gate::Why::Unread,
        ids: ids(&["ledger"]),
        digest: None,
    })
    .expect("問いの答え");
    assert_ne!(unknown, question);
    for why in MemoWhy::ALL {
        let text = memo_gate::output(&MemoGate::Deny { why, ids: Vec::new(), digest: None })
            .expect("答え");
        assert!(text.contains(&format!("（{}）", why.word())), "{text}");
    }
}

/// drop で path を消す守り（dir なら中身ごと・file なら file を・誤りは捨てる）。
struct Tidy(PathBuf);

impl Drop for Tidy {
    fn drop(&mut self) {
        let _ = match fs::symlink_metadata(&self.0) {
            Ok(meta) if meta.is_dir() => fs::remove_dir_all(&self.0),
            _ => fs::remove_file(&self.0),
        };
    }
}

/// 歯ごとの作業場（.git が file の repo・記録の置き場・偽の bd と偽の設計の道具と、それらが出す字の写し）。
struct Place {
    root: PathBuf,
    repo: PathBuf,
    log: PathBuf,
    _git: Tidy,
}

impl Place {
    fn new(name: &str) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("mlnk")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        let (repo, log) = (root.join("repo"), root.join("log"));
        fs::create_dir_all(&repo).expect("repo の置き場");
        fs::create_dir_all(&log).expect("記録の置き場");
        let git = repo.join(".git");
        let tidy = Tidy(git.clone());
        fs::write(&git, "gitdir: /nonexistent/mlnk\n").expect(".git の file");
        fs::write(root.join("ledger.json"), ledger("題 3", "題 4", 0)).expect("台帳の写し");
        fs::write(root.join("index.tsv"), index()).expect("索引の写し");
        for (program, out) in [("bd", "ledger.json"), ("folio", "index.tsv")] {
            let path = root.join(program);
            let body = format!(
                "#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{}'\nexec cat '{}'\n",
                log.join(format!("{program}.log")).display(),
                root.join(out).display()
            );
            fs::write(&path, body).expect("偽の program");
            fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).expect("権限");
        }
        Place { root, repo, log, _git: tidy }
    }

    /// 偽の program が撃たれた回ごとの引数の行。
    fn calls(&self, program: &str) -> Vec<String> {
        fs::read_to_string(self.log.join(format!("{program}.log")))
            .map(|t| t.lines().map(str::to_string).collect())
            .unwrap_or_default()
    }

    /// tz hook question-gate を `repo` と偽の program で撃ち、標準入力に payload を書いて閉じ、終わりまで待つ。
    fn tz(&self, repo: &Path, payload: &str) -> Output {
        let mut child = Command::new(env!("CARGO_BIN_EXE_tz"))
            .args(["hook", "question-gate", "--repo"])
            .arg(repo)
            .arg("--bd")
            .arg(self.root.join("bd"))
            .arg("--folio")
            .arg(self.root.join("folio"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("tz を撃つ");
        let mut stdin = child.stdin.take().expect("標準入力");
        let _ = stdin.write_all(payload.as_bytes());
        drop(stdin);
        child.wait_with_output().expect("tz の終わり")
    }

    /// repo の中の file の path と byte の一覧（書かれていないことを比べる）。
    fn tree(&self) -> Vec<(PathBuf, Vec<u8>)> {
        let mut out = Vec::new();
        let mut stack = vec![self.repo.clone()];
        while let Some(dir) = stack.pop() {
            for entry in fs::read_dir(&dir).expect("dir を読む") {
                let path = entry.expect("entry").path();
                if path.is_dir() {
                    out.push((path.clone(), Vec::new()));
                    stack.push(path);
                } else {
                    out.push((path.clone(), fs::read(&path).expect("file を読む")));
                }
            }
        }
        out.sort();
        out
    }
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).expect("UTF-8")
}

#[test]
fn mlnk_bin_memo_draft_is_judged() {
    let place = Place::new("judged");
    let before = place.tree();
    let reasons = [("P-9", "条は関わらない"), ("ADR-9", "決めは関わらない")];
    let deps = "relates-to:fx-m.1,discovered-from:fx-m.4";
    let p = payload(&memo_command(deps, &meta(&["FR9"], &reasons, None)));
    let out = place.tz(&place.repo, &p);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let want = memo_gate::output(&memo_gate::judge(&memo_gate::drafts(&p), &graph()))
        .map(|t| format!("{t}\n"))
        .expect("止める答え");
    assert!(want.contains("undisposed") && want.contains("id = fx-m.2 fx-m.5 nx#r1"), "{want}");
    assert_eq!(text(&out.stdout), want);
    assert_eq!(place.calls("bd"), [BD_ARGS.join(" ")]);
    assert_eq!(
        place.calls("folio"),
        [format!("{} {}", FOLIO_ARGS.join(" "), place.repo.join(DESIGN_DIR).display())]
    );
    assert!(before == place.tree(), "repo の byte が変わる");

    // 7 つの候補を全部処分して要約値を写した下書きは通る（stdout は空）。
    let digest = memo_digest(&graph(), &ids(&["FR9"]));
    let all = [
        ("P-9", "条は関わらない"),
        ("ADR-9", "決めは関わらない"),
        ("fx-m.2", "直った問題の memo"),
        ("fx-m.5", "別の契約"),
        ("nx#r1", "別の行"),
    ];
    let pass = payload(&memo_command(deps, &meta(&["FR9"], &all, Some(&digest))));
    let out = place.tz(&place.repo, &pass);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    assert!(out.stdout.is_empty(), "{out:?}");
    assert!(before == place.tree(), "repo の byte が変わる");
}

#[test]
fn mlnk_bin_question_answer_comes_first() {
    let place = Place::new("question-first");
    let question = format!(
        "bdw create --parent=fx-m --labels=intake:question --metadata='{}' 門の歯の問い",
        meta(&["FR9"], &[], None)
    );
    let memo = memo_command("relates-to:fx-m.1", &meta(&["FR9"], &[], None));
    let p = payload(&format!("{question}; {memo}"));
    assert_eq!(memo_gate::drafts(&p).len(), 1);
    let out = place.tz(&place.repo, &p);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let g = graph();
    let want = gate::output(&gate::judge(&gate::drafts(&p), &g)).expect("問いの答え");
    assert!(want.contains("問いの起票の門は止める（undisposed）"), "{want}");
    assert_eq!(text(&out.stdout), format!("{want}\n"));
    let memo_answer = memo_gate::output(&memo_gate::judge(&memo_gate::drafts(&p), &g)).expect("memo の答え");
    assert!(!text(&out.stdout).contains("memo の起票の門"), "{memo_answer}");
    assert_eq!(place.calls("bd").len(), 1);
    assert_eq!(place.calls("folio").len(), 1);

    // 問いの門が通す時は memo の門の答えを返す（問いの下書きは全部処分した束の要約値を持つ）。
    let nr = [("P-9", "条は関わらない"), ("ADR-9", "決めは関わらない")];
    let digest = gate::bundle_digest(&g, &ids(&["FR9"]));
    let pass = format!(
        "bdw create --labels=intake:question --metadata='{}' 門の歯の問い",
        meta(&["FR9"], &nr, Some(&digest))
    );
    let p = payload(&format!("{pass}; {memo}"));
    let out = place.tz(&place.repo, &p);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    assert_eq!(text(&out.stdout), format!("{memo_answer}\n"));
}

#[test]
fn mlnk_bin_bad_args_close_memo_drafts() {
    let place = Place::new("bad-args");
    let file = place.root.join("file.txt");
    fs::write(&file, "x").expect("dir でない file");
    let memo = payload(&memo_command("relates-to:fx-m.1", &meta(&["FR9"], &[], None)));
    let args = memo_gate::output(&MemoGate::Deny {
        why: MemoWhy::Args,
        ids: Vec::new(),
        digest: None,
    })
    .expect("args の答え");
    let out = place.tz(&file, &memo);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    assert_eq!(text(&out.stdout), format!("{args}\n"));
    assert!(text(&out.stderr).contains("tz hook question-gate"), "{out:?}");

    let out = place.tz(&file, &payload("ls -la"));
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    assert!(out.stdout.is_empty(), "{out:?}");
    assert!(text(&out.stderr).contains("tz hook question-gate"), "{out:?}");

    // 問いの下書きも在れば、今までの問いの門の args の答え。
    let both = payload(&format!(
        "{}; bdw create --labels=intake:question --metadata='{}' 問い",
        memo_command("relates-to:fx-m.1", &meta(&["FR9"], &[], None)),
        meta(&["FR9"], &[], None)
    ));
    let question = gate::output(&gate::Gate::Deny {
        why: gate::Why::Args,
        ids: Vec::new(),
        digest: None,
    })
    .expect("args の答え");
    let out = place.tz(&file, &both);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    assert_eq!(text(&out.stdout), format!("{question}\n"));
    assert!(place.calls("bd").is_empty(), "偽の bd を撃つ");
    assert!(place.calls("folio").is_empty(), "偽の設計の道具を撃つ");
}
