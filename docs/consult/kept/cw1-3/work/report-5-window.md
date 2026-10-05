# 相談の窓の整備 — 口座・Remote Control・status line・GitHub の配り元

- 日: 2026-10-05（UTC）・窓 cw1・Claude Code 2.1.289・repo の HEAD 4b0fa8b8（写しの時点）
- 持ち主の求め（チャット・席が言い換えた）: 窓が席と同じ口座で起きているか（mobile の app から管理できない）・窓に status line を足す事・GitHub の配り元を足しても大本の remote に影響が無いか。

## 1. 口座 — 窓は席と同じ口座で起きている（verified）

| 見た物 | 値 |
|---|---|
| この窓の環境 `CLAUDE_CONFIG_DIR` | state dir の下の口座の置き場（席の口座と同じ名・保存の時に席が名を伏せた） |
| fleet/events.jsonl の `t3:orchestrator` の最新の `SeatRegistered`（2026-10-03T15:17:10Z） | 同じ口座（保存の時に席が名を伏せた） |
| その後の `GroupMoved`（2026-10-03T12:44:35Z） | ほかの project の群の移動で tsuzuri の群 Tier1 ではない |
| 窓の起動（tsuzuri-core consult/launch.rs・`TALK_ENV`） | tmux の `-e` で席の `CLAUDE_CONFIG_DIR` をそのまま渡す（ADR-29 決定 (5)） |

つまり口座は一致している。mobile の app に窓が出ないのは口座の問題ではなく、**窓に Remote Control が付いていない**から。席も起動行に `--remote-control` を持たない（SeatRegistered の launch の字）ので、席は持ち主が `/rc` で付けているはず。

注: 群の移動（GroupMoved）が tsuzuri の群に起きると、席は新しい口座で立て直され、開いている窓は前の口座のまま動く（ADR-29 決定 (5) の字どおり）。その時だけ口座がずれる。

## 2. Remote Control — 窓の起動に `--remote-control <窓の名>` を足す（案）

- `claude --help`（2.1.289）: `--remote-control [name]`「Start an interactive session with Remote Control enabled (optionally named)」・`--remote-control-session-name-prefix <prefix>`。後から付けるのは `/rc`（`/remote-control`）。
- 要件（公式と紹介記事）: Pro・Max・Team・Enterprise の claude.ai の login（API key は不可）。接続する側（claude.ai/code・iOS・Android）は同じ claude.ai の口座。席と窓は同じ口座なので、付けば同じ一覧に並ぶ。
- `--restricted` との組み合わせは公式の文書に記述が無い。`--restricted` の help は「code を走らせる道具と WebFetch を外す（--tools に無ければ）・user/project/local の settings を読まない（managed と --settings は効く）・file の道具を作業 dir に閉じる」で、Remote Control を外すとは書いていない。**この窓で `/rc` を打てば今すぐ確かめられる**（持ち主の手）。
- 置き場: 話す窓の argv の `--restricted` の直後に `--remote-control consult-cw<n>`（値を明示する。値を省くと最後の位置の引数＝最初の指示が名に吸われる恐れがあるので省かない）。問う窓（`-p`）には付けない。audit は「話す窓に在って値が窓の名・問う窓に無い」を検める。

## 3. status line — 窓向きの口 `tz consult statusline` を settings に置く（案）

- 出ない理由（verified・report-2 §4）: 席の status line は scribe2 が口座の settings.json に書く `statusLine`（`scribe2 seat statusline`）で出るが、窓は `--restricted` で口座の settings.json を読まず、起動の `--settings` に `statusLine` が無い。
- 案: `--settings` に `"statusLine": {"type": "command", "command": "<tz> consult statusline"}` を足す（話す窓・問う窓とも同じ settings・問う窓では描かれないだけ）。口は読むだけで何も書かず、1 行を出す:

  `窓 cw1 ｜ 題 テスト ｜ fable・xhigh ｜ 所見 2 ｜ 束 3766ac77 ｜ 文脈 37%`

  - 窓・題・model と念入りさ: `.consult/window.json`。所見の数: `findings/`。束の要約値の頭 8 字: `bundle/digest`。context の割合: Claude Code が stdin に渡す session の JSON の `context_window.used_percentage`（無ければ欄を出さない・ほかの鍵は読み飛ばす）。
  - 字の組みと JSON の読みは中核（tsuzuri-core `consult::status`・純な関数・歯 cwsts_）、I/O は境界（tsuzuri-boundary `consult/statusline.rs`）。
  - 窓の守り（`SEAT_VERBS` の 8 つ）には入れない（席の口でない・読むだけ）。status line の process は Claude Code が囲いの外で起こすので、窓の中の Bash の守りは関わらない。
