//! 便 g-steady の歯: 読みの後の決め方（置く・置かない・読み直す）・知らせが切れたときは決め方を通さない・
//! 畳める段の開き閉じの記録（鍵ごとに 1 つの値）・project の下の details の要素は全部記録から読んで toggle で書き戻す。
#![cfg(test)]

use std::collections::BTreeSet;
use std::path::PathBuf;

use tsuzuri_surface::project::{Folds, Module, fold_key_ok};
use tsuzuri_surface::view::{Fetched, RETRY_MS, Settle, settle};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn body(s: &str) -> Fetched {
    Fetched::Body(s.to_string())
}

/// (1) 同じ本文は置かない・違う本文は置く・まだ読んでいないから本文へは置く（何回目でも）。
#[test]
fn steady_settle_bodies() {
    for attempt in [1, 2] {
        assert_eq!(settle(&body("a"), &body("a"), attempt), Settle::Keep);
        assert_eq!(settle(&body("a"), &body("b"), attempt), Settle::Set);
        assert_eq!(settle(&Fetched::NotRead, &body("a"), attempt), Settle::Set);
        assert_eq!(settle(&Fetched::Failed, &body("a"), attempt), Settle::Set);
    }
}

/// (2) 本文の後の読めないは 1 回目なら読み直し、2 回目なら置く。まだ読んでいないの後の読めないは 1 回目でも置く。
/// 読めないの後の読めないは置かない。
#[test]
fn steady_settle_failed() {
    assert_eq!(settle(&body("a"), &Fetched::Failed, 1), Settle::Retry);
    assert_eq!(settle(&body("a"), &Fetched::Failed, 2), Settle::Set);
    assert_eq!(settle(&body("a"), &Fetched::Failed, 3), Settle::Set);
    assert_eq!(settle(&Fetched::NotRead, &Fetched::Failed, 1), Settle::Set);
    assert_eq!(settle(&Fetched::NotRead, &Fetched::Failed, 2), Settle::Set);
    assert_eq!(settle(&Fetched::Failed, &Fetched::Failed, 1), Settle::Keep);
    assert_eq!(settle(&Fetched::Failed, &Fetched::Failed, 2), Settle::Keep);
    assert_eq!(RETRY_MS, 1000);
}

/// net.rs の関数の本文（`fn name(` から次の `\nfn ` か `\npub fn ` まで）。
fn function<'a>(text: &'a str, name: &str) -> &'a str {
    let start = text
        .find(&format!("fn {name}("))
        .unwrap_or_else(|| panic!("net に fn {name} が在る"));
    let rest = &text[start..];
    let end = ["\nfn ", "\npub fn ", "\nasync fn ", "\npub async fn "]
        .iter()
        .filter_map(|m| rest[1..].find(m).map(|i| i + 1))
        .min()
        .unwrap_or(rest.len());
    &rest[..end]
}

/// (3) 口の読みは決め方を通し、知らせが切れたとき（lose_all）は通さずに全部を読めないに置く。
#[test]
fn steady_net_uses_settle_but_lose_all_does_not() {
    let net = read("src/net.rs");
    assert!(
        net.contains("settle(now, &fetched, attempt)"),
        "読みが決め方を通さない"
    );
    let lose = function(&net, "lose_all");
    assert!(lose.contains("signal.set(Fetched::Failed)"), "{lose}");
    assert!(!lose.contains("settle"), "lose_all が決め方を通す: {lose}");
    let load_at = function(&net, "load_at");
    for arm in ["Settle::Set", "Settle::Keep", "Settle::Retry", "RETRY_MS"] {
        assert!(load_at.contains(arm), "load_at に {arm} が無い: {load_at}");
    }
    // 外形は変えない（read・reload_all・post の名と引数と返りの型）。
    for sig in [
        "pub fn read(path: &'static str) -> ReadSignal<Fetched>",
        "pub fn reload_all()",
        "pub async fn post(path: &str, body: String) -> Option<(u16, String)>",
    ] {
        assert!(net.contains(sig), "net の外形が変わった: {sig}");
    }
}

/// (4) 記録は鍵ごとに 1 つの値: 初めての鍵は与えた初めの値・置いた値はその後の読みで返る・別の鍵を変えない。
#[test]
fn steady_folds_per_key() {
    let mut f = Folds::default();
    assert_eq!(f.recorded("seat:hist"), None);
    assert!(!f.open("seat:hist", false));
    assert!(f.open("seat:hist", true));
    f.set("seat:hist", true);
    assert_eq!(f.recorded("seat:hist"), Some(true));
    assert!(f.open("seat:hist", false));
    assert_eq!(f.recorded("seat:more"), None);
    assert!(!f.open("seat:more", false));
    f.set("seat:more", false);
    assert!(f.open("seat:hist", false), "別の鍵の値が変わった");
    f.set("seat:hist", false);
    assert!(!f.open("seat:hist", true));
    assert_eq!(f.recorded("seat:more"), Some(false));
}

