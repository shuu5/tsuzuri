//! write-set の項の畳みと交差（親 [`super`] の断りが読む・設計 contract-source.md §3）。

/// `path` が write-set のどれかに含まれるか（dir は配下全部・file は字面の一致・照合は正規化した形）。
///
/// 契約表の閉包 ⊆ write-set の判定（設計 contract-source.md §3）が使う。畳み方は [`overlaps`] と同じ 1 本である。
pub(crate) fn covered(write_set: &[String], path: &str) -> bool {
    let target = normalize(path);
    write_set.iter().any(|item| covers(&normalize(item), &target))
}

/// 2 つの write-set が交差した**全組**（`(左の字面, 右の字面)`・先頭が理由の 1 行に載る）。
///
/// **照合は正規化した形で、返すのは契約が書いた字面のまま**である（読み手が自分の契約の
/// どの行を直せばよいかは、器が畳んだ形ではなく書いた字面でしか分からない）。
///
/// dir 項目は `tracked`（base の tracked file の repo 相対 path）に**展開してから**数える（設計
/// contract-source.md §3）: dir × dir は段の境目の prefix、dir × file はその file が base に在って配下の周だけ、
/// file × file は字面の一致（`+` 接頭辞の新規 file は剥がして比べる）。base に無い file と dir は交差しない
/// （dir で書いた snapshot の置き場が、配下に新規 file 1 つを持つ別便と偽の交差を起こした `s2-07l.243 × .248` の型）。
///
/// `pub(crate)` なのは、回答で write-set を広げる周（`.133`）が**同じ 1 本**を呼ぶためである
/// （本便は口だけを置き、answer への配線は `.133`）。
pub(crate) fn overlaps(left: &[String], right: &[String], tracked: &[String]) -> Vec<(String, String)> {
    let base: Vec<String> = tracked.iter().map(|path| normalize(path)).collect();
    let mut found = Vec::new();
    for mine in left {
        for theirs in right {
            if touches(&normalize(mine), &normalize(theirs), &base) {
                found.push((mine.clone(), theirs.clone()));
            }
        }
    }
    found
}

/// 新規 file の項目の接頭辞（設計 contract-source.md §3「項目の実在と展開」）。
pub(crate) const NEW_FILE: char = '+';

/// 縮む面の項目の接頭辞（設計 contract-source.md §3「項目の実在と展開」・base に在る file を減らす便が宣言する）。
/// 受付の宣言だけの文法で、guard と交差の照合は素の path で持つ。
pub(crate) const SHRINK_FILE: char = '-';

/// 着地で消える file の項目の接頭辞（設計 contract-source.md §24・受付は base に**実在する** file を要し、契約表の
/// 検査は tracked に無ければ着地で消えたと読む）。[`SHRINK_FILE`] と同じく受付の宣言だけの文法で、guard と交差の
/// 照合と gate の write-set 照合は素の path で持つ（消す file は触る file）。
pub(crate) const DELETE_FILE: char = '~';

/// 中身を変えない・verify の置き場として載せただけの項目の接頭辞（設計 contract-source.md §43 (1)・行 ar・base に
/// **実在する** file を要する）。[`SHRINK_FILE`] と同じく受付の宣言だけの文法で、上限の余地も core の見積の本数も
/// 求めない。guard と交差の照合と gate の write-set 照合は素の path で持つ。
pub(crate) const PLACE_ONLY_FILE: char = '=';

/// path 1 本を字面で畳む（write-set guard の `relative_to` と同じ規則）。
///
/// 先頭の `./` を落とす・連続する `/` を 1 つにする・`..` を畳む・**末尾の `/` は dir の印
/// として残す**・接頭辞（新規 file の `+`・縮む面の `-`・消える file の `~`・置き場だけの `=`）は剥がす。root の外へ出る `..`
/// （畳めない分）はそのまま残す
/// ＝字面が違うものを同じ path に化けさせない。**存在は見ない**ので、まだ無い file を書く契約も同じ規則で測れる。
///
/// `pub(crate)` なのは、spawn が guard へ写す policy（`spawn::write_policy`）が**同じ 1 本**で接頭辞を剥がすため
/// である（剥がす規則を 2 か所に持たない）。
pub(crate) fn normalize(raw: &str) -> String {
    let raw = raw.strip_prefix([NEW_FILE, SHRINK_FILE, DELETE_FILE, PLACE_ONLY_FILE]).unwrap_or(raw);
    let is_dir = raw.ends_with('/');
    let mut parts: Vec<&str> = Vec::new();
    for part in raw.split('/') {
        match part {
            "" | "." => {}
            ".." if parts.last().is_some_and(|last| *last != "..") => {
                parts.truncate(parts.len().saturating_sub(1));
            }
            name => parts.push(name),
        }
    }
    let joined = parts.join("/");
    if is_dir && !joined.is_empty() {
        format!("{joined}/")
    } else {
        joined
    }
}

