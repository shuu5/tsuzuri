//! 便ごとの版の記録（行 xp-provenance・判断の記録 ADR-37 の決定 (7)）。
//!
//! runner と lens は claude を起こす直前に 4 語を組む（[`probe`]）: 器の binary の組みの commit（[`BUILD_COMMIT`] の字のまま）・
//! claude の CLI の版（`<claude> --version` を 1 回撃った stdout の版の語）・claude に渡す model と effort。runner は stderr の
//! 1 行（[`line`]）で、lens は消費の 6 値を運ぶ判定 object の 1 対（[`with_object`]）で呼び手へ渡し、pipe の spawn と gate が
//! 消費の event の `detail` に囲いの書きの語に続けて写す（[`detail`]）。版を読めない周（撃てない・rc ≠ 0・版の語でない）は
//! 字 `unmeasured`、行や対が無いか形の違う周は 4 語とも `unmeasured` と書き、黙って落とさない（C10）。

use super::runner::has_top_level_key;
use super::{claude_program, Call};
use crate::fleet::json_lite::{self, Value};
use crate::name::BUILD_COMMIT;
use std::process::Stdio;

/// 測れない語の字。
pub const UNMEASURED: &str = "unmeasured";

/// 4 語とも測れない周の字（行も対も無い・形の違う周）。
pub const UNMEASURED_WORDS: &str = "build:unmeasured claude:unmeasured model:unmeasured effort:unmeasured";

/// runner の stderr の行の頭。
pub const LINE_HEAD: &str = "runner: provenance ";

/// lens の判定 object の key。
pub const KEY: &str = "provenance";

/// 4 語の頭（この順）。
const HEADS: [&str; 4] = ["build:", "claude:", "model:", "effort:"];

/// `call` の claude を起こす直前の 4 語（[`version_of`] を 1 回撃つ）。
pub fn probe(call: &Call<'_>) -> String {
    let version = version_of(call.claude);
    words(version.as_deref(), call.model.unwrap_or(UNMEASURED), call.effort.unwrap_or(UNMEASURED))
}

/// `<claude> --version` を 1 回撃ち、rc 0 の周だけ stdout の版の語（[`version_word`]）を返す（stdin と stderr は null）。
pub fn version_of(claude: &str) -> Option<String> {
    let mut command = claude_program(claude);
    command.arg("--version").stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::null());
    let out = command.output().ok().filter(|found| found.status.success())?;
    version_word(&String::from_utf8_lossy(&out.stdout)).map(str::to_owned)
}

/// stdout の最初の行の最初の語が数字で始まり数字と点だけなら版の語。
pub fn version_word(stdout: &str) -> Option<&str> {
    let word = stdout.lines().next()?.split_whitespace().next()?;
    let digits = word.chars().all(|found| found.is_ascii_digit() || found == '.');
    (digits && word.starts_with(|found: char| found.is_ascii_digit())).then_some(word)
}

/// 4 語（`build:<commit> claude:<版|unmeasured> model:<model> effort:<effort>`）。
pub fn words(version: Option<&str>, model: &str, effort: &str) -> String {
    format!("build:{BUILD_COMMIT} claude:{} model:{model} effort:{effort}", version.unwrap_or(UNMEASURED))
}

/// runner の stderr の 1 行。
pub fn line(words: &str) -> String {
    format!("{LINE_HEAD}{words}")
}

/// 4 語の形（空白で割って 4 語・[`HEADS`] の頭をこの順に持ち値が空でない）の字だけを返す。
pub fn checked(text: &str) -> Option<&str> {
    let found: Vec<&str> = text.split(' ').collect();
    let shaped = found.len() == HEADS.len()
        && found.iter().zip(HEADS).all(|(word, head)| word.strip_prefix(head).is_some_and(|value| !value.is_empty()));
    shaped.then_some(text)
}

/// 捕らえた runner の stderr の最後の [`LINE_HEAD`] の行の 4 語（無い・形の違う周は `None`）。
pub fn from_stderr(stderr: &str) -> Option<&str> {
    stderr.lines().rev().find_map(|found| found.strip_prefix(LINE_HEAD)).and_then(checked)
}

/// lens の判定 object の [`KEY`] の 4 語（無い・字でない・形の違う周は `None`）。
pub fn of_pairs(pairs: &[(String, Value)]) -> Option<String> {
    let found = pairs.iter().find(|(key, _)| key == KEY).and_then(|(_, value)| value.as_str())?;
    checked(found).map(str::to_owned)
}

/// 判定 object へ [`KEY`] の対を足す。消費の 6 値（key `usage`）を運ぶ object だけで、key を既に持つ周と `}` で
/// 閉じない周は 1 字も変えない（消費の event を書く周だけ版を運ぶ・判定の意味は動かさない）。
pub fn with_object(verdict: &str, words: &str) -> String {
    if !has_top_level_key(verdict, "usage") || has_top_level_key(verdict, KEY) {
        return verdict.to_owned();
    }
    let Some(head) = verdict.strip_suffix('}').map(str::trim_end) else {
        return verdict.to_owned();
    };
    let added = json_lite::write_object(&[(KEY, Value::Str(words.to_owned()))]);
    let added = added.strip_prefix('{').and_then(|rest| rest.strip_suffix('}')).unwrap_or_default();
    format!("{head},{added}}}")
}

/// 消費の event の `detail`（囲いの書きの語・半角の空白 1 つ・4 語か [`UNMEASURED_WORDS`]）。
pub fn detail(written: &str, words: Option<&str>) -> String {
    format!("{written} {}", words.unwrap_or(UNMEASURED_WORDS))
}

