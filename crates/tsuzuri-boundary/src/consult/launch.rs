//! tz consult launch <窓 id> [--again [--session <会話の id>]] [--dry-run]
//! （判断の記録 ADR-29 決定 (3)(5)(6)(7)(10)・受入 AC19）。
//! --follow [--session <会話の id>] [--wait <秒>] は口座の移動に付いて来させる口（`follow`）へ渡す。
//! 窓を起こす口。中核の `consult::launch` で argv と設定と環境を組み、`audit` の欠けが 1 つでも在れば起こさない。
//! 話す窓は席の tmux の session に名 consult-cw<n> の窓を -d で開き（持ち主の見ている窓を替えない）、環境は -e の閉じた
//! 列（`window_env`・`TALK_ENV` と `BASE_ENV`・席の環境に無い名は渡さない）だけを渡し、claude を env -S（`keep_only`）で包んで
//! tmux の server の環境を切る（残すのは -e の名と tmux の置く `PANE_ENV` だけ）。問う窓は席の背景の子として claude -p を
//! cwd = 作業場で撃ち、環境を空にして同じ閉じた列だけを置き、終わりまで待って（上限 `ASK_LIMIT`）、新しい所見ごとに
//! 経路 完了 の固定の 1 行を、無ければ止まった窓の固定の 1 行を標準出力に出す。
//! 起こすごとに process の印 `.consult/proc-<k>.json`（起こした時の口座の置き場を含む）を書き、台帳の根に相談の開きの行（結果 = 開いた か 落ちた・
//! --again は撃ち直しの印）を書く。版のずれ・閉じた窓・規則の行 R-38 の上限（起こし手が席の問う窓だけ）は行を書かずに断る。
//! --dry-run は program の名と argv を 1 行ずつ出して起こさない（台帳も書かない）。
//! 起こす前に state dir の accounts と accounts/.retired の子の symlink の先（口座の置き場の実体）を解き、囲いと読む道具から隠す
//! 材料にする（解けない先は link の字の path で隠し、読めない dir と link は起こさずに断る）。
//! 話す窓の --again は、作業場の会話の印の最後の会話の id を `--resume` で続ける（印を持たない窓は席が --session で名指す・
//! 中核の `resume::pick`）。前の process が生きている話す窓の --again は断る（同じ会話を 2 つが書くと枝が割れる）。
//! 会話を続ける時は、起こす前に環境の口座の置き場の設定 file に作業場 1 つだけの信頼の印を置く（`trust::place_trust`・
//! 置けなければ起こさずに断る・判断の記録 ADR-55 決定 (1)(3)）。
//! 状態の行の再描画の間は、環境の口座の置き場の設定 file の statusLine の値を材料に写す（`statusline::account_settings`・
//! 判断の記録 ADR-67）。

use std::fs::{self, File};
use std::io::Read;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use tsuzuri_contract::consult::{Form, ProcMark, Starter, WindowFile, WindowId};
use tsuzuri_contract::wire;
use tsuzuri_core::consult::launch::{
    Launch, PROGRAM, VERSION_ENV, argv, audit, keep_only, private_tmp, read_roots, version_ok,
    window_env,
};
use tsuzuri_core::consult::lines::{By, Event, Line, WORD_MAX, cited, free, notice};
use tsuzuri_core::consult::quota::{admit, count};
use tsuzuri_core::consult::resume::pick;
use tsuzuri_core::consult::stamp::{last_sid, lines};
use tsuzuri_core::consult::status::account_refresh;

use super::follow;
use super::plain::{create_plain, odd_dir, plain_file, plain_path, same_file, write_plain};
use super::stamp::STAMPS;
use super::statusline::account_settings;
use super::trust::place_trust;
use super::{
    COMMON, Ctx, FAIL, GIT_TIMEOUT, Refused, UNKNOWN, append, ctx, findings, flags, ledger,
    lines_of, live, minute_now, plain_ws, procs, read_window, refuse, tz_path, tzw, windows,
    workspace,
};
use crate::out::{emit, emit_err};
use crate::server::ledger::{KILL, capture, stop_with};

/// tmux の program の名。
pub const TMUX: &str = "tmux";

/// 新しい tmux の窓の id と pane の pid を出させる形。
pub const WINDOW_FORMAT: &str = "#{window_id} #{pane_pid}";

/// 話す窓の claude を包む program の名（`-S` の字で環境を閉じた列に絞る）。
pub const ENV: &str = "env";

