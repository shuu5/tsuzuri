//! 器の doctor の台帳の形の行（判断の記録 ADR-16 の決定 (6)）。
//! 器の doctor は引数 --repo を受けた周だけ台帳の形の行（頭 `tsuzuri_core::pipeline::FORM_PREFIX`）を出す。
//! 撃つ形は `<program> doctor --state-dir <dir> --repo .`（cwd は repo の置き場・標準入力は空・標準エラーは捨てる）。
//! 撃つのは台帳の見張りの読みの周だけ（`Source::read` がその周の台帳の字を渡す）で、口 /api/pipeline の最初の要求か
//! 知らせの接続（GET /api/surface/events）が撃ちを許した後から撃つ（`Form::arm`）。
//! 口は持った組を読むだけで器を撃たず、席の card には触らない。
//! 撃ちは別の thread で同時に 1 本だけで、走っている間の読みは終わった後に最後の周の台帳の字でもう 1 回だけ撃つ。
//! 持つ組（`Kept`）は最後に終えた撃ちの周の台帳の字と台帳の形の行で、台帳の読みが落ちた周は器を撃たずに None、
//! 器が落ちるか上限を越えるか台帳の形の行が無い周も None（持ち回さない）。組から写した一覧（`misfits`）が変われば
//! 台帳の種類（`ChangeKind::Ledger`・台帳の見張りの読みの周の字から撃つ）の board-changed を送る
//! （台帳の字だけが変わった周は見張りの ledger-changed が面に読み直させる）。

use std::ffi::OsString;
use std::fmt;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock, Weak};
use std::thread;
use std::time::Duration;

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::{MisfitBead, Reading};
use tsuzuri_contract::surface::ChangeKind;
use tsuzuri_core::pipeline::{FORM_PREFIX, board_with_doctor};

use super::events::{Hub, now};
use super::proc::capture;

/// 器の CLI に渡す頭の引数。
pub const FORM_ARGS: [&str; 1] = ["doctor"];

/// 台帳の形の行を出させる引数（子 process の cwd が repo の置き場なので字 . がその置き場を指し、
/// 相対の置き場でも二重にならない）。
pub const REPO_ARGS: [&str; 2] = ["--repo", "."];

/// 撃ちの上限。doctor --state-dir と --repo . の組を測った最長 15.76 秒（2026-09-28）の約 2 倍で、
/// 器の中の台帳の読みの上限（seat.ledger_timeout_s の 60 秒）の半分。
pub const FORM_TIMEOUT: Duration = Duration::from_secs(30);

/// 器の出力のうち頭が `FORM_PREFIX` の行だけを改行でつないだ字（無ければ空の字）。
pub fn form_lines(out: &str) -> String {
    out.lines()
        .filter(|l| l.starts_with(FORM_PREFIX))
        .collect::<Vec<_>>()
        .join("\n")
}

/// 撃った周の組（見張りの読みの台帳の字と、その周の器の出力の台帳の形の行・空でない）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Kept {
    pub ledger: String,
    pub form: String,
}

/// 組から写した形の崩れの一覧（組が無ければ Unknown・一覧は event log の字を読まないので空の字を渡す）。
pub fn misfits(kept: Option<&Kept>, now: EpochSecs) -> Reading<Vec<MisfitBead>> {
    match kept {
        Some(kept) => {
            board_with_doctor(&kept.ledger, "", Some(&kept.form), now)
                .board
                .misfits
        }
        None => Reading::Unknown,
    }
}

/// 撃ちの状態（走っているか・走っている間の最後の周の台帳の字〔読めなかった周は内の None〕）。
#[derive(Default)]
struct Run {
    running: bool,
    again: Option<Option<String>>,
}

struct Inner {
    program: OsString,
    state_dir: PathBuf,
    cwd: PathBuf,
    timeout: Duration,
    /// 真なら見張りの読みの周に撃つ（口 /api/pipeline の最初の要求と知らせの接続が置く）。
    armed: AtomicBool,
    run: Mutex<Run>,
    /// 最後に終えた撃ちの組（台帳の読みか器が落ちたか、上限を越えたか、台帳の形の行が無い周は None）。
    kept: Mutex<Option<Kept>>,
    /// 板の変化の送り先。
    hub: OnceLock<Weak<Hub>>,
}

/// 器の doctor の台帳の形の行の撃ちと、持つ字（clone は同じ撃ちと字を分け合う）。
#[derive(Clone)]
pub struct Form {
    inner: Arc<Inner>,
}

