//! 起動の口の argv と設定と検め（設計ノート surface-wave27a 行 cs-argv・判断の記録 ADR-29 決定 (5)(6)(7)(10)・受入 AC19）。
//! 窓を起こす `claude` の引数と `--settings` の JSON の字と環境の閉じた列と私用の temp の path と最初の手引きの字を組み、
//! 組んだ argv を `audit` で検める（欠けや広い許しが 1 つでも在れば起動の口は起こさない）。
//! 旗と鍵の名は Claude Code の字のまま置く。path は呼ぶ側が「/」で始まる絶対 path で渡す（読む根は在る dir だけ）。

use serde_json::{Value, json};
use tsuzuri_contract::consult::{DRAFT_FIELDS, Form, WindowId};
use tsuzuri_contract::ledger::fnv1a64;

/// 起こす program。
pub const PROGRAM: &str = "claude";

/// tz の consult と組む plugin の版（plugin.json の version と同じ字にする・行 cs-plugin が plugin.json を上げる）。
pub const PLUGIN_VERSION: &str = "0.2.0";

/// plugin の解き方が版を置く環境変数。
pub const VERSION_ENV: &str = "TZ_PLUGIN_VERSION";

/// 窓の id を窓に渡す環境変数。
pub const ID_ENV: &str = "TZ_CONSULT_ID";

/// 私用の temp の置き場を渡す環境変数。
pub const TMPDIR_ENV: &str = "CLAUDE_CODE_TMPDIR";

/// 話す窓の tmux の `-e` で渡す閉じた 4 つ（口座の置き場・PATH・窓の id・私用の temp）。
pub const TALK_ENV: [&str; 4] = ["CLAUDE_CONFIG_DIR", "PATH", ID_ENV, TMPDIR_ENV];

/// 問う窓が席の環境から外す 2 つ。
pub const ASK_DROP: [&str; 2] = ["TMUX", "TMUX_PANE"];

/// 道具の閉じた列。
pub const TOOLS: &str = "Read,Grep,Glob,Bash,Edit,Write,WebSearch,WebFetch";

/// 殻の命令のネットワークに許すドメイン（package の配り元 6 つと、GitHub の読むだけの配り元 2 つ
/// = file 1 本ずつの raw と source の tar.gz の codeload・判断の記録 ADR-53）。
pub const DOMAINS: [&str; 8] = [
    "pypi.org",
    "files.pythonhosted.org",
    "crates.io",
    "index.crates.io",
    "static.crates.io",
    "registry.npmjs.org",
    "raw.githubusercontent.com",
    "codeload.github.com",
];

/// Claude の口座の資格の file（口座の置き場を替えない時の置き場・読む道具の断りにも置く）。
const CLAUDE_CREDENTIALS: &str = "~/.claude/.credentials.json";

/// 囲いと読む道具の断りの両方から見えなくする資格の file（home の下・囲いはほかに state dir の accounts とその symlink の先と repo の台帳の鍵と共有の temp）。
pub const CREDENTIALS: [&str; 6] = [
    "~/.config/gh",
    "~/.ssh",
    "~/.git-credentials",
    "~/.config/git/credentials",
    "~/.cld-env",
    CLAUDE_CREDENTIALS,
];

/// argv に在ってはならない字（plugin の置き場・確かめを飛ばす形・外の道具の設定・指示の足し）。
pub const FORBIDDEN: [&str; 5] = [
    "--plugin-dir",
    "--dangerously-skip-permissions",
    "bypassPermissions",
    "--mcp-config",
    "--append-system-prompt",
];

/// 囲いの値の決まった 4 つ（JSON pointer と値）。
pub const SANDBOX_FIXED: [(&str, bool); 4] = [
    ("/sandbox/enabled", true),
    ("/sandbox/autoAllowBashIfSandboxed", true),
    ("/sandbox/allowUnsandboxedCommands", false),
    ("/sandbox/failIfUnavailable", true),
];

/// 設定に置かない鍵（JSON pointer）。
pub const ABSENT: [&str; 4] = [
    "/sandbox/excludedCommands",
    "/sandbox/network/allowUnixSockets",
    "/sandbox/network/allowAllUnixSockets",
    "/permissions/defaultMode",
];