/// 問う窓を待つ上限（Bash の道具の背景の上限より短く）。
pub const ASK_LIMIT: Duration = Duration::from_secs(6600);

/// 起こした時の口座の置き場を読む環境変数（process の印に書く・判断の記録 ADR-55 決定 (2)）。
pub const ACCOUNT_ENV: &str = "CLAUDE_CONFIG_DIR";

/// 問う窓の終わりを見る間。
const ASK_STEP: Duration = Duration::from_millis(200);

/// 撃ちの形（撃ち直しか・起こさずに argv だけを出すか・席が名指す会話の id）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shot<'a> {
    pub again: bool,
    pub dry: bool,
    pub session: Option<&'a str>,
}

/// tz consult launch の残りの引数を受けて終了 code を返す。
pub fn run(rest: &[&str]) -> u8 {
    let mut values = COMMON.to_vec();
    values.extend(["--session", "--wait"]);
    let f = match flags(rest, &values, &["--again", "--dry-run", "--follow"], &[]) {
        Ok(f) => f,
        Err(e) => return refuse("launch", e),
    };
    let [id] = f.pos.as_slice() else {
        return refuse("launch", (FAIL, "窓の id を 1 つ渡す".to_string()));
    };
    let Ok(id) = WindowId::parse(id) else {
        return refuse("launch", (FAIL, format!("窓の id {id} の形でない")));
    };
    if !version_ok(std::env::var(VERSION_ENV).ok().as_deref()) {
        let why = format!("{VERSION_ENV} が tz の版と違う（plugin の tz の解き方で撃つ）");
        return refuse("launch", (FAIL, why));
    }
    let shot = Shot {
        again: f.has("--again"),
        dry: f.has("--dry-run"),
        session: f.get("--session"),
    };
    let wait = match (f.has("--follow"), f.get("--wait").map(str::parse::<u64>)) {
        (false, None) => None,
        (true, None) if !shot.again && !shot.dry => Some(follow::WAIT),
        (true, Some(Ok(s))) if s > 0 && !shot.again && !shot.dry => Some(s),
        _ => {
            let why = "--wait は --follow と一緒に 1 以上の秒の数で渡し、--follow は --again と --dry-run と一緒に渡さない";
            return refuse("launch", (FAIL, why.to_string()));
        }
    };
    let made = ctx(&f).and_then(|c| match wait {
        Some(s) => follow::follow(&c, id, shot.session, Duration::from_secs(s)),
        None => launch(&c, id, &shot),
    });
    match made {
        Ok(()) => 0,
        Err(e) => refuse("launch", e),
    }
}

/// 窓の材料（path は「/」で始まる絶対 path・読む根は在る dir だけ・uid は作業場の持ち主）。
pub fn material(c: &Ctx, ws: &Path, w: &WindowFile) -> Result<Launch, Refused> {
    let abs = |p: &Path| p.canonicalize().unwrap_or_else(|_| p.to_path_buf());
    let (repo, state) = (abs(&c.repo), abs(&c.state));
    let (repo, state) = (repo.display().to_string(), state.display().to_string());
    let roots = read_roots(&repo, &state)
        .into_iter()
        .filter(|r| Path::new(r).is_dir())
        .collect();
    let uid = fs::metadata(ws)
        .map_err(|e| (UNKNOWN, format!("作業場が読めない: {e}")))?
        .uid();
    let account_dirs = account_dirs(Path::new(&state))?;
    Ok(Launch {
        form: w.form,
        window: w.id,
        workspace: abs(ws).display().to_string(),
        repo,
        state,
        account_dirs,
        roots,
        tz: tz_path(),
        uid: uid.to_string(),
        model: w.model.clone(),
        effort: w.effort.clone(),
        question: plain_file(ws, "bundle/question.md"),
        resume: None,
        refresh: account_settings().and_then(|t| account_refresh(&t)),
    })
}

/// 会話の印の file を読む上限（byte・越える印は断る）。
pub const STAMPS_MAX: u64 = 1 << 22;

/// 作業場の会話の印の字（印の file が無ければ None）。窓が作業場に置ける symlink と fifo を辿らないよう `plain` で照らし、
/// 途中の段の symlink・普通でない file・上限 `STAMPS_MAX` を越える file・開いた後の照らしの違いは読まずに誤りを返す。
pub fn read_stamps(ws: &Path) -> std::io::Result<Option<String>> {
    let refuse = |why: &str| std::io::Error::other(why.to_string());
    let path =
        plain_path(ws, STAMPS).ok_or_else(|| refuse("途中の段が symlink でない dir でない"))?;
    let before = match fs::symlink_metadata(&path) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e),
    };
    if !before.file_type().is_file() || before.len() > STAMPS_MAX {
        return Err(refuse("普通の file でないか上限を越える"));
    }
    let file = File::open(&path)?;
    if !same_file(&file, &before) {
        return Err(refuse("開いた file が照らした file と違う"));
    }
    let mut text = String::new();
    file.take(STAMPS_MAX).read_to_string(&mut text)?;
    Ok(Some(text))
}

