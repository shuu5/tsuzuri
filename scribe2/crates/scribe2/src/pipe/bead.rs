//! 契約を台帳の bead に置く形の共有の口（tsuzuri の判断の記録 ADR-72 の決定 (2)・裁定 t3-hub.92.7.2・契約表の行 v-bead-intake）。
//!
//! 契約の正本が台帳の bead 1 本（欄 acceptance に契約表の導出の形の `[[contract]]` の 1 行・本文に設計の節）の形を、受付と
//! preflight と列が**同じ 1 本**で読む（C2）: 形の判じ（[`form_of`]）・写しの字と行の読み（[`copy_text`] / [`row_of`]）・写しの
//! 名の鍵（[`digest`]）・置き場の写しの path と pointer（[`copy_path`] / [`copy_pointer`] / [`digest_of_design`]）。写しは**中身で名が
//! 決まる**ので、bead が後で替わっても前の便の写しは替わらない。判定の関数と表の検査はここに持たず、行の本文の出所だけを替える。

use super::dispatch::pointer_of;
use super::row_review::row_digest;
use super::table::{self, ContractRow, Pointer, TableError, WHOLE_HEAD};
use std::path::{Path, PathBuf};

/// bead の形の契約の行の見出し（acceptance にこの行がちょうど 1 行在る）。
pub const CONTRACT_HEAD: &str = "[[contract]]";

/// 置き場の下の写しの dir の名（`<置き場>/bead-contracts/<bead>/<digest>.toml`）。
pub const COPY_DIR: &str = "bead-contracts";

/// 契約表の pointer の行の鍵（列の読み手 [`pointer_of`] と同じ字面）。
const DESIGN_KEY: &str = "design = ";

/// 写しの字を表の読み手に渡す名（置き場に在る file の名ではなく、読みの理由に出る名）。
const COPY_NAME: &str = "bead.toml";

/// 写しの字で器が足す欄の名（acceptance が持てば bead の形でない）。
const OWN_FIELDS: [&str; 3] = ["section", "goal", "depends"];

/// bead の acceptance が名乗る契約の形（閉じた 4 値）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Form {
    /// 契約表の pointer の行（`design = `）だけを持ち、pointer が解ける（今の形・並ぶ間の 2 形のひとつ）。
    Design(Pointer),
    /// `[[contract]]` の行だけを持つ。
    Bead,
    /// 両方の形を持つ（`design = ` の行の pointer が解けなくても）。
    Both,
    /// どちらも持たない（pointer の解けない `design = ` の行だけの字も）。
    Neither,
}

/// acceptance の形を判じる。前後の空白を除いた行がちょうど [`CONTRACT_HEAD`] の行が bead の形、行頭が `design = ` の行が design の形
/// （鍵は列の [`pointer_of`] と同じ）。design の行だけで pointer が解けなければ [`Form::Neither`]（今の読みと同じ極性）。
pub fn form_of(acceptance: &str) -> Form {
    let lines = || acceptance.lines().map(str::trim);
    let bead = lines().any(|line| line == CONTRACT_HEAD);
    let design = lines().any(|line| line.starts_with(DESIGN_KEY));
    match (bead, design) {
        (true, true) => Form::Both,
        (true, false) => Form::Bead,
        (false, true) => pointer_of(acceptance).map_or(Form::Neither, Form::Design),
        (false, false) => Form::Neither,
    }
}

/// 写しの名の鍵（16 桁）。引数の順（acceptance が先・本文が後）をここ 1 か所に置き、写しの名と列外の鍵と事前審査が同じ値を得る。
pub fn digest(acceptance: &str, description: &str) -> String {
    row_digest(acceptance, description)
}

