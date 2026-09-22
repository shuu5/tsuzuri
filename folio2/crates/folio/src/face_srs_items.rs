//! 要件書の面の章 03〜06（機能要件・非機能要件・受入基準・制約）の生成（便 100・docs/design/delivery-100.md §1 (b)）。
//! 便 100 で `face_srs.rs` から 1 字も変えずに移した。文脈（`Ctx`）と共有の口（`band`・`figure_open`・`figure_close`・
//! `fig_req`・`article_link`・`xref`・`slots`）は `face_srs.rs` のもの。

use crate::constitution_enums as ce;
use crate::face::{
    METHOD, R, TONE, X, anchor, card, hint, hint_q, method_label, pattern_label, strength_label,
    strength_meaning, strength_prio,
};
use crate::face_srs::{
    Ctx, Item, article_link, band, fig_req, figure_close, figure_open, slots, xref,
};
use crate::parts::catalog::Component;

/// 凡例の 1 行（言葉 = 強度と型の名札・確かめ方 = 確かめ方の名札）。強度と型は導出した型の ALL の順（= 憲法の値域の file の順）。
fn legend_line() -> String {
    let words = ce::Strength::ALL
        .iter()
        .map(|s| format!("{} = {}", strength_label(*s), strength_meaning(*s)))
        .chain(
            ce::Pattern::ALL
                .iter()
                .map(|p| pattern_label(*p).to_string()),
        )
        .collect::<Vec<_>>()
        .join("／");
    let methods = METHOD
        .iter()
        .map(|(_, label)| *label)
        .collect::<Vec<_>>()
        .join("／");
    format!(
        "<div class=\"legend-line\"><span>凡例:</span>{}{}</div>",
        hint("言葉", &words),
        hint("確かめ方", &methods)
    )
}

pub(crate) fn fr_chapter(o: &mut Vec<String>, ctx: &Ctx<'_>, m: &X<'_>) -> R<()> {
    band(o, ctx, 3, None);
    o.push("<div class=\"chapbody\">".to_string());
    o.push(legend_line());

    // 図 2
    let count = ctx.rail.len();
    let legend = "<div class=\"fig-legend\"><span class=\"lg\"><span class=\"sw ok\"></span>あなたの手番</span><span class=\"lg\"><span class=\"sw neutral\"></span>folio がやる</span><span class=\"lg\"><span class=\"sw line\"></span>番号の順に進む</span><span class=\"lg\">枠線が点線 = 定める要件が無い段</span></div>";
    figure_open(
        o,
        ctx,
        "fig-rail",
        "図 2",
        &format!("folio が 1 回で通す {count} 段 — 各段を決めている要件"),
        Some(legend),
    );
    o.push(format!(
        "<ol {} style=\"--rail-n:{count}\">",
        ctx.frame.dc(Component::PipelineRail)
    ));
    for st in &ctx.rail {
        let x = &st.x;
        let nt = match x.g("note")? {
            Some(n) => format!("{} {}", st.what, hint_q(&n.e()?)),
            None => st.what.clone(),
        };
        let basis = match x.g("basis")? {
            Some(b) => b
                .seq()?
                .iter()
                .map(|q| {
                    let (id, _) = ctx.article(q)?;
                    Ok(article_link(id, &format!("憲法 {id}")))
                })
                .collect::<R<Vec<_>>>()?,
            None => Vec::new(),
        };
        let reqs = x
            .f("reqs")?
            .seq()?
            .iter()
            .map(|q| Ok(fig_req(ctx.req(q)?)))
            .collect::<R<Vec<_>>>()?;
        let fig_reqs = if reqs.is_empty() {
            let refs = if basis.is_empty() {
                String::new()
            } else {
                format!("（{}）", basis.join("・"))
            };
            format!("<span class=\"fig-req none\">この段を定める要件は無い{refs}</span>")
        } else {
            reqs.concat()
        };
        let node = format!(
            "<article {}{} id=\"rail-{}\"><span class=\"actor\">担当: {}</span><p class=\"nt\">{nt}</p><p class=\"fig-reqs\">{fig_reqs}</p></article>",
            ctx.frame.dc(Component::RailNode),
            if st.owner { " class=\"tone-ok\"" } else { "" },
            st.n,
            st.who
        );
        o.push(format!(
            "<li class=\"rail-col\"><span class=\"rail-step\"><span class=\"no\">{}</span></span>",
            st.n
        ));
        let (upper, lower) = slots(st.owner, &node);
        o.push(upper);
        o.push(format!("{lower}</li>"));
    }
    o.push("</ol>".to_string());
    figure_close(o, "図 2", m)?;

    o.push("<div class=\"stack\">".to_string());
    for it in &ctx.fr {
        item_row(o, ctx, it, false)?;
    }
    o.push("</div>".to_string());

    // 図 3（verdicts の節が在るときだけ）
    if let Some(verdicts) = &ctx.verdicts {
        let mut legend = String::new();
        let mut nodes = Vec::new();
        for v in verdicts {
            let tone_x = v.f("tone")?;
            let class = tone_x.lookup(TONE, "図の色")?;
            let tone = tone_x.e()?;
            let name = v.ef("name")?;
            legend.push_str(&format!(
                "<span class=\"lg\"><span class=\"sw {tone}\"></span>{name}</span>"
            ));
            nodes.push(format!(
                "<li {} class=\"{class}\" id=\"v-{}\"><p class=\"nt\">{name}</p><span class=\"cond\">ここへ来る条件</span><p class=\"np\">{}</p></li>",
                ctx.frame.dc(Component::StateNode),
                v.f("id")?.id()?,
                v.ef("cond")?
            ));
        }
        figure_open(
            o,
            ctx,
            "fig-verdicts",
            "図 3",
            &format!(
                "検査の答えは {} つ — どういうときにその答えになるか",
                verdicts.len()
            ),
            Some(&format!("<div class=\"fig-legend\">{legend}</div>")),
        );
        o.push(format!(
            "<ul {} style=\"--state-n:{}\">",
            ctx.frame.dc(Component::StateStrip),
            verdicts.len()
        ));
        o.extend(nodes);
        o.push("</ul>".to_string());
        figure_close(o, "図 3", m)?;
    }
    o.push("</div>".to_string());
    Ok(())
}