/// 作業場の会話の印の最後の会話の id（印の file が無ければ None・読めなければ断る）。
pub fn last_conversation(ws: &Path) -> Result<Option<String>, Refused> {
    let text =
        read_stamps(ws).map_err(|e| (UNKNOWN, format!("会話の印 {STAMPS} が読めない: {e}")))?;
    Ok(text.and_then(|t| last_sid(&lines(&t)).map(str::to_string)))
}

/// state dir の accounts と accounts/.retired の子の symlink の先（器の口座の置き場の形・解けた先は canonical の path・
/// 解けない先は link の字の path・字の順で重なりなし・accounts が無ければ空・読めない dir と link は断る）。
pub fn account_dirs(state: &Path) -> Result<Vec<String>, Refused> {
    let unreadable = |p: &Path, e: std::io::Error| {
        let why = format!(
            "口座の置き場 {} が読めない: {e}（読める形に直してから起こし直す）",
            p.display()
        );
        (UNKNOWN, why)
    };
    let accounts = state.join("accounts");
    let mut dirs = Vec::new();
    for dir in [accounts.clone(), accounts.join(".retired")] {
        let entries = match fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => return Err(unreadable(&dir, e)),
        };
        for entry in entries {
            let link = entry.map_err(|e| unreadable(&dir, e))?.path();
            let meta = fs::symlink_metadata(&link).map_err(|e| unreadable(&link, e))?;
            if !meta.file_type().is_symlink() {
                continue;
            }
            let target = match fs::canonicalize(&link) {
                Ok(target) => target,
                Err(_) => dir.join(fs::read_link(&link).map_err(|e| unreadable(&link, e))?),
            };
            dirs.push(target.display().to_string());
        }
    }
    dirs.sort();
    dirs.dedup();
    Ok(dirs)
}

/// 窓の起こし手（開きの行の欄）。
fn by_of(w: &WindowFile) -> Result<By, Refused> {
    let missing = || (FAIL, format!("窓 {} の控えに起こし手の添えが無い", w.id));
    Ok(match w.starter {
        Starter::Seat => By::Seat,
        Starter::Chat => By::Chat {
            uttered: w.uttered.clone().ok_or_else(missing)?,
        },
        Starter::Button => By::Button {
            request: w.request.clone().ok_or_else(missing)?,
        },
    })
}

/// 相談の開きの行を根に書く（題は器の引用の形を含めば題なし）。
fn record(
    c: &Ctx,
    items: &[tsuzuri_contract::ledger::LedgerItem],
    w: &WindowFile,
    opened: bool,
    again: bool,
) -> Result<(), Refused> {
    let line = Line::Open {
        window: w.id,
        form: w.form,
        topic: w.topic.clone().filter(|t| !cited(&free(t, WORD_MAX))),
        by: by_of(w)?,
        model: w.model.clone(),
        effort: w.effort.clone(),
        opened,
        again,
        at: minute_now(),
    };
    append(c, items, None, &line).map(|_| ())
}

/// 窓の控えと会話の印の最後の会話の id（会話の印と控えの dir の断りは会話の印の読みの字・rc 2 で先に出し、残りの symlink と
/// fifo は `plain_ws` の 1 行・rc 1）。
fn window_and_last(ws: &Path, id: WindowId) -> Result<(WindowFile, Option<String>), Refused> {
    let w = read_window(ws);
    let last = if w.is_some() || odd_dir(ws, ".consult").is_some() {
        last_conversation(ws)?
    } else {
        None
    };
    plain_ws(ws, id)?;
    let w = w.ok_or((FAIL, format!("窓 {id} の作業場が無い: {}", ws.display())))?;
    Ok((w, last))
}

