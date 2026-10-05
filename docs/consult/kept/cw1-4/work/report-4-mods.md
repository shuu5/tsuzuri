# Claude Mods × tsuzuri — 仕組みの要点と、今の妥協（画面を読む・入力欄に打つ）を器の中で処理できるかの検証

- 日: 2026-10-05（UTC）・窓 cw1・Claude Code 2.1.289（この窓）
- 読んだ物: 公式 code.claude.com/docs/en/plugins/mods/{overview,reference,create,events,api,test,admin}、この build が書いた型定義 work/mods/first-mod/.claude-plugin/types/claude-code/index.d.ts（11,500 行・公式より新しい正本）、scribe2 の seat/inject.rs・hook/stamp.rs・pipe/notify.rs・seat/cycle/relaunch.rs・docs/design/seat-heartbeat.md・dispatcher.md §21、scribe2 と tsuzuri の plugin の hooks.json。
- 撃った物: work/mods/first-mod で `claude plugin validate`・`claude plugin test`・`claude -p "/tally" --plugin-dir`（全部この窓の囲いの中で通った・report-2）。

## 1. 今の妥協（器が Claude Code と話す所・現物）

| 所 | 現物 | 妥協の形 |
|---|---|---|
| a. 席への注入 | `seat/inject.rs` `deliver_within`。呼び手は席の起動（relaunch）・dispatcher の通知（`pipe/notify.rs`）・管理 tick（heartbeat） | tmux `send-keys -l` で入力欄に 1 行を打ち、`send-keys Enter`。**送る前に `capture-pane` で入力欄を読み**、非空なら 1 key も送らない（人の打ちかけとの merge を避ける）。送達は pane の字面で、消費は hook の打刻で測る。Enter が落ちた周は 1 回だけ再送（§21） |
| b. 席の状態 | `hook/stamp.rs` → `seat/<target>/state.jsonl` | shell の settings hook（SessionStart・Stop → Idle、UserPromptSubmit → Busy）。`--pane "$TMUX_PANE"` で自席を解く。ADR-0015「pane を読まない」はここで守る |
| c. 入力欄の門 | `guard_input`／`pass_input`（Clear・OwnQueued・Foreign） | pane の prompt 行を字面で特定する（TUI の折り返しを畳んで照合） |
| d. 席の起動と口座の移動 | `seat/cycle/relaunch.rs`・heartbeat の歯 (a)(c) | shell の pane に起動行を send-keys。移動は `/exit` を注入してから起動行 |
| e. 守りと配達 | scribe2 plugin の hooks（SessionStart・PreToolUse・PermissionRequest・UserPromptSubmit・Stop・PreCompact）、tsuzuri plugin の hooks（PreToolUse question-gate・PostToolUse question-signal・PostToolBatch・Stop・UserPromptSubmit deliver） | 全部 shell の settings hook。stdin の JSON を読み、stdout の JSON（additionalContext・decision）か rc 2 で答える。timeout 10〜30 s。配達（裁定の指し示し）は hook の答えの字で席に渡す |
| f. 席の画面 | `scribe2 seat statusline`（口座の settings.json の statusLine） | 1 行の shell |
| g. 便（係） | runner が claude を子 process で起こし `--plugin-dir` で器の plugin を載せる | in-loop guard も shell hook |
| h. 相談の窓の受け | `tz consult watch`（背景の命令）と hook の拾い | 席の背景の命令の終わりで席を起こす |

共通の性質: 席を「外から」動かす道が tmux の入力欄しか無いので、人の打ちかけとの衝突・Enter の落ち・消費の測り・pane の字面の折り返し、という一連の処理が要る。

## 2. Mods の仕組み（要点）

