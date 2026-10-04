//! 待っている相手の口の id の字を縮めない stylesheet の規則の歯（接頭辞 shcss_・設計ノート surface-wave27b 行 c-short-cap・
//! 判断の記録 ADR-30 決定 (5)）。口は吹き出しの partner_view の button（class plk）で、短い題の span と id の code を並べる。
#![cfg(test)]

const CSS: &str = include_str!("../style.css");

/// 行頭が `selector` と空白と字 { の規則の宣言（{ と } の間の字・前後の空白を除く）。無ければ None。
fn rule<'a>(css: &'a str, selector: &str) -> Option<&'a str> {
    let head = format!("{selector} {{");
    let line = css.lines().find(|l| l.starts_with(&head))?;
    let body = line.strip_prefix(&head)?;
    Some(body.split('}').next()?.trim())
}

#[test]
fn shcss_partner_id_not_shrunk() {
    let decls = |sel: &str| -> Vec<String> {
        rule(CSS, sel)
            .unwrap_or_else(|| panic!("{sel} の規則が無い"))
            .split(';')
            .map(|d| d.trim().to_string())
            .filter(|d| !d.is_empty())
            .collect()
    };
    assert!(
        decls(".plk code").contains(&"flex: none".to_string()),
        "id の code は縮まない"
    );
    assert!(
        decls(".plk > span").contains(&"min-width: 0".to_string()),
        "短い題の span が先に縮む"
    );
}
