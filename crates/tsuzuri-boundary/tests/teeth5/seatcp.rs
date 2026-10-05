//! 席の手元の 2 つの写しを tz derive が導く歯（接頭辞 seatcp_・設計ノート surface-wave27b 行 t-seatcopy・判断の記録 ADR-38 決定 (3)(4)）。
//! 置き場は CARGO_TARGET_TMPDIR の下に歯ごとに作り、根で git init する（版管理の根が置き場の親になる）。正本は手で書いた小さな憲法と
//! 規則の表（欄 key が seat-bytes と seat-role-bytes の 2 行）と空の design-note/ で、tz derive --dir design-intent --out ../contracts を撃つ。
#![cfg(test)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// 小さな憲法（段の順は file の中で always・never・ask-first・always・空白の続きを持つ字）。
const CONSTITUTION: &str = "meta:
  version: v9.9
precedence:
  text: 段の順  で解く。
articles:
  - id: P-1
    tier: always
    title: 題  いち
    statements:
      - {id: P-1.1, strength: must, text: いつもの  文。}
  - id: N-1
    tier: never
    title: 題に
    statements:
      - {id: N-1.1, strength: must-not, text: 消さない。}
      - {id: N-1.2, strength: must, text: 移す  だけ。}
  - id: A-1
    tier: ask-first
    title: 題さん
    statements:
      - {id: A-1.1, strength: should, text: 問う。}
  - id: P-2
    tier: always
    title: 題よん
";

/// 規則の表（欄 key の 2 行・値は 3 桁ごとの , を持つ）。
const RULES: &str = "thresholds:
  - {id: R-1, value: \"8,000 byte 以下\", key: seat-bytes}
  - {id: R-41, value: \"2,000 byte 以下\", key: seat-role-bytes}
discipline: []
";

/// 要の写しの字（頭の行・順位・段の名と規範文・段「いつも守る」の条の id と題・在りか）。
const BRIEF: &str = "生成物・手で直さない・design-intent/constitution.yaml v9.9
順位 段の順 で解く。
絶対にやらない
N-1.1 消さない。
N-1.2 移す だけ。
確認してから
A-1.1 問う。
いつも守る
P-1 題 いち
P-2 題よん
全文 contracts/seat/constitution.txt
";

/// 全文の写しの字（頭の行・順位・全部の規範文を正本の順に）。
const FULL: &str = "生成物・手で直さない・design-intent/constitution.yaml v9.9
順位 段の順 で解く。
P-1.1 いつもの 文。
N-1.1 消さない。
N-1.2 移す だけ。
A-1.1 問う。
";

/// 歯ごとの置き場（根で git init・design-intent/ に正本・contracts/ が導出物の置き場）。
struct Place {
    root: PathBuf,
}

impl Place {
    fn new(name: &str) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("seatcp")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("design-intent/design-note")).expect("正本の置き場");
        let ok = Command::new("git")
            .arg("-C")
            .arg(&root)
            .args(["init", "-q"])
            .status()
            .expect("git");
        assert!(ok.success(), "git init");
        let p = Place { root };
        p.put("constitution.yaml", CONSTITUTION);
        p.put("rules.yaml", RULES);
        p
    }

    fn put(&self, name: &str, text: &str) {
        fs::write(self.root.join("design-intent").join(name), text).expect("正本を書く");
    }

    /// 正本の字の 1 か所を替える（元の字はちょうど 1 度在る）。
    fn swap(&self, name: &str, from: &str, to: &str) {
        let path = self.root.join("design-intent").join(name);
        let text = fs::read_to_string(&path).expect("正本を読む");
        assert_eq!(text.matches(from).count(), 1, "{from}");
        fs::write(&path, text.replacen(from, to, 1)).expect("正本を書く");
    }

    fn derive(&self, flag: &str) -> (i32, String) {
        let out: Output = Command::new(env!("CARGO_BIN_EXE_tz"))
            .args([
                "derive",
                flag,
                "--dir",
                "design-intent",
                "--out",
                "../contracts",
            ])
            .current_dir(&self.root)
            .output()
            .expect("tz を撃つ");
        let text = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        (out.status.code().expect("終了 code"), text)
    }

    fn seat(&self, name: &str) -> Option<String> {
        fs::read_to_string(self.root.join("contracts/seat").join(name)).ok()
    }

    fn contracts(&self) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(self.root.join("contracts"))
            .map(|d| {
                d.map(|e| e.expect("entry").file_name().to_string_lossy().into_owned())
                    .collect()
            })
            .unwrap_or_default();
        names.sort();
        names
    }
}

#[test]
fn seatcp_two_files_shape() {
    let p = Place::new("shape");
    let (rc, out) = p.derive("--write");
    assert_eq!(rc, 0, "{out}");
    assert!(out.contains("書いた 3 file・変わらない 0 file"), "{out}");
    assert_eq!(p.seat("brief.txt").as_deref(), Some(BRIEF));
    assert_eq!(p.seat("constitution.txt").as_deref(), Some(FULL));
    assert_eq!(p.contracts(), ["seat"], "置き場の直下には dir seat だけ");
    for text in [BRIEF, FULL] {
        assert!(!text.contains(": "), "区切りに「: 」を使わない");
    }
    let (rc, out) = p.derive("--check");
    assert_eq!(rc, 0, "{out}");
    assert!(out.contains("一致 3・差分 0"), "{out}");
}

