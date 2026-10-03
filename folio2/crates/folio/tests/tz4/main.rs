//! folio の歯の群 tz4（1 つの test target・nextest の歯の名は <module>::<fn>・判断の記録 ADR-32）。
//! この file は mod の行だけを持つ（tests/tz4/ の下の .rs は全部ここで名指す・xtask の歯 kfold_ が照らす）。
#![cfg(test)]

mod note;
mod outside_faces;
mod parts;
mod place_name;
mod plan;
mod polarity;
mod proposed;
mod prose;
mod refs;
mod resolve;
mod ruling;
mod schema;
mod schema_docs;
mod seal;
mod sheet;
mod site;
