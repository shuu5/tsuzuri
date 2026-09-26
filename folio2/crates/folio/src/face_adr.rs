//! 判断の記録の面の生成器（便 25・docs/design/delivery-25.md §1 (b)〜(d)）。判断の記録の正本 1 本
//! （`adr/ADR-n.yaml`）から人が読むページ 1 枚を組む。生成物の文字列は 正本の値（α・escape して逐語）・
//! 名札の表（β・この file の表）・正本から数えた数（γ）のどれかで、案内の文は出さない。
//! 面に依らない口（head と site-bar・章の帯・card・toc・承認欄の帯・foot・部品の名札）は `face.rs` の `Frame` を呼ぶ。
//! 章の h2 のうち契約が名指すのは 01・02 と承認欄だけなので、03〜05 は章の名をそのまま h2 に出す（新しい字面を持たない）。
//! 見た目は便 27（delivery-27.md §1 (a)〜(c)）で直した: 表紙は h1 が短い名で title は副題・章 01/02 は文の頭の
//! 列挙を ol へ・章 04 は根拠を 4 群の card（id + 行き先の題）にして撤退条件を格子の外へ出す。
//! 図の章（便 33・FR15）: 正本に任意の図の節（figures）が 1 枚以上あれば章 06「図」を「改訂と帰結」の後・承認欄の
//! 前に置く。中身は設計ノートの面の図の章と同じ字面（便 34 からは `face.rs` の共有の図の枠）で、図の本体（SVG）は
//! `figure.rs` の `render` が図の道具で描いたものを逐語で埋める。図が 1 枚でも導出できなければ面全体を導出しない
//! （全部か無しか）。図が無い面は便 32 までと byte 不変。
//! 改訂の欄（便 137・docs/design/delivery-137.md §1 (b)）: 表紙の札を条文の改訂（amends）と判断の記録の改訂（revises）
//! の 2 つに分け（0 件でも出す）、章 05 の頭に空の欄の断りを 1 段落にまとめ、中身の在る欄は名札と同じ字の h3 の下に
//! 並べる。revises の行は相手の面へのリンク・決定・向きの名札（表 REVISE）・summary の逐語。
//! 逆向きの導線（便 148・docs/design/delivery-148.md §1 (b)）: 正本の revises は改訂する側だけが持つので、改訂される
//! 側の面は `adr/` の発効の全記録の revises を走査して行を導出する（P-6.3）。行が 1 つ以上なら章 05 の自分の revises の
//! 一覧の後・帰結の前に h3 と一覧を、表紙に札を 1 つ足す（0 件の面は便 147 までと byte 不変）。走査が読めなければ面を
//! 導出しない（P-4.1）。
//! 強調の印（便 149・docs/design/delivery-149.md §1 (b)）: 散文の 6 つの欄（context と decision は列挙で分けた後の
//! 断片ごと・案の text と reason・帰結の各行・注）は、escape の後に左から順に対になった印 STRONG_MARK を strong の
//! 要素に写す（閉じない印と中身が空の対は生のまま）。題・平易文・逐語の引用などほかの欄は写さない。

use std::fs;
use std::path::Path;

use crate::catalog::Component;
use crate::constitution_enums as ce;
use crate::cursor::{self, R, X};
use crate::face::{self, Frame, anchor, card};
use crate::face_index_read;

/// 判断の記録の面が使う部品（9 種・便 33 で figure-panel を・便 40 で ceiling-stamp を足した）。
pub const PARTS: [Component; 9] = [
    Component::FreshnessStamp,
    Component::CeilingStamp,
    Component::FontSizeControl,
    Component::DocCoverBand,
    Component::ApprovalBlock,
    Component::ChapterDeckBand,
    Component::SectionLeadCallout,
    Component::FigurePanel,
    Component::ItemRow,
];

/// 章 01〜05 の名（見出しの帯の kicker・目次の名）。
const CHAPTERS: [&str; 5] = ["問題", "決定", "案", "根拠と撤退条件", "改訂と帰結"];

/// 章 01〜05 の h2（01・02 は契約の字面・03〜05 は章の名）。
const H2: [&str; 5] = [
    "何が問題か",
    "何を決めたか",
    "案",
    "根拠と撤退条件",
    "改訂と帰結",
];

/// 章 06（図）の名（帯の kicker・目次の名）。
const FIGURES_CHAPTER: &str = "図";

/// 章 01〜06 の帯の class と kicker の絵記号（01〜05 は要件書の面の章 01〜05 の写し・06 は要件書の面の band-6 の写し）。
/// 章の数だけ先頭から切り出して `Frame` に渡す（`static` なので切り出しは 'static）。
static BANDS: [(&str, &str); 6] = [
    (
        "band-1",
        "<circle cx=\"12\" cy=\"12\" r=\"9\"/><circle cx=\"12\" cy=\"12\" r=\"3\"/>",
    ),
    (
        "band-2",
        "<path d=\"M3 7l9-4 9 4-9 4-9-4z\"/><path d=\"M3 7v10l9 4 9-4V7\"/>",
    ),
    ("band-3", "<path d=\"M20 6L9 17l-5-5\"/>"),
    ("band-4", "<path d=\"M4 18h16M6 14V8M12 14V4M18 14v-4\"/>"),
    (
        "band-5",
        "<path d=\"M9 11l3 3L22 4\"/><path d=\"M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11\"/>",
    ),
    ("band-6", "<path d=\"M4 18h16M6 14V8M12 14V4M18 14v-4\"/>"),
];

/// 判断の記録の面の FRAME の favicon の字面。
const FAVICON: &str = "<link rel=\"icon\" href=\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 32 32'%3E%3Crect width='32' height='32' rx='7' fill='%235f45a6'/%3E%3Ctext x='16' y='22' font-size='16' font-weight='700' text-anchor='middle' fill='%23ffffff' font-family='sans-serif'%3E要%3C/text%3E%3C/svg%3E\">";

