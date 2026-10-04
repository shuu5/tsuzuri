//! 席の手元の要の写し（tsuzuri の判断の記録 ADR-38 の決定 (5)(11)・行 v-brief-const）: 宣言の任意 key `seat-constitution` が名乗る
//! file を SessionStart の brief が字のまま出し、その後に役割の行（[`super::role_lines`]）を出す。key の無い宣言の席は雛形の 12 行の
//! まま（ADR-0045・ADR-0046 の決めを受け直した形）。写しか宣言を読めない周は、写しの代わりに何を読めなかったかと次の 1 手を名指す
//! 1 行を出し、憲法の 5 行へ黙って戻さない（tsuzuri の条 P-7）。写しは anchor の作業ツリーの file を読む（導く道具が書く場所）。

use super::role_lines;
use crate::name::NAME;
use crate::pipe::declaration::SeatConstitution;
use std::path::Path;

/// 写しを導き直す次の 1 手（tsuzuri の道具の口・写しを導く行 t-seatcopy）。
pub const NEXT_STEP: &str = "tz derive --write";

/// brief の出し分け（閉じた 3 値）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Copy {
    /// key の無い宣言（雛形の 12 行のまま）。
    Absent,
    /// 写しを読めた（file の字のまま）。
    Read(String),
    /// 写しか宣言を読めない（写しの代わりに出す 1 行）。
    Refused(String),
}

impl Copy {
    /// HEAD の宣言の読みから写しを読む（`root` は anchor・path は宣言の repo 相対の字）。
    pub fn read(root: &Path, declared: &SeatConstitution) -> Self {
        match declared {
            SeatConstitution::Absent => Self::Absent,
            SeatConstitution::Unreadable => Self::Refused(format!(
                "{NAME}: 要の写しを出せない reason=declaration-unreadable（HEAD の .vessel.toml を読めず seat-constitution の有無が分からない・憲法の 5 行は出さない）"
            )),
            SeatConstitution::Declared(path) => match std::fs::read_to_string(root.join(path)) {
                Ok(text) => Self::Read(text),
                Err(err) => Self::Refused(format!(
                    "{NAME}: 要の写しを読めない path={path} reason={}（次の 1 手: {NEXT_STEP}・憲法の 5 行は出さない）",
                    reason_of(&err)
                )),
            },
        }
    }

    /// brief の行（`brief` は穴を埋めた雛形の 12 行）: key の無い席は 12 行のまま、名乗った席は写しの字（読めない周は断りの 1 行）の後ろに
    /// 役割の行。
    pub fn lines(&self, brief: &str) -> Vec<String> {
        match self.parts(brief) {
            None => brief.lines().map(str::to_owned).collect(),
            Some((head, role)) => head.into_iter().chain(role).collect(),
        }
    }

    /// 名乗った席の写しの行（読めない周は断りの 1 行）と役割の行の組（記録を分ける読み手は [`super::meter`]・key の無い席は `None`）。
    pub fn parts(&self, brief: &str) -> Option<(Vec<String>, Vec<String>)> {
        let head: Vec<String> = match self {
            Self::Absent => return None,
            Self::Read(text) => verbatim(text),
            Self::Refused(line) => vec![line.clone()],
        };
        Some((head, role_lines(brief).map(str::to_owned).collect()))
    }
}

/// 写しの字を行に割る（出力層が行ごとに改行 1 つを足すので、末尾の改行 1 つだけを外す・行の中の字と `\r` は替えない・空の file は 0 行）。
fn verbatim(text: &str) -> Vec<String> {
    if text.is_empty() {
        return Vec::new();
    }
    text.strip_suffix('\n').unwrap_or(text).split('\n').map(str::to_owned).collect()
}

/// 読めない理由の 1 語（閉じた 3 語）。
fn reason_of(err: &std::io::Error) -> &'static str {
    match err.kind() {
        std::io::ErrorKind::NotFound => "not-found",
        std::io::ErrorKind::InvalidData => "not-utf8",
        _ => "unreadable",
    }
}

#[cfg(test)]
mod tests {
    use super::{Copy, NEXT_STEP};
    use crate::pipe::declaration::SeatConstitution;
    use crate::pipe::fixture::scratch;
    use crate::seat::brief::template;
    use crate::seat::role::Role;

    /// 写しの見本（多字節・空白の並び・末尾の改行）。
    const SAMPLE: &str = "生成物・手で直さない・design-intent/constitution.yaml v9.9\n順位 甲  乙\n全文 contracts/seat/constitution.txt\n";