/// 正規化した 2 本が交差するか（**対称**）。dir × dir は段の境目の prefix・dir × file は base に在る配下の file
/// だけ・file × file は字面の一致（`base` は正規化済みの tracked path の列）。
fn touches(left: &str, right: &str, base: &[String]) -> bool {
    match (left.ends_with('/'), right.ends_with('/')) {
        (true, true) => covers(left, right) || covers(right, left),
        (true, false) => in_base(right, base) && covers(left, right),
        (false, true) => in_base(left, base) && covers(right, left),
        (false, false) => left == right,
    }
}

/// 正規化した file が base の tracked file に在るか。
fn in_base(file: &str, base: &[String]) -> bool {
    base.iter().any(|path| path == file)
}

/// `left` が `right` を含むか。dir（末尾 `/`）は配下を全部含み、file は字面の一致だけ。
///
/// `a/` は `a/b.rs` と `a` を含み、`ab/` は含まない（prefix の比較を段の境目で切る）。
fn covers(left: &str, right: &str) -> bool {
    match left.strip_suffix('/') {
        Some(dir) => {
            right == dir || right.strip_prefix(dir).is_some_and(|rest| rest.starts_with('/'))
        }
        None => left == right,
    }
}

#[cfg(test)]
mod tests {
    // flip-check: moved t3-hub.92.10.6
    use super::{covered, normalize, overlaps};
    use proptest::prelude::*;
    use proptest::test_runner::Config;

    /// 反例の永続化を切り、case 数を 256 に pin する（`tests/e2e/prop.rs` と同じ形）。
    fn config() -> Config {
        Config {
            cases: 256,
            failure_persistence: None,
            ..Config::default()
        }
    }

    /// 閉包の file が write-set に含まれるか: dir（末尾 `/`）は配下全部・file は字面の一致・正規化してから比べる。
    #[test]
    fn refuse_covered_follows_the_overlap_folding() {
        let set = vec!["src/".to_owned(), "docs/a.md".to_owned()];
        for (path, want) in [("src/x.rs", true), ("src/deep/y.rs", true), ("./docs/a.md", true), ("docs/b.md", false), ("srcx/z.rs", false)] {
            assert_eq!(covered(&set, path), want, "{path}");
        }
        assert!(!covered(&["src".to_owned()], "src/x.rs"), "末尾 / 無しは dir として配下を含まない");
    }

    /// base の tracked file（交差の表の fixture）。
    fn tracked() -> Vec<String> {
        ["a/b.rs", "src/x.rs", "src/a.rs", "src/b.rs"].iter().map(|path| (*path).to_owned()).collect()
    }

    /// 設計 §2 の表（dir と file・正規化の 3 形）と dir の展開（contract-source.md §3・新規 file と dir は交差しない）
    /// を字面で測る。
    #[test]
    fn refuse_overlap_table_follows_the_design() {
        let set = |item: &str| vec![item.to_owned()];
        let base = tracked();
        for (left, right, want) in [
            ("a/", "a/b.rs", true),
            ("a/", "ab/", false),
            ("a/", "a/c/", true),
            ("./a/b.rs", "a/b.rs", true),
            ("a//b.rs", "a/b.rs", true),
            ("src/../src/x.rs", "src/x.rs", true),
            ("src/a.rs", "src/b.rs", false),
            // dir は base の file 一覧に展開して数える＝新規 file（`+`）と base に無い file は配下でも交差しない。
            ("a/", "+a/new.rs", false),
            ("a/", "a/new.rs", false),
            ("+a/b.rs", "a/b.rs", true),
            ("+a/new.rs", "a/new.rs", true),
            // 縮む面（`-`）も素の path で照合する＝同じ file を減らす便と書く便は交差する。
            ("-a/b.rs", "a/b.rs", true),
            ("a/", "-a/b.rs", true),
        ] {
            let found = !overlaps(&set(left), &set(right), &base).is_empty();
            assert_eq!(found, want, "{left} × {right}");
            // 返るのは**契約が書いた字面のまま**（器が畳んだ形ではない）。
            if want {
                assert_eq!(
                    overlaps(&set(left), &set(right), &base).first().cloned(),
                    Some((left.to_owned(), right.to_owned())),
                    "{left} × {right} の組は字面のまま"
                );
            }
        }
        assert_eq!(normalize("./src//../src/x.rs"), "src/x.rs", "正規化の 3 形を畳む");
        assert_eq!(normalize("src/"), "src/", "末尾の / は dir の印として残る");
        assert_eq!(normalize("+src/new.rs"), "src/new.rs", "新規 file の接頭辞は剥がす");
        assert_eq!(normalize("-src/big.rs"), "src/big.rs", "縮む面の接頭辞も剥がす（交差の照合は素の path）");
    }

