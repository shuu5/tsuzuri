//! 入口の面の生成器（便 16・docs/design/delivery-16.md §1 (b)）。見本 `preview/index.html` の骨格（部品・要素の入れ子・
//! class の並び）に、入口の正本 `index.yaml` の案内文を escape して差し込み、数と日付（条の数・要件の数・語彙と rules の
//! 数・判断の記録の本数・各文書の版と生成日・文書が読めるか）は生成のたびに他の正本から数えて出す（ADR-5 決定 (2)）。
//! 生成物の文字列は 正本の値（α）・名札の表（β・`face.rs`）・正本から数えた数（γ）のどれかで、見本の layer-line・
//! 「次の一手」「まだ測っていない」の行は正本に無いので出さない。面に依らない口（head と site-bar・部品の名札・小窓）は
//! `face.rs` を呼ぶ。入口は番号付きの章を持たないので、帯は slim だけ・目次と prevnext は出さない。
//! 節「支度表」（便 20・delivery-20.md §1 (b)）は相談窓口の正本 `intake.yaml` の sheet の節と、在れば支度表
//! `<dir>/<sheet.file>`（`folio intake` の生成物）から出す。支度表が無ければ「まだ無い」の 1 行（導出できないではない）。
//! 支度表の documents の各行の id は `intake.yaml` の targets に解き、with（付録の id）は入口の正本 `index.yaml` の
//! shelf.annexes に解く（便 22・delivery-22.md §1 (a)）——付録は targets（行き先＝棚の文書 4 つと注入）には無く、
//! 棚の付録の行にだけ在るので、便 18 の床（targets の with を annexes に解く）と同じ規則で読む。
//! 棚の「判断の記録」の行は、記録が 1 本以上なら「読める」側（本数・状態ごとの数・各記録の面へのリンク）を出す
//! （便 26・delivery-26.md §1 (a)）——面は 1 本につき 1 枚（`adr-<数>.html`・便 25 の生成器）なので、記録が在れば
//! ページも在る。棚の「設計ノート」の行も同じ形で、設計ノートが 1 本以上なら「読める」側（本数・状態ごとの数・
//! 各設計ノートの面へのリンク）を出す（便 29・delivery-29.md §1 (a)）——面は 1 本につき 1 枚
//! （`note-<文書 id>.html`・便 28 の生成器）。入口の正本 `index.yaml` は改訂しない（ADR-7 帰結・棚の行の読み方だけが変わる）。
//! 読み手（正本を読んで面の文脈を組む側・部品の一覧と枠・読みと数え・支度表の読み）は便 115 で `face_index_read.rs`（層 4）へ降ろした。面の口と HTML を書く側はここに残す。

use std::path::Path;

use crate::catalog::Component;
use crate::cursor::{self, R, X, esc};
use crate::face::{self, INDEX_STATUS, hint, hint_q, stop_anchor};
#[cfg(test)]
use crate::face_index_read::PARTS;
use crate::face_index_read::{
    ADR_STATUS, Ctx, Doc, FRAME, INTAKE, NOTE_DIR, Note, Record, SheetBody, SheetHead, context,
    exact, notes, records, sheet_body, sheet_head,
};
use crate::shelf::{self, SHELF_DOCS, SHELF_LEGEND};
use crate::yaml::Value;

/// 読んだ正本（foot の sources）。支度表は生成物なので載せない（節の中で file 名を出す）。
const SOURCES: [&str; 8] = [
    "index.yaml",
    "constitution.yaml",
    "srs.yaml",
    "vocabulary.yaml",
    "rules.yaml",
    "adr/",
    "design-note/",
    "intake.yaml",
];

/// 数が 0 のときの字（β・shelf-head の「これから増える文書」と status-line の「まだ無い」）。
const NONE: &str = "なし";

