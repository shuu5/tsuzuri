//! memo の起票の門の似た memo と code の候補の歯（接頭辞 msim_・設計ノート surface-v4b 行 t-memo-similar・判断の記録 ADR-72 の決定 (5)）。
//! 純関数の歯は字と `CodeGraph` と `Def` の字の組みから `memo_gate` の関数を直に撃つ。口の歯は tz hook question-gate を
//! mlnk.rs の作業場と同じ形（偽の bd と偽の設計の道具）に、git init の一時の repo と偽の ast-grep（撃たれた引数を記録の file に
//! 1 行足し、見本の stream を出す script・子の PATH の頭に置く）を足して撃つ。作業場は CARGO_TARGET_TMPDIR の下の msim/<歯の名>。
#![cfg(test)]

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use tsuzuri_core::graph::code::{CodeGraph, Def, DefKind};
use tsuzuri_core::graph::{Graph, Inputs, build};
use tsuzuri_core::memo_gate::{
    self, CODE_CAP, MemoDraft, MemoGate, MemoWhy, Seen, code_needs, code_words, memo_digest,
    memo_texts, payload_cwd, similar,
};

/// 索引の字（FR9 の 1 節点で辺は無し）。
const INDEX: &str = "FR9\t要件\tsrs.yaml\td-FR9\t節点 FR9\n";

