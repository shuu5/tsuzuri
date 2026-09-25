//! `folio face`（便 14・docs/design/delivery-14.md §1 (a)(c)／便 15・delivery-15.md §1 (b)）。見本 3 面の 1 面を正本から
//! 導出して書く（--write）・検査する（--check）。生成器は憲法の面（`face_constitution.rs`）・要件書の面
//! （`face_srs.rs`）・入口の面（`face_index.rs`・便 16・delivery-16.md §1 (b)）の 3 つ。
//! この file は命令の口（面の名の解決・正本の読み・3 値と文言）と、生成器が共有する口（名札・値の読める形・
//! 小窓・面の骨格・図の枠）を持つ。導出できない入力は 2「まだ分からない」に倒し、出力先に 1 byte も書かない（P-4.1）。
//! 正本の cursor（結果の型 R・load・木を辿る型 X・escape・safe_id）は `cursor.rs` へ降ろした（便 107・ADR-15）。
//! 憲法の値域の名札（便 50・ADR-11 決定 (4)②）は、組み立て時に憲法の正本から導出した型（`constitution_enums`）への
//! 網羅の場合分けで持つ = 値域の値の字面を鍵にした表を持たない（値が足されても消えても組み立てが通らない）。
//! 部品目録の上限 3 本と図の型の名札（便 52・ADR-11 決定 (4)③）は、組み立て時に部品目録から導出した定数
//! （`catalog`）を指す = 手書きの写しを持たない。
//! 図の枠（便 34・P-2.1）: 図の節（figures）の 1 枚の枠（figure-panel・fig-title・図の本体・figcaption）は
//! 設計ノート・判断の記録・要件書の 3 面が同じ字面で出すので、`figure_body` と `figure_panel` をここに 1 つ持つ。
//! 天井の名札（便 40・delivery-40.md §1 (c)(d)・ADR-8 決定 (4)・P-3.3）: 5 面の site-bar に床の名札（freshness-stamp）の
//! 直後に部品 ceiling-stamp を置く。字は `ceiling_stamp` の 1 つで組む（面ごとに組み直さない）ので 5 面で同じになる。
//! 出所は天井の印 preview/ceiling-stamp.yaml の 1 つ（便 83・P-6.3）で、印が無ければ 4 観点とも「まだ分からない」（未実施）を出す（P-4.2）。
//! 印の正本の要約値が今の正本と違えば「印の後に変わった所はまだ読まれていない」を添える（便 127・ADR-18 決定 (6)・P-3.3）。

use std::fs;
use std::path::Path;

use crate::catalog::{self, Component};
use crate::ceiling_src;
use crate::constitution_enums as ce;
use crate::cursor::{R, X, esc, load};
use crate::figure;
use crate::findings;
use crate::gate;
use crate::shelf::ANNEXES;
use crate::stamp;
use crate::verdict::Verdict;
use crate::yaml::Value;
use crate::{face_adr, face_constitution, face_index, face_note, face_srs};

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