/// 写しの字と行を組む（[`copy_text`] と [`row_of`] の本体）。断りの句の順は本文の字の照らし・本文の空・欄の照らし・行の読み・行の数。
fn built(bead: &str, acceptance: &str, description: &str) -> Result<(String, ContractRow), String> {
    let lines: Vec<&str> = description.lines().map(str::trim).filter(|line| !line.is_empty()).collect();
    let goal = lines.join(" ");
    if goal.contains(['"', '\\']) {
        return Err("本文に二重引用符か逆斜線が在る".to_owned());
    }
    if goal.is_empty() {
        return Err("本文が空".to_owned());
    }
    if let Some(name) = written_field(acceptance) {
        return Err(format!("acceptance に欄 {name} が在る"));
    }
    let parts = [WHOLE_HEAD, "", acceptance.trim_end(), &format!("section = \"{bead}\""), &format!("goal = \"{goal}\"")];
    let text = format!("{}\n", parts.join("\n"));
    let mut rows = table::read_rows(COPY_NAME, &text).map_err(|errors| {
        format!("行を読めない（{}）", errors.first().map(TableError::reason).unwrap_or_default())
    })?;
    let count = rows.len();
    match rows.pop() {
        Some(row) if count == 1 => Ok((text, row)),
        _ => Err(format!("{CONTRACT_HEAD} の行が {count} 本")),
    }
}

/// acceptance が書いた欄のうち器が足す欄（section・goal・depends）の最初の名（書かれた順）。
fn written_field(acceptance: &str) -> Option<&'static str> {
    acceptance.lines().find_map(|line| {
        let key = line.split_once('=')?.0.trim();
        OWN_FIELDS.iter().find(|name| **name == key).copied()
    })
}

/// bead の写しの字（`schema = 1`・空行・acceptance の末の空白と改行を除いた字・`section = "<bead>"`・`goal = "<本文の 1 行>"` を改行で
/// 繋ぎ、末に改行 1 つ）。本文の 1 行は description の各行の前後の空白を除き、空の行を落として空白 1 つで繋いだ字（tz derive の goal と同じ）。
/// Err は理由の句 1 つ。
pub fn copy_text(bead: &str, acceptance: &str, description: &str) -> Result<String, String> {
    built(bead, acceptance, description).map(|(text, _)| text)
}

/// 写しの字を表の読み手で読んだ行（欄 `goal` に本文の 1 行・欄 `section` に bead の id・Err は [`copy_text`] と同じ句）。
pub fn row_of(bead: &str, acceptance: &str, description: &str) -> Result<ContractRow, String> {
    built(bead, acceptance, description).map(|(_, row)| row)
}

/// 置き場の写しの path（`<置き場>/bead-contracts/<bead>/<digest>.toml`・[`digest`] の値を名にする）。
pub fn copy_path(state_dir: &Path, bead: &str, digest: &str) -> PathBuf {
    state_dir.join(COPY_DIR).join(bead).join(format!("{digest}.toml"))
}

/// 契約の design の pointer の字が写しの path の形（`…/bead-contracts/<bead>/<digest>.toml#<行 id>`）なら digest（16 桁）。ほかは `None`。
pub fn digest_of_design(design: &str) -> Option<String> {
    let pointer = table::parse_pointer(design).ok()?;
    let path = Path::new(&pointer.path);
    let stem = path.file_stem()?.to_str()?;
    let dir = path.parent()?.parent()?.file_name()?;
    let named = path.extension().is_some_and(|ext| ext == "toml") && stem.len() == 16 && stem.chars().all(|c| c.is_ascii_hexdigit());
    (named && dir == COPY_DIR).then(|| stem.to_owned())
}

/// 写しの pointer（写しの絶対 path と井桁と行の id）。**写しを書かない純な関数**で、受付と事前審査が同じ 1 本で組む。
/// Err は [`row_of`] の理由の句か pointer の読みの理由の字。
pub fn copy_pointer(state_dir: &Path, bead: &str, acceptance: &str, description: &str) -> Result<Pointer, String> {
    let row = row_of(bead, acceptance, description)?;
    let path = copy_path(state_dir, bead, &digest(acceptance, description));
    table::parse_pointer(&format!("{}#{}", path.display(), row.id)).map_err(|error| error.reason().to_owned())
}

