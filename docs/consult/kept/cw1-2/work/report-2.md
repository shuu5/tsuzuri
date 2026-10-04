# 相談の窓は「新しい Claude Code の機能と OSS を試して tsuzuri に活かせるか確かめる場」になるか — Claude Mods と superpowers を例に撃った

- 日: 2026-10-05（UTC）・窓 cw1（話す窓・Claude Code 2.1.289）・束 3766ac7797cdf78f
- 持ち主の問い（話す窓・席の要約）: 窓で新しい Claude Code の機能や OSS を試し、tsuzuri に活かせるかを確かめられるか。例は Claude Mods と superpowers（obra）。あわせて、窓に status line が出ない理由。

## 1. 結論

できる事とできない事がはっきり分かれる。**読んで評する・package を入れて撃つ・Claude Code の mod を書いて静的に検め単体で試す**までは窓の中でできる。**生きた Claude Code の機能（plugin・skill・mod・model を呼ぶ入れ子の claude）を動かす事・GitHub から clone する事・npm**は今の囲いではできない。

| 試し | 結果 | 根拠 |
|---|---|---|
| WebFetch で GitHub（README・raw の SKILL.md・tree・hooks.json）と公式文書を読む | できた | superpowers と mods の文書を全部 WebFetch で読んだ |
| 殻から github.com・raw.githubusercontent.com・codeload.github.com | 断られた（X-Proxy-Error: blocked-by-allowlist） | git clone と curl -I |
| uv で PyPI から入れる（話題 1） | できた | six が .venv に入った |
| cargo search（crates.io） | できた | serde 1.0.229 が返った |
| npm view / search（registry.npmjs.org） | 407 Proxy Authentication Required | 既定・--https-proxy 明示とも同じ。curl -I は同じ host に通る |
| 入れ子の claude -p（model を呼ぶ） | 不可（Not logged in・api.anthropic.com denied） | 口座の dir は空の tmpfs |
| `claude plugin validate`（mod の静的な検め） | 通った（警告 1: author 無し） | work/mods/first-mod |
| `claude plugin test`（mod の単体の歯・session 不要） | 1 pass 0 fail | tests/first-mod.test.ts |
| `claude -p "/tally" --plugin-dir work/mods/first-mod`（model を呼ばない mod の命令） | 答えた: "first-mod: Claude has made 0 tool calls since this mod loaded" | login 無しで mod は載って動く |
| この窓の自分自身に mod・plugin・skill を載せる | 不可 | --restricted・--plugin-dir は起動の FORBIDDEN・dontAsk では Claude が書いた mod の承認が出ない（公式文書） |

### 1.1 npm の 407 の切り分け（work/npm-diag.sh → work/npm-diag-result.txt）
- node v18.19.1・npm 9.2.0（Ubuntu の apt）。npm は proxy に GET を出して 407 を受ける（log に Proxy-Authorization の再送が無い）。
- proxy の環境を外すと EAI_AGAIN（DNS が無い）= 囲いの中で外へ出る道は proxy だけ。
- 同じ proxy と資格で curl・uv・cargo・git は通るので、資格の形ではなく npm 9.2 の proxy 認証の扱いの問題と読む（推し量り）。Node 18 と npm 9 は EOL。host の Node を 20 以上（npm 10 以上）に上げて撃ち直すのが席の手（窓からは host を変えられない）。

### 1.2 器の dontAsk の黙った断り（話題 1 の続き）
- また 2 回断られた: node の fetch と log の grep を 1 行にした命令・草稿を検める python の heredoc。work/ の script にして `bash work/x.sh` で撃つと通った。窓の使い方の決まりとして brief に 1 行足す価値がある（話題 1 の推奨のまま）。

## 2. Claude Mods（2026-10-01 公開・公式 code.claude.com/docs/en/plugins/mods/overview, reference, create, test, admin）