/// `--out` は相対なら `--dir` からの相対・絶対ならそのまま。
pub fn run(face: &str, id: Option<&str>, dir: &Path, out: &Path, mode: Mode) -> Outcome {
    if !matches!(face, "index" | "constitution" | "srs" | "adr" | "note") {
        return Outcome::unknown(format!(
            "面の名「{face}」は index・constitution・srs・adr・note のどれでもない"
        ));
    }
    // 判断の記録の面と設計ノートの面は 1 本 1 枚なので id が要る。ほかの面は id を取らない
    if !matches!(face, "adr" | "note") && id.is_some() {
        return Outcome::unknown("--id は面 adr と note にだけ付く");
    }
    let doc_id = match id {
        Some(id) => id,
        None if face == "adr" => {
            return Outcome::unknown("--id が無い（面 adr は判断の記録の id が要る）");
        }
        None if face == "note" => {
            return Outcome::unknown("--id が無い（面 note は設計ノートの文書 id が要る）");
        }
        None => "",
    };
    // --out が相対なら --dir からの相対・絶対ならそのまま
    let out_path = dir.join(out);
    if !out_path.parent().is_some_and(Path::is_dir) {
        return Outcome::unknown(format!("{}: 出力先の親 dir が無い", out_path.display()));
    }
    let derived = match face {
        "index" => face_index::derive(dir),
        "constitution" => face_constitution::derive(dir),
        "srs" => face_srs::derive(dir),
        "note" => face_note::derive(dir, doc_id),
        _ => face_adr::derive(dir, doc_id),
    };
    let html = match derived {
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

// ── 字面の口 ──

/// 同じ面の anchor（id を ASCII 小文字に）。
pub fn anchor(id: &str) -> String {
    id.to_ascii_lowercase()
}

/// 判断の記録の面の file 名（便 135・delivery-135.md §1 (b) の 1）。正本 `<dir>/adr/<id>.yaml` が file として在れば
/// `adr-<数>.html`（`folio build` が面を出す番号の規則と同じ）・無ければ None。
pub fn adr_face(dir: &Path, id: &str) -> Option<String> {
    dir.join("adr")
        .join(format!("{id}.yaml"))
        .is_file()
        .then(|| format!("{}.html", anchor(id)))
}

/// 組み立てた HTML の字の部分の番号を行き先へのリンクにする（便 135・delivery-135.md §1 (b) の 1）。タグの外の字だけを
/// 走査し、`<a>`・`<head>`・`<script>`・`<svg>` の中と注釈は触らない。番号は床と同じ口で拾う（判断の記録は
/// `link::adr_end`・`all` が真なら条・rules 行・要件の id も `refs::id_end`）。`href` が行き先を返した番号だけを
/// `<a class="xref" href="<行き先>">` で包み、字は 1 字も変えない（P-6.1）。
pub fn link_ids(html: &str, all: bool, href: impl Fn(&str) -> Option<String>) -> String {
    let chars: Vec<char> = html.chars().collect();
    let at = |i: usize, s: &str| s.chars().enumerate().all(|(k, c)| chars.get(i + k) == Some(&c));
    let find = |from: usize, s: &str| (from..chars.len()).find(|&j| at(j, s));
    let mut out = String::with_capacity(html.len());
    // 触らない要素（a・head・script・svg）の深さ
    let mut shut = 0usize;
    let mut i = 0;
    while i < chars.len() {
        let start = i;
        if at(i, "<!--") {
            i = find(i + 4, "-->").map_or(chars.len(), |j| j + 3);
        } else if chars[i] == '<' {
            // タグの終わり（引用符の中の「>」は数えない）
            let mut quote = None;
            i += 1;
            while i < chars.len() {
                let c = chars[i];
                i += 1;
                match quote {
                    Some(q) if c == q => quote = None,
                    Some(_) => {}
                    None if c == '"' || c == '\'' => quote = Some(c),
                    None if c == '>' => break,
                    None => {}
                }
            }
            let close = chars.get(start + 1) == Some(&'/');
            let name: String = chars[start + 1 + usize::from(close)..i]
                .iter()
                .take_while(|c| c.is_ascii_alphanumeric())
                .collect::<String>()
                .to_ascii_lowercase();
            if matches!(name.as_str(), "a" | "head" | "script" | "svg") && chars[i - 1] == '>' {
                if close {
                    shut = shut.saturating_sub(1);
                } else if chars[i - 2] != '/' {
                    shut += 1;
                    // script の中身は字でもタグでもない（閉じタグの前まで写す）
                    if name == "script" {
                        i = find(i, "</script").unwrap_or(chars.len());
                    }
                }
            }
        } else if shut == 0
            && let Some(end) = crate::link::adr_end(&chars, i)
                .or_else(|| all.then(|| crate::refs::id_end(&chars, i)).flatten())
        {
            let id: String = chars[i..end].iter().collect();
            match href(&id) {
                Some(h) => out.push_str(&format!("<a class=\"xref\" href=\"{h}\">{id}</a>")),
                None => out.push_str(&id),
            }
            i = end;
            continue;
        } else {
            i += 1;
        }
        out.extend(&chars[start..i]);
    }
    out
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

/// 印の正本の要約値が今の正本と違うときの添え書き（判断の記録 ADR-18 決定 (6) の逐語・便 127）。
const UNREAD: &str = "印の後に変わった所はまだ読まれていない";

/// 天井の名札の字（部品 ceiling-stamp の中身・便 40 §1 (c)・便 83 §1 (b)(d)）。出所は天井の印
/// `<dir>/preview/ceiling-stamp.yaml`（`stamp::marks`）と天井の正本の viewpoints の名（`findings::viewpoint_names`）の 2 つだけ。
/// 印が在るとき「天井 <b>名 3 値</b> · …（<印の at>・束 <8 字>/…）」——日付と要約値は印の値のまま（面の側で数え直さない）。
/// 印の sources が今の正本の要約値（門と印と同じ関数 `gate::sources_digest`・P-6.3）と違えば、閉じ括弧の直後に
/// `<b>印の後に変わった所はまだ読まれていない</b>` を添える（便 127・delivery-127.md §1 (b)・3 値のどれにも同じ）。
/// 測れなければ Err（天井の正本が load で読めない・読む文書が無い＝面は導出できない・P-4.1）。引き金と節点の数は読まない。
/// 印が無いとき「天井 <b>名 まだ分からない</b> · …（未実施）」（要約値は測らない）。印の観点の id の列が正本と順まで
/// 同じでなければ Err（印・正本が読めないときも Err＝面は導出できない・P-4.1）。字の後ろに説明の小窓を 1 つ置く（5 面で同じ字）。
pub fn ceiling_stamp(dir: &Path) -> R<String> {
    let names = findings::viewpoint_names(dir)?;
    let (cells, tail, note) = match stamp::marks(dir)? {
        None => (
            names
                .iter()
                .map(|(_, name)| format!("<b>{} {}</b>", esc(name), Verdict::Unknown))
                .collect::<Vec<_>>(),
            "未実施".to_string(),
            String::new(),
        ),
        Some(stamp::Marks { at, sources, rows: marks }) => {
            let ids: Vec<&str> = marks.iter().map(|m| m.id.as_str()).collect();
            let want: Vec<&str> = names.iter().map(|(id, _)| id.as_str()).collect();
            if ids != want {
                return Err(format!(
                    "{}: 印の観点が天井の正本と違う（印 {}・正本 {}）",
                    ceiling_src::STAMP_FILE,
                    ids.join(", "),
                    want.join(", ")
                ));
            }
            let cells = marks
                .iter()
                .zip(&names)
                .map(|(m, (_, name))| format!("<b>{} {}</b>", esc(name), esc(&m.verdict)))
                .collect();
            let now = ceiling_src::load(dir)
                .and_then(|ceiling| gate::sources_digest(dir, &ceiling))
                .map_err(|e| format!("天井の名札: 正本の要約値が測れない: {e}"))?;
            let note = if now == sources {
                String::new()
            } else {
                format!("<b>{UNREAD}</b>")
            };
            let digests: Vec<String> = marks.iter().map(|m| esc(&m.bundle)).collect();
            (cells, format!("{}・束 {}", esc(&at), digests.join("/")), note)
        }
    };
    Ok(format!(
        "天井 {}（{tail}）{note}{}",
        cells.join(" · "),
        hint_q(&ceiling_hint()?)
    ))
}

/// 天井の名札の小窓の本体（便 83 §1 (d)・面によらず同じ字）。用語集への導線は `glossary_chip` と同じ規則で章を引く。
fn ceiling_hint() -> R<String> {
    let c = ANNEXES
        .iter()
        .find(|(id, _)| *id == "vocabulary")
        .map(|(_, (c, _))| *c)
        .ok_or("付録の表に vocabulary が無い")?;
    Ok(format!(
        "<p>天井は AI が意味を読む検査です。観点ごとに 合格・不合格・まだ分からない の 3 値を出します。1 つでも まだ分からない が在れば合格にしません。</p><p>括弧の中は、最後に数えた日付と、観点ごとの材料の束の要約値の先頭 8 字です。</p><p><a href=\"constitution.html#s{c}\">用語集（天井の判定の印）</a></p>"
    ))
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
                rationale_kind_label(
                    r.f("kind")?
                        .parse(ce::RationaleKind::from_name, "根拠の種別")?
                ),
                r.ef("ref")?
            ))
        })
        .collect::<R<Vec<_>>>()?
        .join("／"))
}

