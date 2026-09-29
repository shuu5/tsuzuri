//! 表示先の設定の手引きの skill の歯（行 i-9・接頭辞 stskill_・要件 FR16・判断の記録 ADR-15 の決定 (2)）。
//! plugin/skills/stage-setup/SKILL.md を読むだけで、端末も器も撃たず網に出ない。
//! 囲みの命令は境界の parse_target で読み、層 A の欄の名は境界の KEYS と突き合わせる。

use std::fs;
use std::path::Path;

use tsuzuri_boundary::stage::cli::{Setting, parse_target};
use tsuzuri_boundary::stage::terminal::KEYS;

/// skill の名（skill の dir の名と同じ）。
const NAME: &str = "stage-setup";

/// skill の file（workspace の根からの path）。
const SKILL: &str = "plugin/skills/stage-setup/SKILL.md";

/// 席専用の Chrome の設定の置き場（PC のホームの下・ADR-15 の決定 (4)・持ち主の裁定 t3-hub.59.2）。
const PROFILE: &str = "~/.cache/tsuzuri-stage";

/// SKILL.md の字（CARGO_MANIFEST_DIR の 2 つ上の dir から読む）。
fn skill() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join(SKILL);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// 字を頭の欄の鍵と値の組と本文に分ける（最初の行が --- で次の --- の行まで・どの行もコロンと空白で分かれる）。
fn head(text: &str) -> Result<(Vec<(String, String)>, String), String> {
    let mut lines = text.lines();
    if lines.next() != Some("---") {
        return Err("最初の行が --- でない".to_string());
    }
    let mut pairs = Vec::new();
    for line in lines.by_ref() {
        if line == "---" {
            let body: Vec<&str> = lines.collect();
            return Ok((pairs, body.join("\n")));
        }
        let (key, value) = line
            .split_once(": ")
            .ok_or_else(|| format!("頭の欄の行 {line:?} がコロンと空白で分かれない"))?;
        pairs.push((key.to_string(), value.trim().to_string()));
    }
    Err("頭の欄が --- の行で閉じない".to_string())
}

/// 本文の囲み（3 つの逆引用符で始まる行の間）の中の空でない行（閉じない囲みは Err）。
fn fenced(body: &str) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    let mut open = false;
    for line in body.lines() {
        if line.trim_start().starts_with("```") {
            open = !open;
            continue;
        }
        if open && !line.trim().is_empty() {
            out.push(line.trim().to_string());
        }
    }
    if open {
        return Err("囲みが閉じない".to_string());
    }
    Ok(out)
}

#[test]
fn stskill_head() {
    let (pairs, body) = head(&skill()).expect("頭の欄");
    let keys: Vec<&str> = pairs.iter().map(|(k, _)| k.as_str()).collect();
    assert_eq!(
        keys,
        ["name", "description", "disable-model-invocation"],
        "頭の欄の鍵は name と description と disable-model-invocation の 3 つだけ"
    );
    let value = |key: &str| {
        pairs
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
            .unwrap_or_else(|| panic!("鍵 {key} が無い"))
    };
    assert_eq!(value("disable-model-invocation"), "true");
    assert_eq!(value("name"), NAME);
    let dir = Path::new(SKILL)
        .parent()
        .and_then(Path::file_name)
        .and_then(|n| n.to_str());
    assert_eq!(dir, Some(NAME), "skill の dir の名");
    assert!(!value("description").is_empty(), "description が空");
    assert!(!body.trim().is_empty(), "本文が空");
}

#[test]
fn stskill_commands() {
    let text = skill();
    let (_, body) = head(&text).expect("頭の欄");
    let lines = fenced(&body).expect("囲み");
    let (mut show, mut all, mut project) = (0, 0, 0);
    for line in &lines {
        let words: Vec<&str> = line.split_whitespace().collect();
        assert!(
            words.len() >= 4 && words[..3] == ["tz", "stage", "target"],
            "囲みの行 {line:?} が tz stage target で始まらない"
        );
        for word in &words {
            assert!(
                !word.starts_with('/') && !word.starts_with('~'),
                "囲みの行 {line:?} の語 {word} が / か ~ で始まる"
            );
            if word.starts_with("--") {
                assert!(
                    matches!(*word, "--all" | "--project"),
                    "囲みの行 {line:?} の旗 {word} は --all と --project のほか"
                );
            }
        }
        let call = parse_target(&words[3..]).unwrap_or_else(|e| panic!("{line:?}: {e}"));
        let named: Vec<&String> = match &call.setting {
            Setting::Show => {
                show += 1;
                Vec::new()
            }
            Setting::All(n) => {
                all += 1;
                vec![n]
            }
            Setting::Project(p, n) => {
                project += 1;
                vec![p, n]
            }
            Setting::Clear(p) => vec![p],
        };
        for name in named {
            assert!(
                name.starts_with('<') && name.ends_with('>'),
                "囲みの行 {line:?} の名 {name} が置き字でない"
            );
        }
    }
    assert!(show >= 1, "囲みに show が無い");
    assert!(all >= 1, "囲みに set --all が無い");
    assert!(project >= 1, "囲みに set --project が無い");
    let stage = text
        .match_indices("tz stage")
        .filter(|(i, s)| {
            text[i + s.len()..]
                .chars()
                .next()
                .is_some_and(char::is_whitespace)
        })
        .count();
    let target = text.matches("tz stage target").count();
    assert_eq!(stage, target, "tz stage target のほかの tz stage の命令の字が在る");
}

#[test]
fn stskill_layer_a() {
    assert!(KEYS.contains(&"profile-dir"), "境界の KEYS に profile-dir が無い");
    let text = skill();
    for word in ["profile-dir", PROFILE, "s2-07l.727"] {
        assert!(text.contains(word), "SKILL.md に {word} が無い");
    }
    assert_eq!(
        text.matches('~').count(),
        text.matches(PROFILE).count(),
        "SKILL.md の字 ~ が {PROFILE} のほかに在る"
    );
    assert!(!text.contains('@'), "SKILL.md に字 @ が在る");
}