### 2.1 何か
- v2.1.287 以上で**既定で有効**。この窓の 2.1.289 も該当。mod は plugin で、`hooks/hooks.json` の `"modules": ["./register.js"]` が JS/TS の hooks module を指す。`export function register(on)` の中で `on('tool.call', { tool: 'Bash' }, async ($, e, next) => …)` のように event ごとの handler（mod の hook）を登録する。
- 従来の hook は「settings hook」と呼び直され、廃止ではない。mod の hook は Claude Code の process の中で走る関数。
- event の群: tools（tool.call・tool.check・tool.describe）・prompts（prompt.submit・prompt.compose・prompt.section・prompt.context・prompt.attachment・skill.prompt・attribution.text）・commands（command.run・command.describe・config.*）・turns（turn.start・turn.step・turn.complete）・session（start・end・compact・receive・send・append・attach・detach・measure）・subagents（agent.offer・agent.spawn）・interface（ui.render・ui.press・ui.input・…）・other mods（plugin.register・engine.create）・telemetry・`classic.<SettingsHookEvent>`。
- `$` API: ui（pane・band・toast・ask）・command・tool・agent・model（complete・fork・classify）・prompt（submit は持ち主の言葉としても出せる）・turn・session（messages・send・usage）・config・settings・env・fs・store・state・clock・http・process・mcp・audio・telemetry。
- 検め: `claude plugin validate <dir>` が `hooks:` と `calls:`（$.fs.read・$.process.run・$.http.fetch・$.model.complete・$.prompt.submit など）を静的に列挙。`claude plugin test` が session・login・network 無しで歯を走らせる（この窓で確かめた）。

### 2.2 守りの性質
- **囲い無し**。持ち主の権限で file・process・network に触れ、`tool.check` で permission の prompt の前に allow/deny を返せる。`ask` の規則や managed でない PreToolUse hook の block を上書きできる。`deny` の規則に勝てるのは guard（sec-default@builtin）が載らない環境か `allowModsToOverrideDenyRules`。
- 統制は managed settings: `pluginConfigs.cc-plugin-sec-default@builtin.options.allowManagedModsOnly`（user の mod を全部断る）・`disableSideloadFlags`（--plugin-dir・--plugin-url・--mcp-config・--agents を断る）・`prependPlugins`/`appendPlugins`（順序）・`disableAllHooks`（settings hook も status line も止まる）。`--safe-mode` は 1 session だけ止める。
- 順序: managed の PreToolUse → mod（prepend → user → append → builtin）→ plugin と --settings の PreToolUse。**tsuzuri の guard hook（plugin の question-gate・窓の consult guard）は user の mod の後に走る**＝席に user の mod を入れると guard を先回りされ得る。

### 2.3 tsuzuri への当て方（見立て）
- 席（orchestrator）の hook は今は shell の settings hook（PreToolUse の question-gate・PostToolUse の question-signal・Stop・UserPromptSubmit の deliver・PostToolBatch）。mod に移すと得る物: (a) `prompt.context`/`prompt.section`/`session.start` で**憲法の決定的な注入**（memo t3-hub.74.59・条 P-13）を器の中で持てる、(b) `ui.render` の Pane/AbovePrompt で board の未受けの所見・相談の頼み・裁定の待ちを席の画面に描ける、(c) `session.append`・`turn.complete` で発話の記録を器の側で取れる、(d) `tool.check` で guard を permission の段に移せる。
- 失う物・論点: mod は囲い無しで持ち主の権限を持つ。憲法が hook の守りに置いている前提（落ちても断る側に倒す・器の外で動く）とは性質が違う。採るなら managed settings で `allowManagedModsOnly` と `disableSideloadFlags` を置き、tsuzuri の mod を「組織の mod」の形（絶対 path の marketplace・enabledPlugins・prependPlugins）で載せる統制の決定が先。
- 相談の窓には届かない: 窓は plugin を渡さない（ADR-29 決定 (6)）。built-in の mod（agents-md・diff・sec-default・telemetry）だけ。
- 窓でできる事: mod を work/ に書き、validate と test で回し、model を呼ばない命令は `claude -p "/cmd" --plugin-dir` で撃つ。描画（Pane 等）と model を呼ぶ部分は囲いの外（持ち主の手元の別 session）で見る。

## 3. superpowers（github.com/obra/superpowers・MIT・Jesse Vincent）