/// 判断の記録の面の骨格（章の数 = 5 + 図の章の有無）。nav の aria-current は付かない（読める面の nav に判断の記録は無い）。
/// prevnext は ADR-n の数の順で隣の記録（廃止も飛ばさない）・両端は入口（便 65）。
fn frame(chapters: usize, dir: &Path, id: &str) -> R<Frame> {
    let ids = record_ids(dir)?;
    let at = ids
        .iter()
        .position(|x| x == id)
        .ok_or_else(|| format!("adr/: 判断の記録の列に {id} が無い"))?;
    // 名 = id + 半角空白 + 題（題が無ければ id だけ）
    let links: Vec<(String, String)> = ids
        .iter()
        .map(|x| {
            let title = record_title(dir, x);
            let name = if title.is_empty() {
                x.clone()
            } else {
                format!("{x} {title}")
            };
            (format!("{}.html", anchor(x)), name)
        })
        .collect();
    let (prev, next) = face::neighbors(&links, at);
    Ok(Frame {
        name: "判断の記録",
        source: "adr/ADR-n.yaml",
        favicon: FAVICON,
        current: 3,
        first: 1,
        bands: &BANDS[..chapters],
        prev,
        next,
        parts: &PARTS,
    })
}

// ── 名札の表（β・表に無い値は導出できない）──

/// 状態（status）→ 見出しの帯と機械の面に出す名札。
const STATUS: &[(&str, &str)] = &[
    ("proposed", "提案中・拘束力なし"),
    ("accepted", "発効"),
    ("retired", "廃止"),
];

/// 案の判定（verdict）→ 名札。
const VERDICT: &[(&str, &str)] = &[("adopted", "採用"), ("rejected", "退けた")];

/// 改訂の向き（revises[].kind）→ 名札（鍵の並びは床の定数 REVISE_KIND と同じ・便 137）。
const REVISE: &[(&str, &str)] = &[("narrow", "狭める"), ("widen", "広げる")];

/// 条文の改訂（amends）の欄の名札（表紙の札と章 05 の h3）。
const AMENDS_LABEL: &str = "条文の改訂";

/// 判断の記録の改訂（revises）の欄の名札（表紙の札と章 05 の h3）。
const REVISES_LABEL: &str = "判断の記録の改訂";

/// ほかの判断の記録から受けた改訂（逆向きの行）の名札（表紙の札と章 05 の h3・便 148）。
const REVISED_BY_LABEL: &str = "ほかの判断の記録による改訂";

/// 逆向きの行を読む改訂する側の状態（発効だけ・便 148 §1 (d) の 3）。
const REVISED_BY_STATUS: &[&str] = &["accepted"];

/// 散文の強調の印（markdown の星 2 つ・便 149）。
const STRONG_MARK: &str = "**";

/// 印の対を写す先の要素の開きと閉じ（属性なし・部品目録の外にならない・便 149）。
const STRONG_TAG: (&str, &str) = ("<strong>", "</strong>");

/// escape 済みの断片の印の対を左から順に strong の要素に写す（便 149 §1 (b) の 2）。中身は escape 済みのまま。
/// 閉じない印（後に印が無い）はそこから末尾まで・中身が空の対は星 4 つのまま出す。入れ子は作らず、1 つの星は触らない。
fn strong(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(open) = rest.find(STRONG_MARK) {
        let after = &rest[open + STRONG_MARK.len()..];
        let Some(close) = after.find(STRONG_MARK) else {
            break;
        };
        out.push_str(&rest[..open]);
        let inner = &after[..close];
        if inner.is_empty() {
            out.push_str(STRONG_MARK);
            out.push_str(STRONG_MARK);
        } else {
            out.push_str(STRONG_TAG.0);
            out.push_str(inner);
            out.push_str(STRONG_TAG.1);
        }
        rest = &after[close + STRONG_MARK.len()..];
    }
    out.push_str(rest);
    out
}

/// 撤退条件の種類（retreat.kind）→ 判断の記録の面の名札（憲法の面の名札 `face::retreat_kind_label` と字面が違う）。
/// 憲法の値域から導出した型への網羅の場合分け（便 50・その他の枝なし = 値が足されても消えても組み立てが通らない）。
fn retreat_kind_label(k: ce::RetreatKind) -> &'static str {
    match k {
        ce::RetreatKind::Spike => "小さな試し",
        ce::RetreatKind::Measure => "数えた値",
        ce::RetreatKind::Ruling => "持ち主の裁定",
    }
}

/// 撤退条件の種類の欄 → 名札（値域に無い値は Err・文言は表引きと同じ）。
fn retreat_kind(x: &X<'_>) -> R<&'static str> {
    Ok(retreat_kind_label(
        x.parse(ce::RetreatKind::from_name, "撤退条件の種類")?,
    ))
}

/// 撤退条件の種類 → 表紙の 1 文（何をもって捨てるか・便 71 §1 (b)・`retreat_kind_label` と同じ網羅の場合分け）。
fn retreat_sentence(k: ce::RetreatKind) -> &'static str {
    match k {
        ce::RetreatKind::Spike => "小さな試しで確かめて、外れたら捨てる",
        ce::RetreatKind::Measure => "数えた値が条件を超えたら捨てる",
        ce::RetreatKind::Ruling => "持ち主の裁定で捨てる",
    }
}

/// 状態の名札（短い）と状態の行（組み立て済みの HTML）。
struct Status {
    label: &'static str,
    line: String,
}

