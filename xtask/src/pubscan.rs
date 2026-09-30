//! 公開の repo へ漏らす字の走査（行 t-pub-scan・判断の記録 ADR-18 の決定 (9)）。
//! 追跡される file の作業の木の今の字と、基準の commit `BASE` より後の本流の commit（HEAD から 1 本目の親だけを辿る列）の作者・記録者の欄と本文に、
//! tailnet の住所の形（100.64.0.0/10 と fd7a:115c:a1e0::/48 の中）・tailnet の名の形（字 .ts.net で終わる名）・
//! 一覧の語（口座の名札と host の名・追跡しない file から読む）を探す。当たりは場所と行の番号と種類だけを出し、当たった字は出さない。
//! merge で持ち込んだ別の根の commit は読まない（持ち込む repo の全履歴は、持ち込む前に席が同じ規則で走査する）。持ち込んだ木の file は全部見る。
//! 当たりの在る commit が main に入ったら、`BASE` をその commit へ進める（走査を止める旗・環境変数・除外の一覧は持たない・条 N-3）。

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::emit_err;

/// 基準の commit（判断の記録 ADR-18 と計画の判断の表 d44 を足した commit・公開を記した commit）。これより後の commit を見る。
pub const BASE: &str = "244d0e2c0f208f18b31f25731b68ca429600894e";

/// 一覧の file の名（表示先の設定の file と同じ dir に置く・版管理に書かない）。
pub const LIST_FILE: &str = "pub-scan.txt";

/// IPv4 の範囲そのものを書いた字（後に数字が続かなければ当てない）。
const V4_RANGE: &str = "100.64.0.0/10";

/// IPv6 の範囲そのものを書いた字（後に数字が続かなければ当てない）。
const V6_RANGE: &str = "fd7a:115c:a1e0::/48";

/// IPv6 の住所の頭（頭の 48 bit が tailnet の範囲）。
const V6_HEAD: &str = "fd7a:115c:a1e0:";

/// tailnet の名の終わり。
const NAME_TAIL: &str = ".ts.net";

/// git の走査の外から渡る変数（撃つ root の repo を読むために外す）。
const GIT_ENV: [&str; 3] = ["GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE"];

/// 当たりの種類。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    /// tailnet の IPv4 の住所の形。
    Address,
    /// tailnet の IPv6 の住所の形。
    Address6,
    /// tailnet の名の形。
    Name,
    /// 一覧の語（一覧の何語目かを 1 から）。
    Listed(usize),
}

impl Kind {
    /// 出力の名（当たった字を含まない）。
    fn label(self) -> String {
        match self {
            Kind::Address => "address".to_string(),
            Kind::Address6 => "address6".to_string(),
            Kind::Name => "name".to_string(),
            Kind::Listed(n) => format!("listed {n}"),
        }
    }
}

/// 当たり 1 つ（場所は file の path か commit の id・行の番号は 1 から）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Hit {
    pub place: String,
    pub line: usize,
    pub kind: Kind,
}

/// 字を英字の小文字にしてから 4 種を探し、字の中の順に（行の番号・種類）を返す。
pub(crate) fn find(text: &str, words: &[String]) -> Vec<(usize, Kind)> {
    let text = text.to_ascii_lowercase();
    let bytes = text.as_bytes();
    let mut found: Vec<(usize, Kind)> = Vec::new();
    for i in 0..bytes.len() {
        if v4_at(bytes, i) {
            found.push((i, Kind::Address));
        }
        if v6_at(bytes, i) {
            found.push((i, Kind::Address6));
        }
    }
    for (at, _) in text.match_indices(NAME_TAIL) {
        let before = at.checked_sub(1).and_then(|j| bytes.get(j).copied());
        let after = bytes.get(at + NAME_TAIL.len()).copied();
        if before.is_some_and(name_char) && !after.is_some_and(name_char) {
            found.push((at, Kind::Name));
        }
    }
    for (k, word) in words.iter().enumerate() {
        let word = word.to_ascii_lowercase();
        if word.is_empty() {
            continue;
        }
        for (at, _) in text.match_indices(word.as_str()) {
            let before = at.checked_sub(1).and_then(|j| bytes.get(j).copied());
            let after = bytes.get(at + word.len()).copied();
            if !before.is_some_and(|b| b.is_ascii_alphanumeric())
                && !after.is_some_and(|b| b.is_ascii_alphanumeric())
            {
                found.push((at, Kind::Listed(k + 1)));
            }
        }
    }
    found.sort_by_key(|(at, _)| *at);
    let starts: Vec<usize> = std::iter::once(0)
        .chain(text.match_indices('\n').map(|(j, _)| j + 1))
        .collect();
    found
        .into_iter()
        .map(|(at, kind)| (starts.partition_point(|&s| s <= at), kind))
        .collect()
}

