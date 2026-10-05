//! 役割の行の上限の file を tz derive が導く歯（接頭辞 rcapf_・設計ノート surface-wave27b 行 t-seatcap・判断の記録 ADR-38 決定 (4)・条 P-2.3）。
//! 置き場は CARGO_TARGET_TMPDIR の下に歯ごとに作り、根で git init する。正本は手で書いた小さな憲法と規則の表（欄 key が seat-bytes と
//! seat-role-bytes の 2 行）と空の design-note/ で、tz derive --dir design-intent --out ../contracts を撃つ。
#![cfg(test)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// 小さな憲法（規範文 2 本）。
const CONSTITUTION: &str = "meta:
  version: v9.9
precedence:
  text: 段の順で解く。
articles:
  - id: N-1
    tier: never
    title: 題
    statements:
      - {id: N-1.1, strength: must-not, text: 消さない。}
  - id: P-1
    tier: always
    title: 題に
    statements:
      - {id: P-1.1, strength: must, text: 守る。}
";

/// 規則の表（欄 key の 2 行・値は 3 桁ごとの , を持つ）。
const RULES: &str = "thresholds:
  - {id: R-1, value: \"8,000 byte 以下\", key: seat-bytes}
  - {id: R-41, value: \"2,000 byte 以下\", key: seat-role-bytes}
discipline: []
";

/// 上限の file の字（頭の 1 行と、seat-role-bytes の行の値の 10 進の数の 1 行）。
const ROLE_MAX: &str = "生成物・手で直さない・design-intent/rules.yaml seat-role-bytes\n2000\n";

/// 歯ごとの置き場（根で git init・design-intent/ に正本・contracts/ が導出物の置き場）。
struct Place {
    root: PathBuf,
}

impl Place {
    fn new(name: &str) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("rcapf")
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
        fs::write(root.join("design-intent/constitution.yaml"), CONSTITUTION).expect("憲法");
        fs::write(root.join("design-intent/rules.yaml"), RULES).expect("規則の表");
        Place { root }
    }

    /// 規則の表の字の 1 か所を替える（元の字はちょうど 1 度在る）。
    fn swap(&self, from: &str, to: &str) {
        let path = self.root.join("design-intent/rules.yaml");
        let text = fs::read_to_string(&path).expect("規則の表を読む");
        assert_eq!(text.matches(from).count(), 1, "{from}");
        fs::write(&path, text.replacen(from, to, 1)).expect("規則の表を書く");
    }

    fn derive(&self, flag: &str) -> (i32, String) {
        let out = Command::new(env!("CARGO_BIN_EXE_tz"))
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

    fn file(&self) -> PathBuf {
        self.root.join("contracts/seat/role-max-bytes.txt")
    }

    fn role_max(&self) -> Option<String> {
        fs::read_to_string(self.file()).ok()
    }
}

#[test]
fn rcapf_role_max_file_shape() {
    let p = Place::new("shape");
    let (rc, out) = p.derive("--write");
    assert_eq!(rc, 0, "{out}");
    assert!(out.contains("書いた 3 file・変わらない 0 file"), "{out}");
    assert_eq!(
        p.role_max().as_deref(),
        Some(ROLE_MAX),
        "頭の 1 行と , を除いた 10 進の数の 1 行"
    );
    let (rc, out) = p.derive("--check");
    assert_eq!(rc, 0, "{out}");
    assert!(out.contains("一致 3・差分 0"), "{out}");
}

#[test]
fn rcapf_value_follows_the_role_row() {
    let p = Place::new("value");
    assert_eq!(p.derive("--write").0, 0);
    p.swap("2,000 byte 以下", "1,500 byte 以下");
    let (rc, out) = p.derive("--check");
    assert_eq!(rc, 1, "{out}");
    assert!(
        out.contains("DRIFT: seat/role-max-bytes.txt（導出と byte で違う）"),
        "{out}"
    );
    assert!(
        !out.contains("DRIFT: seat/brief.txt") && !out.contains("DRIFT: seat/constitution.txt"),
        "上限の file だけ: {out}"
    );
    assert_eq!(p.derive("--write").0, 0);
    assert_eq!(
        p.role_max().as_deref(),
        Some(ROLE_MAX.replace("\n2000\n", "\n1500\n").as_str()),
        "2 行目は新しい値"
    );
}

#[test]
fn rcapf_hand_edit_drifts() {
    let p = Place::new("hand");
    assert_eq!(p.derive("--write").0, 0);
    fs::write(p.file(), ROLE_MAX.replace("2000", "2001")).expect("1 字を替える");
    let (rc, out) = p.derive("--check");
    assert_eq!(rc, 1, "{out}");
    assert!(
        out.contains("DRIFT: seat/role-max-bytes.txt（導出と byte で違う）"),
        "{out}"
    );
    fs::remove_file(p.file()).expect("上限の file を消す");
    let (rc, out) = p.derive("--check");
    assert_eq!(rc, 1, "{out}");
    assert!(
        out.contains("DRIFT: seat/role-max-bytes.txt（置き場に無い）"),
        "{out}"
    );
}

#[test]
fn rcapf_no_role_max_without_the_copies() {
    for (name, from, to, rc_want) in [
        ("over", "8,000 byte", "2,001 byte", 1),
        ("form", "2,000 byte 以下", "2,000 byte 未満", 2),
        (
            "half",
            "  - {id: R-1, value: \"8,000 byte 以下\", key: seat-bytes}\n",
            "",
            2,
        ),
    ] {
        let p = Place::new(&format!("none-{name}"));
        p.swap(from, to);
        let (rc, out) = p.derive("--write");
        assert_eq!(rc, rc_want, "{name}: {out}");
        assert_eq!(p.role_max(), None, "{name}: 上限の file も書かない");
        assert!(!p.root.join("contracts").exists(), "{name}: 何も書かない");
    }
}