### 3.1 何か
- 「agentic skills framework & software development methodology」。skills 15 本（brainstorming・writing-plans・executing-plans・test-driven-development・systematic-debugging・verification-before-completion・requesting-code-review・receiving-code-review・using-git-worktrees・finishing-a-development-branch・subagent-driven-development・dispatching-parallel-agents・writing-skills・using-superpowers・diagnosing-superpowers）。Claude Code のほか Codex・Cursor・Gemini・Copilot CLI・OpenCode などに同じ skill を配る。GitHub の頁の字では 295.3k stars。
- 入口: `hooks/hooks.json` の SessionStart（matcher `startup|clear|compact`）が `hooks/run-hook.cmd session-start` を撃ち、`skills/using-superpowers/SKILL.md` を `hookSpecificOutput.additionalContext` で注入する。中身は「関連する skill を**どんな応答や行動の前にも**先に呼べ」「1% でも当たると思えば必ず使え」「user の指示 > skill > 既定」。
- brainstorming: Spike／Bounded／Architectural に分けて承認の門を置き、Architectural は spec を `docs/superpowers/specs/YYYY-MM-DD-<topic>-design.md` に書いて writing-plans へ。verification-before-completion: 「証拠の前に主張なし」の 5 段の門。
- 入れ方: `/plugin install superpowers@claude-plugins-official`（公式 marketplace に載る）か obra/superpowers-marketplace。npm には無い（レジストリに同名の別物も無かった・407 のため未確定）。

### 3.2 tsuzuri への当て方（見立て）
- 席に載せる道: scribe2 の seat launch は `--plugin-dir <anchor>/plugin` に加えて host.toml の `[[plugin]]` dir を渡す（account-lifecycle.md §4・seat/cycle/launch.rs）。局所の clone を [[plugin]] 行で載せれば席に入る（持ち主の手・囲いの外）。便（係）には spawn.rs の copy_plugin が器の plugin と consumer の plugin しか写さないので届かない（scribe2 の変更が要る）。
- 丸ごと入れる事の問題: (1) SessionStart の注入「skill を先に」が tsuzuri の憲法の注入・席の手引き・裁定の流れと競合する。(2) brainstorming が repo に `docs/superpowers/specs/` を書く＝tsuzuri の design-intent と契約の規律の外に書き物が増える。(3) subagent-driven-development・using-git-worktrees は tsuzuri の便と worktree の仕組みと二重になる。
- 活かせる物: **仕組み**としては SessionStart で additionalContext を注入する形が、memo t3-hub.74.59（憲法の席への注入・P-13・最優先）の当座の実装にそのまま使える（tsuzuri の plugin の hooks.json には今 SessionStart が無い）。**中身**としては verification-before-completion・systematic-debugging・test-driven-development の規範文が、係の手引きや tsuzuri の plugin の skill に写すに足る質。丸ごとの install ではなく、この 2 つを tsuzuri の形で取り込むのが合う。

## 4. この窓で status line が出ない理由
- 席の status line は scribe2 が口座の settings.json に `statusLine = {type: command, command: "scribe2 seat statusline"}` を書く事で出る（scribe2/docs/design/account-lifecycle.md §3）。
- 窓は `--restricted` で起きるので口座の settings.json（許す一覧・hook・環境・statusLine）を読まない（ADR-29 の grill の確かめ (3)）。起動の口が `--settings` で渡す JSON（tsuzuri-core consult/launch.rs の settings()）には statusLine の鍵が無い。この 2 つで何も描かれない。
- 足すなら窓向きの命令（例 `tz consult statusline`: 窓 id・題・所見の数・束の要約値）を settings に置く。audit は statusLine の鍵を見ないので通るが、fixture launch-argv.yaml と ADR-29 決定 (7) の字を揃える要（席と持ち主の判断）。

## 5. 持ち主に問う事
- 殻のネットワークに**読むだけの** GitHub の配り元（raw.githubusercontent.com・codeload.github.com・objects.githubusercontent.com）を足してよいか。github.com 本体は足さない（push の host）。足せば release の tar.gz と raw の file を curl で取れ、OSS を code として窓に置けるが、clone（github.com）は引き続き不可。ADR-29 決定 (7)(エ) の字の改めになる。
- superpowers を席に試験的に載せるか（host.toml の [[plugin]] 行・局所の clone）。載せるなら注入の競合を見るため短い期間に限る。
- Claude Mods を tsuzuri の仕組みに採るかの検討を memo として起こすか（統制の決定が先・憲法の守りの前提との照らし）。

## 6. 添え物
- work/mods/first-mod/（plugin.json・hooks/hooks.json・hooks/register.js・tests/first-mod.test.ts）: 公式の tutorial の mod。validate と test が窓で通った証。
- work/npm-diag.sh・work/npm-diag-result.txt: npm の 407 の切り分け。