/// 章の見出しの数えの字（便 63 で憲法の面に置いた規則を便 70 で 3 面の共有の口へ・天井の 11 周目の読みやすさ F-2 と
/// 12 周目の F-2）。日本語の助数詞「つ」は 9 までにしか付かないので、10 以上は「{n} の{noun}」にする。目次と章の帯の
/// 2 か所が同じ字面になるよう、面ごとにここ 1 か所から出す（P-6.3）。
pub fn count_word(n: usize, noun: &str) -> String {
    if n <= 9 {
        format!("{n} つの{noun}")
    } else {
        format!("{n} の{noun}")
    }
}

// ── 名札と名札の表は face_labels.rs へ移した（便 87・振る舞いは不変）──
pub use crate::face_labels::*;

// ── 部品目録の上限（組み立て時に部品目録から導出する・便 52・ADR-11 決定 (4)③）──

/// pipeline-rail の max_nodes。
pub const MAX_RAIL_NODES: usize = catalog::PIPELINE_RAIL_MAX_NODES;
/// state-strip の max_nodes。
pub const MAX_STATE_NODES: usize = catalog::STATE_STRIP_MAX_NODES;
/// context-band の max_per_band。
pub const MAX_PER_BAND: usize = catalog::CONTEXT_BAND_MAX_PER_BAND;

// ── 面の骨格（面に依らない口・P-6.3）──

/// 読める面の nav（href・名）。
const NAV: [(&str, &str); 3] = [
    ("index.html", "入口"),
    ("constitution.html", "憲法"),
    ("srs.html", "要件書"),
];

/// prevnext の 1 つ（href・名）。
pub fn link(href: &str, name: &str) -> (String, String) {
    (href.to_string(), name.to_string())
}

/// 並んだ面の列（`links`）の `at` 番目の前と次（便 65）。先頭の前と末尾の次は入口。
pub fn neighbors(links: &[(String, String)], at: usize) -> ((String, String), (String, String)) {
    let entrance = || link("index.html", "入口");
    let prev = at
        .checked_sub(1)
        .and_then(|i| links.get(i))
        .cloned()
        .unwrap_or_else(entrance);
    let next = links.get(at + 1).cloned().unwrap_or_else(entrance);
    (prev, next)
}

/// 面ごとの固定値。
pub struct Frame {
    /// 面の名（crumb・here・foot）
    pub name: &'static str,
    /// 正本の file 名（foot）
    pub source: &'static str,
    /// favicon の link 要素
    pub favicon: &'static str,
    /// nav の aria-current の位置（NAV の添字）
    pub current: usize,
    /// 最初の章の番号（憲法 00・要件書 01）
    pub first: usize,
    /// 章ごとの帯の class と kicker の絵記号（first から順に）
    pub bands: &'static [(&'static str, &'static str)],
    /// prevnext の前と次（href・名は escape 済み・判断の記録と設計ノートは隣の 1 本を指す・便 65）
    pub prev: (String, String),
    pub next: (String, String),
    /// 面が使う部品
    pub parts: &'static [Component],
}

impl Frame {
    /// 章の数（帯の章と承認欄）。
    pub fn chapters(&self) -> usize {
        self.bands.len() + 1
    }

