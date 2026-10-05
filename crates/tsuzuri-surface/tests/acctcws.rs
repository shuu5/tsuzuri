//! 行 c-acct-consult の面の歯（接頭辞 acctcws_・判断の記録 ADR-55 決定 (4)）: session の表の相談の窓の行（役 consult）の
//! 役の字と語・合図と停止の切り替えと段を持たない・口座の無い窓は「分からない」・session の欄の card は窓の card。
//! fixture は tests/fixtures/account/acct-doc.json（読むだけ）で、3 行目（proj-b の席）を相談の窓の行に替えて試す。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_contract::account::AccountDoc;
use tsuzuri_contract::board::Stage;
use tsuzuri_contract::surface::SeatRole;
use tsuzuri_contract::wire;
use tsuzuri_surface::account::session::{
    self, CONSULT_SRC, no_account_key, role_key, role_word, sess_card,
};
use tsuzuri_surface::project::state_key;
use tsuzuri_surface::vocab::{label, vocab};

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// fixture の 3 行目を窓 cw1 の行にした電文（口座 acct-1・段 Running を持たせても出さない）。
fn doc() -> AccountDoc {
    let mut d: AccountDoc =
        wire::decode(&read("../../tests/fixtures/account/acct-doc.json")).expect("fixture");
    let line = d.sessions.get_mut(2).expect("3 行目");
    line.role = SeatRole::Consult;
    line.name = "cw1".into();
    line.stage = Some(Stage::Running);
    d
}

/// (1) 役の字は consult・語の鍵は role:consult（語は相談の窓）・行の class は srow r-consult。
/// 合図・停止の切り替え・段の字を持たない。口座の無い行の語の鍵は窓だけ consult_account_unknown（語は分からない）。
#[test]
fn acctcws_consult_row_words() {
    assert_eq!(
        (role_word(SeatRole::Consult), role_key(SeatRole::Consult)),
        ("consult", "role:consult")
    );
    let d = doc();
    let r = session::row(&d, 2);
    assert_eq!(r.class, "srow r-consult");
    assert_eq!(r.role_key, "role:consult");
    assert_eq!(r.session.as_deref(), Some("cw1"));
    assert_eq!(r.account.as_deref(), Some("acct-1"));
    assert_eq!((r.signs, r.toggle, r.stage), (None, None, None));
    assert_eq!(r.no_account, "consult_account_unknown");
    assert_eq!(session::row(&d, 0).no_account, "st_unknown");
    assert_eq!(no_account_key(SeatRole::Pipeline), "st_unknown");
    let terms = vocab();
    let word = |k: &str| terms.term(k).map(|t| t.label.clone());
    assert_eq!(word("role:consult").as_deref(), Some("相談の窓"));
    assert_eq!(word("consult_account_unknown").as_deref(), Some("分からない"));
    let css = read("style.css");
    assert!(css.contains(".srow.r-pipeline .c-role, .srow.r-consult .c-role { padding-left: var(--s4); }"));
    assert!(css.contains(".srow.r-pipeline .c-role, .srow.r-consult .c-role { padding-left: var(--s2); }"));
    let src = read("src/account/session.rs");
    assert!(src.contains("{label(row.no_account)}"), "DOM は行の鍵の語を出す");
}

/// (2) 窓の行の session の欄の card: 題は窓の id・種類は役の語と project・値は状態の語・出所は作業場の印・
/// 口座といつから（口座が無ければ「分からない」）。席の card と project の card を出さない。
#[test]
fn acctcws_consult_card() {
    let d = doc();
    let c = sess_card(&d, 2);
    assert_eq!(c.title, "cw1");
    assert_eq!(c.kind, format!("{} · proj-b", label("role:consult")));
    assert_eq!(c.value, label(state_key("wait")));
    assert_eq!(c.src, CONSULT_SRC);
    assert_eq!(c.more.first().map(String::as_str), Some("口座 acct-1"));
    assert!(c.more.get(1).is_some_and(|m| m.starts_with("◷ ") && m.ends_with(" から")));
    let mut x = d.clone();
    let line = x.sessions.get_mut(2).expect("3 行目");
    line.account = None;
    line.since = None;
    let c = sess_card(&x, 2);
    assert_eq!(c.more, vec![format!("口座 {}", label("consult_account_unknown"))]);
    let orch = sess_card(&fixture_orch(), 2);
    assert_ne!(orch.src, CONSULT_SRC, "席の行は窓の card にしない");
}

fn fixture_orch() -> AccountDoc {
    wire::decode(&read("../../tests/fixtures/account/acct-doc.json")).expect("fixture")
}
