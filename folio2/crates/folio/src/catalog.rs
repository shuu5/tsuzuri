//! 部品目録の閉じた一覧の置き場（便 108・docs/design/delivery-108.md §1 (b)・ADR-15・層 1 読む）。組み立て時に
//! 部品目録 `preview/parts.json` から build.rs が導出し `OUT_DIR` に書いた閉じた一覧（部品・図の型・棚の型・行内の様式に
//! 許す性質・上限・図の型の名札）を取り込む（P-2.4・P-6.4）。検査は `parts.rs` が持つ。
#![allow(dead_code)]

include!(concat!(env!("OUT_DIR"), "/parts_catalog.rs"));
