# 画面の便を並べて走らせる段取り（2026-09-27・設計席の作業記憶）

持ち主の指摘（2026-09-27・逐語は t3-hub.1）= 最初の画面は見本（mock v3）に機能も見た目も遠い・速さを上げたい。作る順の計画（docs/handoff/2026-09-27-t3-build-plan.md・承認済み）の順は変えず、便を並べて走らせる。

## 1. 並べ方（write-set が重ならない便は同時に走る）
| 波 | 便 | 触る所 | 状態（11:40Z） |
|---|---|---|---|
| 0 | g-frame（枠と見た目）・c（導出グラフ） | 面・中核 | 着地 2a638ad・58a3ea6 |
| 1 | g-parts（block の読みの口と hover の card）∥ d（指標・未反映・次の一手・pipeline の板） | 面 ∥ 契約の型と中核 | 着地 8da13ac・715840c |
| 2 | e-read（読む側の口 4 つと board-changed）∥ g-pipe ∥ g-next ∥ b-cards（問いと席の card の型） | 境界 ∥ 面 ∥ 契約の型 | 着地 fb8053f・63f7ebb・6bdbcee・3a453c2 |
| 3 | e-ask ∥ g-ledger ∥ g-seat ∥ g-ask（問いの頁と頁の枠の直し）∥ g-map（圧縮・一覧・表） | 境界と中核 ∥ 面の module 1 つずつ | 着地 8d2faf9・e7f9754・8329a61・040bb05・3602b56 |
| 4 | c-view（辺の向き・グラフの眺め・近傍の電文）∥ g-gaps（抜けの検査の頁）∥ e-seat（席と口座の読み）∥ g-ledger-fix ∥ g-graph（地図のグラフの面） | 契約の型と中核 ∥ 面 ∥ 境界 | c-view 着地 41b138a・g-gaps 着地 9cfea63・ほかは走行中（11:40Z） |
| 5 | e-view（口 /api/graph/view・/api/around・/api/unreflected）∥ 節点の頁と近傍の図 ∥ 未反映の一覧 | 境界 ∥ 面 | 未起草（e-view は e-seat の着地の後） |
| 6 | 束の承認と方針の欄と取り消し ∥ f（hook）∥ account board（行 h） | 境界 ∥ 面 | 未起草 |

block の便は自分の module の file と自分の歯の file と fixture だけを書く（便 g-parts が関数の外形と口の path を先に決める）。枠の歯 tests/frame.rs は block の便の write-set に入れない。

## 2. 口の path（閉じた 6 本）
/api/ledger（着地済み）・/api/next・/api/pipeline・/api/seat・/api/metrics・/api/graph。変化の知らせは /api/surface/events の ledger-changed と board-changed。

## 3. 器の読み口（scribe2 の回答 2026-09-27T09:5xZ・state dir に書かないことは scribe2 が実測）
- 跨版を約束している面は 2 つ: event log `<state dir>/fleet/events.jsonl`（schema 1）と `scribe2 fleet export --state-dir S`（JSON・1 行目が header・以降 run と seat の行・過去の run も全部入る）。
- run の stage は閉じた 11 値: Intake・Reviewed・Blocked・Spawned・Questioned・RateLimited・Implemented・Gated・Landed・Stopped・Failed。Gated の判定は export に無く event log の RunStage の detail に在る。
- 口座の残量: event log の AllowanceMeasured / AllowanceUnmeasured。text の口は `scribe2 fleet usage --show --state-dir S`（`--show` を付けずに撃たない = 測り直して event を書く）。
- 席の健康: `scribe2 seat tick status --state-dir S`（target・last・age・healthy・heartbeat）。群と口座: `scribe2 doctor --state-dir S`（seat の行と group の行・current と next）。
- file を直に読む場合（版の約束は jsonl だけ）: `<state dir>/seat/<session>_<window>/state.jsonl`（busy / idle）・tick-last・account・move-signal。移動の履歴は `<state dir の親>/scribe2-host/groups/`。
- 4 状態は面の側で合成する: 動いている / 待ち = state.jsonl・応答なし = busy のまま age が seat.tick_stale_s を越えた・限度 = 残量 100% 以上か tick-last の reason=account-pressed。上限で終わった turn は busy のまま残る。