#[cfg(test)]
mod tests {
    use super::{checked, detail, from_stderr, line, of_pairs, version_of, version_word, with_object, words, UNMEASURED_WORDS};
    use crate::fleet::json_lite;
    use crate::name::BUILD_COMMIT;
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;

    /// 偽 claude を temp の dir に置く（引数を `args` へ 1 行で足し、`body` を撃つ）。host の claude は撃たない。
    fn fake(tag: &str, body: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("xpprov-{}-{tag}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("claude");
        let script = format!("#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{}'\n{body}\n", dir.join("args").display());
        let _ = std::fs::write(&path, script);
        let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755));
        path
    }

    /// 版の語は最初の行の最初の語で、数字で始まり数字と点だけの周だけ（ほかは測れない）。
    #[test]
    fn xpprov_version_word_reads_the_first_word_of_the_first_line() {
        assert_eq!(version_word("2.1.289 (Claude Code)\nnext 9.9\n"), Some("2.1.289"));
        assert_eq!(version_word("\n2.1.289 (Claude Code)\n"), None, "最初の行が空");
        assert_eq!(version_word("2.1.289-beta (Claude Code)\n"), None, "数字と点だけでない");
        assert_eq!(version_word(".2.1 (Claude Code)\n"), None, "数字で始まらない");
        assert_eq!(version_word(""), None, "空");
    }

    /// 撃つのは引数 --version の 1 回で、rc 0 の周だけ版を読み、rc ≠ 0 と撃てない周は測れない。
    #[test]
    fn xpprov_version_of_runs_the_flag_once_and_needs_rc_zero() {
        let ok = fake("ok", "echo '9.8.7 (Claude Code)'");
        assert_eq!(version_of(&ok.display().to_string()).as_deref(), Some("9.8.7"));
        let args = std::fs::read_to_string(ok.with_file_name("args")).unwrap_or_default();
        assert_eq!(args, "--version\n", "引数は --version だけで 1 回");
        let failed = fake("rc1", "echo '9.8.7 (Claude Code)'\nexit 1");
        assert_eq!(version_of(&failed.display().to_string()), None, "rc 1");
        let missing = ok.with_file_name("absent");
        assert_eq!(version_of(&missing.display().to_string()), None, "撃てない");
        for path in [ok, failed] {
            let _ = path.parent().map(std::fs::remove_dir_all);
        }
    }

    /// 4 語は build（器の BUILD_COMMIT の字）・claude（版か字 unmeasured）・model・effort の順で、runner の行はその頭に字を足す。
    #[test]
    fn xpprov_words_name_build_claude_model_effort() {
        let found = words(Some("9.8.7"), "opus", "high");
        assert_eq!(found, format!("build:{BUILD_COMMIT} claude:9.8.7 model:opus effort:high"));
        assert_eq!(words(None, "opus", "high"), format!("build:{BUILD_COMMIT} claude:unmeasured model:opus effort:high"));
        assert_eq!(line(&found), format!("runner: provenance {found}"));
    }

    /// 形は 4 語・頭の順・空でない値で、runner の stderr からは最後の行の 4 語を読み、無い・形の違う周は読まない。
    #[test]
    fn xpprov_stderr_line_is_read_only_in_shape() {
        let good = "build:abc claude:9.8.7 model:opus effort:high";
        assert_eq!(checked(good), Some(good));
        assert_eq!(checked("build:abc claude:9.8.7 model:opus"), None, "3 語");
        assert_eq!(checked("claude:9.8.7 build:abc model:opus effort:high"), None, "頭の順");
        assert_eq!(checked("build:abc claude: model:opus effort:high"), None, "空の値");
        let stderr = format!("runner: scope=gone\nrunner: provenance build:old claude:1 model:a effort:b\n{}\n", line(good));
        assert_eq!(from_stderr(&stderr), Some(good), "最後の行");
        assert_eq!(from_stderr("runner: scope=gone\n"), None, "行が無い");
        assert_eq!(from_stderr("runner: provenance build:abc\n"), None, "形の違う行");
    }

    /// 判定 object へ足すのは消費の 6 値を運ぶ object だけで、key を既に持つ・閉じない object は変えず、読み返すと同じ 4 語。
    #[test]
    fn xpprov_lens_object_carries_words_only_with_usage() {
        let good = "build:abc claude:9.8.7 model:opus effort:high";
        let costed = r#"{"verdict":"PASS","usage":"in:1,out:2,cache_read:3,cache_create:4","turns":5,"wall_ms":6}"#;
        let added = with_object(costed, good);
        assert_eq!(added, format!("{},\"provenance\":\"{good}\"}}", costed.strip_suffix('}').unwrap_or_default()));
        let pairs = json_lite::parse_object(&added).unwrap_or_default();
        assert_eq!(of_pairs(&pairs).as_deref(), Some(good));
        let plain = r#"{"verdict":"PASS"}"#;
        assert_eq!(with_object(plain, good), plain, "usage が無い");
        assert_eq!(with_object(&added, "build:x claude:1 model:a effort:b"), added, "key を既に持つ");
        assert_eq!(of_pairs(&json_lite::parse_object(plain).unwrap_or_default()), None, "対が無い");
    }

    /// detail は囲いの書きの語の後に半角の空白 1 つと 4 語、4 語が無い周は 4 語とも unmeasured。
    #[test]
    fn xpprov_detail_follows_the_write_word() {
        let good = "build:abc claude:9.8.7 model:opus effort:high";
        assert_eq!(detail("write:4096", Some(good)), format!("write:4096 {good}"));
        assert_eq!(detail("write:unmeasured", None), format!("write:unmeasured {UNMEASURED_WORDS}"));
        assert_eq!(UNMEASURED_WORDS, "build:unmeasured claude:unmeasured model:unmeasured effort:unmeasured");
    }
}