/// 節「支度表」の名札の表（β・本便）。数と日付は正本から数えたものだけ（ここには書かない）。
const SHEET_KICKER: &str = "支度表";
const SHEET_DOCUMENTS: &str = "持つ文書";
const SHEET_RECOMMENDED: &str = "推奨で進めた項目";
const SHEET_APPROVAL: &str = "承認";
const SHEET_SOURCE: &str = "正本";
const SHEET_ABSENT: &str = "まだ無い";
const SHEET_NO_DOCUMENTS: &str = "なし";
const SHEET_NO_RECOMMENDED: &str = "なし（全部に答えた）";
const SHEET_NO_APPROVAL: &str = "まだ（対話面で承認したら台帳と承認欄に記帳する）";
/// 承認が在るときの行の字（便 66・天井の 11 周目 F-7）。逐語は行に出さず折りたたみの中だけに置く。
const SHEET_APPROVED: &str = "承認済み";
const SHEET_VERBATIM: &str = "言葉どおりの記録";
const SHEET_MADE_BY: &str = "（folio intake の生成物・手で直さない）";

/// 判断の記録の card の折りたたみの名札（β・便 139）。本数は数えたものだけ（ここには書かない）。
const ADR_TITLES: &str = "番号と見出しの一覧";

fn dc(c: Component) -> String {
    FRAME.dc(c)
}

/// 読める面の数（入口 1 + 面が在る文書 + 判断の記録の本数 + 設計ノートの本数・どちらも 1 本につき面 1 枚）。
fn readable_faces(adr: usize, notes: usize) -> usize {
    1 + SHELF_DOCS.iter().filter(|(_, s)| s.face.is_some()).count() + adr + notes
}

/// 正本 → 入口の面の HTML（決定的）。天井の名札は印から読む（便 40・便 83）。
pub fn derive(dir: &Path) -> R<String> {
    let i_doc = cursor::load(dir, "index.yaml")?;
    let c_doc = cursor::load(dir, "constitution.yaml")?;
    let s_doc = cursor::load(dir, "srs.yaml")?;
    let v_doc = cursor::load(dir, "vocabulary.yaml")?;
    let r_doc = cursor::load(dir, "rules.yaml")?;
    let n_doc = cursor::load(dir, INTAKE)?;
    let adr = records(dir)?;
    let notes = notes(dir)?;
    let i = X::root(&i_doc, "index.yaml");
    let c = X::root(&c_doc, "constitution.yaml");
    let s = X::root(&s_doc, "srs.yaml");
    let v = X::root(&v_doc, "vocabulary.yaml");
    let r = X::root(&r_doc, "rules.yaml");
    let n = X::root(&n_doc, INTAKE);

    let ctx = context(&i, &c, &s, &v, &r, adr, notes)?;
    let sheet = sheet_head(&n)?;
    let filled = sheet_body(dir, &n, &sheet, &ctx.annex_types)?;
    let m = i.f("meta")?;
    let stamp = face::ceiling_stamp(dir)?;

    let mut o: Vec<String> = Vec::new();
    head(&mut o, &m, &stamp)?;
    cover(&mut o, &i, &c)?;
    shelf(&mut o, &ctx, &i, &m)?;
    status_line(&mut o, &ctx);
    intake_line(&mut o, &i)?;
    lanes(&mut o, &ctx, &i)?;
    intake(&mut o, &i)?;
    sheet_section(&mut o, &i, &sheet, filled.as_ref())?;
    foot(&mut o, &m)?;
    Ok(format!("{}\n", o.join("\n")))
}

// ── 骨格 ──

fn head(o: &mut Vec<String>, m: &X<'_>, stamp: &str) -> R<()> {
    let version = m.ef("version")?;
    let status = m.f("status")?.lookup(INDEX_STATUS, "入口の状態")?;
    m.f("id")?.text()?;
    FRAME.head(
        o,
        &format!("folio2 — 設計文書の入口（{version}）"),
        &m.ef("generated")?,
        &version,
        status,
        stamp,
    );
    // 入口は番号付きの章を持たないので、site-bar の here は「文書の一覧」
    if let Some(here) = o
        .iter_mut()
        .find(|l| l.starts_with("<span class=\"here\">"))
    {
        *here = format!(
            "<span class=\"here\"><b>{}</b> ▸ 文書の一覧</span>",
            FRAME.name
        );
    }
    Ok(())
}