## 4. 審査の diff の上限 150000 byte
除外の宣言も値を変える口も無い（scribe2 の memo s2-07l.691・今日 2 回）。大きい fixture と写しの asset は、実装役が書いた file を設計席が先に main へ置き（tests の下か面の asset）、`scribe2 pipe follow --run <id>` で run を載せ直す。契約の行を書くときは、大きい file を作らせない（fixture は小さく・件数の多い入力は歯の中で組む）。

## 5. 着地の後の手（remote の宣言が無い間）
RunDone（Landed）を event log で確かめてから、`git status` → `cargo run -q -p xtask -- check` → push → `bdw close` → `bdw dolt push` → `bash ~/.cache/tsuzuri-admin/dispatch.sh`。RunDone の前に main で cargo を回すと器の着地の検証と lock を取り合う。着地の間は作業の木を汚さない（起草中の行は stash か席の作業場へ）。

## 6. folio2 の天井の 1 周の所見（2026-09-27・写しでの検証）
支持の 止める 6 件は tsuzuri の文書の直し（srs AC14・srs FR18・surface-board §8・憲法 P-13 の機構・P-26.2・surface §17 の問いの見分け）。画面の波の後にまとめて起こす。憲法の字に触る 2 件は持ち主に問う。門は folio2 の便 174 の着地まで撃たない。報告は ~/.local/share/folio2/handoff-2026-09-27/t3v-report.md。

## 7. 契約の行を書くときの点検（審査で止まった実例から・2026-09-27）
審査は行と節と要件と、write-set の file の宣言の名だけを見る。節に無い事実は「材料が無い」で止まる。行を書いたら次を確かめる。
1. **在りか**: 使う型は module の path と着地の番号を書く。電文の型は欄を全部書く。符号や単位の向きも書く（例: net_drop は増えた向きが正）。
2. **語の鍵と class**: 使う語の鍵を全部名指し、語の辞書に在ることを先に grep で確かめる。辞書に無い字（本文の字）は「module の定数に置く」と書く。class は規則の在る無しを確かめ、無い class を「stylesheet に在る」と書かない。
3. **依存と呼び手**: Cargo.toml を触らずに済む理由（既に依存している）と、関数の呼び手と引数（誰がどの引数で呼ぶか）を書く。
4. **write-set の外の歯**: 変える関数を呼ぶ歯の file を grep で全部探す。見つけたら write-set に足すか、関数の外形を変えないと書く。dir の中の項を数える歯にも注意する（src/project の下に file や dir を足すと枠の歯が落ちる）。
5. **歯の語**: verify の filter の語は、src の comment も含めて repo の全部で部分一致する。受付の前に `scribe2 pipe preflight` で teeth-outside-write-set を見る。
6. **節と done の一致**: done の 1 項ずつが節の字と同じ配置・同じ数を指す。数は要件と規則の行の字と 1 字も違わない。歯が「在る無し」でなく値を比べる形にし、例の値を節に書く。
7. **面に判定を書かせない**: 面が出す数は電文の欄に在るものだけ。欄に無いものを面に出させると、実装役が面で数え直して審査で落ちる。
8. **URL に残す切り替え**: query の字の読みと書きは純粋な関数にして host の歯で測ると書く。
9. **大きい file を作らせない**: fixture の大きさの上限を節に書く（審査の diff の上限 150000 byte）。

## 8. 器の口の使い方（scribe2 の回答 2026-09-27T10:1xZ〜10:4xZ）
- live の走行の行の字は変えない（終端の後に同じ bead の走行が起き直る）。別の行を足すのは安全。
- main へ commit する前に着地の窓を待つ: `~/.cache/tsuzuri-admin/land-clear.sh && git add … && git commit …`。
- 走行を止めるのは literal の 1 行だけ（変数・引用符・`cd &&`・`2>&1`・pipe を付けない）: `scribe2 pipe stop --run <id> --state-dir <絶対 path> --repo <絶対 path>`。Questioned の走行は答えが付くまで live で、write-set を変える直しは止めてから行う。
- 群の所属は `<state dir>/host.toml` の `[[account-group]]`（宣言順で最初に anchor を含む群）。tsuzuri は Tier1。
- 限度の判定（席の逼迫）は 5 時間窓と 7 日窓をつねに数え、model の窓は席の model と同じ行だけ数える。閾値は器の rules 行（85・95・95）。面は tick-last の reason（account-pressed・state-stale）を写す。