/// 正本から数えた数（γ）。
struct Counts {
    options: usize,
    adopted: usize,
    rejected: usize,
    basis: usize,
    amends: usize,
    revises: usize,
    /// ほかの記録から受けた改訂の行の数（正本の欄でなく走査の数・表紙の札だけが使う）
    revised_by: usize,
    figures: usize,
}

/// ほかの判断の記録から受けた改訂の 1 行（改訂する側の id・decision と summary は escape 済み）。
struct RevisedBy {
    by: String,
    decision: String,
    kind: &'static str,
    summary: String,
}

/// 根拠の id の行き先を解くための、他の正本の id と題の一覧。
struct Ctx {
    /// 条の（id・title）
    articles: Vec<(String, String)>,
    /// 規範文 id → その条の id
    statements: Vec<(String, String)>,
    /// rules 行の（id・what）
    rules: Vec<(String, String)>,
    /// 要件書の（id・title）（要件・非機能要件・受入基準・制約・ゴール）
    reqs: Vec<(String, String)>,
}

/// 根拠の 4 群の名（順もこのとおり・便 27 §1 (c)）。
const BASIS_GROUPS: [&str; 4] = ["憲法の条", "数値の表", "要件書", "判断の記録"];

/// 根拠の id 1 つの行き先（群・href・題）。href が None なら行き先が無い。
struct Target {
    group: usize,
    href: Option<String>,
    /// 行き先の欄の題（読めなければ空・題の span を出さない）
    title: String,
}

/// 正本 1 本 → 判断の記録の面の HTML（決定的）。天井の名札は印から読む（便 40・便 83）。
pub fn derive(dir: &Path, id: &str) -> R<String> {
    check_id_shape(id)?;
    let name = format!("adr/{id}.yaml");
    let a_doc = cursor::load(dir, &name)?;
    let c_doc = cursor::load(dir, "constitution.yaml")?;
    let r_doc = cursor::load(dir, "rules.yaml")?;
    let s_doc = cursor::load(dir, "srs.yaml")?;
    let a = X::root(&a_doc, &name);
    let ctx = context(
        &X::root(&c_doc, "constitution.yaml"),
        &X::root(&r_doc, "rules.yaml"),
        &X::root(&s_doc, "srs.yaml"),
    )?;

    let file_id = a.f("id")?.id()?;
    if file_id != id {
        return Err(format!("{name}: 欄 id「{file_id}」が --id「{id}」と違う"));
    }
    let st = status(&a, dir, &ctx)?;
    let figs = match a.g("figures")? {
        Some(x) => x.seq()?,
        None => Vec::new(),
    };
    let revised = revised_by(dir, id)?;
    let mut counts = counts(&a, figs.len())?;
    counts.revised_by = revised.len();
    let f = frame(CHAPTERS.len() + usize::from(!figs.is_empty()), dir, id)?;
    let stamp = face::ceiling_stamp(dir)?;

    let mut o: Vec<String> = Vec::new();
    head(&mut o, &f, &a, id, &st, &stamp)?;
    cover(&mut o, &f, &a, id, &st, &counts)?;
    toc(&f, &mut o, figs.len());
    prose_chapter(&mut o, &f, 1, &a.ef("context")?);
    prose_chapter(&mut o, &f, 2, &a.ef("decision")?);
    options_chapter(&mut o, &f, &a)?;
    basis_chapter(&mut o, &f, &a, dir, &ctx)?;
    amends_chapter(&mut o, &f, &a, dir, &ctx, &revised)?;
    if !figs.is_empty() {
        figures_chapter(&mut o, &f, CHAPTERS.len() + 1, &figs, dir, &ctx)?;
    }
    approval_chapter(&mut o, &f, &a, &st)?;
    foot(&mut o, &f, &a, id, &counts, &face::glossary_chip(dir)?)?;
    Ok(format!("{}\n", o.join("\n")))
}

// ── 読みと解き ──

/// `--id` は `ADR-` + ASCII の数字列 1 つ以上。
fn check_id_shape(id: &str) -> R<()> {
    if id
        .strip_prefix("ADR-")
        .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
    {
        Ok(())
    } else {
        Err(format!("--id「{id}」は id の形でない（ADR- と数字）"))
    }
}

fn digits(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())
}

/// 行き先の欄の題（欄が無い・空・読めないときは空）。題は行き先の欄が読めるときだけ出す。
fn field_title(x: &X<'_>, key: &str) -> String {
    match x.g(key) {
        Ok(Some(v)) => v.e().unwrap_or_default(),
        _ => String::new(),
    }
}

fn context(c: &X<'_>, r: &X<'_>, s: &X<'_>) -> R<Ctx> {
    let mut articles = Vec::new();
    let mut statements = Vec::new();
    for a in c.f("articles")?.seq()? {
        let aid = a.f("id")?.id()?.to_string();
        for st in a.f("statements")?.seq()? {
            statements.push((st.f("id")?.id()?.to_string(), aid.clone()));
        }
        articles.push((aid, field_title(&a, "title")));
    }
    let mut rules = Vec::new();
    for section in ["thresholds", "discipline"] {
        for x in r.f(section)?.seq()? {
            rules.push((x.f("id")?.id()?.to_string(), field_title(&x, "what")));
        }
    }
    let mut reqs = Vec::new();
    for section in [
        "requirements",
        "nonfunctional",
        "acceptance",
        "constraints",
        "goals",
    ] {
        for x in s.f(section)?.seq()? {
            reqs.push((x.f("id")?.id()?.to_string(), field_title(&x, "title")));
        }
    }
    Ok(Ctx {
        articles,
        statements,
        rules,
        reqs,
    })
}

