//! 撃ち直しの会話の続きの歯（接頭辞 cwrsm_・設計ノート surface-wave29b 行 cs-trust・判断の記録 ADR-55 決定 (1)(3)）。
#![cfg(test)]

use tsuzuri_contract::consult::Form;
use tsuzuri_core::consult::resume::pick;

const SID: &str = "bf1f3252-2ac9-42ce-bfb7-b5897fa42308";
const SID2: &str = "772c5b4c-0000-4000-8000-00000000ABCD";

/// 話す窓の撃ち直しは印の最後の id で続け、撃ち直しでない周・問う窓・印の無い窓は続けず、印の最後の id が uuid の形でなければ
/// 断る。席の名指しは話す窓の撃ち直しで印を持たない窓の uuid の形の id だけを受け、撃ち直しでない・問う窓・形でない・印を持つ窓を
/// この順に断る（断りの字は互いに違う）。
#[test]
fn cwrsm_pick_continues_or_takes_a_named_id() {
    let id = |s: &str| Ok(Some(s.to_string()));
    let (t, a) = (Form::Talk, Form::Ask);
    assert_eq!(pick(t, true, Some(SID), None), id(SID));
    assert_eq!(pick(t, true, Some(SID2), None), id(SID2));
    assert_eq!(pick(t, true, None, Some(SID)), id(SID));
    for (form, again, last) in [
        (t, false, Some(SID)),
        (a, true, Some(SID)),
        (t, true, None),
        (t, false, Some("x")),
    ] {
        assert_eq!(
            pick(form, again, last, None),
            Ok(None),
            "{form:?} {again} {last:?}"
        );
    }
    let refused = [
        (t, true, Some("bf1f3252"), None),
        (t, false, None, Some(SID)),
        (a, false, None, Some("x")),
        (a, true, Some(SID), Some("x")),
        (t, true, None, Some("bf1f3252-2ac9-42ce-bfb7-b5897fa4230")),
        (t, true, Some(SID2), Some(SID)),
    ];
    let words: Vec<&str> = refused
        .iter()
        .map(|(f, g, l, n)| pick(*f, *g, *l, *n).expect_err("断る"))
        .collect();
    assert_eq!(words[1], words[2], "撃ち直しでない名指しは形より先に断る");
    assert!(words[3].contains("問う窓"), "{}", words[3]);
    let mut uniq = words.clone();
    uniq.sort_unstable();
    uniq.dedup();
    assert_eq!(uniq.len(), 5, "{words:?}");
}