    /// 雛形の 12 行の 4〜8 行目（憲法の 5 行）。
    fn five() -> Vec<String> {
        template(Role::Orchestrator).lines().skip(3).take(5).map(str::to_owned).collect()
    }

    /// 雛形の 12 行から 5 行を除いた 7 行。
    fn seven() -> Vec<String> {
        let all: Vec<&str> = template(Role::Orchestrator).lines().collect();
        all.iter().enumerate().filter(|(at, _)| !(3..8).contains(at)).map(|(_, line)| (*line).to_owned()).collect()
    }

    /// key の無い宣言は写しを読まず、brief は雛形の 12 行のまま（5 行を含む）。
    #[test]
    fn vbconst_absent_key_keeps_the_twelve_lines() {
        let root = scratch("vbconst-absent");
        std::fs::create_dir_all(root.join("contracts/seat")).expect("置き場");
        std::fs::write(root.join("contracts/seat/brief.txt"), SAMPLE).expect("写し");
        let copy = Copy::read(&root, &SeatConstitution::Absent);
        assert_eq!(copy, Copy::Absent, "key が無ければ写しを読まない");
        let brief = template(Role::Orchestrator);
        let lines = copy.lines(brief);
        assert_eq!(lines, brief.lines().map(str::to_owned).collect::<Vec<String>>(), "12 行のまま");
        assert!(five().iter().all(|line| lines.contains(line)), "5 行を出す");
    }

    /// 名乗った写しは字のまま（末尾の改行まで byte で同じ）先に出て、その後ろに役割の 7 行が並び、5 行は 1 行も出ない。
    #[test]
    fn vbconst_copy_is_printed_verbatim_before_the_role_lines() {
        let root = scratch("vbconst-read");
        std::fs::create_dir_all(root.join("contracts/seat")).expect("置き場");
        std::fs::write(root.join("contracts/seat/brief.txt"), SAMPLE).expect("写し");
        let copy = Copy::read(&root, &SeatConstitution::Declared("contracts/seat/brief.txt".to_owned()));
        assert_eq!(copy, Copy::Read(SAMPLE.to_owned()), "file の字のまま");
        let lines = copy.lines(template(Role::Orchestrator));
        let printed: String = lines.iter().map(|line| format!("{line}\n")).collect();
        assert!(printed.starts_with(SAMPLE), "写しの byte が頭に並ぶ: {printed}");
        assert_eq!(lines.get(3..).map(<[String]>::to_vec), Some(seven()), "写しの 3 行の後ろは役割の 7 行");
        assert!(!five().iter().any(|line| lines.contains(line)), "5 行を出さない");
    }

    /// 名乗った写しが無い周は、写しの代わりに path と理由と次の 1 手を名指す 1 行を出し、その後ろは役割の 7 行で、5 行へ戻さない。
    #[test]
    fn vbconst_missing_copy_names_the_path_and_the_next_step() {
        let root = scratch("vbconst-missing");
        let copy = Copy::read(&root, &SeatConstitution::Declared("contracts/seat/brief.txt".to_owned()));
        let Copy::Refused(line) = &copy else {
            panic!("読めない周は断りの 1 行: {copy:?}");
        };
        assert!(line.contains("path=contracts/seat/brief.txt reason=not-found"), "{line}");
        assert!(line.contains(NEXT_STEP), "次の 1 手を名指す: {line}");
        let lines = copy.lines(template(Role::Orchestrator));
        assert_eq!(lines.first(), Some(line), "断りの 1 行が頭");
        assert_eq!(lines.get(1..).map(<[String]>::to_vec), Some(seven()), "その後ろは役割の 7 行");
        assert!(!five().iter().any(|five| lines.contains(five)), "5 行へ黙って戻さない");
    }

    /// 宣言を読めない周は 12 行に倒さず、宣言を名指す 1 行と役割の 7 行（5 行は出さない）。
    #[test]
    fn vbconst_unreadable_declaration_is_not_folded_into_twelve_lines() {
        let root = scratch("vbconst-unreadable");
        let copy = Copy::read(&root, &SeatConstitution::Unreadable);
        let Copy::Refused(line) = &copy else {
            panic!("宣言を読めない周は断りの 1 行: {copy:?}");
        };
        assert!(line.contains("reason=declaration-unreadable") && line.contains(".vessel.toml"), "{line}");
        let lines = copy.lines(template(Role::Orchestrator));
        assert_eq!(lines.len(), 8, "断りの 1 行と役割の 7 行: {lines:?}");
        assert_eq!(lines.get(1..).map(<[String]>::to_vec), Some(seven()));
    }
}