/// hub-cover。meta の title と憲法の north_star の statement はここでだけ読む（検査もここ 1 か所）。
fn cover(o: &mut Vec<String>, i: &X<'_>, c: &X<'_>) -> R<()> {
    let title = i.f("meta")?.ef("title")?;
    let statement = c.f("north_star")?.ef("statement")?;
    let au = i.f("audience")?;
    let text = au.ef("text")?;
    o.push(format!("<header {}>", dc(Component::HubCover)));
    o.push("<p class=\"cover-eyebrow\"><span class=\"doc-type\">入口 (index)</span> <span>folio2 — 設計文書（design-intent）</span></p>".to_string());
    o.push(format!("<h1>folio2 — {title}</h1>"));
    o.push(format!(
        "<p class=\"north-star\">目指すこと: <q>{statement}</q> — <a href=\"constitution.html#s0\">憲法 §0 で読む</a></p>"
    ));
    o.push(format!(
        "<div class=\"summary-card\"><span class=\"ic\">誰</span><div><p class=\"lab\">{}</p><p class=\"txt\">{text}</p></div></div>",
        au.ef("label")?
    ));
    o.push(format!(
        "<p class=\"hub-who\">誰のため: {} {}</p>",
        au.ef("short")?,
        hint_q(&text)
    ));
    o.push("</header>".to_string());
    Ok(())
}

fn types<'d>(docs: impl Iterator<Item = &'d Doc>) -> String {
    docs.map(|d| d.ty.as_str()).collect::<Vec<_>>().join("・")
}

fn shelf(o: &mut Vec<String>, ctx: &Ctx, i: &X<'_>, m: &X<'_>) -> R<()> {
    let sh = i.f("shelf")?;
    let mut legend = String::from("<div class=\"fig-legend\">");
    // 凡例も表の id と過不足なく（便 76・並びは表が決める）
    for (_, sw, row) in exact(&sh.f("legend")?, SHELF_LEGEND, "凡例")? {
        legend.push_str(&format!(
            "<span class=\"lg\"><span class=\"{sw}\"></span>{}</span>",
            row.ef("text")?
        ));
    }
    legend.push_str("</div>");
    let explain = format!("<p>{}</p>", sh.ef("explain")?);

    o.push(format!(
        "<figure {} data-role=\"diagram\" id=\"docset\">",
        dc(Component::FigurePanel)
    ));
    o.push(format!(
        "<div class=\"fig-title\"><span class=\"fn\">棚</span>{} ── いま読めるのは <b>{} 面</b> ＝ このページ（入口）と、{} <span class=\"fig-tools\">{}{}<button class=\"zoom-btn\" type=\"button\">拡大</button></span><button class=\"zoom-close\" type=\"button\">✕ 閉じる</button></div>",
        sh.ef("title")?,
        readable_faces(ctx.adr.len(), ctx.notes.len()),
        ctx.readable_types(false),
        hint("凡例", &legend),
        hint("図の説明", &explain)
    ));
    // これから増える文書が 1 つも無ければ数の代わりに「なし」（括弧は出さない）
    let coming = if ctx.missing().next().is_none() {
        NONE.to_string()
    } else {
        format!("{}（{}）", ctx.missing().count(), types(ctx.missing()))
    };
    o.push(format!(
        "<div class=\"shelf-head\"><span class=\"shelf-here\">▣ 入口 = いま見ているページ</span><span class=\"sub\">これから増える文書 {coming} ／ 付録 {}（憲法の中）</span></div>",
        ctx.annexes.len()
    ));
    let mut minimap =
        String::from("<p class=\"shelf-minimap\"><span class=\"state ok\">入口（いまここ）</span>");
    for d in &ctx.docs {
        let class = if d.readable.is_some() || ctx.pages(d.id) > 0 {
            "state ok"
        } else {
            "state"
        };
        minimap.push_str(&format!(
            "<span>{}</span><a class=\"{class}\" href=\"#doc-{}\">{}</a>",
            d.shelf.sep, d.id, d.ty
        ));
    }
    minimap.push_str(&format!(
        "<span>｜</span><a class=\"state\" href=\"constitution.html#s7\">付録 {}</a></p>",
        ctx.annexes.len()
    ));
    o.push(minimap);

    let shelf_n = ctx.readable().count() + 1;
    o.push(format!(
        "<div {} class=\"shelf-grid\" style=\"--shelf-n:{shelf_n}\">",
        dc(Component::DocShelf)
    ));
    // 棚の置き場の順: shelf-c・l1・shelf-s・l2・shelf-d・branch・adr
    for (n, d) in ctx.docs.iter().enumerate() {
        if d.id == "adr" {
            o.push("<div class=\"shelf-adr\">".to_string());
            let up = &ctx.relations[3];
            o.push(format!(
                "<span {} class=\"{}\"><span class=\"arrow\"></span><span class=\"lbl\">{} → {} {}</span></span>",
                dc(Component::ShelfLink),
                up.class,
                up.from_ty,
                up.label,
                hint_q(&up.hint)
            ));
            // 棚の置き場（div の shelf-adr）は grid の外なので、card 自身の class だけが「読める」側で変わる
            let class = if ctx.adr.is_empty() {
                "is-absent"
            } else {
                d.shelf.place
            };
            shelf_card(o, ctx, d, class);
            o.push("</div>".to_string());
        } else {
            let class = if d.readable.is_some() || ctx.pages(d.id) > 0 {
                d.shelf.place.to_string()
            } else {
                format!("{} is-absent", d.shelf.place)
            };
            shelf_card(o, ctx, d, &class);
            shelf_link(o, ctx, n);
        }
    }
    o.push("</div>".to_string());
    o.push(format!(
        "<figcaption><span class=\"ver\">棚 · {} {} · index.yaml</span></figcaption>",
        m.ef("version")?,
        m.ef("generated")?
    ));
    o.push("</figure>".to_string());
    Ok(())
}

