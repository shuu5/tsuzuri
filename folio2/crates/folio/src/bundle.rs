//! `folio ceiling --write`（便 38・docs/design/delivery-38.md §1 (a)〜(d)）。天井（AI による意味の検査・ADR-8 決定 (2)・FR17）の
//! 材料の束を観点ごとに 1 つの置き場へ組む。folio は AI を起動しない。束を読んで AI を回すのは席か器で、所見と起動の記録を
//! 同じ置き場へ書く（便 39 がそれを数える）。
//! 束の中身 = 床の定数 `ceiling::BUNDLE_CONTENTS` の 5 つ（sources/・faces/・question.yaml・finding.yaml・reads.yaml）と要約値 digest.txt。
//! 要約値の規則 = 5 つの下の file を観点の dir からの相対 path の byte 順に並べ、中身を区切りなしに連結した byte 列の sha256
//! （rules 行 R-15 の写しの要約値と同じ規則・`sha256::hex`）。
//! 天井の正本から読むのは documents・viewpoints・weights だけで、所見の欄の決まりの残りは床の定数（`ceiling.rs`）から取る
//! （便 47・ADR-11 決定 (3)(イ)・P-5.1）。file の側から束の形を変える経路は無い（正本の生成区間は床 `ceiling.rs` が突き合わせる）。
//! 決定性: 同じ入力から byte まで同じ束が組める（時刻・絶対 path・環境の値をどの file にも書かない）。
//! 全部か無しか: 4 観点の全 file を memory の上で先に用意し、1 本でも用意できなければ何も書かず 2（P-4.1）。
//! 書くときは各観点の sources/ と faces/ を消してから作り直す（古い写しが要約値に混ざらないため）。
//! 席や器が書く所見 file・起動の記録は触らない。判定を持たないので 1（不合格）は返さない（FR5 の 3 値のうち 2 つ）。
//! 便 39（`findings.rs`・`--check`）は正本の読み `load`・観点 1 つの組み立て `build_one`・要約値 `digest_text` を
//! crate の中から呼ぶ（--write の振る舞いと文言は不変）。便 42（`--refute`）は `question_text` も呼ぶ。
//! 便 98（docs/design/delivery-98.md §1 (b)(c)・FR17）: sources/ の写しは観点の reads が挙げた最上位の節・骨格 6 語・file の頭
//! の行だけを byte のまま正本の順に残す（行の中の欄には降りない）。落とした節と 宣言に在るが正本に無い節 は reads.yaml の
//! 末尾の注釈の行で名指し、標準出力の 1 行にその数を足す（束の中身の閉じた一覧は動かさない・sources/ へは書かない）。
//! 天井の正本の読み手・束の組み直し・要約値・面の名の形・結果の型は便 110 で `ceiling_src.rs` へ降ろした（ADR-15・層 1 読む）。

use std::fs;
use std::path::{Path, PathBuf};

use crate::ceiling_src::{Built, Outcome, build_all, load};
use crate::verdict::Verdict;

// ── 命令の口 ──

/// `--faces` と `--out` は相対なら `--dir` からの相対・絶対ならそのまま（`folio build` と同じ読み）。
pub fn run(dir: &Path, faces: &Path, out: &Path) -> Outcome {
    let faces_dir = dir.join(faces);
    let out_dir = dir.join(out);
    let ceiling = match load(dir) {
        Ok(c) => c,
        Err(e) => return Outcome::unknown(e),
    };
    let bundles = match build_all(dir, &faces_dir, &ceiling) {
        Ok(b) => b,
        Err(e) => return Outcome::unknown(e),
    };
    write_all(&out_dir, &bundles)
}

// ── 置き場へ書く ──

fn write_all(out_dir: &Path, bundles: &[(String, Built)]) -> Outcome {
    if !out_dir.is_dir() {
        if !out_dir.parent().is_some_and(Path::is_dir) {
            return Outcome::unknown(format!("{}: 置き場の親 dir が無い", out_dir.display()));
        }
        if let Err(e) = fs::create_dir(out_dir) {
            return Outcome::unknown(format!("{}: 置き場を作れない: {e}", out_dir.display()));
        }
    }
    let mut count = 0;
    let mut total = 0;
    let (mut dropped, mut absent) = (0, 0);
    for (id, built) in bundles {
        let files = &built.files;
        dropped += built.dropped;
        absent += built.absent;
        let vp_dir = out_dir.join(id);
        // 古い写しが要約値に混ざらないよう sources/ と faces/ は消してから作り直す。他の file は触らない
        for sub in ["sources", "faces"] {
            let path = vp_dir.join(sub);
            if path.exists()
                && let Err(e) = fs::remove_dir_all(&path)
            {
                return Outcome::unknown(format!("{}: 消せない: {e}", path.display()));
            }
        }
        for (rel, bytes) in files {
            let path: PathBuf = vp_dir.join(rel);
            if let Some(parent) = path.parent()
                && let Err(e) = fs::create_dir_all(parent)
            {
                return Outcome::unknown(format!("{}: 作れない: {e}", parent.display()));
            }
            if let Err(e) = fs::write(&path, bytes) {
                return Outcome::unknown(format!("{}: 書けない: {e}", path.display()));
            }
            count += 1;
            total += bytes.len();
        }
    }
    Outcome {
        verdict: Verdict::Pass,
        stdout: Some(format!(
            "folio ceiling: 束を組んだ（観点 {}・file {count}・{total} byte・落とした節 {dropped}・正本に無い節 {absent}）",
            bundles.len()
        )),
        stderr: None,
    }
}

#[cfg(test)]
mod bundle_tests {
    use crate::ceiling_src::FACE_NAMES;

    /// 読む文書の id（`ceiling::DOCUMENT_IDS`）と面の名の形の表の id は同じ集合で、どちらにも重複が無い（便 102 §1 (g)3）。
    #[test]
    fn f102_the_face_name_table_covers_every_document_id() {
        use std::collections::BTreeSet;
        let docs: BTreeSet<&str> = crate::ceiling::DOCUMENT_IDS.into_iter().collect();
        let faces: BTreeSet<&str> = FACE_NAMES.iter().map(|(id, _)| *id).collect();
        assert_eq!(docs.len(), crate::ceiling::DOCUMENT_IDS.len(), "DOCUMENT_IDS に重複");
        assert_eq!(faces.len(), FACE_NAMES.len(), "FACE_NAMES に重複");
        assert_eq!(docs, faces);
    }
}