/// `adr/` の下の判断の記録の id（file 名 `ADR-<数>.yaml` の stem）を数の昇順に並べる（字の順でない・廃止も入れる・便 65）。
fn record_ids(dir: &Path) -> R<Vec<String>> {
    let entries = fs::read_dir(dir.join("adr")).map_err(|e| format!("adr/: 読めない: {e}"))?;
    let mut ids: Vec<(u64, String)> = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("adr/: 読めない: {e}"))?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let Some(stem) = name.strip_suffix(".yaml").filter(|s| s.starts_with("ADR-")) else {
            continue;
        };
        if let Some(n) = face_index_read::adr_number(stem) {
            ids.push((n, stem.to_string()));
        }
    }
    ids.sort();
    Ok(ids.into_iter().map(|(_, id)| id).collect())
}

/// 判断の記録の題（file が無い・読めない・title の欄が無い・空なら空。面は 2 にしない）。
fn record_title(dir: &Path, id: &str) -> String {
    match cursor::load(dir, &format!("adr/{id}.yaml")) {
        Ok(doc) => field_title(&X::root(&doc, id), "title"),
        Err(_) => String::new(),
    }
}

/// ほかの判断の記録から `id` が受けた改訂の行（便 148 §1 (b) の 2）。改訂する側の id の数の順・正本の順に、
/// 状態が REVISED_BY_STATUS の記録の revises だけを読む。記録が読めない・状態の欄が無い・revises が一覧でない・
/// target が判断の記録の id の形でない・向きが表 REVISE の外なら Err（改訂されていないと黙って出さない・P-4.1）。
fn revised_by(dir: &Path, id: &str) -> R<Vec<RevisedBy>> {
    let mut rows = Vec::new();
    for by in record_ids(dir)? {
        if by == id {
            continue;
        }
        let name = format!("adr/{by}.yaml");
        let doc = cursor::load(dir, &name)?;
        let r = X::root(&doc, &name);
        if !REVISED_BY_STATUS.contains(&r.f("status")?.text()?.as_str()) {
            continue;
        }
        for e in entries(&r, "revises")? {
            let tx = e.f("target")?;
            let target = tx.id()?;
            if check_id_shape(target).is_err() {
                return Err(format!(
                    "{}: id「{target}」が判断の記録の id の形でない",
                    tx.at
                ));
            }
            let kind = e.f("kind")?.lookup(REVISE, "改訂の向き")?;
            if target == id {
                rows.push(RevisedBy {
                    by: by.clone(),
                    decision: e.ef("decision")?,
                    kind,
                    summary: e.ef("summary")?,
                });
            }
        }
    }
    Ok(rows)
}

/// 根拠の id の群と行き先と題。4 形のどれでもない id は Err。
fn resolve(dir: &Path, ctx: &Ctx, id: &str) -> R<Target> {
    let bad = || format!("根拠の id「{id}」は id の形でない（条・rules 行・要件・判断の記録）");
    let found = |pairs: &[(String, String)], group: usize, href: String| match pairs
        .iter()
        .find(|(k, _)| k == id)
    {
        Some((_, title)) => Target {
            group,
            href: Some(href),
            title: title.clone(),
        },
        None => Target {
            group,
            href: None,
            title: String::new(),
        },
    };
    if let Some(rest) = ["P-", "A-", "N-"].iter().find_map(|p| id.strip_prefix(p)) {
        return match rest.split_once('.') {
            // 枝番付きの規範文 id は条の anchor へ（字は枝番付きのまま・題は条の title）
            Some((n, sub)) if digits(n) && digits(sub) => Ok(ctx
                .statements
                .iter()
                .find(|(sid, _)| sid == id)
                .map_or_else(
                    || Target {
                        group: 0,
                        href: None,
                        title: String::new(),
                    },
                    |(_, aid)| Target {
                        group: 0,
                        href: Some(format!("constitution.html#{}", anchor(aid))),
                        title: ctx
                            .articles
                            .iter()
                            .find(|(k, _)| k == aid)
                            .map_or_else(String::new, |(_, t)| t.clone()),
                    },
                )),
            None if digits(rest) => Ok(found(
                &ctx.articles,
                0,
                format!("constitution.html#{}", anchor(id)),
            )),
            _ => Err(bad()),
        };
    }
    if let Some(rest) = ["R-", "D-"].iter().find_map(|p| id.strip_prefix(p)) {
        return if digits(rest) {
            Ok(found(
                &ctx.rules,
                1,
                format!("constitution.html#{}", anchor(id)),
            ))
        } else {
            Err(bad())
        };
    }
    if let Some(rest) = ["FR", "NFR", "AC", "CON", "GOAL"]
        .iter()
        .find_map(|p| id.strip_prefix(p))
    {
        return if digits(rest) {
            Ok(found(&ctx.reqs, 2, format!("srs.html#{}", anchor(id))))
        } else {
            Err(bad())
        };
    }
    if let Some(rest) = id.strip_prefix("ADR-") {
        return if digits(rest) {
            let file = dir.join("adr").join(format!("{id}.yaml"));
            Ok(if file.is_file() {
                Target {
                    group: 3,
                    href: Some(format!("{}.html", anchor(id))),
                    title: record_title(dir, id),
                }
            } else {
                Target {
                    group: 3,
                    href: None,
                    title: String::new(),
                }
            })
        } else {
            Err(bad())
        };
    }
    Err(bad())
}

/// 行き先の字面（在ればリンク・無ければ「（まだ分からない）」）。
fn link_text(t: &Target, id: &str) -> String {
    match &t.href {
        Some(href) => format!("<a class=\"xref\" href=\"{href}\">{id}</a>"),
        None => format!("{id}（まだ分からない）"),
    }
}

/// 根拠の id 1 つ（在ればリンク・無ければ「（まだ分からない）」）。
fn id_link(dir: &Path, ctx: &Ctx, x: &X<'_>) -> R<String> {
    let id = x.id()?;
    Ok(link_text(&resolve(dir, ctx, id)?, id))
}