/// 名の字か（英数字か字 -）。
fn name_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-'
}

/// 範囲そのものを書いた字（後に数字が続かない）が i から始まるか。
fn range_at(bytes: &[u8], i: usize, range: &str) -> bool {
    bytes
        .get(i..)
        .is_some_and(|rest| rest.starts_with(range.as_bytes()))
        && !bytes
            .get(i + range.len())
            .is_some_and(|b| b.is_ascii_digit())
}

/// i から tailnet の IPv4 の住所の形が始まるか。
fn v4_at(bytes: &[u8], i: usize) -> bool {
    let prev = i.checked_sub(1).and_then(|j| bytes.get(j));
    if prev.is_some_and(|b| b.is_ascii_digit() || *b == b'.') {
        return false;
    }
    let mut at = i;
    let mut parts = [0u32; 4];
    for (n, part) in parts.iter_mut().enumerate() {
        if n > 0 {
            if bytes.get(at) != Some(&b'.') {
                return false;
            }
            at += 1;
        }
        let digits = bytes
            .iter()
            .skip(at)
            .take_while(|b| b.is_ascii_digit())
            .count();
        if !(1..=3).contains(&digits) {
            return false;
        }
        *part = bytes
            .iter()
            .skip(at)
            .take(digits)
            .fold(0, |v, b| v * 10 + u32::from(b - b'0'));
        at += digits;
    }
    let dot_digit =
        bytes.get(at) == Some(&b'.') && bytes.get(at + 1).is_some_and(|b| b.is_ascii_digit());
    !dot_digit
        && parts[0] == 100
        && (64..=127).contains(&parts[1])
        && parts.iter().all(|&p| p <= 255)
        && !range_at(bytes, i, V4_RANGE)
}

/// i から tailnet の IPv6 の住所の形が始まるか。
fn v6_at(bytes: &[u8], i: usize) -> bool {
    let prev = i.checked_sub(1).and_then(|j| bytes.get(j));
    if prev.is_some_and(|b| b.is_ascii_hexdigit() || *b == b':') {
        return false;
    }
    bytes
        .get(i..)
        .is_some_and(|rest| rest.starts_with(V6_HEAD.as_bytes()))
        && bytes
            .get(i + V6_HEAD.len())
            .is_some_and(|b| b.is_ascii_hexdigit() || *b == b':')
        && !range_at(bytes, i, V6_RANGE)
}

/// 一覧の file の path（境界の crate の表示先の設定の file の名を `LIST_FILE` に替える）。
pub(crate) fn list_path(xdg_config_home: Option<&OsStr>, home: Option<&OsStr>) -> Option<PathBuf> {
    tsuzuri_boundary::stage::target::path(xdg_config_home, home).map(|p| p.with_file_name(LIST_FILE))
}

/// 一覧を 1 行 1 語で読む（空の行と字 # で始まる行を除き、前後の空白を除いて小文字にする）。
/// file が無ければ None、読めない file は誤り。
pub(crate) fn read_list(path: &Path) -> Result<Option<Vec<String>>, String> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(format!("{} を読めない: {e}", path.display())),
    };
    Ok(Some(
        text.lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(str::to_ascii_lowercase)
            .collect(),
    ))
}