    /// 属性 data-component（名札は部品目録の一覧からだけ出す）。
    pub fn dc(&self, c: Component) -> String {
        debug_assert!(
            self.parts.contains(&c),
            "{} は{}の面の部品の一覧に無い",
            c.name(),
            self.name
        );
        format!("data-component=\"{}\"", c.name())
    }

    /// head と site-bar（skip-link から main の開始まで）。値は escape 済み。`ceiling` = 天井の名札の字（`ceiling_stamp` で
    /// 組み立て済み・床の名札 freshness-stamp の直後に部品 ceiling-stamp として置く・便 40）。
    pub fn head(
        &self,
        o: &mut Vec<String>,
        title: &str,
        generated: &str,
        version: &str,
        status: &str,
        ceiling: &str,
    ) {
        o.push("<!DOCTYPE html>".to_string());
        o.push("<html lang=\"ja\" class=\"no-js\">".to_string());
        o.push("<head>".to_string());
        o.push("<meta charset=\"utf-8\">".to_string());
        o.push(
            "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">".to_string(),
        );
        o.push(format!("<title>{title}</title>"));
        o.push(self.favicon.to_string());
        o.push("<link rel=\"stylesheet\" href=\"folio.css\">".to_string());
        o.push("<script src=\"folio-ui.js\"></script>".to_string());
        o.push("</head>".to_string());
        o.push("<body>".to_string());
        o.push("<a class=\"skip-link\" href=\"#main\">本文へ移動</a>".to_string());
        o.push("<header class=\"site-bar\">".to_string());
        o.push("<span class=\"brand\"><span class=\"long\">folio2</span><span class=\"short\">f2</span></span>".to_string());
        let nav = NAV
            .iter()
            .enumerate()
            .map(|(i, (href, name))| {
                let cur = if i == self.current {
                    " aria-current=\"page\""
                } else {
                    ""
                };
                format!("<a href=\"{href}\"{cur}>{name}</a>")
            })
            .collect::<String>();
        o.push(format!("<nav aria-label=\"読める面\">{nav}</nav>"));
        o.push(format!(
            "<span {}><button type=\"button\" class=\"fs-btn\" aria-label=\"文字の大きさを切り替える（いま: 標準）\" title=\"押すたびに 標準 → 大 → 特大 → 標準 と切り替わります\"><span class=\"aa\">Aa</span><span class=\"fs-k\">文字の大きさ</span><span class=\"fs-now\">標準</span></button></span>",
            self.dc(Component::FontSizeControl)
        ));
        o.push(format!(
            "<span class=\"here\"><b>{}</b><span class=\"here-doc\"> ▸ 全 {} 章 — </span><a href=\"#toc\">目次へ</a></span>",
            self.name,
            self.chapters()
        ));
        o.push(format!(
            "<span {}>生成 <b>{generated}</b> · <b>{version}</b>（{status}）</span>",
            self.dc(Component::FreshnessStamp)
        ));
        o.push(format!(
            "<span {}>{ceiling}</span>",
            self.dc(Component::CeilingStamp)
        ));
        o.push("</header>".to_string());
        o.push("<main id=\"main\" class=\"page\">".to_string());
    }

    /// 目次。`heads` は章ごとの（章の名・h2 の字面）を first から順に。
    pub fn toc(&self, o: &mut Vec<String>, heads: &[(String, String)], approval_t: &str) {
        o.push("<nav class=\"toc\" id=\"toc\" aria-label=\"目次\"><h2>目次</h2><ol>".to_string());
        for (i, (k, t)) in heads.iter().enumerate() {
            o.push(toc_li(
                &format!("s{}", self.first + i),
                &format!("{:02}", self.first + i),
                k,
                t,
            ));
        }
        o.push(toc_li("approval", "—", "承認欄", approval_t));
        o.push("</ol></nav>".to_string());
    }

    /// 章の帯（section）。`n` は章の番号・`lead` は組み立て済みの HTML。
    pub fn band(&self, o: &mut Vec<String>, n: usize, name: &str, h2: &str, lead: Option<&str>) {
        let (class, svg) = self.bands[n - self.first];
        o.push(format!(
            "<section id=\"s{n}\" {} class=\"{class}\"><span class=\"num\">{n:02}</span>",
            self.dc(Component::ChapterDeckBand)
        ));
        o.push(format!(
            "<p class=\"crumb\"><b>{}</b><span>›</span><span>{n:02} {name} {}/{}</span></p>",
            self.name,
            n - self.first + 1,
            self.chapters()
        ));
        o.push(format!(
            "<span class=\"kicker\"><svg class=\"ico\" viewBox=\"0 0 24 24\">{svg}</svg>{name}</span>"
        ));
        o.push(format!("<h2>{h2}</h2>"));
        if let Some(lead) = lead {
            o.push(format!("<p class=\"lead\">{lead}</p>"));
        }
        o.push("</section>".to_string());
    }

    /// 承認欄の帯（slim）。
    pub fn approval_band(&self, o: &mut Vec<String>, h2: &str, lead: &str) {
        o.push(format!(
            "<section id=\"approval\" {} class=\"band-3 slim\">",
            self.dc(Component::ChapterDeckBand)
        ));
        o.push("<span class=\"kicker\">承認欄</span>".to_string());
        o.push(format!("<h2>{h2}</h2>"));
        o.push(format!("<p class=\"lead\">{lead}</p>"));
        o.push("</section>".to_string());
    }