#[cfg(test)]
mod tests {
    use super::{copy_path, copy_pointer, copy_text, digest, digest_of_design, form_of, row_of, Form};
    use crate::pipe::row_review::row_digest;
    use crate::pipe::table::Pointer;
    use crate::seat::ledger::issues_of;
    use std::path::Path;

    /// 行 b の導出の形（欄 section の行を除く）。
    const ROW_B: &str = "[[contract]]\nid = \"b\"\ntitle = \"行 b\"\nreq = [\"FR1\"]\nverify = [\"cargo nextest run -p toy --no-tests=fail derive_\"]\nsize = \"S\"\ndone = \"b が通る\"\n";

    /// 本文の見本（空白 2 つ・空の行・タブ・CR と LF の区切り・末に改行無し）。
    const BODY: &str = "  第一の段。\r\n\r\n\t- 二つ目 ab\r\n末";

    /// 4 つの形を 1 つずつ（design の行の pointer が解けない字は Neither・両方は pointer が解けなくても Both）。
    #[test]
    fn vbin_form_of_names_the_four_forms() {
        let pointer = Pointer { path: "docs/design/toy.md".to_owned(), id: "a".to_owned() };
        assert_eq!(form_of("先の行\n  design = docs/design/toy.md#a  \n"), Form::Design(pointer));
        assert_eq!(form_of("先の行\n \t[[contract]]  \nid = \"b\"\n"), Form::Bead);
        assert_eq!(form_of("[[contract]]\ndesign = docs/design/toy.md#a\n"), Form::Both);
        assert_eq!(form_of("[[contract]]\ndesign = not a pointer\n"), Form::Both, "pointer が解けなくても両方");
        assert_eq!(form_of("先の行だけ\n"), Form::Neither);
        assert_eq!(form_of(""), Form::Neither);
        assert_eq!(form_of("design = not a pointer\n"), Form::Neither, "pointer の解けない design の行だけは Neither");
        assert_eq!(form_of("design =docs/design/toy.md#a\n"), Form::Neither, "design = の後の空白が無い行は design の行でない");
        assert_eq!(form_of("x [[contract]]\n"), Form::Neither, "行がちょうど [[contract]] でない");
    }

    /// 節 30 の見本: 写しの字・goal の値・行 b と section s2-b・digest の鍵・写しの pointer と digest_of_design。
    #[test]
    fn vbin_copy_text_joins_the_body_like_derive() {
        let text = copy_text("s2-b", ROW_B, BODY).unwrap_or_default();
        let want = format!("schema = 1\n\n{}section = \"s2-b\"\ngoal = \"第一の段。 - 二つ目 ab 末\"\n", ROW_B.trim_end().to_owned() + "\n");
        assert_eq!(text, want);
        let row = row_of("s2-b", ROW_B, BODY);
        assert!(row.as_ref().is_ok_and(|found| found.id == "b" && found.section == "s2-b" && found.goal == "第一の段。 - 二つ目 ab 末"), "{row:?}");
        let key = digest(ROW_B, BODY);
        assert_eq!(key, row_digest(ROW_B, BODY), "row_digest に acceptance と本文をこの順に渡した値");
        assert_ne!(key, digest(ROW_B, &format!("{BODY}。")), "本文の 1 byte で替わる");
        assert_ne!(key, digest(&format!("{ROW_B} "), BODY), "acceptance の 1 byte で替わる");
        assert_ne!(key, digest(BODY, ROW_B), "引数の順で替わる");
        let pointer = copy_pointer(Path::new("/s"), "s2-b", ROW_B, BODY);
        let path = copy_path(Path::new("/s"), "s2-b", &key);
        assert_eq!(path.display().to_string(), format!("/s/bead-contracts/s2-b/{key}.toml"));
        assert!(pointer.as_ref().is_ok_and(|found| found.path == path.display().to_string() && found.id == "b"), "{pointer:?}");
        assert!(!path.exists(), "写しを書かない");
        let design = format!("{}#b", path.display());
        assert_eq!(digest_of_design(&design), Some(key.clone()));
        assert_eq!(digest_of_design("docs/design/toy.md#b"), None);
        assert_eq!(digest_of_design(&format!("/s/other/s2-b/{key}.toml#b")), None, "写しの dir の名でない");
        assert_eq!(digest_of_design(&format!("/s/bead-contracts/s2-b/{}.toml#b", "z".repeat(16))), None, "16 桁の 16 進でない");
    }