/// (6) 書き戻しは出している値と違うときだけ置く: 経験者の mode で開いた「詳しく」は記録に値が無いまま mode に従い、
/// 持ち主が閉じた後は mode を切り替えても記録の値を使う。
#[test]
fn steady_folds_write_back_keeps_mode_until_owner_toggles() {
    let key = "ledger:more";
    let mut f = Folds::default();
    // 経験者の mode: 初めの値 true で開き、その toggle は置かない。
    assert!(f.open(key, true));
    assert!(!f.write_back(key, true, true));
    assert_eq!(f.recorded(key), None);
    // mode を切り替えると初めの値に従う。
    assert!(!f.open(key, false));
    // 持ち主が開いた（初めの値 false と違う）ので置く。
    assert!(f.write_back(key, true, false));
    assert_eq!(f.recorded(key), Some(true));
    // その後は mode に関わらず記録の値。
    assert!(f.open(key, false));
    assert!(f.open(key, true));
    // 持ち主が閉じた。
    assert!(f.write_back(key, false, true));
    assert!(!f.open(key, true));
    assert!(!f.open(key, false));
}

/// 鍵の形は着地済みの 6 つを全部含み（{} は id・数と順は見ない）、形に合う字だけを通す。
#[test]
fn steady_fold_key_forms() {
    let forms = tsuzuri_surface::project::fold_keys();
    for want in [
        "ledger:more",
        "seat:hist",
        "seat:more",
        "ask:hist",
        "gaps:{}",
        "ask:around:{}",
    ] {
        assert!(forms.contains(&want), "fold_keys に {want} が無い: {forms:?}");
    }
    for ok in [
        "ledger:more",
        "seat:hist",
        "seat:more",
        "ask:hist",
        "gaps:g-1",
        "ask:around:t3-hub.4",
    ] {
        assert!(fold_key_ok(ok), "{ok}");
    }
    for bad in [
        "ledger",
        "seat:hist2",
        "gaps:",
        "ask:around:",
        "ask:more",
        "",
    ] {
        assert!(!fold_key_ok(bad), "{bad}");
    }
}

/// 記録の関数を呼ぶ所の最初の字の引数（`fold("…"` か `fold(format!("…"`）。
/// 定義の `fn fold(`・別の名の中・iterator の `.fold(` は数えない。
fn fold_keys(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(i) = text[from..].find("fold(") {
        let at = from + i;
        from = at + "fold(".len();
        let before = &text[..at];
        if before
            .chars()
            .next_back()
            .is_some_and(|c| c.is_alphanumeric() || c == '_' || c == '.')
            || before.ends_with("fn ")
        {
            continue;
        }
        let rest = &text[from..];
        let rest = rest.strip_prefix("format!(").unwrap_or(rest);
        let lit = rest
            .strip_prefix('"')
            .and_then(|r| r.split_once('"'))
            .map(|(k, _)| k.to_string())
            .unwrap_or_else(|| panic!("fold の鍵が字でない: {}", &rest[..rest.len().min(40)]));
        out.push(lit);
    }
    out
}

/// (5) project の下の details の要素は全部、記録を呼ぶ所と同じ数で、prop の open と toggle に結び、
/// module ごとに数と鍵の字の集合がその module の folds と同じ。
/// src/project_dom の下に module と同じ名の file（DOM を割った先）が在れば、その字を後に足して見る。
#[test]
fn steady_details_use_fold_record() {
    for module in Module::ALL {
        let path = crate_dir()
            .join("src/project")
            .join(format!("{}.rs", module.name()));
        let mut text = std::fs::read_to_string(&path).expect("src の file");
        let dom = crate_dir()
            .join("src/project_dom")
            .join(format!("{}.rs", module.name()));
        if dom.is_file() {
            text.push_str(&std::fs::read_to_string(&dom).expect("src/project_dom の file"));
        }
        let name = path.display().to_string();
        let tags: Vec<&str> = text
            .match_indices("<details")
            .map(|(i, _)| {
                let rest = &text[i..];
                &rest[..rest.find('>').expect("details の tag の終わり")]
            })
            .collect();
        for tag in &tags {
            assert!(tag.contains("prop:open=open"), "{name}: {tag}");
            assert!(tag.contains("on:toggle=toggle"), "{name}: {tag}");
            assert!(
                !tag.contains(" open="),
                "{name} が記録の外の open を持つ: {tag}"
            );
        }
        let here = fold_keys(&text);
        assert_eq!(
            tags.len(),
            here.len(),
            "{name} の details と記録を呼ぶ所の数"
        );
        assert_eq!(
            here.len(),
            module.folds().len(),
            "{name} の記録を呼ぶ所と folds の数"
        );
        let got: BTreeSet<&str> = here.iter().map(String::as_str).collect();
        let want: BTreeSet<&str> = module.folds().iter().copied().collect();
        assert_eq!(got, want, "{name} の鍵の字が folds と合わない");
        for k in &here {
            assert!(fold_key_ok(&k.replace("{}", "x")), "{k}");
        }
    }
}

/// 字を読む関数は、呼ぶ所の鍵を拾い、定義と別の名を数えない。
#[test]
fn steady_fold_key_reader() {
    let text = r#"
        pub fn fold(key: String) {}
        let (o, t) = fold("seat:hist".to_string(), || false);
        let (o, t) = fold(format!("gaps:{}", r.id), move || i);
        let x = unfold("no");
        let n = days.iter().fold(1, u32::max);
    "#;
    assert_eq!(fold_keys(text), vec!["seat:hist", "gaps:{}"]);
}
