//! folio v2 の lib。区切りの宣言はここに置き、crate の外へ公開するのは命令の入口 `entry` の 1 つだけ（ほかは crate の中）。
//! tz の口が同じ入口を撃つ（行 k-tz-entry・要件 FR17）。
//! 下の lint の一覧は規則の行 R-10 の lint の全部と条 P-24.3 の 2 つを持ち、除外の表の R-10 の行を消して manifest の [lints] を
//! workspace = true にした後も、表がまだ持たない lint のために残す（行 k-lint-folio の後も成り立つ字）。
#![forbid(unsafe_code)]
#![deny(
    unused_must_use,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::todo,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::exit,
    clippy::indexing_slicing,
    clippy::dbg_macro,
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::allow_attributes,
    clippy::allow_attributes_without_reason
)]

pub mod entry;

mod adr;
mod anchor;
mod bundle;
mod catalog;
mod ceiling;
mod ceiling_src;
mod check;
mod constitution_enums;
mod cursor;
mod derive;
mod entrance;
mod face;
mod face_adr;
mod face_constitution;
mod face_constitution_read;
mod face_index;
mod face_index_read;
mod face_labels;
mod face_note;
mod face_srs;
mod face_srs_items;
mod face_srs_rtm;
mod figure;
mod findings;
mod floor;
mod floor_adr;
mod floor_note;
mod freeze;
mod gate;
mod gitcheck;
mod graph;
mod hello;
mod ids;
mod init;
mod intake;
mod lineage;
mod link;
mod mentions;
mod note;
mod parts;
mod phase;
mod plan;
mod polarity;
mod proposed;
mod prose;
mod refs;
mod rules;
mod ruling;
mod schema;
mod seal;
mod sha256;
mod sheet;
mod shelf;
mod site;
mod stamp;
mod verdict;
mod vocab;
mod yaml;
