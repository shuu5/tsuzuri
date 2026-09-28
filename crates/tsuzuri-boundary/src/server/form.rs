//! 器の doctor の台帳の形の行（行 c-pipe-misfit・判断の記録 ADR-16 の決定 (6)）。
//! 器の doctor は引数 --repo を受けた周だけ台帳の形の行（頭 `tsuzuri_core::pipeline::FORM_PREFIX`）を出す。
//! 撃つ形は `<program> doctor --state-dir <dir> --repo .`（cwd は repo の置き場・標準入力は空・標準エラーは捨てる）。
//! 撃つのは台帳の見張りの読みの周だけ（`Source::read` の後）で、口 /api/pipeline の最初の要求の後から撃つ（`Form::arm`）。
//! 口は持った字を読むだけで器を撃たず、席の card には触らない。
//! 撃ちは別の thread で同時に 1 本だけで、走っている間の読みは終わった後にもう 1 回だけ撃つ。
//! 持つ字は最後に終えた撃ちの台帳の形の行で、落ちるか上限を越えた周は None（持ち回さない）。字が変われば board-changed を送る。

use std::ffi::OsString;
use std::fmt;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock, Weak};
use std::thread;
use std::time::Duration;

use tsuzuri_core::pipeline::FORM_PREFIX;

use super::events::{Hub, now};
use super::proc::capture;

/// 器の CLI に渡す頭の引数。
pub const FORM_ARGS: [&str; 1] = ["doctor"];

/// 台帳の形の行を出させる引数（子 process の cwd が repo の置き場なので字 . がその置き場を指し、
/// 相対の置き場でも二重にならない）。
pub const REPO_ARGS: [&str; 2] = ["--repo", "."];

/// 撃ちの上限。doctor --state-dir と --repo . の組を測った最長 15.76 秒（2026-09-28）の約 2 倍で、
/// 器の中の台帳の読みの上限（seat.ledger_timeout_s の 60 秒）と `events::STORE_REREAD`（60 秒）の半分。
pub const FORM_TIMEOUT: Duration = Duration::from_secs(30);

/// 器の出力のうち頭が `FORM_PREFIX` の行だけを改行でつないだ字（無ければ空の字）。
pub fn form_lines(out: &str) -> String {
    out.lines()
        .filter(|l| l.starts_with(FORM_PREFIX))
        .collect::<Vec<_>>()
        .join("\n")
}

/// 撃ちの状態（走っているか・走っている間の読みが在ったか）。
#[derive(Default)]
struct Run {
    running: bool,
    again: bool,
}

struct Inner {
    program: OsString,
    state_dir: PathBuf,
    cwd: PathBuf,
    timeout: Duration,
    /// 真なら見張りの読みの周に撃つ（口 /api/pipeline の最初の要求が置く）。
    armed: AtomicBool,
    run: Mutex<Run>,
    /// 最後に終えた撃ちの台帳の形の行（落ちたか上限を越えた周は None）。
    text: Mutex<Option<String>>,
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
                text: Mutex::new(None),
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

    /// 持つ字の複製（撃たない）。
    pub fn text(&self) -> Option<String> {
        lock(&self.inner.text).clone()
    }

    /// 許されていれば別の thread で器を撃つ（待たない）。走っていれば、終わった後にもう 1 回だけ撃つ。
    pub fn kick(&self) {
        if !self.armed() {
            return;
        }
        {
            let mut run = lock(&self.inner.run);
            if run.running {
                run.again = true;
                return;
            }
            run.running = true;
        }
        let inner = Arc::clone(&self.inner);
        thread::spawn(move || {
            loop {
                inner.shoot();
                let mut run = lock(&inner.run);
                if run.again {
                    run.again = false;
                    continue;
                }
                run.running = false;
                return;
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

    /// 器を 1 度撃ち、台帳の形の行を持つ字に置く（変われば board-changed を送る）。
    fn shoot(&self) {
        let got = capture(&self.program, self.argv(), &self.cwd, self.timeout)
            .and_then(|out| String::from_utf8(out).ok())
            .map(|out| form_lines(&out));
        {
            let mut text = lock(&self.text);
            if *text == got {
                return;
            }
            *text = got;
        }
        if let Some(hub) = self.hub.get().and_then(Weak::upgrade) {
            hub.board_changed(now());
        }
    }
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}
