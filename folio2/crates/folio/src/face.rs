//! `folio face`（便 14・docs/design/delivery-14.md §1 (a)(c)）。見本 3 面の 1 面を正本から導出して書く（--write）・
//! 検査する（--check）。本便で生成器を持つのは憲法の面（`face_constitution.rs`）だけで、入口と要件書は「まだ分からない」。
//! この file は命令の口（面の名の解決・正本の読み・3 値と文言）と、生成器が共有する口（木を辿る型 X・escape・
//! 名札の表・値の読める形・小窓）を持つ。導出できない入力は 2「まだ分からない」に倒し、出力先に 1 byte も書かない（P-4.1）。

use std::fs;
use std::path::Path;

use crate::face_constitution;
use crate::parts::FACES;
use crate::verdict::Verdict;
use crate::yaml::{self, Value};

pub type R<T> = Result<T, String>;

pub enum Mode {
    Write,
    Check,
}

/// 1 回の実行の結果。`stdout` / `stderr` は 1 行ずつ。
pub struct Outcome {
    pub verdict: Verdict,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
}

impl Outcome {
    fn unknown(reason: impl Into<String>) -> Self {
        Outcome {
            verdict: Verdict::Unknown,
            stdout: None,
            stderr: Some(format!("folio face: まだ分からない: {}", reason.into())),
        }
    }
}

// ── 命令の口 ──

pub fn run(face: &str, dir: &Path, out: &Path, mode: Mode) -> Outcome {
    let derive: fn(&Path) -> R<String> = match face {
        "constitution" => face_constitution::derive,
        f if FACES.contains(&f) => {
            return Outcome::unknown(format!("面「{f}」の生成器はまだ無い"));
        }
        f => {
            return Outcome::unknown(format!(
                "面の名「{f}」は index・constitution・srs のどれでもない"
            ));
        }
    };
    // --out が相対なら --dir からの相対・絶対ならそのまま
    let out_path = dir.join(out);
    if !out_path.parent().is_some_and(Path::is_dir) {
        return Outcome::unknown(format!("{}: 出力先の親 dir が無い", out_path.display()));
    }
    let html = match derive(dir) {
        Ok(h) => h,
        Err(e) => return Outcome::unknown(e),
    };
    let size = html.len();
    match mode {
        Mode::Check => {
            if !out_path.exists() {
                return Outcome {
                    verdict: Verdict::Unknown,
                    stdout: None,
                    stderr: Some("folio face: 面が無い（未生成）".to_string()),
                };
            }
            let cur = match fs::read(&out_path) {
                Ok(b) => b,
                Err(e) => {
                    return Outcome::unknown(format!("{}: 読めない: {e}", out_path.display()));
                }
            };
            if cur != html.as_bytes() {
                return Outcome {
                    verdict: Verdict::Fail,
                    stdout: None,
                    stderr: Some(format!(
                        "folio face: DRIFT — 面 {} byte ≠ 導出 {size} byte（手で直したか正本が変わった）",
                        cur.len()
                    )),
                };
            }
            Outcome {
                verdict: Verdict::Pass,
                stdout: Some(format!("folio face: OK — 面は正本と一致（{size} byte）")),
                stderr: None,
            }
        }
        Mode::Write => {
            if let Err(e) = fs::write(&out_path, &html) {
                return Outcome::unknown(format!("{}: 書けない: {e}", out_path.display()));
            }
            Outcome {
                verdict: Verdict::Pass,
                stdout: Some(format!("folio face: 書いた（{size} byte）")),
                stderr: None,
            }
        }
    }
}

/// 正本 1 file を型付きで読む。無い・読めない・UTF-8 でない・重複キー・空の文書は Err（まだ分からない）。
pub fn load(dir: &Path, name: &str) -> R<Value> {
    let bytes = fs::read(dir.join(name)).map_err(|e| format!("{name}: 読めない: {e}"))?;
    let text = String::from_utf8(bytes).map_err(|_| format!("{name}: UTF-8 でない"))?;
    let doc = yaml::parse(&text).map_err(|e| format!("{name}: 読めない: {e}"))?;
    if let Some(d) = doc.duplicates.first() {
        return Err(format!("{name}: 重複キー「{}」（{} 行）", d.key, d.line));
    }
    yaml::parse_typed(&text).map_err(|e| format!("{name}: {e}"))
}

// ── 木を辿る口（便 11 の render.rs の X と同じ形）──

/// 型付きの木の 1 点（欄の道つき）。無い欄と型違いは文言つきの Err。
pub struct X<'a> {
    pub v: &'a Value,
    pub at: String,
}

impl<'a> X<'a> {
    pub fn root(v: &'a Value, at: &str) -> Self {
        X {
            v,
            at: at.to_string(),
        }
    }