/// 状態の名札と状態の行。accepted に承認欄が無い・retired に後継が無い、は導出できない。
fn status(a: &X<'_>, dir: &Path, ctx: &Ctx) -> R<Status> {
    let sx = a.f("status")?;
    let label = sx.lookup(STATUS, "状態")?;
    let line = match sx.v.as_str() {
        Some("accepted") => {
            let ap = a
                .g("approval")?
                .ok_or_else(|| format!("{}: accepted に承認欄が無い", a.at))?;
            format!("発効・拘束力あり（承認 {}）", ap.ef("date")?)
        }
        Some("retired") => {
            let sb = a
                .g("superseded_by")?
                .ok_or_else(|| format!("{}: retired に superseded_by が無い", a.at))?;
            format!("廃止 → 後継 {}", id_link(dir, ctx, &sb)?)
        }
        _ => "提案中・拘束力なし → 持ち主の承認で発効".to_string(),
    };
    Ok(Status { label, line })
}

fn counts(a: &X<'_>, figures: usize) -> R<Counts> {
    let options = a.f("options")?.seq()?;
    if options.is_empty() {
        return Err(format!("{}: 案が 1 つも無い", a.f("options")?.at));
    }
    let mut adopted = 0;
    for o in &options {
        if o.f("verdict")?.lookup(VERDICT, "判定")? == "採用" {
            adopted += 1;
        }
    }
    Ok(Counts {
        options: options.len(),
        adopted,
        rejected: options.len() - adopted,
        basis: a.f("basis")?.seq()?.len(),
        amends: entries(a, "amends")?.len(),
        revises: entries(a, "revises")?.len(),
        revised_by: 0,
        figures,
    })
}

/// 任意の一覧の欄の要素（無い・null は空）。
fn entries<'a>(a: &X<'a>, key: &str) -> R<Vec<X<'a>>> {
    match a.g(key)? {
        Some(x) => x.seq(),
        None => Ok(Vec::new()),
    }
}

// ── 骨格 ──

fn head(o: &mut Vec<String>, f: &Frame, a: &X<'_>, id: &str, st: &Status, stamp: &str) -> R<()> {
    // 鮮度の札と足の行の（名・日付）は入口のカードと同じ口（便 146・147）
    let (dated, date) = face::adr_dated(a)?;
    f.head_dated(
        o,
        &format!("folio2 — 判断の記録 {id}（{}）", st.label),
        (dated, &date),
        id,
        st.label,
        stamp,
    );
    Ok(())
}

fn meta_span(k: &str, v: &str) -> String {
    format!("<span class=\"m\"><span class=\"k\">{k}</span><span class=\"v\">{v}</span></span>")
}

fn cover(o: &mut Vec<String>, f: &Frame, a: &X<'_>, id: &str, st: &Status, n: &Counts) -> R<()> {
    o.push(format!("<header {}>", f.dc(Component::DocCoverBand)));
    o.push(format!(
        "<p class=\"cover-eyebrow\"><span class=\"doc-type\">判断の記録 (ADR)</span> <span>folio2 — {id}</span></p>"
    ));
    // h1 は短い名（title は文の長さなので副題へ・便 27 §1 (a)）
    o.push(format!("<h1>判断の記録 {id}</h1>"));
    o.push(format!("<p class=\"sub-title\">{}</p>", a.ef("title")?));
    o.push(format!(
        "<div class=\"summary-card\"><span class=\"ic\">要</span><div><p class=\"lab\">やさしく言うと</p><p class=\"txt\">{}</p></div></div>",
        a.ef("plain")?
    ));
    o.push("<div class=\"cover-meta\">".to_string());
    o.push(meta_span("状態", st.label));
    o.push(meta_span("日付", &a.ef("date")?));
    o.push(meta_span(
        "案",
        &format!(
            "{} 件（採用 {}・退けた {}）",
            n.options, n.adopted, n.rejected
        ),
    ));
    o.push(meta_span("根拠", &format!("{} 件", n.basis)));
    o.push(meta_span(AMENDS_LABEL, &format!("{} 件", n.amends)));
    o.push(meta_span(REVISES_LABEL, &format!("{} 件", n.revises)));
    // 受けた改訂は 1 つ以上のときだけ（改訂されていない面は便 147 までと byte 不変・便 148）
    if n.revised_by > 0 {
        o.push(meta_span(REVISED_BY_LABEL, &format!("{} 件", n.revised_by)));
    }
    o.push(meta_span(
        "撤退条件",
        retreat_sentence(
            a.f("retreat")?
                .f("kind")?
                .parse(ce::RetreatKind::from_name, "撤退条件の種類")?,
        ),
    ));
    // 図は 1 枚以上のときだけ（図なしの面は便 32 までと byte 不変）
    if n.figures > 0 {
        o.push(meta_span(FIGURES_CHAPTER, &format!("{} 枚", n.figures)));
    }
    o.push("</div>".to_string());
    o.push(format!(
        "<p class=\"cover-status\"><span class=\"k\">状態</span><span>{}（<a href=\"#approval\">承認欄へ</a>）</span></p>",
        st.line
    ));
    o.push("</header>".to_string());
    Ok(())
}

fn toc(f: &Frame, o: &mut Vec<String>, figures: usize) {
    let mut heads = CHAPTERS
        .iter()
        .zip(H2)
        .map(|(k, t)| ((*k).to_string(), t.to_string()))
        .collect::<Vec<_>>();
    if figures > 0 {
        heads.push((FIGURES_CHAPTER.to_string(), format!("{figures} 枚")));
    }
    f.toc(o, &heads, "承認");
}

fn band(o: &mut Vec<String>, f: &Frame, n: usize) {
    f.band(o, n, CHAPTERS[n - 1], H2[n - 1], None);
}

// ── 章 ──

