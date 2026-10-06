//! 契約の bead の便の審査の event の detail に足す 2 つの digest の語（tsuzuri の判断の記録 ADR-72 の撤退の窓 C の数える材料・契約表の行 v-bead-dispatch）。
//!
//! 審査の材料の dir の契約の写しと設計の材料（lens が読んだ字）の digest と、run の dir の契約 file と今の写しの pointer から読み直した字の digest を、
//! 語 `material:` と `copy:` で並べる。2 つは同じ便では同じ値のはずで、食い違う event が窓 C の材料になる。写しを置いた便（契約の design が
//! 写しの path の形）だけが語を持ち、表の側の便の detail は 1 字も替わない（[`read_detail`](super::read_detail) は語で読むので判定と理由の型の読みも替わらない）。

use super::{design_material, review_dir, trimmed, Review, DESIGN_FILE};
use crate::pipe::bead::digest_of_design;
use crate::pipe::row_review::row_digest;
use crate::pipe::{contract_path, CONTRACT_FILE};

/// detail に足す字（先頭に空白 1 つ・`material:<16 桁> copy:<16 桁>`）。引数は（契約の字・設計の節の字）の対で、末の改行 1 つは呼び手が除く。
pub(super) fn words(material: (&str, &str), copy: (&str, &str)) -> String {
    format!(" material:{} copy:{}", row_digest(material.0, material.1), row_digest(copy.0, copy.1))
}

/// 便の審査の detail に足す字。契約の design が写しの path の形の便だけ [`words`] の字を返し、ほかと材料も写しも読めない周は空の字。
pub(super) fn words_of(entry: &Review<'_>) -> String {
    if digest_of_design(&entry.contract.design).is_none() {
        return String::new();
    }
    let dir = review_dir(entry.state_dir, entry.run);
    let read = |path: std::path::PathBuf| std::fs::read_to_string(path).ok();
    let material = read(dir.join(CONTRACT_FILE)).zip(read(dir.join(DESIGN_FILE)));
    let copy = read(contract_path(entry.state_dir, entry.run)).map(|contract| (contract, design_material(entry.repo, &entry.contract.design)));
    match (material, copy) {
        (Some(material), Some(copy)) => words((trimmed(&material.0), trimmed(&material.1)), (trimmed(&copy.0), trimmed(&copy.1))),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::words;
    use crate::pipe::gate::Verdict;
    use crate::pipe::review::{detail_of, read_detail, FindingKind};
    use crate::pipe::row_review::row_digest;

    /// 材料と写しの字の対から 2 つの語の値（16 桁）を取る。
    fn values_of(text: &str) -> (String, String) {
        let value = |head: &str| text.split_whitespace().find_map(|word| word.strip_prefix(head)).unwrap_or_default().to_owned();
        (value("material:"), value("copy:"))
    }

    /// 同じ字の対は同じ 16 桁を `material:` と `copy:` に並べ（値は `row_digest`）、写しの 1 字で 2 つが違い、detail の語を足しても判定と理由の型の読みは替わらない。
    #[test]
    fn vbdisp_copy_words_name_both_digests() {
        let (contract, section) = ("契約 file の字", "設計の節の本文");
        let same = words((contract, section), (contract, section));
        let digest = row_digest(contract, section);
        assert_eq!(same, format!(" material:{digest} copy:{digest}"));
        assert_eq!(digest.len(), 16, "16 桁");
        let moved = words((contract, section), (contract, "設計の節の本文。"));
        let (material, copy) = values_of(&moved);
        assert_eq!(material, digest, "材料の側は替わらない");
        assert_eq!(copy, row_digest(contract, "設計の節の本文。"), "写しの側は row_digest の値");
        assert_ne!(material, copy, "写しを 1 字替えると 2 つが違う");
        let base = detail_of(Verdict::Fail, Some(FindingKind::Other));
        for text in [&same, &moved] {
            assert_eq!(read_detail(&format!("{base}{text}")), read_detail(&base), "語を足しても読みは同じ: {text}");
        }
        assert_eq!(read_detail(&format!("{base}{same}")), (Some(Verdict::Fail), FindingKind::Other));
    }
}