    fn child(&self, v: &'a Value, seg: &str) -> Self {
        X {
            v,
            at: format!("{}{seg}", self.at),
        }
    }

    fn entries(&self) -> R<&'a [(Value, Value)]> {
        self.v
            .as_map()
            .ok_or_else(|| format!("{}: 表でない", self.at))
    }

    /// 必須の欄（無ければ Err）。
    pub fn f(&self, key: &str) -> R<X<'a>> {
        self.entries()?;
        self.v
            .get(key)
            .map(|v| self.child(v, &format!(".{key}")))
            .ok_or_else(|| format!("{}: 欄 {key} が無い", self.at))
    }

    /// 任意の欄（無い・null は None）。
    pub fn g(&self, key: &str) -> R<Option<X<'a>>> {
        self.entries()?;
        Ok(self
            .v
            .get(key)
            .filter(|v| !matches!(v, Value::Null))
            .map(|v| self.child(v, &format!(".{key}"))))
    }

    /// 一覧の要素。
    pub fn seq(&self) -> R<Vec<X<'a>>> {
        let items = self
            .v
            .as_seq()
            .ok_or_else(|| format!("{}: 一覧でない", self.at))?;
        Ok(items
            .iter()
            .enumerate()
            .map(|(i, v)| self.child(v, &format!("[{i}]")))
            .collect())
    }

    /// 表の（キー, 値）を書かれた順に（キーは文字列だけ）。
    pub fn pairs(&self) -> R<Vec<(&'a str, X<'a>)>> {
        self.entries()?
            .iter()
            .map(|(k, v)| {
                let k = k
                    .as_str()
                    .ok_or_else(|| format!("{}: 表のキーが文字列でない", self.at))?;
                Ok((k, self.child(v, &format!(".{k}"))))
            })
            .collect()
    }

    /// scalar の字面（文字列・整数・小数・日付）。null・真偽・一覧・表は Err。
    pub fn text(&self) -> R<String> {
        match self.v {
            Value::Str(_) | Value::Int(_) | Value::Float(_) | Value::Date(_) => Ok(self.v.py_str()),
            _ => Err(format!("{}: 文字列でない", self.at)),
        }
    }

    /// escape した scalar の字面。
    pub fn e(&self) -> R<String> {
        Ok(esc(&self.text()?))
    }

    /// 必須の欄の escape した字面。
    pub fn ef(&self, key: &str) -> R<String> {
        self.f(key)?.e()
    }

    /// id と href に使う id（文字列で、英数字と「-」「.」だけ）。
    pub fn id(&self) -> R<&'a str> {
        let s = self
            .v
            .as_str()
            .ok_or_else(|| format!("{}: 文字列でない", self.at))?;
        safe_id(s).map_err(|e| format!("{}: {e}", self.at))
    }

    /// 整数（0 以上）。
    pub fn count(&self) -> R<u64> {
        match self.v {
            Value::Int(n) => n
                .parse()
                .map_err(|_| format!("{}: 0 以上の整数でない", self.at)),
            _ => Err(format!("{}: 整数でない", self.at)),
        }
    }

    /// 名札の表引き（表に無い値は Err）。
    pub fn lookup<T: Copy>(&self, table: &[(&str, T)], what: &str) -> R<T> {
        if let Value::Str(s) = self.v
            && let Some((_, t)) = table.iter().find(|(k, _)| k == s)
        {
            return Ok(*t);
        }
        Err(format!(
            "{}: {what} の表に無い値「{}」",
            self.at,
            self.v.py_str()
        ))
    }

    /// 機械のための面の字面（正本の値のまま・scalar だけ）。
    pub fn raw(&self) -> R<String> {
        match self.v {
            Value::Null => Ok("null".to_string()),
            Value::Bool(b) => Ok(b.to_string()),
            Value::Seq(_) | Value::Map(_) => Err(format!("{}: scalar でない", self.at)),
            other => Ok(esc(&other.py_str())),
        }
    }
}

// ── 字面の口 ──

/// 便 11 と同じ 5 字の escape。
pub fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#x27;"),
            c => out.push(c),
        }
    }
    out
}

/// id と href に使う id は英数字と「-」「.」だけ（空は受けない）。
pub fn safe_id(s: &str) -> R<&str> {
    if !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '.'))
    {
        Ok(s)
    } else {
        Err(format!("id「{s}」が英数字と「-」「.」以外を含む"))
    }
}

/// 同じ面の anchor（id を ASCII 小文字に）。
pub fn anchor(id: &str) -> String {
    id.to_ascii_lowercase()
}

/// 最初の「 — 」（前後に半角空白 1 つずつの全角ダッシュ）で前と後に割る（無ければ全体と None）。
pub fn split_dash(s: &str) -> (&str, Option<&str>) {
    match s.split_once(" — ") {
        Some((head, tail)) => (head, Some(tail)),
        None => (s, None),
    }
}

