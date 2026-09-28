//! 契約の型の外形の歯（接頭辞 contract_form_）の共通の手（群の歯の file tests/cform_*.rs が行 mod common; で使う）。
//! 見本の値は持たない（見本は群の file に在る）。snapshot の読み比べと往復と閉じた列の語の数えをここに 1 つだけ置く。
//! 群の binary はどれも共通の手の一部だけを使うので、使わない手を咎めない。

#![allow(dead_code)]

use std::collections::BTreeSet;
use std::fmt::Debug;
use std::path::Path;

use serde::Serialize;
use serde::de::DeserializeOwned;
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_contract::surface::RulingId;
use tsuzuri_contract::wire;

pub const AT: u64 = 1_790_494_620;

pub fn bead(s: &str) -> BeadId {
    BeadId::new(s).expect("見本の bead id")
}

pub fn ruling(s: &str) -> RulingId {
    RulingId::new(s).expect("見本の記帳 id")
}

/// 1 つの型の見本の値の列。
pub struct Samples<T> {
    pub name: &'static str,
    pub values: Vec<T>,
}

pub trait Form {
    fn name(&self) -> &'static str;
    fn json(&self) -> String;
    fn roundtrip(&self) -> Result<(), String>;
}

impl<T: Serialize + DeserializeOwned + PartialEq + Debug> Form for Samples<T> {
    fn name(&self) -> &'static str {
        self.name
    }

    fn json(&self) -> String {
        serde_json::to_string_pretty(&self.values).expect("見本を JSON にする")
    }

    fn roundtrip(&self) -> Result<(), String> {
        for v in &self.values {
            let text = wire::encode(v).map_err(|e| format!("{}: encode: {e}", self.name))?;
            let back: T =
                wire::decode(&text).map_err(|e| format!("{}: decode {text}: {e}", self.name))?;
            if &back != v {
                return Err(format!("{}: 値が戻らない {v:?} → {back:?}", self.name));
            }
            let again = wire::encode(&back).map_err(|e| format!("{}: encode: {e}", self.name))?;
            if again != text {
                return Err(format!("{}: 字が戻らない {text} → {again}", self.name));
            }
        }
        Ok(())
    }
}

pub fn form<T: Serialize + DeserializeOwned + PartialEq + Debug + 'static>(
    name: &'static str,
    values: Vec<T>,
) -> Box<dyn Form> {
    Box::new(Samples { name, values })
}

/// snapshot の字（型の名を key にした 1 つの JSON の object・key は `forms` の順・値は電文の欄の順のまま）。
pub fn snapshot_text(forms: &[Box<dyn Form>]) -> String {
    let names: BTreeSet<&str> = forms.iter().map(|f| f.name()).collect();
    assert_eq!(names.len(), forms.len(), "型の名が重なる");
    let entries: Vec<String> = forms
        .iter()
        .map(|f| {
            let body = f.json().replace('\n', "\n  ");
            format!("  {}: {body}", serde_json::to_string(f.name()).expect("名"))
        })
        .collect();
    let text = format!("{{\n{}\n}}\n", entries.join(",\n"));
    serde_json::from_str::<serde_json::Value>(&text).expect("snapshot の字が JSON である");
    text
}

/// 群の snapshot（tests/snapshots の群の名の json）の字が群の見本の字と同じことを見る。
pub fn snapshot_matches(group: &str, forms: &[Box<dyn Form>]) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("tests/snapshots/{group}.json"));
    let want = std::fs::read_to_string(&path).unwrap_or_default();
    let got = snapshot_text(forms);
    if got != want {
        let out = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("{group}.json"));
        std::fs::write(&out, &got).expect("今の字を書く");
        let line = got
            .lines()
            .zip(want.lines())
            .position(|(g, w)| g != w)
            .map_or_else(
                || got.lines().count().min(want.lines().count()) + 1,
                |i| i + 1,
            );
        panic!(
            "snapshot の字が {} 行目で違う（今の字 = {}）",
            line,
            out.display()
        );
    }
}

/// 群の見本がどれも電文の往復で値と字を保つことを見る。
pub fn roundtrip_all(forms: &[Box<dyn Form>]) {
    let fails: Vec<String> = forms.iter().filter_map(|f| f.roundtrip().err()).collect();
    assert!(fails.is_empty(), "往復が一致しない: {fails:#?}");
}

/// 閉じた列の異なる語の数。
pub fn distinct<T: Serialize>(all: &[T]) -> usize {
    all.iter()
        .map(|v| serde_json::to_string(v).expect("語"))
        .collect::<BTreeSet<_>>()
        .len()
}