    /// prevnext・foot（ft-plain と機械のための面）・doc-locator・body と html の閉じ。`dl` は組み立て済みの HTML。
    pub fn foot(&self, o: &mut Vec<String>, version: &str, generated: &str, dl: &str) {
        self.foot_aside(o, version, generated, dl, "");
    }

    /// `foot` と同じ・doc-locator の行の末尾（入口へ戻る の後）に組み立て済みの `aside` を差し込む（便 74）。
    pub fn foot_aside(
        &self,
        o: &mut Vec<String>,
        version: &str,
        generated: &str,
        dl: &str,
        aside: &str,
    ) {
        o.push(format!(
            "<nav class=\"prevnext\"><a href=\"{}\"><span class=\"k\">前</span>{}</a><a href=\"{}\"><span class=\"k\">次</span>{}</a></nav>",
            self.prev.0, self.prev.1, self.next.0, self.next.1
        ));
        o.push("<footer class=\"foot\">".to_string());
        o.push(format!(
            "<p class=\"ft-plain\">このページは正本 {} から folio が生成した · {} {version}（{generated}）· 手で直さない</p>",
            self.source, self.name
        ));
        o.push(format!(
            "<details class=\"machine\" data-audience=\"machine\"><summary>機械のための面</summary><dl>{dl}</dl></details>"
        ));
        o.push("</footer>".to_string());
        o.push("</main>".to_string());
        o.push(format!(
            "<p class=\"doc-locator\">この文書の所属: 設計文書（design-intent）/ {} — <a href=\"index.html\">入口へ戻る</a>{aside}</p>",
            self.name
        ));
        o.push("</body>".to_string());
        o.push("</html>".to_string());
    }
}

/// 用語集への札 1 枚（入口の面の棚の札と同じ字面・語の数は語彙の正本の terms から・章と単位は ANNEXES から・便 74）。
pub fn glossary_chip(dir: &Path) -> R<String> {
    let v_doc = load(dir, "vocabulary.yaml")?;
    let terms = X::root(&v_doc, "vocabulary.yaml").f("terms")?.seq()?.len();
    let (c, unit) = ANNEXES
        .iter()
        .find(|(id, _)| *id == "vocabulary")
        .map(|(_, a)| *a)
        .ok_or("付録の表に vocabulary が無い")?;
    Ok(format!(
        "<span class=\"annex-chips\"><a href=\"constitution.html#s{c}\">付録 語彙 <span class=\"cnt\">{terms} {unit} → 憲法 §{c}</span></a></span>"
    ))
}

fn toc_li(href: &str, n: &str, k: &str, t: &str) -> String {
    format!(
        "<li><a href=\"#{href}\"><span class=\"n\">{n}</span><span class=\"k\">{k}</span><span class=\"t\">{t}</span></a></li>"
    )
}

/// card（`id` が在れば属性 id）。`body` は組み立て済みの HTML。
pub fn card(class: &str, id: Option<&str>, cid: &str, body: &str) -> String {
    let id = id.map_or_else(String::new, |i| format!(" id=\"{i}\""));
    format!("<div class=\"{class}\"{id}><div class=\"cid\">{cid}</div>{body}</div>")
}

// ── 用語集の語の行（憲法の面と要件書の面が共有する口・便 36）──

/// 語彙の正本（`v` = vocabulary.yaml の根）の terms を正本の順に 1 語 1 行で出す（部品 glossary-term-table の中身）。
/// 1 行 = div.grow〔id g-語の id〕・div.gword〔term + en が在れば span.en〕・div〔p.gdef の def + note が在れば
/// p.gdef + a.back「目次へ」〕。目次へのリンクは同じ面の #toc。憲法の面の章 07 と要件書の面の章 08 が同じ字面で出す。
/// `section` = 読む節の名（terms か field_terms・便 84）。
pub(crate) fn glossary_rows(o: &mut Vec<String>, v: &X<'_>, section: &str) -> R<()> {
    for t in v.f(section)?.seq()? {
        let en = t.f("en")?;
        let en = if matches!(en.v, Value::Null) {
            String::new()
        } else {
            format!("<span class=\"en\">{}</span>", en.e()?)
        };
        let note = match t.g("note")? {
            Some(n) => format!("<p class=\"gdef\">{}</p>", n.e()?),
            None => String::new(),
        };
        o.push(format!(
            "<div class=\"grow\" id=\"g-{}\"><div class=\"gword\">{}{en}</div><div><p class=\"gdef\">{}</p>{note}<a class=\"back\" href=\"#toc\">目次へ</a></div></div>",
            t.f("id")?.id()?,
            t.ef("term")?,
            t.ef("def")?
        ));
    }
    Ok(())
}

/// 用語集の欄の名前の節（便 84）: 語彙の根が field_terms を持ち空でないときだけ h3 1 行と部品 glossary-term-table の
/// div 1 つ（`dc` = その面の属性 data-component）を足す。無い・空なら 1 行も足さない。2 面が同じ字面で出す。
pub(crate) fn glossary_field_terms(o: &mut Vec<String>, v: &X<'_>, dc: &str) -> R<()> {
    let Some(ft) = v.g("field_terms")? else {
        return Ok(());
    };
    let n = ft.seq()?.len();
    if n == 0 {
        return Ok(());
    }
    o.push(format!("<h3>欄の名前 — {}</h3>", count_word(n, "語")));
    o.push(format!("<div {dc}>"));
    glossary_rows(o, v, "field_terms")?;
    o.push("</div>".to_string());
    Ok(())
}

