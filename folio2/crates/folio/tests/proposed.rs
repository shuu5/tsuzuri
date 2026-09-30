//! 編集時の口の歯（便 198・docs/design/delivery-198.md §1 (c)・判断の記録 ADR-33 決定 (1)(2)・要件書 FR28・AC31）。
//! 土台は凍結した写し（tests/fixtures/floor_base/design-intent/）を一時 dir の `repo/design-intent/` に、器の導出 file を
//! `repo/contracts/` に作って git の 1 commit にしたもの（素の床は合格 0）。口の一時の作業場所は版管理の外の `tmp/` に作らせる。口が止めた行は、同じ中身を書いた置き場の素の床にも
//! 同じ字で在ること（条 P-15.2・編集時の判定 ⊆ 事後の判定）を、同じ写しで撃ち比べて見る。期待の行は歯の側の手書き。
#![cfg(test)]

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const FLOOR_BASE: &str = "tests/fixtures/floor_base/design-intent";
const BOGUS: &str = "[未知の欄] rules.yaml: 行 R-2 の未知の欄「bogus」";
const DANGLING: &str = "[参照 id] srs.yaml: requirements[0].basis[1]: id P-99 が実在しない";
const LINK_HEAD: &str = "# つながり（編集は止めない・事後の床が数える）: ";
const CONTRACT: &str = "[note] design-note/example.yaml: §6 の行 a: 契約表の欄「bogus」が器の導出 file に無い";
const SEAL: &str = "[adr] ADR-4: 発効した判断の記録の本文が封（anchors/adr-seals.yaml）の行と違う（発効した記録の本文は変えない・退役で変えてよいのは status と superseded_by だけ・判断を変えるなら新しい判断の記録を立てる）";
const GONE: &str = "[P-7] FR19 が消えた（baseline の anchors/ids-*.yaml に在る・番号は消さず、廃止は状態で表す・P-7.2）";
const DUP: &str = "[重複キー] rules.yaml: 行 id「R-2」が重複";
const REFUSED: &str = "folio check --proposed: まだ分からない（口は数えていない）";
const INDEX: &str = "[索引の節点] srs.yaml: 索引の節点 FR1 の行を行の逐語で切れない（id か節の見出しの key が引用符つきか裸の形でない＝folio graph --print が組めない）";
const RAIL7: &str = "  - {n: 7, who: folio, what: 組み立てて、見せる, reqs: [FR7]}\n";
const RAIL8: &str = "  - {n: 7, who: folio, what: 組み立てて、見せる, reqs: [FR7]}\n  - {n: 8, who: folio, what: 余分の段, reqs: [FR7]}\n";
const FACE: &str = "[面] srs.yaml.rail: 段が 8 で上限 7（部品目録の pipeline-rail の max_nodes）を超える";

fn copy_tree(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).unwrap();
    for entry in fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let to = dst.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &to);
        } else {
            fs::copy(entry.path(), &to).unwrap();
        }
    }
}

/// git を呼ぶ。環境変数 GIT_* は継承しない。
fn git(cwd: &Path, args: &[&str]) {
    let mut cmd = Command::new("git");
    for (key, _) in std::env::vars_os() {
        if key.to_string_lossy().starts_with("GIT_") {
            cmd.env_remove(key);
        }
    }
    let out = cmd
        .current_dir(cwd)
        .args(["-c", "user.email=fx@example", "-c", "user.name=fx", "-c", "commit.gpgsign=false"])
        .args(args)
        .output()
        .expect("git を起動できない");
    assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
}

/// 置き場の写しの一時 dir（根・歯の終わりに消す）。版管理は根の下の repo/、口の一時の作業場所は根の下の tmp/（版管理の外）。
struct Work(PathBuf);

/// 1 回の起動の結果（終了コード・標準出力の行・標準エラーの行）。
struct Run {
    code: i32,
    out: Vec<String>,
    err: Vec<String>,
}