- 変更の範囲（repo の写しで組んで歯を撃った・§5）: tsuzuri-core `consult/launch.rs`（`REMOTE_FLAG`・argv・settings・audit）・`consult/status.rs`（新）・`consult/mod.rs`、歯 `tests/teeth3/cwarg.rs`（statusLine と remote の欠けと違いを断る）・`cwsts.rs`（新）・`main.rs`、fixture `tests/fixtures/consult/launch-argv.yaml`、tsuzuri-boundary `consult/mod.rs`（口の 1 つ・USAGE）・`consult/statusline.rs`（新）。差分: work/patch/consult-statusline-remote.patch。
- 設計の字の揃え（席の手）: ADR-29 決定 (7) の settings の列に statusLine と Remote Control を足す改め・設計ノート surface-wave29a に行 cs-statusline・受入 AC19 の fixture。
- 確かめられていない事（席か持ち主の手）: `--settings` の statusLine が `--restricted` の対話の窓で描かれる事（文書は「managed と --settings は効く」と言う）、`context_window.used_percentage` の鍵の名（この版の公式の status line の頁は大きすぎて窓の道具で読めなかった）、`--remote-control` と `--restricted` の両立。3 つとも新しい窓を 1 つ開けば分かる。

## 4. GitHub の配り元 — 足しても大本の remote への道は増えない（deduced）

- tsuzuri の remote: `origin` は GitHub の HTTPS の URL（fetch と送り出しが同じ）。送り出しは github.com（HTTPS）か ssh の github.com に向く。
- 足す候補は raw.githubusercontent.com（raw の file）・codeload.github.com（tar.gz・zip）・objects.githubusercontent.com（release の資産）。どれも**配るだけの host**で、git の protocol も API も受けない。github.com 本体と api.github.com と ssh は引き続き断られるので、`git push`・`git fetch`・PR の作成・issue の投稿の道は増えない（ADR-29 決定 (7)(エ) の「送り出しと投稿の道を持たない」は保たれる）。
- 資格の file（gh・ssh・git-credentials・Claude の credentials）は囲いから見えない（report §1 で確かめた）ので、private な repo の tar.gz も取れない。取れるのは public の物だけ。
- 得る物: OSS を code として窓に置いて撃てる（今は WebFetch で読むだけ）。例: `curl -L https://codeload.github.com/obra/superpowers/tar.gz/refs/heads/main | tar xz`。
- 変更の範囲: tsuzuri-core `consult/launch.rs` の `DOMAINS` に 3 つ・fixture・歯 `cwarg_audit_refuses_wide_permissions` の「github.com を足すと断る」は残す。ADR-29 決定 (7)(エ) の字の改め（持ち主の裁定）。

## 5. 試しの記録（repo の写し work/tz-build・HEAD 4b0fa8b8・2026-10-05T00:24Z）

- 写し: crates 3 つ（contract・core・boundary）・tests/fixtures・design-intent・folio2（target と .git を除く全部）・Cargo.lock。根の Cargo.toml は面の crate と xtask を外した物（work/tz-build/Cargo.toml）。folio の build.rs が folio2/design-intent/preview/parts.json・constitution.yaml・vendor/archify を、folio の src が include_str で folio2/tests/fixtures/figure を読むので、folio2 は丸ごと要る（repo の外で tz を組む時の学び）。写しと組みは work/tz-sync-build.sh、変更の当ては work/tz-apply-patch.py。
- 土台（変更の前）: cargo test -p tsuzuri-core --test teeth3 → 27 passed 1 failed（cwquo が design-intent/rules.yaml を要る → design-intent を写して解消）。
- 変更の当て: 置き換え 17・末尾の歯 1・新しい file 4 が全部 1 回ずつ当たった。差分は work/patch/consult-statusline-remote.patch（411 行）。
- 組み: `cargo build -p tsuzuri-boundary` rc 0。`cargo clippy -p tsuzuri-core -p tsuzuri-boundary --all-targets` rc 0・warning 0（根の lint の表と clippy.toml の閾値のまま）。
- 歯: `cargo test -p tsuzuri-core --test teeth3` → **32 passed 0 failed**。新しい歯 4 本: cwarg_audit_refuses_remote_control_mismatch（話す窓に無い・問う窓に在る・名が違う を断る・settings の statusLine の字）・cwsts_command_names_the_verb・cwsts_context_percent_reads_used_percentage_only・cwsts_render_joins_the_present_fields。既存の cwarg_matches_fixture は新しい fixture と一致。
- demo（組んだ tz をこの窓の作業場で撃った）:
  - stdin に `{"context_window":{"used_percentage":37.2}}` → `窓 cw1 ｜ 題 テスト ｜ fable・xhigh ｜ 所見 2 ｜ 束 3766ac77 ｜ 文脈 37%`（rc 0）
  - stdin 空 → 文脈の欄なしの 1 行（rc 0）。作業場でない dir → `窓 ?`（rc 0）。引数あり → 「引数を取らない」rc 1。`tz consult nothing` → usage に statusline が並ぶ。
- 撃っていない（席か持ち主の手・新しい窓を 1 つ開けば分かる）: 対話の窓で `--settings` の statusLine が描かれる事・`--remote-control` と `--restricted` の両立・Claude Code が渡す JSON の鍵が `context_window.used_percentage` である事。
- 残る手（席）: 判断の記録 ADR-29 決定 (7) の字の改め（settings の列に statusLine・話す窓の argv に Remote Control）・設計ノートに行 cs-statusline・契約表の行・便。差分はそのまま契約の材料になる。
