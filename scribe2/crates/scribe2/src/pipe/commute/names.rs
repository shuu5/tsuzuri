//! 同じ宣言の名を足す組の拾い。
//! 候補の差と相手の差の 1 本ずつが足す名を拾い、候補と 相手の 1 本が同じ名を足せば [`Verdict::SameName`] で断る。
//! 名は字で拾い、拾いすぎは断る側に倒れる。
//! 出所: 判断の記録 ADR-60

use super::{Round, Verdict};
use crate::fleet::json_tree::{parse, Tree};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// 宣言の語（Rust の関数・型・定数・mod）。
const KEYWORDS: [&str; 9] = ["fn", "struct", "enum", "union", "trait", "type", "const", "static", "mod"];

/// 宣言の語の前に立つ修飾の語（`const fn` の `const` は別に見る・`extern "C"` の ABI の字は `"` で始まる語）。
const QUALIFIERS: [&str; 4] = ["async", "unsafe", "extern", "default"];

/// 配列の要素の鍵の道の 1 段。
const ELEMENT: &str = "[]";

/// 差が足す名（形ごと・JSON は file の path を添える）。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Name {
    /// Rust の宣言の名。
    Rust(String),
    /// JSON の file と鍵の道。
    Json(String, Vec<String>),
    /// CSS の最上段の規則の選び手の字。
    Css(String),
}

/// 差の file の 1 本の file の頭（`diff --git a/<元> b/<先>` と、新しい file か消す file か）。
struct Target {
    /// 元の path（main の先端の木の字を読む）。
    from: String,
    /// 先の path（差を当てた index の字を読む）。
    to: String,
    /// 新しい file（前の字は無い）。
    new: bool,
    /// 消す file（後の字は無い）。
    gone: bool,
}

impl Round<'_> {
    /// 名の段: 候補の差 `own` と相手の差 `theirs` の 1 本が同じ名を足せば same-name、どれかの差の名を読めない周は
    /// unreadable、ほかは commutes。
    pub(super) fn names(&self, own: &Path, theirs: &[PathBuf]) -> Verdict {
        let Some(mine) = self.added(own) else {
            return Verdict::Unreadable;
        };
        for path in theirs {
            match self.added(path) {
                None => return Verdict::Unreadable,
                Some(found) if !found.is_disjoint(&mine) => return Verdict::SameName,
                Some(_) => {}
            }
        }
        Verdict::Commutes
    }

    /// 差 1 本が足す名。JSON と CSS の file は、main の先端の木の字（新しい file は無し）と、main の先端にその差だけを当てた
    /// index の字を比べる（消す file は拾わない）。差を読めない・file の頭を読めない・当たらない・字を読めない周は `None`。
    fn added(&self, patch: &Path) -> Option<BTreeSet<Name>> {
        let text = std::fs::read_to_string(patch).ok()?;
        let mut found = rust(&text);
        let shaped: Vec<Target> = targets(&text)?
            .into_iter()
            .filter(|target| !target.gone && (target.to.ends_with(".json") || target.to.ends_with(".css")))
            .collect();
        if shaped.is_empty() {
            return Some(found);
        }
        (self.applies(&[patch]) == Verdict::Commutes).then_some(())?;
        for (at, target) in shaped.iter().enumerate() {
            let before = if target.new { None } else { Some(self.copy(self.ask.main, &target.from, &format!("b{at}"))?.1) };
            let after = self.copy("", &target.to, &format!("a{at}"))?.1;
            if target.to.ends_with(".json") {
                found.extend(json(&target.to, before.as_deref(), &after)?);
            } else {
                found.extend(css(before.as_deref(), &after));
            }
        }
        Some(found)
    }
}

/// 差の file の本文で、`.rs` の file の足す行（`+` の行）が宣言する名。
fn rust(patch: &str) -> BTreeSet<Name> {
    let mut rs = false;
    let mut out = BTreeSet::new();
    for line in patch.lines() {
        if let Some(head) = line.strip_prefix("diff --git ") {
            rs = head.ends_with(".rs");
        } else if let Some(name) = line.strip_prefix('+').filter(|_| rs).and_then(declared) {
            out.insert(Name::Rust(name.to_owned()));
        }
    }
    out
}