/// 起こす（版の照らしの後）。
pub fn launch(c: &Ctx, id: WindowId, shot: &Shot) -> Result<(), Refused> {
    let again = shot.again;
    let ws = workspace(&c.drafts, id);
    let (w, last) = window_and_last(&ws, id)?;
    let resume = pick(w.form, again, last.as_deref(), shot.session)
        .map_err(|why| (FAIL, why.to_string()))?;
    let mut l = material(c, &ws, &w)?;
    l.resume = resume;
    let args = argv(&l);
    let gaps = audit(&args, &l);
    if shot.dry {
        emit(PROGRAM);
        args.iter().for_each(|a| emit(a));
        return if gaps.is_empty() {
            Ok(())
        } else {
            Err((FAIL, format!("検めの欠け: {}", gaps.join(" "))))
        };
    }
    let (_, items) = ledger(c)?;
    let lines = lines_of(&items);
    if lines
        .iter()
        .any(|l| matches!(l, Line::Close { window, .. } if *window == id))
    {
        return Err((FAIL, format!("窓 {id} は閉じた")));
    }
    if again && w.form == Form::Talk && live(&ws) {
        let why = format!(
            "話す窓 {id} の前の process が生きている（同じ会話を 2 つが書くと枝が割れる・窓が終わってから撃ち直す）"
        );
        return Err((FAIL, why));
    }
    if w.starter == Starter::Seat && w.form == Form::Ask {
        let live: Vec<WindowId> = windows(&c.drafts)
            .into_iter()
            .filter(|(n, gone)| !gone && live(&workspace(&c.drafts, *n)))
            .map(|(n, _)| n)
            .collect();
        admit(count(&lines, &minute_now(), &live), again).map_err(|why| (FAIL, why.to_string()))?;
    }
    if l.resume.is_some() && gaps.is_empty() {
        let account = std::env::var(ACCOUNT_ENV).map_err(|_| {
            let why = format!("席の環境に {ACCOUNT_ENV} が無い（信頼の印を置く口座が分からない・器が起こした席から撃つ）");
            (FAIL, why)
        })?;
        place_trust(Path::new(&account), &l.workspace)?;
    }
    let started = start(&ws, &l, &args, again, &gaps);
    record(c, &items, &w, started.is_ok(), again)?;
    let mark = started?;
    if w.form == Form::Talk {
        emit(&format!("窓 {id} を開いた（tmux の窓 {}）", id.name()));
        return Ok(());
    }
    wait_ask(&ws, id, mark)
}

/// 検めと私用の temp の後に窓を起こし、process の印を書く。
fn start(
    ws: &Path,
    l: &Launch,
    args: &[String],
    again: bool,
    gaps: &[String],
) -> Result<Started, Refused> {
    if !gaps.is_empty() {
        return Err((
            FAIL,
            format!("検めの欠け（起こさない）: {}", gaps.join(" ")),
        ));
    }
    let uid: u32 = l.uid.parse().map_err(|_| (FAIL, "uid の字".to_string()))?;
    private(&private_tmp(&l.workspace), uid)?;
    let k = procs(ws).last().map_or(0, |p| p.k) + 1;
    let before = findings(ws, l.window);
    let (pid, tmux_window, child) = match l.form {
        Form::Talk => {
            let (window, pane) = talk(ws, l, args)?;
            (pane, Some(window), None)
        }
        Form::Ask => {
            let child = ask(ws, l, args, k)?;
            (child.id(), None, Some(child))
        }
    };
    let mark = ProcMark {
        k,
        form: l.form,
        pid,
        at: minute_now(),
        again,
        tmux_window,
        account: std::env::var(ACCOUNT_ENV).ok(),
    };
    let text = wire::encode(&mark).map_err(|e| (UNKNOWN, e.to_string()))?;
    write_plain(
        ws,
        &format!(".consult/proc-{k}.json"),
        (text + "\n").as_bytes(),
    )
    .map_err(|e| (UNKNOWN, format!("process の印を書けない: {e}")))?;
    Ok(Started { before, child })
}

/// 起こした窓（問う窓は子と起こす前の所見）。
struct Started {
    before: Vec<tsuzuri_contract::consult::FindingId>,
    child: Option<std::process::Child>,
}

/// 私用の temp を mode 0700 で作る（在れば持ち主の uid と 0700 の dir であることを確かめる）。
fn private(path: &str, uid: u32) -> Result<(), Refused> {
    match fs::create_dir(path) {
        Ok(()) => fs::set_permissions(path, fs::Permissions::from_mode(0o700))
            .map_err(|e| (FAIL, format!("私用の temp {path} の権限: {e}"))),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            let m = fs::symlink_metadata(path)
                .map_err(|e| (FAIL, format!("私用の temp {path}: {e}")))?;
            if m.is_dir() && m.uid() == uid && m.mode() & 0o777 == 0o700 {
                Ok(())
            } else {
                Err((
                    FAIL,
                    format!("私用の temp {path} が持ち主の 0700 の dir でない"),
                ))
            }
        }
        Err(e) => Err((FAIL, format!("私用の temp {path} を作れない: {e}"))),
    }
}