/// root の repo で git を撃ち、標準出力を返す（落ちれば誤り）。
fn git(root: &Path, args: &[&str]) -> Result<Vec<u8>, String> {
    let mut cmd = Command::new("git");
    cmd.args(args).current_dir(root);
    for var in GIT_ENV {
        cmd.env_remove(var);
    }
    let out = cmd
        .output()
        .map_err(|e| format!("git を起動できない: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "git {} が落ちた: {}",
            args.first().copied().unwrap_or(""),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(out.stdout)
}

/// 追跡される file の作業の木の字と、base より後の本流の commit の作者・記録者・本文を探す。
/// 本流は HEAD から 1 本目の親だけを辿る列（`--first-parent`）で、merge の commit は本流に在るので見る。
/// merge で持ち込んだ側の根の commit は読まない（持ち込む repo の全履歴は持ち込む前に席が走査する）。
/// 作業の木の file は持ち込んだ木のも全部見る。
/// commit の行の番号は 1 行目が作者・2 行目が記録者・3 行目から本文。作業の木から消した file は飛ばす。
pub(crate) fn scan(root: &Path, base: &str, words: &[String]) -> Result<Vec<Hit>, String> {
    let mut hits = Vec::new();
    let files = git(root, &["ls-files", "-z"])?;
    for name in files.split(|&b| b == 0).filter(|n| !n.is_empty()) {
        let name = String::from_utf8_lossy(name).into_owned();
        let bytes = match std::fs::read(root.join(&name)) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => return Err(format!("{name} を読めない: {e}")),
        };
        for (line, kind) in find(&String::from_utf8_lossy(&bytes), words) {
            hits.push(Hit {
                place: name.clone(),
                line,
                kind,
            });
        }
    }
    let range = format!("{base}..HEAD");
    let log = git(
        root,
        &[
            "log",
            "-z",
            "--first-parent",
            "--format=%H%n%an <%ae>%n%cn <%ce>%n%B",
            &range,
        ],
    )?;
    for record in log.split(|&b| b == 0).filter(|r| !r.is_empty()) {
        let record = String::from_utf8_lossy(record);
        let (id, rest) = record.split_once('\n').unwrap_or((record.as_ref(), ""));
        for (line, kind) in find(rest, words) {
            hits.push(Hit {
                place: id.to_string(),
                line,
                kind,
            });
        }
    }
    Ok(hits)
}

/// 一覧を読み、root の repo を走査して当たりを 1 つ 1 行で出す（当たりが在れば 1・無ければ 0・走査できなければ 1）。
pub fn run(root: &Path) -> i32 {
    let path = list_path(
        std::env::var_os("XDG_CONFIG_HOME").as_deref(),
        std::env::var_os("HOME").as_deref(),
    );
    let list = match path.as_deref().map(read_list) {
        Some(Ok(list)) => list,
        None => None,
        Some(Err(e)) => {
            emit_err(&format!("xtask pub-scan: 一覧: {e}"));
            return 1;
        }
    };
    let words = match list {
        Some(words) => {
            emit_err(&format!("xtask pub-scan: 一覧の語 {} 語", words.len()));
            words
        }
        None => {
            emit_err("xtask pub-scan: 一覧の file が無いので形だけを見る");
            Vec::new()
        }
    };
    let hits = match scan(root, BASE, &words) {
        Ok(hits) => hits,
        Err(e) => {
            emit_err(&format!("xtask pub-scan: 走査できない: {e}"));
            return 1;
        }
    };
    for hit in &hits {
        emit_err(&format!(
            "xtask pub-scan: {}:{}: {}",
            hit.place,
            hit.line,
            hit.kind.label()
        ));
    }
    emit_err(&format!("xtask pub-scan: 当たり {}", hits.len()));
    i32::from(!hits.is_empty())
}

#[cfg(test)]
mod tests {
    use super::{BASE, Hit, Kind, LIST_FILE, V4_RANGE, V6_RANGE, find, list_path, read_list, scan};
    use std::ffi::OsStr;
    use std::net::{Ipv4Addr, Ipv6Addr};
    use std::path::{Path, PathBuf};
    use std::process::Command;

    const NONE: &[String] = &[];

    fn v4(a: [u8; 4]) -> String {
        Ipv4Addr::from(a).to_string()
    }

    fn v6(a: [u16; 8]) -> String {
        Ipv6Addr::from(a).to_string()
    }

    /// 見本の名（字で書くとこの file が走査に当たるので組む）。
    fn name() -> String {
        ["fx-node", "example", "ts", "net"].join(".")
    }

    fn words() -> Vec<String> {
        vec!["fxhost7".to_string(), "fx-acct".to_string()]
    }

    fn root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("workspace の root")
            .to_path_buf()
    }

    #[test]
    fn pubscan_address_range() {
        let head = v4([100, 64, 0, 0]);
        assert_eq!(V4_RANGE, format!("{head}/10"));
        for a in [
            [100, 64, 0, 1],
            [100, 100, 100, 100],
            [100, 127, 255, 254],
            [100, 64, 0, 0],
        ] {
            let s = v4(a);
            for hit in [
                s.clone(),
                format!("at {s}."),
                format!("x{s}"),
                format!("({s}:8080)"),
            ] {
                assert_eq!(find(&hit, NONE), vec![(1, Kind::Address)], "{hit}");
            }
            assert_eq!(
                find(&format!("a\nb\n住所 {s}"), NONE),
                vec![(3, Kind::Address)]
            );
            for miss in [format!("1{s}"), format!(".{s}"), format!("{s}.5")] {
                assert_eq!(find(&miss, NONE), vec![], "{miss}");
            }
        }
        assert_eq!(find(&format!("{head}/100"), NONE), vec![(1, Kind::Address)]);
        assert_eq!(find(&format!("{head}/10"), NONE), vec![]);
        assert_eq!(find(&format!("範囲は {head}/10。"), NONE), vec![]);
        for a in [
            [100, 63, 255, 255],
            [100, 128, 0, 1],
            [10, 64, 0, 1],
            [101, 64, 0, 1],
        ] {
            let s = v4(a);
            assert_eq!(find(&s, NONE), vec![], "{s}");
        }
        for miss in [
            format!("{}.{}.{}.{}", 100, "0064", 0, 1),
            format!("{}.{}.{}.{}", 100, 64, 0, 1000),
            format!("{}.{}.{}.{}", 100, 64, 256, 1),
            format!("{}.{}.{}.{}", 100, 64, 0, 256),
        ] {
            assert_eq!(find(&miss, NONE), vec![], "{miss}");
        }
    }

    #[test]
    fn pubscan_address6_range() {
        let zero = v6([0xfd7a, 0x115c, 0xa1e0, 0, 0, 0, 0, 0]);
        assert_eq!(V6_RANGE, format!("{zero}/48"));
        for a in [
            [0xfd7a, 0x115c, 0xa1e0, 0, 0, 0, 0, 1],
            [0xfd7a, 0x115c, 0xa1e0, 0xab12, 0x4843, 0xcd96, 0x6258, 0xb240],
            [0xfd7a, 0x115c, 0xa1e0, 0, 0, 0, 0, 0],
        ] {
            let s = v6(a);
            for hit in [s.clone(), s.to_ascii_uppercase(), format!("[{s}]:80")] {
                assert_eq!(find(&hit, NONE), vec![(1, Kind::Address6)], "{hit}");
            }
            for miss in [format!("0{s}"), format!("f{s}"), format!(":{s}")] {
                assert_eq!(find(&miss, NONE), vec![], "{miss}");
            }
        }
        assert_eq!(find(&format!("{zero}/48"), NONE), vec![]);
        assert_eq!(find(&format!("範囲は {zero}/48。"), NONE), vec![]);
        assert_eq!(
            find(&format!("{zero}/480"), NONE),
            vec![(1, Kind::Address6)]
        );
        for a in [
            [0xfd7a, 0x115c, 0xa1e1, 0, 0, 0, 0, 1],
            [0xfd7b, 0x115c, 0xa1e0, 0, 0, 0, 0, 1],
        ] {
            let s = v6(a);
            assert_eq!(find(&s, NONE), vec![], "{s}");
        }
        let head = format!("{:x}:{:x}:{:x}:", 0xfd7a, 0x115c, 0xa1e0);
        for miss in [head.clone(), format!("{head} x"), format!("{head}g")] {
            assert_eq!(find(&miss, NONE), vec![], "{miss}");
        }
    }

    #[test]
    fn pubscan_name_shape() {
        let n = name();
        let tail = ["", "ts", "net"].join(".");
        for hit in [
            n.clone(),
            n.to_ascii_uppercase(),
            format!("see {n}."),
            format!("me@{n}>"),
            format!("a-{tail}"),
        ] {
            assert_eq!(find(&hit, NONE), vec![(1, Kind::Name)], "{hit}");
        }
        for miss in [
            format!("*{tail}"),
            tail.clone(),
            format!(" {tail}"),
            format!("{n}x"),
            format!("{n}-a"),
            format!("{n}0"),
        ] {
            assert_eq!(find(&miss, NONE), vec![], "{miss}");
        }
    }

    #[test]
    fn pubscan_listed_words() {
        let w = words();
        for (text, want) in [
            ("on fxhost7 now", Kind::Listed(1)),
            ("FXHOST7", Kind::Listed(1)),
            ("a/fxhost7-b", Kind::Listed(1)),
            ("日本fxhost7語", Kind::Listed(1)),
            ("by FX-Acct.", Kind::Listed(2)),
        ] {
            assert_eq!(find(text, &w), vec![(1, want)], "{text}");
        }
        for text in ["xfxhost7", "fxhost7x", "fxhost70", "1fx-acct", "fx-accts"] {
            assert_eq!(find(text, &w), vec![], "{text}");
        }
        assert_eq!(
            find("fx-acct\nfxhost7 fx-acct", &w),
            vec![
                (1, Kind::Listed(2)),
                (2, Kind::Listed(1)),
                (2, Kind::Listed(2))
            ]
        );
        assert_eq!(find("fxhost7 fx-acct", NONE), vec![]);
    }

    #[test]
    fn pubscan_list_file() {
        assert_eq!(LIST_FILE, "pub-scan.txt");
        let home = OsStr::new("/h");
        assert_eq!(
            list_path(Some(OsStr::new("/x/cfg")), Some(home)),
            Some(PathBuf::from("/x/cfg/tsuzuri/pub-scan.txt"))
        );
        assert_eq!(
            list_path(Some(OsStr::new("rel/cfg")), Some(home)),
            Some(PathBuf::from("/h/.config/tsuzuri/pub-scan.txt"))
        );
        assert_eq!(
            list_path(None, Some(home)),
            Some(PathBuf::from("/h/.config/tsuzuri/pub-scan.txt"))
        );
        let dir = std::env::temp_dir().join(format!("tsuzuri-pubscan-list-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("一時の dir");
        let file = dir.join(LIST_FILE);
        assert_eq!(read_list(&file), Ok(None));
        std::fs::write(&file, "# 注\n\n  FXhost7  \nfx-acct\n   \n#fxhost8\n").expect("一覧");
        assert_eq!(read_list(&file), Ok(Some(words())));
        assert!(read_list(&dir).is_err(), "dir は読めない file");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// 見本の repo で git を撃つ（利用者の git の設定を読まない）。
    fn git_in(dir: &Path, args: &[&str], env: &[(&str, &str)]) -> String {
        let out = Command::new("git")
            .args(args)
            .current_dir(dir)
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .env_remove("GIT_INDEX_FILE")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_AUTHOR_NAME", "fx")
            .env("GIT_AUTHOR_EMAIL", "fx@example.invalid")
            .env("GIT_COMMITTER_NAME", "fx")
            .env("GIT_COMMITTER_EMAIL", "fx@example.invalid")
            .envs(env.iter().copied())
            .output()
            .expect("git を撃つ");
        assert!(out.status.success(), "git {args:?}: {out:?}");
        String::from_utf8(out.stdout).expect("git の出力").trim().to_string()
    }

    #[test]
    fn pubscan_repo_range() {
        let dir = std::env::temp_dir().join(format!("tsuzuri-pubscan-repo-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("一時の dir");
        let addr = v4([100, 64, 0, 1]);
        let n = name();
        git_in(&dir, &["init", "-q"], &[]);
        std::fs::write(dir.join("a.txt"), "one\ntwo\non fxhost7\n").expect("a.txt");
        std::fs::write(dir.join("gone.txt"), &addr).expect("gone.txt");
        git_in(&dir, &["add", "a.txt", "gone.txt"], &[]);
        git_in(&dir, &["commit", "-q", "-m", &format!("base {addr}")], &[]);
        let base = git_in(&dir, &["rev-parse", "HEAD"], &[]);
        let author = format!("a@{n}");
        let committer = format!("c@{n}");
        git_in(
            &dir,
            &["commit", "-q", "--allow-empty", "-m", &format!("later\n\nsee {n}")],
            &[
                ("GIT_AUTHOR_EMAIL", &author),
                ("GIT_COMMITTER_EMAIL", &committer),
            ],
        );
        let later = git_in(&dir, &["rev-parse", "HEAD"], &[]);
        std::fs::remove_file(dir.join("gone.txt")).expect("消す");
        std::fs::write(dir.join("untracked.txt"), &addr).expect("追跡しない file");

        let hits = scan(&dir, &base, &words()).expect("走査");
        let hit = |place: &str, line, kind| Hit {
            place: place.to_string(),
            line,
            kind,
        };
        assert_eq!(
            hits,
            vec![
                hit("a.txt", 3, Kind::Listed(1)),
                hit(&later, 1, Kind::Name),
                hit(&later, 2, Kind::Name),
                hit(&later, 5, Kind::Name),
            ]
        );
        let missing = "1".repeat(40);
        assert!(scan(&dir, &missing, &words()).is_err(), "無い基準の commit");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn pubscan_repo_tree_clean() {
        assert_eq!(BASE.len(), 40);
        let hits = scan(&root(), BASE, NONE).expect("走査");
        assert_eq!(hits, vec![]);
    }

    #[test]
    fn pubfp_side_root_commits_skipped() {
        let dir = std::env::temp_dir().join(format!("tsuzuri-pubfp-repo-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("一時の dir");
        let n = name();
        git_in(&dir, &["init", "-q", "-b", "main"], &[]);
        std::fs::write(dir.join("a.txt"), "one\n").expect("a.txt");
        git_in(&dir, &["add", "a.txt"], &[]);
        git_in(&dir, &["commit", "-q", "-m", "base"], &[]);
        let base = git_in(&dir, &["rev-parse", "HEAD"], &[]);
        git_in(
            &dir,
            &["commit", "-q", "--allow-empty", "-m", &format!("trunk {n}")],
            &[],
        );
        let trunk = git_in(&dir, &["rev-parse", "HEAD"], &[]);
        git_in(&dir, &["checkout", "-q", "--orphan", "side"], &[]);
        git_in(&dir, &["rm", "-q", "-rf", "."], &[]);
        std::fs::write(dir.join("b.txt"), format!("one\nsee {n}\n")).expect("b.txt");
        git_in(&dir, &["add", "b.txt"], &[]);
        git_in(&dir, &["commit", "-q", "-m", &format!("side {n}")], &[]);
        let side = git_in(&dir, &["rev-parse", "HEAD"], &[]);
        git_in(&dir, &["checkout", "-q", "main"], &[]);
        git_in(
            &dir,
            &[
                "merge",
                "-q",
                "--allow-unrelated-histories",
                "-m",
                &format!("merge {n}"),
                "side",
            ],
            &[],
        );
        let merge = git_in(&dir, &["rev-parse", "HEAD"], &[]);
        let parents = git_in(&dir, &["rev-list", "--parents", "-n", "1", "HEAD"], &[]);
        assert_eq!(parents, format!("{merge} {trunk} {side}"));

        let hits = scan(&dir, &base, &words()).expect("走査");
        let hit = |place: &str, line| Hit {
            place: place.to_string(),
            line,
            kind: Kind::Name,
        };
        assert_eq!(
            hits,
            vec![hit("b.txt", 2), hit(&merge, 3), hit(&trunk, 3)]
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn pubscan_wiring() {
        const MAIN: &str = include_str!("main.rs");
        let (_, check) = MAIN.split_once("fn check(").expect("fn check");
        let scan_at = check.find("pubscan::run(root)").expect("check が走査を撃つ");
        let steps_at = check
            .find("for step in CHECK_STEPS")
            .expect("cargo の段の繰り返し");
        assert!(scan_at < steps_at, "走査は cargo の段より前");
        assert!(
            MAIN.contains(r#"Some("pub-scan") if args.len() == 1 =>"#),
            "task pub-scan の腕"
        );
        let ci = std::fs::read_to_string(root().join(".github/workflows/ci.yml")).expect("ci.yml");
        let step: Vec<&str> = ci
            .lines()
            .skip_while(|l| !l.contains("actions/checkout@"))
            .enumerate()
            .take_while(|(i, l)| *i == 0 || !l.trim_start().starts_with("- "))
            .map(|(_, l)| l)
            .collect();
        assert!(!step.is_empty(), "checkout の step");
        assert!(
            step.iter().any(|l| l
                .split_once(':')
                .is_some_and(|(k, v)| k.trim() == "fetch-depth" && v.trim() == "0")),
            "{step:?}"
        );
    }
}
