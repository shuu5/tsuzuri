//! folio v2 の lib。区切りの宣言はここに置き、crate の外へ公開する口は命令の入口 `entry` の 1 つだけ（ほかは crate の中・歯だけが使う 3 つを除く）。
//! tz の口が同じ入口を撃つ（行 k-tz-entry・要件 FR17）。folio の binary は無い（行 k-tz-tests・src/main.rs は folio2/retired/ へ退役）。
//! 歯だけが使う 3 つ（YAML の読み手の yaml_rust2 と、区切り sha256 と yaml）を doc に出さずに公開する（tz の binary を撃つ
//! 境界の package の歯が、外の依存を足さずに読む・行 k-tz-tests）。

pub mod entry;
#[doc(hidden)]
pub use yaml_rust2;

mod adr;
mod anchor;
mod bundle;
mod catalog;
mod ceiling;
mod ceiling_src;
mod check;
mod constitution_enums;
mod context;
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
mod seat;
#[doc(hidden)]
pub mod sha256;
mod sheet;
mod shelf;
mod site;
mod stamp;
mod verdict;
mod vocab;
#[doc(hidden)]
pub mod yaml;