/// 窓を起こす材料（path は「/」で始まる絶対 path・uid は 10 進の字）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Launch {
    pub form: Form,
    pub window: WindowId,
    /// 作業場（窓の作業の dir）。
    pub workspace: String,
    pub repo: String,
    /// project の state dir。
    pub state: String,
    /// state dir の accounts の symlink の先（口座の置き場の実体・境が起こす時に解く・字の順で重なりなし）。
    pub account_dirs: Vec<String>,
    /// 読む根のうち在る dir（`read_roots` の部分の列・順もそのまま）。
    pub roots: Vec<String>,
    /// 解いた tz の path。
    pub tz: String,
    pub uid: String,
    pub model: String,
    pub effort: String,
    /// 束に問い（question.md）が在るか（話す窓の最初の指示が読む）。
    pub question: bool,
}

/// 読む根の 3 つ（repo・state dir の fleet・pipe）。
pub fn read_roots(repo: &str, state: &str) -> [String; 3] {
    [
        repo.to_string(),
        format!("{state}/fleet"),
        format!("{state}/pipe"),
    ]
}

/// 私用の temp の置き場（作業場の path の FNV-1a 64 の 16 進の頭 6 字・15 byte）。
pub fn private_tmp(workspace: &str) -> String {
    let hex = format!("{:016x}", fnv1a64(workspace.as_bytes()));
    format!("/tmp/tzc-{}", hex.get(..6).unwrap_or_default())
}

/// 守りの hook の命令（落ちても時間切れでも断る側に倒す）。
pub fn guard_command(tz: &str) -> String {
    format!("timeout 4 {tz} consult guard || exit 2")
}

/// 最初の指示（問う窓は `-p` の問い・話す窓は最後の位置の引数）。
pub fn prompt(l: &Launch) -> String {
    match (l.form, l.question) {
        (Form::Ask, _) => {
            "bundle/brief.md と bundle/question.md を読んで答え、所見を tz consult answer で出して終われ"
                .to_string()
        }
        (Form::Talk, true) => {
            "bundle/brief.md と bundle/question.md を読み、notes.md と findings/ が在れば続きから始めて"
                .to_string()
        }
        (Form::Talk, false) => {
            "bundle/brief.md を読み、notes.md と findings/ が在れば続きから始めて".to_string()
        }
    }
}

/// 囲いから見えなくする path（資格の file・accounts とその symlink の先・台帳の鍵・共有の temp）。
fn hidden(l: &Launch) -> Vec<String> {
    let mut paths: Vec<String> = CREDENTIALS.iter().map(|c| (*c).to_string()).collect();
    paths.push(format!("{}/accounts", l.state));
    paths.extend(l.account_dirs.iter().cloned());
    paths.push(format!("{}/.beads/.env", l.repo));
    paths.push(format!("/tmp/claude-{}", l.uid));
    paths
}

/// 読む道具の断り（accounts とその symlink の先・台帳の鍵・資格の file は file にも dir にも効くよう path と下の全部の 2 つ）。
fn denied_reads(l: &Launch) -> Vec<String> {
    let mut rules = vec![format!("Read(/{}/accounts/**)", l.state)];
    rules.extend(l.account_dirs.iter().map(|d| format!("Read(/{d}/**)")));
    rules.push(format!("Read(/{}/.beads/.env)", l.repo));
    for c in CREDENTIALS {
        rules.extend([format!("Read({c})"), format!("Read({c}/**)")]);
    }
    rules
}

/// `--settings` に渡す設定。
pub fn settings(l: &Launch) -> Value {
    let w = &l.workspace;
    let files: Vec<Value> = hidden(l)
        .into_iter()
        .map(|p| json!({"path": p, "mode": "deny"}))
        .collect();
    json!({
        "permissions": {
            "allow": ["WebSearch", "WebFetch", format!("Edit(/{w}/**)")],
            "deny": denied_reads(l),
        },
        "sandbox": {
            "enabled": true,
            "autoAllowBashIfSandboxed": true,
            "allowUnsandboxedCommands": false,
            "failIfUnavailable": true,
            "filesystem": {"denyWrite": l.roots},
            "credentials": {"files": files},
            "network": {"allowedDomains": DOMAINS},
        },
        "env": {
            "UV_CACHE_DIR": format!("{w}/.venv/uv-cache"),
            "PIP_CACHE_DIR": format!("{w}/.venv/pip-cache"),
            "CARGO_HOME": format!("{w}/target/cargo-home"),
            "npm_config_cache": format!("{w}/node_modules/.npm-cache"),
        },
        "hooks": {
            "PreToolUse": [{"matcher": "*", "hooks": [
                {"type": "command", "command": guard_command(&l.tz), "timeout": 10}
            ]}]
        },
    })
}