## 9. 事故の記録 — 設計席の commit が着地を巻き戻した（2026-09-27T11:08Z〜11:15Z）
- 起きたこと: anchor（main の checkout）で design-intent を編集して作業の木が汚れている間に着地が入ると、器は ref だけを進め、作業の木と index を揃えない（器の仕様・scribe2 の回答・memo s2-07l.695）。その index のまま commit して、着地 3 本（g-seat・g-map・g-ask）の 27 file を巻き戻した（48eebea・233669f）。
- 戻し: 巻き戻された file を着地の commit の中身のまま取り出して commit した（ffde5f1）。
- 塞ぎ方（今の作法）: 設計の編集は anchor でなく別の worktree で行う。置き場は `~/.cache/tsuzuri-admin/design-wt`（branch design-seat）。編集の前に `~/.cache/tsuzuri-admin/design-sync.sh`、commit は `~/.cache/tsuzuri-admin/commit-design.sh <message の file>`（folio の schema・check・derive を回し、置き場の外の差分が在れば commit せず、main へは ff で入れて push する）。anchor では file を編集しない。画面の写真の file も anchor に置かない。
- land-window の `busy queue=- following=- unpushed=<sha>` は、着地の途中でなく push 待ちの着地が残っているだけ（commit してよい）。

## 10. 実物の画面で見つけた食い違い（host の歯は通るが画面が動かない型）
- 口の本文の包み: /api/metrics の本文は Reading で包んだ形（known の鍵の下に指標・読めないときは字 unknown）。便 g-ledger は包みを読まずに落ちた（行 g-ledger-fix で直す）。契約の行には口の本文の形を書き、実物の本文の写しを fixture に置く（設計席が置ける・tests/fixtures の下）。
- 着地のたびに実物の画面を開いて、block ごとに中身が出ているかを見る（server の組み直しは `~/.cache/tsuzuri-admin/serve/restart.sh`）。
- 小数を持つ電文を読み直して比べる歯は、最後の 1 桁がずれて時刻によって落ちる。電文の字で比べる（8fdfd1b）。

- 見た目の細部の直しの候補（11:56Z に実物の画面で見た・後で 1 行にまとめる）: block「orchestrator と口座」の窓の名（five_hour・seven_day_model）が「?」と割合の字に重なる／地図と抜けの検査の頁で header の最終の記録が「まだ無い」のまま（台帳の読みが home の頁でしか走らない）。

## 11. folio2 の知らせ（2026-09-27T11:1xZ）
便 174 と便 176 が着地（folio2 main efe3e7b）。pin を上げる手順: binary を efe3e7b に替える → `folio schema --dir design-intent --write` → commit（門を通さずに入れる）→ 天井の周。番号を落とした後の注の字の小さな壊れ 4 件は、周が所見に挙げたら folio2 へ id を知らせる（folio2 台帳 f2-648.261）。画面の波の後に行う。

- **pin を c52baba に上げた（2026-09-27T12:5xZ・folio2 の便 175・177 の後の勧め）**: binary は ~/.cache/tsuzuri-admin/folio/folio → folio-c52baba（folio2 main の git archive から build）。`folio schema --write` の後に床は合格 0/0・derive 一致・ceiling.yaml と graph.yaml の「引き金」の字 0（commit 0e7d0f3）。efe3e7b は飛ばした。**天井の周（ceiling --write → 観点ごとの所見 → --check → --refute → --stamp）はまだ回していない**＝画面の波の後に回し、注の字の小さな壊れを所見が挙げたら folio2 の台帳 f2-648.261 へ id を知らせる。

## 12. push 先の宣言を戻す時機（2026-09-27T11:5xZ）
scribe2 の CI の照合の直しが着地（8f6072d・30 秒ごと・上限 900 秒）。PATH の scribe2 は入れ替え済み。scribe2 の回答: 終端は着地の直後の main の .vessel.toml を読む。古い binary の driver（`readlink /proc/<pid>/exe` が `(deleted)` で終わる）が宣言を読むと 20 ms ごとの照合をする。t3-hub.24（g-graph）の driver が古い binary なので、**t3-hub.24 が終端に着いてから** .vessel.toml に remote = "origin" を戻す（.vessel.toml の変更は live の走行を全部の審査のやり直しにする）。戻す前に、live の driver に古い binary が残っていないかを readlink で確かめる。