// ── 図の枠（3 面が共有する口・便 34）──

/// 図の型（図の道具の型）→ figcaption の名札。組み立て時に部品目録 parts.json の figure_body_classes.type_ids から
/// 導出した対の列（便 52・手書きの写しを持たない）。表に無い型は `figure::render` が先に断る。
pub(crate) use catalog::FIGURE_TYPE_LABELS as FIGURE_LABELS;

/// 図の道具で描いた図 1 枚（図の id・図の本体・型の名札）。
pub struct Figure<'a> {
    pub id: &'a str,
    /// 図の本体（SVG・`figure::render` の戻り値そのまま・escape しない）
    pub body: String,
    pub label: &'static str,
}

/// 図の行 1 つ → 図 1 枚（id・本体・型の名札）。型が表に無い・spec が表でない・道具が無い・道具が通らないは
/// Err（面全体が「まだ分からない」）。
pub fn figure_body<'a>(dir: &Path, fig: &X<'a>) -> R<Figure<'a>> {
    let id = fig.f("id")?.id()?;
    let tx = fig.f("type")?;
    let kind =
        tx.v.as_str()
            .ok_or_else(|| format!("{}: 図の型が文字列でない", tx.at))?;
    let body = figure::render(dir, id, kind, &fig.f("spec")?)?;
    let label = FIGURE_LABELS
        .iter()
        .find(|(k, _)| *k == kind)
        .map(|(_, l)| *l)
        .ok_or_else(|| format!("図の型「{kind}」は図の道具の型でない"))?;
    Ok(Figure { id, body, label })
}

/// 図の枠 1 つ（figure-panel の開始タグ〜figcaption〜終了タグ・凡例は無し）。`i` は図の番号（1 から）・`caption` は
/// escape 済み・`refs` は根拠の id ごとの組み立て済みのリンク（空なら「根拠:」以降を出さない）。
pub fn figure_panel(
    o: &mut Vec<String>,
    f: &Frame,
    i: usize,
    fig: &Figure<'_>,
    caption: &str,
    refs: &[String],
) {
    let fn_ = format!("図 {i}");
    o.push(format!(
        "<figure {} data-role=\"diagram\" id=\"{}\">",
        f.dc(Component::FigurePanel),
        esc(fig.id)
    ));
    o.push(format!(
        "<div class=\"fig-title\"><span class=\"fn\">{fn_}</span>{caption} <span class=\"fig-tools\"><button class=\"zoom-btn\" type=\"button\">拡大</button></span><button class=\"zoom-close\" type=\"button\">✕ 閉じる</button></div>"
    ));
    o.push(fig.body.clone());
    let mut ver = format!("{fn_} · {} · {}", fig.label, esc(fig.id));
    if !refs.is_empty() {
        ver.push_str(&format!(" · 根拠: {}", refs.join("・")));
    }
    o.push(format!(
        "<figcaption><span class=\"ver\">{ver}</span></figcaption>"
    ));
    o.push("</figure>".to_string());
}

#[cfg(test)]
mod face_tests {
    use super::*;
    use crate::catalog::{FIGURE_TYPE_LABELS, LIMITS};
    use crate::cursor::safe_id;
    use crate::rules;
    use crate::yaml;

    #[test]
    fn face_tier_lookup_has_three_values_and_rejects_others() {
        assert_eq!(tier_of("always").unwrap().class, "tier-always");
        assert_eq!(tier_of("ask-first").unwrap().name, "確認してから変える");
        assert_eq!(tier_of("never").unwrap().color, "bad");
        assert!(tier_of("sometimes").is_err());
        assert!(tier_of("Always").is_err());
        let v = Value::Str("sometimes".into());
        assert!(X::root(&v, "t").parse(ce::Tier::from_name, "段").is_err());
    }

    /// 導出した型の全部を回し、name で引いた（値の字面・名札）の対の列。
    fn pairs<T: Copy>(
        all: &[T],
        name: fn(T) -> &'static str,
        label: fn(T) -> &'static str,
    ) -> Vec<(&'static str, &'static str)> {
        all.iter().map(|v| (name(*v), label(*v))).collect()
    }