/// 判断の記録の card の 2 行（記録が 1 本以上のとき）。1 行目は本数と要約・2 行目は更新と各記録の面へのリンク。
fn adr_rows(o: &mut Vec<String>, adr: &[Record]) {
    let kinds = ADR_STATUS
        .iter()
        .filter_map(|(_, label)| {
            let n = adr.iter().filter(|r| r.status == *label).count();
            (n > 0).then(|| format!("{label} {n}"))
        })
        .collect::<Vec<_>>()
        .join("・");
    let first = &adr[0];
    let last = &adr[adr.len() - 1];
    o.push(format!(
        "<p class=\"sc-row\"><span class=\"state ok\">● {} 本</span><span>{}〜{}（{kinds}）</span></p>",
        adr.len(),
        first.id,
        last.id
    ));
    let links = adr
        .iter()
        .map(|r| format!("<a class=\"xref\" href=\"{}\">{}</a>", r.file(), r.id))
        .collect::<Vec<_>>()
        .join("・");
    // 「開く →」は最も新しい番号の記録の面へ。当たり判定は置かない（効けば下の一覧を覆う・便 139）
    let file = last.file();
    let updated = adr
        .iter()
        .map(|r| r.date.as_str())
        .max()
        .unwrap_or_default();
    o.push(format!(
        "<p class=\"sc-row\"><span class=\"up\">更新 {updated}</span>{links}<a class=\"sc-open\" href=\"{file}\">開く →</a></p>"
    ));
    // 番号と見出しの一覧（畳んだまま・1 本 1 行・id の数の昇順・見出しは正本の title の逐語で切らない）
    o.push(format!(
        "<details class=\"note\"><summary>{ADR_TITLES}（{} 本）</summary><div><ul class=\"basis\">",
        adr.len()
    ));
    for r in adr {
        o.push(format!(
            "<li><a class=\"xref\" href=\"{}\">{}</a><span>{}</span></li>",
            r.file(),
            r.id,
            r.title
        ));
    }
    o.push("</ul></div></details>".to_string());
}

/// 設計ノートの card の 2 行（設計ノートが 1 本以上のとき）。1 行目は本数と状態ごとの数・2 行目は更新と
/// 各設計ノートの面へのリンク（id は番号でないので範囲は出さない）。
fn note_rows(o: &mut Vec<String>, notes: &[Note]) {
    let kinds = shelf::STATUS
        .iter()
        .filter_map(|(_, label)| {
            let n = notes.iter().filter(|q| q.status == *label).count();
            (n > 0).then(|| format!("{label} {n}"))
        })
        .collect::<Vec<_>>()
        .join("・");
    o.push(format!(
        "<p class=\"sc-row\"><span class=\"state ok\">● {} 本</span><span>{kinds}</span></p>",
        notes.len()
    ));
    let links = notes
        .iter()
        .map(|q| format!("<a class=\"xref\" href=\"{}\">{}</a>", q.file(), q.id))
        .collect::<Vec<_>>()
        .join("・");
    // 「開く →」と当たり判定は生成日が最も新しい 1 本へ（同じ日付が 2 本以上なら id の順で後の 1 本）
    let updated = notes
        .iter()
        .map(|q| q.generated.as_str())
        .max()
        .unwrap_or_default();
    let last = notes
        .iter()
        .rfind(|q| q.generated == updated)
        .map(Note::file)
        .unwrap_or_default();
    o.push(format!(
        "<p class=\"sc-row\"><span class=\"up\">更新 {updated}</span>{links}<a class=\"sc-open\" href=\"{last}\">開く →</a><a class=\"sc-hit\" href=\"{last}\" aria-hidden=\"true\" tabindex=\"-1\"></a></p>"
    ));
}