impl fmt::Debug for Form {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Form")
            .field("program", &self.inner.program)
            .field("state_dir", &self.inner.state_dir)
            .field("cwd", &self.inner.cwd)
            .field("armed", &self.armed())
            .finish()
    }
}

impl Form {
    /// 器の CLI の program・state dir・cwd（repo の置き場）を受ける（上限は `FORM_TIMEOUT`・許しは偽）。
    pub fn new(
        program: impl Into<OsString>,
        state_dir: impl Into<PathBuf>,
        cwd: impl Into<PathBuf>,
    ) -> Form {
        Form::with_timeout(program, state_dir, cwd, FORM_TIMEOUT)
    }

    /// `new` と同じで、上限を受ける。
    pub fn with_timeout(
        program: impl Into<OsString>,
        state_dir: impl Into<PathBuf>,
        cwd: impl Into<PathBuf>,
        timeout: Duration,
    ) -> Form {
        Form {
            inner: Arc::new(Inner {
                program: program.into(),
                state_dir: state_dir.into(),
                cwd: cwd.into(),
                timeout,
                armed: AtomicBool::new(false),
                run: Mutex::new(Run::default()),
                kept: Mutex::new(None),
                hub: OnceLock::new(),
            }),
        }
    }

    /// 器に渡す引数の列（`FORM_ARGS`・--state-dir と state dir・`REPO_ARGS`）。
    pub fn argv(&self) -> Vec<OsString> {
        self.inner.argv()
    }

    /// 以後の `kick` を許す（撃たない）。
    pub fn arm(&self) {
        self.inner.armed.store(true, Ordering::SeqCst);
    }

    /// 許されているか。
    pub fn armed(&self) -> bool {
        self.inner.armed.load(Ordering::SeqCst)
    }

    /// 板の変化の送り先を置く（弱い参照・1 度だけ）。
    pub fn notify(&self, hub: &Arc<Hub>) {
        let _ = self.inner.hub.set(Arc::downgrade(hub));
    }

    /// 持つ組の複製（撃たない）。
    pub fn kept(&self) -> Option<Kept> {
        lock(&self.inner.kept).clone()
    }

    /// 許されていれば別の thread で見張りの読みの台帳の字（読めなかった周は None）の周を撃つ（待たない）。
    /// 走っていればその字を置いて戻り（前に置いた字は捨てる）、終わった後に最後に置いた字でもう 1 回だけ撃つ。
    pub fn kick(&self, ledger: Option<String>) {
        if !self.armed() {
            return;
        }
        {
            let mut run = lock(&self.inner.run);
            if run.running {
                run.again = Some(ledger);
                return;
            }
            run.running = true;
        }
        let inner = Arc::clone(&self.inner);
        thread::spawn(move || {
            let mut ledger = ledger;
            loop {
                inner.shoot(ledger);
                let mut run = lock(&inner.run);
                match run.again.take() {
                    Some(next) => ledger = next,
                    None => {
                        run.running = false;
                        return;
                    }
                }
            }
        });
    }
}

impl Inner {
    fn argv(&self) -> Vec<OsString> {
        let mut argv: Vec<OsString> = FORM_ARGS.iter().map(OsString::from).collect();
        argv.push("--state-dir".into());
        argv.push(self.state_dir.clone().into_os_string());
        argv.extend(REPO_ARGS.iter().map(OsString::from));
        argv
    }

    /// 台帳の字の周を撃ち、組を置く（台帳の字が None なら器を撃たずに None・台帳の形の行が無ければ None）。
    /// 前の組と新しい組から写した一覧が違えば台帳の種類の board-changed を送る。
    fn shoot(&self, ledger: Option<String>) {
        let got = ledger.and_then(|ledger| {
            let form = capture(&self.program, self.argv(), &self.cwd, self.timeout)
                .and_then(|out| String::from_utf8(out).ok())
                .map(|out| form_lines(&out))
                .filter(|form| !form.is_empty())?;
            Some(Kept { ledger, form })
        });
        {
            let mut kept = lock(&self.kept);
            let at = now();
            let changed = misfits(kept.as_ref(), at) != misfits(got.as_ref(), at);
            *kept = got;
            if !changed {
                return;
            }
        }
        if let Some(hub) = self.hub.get().and_then(Weak::upgrade) {
            hub.board_changed(&[ChangeKind::Ledger], now());
        }
    }
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}