#[test]
fn seatcp_refuses_three() {
    for (name, from, to, why) in [
        (
            "strength",
            "strength: must, text: いつもの",
            "strength: may, text: いつもの",
            "P-1.1: strength が規範の値でない",
        ),
        (
            "period",
            "text: 移す  だけ。",
            "text: 移す  だけ",
            "N-1.2: 規範文が「。」で終わらない",
        ),
        (
            "none",
            "articles:\n  - id: P-1\n",
            "articles: []\nrest:\n  - id: P-1\n",
            "規範文が 0 本",
        ),
    ] {
        let p = Place::new(name);
        p.swap("constitution.yaml", from, to);
        for flag in ["--write", "--check"] {
            let (rc, out) = p.derive(flag);
            assert_eq!(rc, 2, "{name} {flag}: {out}");
            assert!(
                out.contains("まだ分からない") && out.contains(why),
                "{name} {flag}: {out}"
            );
        }
        assert!(p.contracts().is_empty(), "{name}: 何も書かない");
    }
}

#[test]
fn seatcp_needs_both_keys() {
    let p = Place::new("nokeys");
    p.put("rules.yaml", "thresholds: []\ndiscipline: []\n");
    let (rc, out) = p.derive("--write");
    assert_eq!(rc, 0, "{out}");
    assert!(out.contains("書いた 0 file・変わらない 0 file"), "{out}");
    assert!(p.contracts().is_empty(), "写しを導かない");
    for (name, gone, why) in [
        (
            "role",
            "  - {id: R-41, value: \"2,000 byte 以下\", key: seat-role-bytes}\n",
            "seat-role-bytes の閾値の行が無い",
        ),
        (
            "total",
            "  - {id: R-1, value: \"8,000 byte 以下\", key: seat-bytes}\n",
            "seat-bytes の閾値の行が無い",
        ),
    ] {
        let p = Place::new(name);
        p.swap("rules.yaml", gone, "");
        let (rc, out) = p.derive("--write");
        assert_eq!(rc, 2, "{name}: {out}");
        assert!(out.contains(why), "{name}: {out}");
        assert!(p.contracts().is_empty(), "{name}: 何も書かない");
    }
}

#[test]
fn seatcp_byte_value_form() {
    let form = "が「<正の整数> byte 以下」の形でない";
    for (name, from, to, refusal) in [
        ("plain", "8,000 byte", "8000 byte", None),
        ("group", "8,000 byte", "8,0000 byte", Some(form)),
        ("zero", "8,000 byte", "08000 byte", Some(form)),
        ("unit", "8,000 byte 以下", "8,000 byte 未満", Some(form)),
        ("wide", "8,000 byte", "８,000 byte", Some(form)),
        (
            "equal",
            "2,000 byte 以下",
            "8,000 byte 以下",
            Some("上限が 0 以下"),
        ),
    ] {
        let p = Place::new(&format!("form-{name}"));
        p.swap("rules.yaml", from, to);
        let (rc, out) = p.derive("--write");
        if let Some(why) = refusal {
            assert_eq!(rc, 2, "{name}: {out}");
            assert!(out.contains(why), "{name}: {out}");
            assert!(p.contracts().is_empty(), "{name}: 何も書かない");
        } else {
            assert_eq!(rc, 0, "{name}: {out}");
            assert_eq!(p.seat("brief.txt").as_deref(), Some(BRIEF), "{name}");
        }
    }
}

#[test]
fn seatcp_check_drift() {
    let p = Place::new("drift");
    assert_eq!(p.derive("--write").0, 0);
    let brief = p.root.join("contracts/seat/brief.txt");
    fs::write(&brief, BRIEF.replacen("問う。", "問え。", 1)).expect("1 字を替える");
    let (rc, out) = p.derive("--check");
    assert_eq!(rc, 1, "{out}");
    assert!(
        out.contains("DRIFT: seat/brief.txt（導出と byte で違う）"),
        "{out}"
    );
    assert!(!out.contains("DRIFT: seat/constitution.txt"), "{out}");
    assert_eq!(p.derive("--write").0, 0);
    fs::remove_file(p.root.join("contracts/seat/constitution.txt")).expect("全文の写しを消す");
    let (rc, out) = p.derive("--check");
    assert_eq!(rc, 1, "{out}");
    assert!(
        out.contains("DRIFT: seat/constitution.txt（置き場に無い）"),
        "{out}"
    );
    assert_eq!(p.derive("--write").0, 0);
    p.swap("constitution.yaml", "text: 問う。", "text: 問うてから。");
    let (rc, out) = p.derive("--check");
    assert_eq!(rc, 1, "{out}");
    assert!(
        out.contains("DRIFT: seat/brief.txt（導出と byte で違う）"),
        "{out}"
    );
    assert!(
        out.contains("DRIFT: seat/constitution.txt（導出と byte で違う）"),
        "{out}"
    );
}

#[test]
fn seatcp_cap_edge() {
    let n = BRIEF.len();
    for (name, total, rc_want) in [("at", n + 1, 0), ("over", n, 1)] {
        let p = Place::new(&format!("cap-{name}"));
        p.swap("rules.yaml", "8,000 byte", &format!("{total} byte"));
        p.swap("rules.yaml", "2,000 byte", "1 byte");
        for flag in ["--write", "--check"] {
            let (rc, out) = p.derive(flag);
            assert_eq!(rc, rc_want, "{name} {flag}: {out}");
            let want = format!("要の写し seat/brief.txt が {n} byte で上限 {} byte", n - 1);
            assert_eq!(out.contains(&want), rc_want == 1, "{name} {flag}: {out}");
        }
        let brief = p.seat("brief.txt");
        assert_eq!(
            brief.as_deref(),
            (rc_want == 0).then_some(BRIEF),
            "{name}: 越えれば何も書かない"
        );
    }
}