pub(crate) fn nfr_chapter(o: &mut Vec<String>, ctx: &Ctx<'_>) -> R<()> {
    band(o, ctx, 4, None);
    o.push("<div class=\"chapbody\">".to_string());
    o.push(legend_line());
    o.push("<div class=\"stack\">".to_string());
    for it in &ctx.nfr {
        item_row(o, ctx, it, true)?;
    }
    o.push("</div>".to_string());
    o.push("</div>".to_string());
    Ok(())
}

fn item_row(o: &mut Vec<String>, ctx: &Ctx<'_>, it: &Item<'_>, nfr: bool) -> R<()> {
    let x = &it.x;
    let pattern = x.f("pattern")?;
    let ears = pattern_label(pattern.parse(ce::Pattern::from_name, "型")?);
    let strength = x.f("strength")?;
    let s = strength.parse(ce::Strength::from_name, "強度")?;
    let (kw, prio) = (strength_label(s), strength_prio(s));
    let milestone = x.g("milestone")?;
    o.push(format!(
        "<article {}{} id=\"{}\">",
        ctx.frame.dc(Component::ItemRow),
        if nfr { " class=\"kind-nfr\"" } else { "" },
        anchor(it.id)
    ));
    let pill = match &milestone {
        Some(ms) => format!("<span class=\"pill\">{}</span>", ms.e()?),
        None => String::new(),
    };
    o.push(format!(
        "<div class=\"ir-head\"><span class=\"rid\">{}</span><h3 class=\"rt\">{}</h3><span class=\"badges\"><span class=\"prio {prio}\">{kw}</span><span class=\"ears\">{ears}</span>{pill}</span></div>",
        it.id, it.title
    ));
    let when = x.ef("when")?;
    let shall = x.ef("shall")?;
    let norm = if pattern.v.as_str() == Some("ubiquitous") {
        shall
    } else {
        format!("<span class=\"ew\">{when}</span> {shall}")
    };
    o.push(format!("<p class=\"norm\">{norm}</p>"));
    o.push(format!(
        "<div class=\"plain\"><span class=\"pk\">やさしく言うと</span>{}</div>",
        x.ef("plain")?
    ));

    // 小窓（確かめ方・根拠・注）と図の参照
    let verify = x.f("verify")?;
    let method = verify.f("method")?;
    let mut how = format!(
        "<span class=\"vbadge\">{}</span> {}",
        method_label(&method)?,
        verify.ef("how")?
    );
    let acs = verify
        .f("ac")?
        .seq()?
        .iter()
        .map(|q| {
            let a = ctx.ac(q)?;
            Ok(xref(a.id, a.id))
        })
        .collect::<R<Vec<_>>>()?;
    if !acs.is_empty() {
        how.push_str(&format!("（{}）", acs.join("・")));
    }
    let mut chips = vec![hint("確かめ方", &how)];
    let mut basis = Vec::new();
    for q in x.f("goals")?.seq()? {
        let g = ctx.goal(&q)?;
        basis.push(xref(g.id, &format!("{}（{}）", g.id, g.title)));
    }
    for q in x.f("basis")?.seq()? {
        let (id, title) = ctx.article(&q)?;
        basis.push(article_link(id, &format!("憲法 {id}（{title}）")));
    }
    if let Some(rules) = x.g("rules")? {
        for q in rules.seq()? {
            let id = ctx.rule(&q)?;
            basis.push(article_link(id, &format!("rules {id}")));
        }
    }
    if !basis.is_empty() {
        chips.push(hint("根拠", &basis.join("・")));
    }
    if let Some(note) = x.g("note")? {
        chips.push(hint("注", &note.e()?));
    }
    for f in x.f("figures")?.seq()? {
        let w = ctx.fig(&f)?;
        chips.push(match w.href {
            Some(h) => format!("<a class=\"rq-where\" href=\"{h}\">{}</a>", w.long),
            None => w.long,
        });
    }
    o.push(format!("<p class=\"meta-chips\">{}</p>", chips.concat()));

    // 機械のための面（正本の値のまま）
    let mut dl = format!(
        "<dt>pattern</dt><dd>{}</dd><dt>strength</dt><dd>{}</dd><dt>when</dt><dd>{when}</dd><dt>verify</dt><dd>{}</dd>",
        pattern.e()?,
        strength.e()?,
        method.e()?
    );
    if let Some(ms) = &milestone {
        dl.push_str(&format!("<dt>milestone</dt><dd>{}</dd>", ms.e()?));
    }
    o.push(format!(
        "<details class=\"machine\" data-audience=\"machine\"><summary>機械のための面</summary><dl>{dl}</dl></details>"
    ));
    o.push("</article>".to_string());
    Ok(())
}