/// 文の頭の列挙の印「(k) 」の位置（開きの括弧の byte・印の直後の byte）。印と数えるのは、印の直前
/// （末尾の空白を除く）が本文の先頭か句点で、かつ番号が 1 から 1 ずつ増えて続くときだけ（便 27 §1 (b)）。
fn item_marks(body: &str) -> Vec<(usize, usize)> {
    let b = body.as_bytes();
    let mut marks = Vec::new();
    let mut want = 1u32;
    let mut i = 0;
    while i < b.len() {
        // 半角の開き括弧 + ASCII の数字 1 つ以上 + 半角の閉じ括弧 + 半角空白 1 つ
        if b[i] == b'(' {
            let mut j = i + 1;
            while j < b.len() && b[j].is_ascii_digit() {
                j += 1;
            }
            if j > i + 1 && b.get(j) == Some(&b')') && b.get(j + 1) == Some(&b' ') {
                let head = body[..i].trim_end();
                let at_head = head.is_empty() || head.ends_with('。');
                if at_head && body[i + 1..j].parse::<u32>() == Ok(want) {
                    marks.push((i, j + 2));
                    want += 1;
                    i = j + 2;
                    continue;
                }
            }
        }
        i += 1;
    }
    marks
}

/// 章 01・02（帯 + chapbody）。`body` は escape 済み。文の頭の印が 2 つ以上なら前置きの p と ol へ分け、
/// 1 つ以下なら p 1 つ（便 27 §1 (b)）。強調の印は分けた後の断片ごとに写す（便 149）。
fn prose_chapter(o: &mut Vec<String>, f: &Frame, n: usize, body: &str) {
    band(o, f, n);
    o.push("<div class=\"chapbody\">".to_string());
    let marks = item_marks(body);
    if marks.len() < 2 {
        o.push(format!("<p>{}</p>", strong(body)));
    } else {
        let intro = body[..marks[0].0].trim();
        if !intro.is_empty() {
            o.push(format!("<p class=\"intro\">{}</p>", strong(intro)));
        }
        o.push("<ol class=\"items\">".to_string());
        for (k, (_, end)) in marks.iter().enumerate() {
            let stop = marks.get(k + 1).map_or(body.len(), |(start, _)| *start);
            o.push(format!("<li>{}</li>", strong(body[*end..stop].trim())));
        }
        o.push("</ol>".to_string());
    }
    o.push("</div>".to_string());
}

/// 章 03（案・採用と退けた案を同じ形で）。
fn options_chapter(o: &mut Vec<String>, f: &Frame, a: &X<'_>) -> R<()> {
    band(o, f, 3);
    o.push("<div class=\"chapbody\">".to_string());
    for opt in a.f("options")?.seq()? {
        let oid = opt.f("id")?.id()?;
        let verdict = opt.f("verdict")?;
        o.push(format!(
            "<article {} id=\"opt-{oid}\">",
            f.dc(Component::ItemRow)
        ));
        o.push(format!(
            "<div class=\"ir-head\"><span class=\"rid\">案 {oid}</span><h3 class=\"rt\">{}</h3><span class=\"badges\"><span class=\"pill\">{}</span></span></div>",
            opt.ef("name")?,
            verdict.lookup(VERDICT, "判定")?
        ));
        o.push(format!("<p class=\"norm\">{}</p>", strong(&opt.ef("text")?)));
        o.push(format!(
            "<div class=\"plain\"><span class=\"pk\">理由</span>{}</div>",
            strong(&opt.ef("reason")?)
        ));
        o.push(format!(
            "<details class=\"machine\" data-audience=\"machine\"><summary>機械のための面</summary><dl><dt>verdict</dt><dd>{}</dd></dl></details>",
            verdict.e()?
        ));
        o.push("</article>".to_string());
    }
    o.push("</div>".to_string());
    Ok(())
}

/// 章 04（根拠の 4 群と撤退条件・便 27 §1 (c)）。
fn basis_chapter(o: &mut Vec<String>, f: &Frame, a: &X<'_>, dir: &Path, ctx: &Ctx) -> R<()> {
    band(o, f, 4);
    o.push("<div class=\"chapbody\">".to_string());
    // 群の順は BASIS_GROUPS の順・群の中は basis の順
    let mut groups: [Vec<String>; 4] = std::array::from_fn(|_| Vec::new());
    for b in a.f("basis")?.seq()? {
        let id = b.id()?;
        let t = resolve(dir, ctx, id)?;
        // 題は行き先の欄が読めるときだけ出す
        let title = if t.href.is_some() && !t.title.is_empty() {
            format!("<span>{}</span>", t.title)
        } else {
            String::new()
        };
        groups[t.group].push(format!("<li>{}{title}</li>", link_text(&t, id)));
    }
    o.push(format!(
        "<div {} style=\"--band-n:{}\">",
        f.dc(Component::SectionLeadCallout),
        groups.iter().filter(|g| !g.is_empty()).count()
    ));
    for (k, g) in groups.iter().enumerate() {
        if g.is_empty() {
            continue;
        }
        o.push(card(
            "card",
            None,
            &format!("{}（{}）", BASIS_GROUPS[k], g.len()),
            &format!("<ul class=\"basis\">\n{}\n</ul>", g.join("\n")),
        ));
    }
    o.push("</div>".to_string());
    // 撤退条件は格子の外（1 列に潰さない）
    let rt = a.f("retreat")?;
    o.push(card(
        "card retreat",
        None,
        &format!("撤退条件（{}）", retreat_kind(&rt.f("kind")?)?),
        &format!("<p>{}</p>", rt.ef("condition")?),
    ));
    o.push("</div>".to_string());
    Ok(())
}