impl Work {
    fn new(case: &str) -> Work {
        let root = std::env::temp_dir().join(format!("folio-proposed-teeth-{case}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../folio2");
        let place = root.join("repo");
        copy_tree(&repo.join(FLOOR_BASE), &place.join("design-intent"));
        fs::create_dir_all(place.join("contracts")).unwrap();
        fs::copy(repo.join("contracts/schema.toml"), place.join("contracts/schema.toml")).unwrap();
        git(&place, &["init", "-q"]);
        git(&place, &["add", "-A"]);
        git(&place, &["commit", "-q", "-m", "fixture"]);
        fs::create_dir_all(root.join("tmp")).unwrap();
        Work(root)
    }

    fn dir(&self) -> PathBuf {
        self.0.join("repo/design-intent")
    }

    fn folio(&self, args: &[&str], stdin: &[u8]) -> Run {
        self.folio_in(args, stdin, &self.0.join("tmp"))
    }

    /// 口の一時の作業場所の置き場（子の環境の TMPDIR）を `tmp` にして撃つ。
    fn folio_in(&self, args: &[&str], stdin: &[u8], tmp: &Path) -> Run {
        let mut child = Command::new(env!("CARGO_BIN_EXE_tz"))
            .args(["check", "--dir"])
            .arg(self.dir())
            .args(args)
            .env("TMPDIR", tmp)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("folio を起動できない");
        let mut pipe = child.stdin.take().unwrap();
        pipe.write_all(stdin).unwrap();
        drop(pipe);
        let o = child.wait_with_output().unwrap();
        let lines = |b: Vec<u8>| String::from_utf8(b).unwrap().lines().map(str::to_string).collect();
        Run {
            code: o.status.code().unwrap(),
            out: lines(o.stdout),
            err: lines(o.stderr),
        }
    }

    /// 編集時の口（`rel` に `text` を書こうとしている）。
    fn propose(&self, rel: &str, text: &str) -> Run {
        self.folio(&["--proposed", rel], text.as_bytes())
    }

    /// 編集時の口に字（UTF-8）でない byte を渡す。
    fn propose_bytes(&self, rel: &str, bytes: &[u8]) -> Run {
        self.folio(&["--proposed", rel], bytes)
    }

    /// 素の床。
    fn floor(&self) -> Run {
        self.folio(&[], &[])
    }

    fn read(&self, rel: &str) -> String {
        fs::read_to_string(self.dir().join(rel)).unwrap()
    }

    fn write(&self, rel: &str, text: &str) {
        fs::write(self.dir().join(rel), text).unwrap();
    }

    /// `rel` の字の `from` をちょうど 1 か所 `to` に替えた字（置き場は書かない）。
    fn edited(&self, rel: &str, from: &str, to: &str) -> String {
        let text = self.read(rel);
        assert_eq!(text.matches(from).count(), 1, "{rel}: 「{from}」が 1 か所でない");
        text.replacen(from, to, 1)
    }

    /// 写しの中の全 file の字（口が置き場を書かないことを見る）。
    fn snapshot(&self) -> BTreeMap<PathBuf, Vec<u8>> {
        fn walk(dir: &Path, out: &mut BTreeMap<PathBuf, Vec<u8>>) {
            for entry in fs::read_dir(dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    walk(&path, out);
                } else {
                    out.insert(path.clone(), fs::read(&path).unwrap());
                }
            }
        }
        let mut out = BTreeMap::new();
        walk(&self.dir(), &mut out);
        out
    }

    /// 口の一時 dir の置き場に残った file と dir の数。
    fn leftovers(&self) -> usize {
        fs::read_dir(self.0.join("tmp")).unwrap().count()
    }
}