/// 受入基準の章の凡例の 1 行（札「まだ分からない」が何を言うか・便 64）。部品と class は §3 の凡例（`legend_line`）と同じ。
/// folio はまだ受入の結果を測らない（P-4.2）——測るのは便の検証の歯で、その結果はこの面に写していない。
fn ac_legend_line() -> &'static str {
    "<div class=\"legend-line\"><span>凡例:</span><span>まだ分からない = この基準の合否を folio はまだ数えていません（合否を測るのは便の検証の歯で、その結果はこのページに写していません）</span></div>"
}

pub(crate) fn ac_chapter(o: &mut Vec<String>, ctx: &Ctx<'_>, s: &X<'_>) -> R<()> {
    band(o, ctx, 5, None);
    o.push("<div class=\"chapbody\">".to_string());
    o.push(ac_legend_line().to_string());
    o.push(format!(
        "<div {} style=\"--band-n:2\">",
        ctx.frame.dc(Component::SectionLeadCallout)
    ));
    for a in &ctx.acs {
        let x = &a.x;
        // folio はまだ受入の結果を測らない（P-4.2）
        let mut cid = format!(
            "{} <span {}>まだ分からない</span>",
            a.id,
            ctx.frame.dc(Component::AcStateChip)
        );
        if let Some(ms) = x.g("milestone")? {
            cid.push_str(&format!(" <span class=\"pill\">{}</span>", ms.e()?));
        }
        let verifies = x
            .f("verifies")?
            .seq()?
            .iter()
            .map(|q| {
                let r = ctx.req(q)?;
                Ok(xref(r.id, r.id))
            })
            .collect::<R<Vec<_>>>()?;
        let red = x.f("red_test")?;
        let mut cd = format!(
            "{} を確かめる。{}",
            verifies.join("・"),
            hint(
                "RED の歯",
                &format!("{}／固定の材料: {}", red.ef("sentence")?, red.ef("fixture")?)
            )
        );
        if let Some(note) = x.g("note")? {
            cd.push_str(&hint("注", &note.e()?));
        }
        o.push(card(
            "card accent",
            Some(&anchor(a.id)),
            &cid,
            &format!(
                "<p class=\"ct\">{}</p><div class=\"plain\"><span class=\"pk\">やさしく言うと</span>{}</div><p class=\"cd\">{cd}</p>",
                a.title,
                x.ef("plain")?
            ),
        ));
    }
    o.push("</div>".to_string());
    o.push(format!(
        "<div class=\"callout warn\"><span class=\"ck\">凍結しないもの</span><p>{}</p></div>",
        s.ef("not_frozen")?
    ));
    o.push("</div>".to_string());
    Ok(())
}

pub(crate) fn con_chapter(o: &mut Vec<String>, ctx: &Ctx<'_>) -> R<()> {
    band(o, ctx, 6, None);
    o.push("<div class=\"chapbody\">".to_string());
    o.push("<div class=\"tbl-wrap\"><table class=\"tbl\">".to_string());
    o.push(
        "<thead><tr><th>id</th><th>制約</th><th>中身</th><th>根拠</th><th>出所</th></tr></thead>"
            .to_string(),
    );
    o.push("<tbody>".to_string());
    for con in &ctx.cons {
        let x = &con.x;
        let mut refs = Vec::new();
        for q in x.f("basis")?.seq()? {
            let (id, _) = ctx.article(&q)?;
            refs.push(article_link(id, id));
        }
        if let Some(rules) = x.g("rules")? {
            for q in rules.seq()? {
                let id = ctx.rule(&q)?;
                refs.push(article_link(id, id));
            }
        }
        let refs = if refs.is_empty() {
            "—".to_string()
        } else {
            refs.join("・")
        };
        o.push(format!(
            "<tr id=\"{}\"><td class=\"id\" data-k=\"id\">{}</td><td data-k=\"制約\">{}</td><td data-k=\"中身\">{}</td><td data-k=\"根拠\">{refs}</td><td data-k=\"出所\">{}</td></tr>",
            anchor(con.id),
            con.id,
            con.title,
            x.ef("text")?,
            x.ef("source")?
        ));
    }
    o.push("</tbody>".to_string());
    o.push("</table></div>".to_string());
    o.push("</div>".to_string());
    Ok(())
}