/// 1 行が宣言する名（属性と pub とその範囲を剥ぎ、修飾の語の後の頭の語が宣言の語なら、次の語の頭の識別子・`static mut`
/// は mut の次）。
fn declared(line: &str) -> Option<&str> {
    let mut words = visible(line)?.split_whitespace().peekable();
    while let Some(word) = words.next() {
        let qualified = words.peek().is_some_and(|next| *next == "fn" || QUALIFIERS.contains(next));
        if QUALIFIERS.contains(&word) || word.starts_with('"') || (word == "const" && qualified) {
            continue;
        }
        if !KEYWORDS.contains(&word) {
            return None;
        }
        let mut name = words.next()?;
        if word == "static" && name == "mut" {
            name = words.next()?;
        }
        let end = name.find(|ch: char| !(ch.is_alphanumeric() || ch == '_')).unwrap_or(name.len());
        return name.get(..end).filter(|found| !found.is_empty());
    }
    None
}

/// 行の頭の属性（`#[…]`）と pub とその範囲（`pub(…)`）を剥いだ字（閉じない属性と範囲は `None`）。
fn visible(line: &str) -> Option<&str> {
    let mut rest = line.trim_start();
    while let Some(attr) = rest.strip_prefix("#[") {
        rest = attr.split_once(']')?.1.trim_start();
    }
    match rest.strip_prefix("pub") {
        Some(scope) if scope.starts_with('(') => Some(scope.split_once(')')?.1),
        Some(after) if after.starts_with(char::is_whitespace) => Some(after),
        _ => Some(rest),
    }
}

/// 差の file の file の頭の列（`a/` と ` b/` で割れない `diff --git` の行〔引用符で囲んだ path ほか〕が在れば `None`）。
fn targets(patch: &str) -> Option<Vec<Target>> {
    let mut out: Vec<Target> = Vec::new();
    for line in patch.lines() {
        if let Some(head) = line.strip_prefix("diff --git ") {
            let (from, to) = head.strip_prefix("a/")?.split_once(" b/")?;
            out.push(Target { from: from.to_owned(), to: to.to_owned(), new: false, gone: false });
        } else if let Some(last) = out.last_mut() {
            last.new |= line.starts_with("new file mode ");
            last.gone |= line.starts_with("deleted file mode ");
        }
    }
    Some(out)
}

/// file `file` の JSON の鍵の道のうち、前の字（無い file は `None`）に無く後の字に在る物（どちらかを JSON として読めない時は
/// `None`）。
fn json(file: &str, before: Option<&str>, after: &str) -> Option<BTreeSet<Name>> {
    let old = match before {
        Some(text) => keys(&parse(text).ok()?),
        None => BTreeSet::new(),
    };
    let new = keys(&parse(after).ok()?);
    Some(new.difference(&old).map(|path| Name::Json(file.to_owned(), path.clone())).collect())
}

/// 値の鍵の道の全部。
fn keys(tree: &Tree) -> BTreeSet<Vec<String>> {
    let mut out = BTreeSet::new();
    walk(tree, &mut Vec::new(), &mut out);
    out
}

/// 鍵の道を `at` の下へ辿って `out` に足す（深さは [`parse`] の上限で止まる）。
fn walk(tree: &Tree, at: &mut Vec<String>, out: &mut BTreeSet<Vec<String>>) {
    match tree {
        Tree::Object(pairs) => {
            for (key, value) in pairs {
                at.push(key.clone());
                out.insert(at.clone());
                walk(value, at, out);
                at.pop();
            }
        }
        Tree::Array(items) => {
            for item in items {
                at.push(ELEMENT.to_owned());
                walk(item, at, out);
                at.pop();
            }
        }
        Tree::Str(_) | Tree::Num(_) | Tree::Bool(_) | Tree::Null => {}
    }
}

/// CSS の最上段の規則の選び手の字のうち、前の字（無い file は `None`）に無く後の字に在る物。
fn css(before: Option<&str>, after: &str) -> BTreeSet<Name> {
    let old = before.map(selectors).unwrap_or_default();
    selectors(after).difference(&old).cloned().map(Name::Css).collect()
}