    /// 凍結の針（P-10.1・便 50 §1 (e) 1）: 11 枚の文字列の名札を、便 50 の前の表の字面と順で固定する。
    #[test]
    fn face_labels_are_frozen_needles_for_the_string_tables() {
        assert_eq!(
            pairs(&ce::Strength::ALL, ce::Strength::name, strength_label),
            [
                ("must", "MUST"),
                ("must-not", "MUST NOT"),
                ("should", "SHOULD")
            ]
        );
        assert_eq!(
            pairs(&ce::Strength::ALL, ce::Strength::name, strength_meaning),
            [
                ("must", "必ず守る"),
                ("must-not", "決してしない"),
                ("should", "強い推奨（外すなら理由が要る）")
            ]
        );
        assert_eq!(
            pairs(&ce::Strength::ALL, ce::Strength::name, strength_prio),
            [("must", "must"), ("must-not", "must"), ("should", "should")]
        );
        assert_eq!(
            pairs(&ce::Pattern::ALL, ce::Pattern::name, pattern_label),
            [
                ("ubiquitous", "つねに"),
                ("event", "〜のとき"),
                ("state", "〜のあいだ"),
                ("unwanted", "〜になったら"),
                ("optional", "〜ならば")
            ]
        );
        assert_eq!(
            pairs(&ce::Binds::ALL, ce::Binds::name, binds_label),
            [("tool", "道具"), ("practice", "作法"), ("both", "両方")]
        );
        assert_eq!(
            pairs(
                &ce::MechanismKind::ALL,
                ce::MechanismKind::name,
                mechanism_kind_label
            ),
            [
                ("reject", "機械が拒む"),
                ("build-check", "生成時の検査"),
                ("human-review", "人が目で確かめる"),
                ("none", "なし")
            ]
        );
        assert_eq!(
            pairs(
                &ce::MechanismLive::ALL,
                ce::MechanismLive::name,
                mechanism_live_label
            ),
            [
                ("now", "いま動く"),
                ("M0", "機構がまだ無い"),
                ("delivery-0", "機構がまだ無い"),
                ("M1", "機構がまだ無い"),
                ("adr", "機構がまだ無い")
            ]
        );
        assert_eq!(
            pairs(
                &ce::MechanismLive::ALL,
                ce::MechanismLive::name,
                mechanism_live_meaning
            ),
            [
                ("now", "今の folio に在る"),
                ("M0", "床は判定しない・憲法の段の値は M0"),
                ("delivery-0", "床は判定しない・憲法の段の値は delivery-0"),
                ("M1", "床は判定しない・憲法の段の値は M1"),
                ("adr", "床は判定しない・憲法の段の値は adr")
            ]
        );
        assert_eq!(
            pairs(&ce::Stage::ALL, ce::Stage::name, stage_label),
            [("in-loop", "編集時"), ("post", "事後")]
        );
        assert_eq!(
            pairs(&ce::Polarity::ALL, ce::Polarity::name, polarity_label),
            [("fail-open", "開く"), ("fail-closed", "閉じる")]
        );
        assert_eq!(
            pairs(
                &ce::RationaleKind::ALL,
                ce::RationaleKind::name,
                rationale_kind_label
            ),
            [
                ("v1-incident", "v1 の実害"),
                ("scribe2-article", "scribe2 の条"),
                ("folio2-ruling", "持ち主の裁定")
            ]
        );
        assert_eq!(
            pairs(
                &ce::RetreatKind::ALL,
                ce::RetreatKind::name,
                retreat_kind_label
            ),
            [
                ("spike", "試して測る"),
                ("measure", "測る"),
                ("ruling", "持ち主に問う")
            ]
        );
    }

    /// 凍結の針（P-10.1・便 54 §1 (c) 1）: 規則の表の種別の名札 4 対と状態の class 3 対を、便 54 の前の表の字面と
    /// 正本（生成区間）の順で固定する。
    #[test]
    fn face_rule_labels_are_frozen_needles_for_kind_and_status() {
        assert_eq!(
            pairs(
                &rules::RuleKind::ALL,
                rules::RuleKind::name,
                rule_kind_label
            ),
            [
                ("deny", "測って落とす"),
                ("build-check", "生成時の検査"),
                ("detect", "記録のみ"),
                ("human-review", "人が守る作法")
            ]
        );
        assert_eq!(
            pairs(
                &rules::RuleStatus::ALL,
                rules::RuleStatus::name,
                rule_status_class
            ),
            [
                ("仮", "state warn"),
                ("凍結", "state ok"),
                ("未定", "state")
            ]
        );
    }

    /// 凍結の針（P-10.1・便 50 §1 (e) 1）: 段の名札（構造体 Tier）を便 50 の前の表の字面と順で固定する。
    #[test]
    fn face_labels_tier_is_a_frozen_needle() {
        // 行 = （値の字面・[name, en, class, color, meaning, remove, remove_html]）
        let rows: Vec<(&str, [&str; 7])> = ce::Tier::ALL
            .iter()
            .map(|k| {
                let t = tier_label(*k);
                (
                    k.name(),
                    [t.name, t.en, t.class, t.color, t.meaning, t.remove, t.remove_html],
                )
            })
            .collect();
        assert_eq!(
            rows,
            [
                (
                    "always",
                    [
                        "いつも守る",
                        "Always",
                        "tier-always",
                        "ok",
                        "道具も AI も、毎回これに従う",
                        "憲法の改訂（§6: 判断の記録 + 持ち主の承認）",
                        "憲法の改訂（<a class=\"xref\" href=\"#s6\">§6</a>: 判断の記録 + 持ち主の承認）",
                    ],
                ),
                (
                    "ask-first",
                    [
                        "確認してから変える",
                        "Ask-first",
                        "tier-askfirst",
                        "warn",
                        "やってよいが、実行前に持ち主へ確認する",
                        "その場の持ち主の確認",
                        "その場の持ち主の確認",
                    ],
                ),
                (
                    "never",
                    [
                        "絶対にやらない",
                        "Never",
                        "tier-never",
                        "bad",
                        "確認があってもやらない",
                        "憲法の改訂（確認では解けない）",
                        "<a class=\"xref\" href=\"#s6\">憲法の改訂</a>（確認では解けない）",
                    ],
                ),
            ]
        );
    }