/// 台帳の memo の bead の 1 本の JSON の字（label intake:memo・本文は在れば）。
fn memo_bead(id: &str, status: &str, title: &str, description: Option<&str>) -> String {
    let description = description
        .map(|d| format!(r#","description":"{d}""#))
        .unwrap_or_default();
    format!(
        r#"{{"id":"{id}","title":"{title}","status":"{status}","issue_type":"task","labels":["intake:memo"],"metadata":{{}}{description}}}"#
    )
}

/// 台帳の見本（ms.1 は閉じた memo・ms.3 は開いた memo で本文付き・ms.4 は label の無い task）。
fn ledger_sample() -> String {
    format!(
        r#"[{},{},{{"id":"ms.4","title":"abcdefghijk","status":"open","issue_type":"task","labels":[],"metadata":{{}}}}]"#,
        memo_bead("ms.1", "closed", "abcdefghijk", None),
        memo_bead("ms.3", "open", "abc", Some("de")),
    )
}

/// 台帳の字（ms.1 の題は引数・ms.1 だけ）。
fn ledger_one(title: &str) -> String {
    format!("[{}]", memo_bead("ms.1", "closed", title, None))
}

fn graph_of(ledger: &str) -> Graph {
    build(&Inputs {
        design_index: INDEX,
        ledger,
        events: "",
    })
}

fn ids(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

fn strings(items: &[&str]) -> Vec<String> {
    ids(items)
}

/// 下書き（metadata は在り・links は無し・欄 text は引数）。
fn draft(metadata: &str, links: &[&str], text: Option<&str>) -> MemoDraft {
    MemoDraft {
        metadata: Some(metadata.to_string()),
        links: ids(links),
        text: text.map(str::to_string),
        body_file: None,
    }
}

/// metadata の JSON の字（touches は FR9・code は在れば 1 つの path か id・not-relevant は id と理由の組・digest は在れば）。
fn meta(code: Option<&str>, not_relevant: &[(&str, &str)], digest: Option<&str>) -> String {
    let code = code
        .map(|c| format!(r#","code":"{c}""#))
        .unwrap_or_default();
    let not_relevant = not_relevant
        .iter()
        .map(|(id, why)| format!("\"{id}\":\"{why}\""))
        .collect::<Vec<_>>()
        .join(",");
    let digest = digest
        .map(|d| format!(r#","digest":"{d}""#))
        .unwrap_or_default();
    format!(
        r#"{{"short":"短い題","touches":["FR9"],"not-relevant":{{{not_relevant}}}{code}{digest}}}"#
    )
}

/// 定義 1 つ（入れ子の mod の中なら id に `mod tests/` が付く）。
fn def(file: &str, kind: DefKind, name: &str, nested: bool) -> Def {
    let nest = if nested { "mod tests/" } else { "" };
    Def {
        id: format!("{file}#{nest}{} {name}", kind.word()),
        file: file.to_string(),
        kind,
        name: name.to_string(),
        spans: vec![(1, 2)],
    }
}

fn files_of(files: &[&str]) -> std::collections::BTreeSet<String> {
    files.iter().map(|f| f.to_string()).collect()
}

/// code の層の見本。files は src/a.rs から src/d.rs と docs/x.md。fn memo_needs は src/a.rs と src/b.rs（mod tests の中）、
/// fn read_all は 4 file、fn load_all は src/a.rs から src/c.rs、mod memo_mod は src/c.rs、fn judge_one は src/d.rs。
/// `with_b` が偽なら src/b.rs を files と定義から外す。
fn layer(with_b: bool) -> CodeGraph {
    let mut files = vec!["src/a.rs", "src/c.rs", "src/d.rs", "docs/x.md"];
    let mut defs = vec![def("src/a.rs", DefKind::Fn, "memo_needs", false)];
    if with_b {
        files.push("src/b.rs");
        defs.push(def("src/b.rs", DefKind::Fn, "memo_needs", true));
    }
    let spread = [
        ("read_all", &["src/a.rs", "src/b.rs", "src/c.rs", "src/d.rs"][..]),
        ("load_all", &["src/a.rs", "src/b.rs", "src/c.rs"][..]),
    ];
    for (name, at) in spread {
        defs.extend(
            at.iter()
                .filter(|f| with_b || **f != "src/b.rs")
                .map(|f| def(f, DefKind::Fn, name, false)),
        );
    }
    defs.push(def("src/c.rs", DefKind::Mod, "memo_mod", false));
    defs.push(def("src/d.rs", DefKind::Fn, "judge_one", false));
    CodeGraph {
        files: files_of(&files),
        defs,
        ..CodeGraph::default()
    }
}

/// Bash の PreToolUse の hook の入力の字（`cwd` は在れば鍵 cwd に入れる）。
fn payload(command: &str, cwd: Option<&Path>) -> String {
    let command = command.replace('\\', "\\\\").replace('"', "\\\"");
    let cwd = cwd
        .map(|c| format!(r#","cwd":"{}""#, c.display()))
        .unwrap_or_default();
    format!(
        r#"{{"session_id":"s-1","hook_event_name":"PreToolUse","tool_name":"Bash"{cwd},"tool_input":{{"command":"{command}"}}}}"#
    )
}

#[test]
fn msim_drafts_read_title_and_body() {
    // 1 つ目: 旗 --title と -d と --body-file（最後の値）・metadata と links は今のまま。
    let one = payload(
        "bdw create --parent=ms --labels=intake:memo --title=題の字 -d 本文の字 --body-file m.md --body-file=n.md --deps relates-to:ms.1 --metadata '{\"a\":1}'",
        None,
    );
    assert_eq!(
        memo_gate::drafts(&one),
        vec![MemoDraft {
            metadata: Some("{\"a\":1}".to_string()),
            links: ids(&["ms.1"]),
            text: Some("題の字\n本文の字".to_string()),
            body_file: Some("n.md".to_string()),
        }]
    );
    // 2 つ目: 位置の題と --description=値。
    let two = payload(
        "bd create 位置の題 --labels intake:memo --description=説明",
        None,
    );
    assert_eq!(
        memo_gate::drafts(&two),
        vec![MemoDraft {
            metadata: None,
            links: Vec::new(),
            text: Some("位置の題\n説明".to_string()),
            body_file: None,
        }]
    );
    // 3 つ目: 値を取る旗の後の語（intake:memo と ms）は位置の題でない。
    let three = payload("bd create --labels intake:memo --parent ms 題", None);
    assert_eq!(
        memo_gate::drafts(&three),
        vec![MemoDraft {
            metadata: None,
            links: Vec::new(),
            text: Some("題".to_string()),
            body_file: None,
        }]
    );
    // 旗も題も無ければ空の字・--description 値も読む。
    let none = memo_gate::drafts(&payload("bd create --labels intake:memo", None));
    assert_eq!(none[0].text.as_deref(), Some(""));
    let long = memo_gate::drafts(&payload(
        "bd create --labels intake:memo --description 説明の字",
        None,
    ));
    assert_eq!(long[0].text.as_deref(), Some("説明の字"));
    // hook の入力の鍵 cwd。
    assert_eq!(
        payload_cwd(&payload("ls", Some(Path::new("/w/x")))).as_deref(),
        Some("/w/x")
    );
    assert_eq!(payload_cwd(&payload("ls", None)), None);
    assert_eq!(payload_cwd(r#"{"cwd":7}"#), None);
    assert_eq!(payload_cwd("not json"), None);
}

#[test]
fn msim_memo_texts_read_memos() {
    let texts = memo_texts(&ledger_sample());
    let want: BTreeMap<String, String> = BTreeMap::from([
        ("ms.1".to_string(), "abcdefghijk\n".to_string()),
        ("ms.3".to_string(), "abc\nde".to_string()),
    ]);
    assert_eq!(texts, want);
    for empty in ["", "  ", "not json", r#"{"id":"ms.1"}"#, "[]"] {
        assert!(memo_texts(empty).is_empty(), "{empty}");
    }
}

#[test]
fn msim_similar_ranks_and_caps() {
    let pairs = |items: &[(&str, &str)]| -> BTreeMap<String, String> {
        items
            .iter()
            .map(|(id, text)| (id.to_string(), text.to_string()))
            .collect()
    };
    // 1 組目: 比の大きい順の上位 5 本を id の natural_cmp の順で返す（ms.10 は同じ比の 3 本で最後・ms.8 は見出しの行を除いて 0）。
    let first = pairs(&[
        ("ms.1", "abcdefghijk"),
        ("ms.2", "ABC-DEF GHI"),
        ("ms.3", "abcdefgh"),
        ("ms.5", "abcde"),
        ("ms.9", "abc\nde"),
        ("ms.10", "abcde"),
        ("ms.8", "zz\n### abcdefghijk"),
    ]);
    assert_eq!(
        similar(&first, "abcdefghijk"),
        strings(&["ms.1", "ms.2", "ms.3", "ms.5", "ms.9"])
    );
    // 2 組目: 比 30 は入り、比 29 は入らない。
    let second = pairs(&[("ms.6", "abcd"), ("ms.7", "abcdefxyzuvwq")]);
    assert_eq!(similar(&second, "abcdefghijk"), strings(&["ms.6"]));
    // 字の空の下書きと空の写しは何も返さない。
    assert!(similar(&first, "").is_empty());
    assert!(similar(&BTreeMap::new(), "abcdefghijk").is_empty());
}

#[test]
fn msim_code_needs_name_files_and_defs() {
    let text = "src/a.rs:12 の memo_needs と read_all と load_all と memo_mod と docs/y.md と tsuzuri_core::memo_gate::judge_one と plan";
    let words = code_words(text);
    assert_eq!(
        words,
        strings(&[
            "src/a.rs",
            "memo_needs",
            "read_all",
            "load_all",
            "memo_mod",
            "docs/y.md",
            "judge_one"
        ])
    );
    // 行の範囲と末の記号と重ねた語。
    assert_eq!(
        code_words("src/a.rs:3-9, src/a.rs. load_all: load_all"),
        strings(&["src/a.rs", "load_all"])
    );
    // read_all は 4 file で広すぎ・memo_mod は mod・docs/y.md は files に無い。
    assert_eq!(
        code_needs(&layer(true), &words),
        strings(&[
            "src/a.rs",
            "src/a.rs#load_all",
            "src/a.rs#memo_needs",
            "src/b.rs#load_all",
            "src/b.rs#memo_needs",
            "src/c.rs#load_all",
            "src/d.rs#judge_one"
        ])
    );
    assert!(code_needs(&layer(true), &[]).is_empty());
}

/// 判じの見本の材料（索引は FR9 だけ・台帳は ms.1 だけ・code は src/b.rs を外した層）。
fn seen_sample(graph: &Graph) -> Seen<'_> {
    Seen {
        graph,
        memos: memo_texts(&ledger_one("abcdefghijk")),
        code: Some(layer(false)),
    }
}

const SAMPLE_TEXT: &str = "abcdefghijk\nsrc/a.rs の memo_needs";

#[test]
fn msim_judge_names_similar_and_code() {
    let g = graph_of(&ledger_one("abcdefghijk"));
    let seen = seen_sample(&g);
    let digest = memo_digest(&g, &ids(&["FR9", "ms.1"]));
    let undisposed = MemoGate::Deny {
        why: MemoWhy::Undisposed,
        ids: ids(&["ms.1", "src/a.rs", "src/a.rs#memo_needs"]),
        digest: Some(digest.clone()),
    };
    let first = draft(&meta(None, &[], None), &[], Some(SAMPLE_TEXT));
    let got = memo_gate::judge(&[first], &seen);
    assert_eq!(got, undisposed);
    let text = memo_gate::output(&got).expect("止める答え");
    assert!(text.contains("metadata の code"), "{text}");

    // code と not-relevant と relates-to で全部を処分し、digest を写せば通る。
    let reasons = [("src/a.rs#memo_needs", "別の関数")];
    let full = |digest: Option<&str>| {
        draft(
            &meta(Some("src/a.rs"), &reasons, digest),
            &["ms.1"],
            Some(SAMPLE_TEXT),
        )
    };
    assert_eq!(
        memo_gate::judge(&[full(None)], &seen),
        MemoGate::Deny {
            why: MemoWhy::Stale,
            ids: Vec::new(),
            digest: Some(digest.clone()),
        }
    );
    assert_eq!(
        memo_gate::judge(&[full(Some(&digest))], &seen),
        MemoGate::Allow
    );
    // ms.1 の題だけを替えたグラフでは、写した digest は古い。
    let moved = graph_of(&ledger_one("別の題"));
    let seen_moved = Seen {
        graph: &moved,
        memos: seen.memos.clone(),
        code: seen.code.clone(),
    };
    assert!(matches!(
        memo_gate::judge(&[full(Some(&digest))], &seen_moved),
        MemoGate::Deny { why: MemoWhy::Stale, .. }
    ));
    // code の処分は metadata の code の字の配列でもよい。
    let listed = r#"{"short":"短い題","touches":["FR9"],"code":["src/a.rs","src/a.rs#memo_needs"]}"#;
    assert!(matches!(
        memo_gate::judge(&[draft(listed, &["ms.1"], Some(SAMPLE_TEXT))], &seen),
        MemoGate::Deny { why: MemoWhy::Stale, .. }
    ));

    // 似た memo も code の候補も無い下書きでは、答えの digest は memo_digest の値と同じ。
    let plain = memo_digest(&g, &ids(&["FR9"]));
    assert_eq!(
        memo_gate::judge(&[draft(&meta(None, &[], None), &[], Some("zz"))], &seen),
        MemoGate::Deny {
            why: MemoWhy::Stale,
            ids: Vec::new(),
            digest: Some(plain.clone()),
        }
    );
    let pass = draft(&meta(None, &[], Some(&plain)), &[], Some("zz"));
    assert_eq!(memo_gate::judge(&[pass], &seen), MemoGate::Allow);
}

#[test]
fn msim_judge_unread_body_and_code() {
    let g = graph_of(&ledger_one("abcdefghijk"));
    let unread = |what: &[&str]| MemoGate::Unknown {
        why: MemoWhy::Unread,
        ids: ids(what),
        digest: None,
    };
    let seen = seen_sample(&g);
    let no_body = draft(&meta(None, &[], None), &[], None);
    assert_eq!(memo_gate::judge(&[no_body], &seen), unread(&["body"]));

    let named = draft(&meta(None, &[], None), &[], Some("src/a.rs の memo_needs"));
    let no_code = Seen {
        code: None,
        ..seen.clone()
    };
    assert_eq!(
        memo_gate::judge(std::slice::from_ref(&named), &no_code),
        unread(&["code"])
    );
    // code の語を持たない下書きは、code の層が無くても unread にならない。
    let plain = draft(&meta(None, &[], None), &[], Some("plain words only"));
    assert!(matches!(
        memo_gate::judge(&[plain], &Seen::of(&g)),
        MemoGate::Deny { why: MemoWhy::Stale, .. }
    ));
    // 読めない元は natural_cmp の順に全部を挙げる（台帳が読めず本文も無く code の層も無い）。
    let blind = graph_of("");
    assert_eq!(
        memo_gate::judge(&[draft(&meta(None, &[], None), &[], None)], &Seen::of(&blind)),
        unread(&["body", "ledger"])
    );
    assert_eq!(
        memo_gate::judge(&[named], &Seen::of(&blind)),
        unread(&["code", "ledger"])
    );
}

#[test]
fn msim_judge_code_cap() {
    let g = graph_of(&ledger_one("abcdefghijk"));
    let paths: Vec<String> = (1..=11).map(|n| format!("f/{n}.rs")).collect();
    let code = CodeGraph {
        files: paths.iter().cloned().collect(),
        ..CodeGraph::default()
    };
    let seen = Seen {
        graph: &g,
        memos: BTreeMap::new(),
        code: Some(code),
    };
    let eleven = draft(&meta(None, &[], None), &[], Some(&paths.join(" ")));
    match memo_gate::judge(&[eleven], &seen) {
        MemoGate::Unknown { why, ids, digest } => {
            assert_eq!(why, MemoWhy::TooMany);
            assert_eq!(ids.len(), 11, "{ids:?}");
            assert!(digest.is_some());
        }
        other => panic!("11 の file で too-many にならない: {other:?}"),
    }
    assert_eq!(CODE_CAP, 10);
    let ten = draft(&meta(None, &[], None), &[], Some(&paths[..10].join(" ")));
    match memo_gate::judge(&[ten], &seen) {
        MemoGate::Deny { why, ids, .. } => {
            assert_eq!(why, MemoWhy::Undisposed);
            assert_eq!(ids, paths[..10].to_vec());
        }
        other => panic!("10 の file で undisposed にならない: {other:?}"),
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

/// 見本の stream の 1 行（src/a.rs の fn memo_needs・中核が読む欄だけ）。
const STREAM: &str = r#"{"ruleId":"fn","file":"src/a.rs","language":"Rust","range":{"byteOffset":{"start":0,"end":9},"start":{"line":0},"end":{"line":0}},"metaVariables":{"single":{"NAME":{"text":"memo_needs"}}}}
"#;

/// 歯ごとの作業場（git init の repo・本文の file を置く cwd・記録の置き場・偽の bd と設計の道具と ast-grep）。
struct Place {
    root: PathBuf,
    repo: PathBuf,
    cwd: PathBuf,
    log: PathBuf,
    _tidy: Tidy,
}

impl Place {
    /// 台帳の字は `ledger`・偽の ast-grep の終了 code は `rc`。
    fn new(name: &str, ledger: &str, rc: u8) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("msim")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        let (repo, cwd, log) = (root.join("repo"), root.join("cwd"), root.join("log"));
        let tidy = Tidy(root.clone());
        for dir in [repo.join("src"), cwd.join("notes"), log.clone(), root.join("bin")] {
            fs::create_dir_all(dir).expect("dir");
        }
        fs::write(repo.join("src/a.rs"), "fn memo_needs() {}\n").expect("src/a.rs");
        let git = |args: &[&str]| {
            let out = Command::new("git")
                .args(args)
                .current_dir(&repo)
                .output()
                .expect("git");
            assert!(out.status.success(), "{out:?}");
        };
        git(&["init", "-q"]);
        git(&["add", "src/a.rs"]);
        fs::write(root.join("ledger.json"), ledger).expect("台帳の写し");
        fs::write(root.join("index.tsv"), INDEX).expect("索引の写し");
        fs::write(root.join("stream.jsonl"), STREAM).expect("stream の写し");
        for (program, out) in [("bd", "ledger.json"), ("folio", "index.tsv")] {
            let body = format!(
                "#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{}'\nexec cat '{}'\n",
                log.join(format!("{program}.log")).display(),
                root.join(out).display()
            );
            Self::script(&root.join(program), &body);
        }
        let sg = format!(
            "#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{}'\ncat '{}'\nexit {rc}\n",
            log.join("ast-grep.log").display(),
            root.join("stream.jsonl").display()
        );
        Self::script(&root.join("bin/ast-grep"), &sg);
        Place { root, repo, cwd, log, _tidy: tidy }
    }

    fn script(path: &Path, body: &str) {
        fs::write(path, body).expect("偽の program");
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("権限");
    }

    /// 偽の program が撃たれた回ごとの引数の行。
    fn calls(&self, program: &str) -> Vec<String> {
        fs::read_to_string(self.log.join(format!("{program}.log")))
            .map(|t| t.lines().map(str::to_string).collect())
            .unwrap_or_default()
    }

    /// tz hook question-gate を偽の program で撃つ（子の PATH の頭に偽の ast-grep の dir を足す）。
    fn tz(&self, payload: &str) -> Output {
        let path = format!(
            "{}:{}",
            self.root.join("bin").display(),
            std::env::var("PATH").unwrap_or_default()
        );
        let mut child = Command::new(env!("CARGO_BIN_EXE_tz"))
            .args(["hook", "question-gate", "--repo"])
            .arg(&self.repo)
            .arg("--bd")
            .arg(self.root.join("bd"))
            .arg("--folio")
            .arg(self.root.join("folio"))
            .env("PATH", path)
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
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).expect("UTF-8")
}

/// memo の起票の command（metadata と残りの引数の字）。
fn memo_command(metadata: &str, rest: &str) -> String {
    format!("bdw create --parent=ms --labels=intake:memo --metadata='{metadata}' {rest}")
}

#[test]
fn msim_bin_similar_memo_is_named() {
    let place = Place::new("similar", &ledger_one("abcdefghijk"), 0);
    fs::write(place.cwd.join("notes/b.md"), "abcdefghijk\n").expect("本文の file");
    let g = graph_of(&ledger_one("abcdefghijk"));
    let digest = memo_digest(&g, &ids(&["FR9", "ms.1"]));
    // cwd からの相対の --body-file を読み、閉じた memo ms.1 が候補に出る（repo の中には本文の file が無い）。
    let command = memo_command(&meta(None, &[], None), "--body-file=notes/b.md zz");
    let out = place.tz(&payload(&command, Some(&place.cwd)));
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let stdout = text(&out.stdout);
    assert!(stdout.ends_with('\n') && stdout.matches('\n').count() == 1, "{stdout}");
    assert!(
        stdout.contains(&format!("（undisposed） id = ms.1 要約値 = {digest} 次の一手 = ")),
        "{stdout}"
    );
    assert!(stdout.contains(r#""permissionDecision":"deny""#), "{stdout}");
    // relates-to で処分して digest を写せば通る（stdout は空）。
    let pass = format!(
        "{} --deps relates-to:ms.1",
        memo_command(&meta(None, &[], Some(&digest)), "--body-file=notes/b.md zz")
    );
    let out = place.tz(&payload(&pass, Some(&place.cwd)));
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    assert!(out.stdout.is_empty(), "{out:?}");
    // 偽の ast-grep は撃たれない（code の語が無い）。
    assert!(place.calls("ast-grep").is_empty());
}

#[test]
fn msim_bin_code_candidates() {
    let place = Place::new("code", &ledger_one("abcdefghijk"), 0);
    let g = graph_of(&ledger_one("abcdefghijk"));
    let digest = memo_digest(&g, &ids(&["FR9"]));
    let command = memo_command(&meta(None, &[], None), "-d 'src/a.rs の memo_needs'");
    let out = place.tz(&payload(&command, None));
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let stdout = text(&out.stdout);
    assert!(
        stdout.contains(&format!(
            "（undisposed） id = src/a.rs src/a.rs#memo_needs 要約値 = {digest} 次の一手 = "
        )),
        "{stdout}"
    );
    assert!(stdout.contains("metadata の code"), "{stdout}");
    assert_eq!(
        place.calls("ast-grep"),
        ["scan --rule .config/code-defs.yml --json=stream"]
    );
}

#[test]
fn msim_bin_unread_body_and_code() {
    let place = Place::new("unread-body", &ledger_one("abcdefghijk"), 0);
    let command = memo_command(&meta(None, &[], None), "--body-file=notes/none.md zz");
    let out = place.tz(&payload(&command, Some(&place.cwd)));
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let stdout = text(&out.stdout);
    assert!(stdout.contains("まだ分からない（unread） id = body 次の一手 = "), "{stdout}");
    assert!(place.calls("ast-grep").is_empty());

    // 偽の ast-grep が rc 1 を返す repo で code を名指す下書きは、code の層が読めず unread。
    let place = Place::new("unread-code", &ledger_one("abcdefghijk"), 1);
    let command = memo_command(&meta(None, &[], None), "-d 'src/a.rs の memo_needs'");
    let out = place.tz(&payload(&command, None));
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let stdout = text(&out.stdout);
    assert!(stdout.contains("まだ分からない（unread） id = code 次の一手 = "), "{stdout}");
    assert_eq!(place.calls("ast-grep").len(), 1);
}