/// 最上段の規則の選び手の字（注を除き、最上段の `{` の前の字を空白を 1 つに詰めて・`;` で終わる最上段の文は捨てる）。
fn selectors(text: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let (mut depth, mut current) = (0_usize, String::new());
    for ch in uncommented(text).chars() {
        match ch {
            '{' => {
                if depth == 0 {
                    out.insert(current.split_whitespace().collect::<Vec<&str>>().join(" "));
                }
                depth = depth.saturating_add(1);
                current.clear();
            }
            '}' => {
                depth = depth.saturating_sub(1);
                current.clear();
            }
            ';' if depth == 0 => current.clear(),
            _ if depth == 0 => current.push(ch),
            _ => {}
        }
    }
    out
}

/// 注（`/* … */`）を除いた字（閉じない注は末まで除く）。
fn uncommented(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some((head, tail)) = rest.split_once("/*") {
        out.push_str(head);
        rest = tail.split_once("*/").map_or("", |(_, after)| after);
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::{css, json, rust, Name};
    use crate::pipe::commute::{judge, Ask, Partner, Verdict};
    use crate::pipe::declaration::RootsAtHead;
    use crate::pipe::fixture::scratch;
    use crate::pipe::{git_bytes, git_line, git_ok};
    use std::collections::BTreeSet;
    use std::path::{Path, PathBuf};
    use std::time::{Duration, Instant};

    /// 面の中の file（固定の根 `crates/` の下）。
    const RS: &str = "crates/a/src/lib.rs";
    const OTHER: &str = "crates/a/src/other.rs";
    const VOCAB: &str = "crates/a/vocab.json";
    const STYLE: &str = "crates/a/style.css";
    /// 差の file の dir。
    const DIR: &str = "docs/design/patch";
    /// 固定の根だけの宣言。
    static FIXED: RootsAtHead = RootsAtHead::Fixed;

    /// 差 1 本（名・（file, 後の中身）の列）。名が `c` で始まる差は main、ほかは base に置く。
    type Spec<'a> = (&'a str, Vec<(&'a str, String)>);

    /// 場（repo・state dir・base と main の sha）。
    struct Place {
        repo: PathBuf,
        state: PathBuf,
        base: String,
        main: String,
    }

    /// 30 行の file（行 n は `l<n>`・行 `at` だけ `text`）。
    fn rs(at: usize, text: &str) -> String {
        (1..=30_usize).map(|n| if n == at { format!("{text}\n") } else { format!("l{n}\n") }).collect()
    }

    /// 語の辞書の項目 1 つ（label・plain・internal の 3 つの鍵・末に `,`）。
    fn entry(key: &str) -> String {
        format!("  \"{key}\": {{\n   \"label\": \"{key}\",\n   \"plain\": \"p\",\n   \"internal\": \"i\"\n  }},\n")
    }

    /// 語の辞書（english の下に鍵 label だけを持つ項目 e1〜e6）。`first` を e1 の前に、`last` を e6 の前に足す。
    fn vocab(first: &str, last: &str) -> String {
        let middle: String = (1..=5).map(|n| format!("  \"e{n}\": {{\n   \"label\": \"e{n}\"\n  }},\n")).collect();
        format!("{{\n \"english\": {{\n{first}{middle}{last}  \"e6\": {{\n   \"label\": \"e6\"\n  }}\n }}\n}}\n")
    }

    /// 語の辞書の部品の file（english の下に項目 `key` 1 つ・rephrase は空）。
    fn part(key: &str) -> String {
        format!("{{\"english\": {{\"{key}\": {{\"label\": \"{key}\", \"plain\": \"p\", \"internal\": \"i\"}}}}, \"rephrase\": {{}}}}\n")
    }

    /// stylesheet（規則 .r1〜.r8・各 3 行）。`first` を頭に、`last` を末に足す。
    fn style(first: &str, last: &str) -> String {
        let rules: String = (1..=8).map(|n| format!(".r{n} {{\n  color: red;\n}}\n")).collect();
        format!("{first}{rules}{last}")
    }

    fn put(repo: &Path, file: &str, text: &[u8]) {
        let path = repo.join(file);
        assert!(path.parent().is_some_and(|dir| std::fs::create_dir_all(dir).is_ok()) && std::fs::write(&path, text).is_ok(), "{file}");
    }

    fn commit(repo: &Path, message: &str) -> String {
        assert!(git_ok(repo, &["add", "-A"]) && git_ok(repo, &["commit", "-q", "--allow-empty", "-m", message]), "{message}");
        git_line(repo, &["rev-parse", "HEAD"]).unwrap_or_default()
    }

    /// seed の後に、相手の差の file を足した base と、相手の差の file を消して候補の差の file を足した main を積む。
    fn place(name: &str, specs: &[Spec<'_>]) -> Place {
        let root = scratch(name);
        let (repo, state) = (root.join("repo"), root.join("state"));
        let setup: [&[&str]; 4] =
            [&["init", "-q", "-b", "main"], &["config", "user.name", "t"], &["config", "user.email", "t@example.invalid"], &["config", "commit.gpgsign", "false"]];
        assert!(std::fs::create_dir_all(&repo).is_ok() && setup.iter().all(|args| git_ok(&repo, args)));
        for (file, text) in [(RS, rs(0, "")), (OTHER, rs(0, "")), (VOCAB, vocab("", "")), (STYLE, style("", ""))] {
            put(&repo, file, text.as_bytes());
        }
        commit(&repo, "seed");
        let mut made: Vec<(&str, Vec<u8>)> = Vec::new();
        for (patch, edits) in specs {
            edits.iter().for_each(|(file, text)| put(&repo, file, text.as_bytes()));
            assert!(git_ok(&repo, &["add", "-A"]), "{patch}");
            let form = ["diff", "--cached", "--no-color", "--no-ext-diff", "--src-prefix=a/", "--dst-prefix=b/"];
            made.push((patch, git_bytes(&repo, &form).unwrap_or_default()));
            let back: [&[&str]; 3] = [&["reset", "-q"], &["checkout", "--", "crates"], &["clean", "-fdq", "crates"]];
            assert!(back.iter().all(|args| git_ok(&repo, args)), "{patch}");
        }
        let file = |patch: &str| format!("{DIR}/{patch}.patch");
        made.iter().filter(|(patch, _)| !patch.starts_with('c')).for_each(|(patch, text)| put(&repo, &file(patch), text));
        let base = commit(&repo, "base");
        for (patch, text) in &made {
            if patch.starts_with('c') {
                put(&repo, &file(patch), text);
            } else {
                assert!(std::fs::remove_file(repo.join(file(patch))).is_ok(), "{patch}");
            }
        }
        let main = commit(&repo, "main");
        Place { repo, state, base, main }
    }

    fn files(paths: &[&str]) -> Vec<(String, String)> {
        paths.iter().map(|path| ((*path).to_owned(), (*path).to_owned())).collect()
    }

    /// 候補 `own` と相手（名の列・base から読む）の判じ。交わる項は `crossed`。
    fn verdict(place: &Place, own: &str, partners: &[&str], crossed: &[(String, String)]) -> Verdict {
        let own = format!("{DIR}/{own}.patch");
        let paths: Vec<String> = partners.iter().map(|patch| format!("{DIR}/{patch}.patch")).collect();
        let list: Vec<Partner<'_>> = paths.iter().map(|patch| Partner { patch: Some(patch), base: &place.base, crossed }).collect();
        let deadline = Instant::now() + Duration::from_secs(60);
        let ask = Ask { repo: &place.repo, state_dir: &place.state, main: &place.main, roots: &FIXED, patch: Some(&own), partners: &list, deadline };
        judge(&ask)
    }

    fn rust_names(names: &[&str]) -> BTreeSet<Name> {
        names.iter().map(|name| Name::Rust((*name).to_owned())).collect()
    }

    /// Rust の名は `.rs` の file の足す行の宣言だけを拾う（文脈と消す行・宣言でない行・ほかの file の行は拾わない）。
    #[test]
    fn vcnames_rust_declarations_on_added_lines_of_rs_files() {
        let patch = [
            "diff --git a/crates/a/src/lib.rs b/crates/a/src/lib.rs",
            "--- a/crates/a/src/lib.rs",
            "+++ b/crates/a/src/lib.rs",
            "@@ -1,2 +1,21 @@",
            " fn context() {}",
            "-fn removed() {}",
            "+pub fn alpha(x: u8) -> u8 {",
            "+pub(crate) struct Beta<T>(T);",
            "+    enum Gamma {",
            "+union Delta { a: u8 }",
            "+pub(in crate::pipe) trait Epsilon {}",
            "+type Zeta = u8;",
            "+const ETA: u8 = 1;",
            "+static mut THETA: u8 = 0;",
            "+mod iota;",
            "+#[test] fn kappa() {}",
            "+pub const unsafe fn lambda() {}",
            "+unsafe extern \"C\" fn mu() {}",
            "+async fn nu() {}",
            "+let fn_like = 1;",
            "+use crate::xi;",
            "+impl Omicron for Pi {}",
            "+    name: String,",
            "diff --git a/docs/a.md b/docs/a.md",
            "--- a/docs/a.md",
            "+++ b/docs/a.md",
            "@@ -1 +1 @@",
            "+fn prose() {}",
        ]
        .join("\n");
        let want = ["alpha", "Beta", "Gamma", "Delta", "Epsilon", "Zeta", "ETA", "THETA", "iota", "kappa", "lambda", "mu", "nu"];
        assert_eq!(rust(&patch), rust_names(&want));
    }

    /// JSON の鍵の道は前の字に無く後の字に在る物だけ（値の替えと在った鍵は拾わない・新しい file は全部・読めない字は `None`）。
    #[test]
    fn vcnames_json_key_paths_new_in_the_after_tree() {
        let before = r#"{"english": {"run": {"label": "r", "plain": "p"}}, "tags": [{"k": 1}]}"#;
        let after = r#"{"english": {"run": {"label": "R", "plain": "p", "internal": "i"}, "alpha": {"label": "a"}}, "tags": [{"k": 1}, {"k": 2, "m": 3}]}"#;
        let path = |keys: &[&str]| Name::Json("v.json".to_owned(), keys.iter().map(|key| (*key).to_owned()).collect());
        let want = BTreeSet::from([
            path(&["english", "run", "internal"]),
            path(&["english", "alpha"]),
            path(&["english", "alpha", "label"]),
            path(&["tags", "[]", "m"]),
        ]);
        assert_eq!(json("v.json", Some(before), after), Some(want), "前後の差");
        let all = BTreeSet::from([
            path(&["english"]),
            path(&["english", "run"]),
            path(&["english", "run", "label"]),
            path(&["english", "run", "plain"]),
            path(&["tags"]),
            path(&["tags", "[]", "k"]),
        ]);
        assert_eq!(json("v.json", None, before), Some(all), "新しい file");
        assert_eq!(json("v.json", Some(before), "{"), None, "後を読めない");
        assert_eq!(json("v.json", Some("{"), after), None, "前を読めない");
    }

    /// CSS の名は最上段の規則の選び手の字の全部で、前の字に無い物だけ（注・入れ子・`;` の文は拾わない）。
    #[test]
    fn vcnames_css_top_level_selectors_new_in_the_after_tree() {
        let before = ".run { color: red; }\n@media (max-width: 600px) {\n  .inner { color: blue; }\n}\n";
        let after = concat!(
            "@import \"x.css\";\n.run { color: green; }\n/* .ghost { } */\n.alpha   .x,\n.y { color: red; }\n",
            "@media (max-width: 600px) {\n  .inner { color: blue; }\n  .nested { color: red; }\n}\n.alpha { }\n"
        );
        let names = |found: &[&str]| found.iter().map(|text| Name::Css((*text).to_owned())).collect::<BTreeSet<Name>>();
        assert_eq!(css(Some(before), after), names(&[".alpha .x, .y", ".alpha"]), "前後の差");
        assert_eq!(css(None, before), names(&[".run", "@media (max-width: 600px)"]), "新しい file");
    }

    /// N2 の見本: label・plain・internal を足す別の項目の 2 本は通し（plain と internal は 2 本とも新しい鍵）、同じ鍵の道を足す 2 本は
    /// 断る。
    #[test]
    fn vcnames_json_n2_entries_pass_and_the_same_key_path_refuses() {
        let place = place("vcnames-json", &[
            ("c", vec![(VOCAB, vocab(&entry("alpha"), ""))]),
            ("p", vec![(VOCAB, vocab("", &entry("omega")))]),
            ("p-same", vec![(VOCAB, vocab("", &entry("alpha")))]),
            ("p-broken", vec![(VOCAB, vocab("", "  \"omega\": oops,\n"))]),
            ("p-quoted", vec![(VOCAB, vocab("", &entry("omega"))), ("crates/a/語.json", "{}\n".to_owned())]),
            ("cp", vec![(RS, rs(5, "c5")), ("crates/a/vocab/alpha.json", part("alpha"))]),
            ("q", vec![(RS, rs(20, "p20")), ("crates/a/vocab/omega.json", part("omega"))]),
        ]);
        let crossed = files(&[VOCAB]);
        assert_eq!(verdict(&place, "c", &["p"], &crossed), Verdict::Commutes, "別の項目");
        assert_eq!(verdict(&place, "c", &["p-same"], &crossed), Verdict::SameName, "同じ鍵の道");
        assert_eq!(verdict(&place, "c", &["p-broken"], &crossed), Verdict::Unreadable, "相手の後の字を読めない");
        assert_eq!(verdict(&place, "c", &["p-quoted"], &crossed), Verdict::Unreadable, "引用符で囲んだ path の頭");
        assert_eq!(verdict(&place, "cp", &["q"], &files(&[RS])), Verdict::Commutes, "部品の file 2 本の english と rephrase");
    }

    /// Rust の同じ名は別の file でも 2 本目の相手でも断り、当たらない組は名の前に not-commuting。CSS は選び手の字の全部を file を
    /// 問わず比べる。
    #[test]
    fn vcnames_rust_and_css_same_names_refuse_after_the_four_ways() {
        let place = place("vcnames-rust-css", &[
            ("c", vec![(RS, rs(5, "pub fn alpha() {}"))]),
            ("p", vec![(RS, rs(20, "p20")), (OTHER, rs(5, "fn omega() {}"))]),
            ("p-same", vec![(RS, rs(20, "p20")), (OTHER, rs(5, "fn alpha() {}"))]),
            ("p-late", vec![(RS, rs(25, "p25")), (OTHER, rs(20, "fn alpha() {}"))]),
            ("p-clash", vec![(RS, rs(5, "fn alpha() {}"))]),
            ("cs", vec![(STYLE, style(".alpha .x {\n  color: red;\n}\n", ""))]),
            ("q", vec![(STYLE, style("", ".omega {\n}\n"))]),
            ("q-prefix", vec![(STYLE, style("", ".alpha {\n}\n"))]),
            ("q-same", vec![(STYLE, style("", ".alpha   .x {\n}\n"))]),
            ("q-other", vec![(STYLE, style("", ".omega {\n}\n")), ("crates/a/style/q.css", ".alpha .x {\n}\n".to_owned())]),
        ]);
        let crossed = files(&[RS]);
        assert_eq!(verdict(&place, "c", &["p"], &crossed), Verdict::Commutes, "別の名");
        assert_eq!(verdict(&place, "c", &["p-same"], &crossed), Verdict::SameName, "別の file の同じ名");
        assert_eq!(verdict(&place, "c", &["p", "p-late"], &crossed), Verdict::SameName, "2 本目の相手");
        assert_eq!(verdict(&place, "c", &["p-clash"], &crossed), Verdict::NotCommuting, "当たらない組");
        let crossed = files(&[STYLE]);
        assert_eq!(verdict(&place, "cs", &["q"], &crossed), Verdict::Commutes, "別の選び手");
        assert_eq!(verdict(&place, "cs", &["q-prefix"], &crossed), Verdict::Commutes, "選び手の字の全部");
        assert_eq!(verdict(&place, "cs", &["q-same"], &crossed), Verdict::SameName, "空白を詰めて同じ");
        assert_eq!(verdict(&place, "cs", &["q-other"], &crossed), Verdict::SameName, "別の file の同じ選び手");
    }
}