/// 話す窓を席の tmux の session に開き、tmux の window id と pane の pid を返す。
fn talk(ws: &Path, l: &Launch, args: &[String]) -> Result<(String, u32), Refused> {
    let pane = std::env::var("TMUX_PANE").map_err(|_| {
        (
            FAIL,
            "席の TMUX_PANE が無い（tmux の中の席から撃つ）".to_string(),
        )
    })?;
    let tmux = std::ffi::OsStr::new(TMUX);
    let session = capture(
        tmux,
        ["display-message", "-p", "-t", &pane, "#{session_name}"],
        ws,
        GIT_TIMEOUT,
    )
    .and_then(|o| String::from_utf8(o).ok())
    .map(|s| s.trim().to_string())
    .filter(|s| !s.is_empty())
    .ok_or((FAIL, "席の tmux の session が読めない".to_string()))?;
    let mut a: Vec<String> = ["new-window", "-d", "-t"].map(String::from).to_vec();
    a.push(format!("{session}:"));
    a.extend([
        "-n".to_string(),
        l.window.name(),
        "-c".to_string(),
        l.workspace.clone(),
    ]);
    let pairs = window_env(l, |name| std::env::var(name).ok());
    for (name, value) in &pairs {
        a.extend(["-e".to_string(), format!("{name}={value}")]);
    }
    let names: Vec<&str> = pairs.iter().map(|(name, _)| *name).collect();
    a.extend(["-P", "-F", WINDOW_FORMAT, "--", ENV, "-S"].map(String::from));
    a.extend([keep_only(&names), PROGRAM.to_string()]);
    a.extend(args.iter().cloned());
    let out = capture(tmux, &a, ws, GIT_TIMEOUT)
        .and_then(|o| String::from_utf8(o).ok())
        .ok_or((FAIL, "tmux の new-window が落ちた".to_string()))?;
    let (window, pane_pid) = out
        .trim()
        .split_once(' ')
        .ok_or((FAIL, format!("tmux の出力 {out:?} の形")))?;
    let pid = pane_pid
        .parse()
        .map_err(|_| (FAIL, format!("pane の pid {pane_pid:?} の形")))?;
    Ok((window.to_string(), pid))
}

/// 問う窓を席の子として起こす（標準出力は `.consult/ask-<k>.json`・標準エラーは `.consult/ask-<k>.err`）。
fn ask(ws: &Path, l: &Launch, args: &[String], k: u32) -> Result<std::process::Child, Refused> {
    let file = |ext: &str| {
        create_plain(ws, &format!(".consult/ask-{k}.{ext}"))
            .map_err(|e| (UNKNOWN, format!("ask-{k}.{ext} を作れない: {e}")))
    };
    let mut cmd = Command::new(PROGRAM);
    cmd.args(args)
        .current_dir(ws)
        .stdin(Stdio::null())
        .stdout(file("json")?)
        .stderr(file("err")?)
        .process_group(0)
        .env_clear()
        .envs(window_env(l, |name| std::env::var(name).ok()));
    cmd.spawn()
        .map_err(|e| (FAIL, format!("{PROGRAM} を起こせない: {e}")))
}

/// 問う窓の終わりを待ち、新しい所見ごとに経路 完了 の 1 行を、無ければ止まった窓の 1 行を出す。
fn wait_ask(ws: &Path, id: WindowId, started: Started) -> Result<(), Refused> {
    let Some(mut child) = started.child else {
        return Ok(());
    };
    let end = Instant::now() + ASK_LIMIT;
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if Instant::now() < end => std::thread::sleep(ASK_STEP),
            _ => {
                emit_err(&format!("tz consult launch: 問う窓 {id} を上限で止めた"));
                stop_with(child, std::ffi::OsStr::new(KILL));
                break;
            }
        }
    }
    let tzw = tzw();
    let new: Vec<_> = findings(ws, id)
        .into_iter()
        .filter(|f| !started.before.contains(f))
        .collect();
    if new.is_empty() {
        emit(&notice(&Event::Stalled(id), &tzw));
    }
    for f in new {
        emit(&notice(
            &Event::Finding {
                id: f,
                via: tsuzuri_contract::consult::Via::Done,
            },
            &tzw,
        ));
    }
    Ok(())
}
