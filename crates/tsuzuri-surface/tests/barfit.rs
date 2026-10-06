//! 帯の収まりの歯（接頭辞 barfit_）。
//! 帯の並びと幅は browser でしか測れないので、振る舞いは受入の runner が測り、ここは基の stylesheet の替えた字が在ることを照らす。
#![cfg(test)]

use std::path::PathBuf;

fn read(rel: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// div.todo は幅の条件の外の規則だけが横 scroll を持ち（どの幅でも div.todo の中だけが scroll する）、
/// 1199 px 以下の段に同じ字の重ねを持たない。
#[test]
fn barfit_todo_scrolls_inside() {
    let css = read("style.css");
    let rule = "#bar .todo { flex: 1 1 auto; min-width: 0; display: flex; align-items: center; gap: 8px; overflow-x: auto; overflow-y: hidden; scrollbar-width: thin; }";
    let outside = css
        .lines()
        .filter(|l| !l.starts_with("@media") && !l.starts_with(char::is_whitespace))
        .filter(|l| l.trim() == rule)
        .count();
    assert_eq!(outside, 1, "幅の条件の外の div.todo の規則");
    let twice = "#bar .todo { overflow-x: auto; overflow-y: hidden; scrollbar-width: thin; }";
    assert!(!css.contains(twice), "1199 px 以下の段に scroll の字が残っている");
}

/// 760 px 以下の 2 段の格子は、2 段目に todo（1〜3 列）・cslt（4 列）・gp（5 列）を置く。
#[test]
fn barfit_narrow_grid_places_consult() {
    let css = read("style.css");
    let lines: Vec<&str> = css.lines().collect();
    let mut blocks: Vec<&[&str]> = Vec::new();
    for (i, l) in lines.iter().enumerate() {
        if *l == "@media (max-width: 760px) {"
            && lines
                .get(i + 1)
                .is_some_and(|n| n.trim().starts_with("#bar { height: 96px;"))
        {
            let end = lines[i..]
                .iter()
                .position(|x| x.trim() == "}")
                .map_or(lines.len(), |e| i + e);
            blocks.push(&lines[i..end]);
        }
    }
    assert_eq!(blocks.len(), 1, "2 段の格子の塊");
    let block = blocks[0];
    let count = |want: &str| block.iter().filter(|l| l.trim() == want).count();
    assert_eq!(count("#bar .cslt { grid-area: 2 / 4; justify-self: end; }"), 1);
    assert_eq!(count("#bar .gp { grid-area: 2 / 5; justify-self: end; }"), 1);
    assert_eq!(
        count("#bar .gp { grid-area: 2 / 4 / 3 / 6; justify-self: end; }"),
        0,
        "gp が cslt の列に重なる古い置き方が残っている"
    );
}