    /// 表引きの口（便 50 (b)）: 値域の値は導出した型へ・外は Err で文言は lookup と同じ形。
    #[test]
    fn face_labels_parse_reads_the_derived_type_and_rejects_the_outside() {
        let v = Value::Str("must-not".into());
        assert_eq!(
            X::root(&v, "s")
                .parse(ce::Strength::from_name, "強度")
                .unwrap(),
            ce::Strength::MustNot
        );
        let v = Value::Str("may".into());
        assert_eq!(
            X::root(&v, "srs.yaml.requirements[0].strength")
                .parse(ce::Strength::from_name, "強度")
                .unwrap_err(),
            "srs.yaml.requirements[0].strength: 強度 の表に無い値「may」"
        );
        let v = Value::Int("1".into());
        assert_eq!(
            X::root(&v, "s")
                .parse(ce::Strength::from_name, "強度")
                .unwrap_err(),
            X::root(&v, "s").lookup(DOC_STATUS, "強度").unwrap_err()
        );
    }

    /// 便 135 §1 (c) 1: 字の部分の番号だけを包み、head・属性・a・svg の中と床の形に当たらない字と行き先の無い番号は
    /// 1 字も変えない（出力の全字で数える）。`all` が真なら条・要件・rules 行も包む。
    #[test]
    fn f135_link_ids_wraps_only_the_text_outside_links() {
        let to = |id: &str| (id != "ADR-3").then(|| format!("to-{id}"));
        let src = "<head><title>ADR-1</title></head><p title=\"ADR-1 > ADR-2\">ADR-1・ADR-12a・xADR-1・ADR-0・ADR-3</p><!-- ADR-1 --><a href=\"x\">ADR-1</a><svg><text>ADR-1</text></svg>(ADR-1)";
        assert_eq!(
            link_ids(src, false, to),
            "<head><title>ADR-1</title></head><p title=\"ADR-1 > ADR-2\"><a class=\"xref\" href=\"to-ADR-1\">ADR-1</a>・ADR-12a・xADR-1・ADR-0・ADR-3</p><!-- ADR-1 --><a href=\"x\">ADR-1</a><svg><text>ADR-1</text></svg>(<a class=\"xref\" href=\"to-ADR-1\">ADR-1</a>)"
        );
        // all が偽なら条・要件・rules 行は字のまま
        let src = "<p>条 P-6.3・FR1・R-4・ADR-2・FR1a</p>";
        assert_eq!(link_ids(src, false, to), "<p>条 P-6.3・FR1・R-4・<a class=\"xref\" href=\"to-ADR-2\">ADR-2</a>・FR1a</p>");
        assert_eq!(
            link_ids(src, true, to),
            "<p>条 <a class=\"xref\" href=\"to-P-6.3\">P-6.3</a>・<a class=\"xref\" href=\"to-FR1\">FR1</a>・<a class=\"xref\" href=\"to-R-4\">R-4</a>・<a class=\"xref\" href=\"to-ADR-2\">ADR-2</a>・FR1a</p>"
        );
        // script の中身と自己終了の svg
        let src = "<script>if (a<b) { ADR-1 }</script><svg/>ADR-1";
        assert_eq!(
            link_ids(src, false, to),
            "<script>if (a<b) { ADR-1 }</script><svg/><a class=\"xref\" href=\"to-ADR-1\">ADR-1</a>"
        );
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

    /// 凍結の針（P-10.1・便 52 §1 (d) 2・(b) の置き換えの番）: face.rs の名を通して読む上限 3 つが 7・4・4 で、face.rs の
    /// FIGURE_LABELS が便 52 の前の手書きの 5 対と順まで同じ。導出した定数の側（上限 3 つ・LIMITS・FIGURE_TYPE_LABELS）も同じ値。
    #[test]
    fn parts_derived_limits_and_figure_labels_are_frozen_needles() {
        use crate::face::{FIGURE_LABELS, MAX_PER_BAND, MAX_RAIL_NODES, MAX_STATE_NODES};
        const LABELS: [(&str, &str); 5] = [
            ("archify-architecture", "構成図（architecture）"),
            ("archify-workflow", "手順図（workflow）"),
            ("archify-sequence", "順序図（sequence）"),
            ("archify-dataflow", "流れ図（dataflow）"),
            ("archify-lifecycle", "状態図（lifecycle）"),
        ];
        // face.rs の名を通して
        assert_eq!((MAX_RAIL_NODES, MAX_STATE_NODES, MAX_PER_BAND), (7, 4, 4));
        assert_eq!(FIGURE_LABELS, LABELS);
        // 導出した定数の側
        assert_eq!(
            (
                catalog::PIPELINE_RAIL_MAX_NODES,
                catalog::STATE_STRIP_MAX_NODES,
                catalog::CONTEXT_BAND_MAX_PER_BAND
            ),
            (7, 4, 4)
        );
        assert_eq!(
            LIMITS,
            [
                ("pipeline-rail", "max_nodes", 7),
                ("context-band", "max_per_band", 4),
                ("state-strip", "max_nodes", 4),
            ]
        );
        assert_eq!(FIGURE_TYPE_LABELS, LABELS);
    }
}
