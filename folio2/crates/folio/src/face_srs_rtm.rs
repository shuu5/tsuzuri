//! 要件書の面の章 07・08 の生成（便 35 で `face_srs.rs` から分けた・出力は不変）。章 07 は要件と根拠の対応
//! （rtm-grid）・章 08 は用語集（glossary-links）。文脈（`Ctx`）と共有の口（band・xref・hint 等）は `face_srs.rs` の
//! もので、字面（出力の HTML）は分ける前と 1 byte も変えない。

use crate::face::{R, X, anchor, hint};
use crate::face_srs::{Ctx, band};
use crate::parts::catalog::Component;
use crate::yaml::Value;

pub(crate) fn rtm_chapter(o: &mut Vec<String>, ctx: &Ctx<'_>) -> R<()> {
    band(o, ctx, 7, None);
    o.push("<div class=\"chapbody\">".to_string());
    o.push(format!(
        "<div class=\"legend-line\"><span>凡例:</span>{}</div>",
        hint(
            "読み方",
            "● = この要件はそのゴールのためにある ／ AC = 受入基準で確かめる ／ — = 受入基準では直接確かめない"
        )
    ));
    o.push(format!(
        "<div {}><table class=\"rtm\">",
        ctx.frame.dc(Component::RtmGrid)
    ));
    let heads = ctx
        .goals
        .iter()
        .map(|g| format!("{} {}", g.id, g.title))
        .collect::<Vec<_>>();
    o.push(format!(
        "<thead><tr><th>要件</th>{}<th>受入で確かめる</th><th>図のどこ</th></tr></thead>",
        heads
            .iter()
            .map(|h| format!("<th class=\"grp\">{h}</th>"))
            .collect::<String>()
    ));
    o.push("<tbody>".to_string());
    for it in ctx.fr.iter().chain(&ctx.nfr) {
        let x = &it.x;
        let goals = x
            .f("goals")?
            .seq()?
            .iter()
            .map(|q| Ok(ctx.goal(q)?.id))
            .collect::<R<Vec<_>>>()?;
        let mut row = format!(
            "<tr><th><a href=\"#{}\">{}</a><span class=\"lbl\">{}</span></th>",
            anchor(it.id),
            it.id,
            it.title
        );
        for (g, h) in ctx.goals.iter().zip(&heads) {
            if goals.contains(&g.id) {
                row.push_str(&format!(
                    "<td class=\"hit\" data-k=\"{h}\"><span class=\"dot\">●</span></td>"
                ));
            } else {
                row.push_str(&format!("<td data-k=\"{h}\"></td>"));
            }
        }
        let acs = x
            .f("verify")?
            .f("ac")?
            .seq()?
            .iter()
            .map(|q| Ok(format!("<span class=\"dot ac\">{}</span>", ctx.ac(q)?.id)))
            .collect::<R<Vec<_>>>()?;
        if acs.is_empty() {
            row.push_str("<td data-k=\"受入\">—</td>");
        } else {
            row.push_str(&format!(
                "<td class=\"hit\" data-k=\"受入\">{}</td>",
                acs.concat()
            ));
        }
        let figs = x
            .f("figures")?
            .seq()?
            .iter()
            .map(|f| {
                let w = ctx.fig(f)?;
                Ok(match w.href {
                    Some(h) => format!("<a class=\"fig\" href=\"{h}\">{}</a>", w.short),
                    None => w.short,
                })
            })
            .collect::<R<Vec<_>>>()?;
        let figs = if figs.is_empty() {
            "—".to_string()
        } else {
            figs.join(" · ")
        };
        row.push_str(&format!("<td data-k=\"図\">{figs}</td></tr>"));
        o.push(row);
    }
    o.push("</tbody>".to_string());
    o.push("</table></div>".to_string());
    o.push("</div>".to_string());
    Ok(())
}

pub(crate) fn glossary_chapter(o: &mut Vec<String>, ctx: &Ctx<'_>, s: &X<'_>, v: &X<'_>) -> R<()> {
    band(o, ctx, 8, Some(&s.ef("glossary_pointer")?));
    o.push("<div class=\"chapbody\">".to_string());
    o.push(format!("<div {}>", ctx.frame.dc(Component::GlossaryLinks)));
    for t in v.f("terms")?.seq()? {
        let href = format!("constitution.html#g-{}", t.f("id")?.id()?);
        let en = t.f("en")?;
        let en = if matches!(en.v, Value::Null) {
            String::new()
        } else {
            format!(" <span class=\"en\">{}</span>", en.e()?)
        };
        o.push(format!(
            "<span class=\"hint\"><a href=\"{href}\">{}{en}</a><label><input type=\"checkbox\" class=\"vh\" aria-label=\"説明を開く\"><span class=\"hint-btn q\">?</span></label><span class=\"hint-body\">{}<br><a href=\"{href}\">憲法 §7 の定義へ</a></span></span>",
            t.ef("term")?,
            t.ef("short")?
        ));
    }
    o.push("</div>".to_string());
    o.push("</div>".to_string());
    Ok(())
}