/// 章 05（改訂・帰結・反対側からの確認・置き換え・注）。空の改訂の欄の断りは頭の 1 段落にまとめ、
/// 中身の在る欄は名札と同じ字の h3 の下に並べる（便 137）。ほかの記録から受けた改訂（`revised`）は自分の revises の
/// 後・帰結の前に（断りは正本の欄の字なので変えない・便 148）。
fn amends_chapter(
    o: &mut Vec<String>,
    f: &Frame,
    a: &X<'_>,
    dir: &Path,
    ctx: &Ctx,
    revised: &[RevisedBy],
) -> R<()> {
    band(o, f, 5);
    o.push("<div class=\"chapbody\">".to_string());
    let amends = entries(a, "amends")?;
    let revises = entries(a, "revises")?;
    let none: Vec<String> = [(AMENDS_LABEL, amends.is_empty()), (REVISES_LABEL, revises.is_empty())]
        .iter()
        .filter(|(_, empty)| *empty)
        .map(|(label, _)| format!("{label}なし"))
        .collect();
    if !none.is_empty() {
        o.push(format!("<p>{}</p>", none.join("・")));
    }
    if !amends.is_empty() {
        o.push(format!("<h3>{AMENDS_LABEL}</h3>"));
        o.push("<ul>".to_string());
        for e in &amends {
            o.push(format!(
                "<li>{} {}.{}: 「{}」→「{}」</li>",
                e.ef("version")?,
                e.ef("target")?,
                e.ef("field")?,
                e.ef("previous_text")?,
                e.ef("new_text")?
            ));
        }
        o.push("</ul>".to_string());
    }
    if !revises.is_empty() {
        o.push(format!("<h3>{REVISES_LABEL}</h3>"));
        o.push("<ul>".to_string());
        for e in &revises {
            o.push(format!(
                "<li>{} の決定 {} を{}: {}</li>",
                id_link(dir, ctx, &e.f("target")?)?,
                e.ef("decision")?,
                e.f("kind")?.lookup(REVISE, "改訂の向き")?,
                e.ef("summary")?
            ));
        }
        o.push("</ul>".to_string());
    }
    if !revised.is_empty() {
        o.push(format!("<h3>{REVISED_BY_LABEL}</h3>"));
        o.push("<ul>".to_string());
        for r in revised {
            o.push(format!(
                "<li>{} がこの判断の決定 {} を{}: {}</li>",
                link_text(&resolve(dir, ctx, &r.by)?, &r.by),
                r.decision,
                r.kind,
                r.summary
            ));
        }
        o.push("</ul>".to_string());
    }
    if let Some(cs) = a.g("consequences")? {
        o.push("<h3>この判断で変わること</h3>".to_string());
        o.push("<ul>".to_string());
        for c in cs.seq()? {
            o.push(format!("<li>{}</li>", strong(&c.e()?)));
        }
        o.push("</ul>".to_string());
    }
    if let Some(g) = a.g("grill")? {
        o.push("<h3>反対側からの確認</h3>".to_string());
        o.push(format!(
            "<p>{}・{}・{}</p>",
            g.ef("when")?,
            g.ef("who")?,
            g.ef("where")?
        ));
        o.push(format!("<p>{}</p>", g.ef("summary")?));
    }
    for (key, label) in [
        ("supersedes", "置き換えた判断"),
        ("superseded_by", "後継の判断"),
    ] {
        if let Some(x) = a.g(key)? {
            o.push(format!("<p>{label}: {}</p>", id_link(dir, ctx, &x)?));
        }
    }
    if let Some(note) = a.g("note")? {
        o.push(format!(
            "<details class=\"note\"><summary>注</summary><div><p>{}</p></div></details>",
            strong(&note.e()?)
        ));
    }
    o.push("</div>".to_string());
    Ok(())
}