/// `claude` に渡す引数の列（program の名 `PROGRAM` は含めない）。
pub fn argv(l: &Launch) -> Vec<String> {
    let mut a: Vec<String> = Vec::new();
    if l.form == Form::Ask {
        a.extend(["-p".to_string(), prompt(l)]);
    }
    a.extend(
        [
            "--restricted",
            "--permission-mode",
            "dontAsk",
            "--tools",
            TOOLS,
            "--disallowedTools",
            "mcp__*",
            "--strict-mcp-config",
        ]
        .map(String::from),
    );
    for root in &l.roots {
        a.extend(["--add-dir".to_string(), root.clone()]);
    }
    a.extend([
        "--settings".to_string(),
        settings(l).to_string(),
        "--model".to_string(),
        l.model.clone(),
        "--effort".to_string(),
        l.effort.clone(),
        "--name".to_string(),
        l.window.name(),
    ]);
    match l.form {
        Form::Ask => a.extend(["--output-format".to_string(), "json".to_string()]),
        Form::Talk => a.push(prompt(l)),
    }
    a
}

/// 起動の口が置く環境のうち中核が決める 2 つ（窓の id と私用の temp）。
pub fn env(l: &Launch) -> [(&'static str, String); 2] {
    [
        (ID_ENV, l.window.to_string()),
        (TMPDIR_ENV, private_tmp(&l.workspace)),
    ]
}

/// plugin の解き方が置いた版の字が `PLUGIN_VERSION` と同じか。
pub fn version_ok(value: Option<&str>) -> bool {
    value == Some(PLUGIN_VERSION)
}

/// 旗の次の字。
fn after<'a>(argv: &'a [String], flag: &str) -> Option<&'a str> {
    let at = argv.iter().position(|a| a == flag)?;
    argv.get(at + 1).map(String::as_str)
}

/// argv の旗の欠けと断る字（欠けの名の列・空なら合格）。
fn audit_flags(argv: &[String], l: &Launch, gaps: &mut Vec<String>) {
    for bad in FORBIDDEN {
        if argv.iter().any(|a| a.starts_with(bad) || a.ends_with(bad)) {
            gaps.push(bad.to_string());
        }
    }
    let name = l.window.name();
    let pairs: [(&str, Option<&str>); 6] = [
        ("--permission-mode", Some("dontAsk")),
        ("--tools", Some(TOOLS)),
        ("--disallowedTools", Some("mcp__*")),
        ("--name", Some(name.as_str())),
        ("--model", None),
        ("--effort", None),
    ];
    for (flag, want) in pairs {
        let got = after(argv, flag);
        if got.is_none_or(|g| g.starts_with("--") || want.is_some_and(|w| w != g)) {
            gaps.push(flag.to_string());
        }
    }
    for flag in ["--restricted", "--strict-mcp-config", "--settings"] {
        if !argv.iter().any(|a| a == flag) {
            gaps.push(flag.to_string());
        }
    }
    let ask = l.form == Form::Ask;
    if argv.iter().any(|a| a == "-p") != ask
        || (after(argv, "--output-format") == Some("json")) != ask
    {
        gaps.push("form".to_string());
    }
    let roots = read_roots(&l.repo, &l.state);
    for (i, a) in argv.iter().enumerate() {
        let root = argv.get(i + 1).filter(|_| a == "--add-dir");
        if root.is_some_and(|r| !roots.contains(r) || !l.roots.contains(r)) {
            gaps.push("--add-dir".to_string());
        }
    }
}