fn shelf_card(o: &mut Vec<String>, ctx: &Ctx, d: &Doc, class: &str) {
    o.push(format!(
        "<article {} class=\"{class}\" id=\"doc-{}\" data-doc-type=\"{}\">",
        dc(Component::ShelfCard),
        d.id,
        d.id
    ));
    o.push(format!(
        "<p class=\"sc-type\">{}<span class=\"sub\">{}</span></p>",
        d.ty, d.shelf.en
    ));
    o.push(format!("<p class=\"sc-use\">{}</p>", d.use_));
    match (&d.readable, d.shelf.face) {
        (Some(rd), Some(file)) => {
            o.push(format!(
                "<p class=\"sc-row\"><span class=\"state ok\">● 揃っている</span><span>{}</span></p>",
                rd.summary
            ));
            o.push(format!(
                "<p class=\"sc-row\"><span class=\"up\">更新 {}</span><a class=\"sc-open\" href=\"{file}\">開く →</a><a class=\"sc-hit\" href=\"{file}\" aria-hidden=\"true\" tabindex=\"-1\"></a></p>",
                rd.updated
            ));
        }
        _ if d.id == "adr" && !ctx.adr.is_empty() => adr_rows(o, &ctx.adr),
        _ if d.id == NOTE_DIR && !ctx.notes.is_empty() => note_rows(o, &ctx.notes),
        _ => {
            let state = if d.id == "adr" {
                format!("○ {} 本", ctx.adr.len())
            } else {
                "○ まだ無い".to_string()
            };
            let text = d.absent.clone().unwrap_or_default();
            o.push(format!(
                "<p class=\"sc-row\"><span class=\"state\">{state}</span><span>{text}</span></p>"
            ));
            o.push("<p class=\"sc-none\">—（まだページはありません）</p>".to_string());
        }
    }
    o.push("</article>".to_string());
}

/// n 番目の文書の後の関係（shelf-c の後 = binds・shelf-s の後 = before-build・shelf-d の後 = inside）。
fn shelf_link(o: &mut Vec<String>, ctx: &Ctx, n: usize) {
    let rel = &ctx.relations[n];
    let chips = if n == 2 {
        let chips = ctx
            .annexes
            .iter()
            .map(|a| {
                format!(
                    "<a href=\"constitution.html#s{c}\">付録 {} <span class=\"cnt\">{} {} → 憲法 §{c}</span></a>",
                    a.ty,
                    a.count,
                    a.unit,
                    c = a.chapter
                )
            })
            .collect::<String>();
        format!("<span class=\"annex-chips\">{chips}</span>")
    } else {
        String::new()
    };
    o.push(format!(
        "<span {} class=\"{}\"><span class=\"arrow\"></span><span class=\"lbl\">{} {}</span>{chips}</span>",
        dc(Component::ShelfLink),
        rel.class,
        rel.label,
        hint_q(&rel.hint)
    ));
}

fn status_line(o: &mut Vec<String>, ctx: &Ctx) {
    let annex_types = ctx
        .annexes
        .iter()
        .map(|a| a.ty.as_str())
        .collect::<Vec<_>>()
        .join("・");
    let missing = ctx
        .missing()
        .map(|d| format!("{}（{}）", d.ty, d.absent.clone().unwrap_or_default()))
        .collect::<Vec<_>>()
        .join("／");
    // まだ無い文書が 1 つも無くても行は残す（部品の数を変えない）
    let missing = if missing.is_empty() {
        NONE.to_string()
    } else {
        missing
    };
    o.push(format!(
        "<section {} aria-label=\"いまの状態\">",
        dc(Component::StatusLine)
    ));
    st(
        o,
        " ok",
        "●",
        "揃っている",
        &format!(
            "{}が読める（付録の{annex_types}は憲法の中）",
            ctx.readable_types(true)
        ),
    );
    st(o, "", "○", "まだ無い", &missing);
    o.push("</section>".to_string());
}