- **どこで走るか**: Claude Code の process の中。plugin の `hooks/hooks.json` の `"modules": ["./register.js"]` が指す ES module。Node の API も timer も無く、外へ触る口は `$` だけ（だから `claude plugin validate` が静的に hooks: と calls: を列挙できる）。`--plugin-dir` か marketplace で載る。`claude -p` でも hook は走る（描画だけ無し）。
- **外とつなぐ口**: `$.process.run(argv)`（shell 無し・30 s 既定・10 min 上限）、`$.process.spawn({argv})`（長生きの子・stdout を stream で読む。型定義に **「session の命の間 bridge を spawn して行を読む」例**がそのまま載る）、`$.http.fetch`、`$.fs.read/write/list`、`$.store`（machine 全体の KV・4 MiB）、`$.env.get/set`、`$.settings.read`。
- **席を動かす口**: `$.prompt.submit({text, asUser?})`（**idle を待って新しい turn を始め、turn が始まった時に resolve**。model は「The <plugin> plugin sent a message: …」の枠で読む・asUser で枠を外せる）、`$.command.run({command, args})`（人が `/command` を打ったように・idle で）、`$.session.send({to, text})`（別 session や subagent へ・SendMessage と同じ）、`$.turn.abort()`、`$.agent.spawn`。
- **観る口**: `turn.start`・`turn.step`（model への 1 request ごと・usage）・`turn.complete`（answer・durationMs・isAborted・usage）、`session.start/end/compact/append/measure`、`tool.call`・`tool.check`、`prompt.submit/compose/section/context/attachment`、`skill.prompt`、`agent.spawn`、`classic.<SettingsHookEvent>`（settings hook と同じ stdin JSON が `e`）。
- **描く口**: `ui.render`（Pane・AbovePrompt・Spinner・ToolUse…）、`$.ui.status`（prompt の下の 1 行）・`toast`・`log`・`ask`。
- **守りの形**: `tool.call` は `{deny}`／`{result}`／書き換え、`tool.check` は permission の段で `allow|ask|deny`（rule と PreToolUse hook の後に決める）。順序は managed の PreToolUse → mod（prepend → user → append → builtin）→ plugin と `--settings` の PreToolUse。hook が throw/timeout すると skip（fail-open）だが、`.catch` で `{deny}` を返せば fail-closed。budget は hook 10 s（`next` と `$` の待ちは数えない）。
- **統制**: managed settings の `allowManagedModsOnly`・`prependPlugins`・`disableSideloadFlags`、`--safe-mode`、`disableAllHooks`。hooks worker が 3 回落ちると built-in 以外の全 mod が外れる。
- **検め**: `claude plugin validate`（hooks:・calls:・env reads:）、`claude plugin test`（session・login・network 無し）。この窓で両方通った。型定義は `--plugin-dir` で載せた時に `.claude-plugin/types/` へ版ごとに書かれる（「文書より型定義を信じよ」）。

## 3. 妥協 → mod の対応表（何が消え、何が残るか）

| 今 | mod でどうなるか | 得る物 | 残る物・注意 |
|---|---|---|---|
| a. 注入（send-keys） | `$.prompt.submit({text})`。外からの契機は (i) `session.start` で `$.process.spawn(['scribe2','seat','bridge',…])` を起こし、その stdout の JSON lines（通知・tick の合図・裁定の指し示し）を読んで submit する bridge、(ii) `$.clock.every` で `scribe2 seat inbox` を `$.process.run` する poll、(iii) `$.http.fetch` で board の server を poll | 入力欄を読まない・co-submit が構造的に無い・**turn が始まった事が promise の resolve で分かる**（consumed の測りと Enter の再送が消える）・pane の字面を一切読まない（ADR-0015 を完全に満たす） | mod は Claude Code が生きている間だけ。**席の起動**は外の手のまま。文は plugin の枠付き（`asUser` で外せる） |
| b. 打刻（stamp） | `session.start`／`turn.start`／`turn.complete`／`session.end` で `$.process.run(['scribe2','hook','stamp',…])`（schema は今のまま） | shell の起動が無い・`usage`・`isAborted`・`durationMs` が構造で取れる・PreCompact 等は `classic.PreCompact` で同じ module に | — |
| c. 入力欄の門 | 不要（submit が idle を待つ） | pane 依存の消滅 | — |
| d. 起動・口座の移動 | 起動は外（tmux か、runner のように子 process で起こす形）。`/exit` は `$.command.run({command:'exit'})` か `$.turn.abort` ＋ 外の終了 | — | built-in の `/exit` を `$.command.run` で撃てるかは型定義で未確認（uncertain）。process の再起動が要る本質は変わらない |
| e. 守り（guard） | `tool.call`（deny・書き換え・`.catch` で fail-closed）と `tool.check`（permission の段）に移す。配達（裁定の指し示し）は `prompt.submit` の `context` か `$.prompt.submit` で | 10 s の budget・構造化された `e`（command・file_path）・tool の結果の書き換え（秘密の伏せ）・1 process 内で速い | mod は囲いの外・持ち主の権限。**managed でない shell の PreToolUse より mod が先に走る**ので、第三者の mod を席に入れると tsuzuri の guard を先回りできる → tsuzuri の mod を `prependPlugins` に置く統制。shell hook は残して二重にしてもよい |
| f. statusline | `ui.render`（AbovePrompt の帯・Pane）か `$.ui.status` | 複数行・button・色・board の未受けの数 | 相談の窓は plugin を渡さないので窓は `statusLine` の設定のまま |
| g. 便（runner） | -p でも hook は走る → in-loop guard を mod に、終端の報告を `turn.complete` で | 速い・構造化 | 描画は無い |
| h. 相談の窓の受け | 席の mod が bridge（`tz consult watch` の出力）か `$.clock.every` で findings/ を見て `$.prompt.submit` | 背景の命令と hook の拾いが 1 本に | 窓の側は変わらない |