/// 字の列か（鍵の値が字の配列ならその字の列・ほかは空）。
fn strings(value: Option<&Value>) -> Vec<&str> {
    value
        .and_then(Value::as_array)
        .map(|xs| xs.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default()
}

/// 設定の囲いの欠けと広い許し。
fn audit_sandbox(set: &Value, l: &Launch, gaps: &mut Vec<String>) {
    for (pointer, want) in SANDBOX_FIXED {
        if set.pointer(pointer) != Some(&Value::Bool(want)) {
            gaps.push(pointer.to_string());
        }
    }
    for pointer in ABSENT {
        if set.pointer(pointer).is_some() {
            gaps.push(pointer.to_string());
        }
    }
    let domains = strings(set.pointer("/sandbox/network/allowedDomains"));
    if domains.is_empty() || domains.iter().any(|d| !DOMAINS.contains(d)) {
        gaps.push("/sandbox/network/allowedDomains".to_string());
    }
    let deny_write = strings(set.pointer("/sandbox/filesystem/denyWrite"));
    if l.roots.iter().any(|r| !deny_write.contains(&r.as_str())) {
        gaps.push("/sandbox/filesystem/denyWrite".to_string());
    }
    let denied: Vec<&str> = set
        .pointer("/sandbox/credentials/files")
        .and_then(Value::as_array)
        .map(|xs| {
            xs.iter()
                .filter(|x| x.get("mode").and_then(Value::as_str) == Some("deny"))
                .filter_map(|x| x.get("path").and_then(Value::as_str))
                .collect()
        })
        .unwrap_or_default();
    if hidden(l).iter().any(|w| !denied.contains(&w.as_str())) {
        gaps.push("/sandbox/credentials/files".to_string());
    }
}

/// 設定の許しと断りと環境と hook の欠けと広い許し。
fn audit_permissions(set: &Value, l: &Launch, gaps: &mut Vec<String>) {
    let edit = format!("Edit(/{}/**)", l.workspace);
    let allow = strings(set.pointer("/permissions/allow"));
    let allowed = ["WebSearch", "WebFetch", edit.as_str()];
    if allow.iter().any(|a| !allowed.contains(a))
        || !allow.contains(&"WebSearch")
        || !allow.contains(&"WebFetch")
    {
        gaps.push("/permissions/allow".to_string());
    }
    let deny = strings(set.pointer("/permissions/deny"));
    if denied_reads(l).iter().any(|w| !deny.contains(&w.as_str())) {
        gaps.push("/permissions/deny".to_string());
    }
    let under = format!("{}/", l.workspace);
    let env = set.pointer("/env").and_then(Value::as_object);
    let env_ok = env.is_some_and(|e| {
        [
            "UV_CACHE_DIR",
            "PIP_CACHE_DIR",
            "CARGO_HOME",
            "npm_config_cache",
        ]
        .iter()
        .all(|k| {
            e.get(*k)
                .and_then(Value::as_str)
                .is_some_and(|v| v.starts_with(&under))
        })
    });
    if !env_ok {
        gaps.push("/env".to_string());
    }
    let hook = set
        .pointer("/hooks/PreToolUse/0/hooks/0/command")
        .and_then(Value::as_str);
    if hook != Some(guard_command(&l.tz).as_str())
        || set.pointer("/hooks/PreToolUse/0/matcher") != Some(&json!("*"))
    {
        gaps.push("/hooks/PreToolUse".to_string());
    }
}

/// 組んだ argv を検める（欠けと広い許しの名の列・空なら起こしてよい）。
pub fn audit(argv: &[String], l: &Launch) -> Vec<String> {
    let mut gaps = Vec::new();
    audit_flags(argv, l, &mut gaps);
    match after(argv, "--settings").map(serde_json::from_str::<Value>) {
        Some(Ok(set)) => {
            audit_sandbox(&set, l, &mut gaps);
            audit_permissions(&set, l, &mut gaps);
        }
        _ => gaps.push("/".to_string()),
    }
    gaps
}

/// 束の手引き bundle/brief.md の字（`tz` は解いた tz の path）。
pub fn brief(tz: &str, window: WindowId) -> String {
    let fields = DRAFT_FIELDS.join(", ");
    format!(
        "# 相談の窓 {window} の手引き\n\n\
         あなたは tsuzuri の相談の窓 {window} です。席（orchestrator）の文脈を汚さないために分けた、自由な調べと検証の場です。\n\n\
         - 作業場はこの dir（cwd）です。書けるのは作業場の下だけです。repo と器の出力は読むだけで、作業場の外へは基本ソフトの囲いで書けません。\n\
         - 実験と検証のプログラムとレポートは work/ に置いてください。package の cache と入れ先は作業場の下に向けてあります。\n\
         - web の調べ（WebSearch・WebFetch）はいつも使えます。資格と private な project の中身を外へ出さないでください。\n\
         - 台帳は bundle/ledger.json の写しで読んでください（囲いの中では bd を撃てません）。束は {tz} consult bundle {window} で組み直せます。\n\
         - 話題ごとの要点は notes.md に書き足してください（開き直した時の続きに使います）。\n\
         - 結論は所見の草稿（JSON）を drafts/ に書き、{tz} consult answer <草稿の file> で出してください。所見は findings/ に置かれ、席が受けます。\n\
         - 草稿の欄は次の 12 です: {fields}。欠けると受けられません。options は 2 つ以上で採る（adopted）のは 1 つ、claims の confidence は verified・deduced・inferred・uncertain のどれか、attachments の path は作業場からの相対です。\n\
         - 囲いに断られて困った事は sandbox_denials に書いてください。\n\
         - 所見を出しても窓は閉じません。repo の編集・台帳の書き・回答・承認はしません（承認は決定の画面だけで受けます）。\n"
    )
}