/// status-line の 1 行（`class` は "" か " ok"・字面はすべて escape 済み）。
fn st(o: &mut Vec<String>, class: &str, mark: &str, k: &str, v: &str) {
    o.push(format!(
        "<p class=\"st{class}\"><span class=\"mark\">{mark}</span><span class=\"k\">{k}</span><span class=\"v\">{v}</span></p>"
    ));
}

fn steps(it: &X<'_>) -> R<Vec<String>> {
    it.f("steps")?
        .seq()?
        .iter()
        .map(|x| match x.v {
            Value::Str(_) => x.e(),
            _ => Err(format!("{}: 文字列でない", x.at)),
        })
        .collect()
}

fn intake_line(o: &mut Vec<String>, i: &X<'_>) -> R<()> {
    let it = i.f("intake")?;
    o.push(format!(
        "<p {}><span class=\"k\">相談を始める:</span><span class=\"cmd\">{}</span><span>（と AI に頼む）→ {}</span><a class=\"more\" href=\"#s2\">くわしく →</a></p>",
        dc(Component::IntakeLine),
        it.ef("command")?,
        steps(&it)?.join(" → ")
    ));
    Ok(())
}

/// slim の帯（読む順番・相談窓口・支度表）。字面はすべて escape 済み。
fn band(o: &mut Vec<String>, n: usize, class: &str, kicker: &str, title: &str, lead: &str) {
    o.push(format!(
        "<section id=\"s{n}\" {} class=\"{class} slim\">",
        dc(Component::ChapterDeckBand)
    ));
    o.push(format!("<span class=\"kicker\">{kicker}</span>"));
    o.push(format!("<h2>{title}</h2>"));
    o.push(format!("<p class=\"lead\">{lead}</p>"));
    o.push("</section>".to_string());
}

/// 欄 title と lead を持つ節の slim の帯。
fn slim_band(o: &mut Vec<String>, n: usize, class: &str, kicker: &str, x: &X<'_>) -> R<()> {
    band(o, n, class, kicker, &x.ef("title")?, &x.ef("lead")?);
    Ok(())
}

/// 読む順番の行き先の href（便 66）。判断の記録は面が 1 本 1 枚なのでその面の file へ、ほかの文書は
/// 面の節の anchor へ。`at` は `stop_anchor` を通った後の字（判断の記録なら `ADR-<数>`）。
fn stop_href(ctx: &Ctx, doc: &Doc, doc_x: &X<'_>, at_x: &X<'_>, at: &str) -> R<String> {
    if doc.id == "adr" {
        let record = ctx
            .adr
            .iter()
            .find(|r| r.id == at)
            .ok_or_else(|| format!("{}: 判断の記録「{at}」が adr/ に無い", at_x.at))?;
        return Ok(record.file());
    }
    let Some(file) = doc.shelf.face else {
        return Err(format!("{}: 文書「{}」は面が無い", doc_x.at, doc.id));
    };
    Ok(format!("{file}#{at}"))
}

fn lanes(o: &mut Vec<String>, ctx: &Ctx, i: &X<'_>) -> R<()> {
    let la = i.f("lanes")?;
    slim_band(o, 1, "band-5", "読む順番", &la)?;
    let rows = la.f("rows")?.seq()?;
    o.push("<div class=\"chapbody\">".to_string());
    o.push(format!(
        "<div class=\"lane-grid\" style=\"--band-n:{}\">",
        rows.len()
    ));
    for row in &rows {
        row.f("id")?.text()?;
        let mut stops = String::new();
        for st in row.f("stops")?.seq()? {
            let doc_x = st.f("doc")?;
            let doc = ctx.doc(&doc_x)?;
            let at_x = st.f("at")?;
            let at = at_x.text()?;
            stop_anchor(doc.id, &at).map_err(|e| format!("{}: {e}", at_x.at))?;
            let href = stop_href(ctx, doc, &doc_x, &at_x, &at)?;
            stops.push_str(&format!(
                "<li><a href=\"{href}\">{} {}</a></li>",
                doc.ty,
                st.ef("label")?
            ));
        }
        o.push(format!("<article {}>", dc(Component::ReaderLane)));
        o.push(format!(
            "<p class=\"who\"><span class=\"av\">{}</span>{}</p>",
            row.ef("mark")?,
            row.ef("who")?
        ));
        o.push(format!("<p class=\"why\">{}</p>", row.ef("why")?));
        o.push(format!("<ol>{stops}</ol>"));
        o.push(format!(
            "<span class=\"time\">約 {} 分</span>",
            row.f("minutes")?.count()?
        ));
        o.push("</article>".to_string());
    }
    o.push("</div>".to_string());
    o.push("</div>".to_string());
    Ok(())
}