## 4. 安全性の見立て（憲法と照らす）

- **機械が落とす線（P-6・P-7・P-14）**: `tool.call` の `{deny}` は「操作が起きない」reject の形で、shell hook の rc 2 と同じ強さ。極性は `.catch` で fail-closed に倒せる（公式の形）。shell の `|| exit 2` と同等。
- **状態は 1 つの database・描画を読まない（P-20）**: mod は `$.store` を持つが正本は scribe2 の state dir のまま、mod は `$.process.run(['scribe2', …])` で書き手を 1 本にするのが筋。pane を読まなくなるので「描画を読まない」を今より強く満たす。
- **host 固有の値で分岐しない（N-7）**: `$.env.get`・`$.settings.read` は validate の env reads: に出る。読まない設計にできる。
- **権限**: mod は持ち主の権限で file・process・network に触れる。shell hook も同じ権限なので、tsuzuri 自身の mod なら信頼の境界は変わらない。差は「prompt と tool call と permission を書き換えられる」事で、第三者の mod を席に入れる時の統制（managed settings）が要る。superpowers は mod を持たない（skills と settings hook だけ）。
- **版の揺れ**: 公式に「events と methods は版で変わる」。`claude plugin validate` と `claude plugin test` を CI の歯にし（session 無しで走る事をこの窓で確かめた）、型定義を版ごとに検める。
- **可用性**: hooks worker が 3 回落ちると mod が全部外れる（`/reload-plugins` まで）。守りを mod だけに頼らず shell hook を残す（二重）か、外れた事を `scribe2` が検知する手が要る。

## 5. 検証の段取り

- **窓でできる事**: tsuzuri の mod の骨（bridge → `$.prompt.submit`、`turn.*` → stamp、`tool.call` guard with `.catch`、`$.ui.status`）を work/mods/ に書き、`claude plugin validate` と `claude plugin test`（`process.spawn` の stream と `prompt.submit` を stub）で回す。model を呼ばない命令は `claude -p "/cmd" --plugin-dir` で動く。
- **席か持ち主の手（囲いの外）**: tsuzuri の外の scratch dir で `claude --plugin-dir <mod>` を対話で起こし、bridge の 1 行で turn が始まる事・`$.ui.status` の描画・`tool.check` の deny を目で見る。
- **その後**: 判断の記録（器の hook を mod へ移す・統制・撤退条件）→ scribe2 と tsuzuri の plugin に hooks module を足す行。

## 6. 結び

できる。今の妥協（pane の字面を読む・入力欄に打つ・Enter の再送・消費の測り）は、`$.prompt.submit` と `turn.*`／`session.*` の event で原理的に消せる。残るのは「process を起こす事と口座の移動」で、これは外の手のまま。安全性は shell hook と同等以上にできるが、mod が持ち主の権限で走る性質・順序（user の mod が先）・版の揺れ・worker の落ちを統制と二重化で受ける要がある。