    /// 書けない周の理由の句（本文の字の照らし・本文の空・欄の照らし・行の読み・行の数の順）。
    #[test]
    fn vbin_copy_text_refuses_what_it_cannot_write() {
        let refused = |acceptance: &str, body: &str| copy_text("s2-b", acceptance, body).err().unwrap_or_default();
        assert_eq!(refused(ROW_B, "二重\"引用符"), "本文に二重引用符か逆斜線が在る");
        assert_eq!(refused(ROW_B, "逆斜線\\"), "本文に二重引用符か逆斜線が在る");
        assert_eq!(refused(ROW_B, " \n\t\r\n "), "本文が空");
        for name in ["section", "goal", "depends"] {
            assert_eq!(refused(&format!("{ROW_B}{name} = \"x\"\n"), "本文。"), format!("acceptance に欄 {name} が在る"));
        }
        assert_eq!(refused(&format!("{ROW_B}depends = [\"a\"]\nsection = \"1\"\n"), "本文。"), "acceptance に欄 depends が在る", "書かれた最初の名");
        assert_eq!(refused(ROW_B, "\"本文\"\\"), "本文に二重引用符か逆斜線が在る", "本文の照らしが先");
        assert_eq!(refused(&format!("{ROW_B}goal = \"x\"\n"), " "), "本文が空", "本文の空が欄の照らしより先");
        let unread = refused("[[contract]]\nid = \"b\"\n", "本文。");
        assert!(unread.starts_with("行を読めない（") && unread.ends_with('）'), "{unread}");
        let two = refused(&format!("{ROW_B}{ROW_B}"), "本文。");
        assert!(two.starts_with("行を読めない（") && two.ends_with('）'), "器が足す欄は最後の行にしか付かず、行の読みが行の数より先: {two}");
        for none in ["", "見出し無し\nid = \"b\"\n"] {
            let zero = refused(none, "本文。");
            assert!(zero.starts_with("行を読めない（") && zero.ends_with('）'), "見出しが 0 本でも行の読みが先: {zero}");
        }
    }

    /// 大きさの見本の JSON の字（escape 5 種）を台帳の読み手で読むと、byte で一致し digest も一致する。
    #[test]
    fn vbin_cap_sized_bead_reads_back_byte_equal() {
        let tile = "あa\"\\\t\r\n";
        assert_eq!(tile.len(), 9);
        let sized = |bytes: usize| {
            let mut text = tile.repeat(bytes / tile.len() + 1);
            text.truncate(bytes);
            text
        };
        let (description, acceptance) = (sized(65536), sized(24576));
        assert_eq!((description.len(), acceptance.len()), (65536, 24576));
        let escaped = |text: &str| {
            text.chars().fold(String::new(), |mut out, c| {
                match c {
                    '"' => out.push_str("\\\""),
                    '\\' => out.push_str("\\\\"),
                    '\t' => out.push_str("\\t"),
                    '\r' => out.push_str("\\r"),
                    '\n' => out.push_str("\\n"),
                    other => out.push(other),
                }
                out
            })
        };
        let json = format!(
            "[{{\"id\":\"s2-b\",\"status\":\"open\",\"acceptance_criteria\":\"{}\",\"description\":\"{}\"}}]",
            escaped(&acceptance),
            escaped(&description)
        );
        let issues = issues_of(&json).unwrap_or_default();
        assert_eq!(issues.len(), 1, "1 本");
        let read = issues.first().map(|issue| (issue.description.clone(), issue.acceptance.clone()));
        assert_eq!(read, Some((description.clone(), acceptance.clone())), "byte で一致");
        assert_eq!(read.map(|(body, accept)| digest(&accept, &body)), Some(digest(&acceptance, &description)));
    }
}