fn intake(o: &mut Vec<String>, i: &X<'_>) -> R<()> {
    let it = i.f("intake")?;
    slim_band(o, 2, "band-3", "相談窓口", &it)?;
    o.push("<div class=\"chapbody\">".to_string());
    o.push(format!("<div {}>", dc(Component::IntakeCallout)));
    o.push("<div>".to_string());
    o.push(format!("<h3>{}</h3>", it.ef("heading")?));
    o.push(format!("<p>{}</p>", it.ef("text")?));
    let steps = steps(&it)?
        .iter()
        .enumerate()
        .map(|(n, s)| format!("<span>{} · {s}</span>", n + 1))
        .collect::<String>();
    o.push(format!("<div class=\"steps\">{steps}</div>"));
    o.push("</div>".to_string());
    o.push(format!("<div class=\"cmd\">{}</div>", it.ef("command")?));
    o.push("</div>".to_string());
    if let Some(note) = it.g("note")? {
        o.push(format!(
            "<details class=\"note\"><summary>補足</summary><div><p>{}</p></div></details>",
            note.e()?
        ));
    }
    o.push("</div>".to_string());
    Ok(())
}

/// 節「支度表」（章 02 の直後・foot の前）。相談窓口の結果が在ればその行を、無ければ「まだ無い」の 1 行を出す。
fn sheet_section(
    o: &mut Vec<String>,
    i: &X<'_>,
    head: &SheetHead,
    body: Option<&SheetBody>,
) -> R<()> {
    band(
        o,
        3,
        "band-4",
        SHEET_KICKER,
        &esc(&head.title),
        &esc(&head.explain),
    );
    o.push("<div class=\"chapbody\">".to_string());
    o.push(format!(
        "<section {} aria-label=\"{SHEET_KICKER}\">",
        dc(Component::StatusLine)
    ));
    match body {
        Some(b) => {
            let documents = if b.documents.is_empty() {
                SHEET_NO_DOCUMENTS.to_string()
            } else {
                b.documents.join("・")
            };
            let recommended = if b.recommended.is_empty() {
                SHEET_NO_RECOMMENDED.to_string()
            } else {
                b.recommended.join("／")
            };
            st(o, " ok", "●", SHEET_DOCUMENTS, &documents);
            st(o, "", "○", SHEET_RECOMMENDED, &recommended);
            match &b.approval {
                Some((when, verbatim)) => {
                    st(
                        o,
                        "",
                        "○",
                        SHEET_APPROVAL,
                        &format!("{SHEET_APPROVED}（{when}）"),
                    );
                    // 逐語は折りたたみの中だけに出す（P-12.2・記録は正本の側）。details は段落を閉じる
                    // 要素なので行（p.st）の中には入れず、その行の隣に置く。
                    o.push(format!(
                        "<details class=\"note\"><summary>{SHEET_VERBATIM}</summary><div><p>{verbatim}</p></div></details>"
                    ));
                }
                None => st(o, "", "○", SHEET_APPROVAL, SHEET_NO_APPROVAL),
            }
            st(
                o,
                "",
                "·",
                SHEET_SOURCE,
                &format!("{}{SHEET_MADE_BY}", esc(&head.file)),
            );
        }
        None => st(
            o,
            "",
            "○",
            SHEET_ABSENT,
            &format!(
                "支度表はまだ無い。「{}」と AI に頼むと作られる",
                i.f("intake")?.ef("command")?
            ),
        ),
    }
    o.push("</section>".to_string());
    o.push("</div>".to_string());
    Ok(())
}