    /// 置き場だけの `=`（§43 (1)・行 ar）も `normalize` の 1 本が既存 3 形の隣で剥がす: 4 形とも同じ素の path に畳まれ、
    /// 交差と guard の照合（[`covered`]）は素の path の項目と同じ面を触ると読む。剥がすのは先頭の 1 字だけ。
    #[test]
    fn contract_place_only_normalize_strips_the_mark_next_to_the_other_three() {
        let marked = ["+src/a.rs", "-src/a.rs", "~src/a.rs", "=src/a.rs"];
        let stripped: Vec<String> = marked.iter().map(|item| normalize(item)).collect();
        assert_eq!(stripped, vec!["src/a.rs".to_owned(); 4], "4 形とも素の path（母集団 {} 形）", marked.len());
        assert_eq!(normalize("=src/"), "src/", "dir の印は残る");
        assert_eq!(normalize("==src/a.rs"), "=src/a.rs", "剥がすのは 1 字だけ");
        let place = vec!["=src/a.rs".to_owned()];
        assert!(covered(&place, "src/a.rs"), "= の項目は素の path を覆う（guard の照合は不変）");
        let crossed = overlaps(&place, &["src/a.rs".to_owned()], &tracked());
        assert_eq!(crossed, vec![("=src/a.rs".to_owned(), "src/a.rs".to_owned())], "交差は素の path で数え、字面のまま返る");
    }

    /// 交差の全組が返る（1 組で止めない＝stderr に全組を並べる材料）。
    #[test]
    fn refuse_overlap_returns_every_pair() {
        let mine = vec!["src/a.rs".to_owned(), "src/b.rs".to_owned()];
        let theirs = vec!["src/".to_owned(), "docs/x.md".to_owned()];
        let found = overlaps(&mine, &theirs, &tracked());
        assert_eq!(found.len(), 2, "2 組とも返る: {found:?}");
        assert_eq!(found.first().cloned(), Some(("src/a.rs".to_owned(), "src/".to_owned())));
    }

    /// path の 1 本。**段に `..` を含む形も空間に入れる**（畳む規則を性質で測るため）。
    fn path() -> impl Strategy<Value = String> {
        segments(vec!["a", "b", "ab", "src", ".."])
    }

    /// `..` を含まない path の 1 本（前置した dir が畳まれない＝prefix が効く形）。
    fn plain_path() -> impl Strategy<Value = String> {
        segments(vec!["a", "b", "ab", "src"])
    }

    /// 段の候補から path を 1 本組む（末尾の `/` の有無も振る）。
    fn segments(choices: Vec<&'static str>) -> impl Strategy<Value = String> {
        (
            prop::collection::vec(prop::sample::select(choices), 1..4),
            any::<bool>(),
        )
            .prop_map(|(parts, dir)| {
                let joined = parts.join("/");
                if dir {
                    format!("{joined}/")
                } else {
                    joined
                }
            })
    }

    /// write-set 1 つ分（1〜3 本）。
    fn write_set() -> impl Strategy<Value = Vec<String>> {
        prop::collection::vec(path(), 1..4)
    }

    /// `..` を持たない write-set 1 つ分。
    fn plain_write_set() -> impl Strategy<Value = Vec<String>> {
        prop::collection::vec(plain_path(), 1..4)
    }

    proptest! {
        #![proptest_config(config())]

        /// 判定は**対称**である（どちらを新しい契約として撃っても同じ答え）。base は右の file 項目を全部持つ形で振る。
        #[test]
        fn prop_refuse_overlap_is_symmetric(left in write_set(), right in write_set()) {
            let base: Vec<String> = right.iter().filter(|item| !item.ends_with('/')).cloned().collect();
            let forward = overlaps(&left, &right, &base).is_empty();
            let backward = overlaps(&right, &left, &base).is_empty();
            prop_assert_eq!(forward, backward);
        }

        /// 非空の集合は**自分自身と交差する**（同じ契約の 2 本目は必ず掛かる・base が空でも）。
        #[test]
        fn prop_refuse_nonempty_set_overlaps_itself(set in write_set()) {
            prop_assert!(!overlaps(&set, &set, &[]).is_empty());
        }

        /// 正規化で結果が変わらない（`./` 前置・`//` 重複・`x/../` の挿入）。
        #[test]
        fn prop_refuse_normalization_does_not_change_the_answer(left in write_set(), right in write_set()) {
            let base: Vec<String> = right.iter().filter(|item| !item.ends_with('/')).cloned().collect();
            let want = overlaps(&left, &right, &base).is_empty();
            let dotted: Vec<String> = left.iter().map(|item| format!("./{item}")).collect();
            let doubled: Vec<String> = left.iter().map(|item| item.replace('/', "//")).collect();
            let hopped: Vec<String> = left.iter().map(|item| format!("q/../{item}")).collect();
            for decorated in [dotted, doubled, hopped] {
                prop_assert_eq!(overlaps(&decorated, &right, &base).is_empty(), want);
            }
        }

        /// 共通 prefix を持たない 2 集合は交差しない（`..` で外へ出る形は前置の外）。
        #[test]
        fn prop_refuse_disjoint_prefixes_never_overlap(left in plain_write_set(), right in plain_write_set()) {
            let mine: Vec<String> = left.iter().map(|item| format!("left/{item}")).collect();
            let theirs: Vec<String> = right.iter().map(|item| format!("right/{item}")).collect();
            let base: Vec<String> = mine.iter().chain(&theirs).cloned().collect();
            prop_assert!(overlaps(&mine, &theirs, &base).is_empty());
        }
    }
}