/// 章 06（図・便 33）。図ごとに共有の図の枠（`face::figure_panel`・便 34）を置き、図の本体は `face::figure_body` が
/// 図の道具で描いたものをそのまま埋める（escape しない・道具の出力は変えない）。
/// figcaption の根拠は refs の各 id を `id_link` で（無いか空なら「根拠:」以降を出さない）。
/// 図が 1 枚でも導出できなければ Err（面全体が「まだ分からない」・前の面は残る）。
fn figures_chapter(
    o: &mut Vec<String>,
    f: &Frame,
    n: usize,
    figs: &[X<'_>],
    dir: &Path,
    ctx: &Ctx,
) -> R<()> {
    f.band(
        o,
        n,
        FIGURES_CHAPTER,
        &format!("{FIGURES_CHAPTER} {} 枚", figs.len()),
        None,
    );
    o.push("<div class=\"chapbody\">".to_string());
    for (i, fig) in figs.iter().enumerate() {
        let drawn = face::figure_body(dir, fig)?;
        let caption = fig.ef("caption")?;
        let refs = match fig.g("refs")? {
            Some(rs) => rs
                .seq()?
                .iter()
                .map(|q| id_link(dir, ctx, q))
                .collect::<R<Vec<_>>>()?,
            None => Vec::new(),
        };
        face::figure_panel(o, f, i + 1, &drawn, &caption, &refs);
    }
    o.push("</div>".to_string());
    Ok(())
}

fn approval_chapter(o: &mut Vec<String>, f: &Frame, a: &X<'_>, st: &Status) -> R<()> {
    f.approval_band(o, "承認", st.label);
    o.push("<div class=\"chapbody\">".to_string());
    o.push(format!("<div {}>", f.dc(Component::ApprovalBlock)));
    match a.g("approval")? {
        Some(ap) => o.push(format!(
            "<div class=\"sign\"><span class=\"role\">承認</span><span class=\"who\">{}</span><span class=\"when\">{}</span><span class=\"when\">逐語「{}」</span><span class=\"stamp\">{}</span></div>",
            ap.ef("who")?,
            ap.ef("date")?,
            ap.ef("verbatim")?,
            ap.ef("ruling")?
        )),
        None => o.push("<p>未（提案中・持ち主の逐語と日付が入ると発効）</p>".to_string()),
    }
    o.push("</div>".to_string());
    o.push("</div>".to_string());
    Ok(())
}

/// 脚（`chip` = 用語集への札・doc-locator の行の末尾・便 74）。
fn foot(o: &mut Vec<String>, f: &Frame, a: &X<'_>, id: &str, n: &Counts, chip: &str) -> R<()> {
    let date = a.ef("date")?;
    let basis = a
        .f("basis")?
        .seq()?
        .iter()
        .map(X::e)
        .collect::<R<Vec<_>>>()?
        .join("・");
    let mut dl = format!(
        "<dt>id</dt><dd>{id}</dd><dt>status</dt><dd>{}</dd><dt>date</dt><dd>{date}</dd><dt>basis</dt><dd>{basis}</dd><dt>amends</dt><dd>{}</dd><dt>revises</dt><dd>{}</dd>",
        a.ef("status")?,
        n.amends,
        n.revises
    );
    // 図の数は 1 枚以上のときだけ（図なしの面は便 32 までと byte 不変）
    if n.figures > 0 {
        dl.push_str(&format!("<dt>figures</dt><dd>{}</dd>", n.figures));
    }
    let (dated, stamp) = face::adr_dated(a)?;
    f.foot_aside(o, id, (dated, &stamp), &dl, chip);
    Ok(())
}

#[cfg(test)]
mod face_adr_tests {
    use super::*;
    use crate::yaml::Value;

    /// 凍結の針（P-10.1・便 50 §1 (e) 1）: 判断の記録の面の撤退条件の種類の名札を便 50 の前の表の字面と順で固定する。
    #[test]
    fn face_labels_retreat_kind_of_the_adr_face_is_a_frozen_needle() {
        let rows: Vec<(&str, &str)> = ce::RetreatKind::ALL
            .iter()
            .map(|k| (k.name(), retreat_kind_label(*k)))
            .collect();
        assert_eq!(
            rows,
            [
                ("spike", "小さな試し"),
                ("measure", "数えた値"),
                ("ruling", "持ち主の裁定")
            ]
        );
        let v = Value::Str("guess".into());
        assert_eq!(
            retreat_kind(&X::root(&v, "adr/ADR-1.yaml.retreat.kind")).unwrap_err(),
            "adr/ADR-1.yaml.retreat.kind: 撤退条件の種類 の表に無い値「guess」"
        );
    }

    /// 凍結の針（便 137 §1 (c) 1）: 改訂の向きの表の鍵の並びは床の定数 REVISE_KIND と同じで、表と 2 つの欄の名札の字を固定する。
    #[test]
    fn f137_revise_kind_labels_follow_the_floor_enum() {
        let keys: Vec<&str> = REVISE.iter().map(|(k, _)| *k).collect();
        assert_eq!(keys, crate::floor_adr::REVISE_KIND);
        assert_eq!(REVISE, [("narrow", "狭める"), ("widen", "広げる")]);
        assert_eq!(AMENDS_LABEL, "条文の改訂");
        assert_eq!(REVISES_LABEL, "判断の記録の改訂");
        let v = Value::Str("shrink".into());
        assert_eq!(
            X::root(&v, "adr/ADR-1.yaml.revises[0].kind")
                .lookup(REVISE, "改訂の向き")
                .unwrap_err(),
            "adr/ADR-1.yaml.revises[0].kind: 改訂の向き の表に無い値「shrink」"
        );
    }

    /// 凍結の針（便 148 §1 (c) 1）: 逆向きの名札の字と読む状態（発効だけ）を固定し、名札が正本の 2 つの欄の名札の字を
    /// 含まない（表紙の札と h3 を字で取り違えない）。
    #[test]
    fn f148_revised_by_label_and_status_are_frozen_needles() {
        assert_eq!(REVISED_BY_LABEL, "ほかの判断の記録による改訂");
        assert_eq!(REVISED_BY_STATUS, ["accepted"]);
        assert!(!REVISED_BY_LABEL.contains("条文の改訂"));
        assert!(!REVISED_BY_LABEL.contains("判断の記録の改訂"));
    }

    /// 便 149 §1 (c) 1: 印の字と写す先を固定し、左から順の対だけが strong になる（閉じない・空・1 つの星は生のまま）。
    #[test]
    fn f149_strong_turns_left_to_right_pairs_into_strong() {
        assert_eq!(STRONG_MARK, "**");
        assert_eq!(STRONG_TAG, ("<strong>", "</strong>"));
        // 2 組
        assert_eq!(
            strong("前**甲**中**乙**後"),
            "前<strong>甲</strong>中<strong>乙</strong>後"
        );
        // 閉じない印・1 組の後の閉じない印・中身が空の対
        assert_eq!(strong("前**閉じない"), "前**閉じない");
        assert_eq!(
            strong("**甲**と**閉じない"),
            "<strong>甲</strong>と**閉じない"
        );
        assert_eq!(strong("空の****対"), "空の****対");
        // 入れ子にならず左から順に 2 組
        assert_eq!(
            strong("**外**内**外**"),
            "<strong>外</strong>内<strong>外</strong>"
        );
        // 1 つの星と印の無い字は触らない
        assert_eq!(strong("欄の *_note の説明"), "欄の *_note の説明");
        assert_eq!(strong("印の無い字"), "印の無い字");
        // escape した中身は文字参照のまま
        assert_eq!(
            strong("**&lt;b&gt;A &amp; B&lt;/b&gt;**"),
            "<strong>&lt;b&gt;A &amp; B&lt;/b&gt;</strong>"
        );
    }
}