fn foot(o: &mut Vec<String>, m: &X<'_>) -> R<()> {
    let version = m.ef("version")?;
    o.push("<footer class=\"foot\">".to_string());
    o.push(format!(
        "<p class=\"ft-plain\">このページは正本 {} と他の正本から folio が生成した · {} {version}（{}）· 手で直さない</p>",
        FRAME.source,
        FRAME.name,
        m.ef("generated")?
    ));
    o.push(format!(
        "<details class=\"machine\" data-audience=\"machine\"><summary>機械のための面</summary><dl><dt>id</dt><dd>{}</dd><dt>version</dt><dd>{version}</dd><dt>status</dt><dd>{}</dd><dt>sources</dt><dd>{}</dd></dl></details>",
        m.ef("id")?,
        m.ef("status")?,
        SOURCES.join(" / ")
    ));
    o.push("</footer>".to_string());
    o.push("</main>".to_string());
    o.push(format!(
        "<p class=\"doc-locator\">この文書の所属: 設計文書（design-intent）/ {} — <a href=\"constitution.html\">憲法へ</a> · <a href=\"srs.html\">要件書へ</a></p>",
        FRAME.name
    ));
    o.push("</body>".to_string());
    o.push("</html>".to_string());
    Ok(())
}

#[cfg(test)]
mod face_index_tests {
    use super::*;
    use crate::yaml;

    #[test]
    fn face_index_document_table_has_four_ids_and_rejects_others() {
        let keys: Vec<&str> = SHELF_DOCS.iter().map(|(k, _)| *k).collect();
        assert_eq!(keys, ["constitution", "srs", "design-note", "adr"]);
        let ok = yaml::parse_typed(
            "- {id: adr}\n- {id: srs}\n- {id: design-note}\n- {id: constitution}\n",
        )
        .unwrap();
        let found = exact(&X::root(&ok, "d"), SHELF_DOCS, "文書").unwrap();
        let order: Vec<&str> = found.iter().map(|(k, _, _)| *k).collect();
        assert_eq!(order, keys, "表の順に並べ直す");
        for bad in [
            "- {id: constitution}\n- {id: srs}\n- {id: design-note}\n",
            "- {id: constitution}\n- {id: srs}\n- {id: design-note}\n- {id: adr}\n- {id: memo}\n",
            "- {id: constitution}\n- {id: srs}\n- {id: design-note}\n- {id: srs}\n",
        ] {
            let v = yaml::parse_typed(bad).unwrap();
            assert!(
                exact(&X::root(&v, "d"), SHELF_DOCS, "文書").is_err(),
                "{bad}"
            );
        }
    }

    #[test]
    fn face_index_stop_anchor_resolves_only_the_face_anchors() {
        for at in ["s0", "s1", "s8"] {
            assert!(stop_anchor("constitution", at).is_ok(), "{at}");
        }
        for at in ["s1", "s8", "fig-context", "fig-rail", "fig-verdicts"] {
            assert!(stop_anchor("srs", at).is_ok(), "{at}");
        }
        // 判断の記録の行き先は記録の id（便 66）
        for at in ["ADR-1", "ADR-2", "ADR-11"] {
            assert!(stop_anchor("adr", at).is_ok(), "{at}");
        }
        for (doc, at) in [
            ("constitution", "s9"),
            ("constitution", "fig-rail"),
            ("srs", "s0"),
            ("srs", "s9"),
            ("srs", "fig-other"),
            ("srs", "s10"),
            ("design-note", "s1"),
            ("constitution", ""),
            ("adr", "s1"),
            ("adr", ""),
            ("adr", "ADR-"),
            ("adr", "ADR-0"),
            ("adr", "ADR-1a"),
            ("adr", "adr-1"),
            ("constitution", "ADR-1"),
        ] {
            assert!(stop_anchor(doc, at).is_err(), "{doc}#{at}");
        }
    }

    #[test]
    fn face_index_counts_readable_faces() {
        assert_eq!(readable_faces(0, 0), 3);
        assert_eq!(readable_faces(7, 1), 11);
    }

    #[test]
    fn face_parts_are_all_allowed_on_the_index_face() {
        for part in PARTS {
            assert!(
                part.faces().contains(&"index"),
                "{} は入口の面に置けない",
                part.name()
            );
        }
        let mut names: Vec<&str> = PARTS.iter().map(|p| p.name()).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), 13);
    }
}