impl Drop for Work {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn bogus(w: &Work) -> String {
    w.edited("rules.yaml", "  - {id: R-2, article: P-14,", "  - {id: R-2, article: P-14, bogus: 1,")
}

/// 要件書の最初の要件（FR1・requirements[0]）の basis に、どこにも無い条 id を足した字。
fn dangling(w: &Work) -> String {
    let text = w.read("srs.yaml");
    let (head, tail) = text.split_once("  - id: FR1\n").unwrap();
    let tail = tail.replacen("    basis: [P-1]\n", "    basis: [P-1, P-99]\n", 1);
    format!("{head}  - id: FR1\n{tail}")
}

/// 便 198 (c) 1: 形の違反を持つ中身は止め（1）、止めた行は同じ中身を書いた置き場の素の床にも同じ字で在る（P-15.2）。口は置き場を書かない。
#[test]
fn f198_stop_line_is_a_line_of_the_floor() {
    let w = Work::new("stop");
    assert_eq!(w.floor().code, 0, "土台の素の床が合格でない");
    let text = bogus(&w);
    let before = w.snapshot();
    let r = w.propose("rules.yaml", &text);
    assert_eq!(r.code, 1, "{:?} {:?}", r.out, r.err);
    assert_eq!(
        r.out,
        [
            BOGUS.to_string(),
            "folio check --proposed: 止める（新しい違反 1・つながり 0・まだ分からない 0・書く前から在る まだ分からない 0・面の段は数えない）".to_string()
        ]
    );
    assert_eq!(w.snapshot(), before, "口が置き場を書いた");
    w.write("rules.yaml", &text);
    let f = w.floor();
    assert_eq!(f.code, 1);
    assert!(f.out.iter().any(|l| l == BOGUS), "素の床に止めた行が無い: {:?}", f.out);
}

/// 便 198 (c) 2: つながりの違反（参照 id の解決）は止めず（0）、標準出力の要約の後に名指し、同じ中身を書いた置き場の素の床は落とす（P-18.2）。
#[test]
fn f198_link_is_not_stopped_but_the_floor_counts_it() {
    let w = Work::new("link");
    let text = dangling(&w);
    let r = w.propose("srs.yaml", &text);
    assert_eq!(r.code, 0, "{:?} {:?}", r.out, r.err);
    assert_eq!(
        r.out,
        [
            "folio check --proposed: 通す（新しい違反 0・つながり 1・まだ分からない 0・書く前から在る まだ分からない 0・面の段は数えない）".to_string(),
            format!("{LINK_HEAD}{DANGLING}")
        ]
    );
    assert!(r.err.is_empty(), "{:?}", r.err);
    w.write("srs.yaml", &text);
    let f = w.floor();
    assert_eq!(f.code, 1);
    assert!(f.out.iter().any(|l| l == DANGLING), "{:?}", f.out);
}

/// 便 198 (c) 3: 書く前から在る違反は新しい違反に数えない（ほかの file の編集も、その違反を直す編集も止めない）。
#[test]
fn f198_old_violations_do_not_stop() {
    let w = Work::new("old");
    let original = w.read("rules.yaml");
    w.write("rules.yaml", &bogus(&w));
    assert_eq!(w.floor().code, 1);
    let r = w.propose("srs.yaml", &w.read("srs.yaml"));
    assert_eq!(r.code, 0, "{:?} {:?}", r.out, r.err);
    let r = w.propose("rules.yaml", &original);
    assert_eq!(r.code, 0, "{:?} {:?}", r.out, r.err);
    assert_eq!(
        r.out,
        ["folio check --proposed: 通す（新しい違反 0・つながり 0・まだ分からない 0・書く前から在る まだ分からない 0・面の段は数えない）".to_string()]
    );
}

/// 便 198 (c) 4: YAML として読めない中身は まだ分からない（2・合格にしない・P-4.1）。
#[test]
fn f198_unreadable_proposal_is_unknown() {
    let w = Work::new("unreadable");
    let r = w.propose("rules.yaml", "a: [\n");
    assert_eq!(r.code, 2, "{:?} {:?}", r.out, r.err);
    assert!(r.out.iter().any(|l| l.starts_with("# まだ分からない: rules.yaml: ")), "{:?}", r.out);
}

/// 便 198 (c) 5: 置き場の外を指す字は数えず まだ分からない（2）で、標準出力に何も出さない。
#[test]
fn f198_outside_the_place_is_unknown() {
    let w = Work::new("outside");
    for rel in ["../contracts/schema.toml", "/etc/hosts", "./rules.yaml"] {
        let r = w.propose(rel, "x: 1\n");
        assert_eq!(r.code, 2, "{rel}: {:?} {:?}", r.out, r.err);
        assert!(r.err.is_empty(), "{rel}: {:?}", r.err);
        assert_eq!(
            r.out,
            [
                format!("# まだ分からない: {rel} は置き場からの相対の file の字でない（絶対 path・.. ・空は数えない）"),
                REFUSED.to_string()
            ]
        );
    }
}

/// 便 198 (c) 6: 新しい file（判断の記録 1 本）も同じ床で数え、止めた行はどれも書いた後の素の床に在る。
#[test]
fn f198_new_file_is_counted_by_the_same_floor() {
    let w = Work::new("new");
    let text = w
        .read("adr/ADR-10.yaml")
        .replacen("id: ADR-10", "id: ADR-11", 1)
        .replacen("status: accepted", "status: bogus", 1);
    let r = w.propose("adr/ADR-11.yaml", &text);
    assert_eq!(r.code, 1, "{:?} {:?}", r.out, r.err);
    let stops: Vec<&String> = r.out.iter().filter(|l| l.starts_with('[')).collect();
    assert!(stops.iter().any(|l| l.starts_with("[adr] ADR-11: status が値域外")), "{:?}", r.out);
    w.write("adr/ADR-11.yaml", &text);
    let f = w.floor();
    for l in stops {
        assert!(f.out.contains(l), "素の床に無い: {l}");
    }
}

/// 便 198 (c) 7: 口の一時 dir は止めた周も通した周も残らない。
#[test]
fn f198_scratch_is_removed() {
    let w = Work::new("scratch");
    assert_eq!(w.propose("rules.yaml", &bogus(&w)).code, 1);
    assert_eq!(w.propose("srs.yaml", &dangling(&w)).code, 0);
    assert_eq!(w.leftovers(), 0);
}

/// 便 198 (c) 8: 索引の床（check_index）も床の 1 本の関数に入る（要件の id を単引用符で囲む中身は止め、素の床にも同じ行が在る）。
#[test]
fn f198_index_floor_is_in_the_same_function() {
    let w = Work::new("index");
    let text = w.edited("srs.yaml", "  - id: FR1\n", "  - id: 'FR1'\n");
    let r = w.propose("srs.yaml", &text);
    assert_eq!(r.code, 1, "{:?} {:?}", r.out, r.err);
    assert_eq!(r.out.first().map(String::as_str), Some(INDEX), "{:?}", r.out);
    w.write("srs.yaml", &text);
    let f = w.floor();
    assert!(f.out.iter().any(|l| l == INDEX), "素の床に索引の行が無い: {:?}", f.out);
}

/// 便 198 (c) 9: 器の導出 file も写しへ写し、設計ノートの契約表の行の欄を同じ床で数える（契約表の行に未知の欄を足す中身は止める）。
#[test]
fn f198_contract_rows_are_counted_with_the_vessel_file() {
    let w = Work::new("contract");
    let text = w.edited(
        "design-note/example.yaml",
        "      - {id: a, title:",
        "      - {id: a, bogus: x, title:",
    );
    let r = w.propose("design-note/example.yaml", &text);
    assert_eq!(r.code, 1, "{:?} {:?}", r.out, r.err);
    assert_eq!(r.out.first().map(String::as_str), Some(CONTRACT), "{:?}", r.out);
    w.write("design-note/example.yaml", &text);
    let f = w.floor();
    assert!(f.out.iter().any(|l| l == CONTRACT), "素の床に契約表の行が無い: {:?}", f.out);
}

/// 便 198 (c) 10: 正本が読めない置き場を直す中身は、写しが版管理の外に在ることの まだ分からない を数えず通す（0）。
/// 書く前から在る まだ分からない（読めない正本）の数は要約に添え、同じ中身を書いた置き場の素の床は合格。
#[test]
fn f198_fixing_an_unreadable_place_is_not_unknown() {
    let w = Work::new("fix");
    let original = w.read("srs.yaml");
    w.write("srs.yaml", &format!("{original}broken: [\n"));
    assert_eq!(w.floor().code, 2);
    let r = w.propose("srs.yaml", &original);
    assert_eq!(r.code, 0, "{:?} {:?}", r.out, r.err);
    assert_eq!(
        r.out,
        ["folio check --proposed: 通す（新しい違反 0・つながり 0・まだ分からない 0・書く前から在る まだ分からない 1・面の段は数えない）".to_string()]
    );
    w.write("srs.yaml", &original);
    assert_eq!(w.floor().code, 0);
}

/// 便 198 (c) 11: 書く先の path が symlink を通れば書かず まだ分からない（2）。写しの外の file は変わらず、外に dir も作らない。
#[test]
fn f198_symlink_on_the_way_is_unknown() {
    let w = Work::new("symlink");
    let outside = w.0.join("outside");
    fs::create_dir_all(&outside).unwrap();
    fs::write(outside.join("v.txt"), "orig\n").unwrap();
    std::os::unix::fs::symlink(&outside, w.dir().join("lnk")).unwrap();
    std::os::unix::fs::symlink(outside.join("v.txt"), w.dir().join("vfile.txt")).unwrap();
    for rel in ["lnk/v.txt", "vfile.txt", "lnk/newdir/x.txt"] {
        let r = w.propose(rel, "changed\n");
        assert_eq!(r.code, 2, "{rel}: {:?} {:?}", r.out, r.err);
        assert!(r.err.is_empty(), "{rel}: {:?}", r.err);
        assert_eq!(
            r.out,
            [format!("# まだ分からない: {rel} は symlink を通る（写しの外を書きうる）"), REFUSED.to_string()]
        );
    }
    assert_eq!(fs::read_to_string(outside.join("v.txt")).unwrap(), "orig\n");
    assert!(!outside.join("newdir").exists());
    assert_eq!(w.leftovers(), 0);
}

/// 便 198 (c) 12: 発効した判断の記録の本文を 1 字変える中身は止め（1・凍結＝封）、止めた行は同じ中身を書いた素の床に在る。
#[test]
fn f198_sealed_body_change_is_stopped() {
    let w = Work::new("seal");
    let text = w.edited("adr/ADR-4.yaml", "\ntitle: ", "\ntitle: X");
    let r = w.propose("adr/ADR-4.yaml", &text);
    assert_eq!(r.code, 1, "{:?} {:?}", r.out, r.err);
    assert_eq!(
        r.out,
        [
            SEAL.to_string(),
            "folio check --proposed: 止める（新しい違反 1・つながり 0・まだ分からない 0・書く前から在る まだ分からない 0・面の段は数えない）".to_string()
        ]
    );
    w.write("adr/ADR-4.yaml", &text);
    assert!(w.floor().out.iter().any(|l| l == SEAL));
}

/// 便 198 (c) 13: id の一覧の凍結 anchor に在る要件を消す中身は止め（1・凍結＝番号の消失）、参照の解決はつながりとして名指すだけ。
#[test]
fn f198_frozen_id_removal_is_stopped() {
    let w = Work::new("gone");
    let srs = w.read("srs.yaml");
    let (head, tail) = srs.split_once("  - id: FR19\n").unwrap();
    let text = format!("{head}{}", &tail[tail.find("nonfunctional:\n").unwrap()..]);
    let r = w.propose("srs.yaml", &text);
    assert_eq!(r.code, 1, "{:?} {:?}", r.out, r.err);
    assert_eq!(r.out.len(), 6, "{:?}", r.out);
    assert_eq!(
        r.out[..2],
        [
            GONE.to_string(),
            "folio check --proposed: 止める（新しい違反 1・つながり 4・まだ分からない 0・書く前から在る まだ分からない 0・面の段は数えない）".to_string()
        ]
    );
    w.write("srs.yaml", &text);
    let f = w.floor();
    assert!(f.out.iter().any(|l| l == GONE));
    // つながりの行は止める行と要約の後で、接頭辞の後ろは同じ中身を書いた置き場の素の床の行と同じ字
    for l in &r.out[2..] {
        let line = l.strip_prefix(LINK_HEAD).unwrap_or_else(|| panic!("つながりの行でない: {l}"));
        assert!(f.out.iter().any(|x| x == line), "素の床に無い: {line}");
    }
}

/// 便 198 (c) 14: 書く前から在る違反と同じ字の違反が 1 つ増える中身は止める（重複ごとに数える差）。
#[test]
fn f198_one_more_of_the_same_words_is_new() {
    let w = Work::new("dup");
    let row = |text: &str| text.lines().find(|l| l.starts_with("  - {id: R-2,")).unwrap().to_string();
    let rules = w.read("rules.yaml");
    let line = row(&rules);
    let once = rules.replacen(&line, &format!("{line}\n{line}"), 1);
    let twice = rules.replacen(&line, &format!("{line}\n{line}\n{line}"), 1);
    w.write("rules.yaml", &once);
    assert_eq!(w.floor().out.iter().filter(|l| *l == DUP).count(), 1);
    let r = w.propose("rules.yaml", &twice);
    assert_eq!(r.code, 1, "{:?} {:?}", r.out, r.err);
    assert_eq!(
        r.out,
        [
            DUP.to_string(),
            "folio check --proposed: 止める（新しい違反 1・つながり 0・まだ分からない 0・書く前から在る まだ分からない 0・面の段は数えない）".to_string()
        ]
    );
}

/// 便 198 (c) 15: 字（UTF-8）でない標準入力は まだ分からない（2・通さない・P-4.1）。
#[test]
fn f198_stdin_not_utf8_is_unknown() {
    let w = Work::new("bytes");
    let r = w.propose_bytes("rules.yaml", &[0xff, 0xfe, b'\n']);
    assert_eq!(r.code, 2, "{:?} {:?}", r.out, r.err);
    assert!(r.err.is_empty(), "{:?}", r.err);
    assert_eq!(r.out.len(), 2, "{:?}", r.out);
    assert!(r.out[0].starts_with("# まだ分からない: 標準入力を字（UTF-8）として読めない: "), "{:?}", r.out);
    assert_eq!(r.out[1], REFUSED);
}

/// 便 198 (c) 16（改訂 b・c）: 口の一時の作業場所の置き場（TMPDIR）が別の版管理の作業ツリーの中でも根そのものでも、写しの床は
/// その版管理を見ない（子の git に一時の作業場所の親を天井に渡す）。器の導出 file の無い別の版管理を読むと契約表の行の欄を数えずに通していた。
#[test]
fn f198_tmpdir_in_another_work_tree_is_not_read() {
    let w = Work::new("ceiling");
    let other = w.0.join("other");
    fs::create_dir_all(&other).unwrap();
    fs::write(other.join("README"), "x\n").unwrap();
    git(&other, &["init", "-q"]);
    git(&other, &["add", "-A"]);
    git(&other, &["commit", "-q", "-m", "other"]);
    let tmp = other.join("tmp");
    fs::create_dir_all(&tmp).unwrap();
    let text = w.edited(
        "design-note/example.yaml",
        "      - {id: a, title:",
        "      - {id: a, bogus: x, title:",
    );
    for at in [&tmp, &other] {
        let r = w.folio_in(&["--proposed", "design-note/example.yaml"], text.as_bytes(), at);
        assert_eq!(r.code, 1, "{at:?}: {:?} {:?}", r.out, r.err);
        assert_eq!(
            r.out,
            [
                CONTRACT.to_string(),
                "folio check --proposed: 止める（新しい違反 1・つながり 0・まだ分からない 0・書く前から在る まだ分からない 0・面の段は数えない）".to_string()
            ]
        );
    }
    assert_eq!(fs::read_dir(&tmp).unwrap().count(), 0);
    let left = fs::read_dir(&other).unwrap().filter(|e| e.as_ref().unwrap().file_name().to_string_lossy().starts_with("folio-proposed-"));
    assert_eq!(left.count(), 0);
}

/// 便 198 (c) 17（改訂 b）: 写しの中で版管理の根が解ける置き場（置き場の dir そのものが版管理の根）は数えず まだ分からない（2）。
#[test]
fn f198_place_that_is_its_own_repo_is_unknown() {
    let w = Work::new("own");
    git(&w.dir(), &["init", "-q"]);
    git(&w.dir(), &["add", "-A"]);
    git(&w.dir(), &["commit", "-q", "-m", "own"]);
    let r = w.propose("rules.yaml", &w.read("rules.yaml"));
    assert_eq!(r.code, 2, "{:?} {:?}", r.out, r.err);
    assert_eq!(
        r.out,
        [
            "# まだ分からない: 一時の作業場所の中で版管理の根が解ける（写しが版管理の中に在る）".to_string(),
            REFUSED.to_string()
        ]
    );
    assert_eq!(w.leftovers(), 0);
}

/// 便 198 (c) 18（改訂 c）: 止める行・まだ分からない の行・要約・つながりの行はこの順に標準出力に出る（器は後ろから切るので
/// つながりの行から落ちる）。標準エラーは空で、どの行も接頭辞の後ろは同じ中身を書いた置き場の素の床の行と同じ字
/// （素の床は まだ分からない の行を標準エラーに出す）。
#[test]
fn f198_lines_come_in_the_order_stop_unknown_summary_link() {
    let w = Work::new("order");
    let text = dangling(&w).replacen("  - id: FR2\n", "  - id: 'FR2'\n", 1);
    let (head, tail) = text.split_once("outputs:\n").unwrap();
    let text = format!("{head}outputs: {{x: 1}}\n{}", &tail[tail.find("\n\n").unwrap() + 1..]);
    let unknown = "srs.yaml: outputs が表の一覧でない（参照 id を集められない）";
    let r = w.propose("srs.yaml", &text);
    assert_eq!(r.code, 2, "{:?} {:?}", r.out, r.err);
    assert!(r.err.is_empty(), "{:?}", r.err);
    assert_eq!(
        r.out,
        [
            INDEX.replace("FR1", "FR2"),
            format!("# まだ分からない: {unknown}"),
            "folio check --proposed: まだ分からない（新しい違反 1・つながり 1・まだ分からない 1・書く前から在る まだ分からない 0・面の段は数えない）".to_string(),
            format!("{LINK_HEAD}{DANGLING}")
        ]
    );
    w.write("srs.yaml", &text);
    let f = w.floor();
    assert_eq!(f.code, 2);
    for l in [INDEX.replace("FR1", "FR2"), DANGLING.to_string()] {
        assert!(f.out.contains(&l), "素の床に無い: {l}");
    }
    assert!(f.err.contains(&format!("# まだ分からない: {unknown}")), "{:?}", f.err);
}

/// 便 198 (c) 19（改訂 d・便 187 の後）: 面を組んで初めて分かる崩れ（便 187 の面の段）は口が止めず（0）、数えていないことを要約の括弧の中で
/// 名乗り（ADR-33 決定 (6)）、同じ中身を書いた置き場の素の床は面の字で落とす。
#[test]
fn f198_face_stage_is_left_to_the_floor() {
    let w = Work::new("faces");
    let text = w.edited("srs.yaml", RAIL7, RAIL8);
    let r = w.propose("srs.yaml", &text);
    assert_eq!(r.code, 0, "{:?} {:?}", r.out, r.err);
    assert_eq!(
        r.out,
        ["folio check --proposed: 通す（新しい違反 0・つながり 0・まだ分からない 0・書く前から在る まだ分からない 0・面の段は数えない）".to_string()]
    );
    assert!(r.err.is_empty(), "{:?}", r.err);
    w.write("srs.yaml", &text);
    let f = w.floor();
    assert_eq!(f.code, 1, "{:?} {:?}", f.out, f.err);
    assert!(f.out.iter().any(|l| l == FACE), "{:?}", f.out);
}

/// 便 199 の場合の 1 つ（置き場からの相対の file・書こうとしている中身を作る関数・つながりの行か止める行の期待）。
type Case = (&'static str, fn(&Work) -> String, &'static [&'static str]);

/// 便 199: 口が止めず（0）つながりの行だけを出し、同じ中身を書いた置き場の素の床はその字で落とす（1）ことを見る。
fn only_links(w: &Work, rel: &str, text: &str, want: &[&str]) {
    let r = w.propose(rel, text);
    assert_eq!(r.code, 0, "{rel}: {:?} {:?}", r.out, r.err);
    assert!(r.out.iter().all(|l| !l.starts_with('[')), "{rel}: 止める行が在る: {:?}", r.out);
    let links: Vec<&str> = r.out.iter().filter_map(|l| l.strip_prefix(LINK_HEAD)).collect();
    assert_eq!(links, want, "{rel}");
    w.write(rel, text);
    let f = w.floor();
    assert_eq!(f.code, 1, "{rel}: {:?}", f.out);
    for l in want {
        assert!(f.out.iter().any(|x| x == l), "{rel}: 素の床に無い: {l}");
    }
}

/// 判断の記録 ADR-10 の字の id を `id` にし、status の行を `status` の字に替えた中身（新しい判断の記録）。
fn adr_like(w: &Work, id: &str, status: &str) -> String {
    w.read("adr/ADR-10.yaml").replacen("id: ADR-10", &format!("id: {id}"), 1).replacen("status: accepted", status, 1)
}

/// 便 199 (c) 1（ADR-33 決定 (2)）: 網の外の関数の中の突き合わせの字（設計ノートの参照 id の解決と別のノートの実在・判断の記録どうしの
/// 後継と前任と欄の決まりの出所・発効した判断の記録の封の欠け・入口と相談窓口と天井の英字の語）はつながりで、口は止めない。
#[test]
fn f199_cross_checks_outside_the_net_are_links() {
    const NOTE: &str = "design-note/example.yaml";
    let cases: [Case; 10] = [
        (NOTE, |w| w.edited(NOTE, "req: [FR15]", "req: [FR15, FR999]"), &["[note] design-note/example.yaml: §6 の行 a の req[1]: id「FR999」が実在しない"]),
        (NOTE, |w| w.edited(NOTE, "  status: example\n", "  status: example\n  supersedes: nosuch\n"), &["[note] design-note/example.yaml: meta.supersedes「nosuch」の設計ノートが実在しない"]),
        (NOTE, |w| w.edited(NOTE, "書き出してはならない（FR15）。", "書き出してはならない（FR15・P-99）。"), &["[note] design-note/example.yaml: §1: 散文の参照 id「P-99」が実在しない"]),
        ("adr/schema.yaml", |w| w.edited("adr/schema.yaml", "decided_by: [ADR-1, ADR-2]", "decided_by: [ADR-1, ADR-2, ADR-99]"), &["[adr] adr/schema.yaml meta.decided_by の ADR-99 が実在しない（欄の決まりの出所の判断が消えている）"]),
        ("adr/ADR-11.yaml", |w| adr_like(w, "ADR-11", "status: proposed\nsupersedes: ADR-99"), &["[adr] ADR-11: supersedes ADR-99 の判断の記録が実在しない", "[adr] ADR-11.supersedes: 判断の記録 ADR-99 が実在しない"]),
        ("adr/ADR-11.yaml", |w| adr_like(w, "ADR-11", "status: retired\nsuperseded_by: ADR-10"), &["[adr] ADR-11: 後継 ADR-10 の supersedes に ADR-11 が無い（双方向）", "[adr] ADR-11: 発効しているのに封の行が無い（folio check --freeze-adrs で封を足し、commit する）"]),
        ("adr/ADR-11.yaml", |w| adr_like(w, "ADR-11", "status: accepted"), &["[adr] ADR-11: 発効しているのに封の行が無い（folio check --freeze-adrs で封を足し、commit する）"]),
        ("index.yaml", |w| w.edited("index.yaml", "  short: 持ち主（非エンジニア）と AI\n", "  short: 持ち主（非エンジニア）と AI と zqword\n"), &["[index] index.yaml audience の short: 語彙に無い英字の語「zqword」"]),
        ("intake.yaml", |w| w.edited("intake.yaml", "  title: 相談窓口（intake）\n", "  title: 相談窓口（intake）zqword\n"), &["[intake] intake.yaml meta の title: 語彙に無い英字の語「zqword」"]),
        ("ceiling.yaml", |w| w.edited("ceiling.yaml", "（観点・所見の欄の決まり・起動の記録）\n", "（観点・所見の欄の決まり・起動の記録）zqword\n"), &["[ceiling] ceiling.yaml meta の title: 語彙に無い英字の語「zqword」"]),
    ];
    for (i, (rel, text, want)) in cases.into_iter().enumerate() {
        let w = Work::new(&format!("x{i}"));
        only_links(&w, rel, &text(&w), want);
    }
}

/// 便 199 (c) 2: retired の後継の列が輪になる中身（相手の ADR-12 は置き場に先に在る）も、口は止めずつながりに名指す。
#[test]
fn f199_retired_cycle_is_a_link() {
    let w = Work::new("cycle");
    w.write("adr/ADR-12.yaml", &adr_like(&w, "ADR-12", "status: retired\nsupersedes: ADR-11\nsuperseded_by: ADR-11"));
    let text = adr_like(&w, "ADR-11", "status: retired\nsupersedes: ADR-12\nsuperseded_by: ADR-12");
    only_links(
        &w,
        "adr/ADR-11.yaml",
        &text,
        &[
            "[adr] ADR-11: retired の後継の列が輪になっている（ADR-11→ADR-12→ADR-11）＝発効している後継が無い（P-7.2）",
            "[adr] ADR-12: retired の後継の列が輪になっている（ADR-12→ADR-11→ADR-12）＝発効している後継が無い（P-7.2）",
            "[adr] ADR-11: 発効しているのに封の行が無い（folio check --freeze-adrs で封を足し、commit する）",
        ],
    );
}

/// 便 199 (c) 3: 同じ関数の中でも 1 つの file の形の字は止める族のまま（設計ノートと判断の記録の retired に後継が無い・節の型の欄が無い）。
/// 計画のノートの外に置いた節の型 row-plan の置き場の字だけがつながり。
#[test]
fn f199_single_file_shapes_stay_stops() {
    const NOTE: &str = "design-note/example.yaml";
    let cases: [(Case, &[&str]); 3] = [
        (
            (NOTE, |w| w.edited(NOTE, "  status: example\n", "  status: retired\n"), &[
                "[note] design-note/example.yaml: meta: status retired なのに approval（承認欄）が無い",
                "[note] design-note/example.yaml: meta: retired なのに superseded_by（後継）が無い（P-7.2）",
            ]),
            &[],
        ),
        (
            ("adr/ADR-11.yaml", |w| adr_like(w, "ADR-11", "status: retired"), &["[adr] ADR-11: retired なのに superseded_by（後継）が無い（P-7.2）"]),
            &["[adr] ADR-11: 発効しているのに封の行が無い（folio check --freeze-adrs で封を足し、commit する）"],
        ),
        (
            (NOTE, |w| w.edited(NOTE, "  - n: 1\n    type: prose\n", "  - n: 1\n    type: row-plan\n"), &[
                "[note] design-note/example.yaml: §1: 節の型 row-plan の欄「rows」が無い",
                "[note] design-note/example.yaml: §6 の行 a: section「1」が同じ文書の prose の節の n でない",
            ]),
            &["[note] design-note/example.yaml: §1: 節の型 row-plan は計画の名札の行（欄 key が plan-note の閾値の行）が名指す計画のノートにだけ置ける（名札の行が無い）"],
        ),
    ];
    for (i, ((rel, text, stops), links)) in cases.into_iter().enumerate() {
        let w = Work::new(&format!("s{i}"));
        let r = w.propose(rel, &text(&w));
        assert_eq!(r.code, 1, "{rel}: {:?} {:?}", r.out, r.err);
        let got: Vec<&str> = r.out.iter().filter(|l| l.starts_with('[')).map(String::as_str).collect();
        assert_eq!(got, stops, "{rel}");
        let got: Vec<&str> = r.out.iter().filter_map(|l| l.strip_prefix(LINK_HEAD)).collect();
        assert_eq!(got, links, "{rel}");
    }
}

/// 便 199: 口が止め（1）、止める行とつながりの行が期待の字の列とちょうど同じで、同じ中身を書いた置き場の素の床にその止める行が在ることを見る。
fn stops_with(w: &Work, rel: &str, text: &str, stops: &[&str], links: &[&str]) {
    let r = w.propose(rel, text);
    assert_eq!(r.code, 1, "{rel}: {:?} {:?}", r.out, r.err);
    let got: Vec<&str> = r.out.iter().filter(|l| l.starts_with('[')).map(String::as_str).collect();
    assert_eq!(got, stops, "{rel}");
    let got: Vec<&str> = r.out.iter().filter_map(|l| l.strip_prefix(LINK_HEAD)).collect();
    assert_eq!(got, links, "{rel}");
    w.write(rel, text);
    let f = w.floor();
    for l in stops {
        assert!(f.out.iter().any(|x| x == l), "{rel}: 素の床に無い: {l}");
    }
}

/// 便 199 (c) 5（検証役の Bb199-1）: 判断の記録が自分自身を後継に指す字（後継の列の輪の長さ 1・後継の supersedes の双方向）は
/// 1 つの file の形で、口は止める。
#[test]
fn f199_adr_pointing_at_itself_stays_a_stop() {
    let w = Work::new("self-adr");
    let text = w.edited("adr/ADR-10.yaml", "status: accepted", "status: retired\nsuperseded_by: ADR-10");
    stops_with(
        &w,
        "adr/ADR-10.yaml",
        &text,
        &[
            "[adr] ADR-10: 後継 ADR-10 の supersedes に ADR-10 が無い（双方向）",
            "[adr] ADR-10: retired の後継の列が輪になっている（ADR-10→ADR-10）＝発効している後継が無い（P-7.2）",
        ],
        &[],
    );
    // supersedes が自分自身（置き換えた記録の superseded_by の双方向）も 1 つの file の形
    let w = Work::new("self-adr-2");
    let text = adr_like(&w, "ADR-11", "status: proposed\nsupersedes: ADR-11");
    stops_with(&w, "adr/ADR-11.yaml", &text, &["[adr] ADR-11: 置き換えた ADR-11 の superseded_by が ADR-11 でない（双方向）"], &[]);
}

/// 便 199 (c) 6（検証役の Bb199-1）: 設計ノートの meta.supersedes が自分自身を指す字は 1 つの file の形で、口は止める。
#[test]
fn f199_note_pointing_at_itself_stays_a_stop() {
    const NOTE: &str = "design-note/example.yaml";
    let w = Work::new("self-note");
    let text = w.edited(NOTE, "  status: example\n", "  status: example\n  supersedes: example\n");
    stops_with(&w, NOTE, &text, &["[note] design-note/example.yaml: meta.supersedes「example」の設計ノートが実在しない"], &[]);
}

/// 便 199 (c) 7（検証役の Bb199-2）: 新しい判断の記録を先に書き（supersedes: ADR-10）、置き換えた ADR-10 の superseded_by を後で書く順は、
/// 途中で双方向の決まりが食い違うだけなので、口は止めずつながりに名指す。
#[test]
fn f199_superseding_before_the_old_record_is_a_link() {
    let w = Work::new("supersedes-first");
    let text = adr_like(&w, "ADR-11", "status: proposed\nsupersedes: ADR-10");
    only_links(&w, "adr/ADR-11.yaml", &text, &["[adr] ADR-11: 置き換えた ADR-10 の superseded_by が ADR-11 でない（双方向）"]);
}