/// 関係の sections の値（「§」+ 0〜8 の数字 1 つ）→ 章の anchor（s0〜s8）。
pub fn section_anchor(s: &str) -> R<String> {
    match s.strip_prefix('§').map(str::as_bytes) {
        Some([d @ b'0'..=b'8']) => Ok(format!("s{}", *d as char)),
        _ => Err(format!("節「{s}」は「§」+ 0〜8 の数字 1 つでない")),
    }
}

/// 説明の小窓（押すと開く）。`label` は名札・`body` は組み立て済みの HTML。
pub fn hint(label: &str, body: &str) -> String {
    format!(
        "<span class=\"hint\"><label><input type=\"checkbox\" class=\"vh\" aria-label=\"{label}を開く\"><span class=\"hint-btn\">{label}</span></label><span class=\"hint-body\">{body}</span></span>"
    )
}

/// 「?」の小窓。
pub fn hint_q(body: &str) -> String {
    format!(
        "<span class=\"hint\"><label><input type=\"checkbox\" class=\"vh\" aria-label=\"説明を開く\"><span class=\"hint-btn q\">?</span></label><span class=\"hint-body\">{body}</span></span>"
    )
}

/// 値の読める形（便 11 の render の関数 val と同じ）。
pub fn val(x: &X<'_>, d: usize) -> R<String> {
    match x.v {
        Value::Map(entries) => {
            let mut parts = Vec::with_capacity(entries.len());
            for (k, vv) in entries {
                let vx = x.child(vv, &format!(".{}", k.py_str()));
                let body = if matches!(vv, Value::Map(_)) {
                    format!("<br>{}", val(&vx, d + 1)?)
                } else {
                    val(&vx, d + 1)?
                };
                parts.push(format!(
                    "{}<b>{}</b>: {body}",
                    "　".repeat(d),
                    esc(&k.py_str())
                ));
            }
            Ok(parts.join("<br>"))
        }
        Value::Seq(_) => {
            let parts = x
                .seq()?
                .iter()
                .map(|i| match i.v {
                    Value::Str(s) => Ok(format!("「{}」", esc(s))),
                    _ => val(i, d),
                })
                .collect::<R<Vec<_>>>()?;
            Ok(parts.join("・"))
        }
        Value::Null => Ok("<code>null</code>".to_string()),
        other => Ok(esc(&other.py_str())),
    }
}

/// 根拠の一覧（各項「<種別の名札>: <ref>」を「／」で繋ぐ）。
pub fn rationale(x: &X<'_>) -> R<String> {
    Ok(x.seq()?
        .iter()
        .map(|r| {
            Ok(format!(
                "{}: {}",
                r.f("kind")?.lookup(RATIONALE_KIND, "根拠の種別")?,
                r.ef("ref")?
            ))
        })
        .collect::<R<Vec<_>>>()?
        .join("／"))
}

// ── 名札の表（β・表に無い値は導出できない）──

/// 段の名札。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tier {
    pub name: &'static str,
    pub en: &'static str,
    pub class: &'static str,
    pub color: &'static str,
    pub meaning: &'static str,
    /// 外すのに要るもの（字面）
    pub remove: &'static str,
    /// 外すのに要るもの（§6 への xref を含む HTML）
    pub remove_html: &'static str,
}

pub const TIERS: &[(&str, Tier)] = &[
    (
        "always",
        Tier {
            name: "いつも守る",
            en: "Always",
            class: "tier-always",
            color: "ok",
            meaning: "道具も AI も、毎回これに従う",
            remove: "憲法の改訂（§6: 判断の記録 + 持ち主の承認）",
            remove_html: "憲法の改訂（<a class=\"xref\" href=\"#s6\">§6</a>: 判断の記録 + 持ち主の承認）",
        },
    ),
    (
        "ask-first",
        Tier {
            name: "確認してから変える",
            en: "Ask-first",
            class: "tier-askfirst",
            color: "warn",
            meaning: "やってよいが、実行前に持ち主へ確認する",
            remove: "その場の持ち主の確認",
            remove_html: "その場の持ち主の確認",
        },
    ),
    (
        "never",
        Tier {
            name: "絶対にやらない",
            en: "Never",
            class: "tier-never",
            color: "bad",
            meaning: "確認があってもやらない",
            remove: "憲法の改訂（確認では解けない）",
            remove_html: "<a class=\"xref\" href=\"#s6\">憲法の改訂</a>（確認では解けない）",
        },
    ),
];

/// 段の表引き（表に無い値は Err）。
pub fn tier_of(key: &str) -> R<Tier> {
    TIERS
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, t)| *t)
        .ok_or_else(|| format!("段 の表に無い値「{key}」"))
}

pub const STRENGTH: &[(&str, &str)] = &[
    ("must", "MUST"),
    ("must-not", "MUST NOT"),
    ("should", "SHOULD"),
];
pub const PATTERN: &[(&str, &str)] = &[
    ("ubiquitous", "つねに"),
    ("event", "〜のとき"),
    ("state", "〜のあいだ"),
    ("unwanted", "〜になったら"),
    ("optional", "〜ならば"),
];
pub const BINDS: &[(&str, &str)] = &[("tool", "道具"), ("practice", "作法"), ("both", "両方")];
pub const MECH_KIND: &[(&str, &str)] = &[
    ("reject", "機械が拒む"),
    ("build-check", "生成時の検査"),
    ("human-review", "人が目で確かめる"),
    ("none", "なし"),
];
pub const LIVE: &[(&str, &str)] = &[
    ("now", "いま動く"),
    ("M0", "M0 で動く"),
    ("delivery-0", "便 0 で動く"),
    ("M1", "M1 で動く"),
    ("adr", "判断の記録の欄の決まりの後"),
];
pub const STAGE: &[(&str, &str)] = &[("in-loop", "編集時"), ("post", "事後")];
pub const POLARITY: &[(&str, &str)] = &[("fail-open", "開く"), ("fail-closed", "閉じる")];
pub const RATIONALE_KIND: &[(&str, &str)] = &[
    ("v1-incident", "v1 の実害"),
    ("scribe2-article", "scribe2 の条"),
    ("folio2-ruling", "持ち主の裁定"),
];
pub const RETREAT_KIND: &[(&str, &str)] = &[
    ("spike", "試して測る"),
    ("measure", "測る"),
    ("ruling", "持ち主に問う"),
];
pub const DOC_STATUS: &[(&str, &str)] = &[
    ("effective", "発効・拘束力あり"),
    ("draft", "未承認・拘束力なし"),
];
/// rules 行の状態 → state の chip の class。
pub const RULE_STATUS: &[(&str, &str)] = &[
    ("凍結", "state ok"),
    ("仮", "state warn"),
    ("未定", "state"),
];
pub const RULE_KIND: &[(&str, &str)] = &[
    ("deny", "測って落とす"),
    ("build-check", "生成時の検査"),
    ("detect", "記録のみ"),
    ("human-review", "人が守る作法"),
];

#[cfg(test)]
mod face_tests {
    use super::*;

    #[test]
    fn face_tier_lookup_has_three_values_and_rejects_others() {
        assert_eq!(tier_of("always").unwrap().class, "tier-always");
        assert_eq!(tier_of("ask-first").unwrap().name, "確認してから変える");
        assert_eq!(tier_of("never").unwrap().color, "bad");
        assert!(tier_of("sometimes").is_err());
        assert!(tier_of("Always").is_err());
        let v = Value::Str("sometimes".into());
        assert!(X::root(&v, "t").lookup(TIERS, "段").is_err());
    }

    #[test]
    fn face_split_dash_splits_at_the_first_spaced_dash() {
        assert_eq!(
            split_dash("理由を書く — 記録を起こす — 続き"),
            ("理由を書く", Some("記録を起こす — 続き"))
        );
        assert_eq!(split_dash("承認する"), ("承認する", None));
        assert_eq!(split_dash("前—後"), ("前—後", None));
    }

    #[test]
    fn face_section_anchor_accepts_only_section_sign_and_one_digit() {
        assert_eq!(section_anchor("§6").unwrap(), "s6");
        assert_eq!(section_anchor("§0").unwrap(), "s0");
        assert!(section_anchor("§9").is_err());
        assert!(section_anchor("6").is_err());
        assert!(section_anchor("§66").is_err());
        assert!(section_anchor("§").is_err());
    }

    #[test]
    fn face_escape_and_safe_id() {
        assert_eq!(esc("a&b<c>d\"e'f"), "a&amp;b&lt;c&gt;d&quot;e&#x27;f");
        assert_eq!(safe_id("P-2.1").unwrap(), "P-2.1");
        assert!(safe_id("P 1").is_err());
        assert!(safe_id("\"x").is_err());
        assert!(safe_id("").is_err());
        assert_eq!(anchor("NFR2"), "nfr2");
    }

    #[test]
    fn face_val_is_the_readable_form_of_render() {
        let v = yaml::parse_typed("a: [x, null]\nb: {c: 1}\nd: \"<&>\"\n").unwrap();
        assert_eq!(
            val(&X::root(&v, "v"), 0).unwrap(),
            "<b>a</b>: 「x」・<code>null</code><br><b>b</b>: <br>　<b>c</b>: 1<br><b>d</b>: &lt;&amp;&gt;"
        );
    }
}
