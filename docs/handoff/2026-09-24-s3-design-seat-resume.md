# t3:design 席の作業記憶（旧 s3:design・改名後に再開・2026-09-25T01:23Z 更新）

## 0. 改名後の現在地（2026-09-25T01:23Z・verified）
- repo = ~/projects/local-projects/tsuzuri（旧 scribe3・ADR-6）。state dir = ~/.local/state/scribe2-v2-state-tsuzuri。tmux の対象 = t3:design・口座 black6。作業場 = ~/projects/local-projects/tsuzuri-spikes・site = ~/projects/local-projects/tsuzuri-site。
- **憲法 v1.0 は凍結済み**（folio2 便 134 の binary・`folio schema --write` 275ae53 → `--freeze-start` 5cc8892・anchors/ 3 file・`folio check` 合格 違反 0 まだ分からない 0）。folio の binary は ~/projects/local-projects/folio2/target/debug/folio（33a7dd4）。
- **台帳は起こした**（`bd init --prefix t3`・6383e85）が、**最初の bead（根の epic）を起票の門が create-without-parent で断る**（bead 0 本の台帳は根を置けない = 器の穴）。持ち主に 1 問（当座: 裁定で rules 行 ledger.denied_writes から語を 1 つ外して根の epic を置く／本式: 門に「bead 0 本の台帳の最初の create は通す」を足す便）。scribe2 の台帳 **s2-07l.622**（memo）に起票済み。scribe2 席の候補: (1) 門に「bead 0 本の台帳の最初の create だけ通す」根の例外（§10 の却下の見直し）／(2) init の段に「根の epic を 1 本置く」を足す（init の子 process の bd は席の PreToolUse に当たらない・scribe2 席の推奨・s3 席も同意 = 器が根を置くのが「人が打つのは init と doctor だけ」の裁定に沿う）。**持ち主の裁定（2026-09-25T01:5xZ）「推奨で進めて」= 本式 (2)・当座は語を外して根を置く。scribe2-f1 に手順 (a) 語を外す → (b) s3 席が根の epic と子の memo「裁定の控え」を create → (c) 語を戻す、を依頼済み（idle 通知を購読）。(b) は「外した」の 1 行が来てから。**根が置けるまで裁定の逐語は引き続き scribe2 の台帳 s2-07l.214 の notes へ（scribe2 席が記帳）。
- 生きた server: 8791〜8793（相対 path・cwd が改名に付いてきたので無変更）・8803（v0.4 の binary で起こし直し・pid は tsuzuri-spikes/live/stage/pid）・hub :8100 と site :8101（tailnet-serve を新 docroot で登録し直し）。
- 次 = 根の bead の裁定 → 論点 B（裁定面の GUI）→ 論点 A の残り（面の便の契約表・.vessel.toml の cargo 化・trunk を xtask の下に）・ADR-5 (2) の端末の一覧。
- 以下は改名前の記述（経緯として残す・path の scribe3 は tsuzuri と読む）。


次の席が最初に読む file。読む順: この file → docs/handoff/2026-09-24-v3-kickoff.md → docs/handoff/2026-09-24-v2-leftovers-for-v3.md（scribe2 席の送り物・c17b4b2）→ docs/grill/2026-09-24-D-tech-stack.md。

## 1. いまどこか（2026-09-24T22:08Z 更新・器が口座を移動中）
- **裁定あり 2026-09-24T22:08Z**: 持ち主「それでうまくいったんならRustがいいんじゃない？」= 面の技術は Rust の wasm（Leptos）。ADR-3 は **accepted**（裁定 id = user 2026-09-24T22:07Z・記帳先 s2-07l.214 notes・scribe2 席が記帳）。契約 v0.2 は席の決定として design-note/bakeoff-surface.yaml §7 に書いた。論点 A は裁定「よい」（2026-09-24T22:22Z）→ ADR-4（proposed・裁定 id は scribe2-2a に依頼中・届いたら approval 欄に写して accepted）。論点 E は裁定「これでよい」（user 2026-09-24T22:39Z・s2-07l.214）。憲法 v1.0 は **effective・binding true・承認欄と rules 全行に裁定 id 記入済み**。始まりの凍結は「列の根の表に scribe3-constitution の行が無い」で断られ、digest 35eb6b369f0504167571a27b50c950e1361609d9b19b71e0f1e9de832f8c5356 を folio2-dc へ送付済み。**次の手 = folio2 が行を足した folio を組み直したら `folio check --dir design-intent --freeze-start` → anchors/ を commit。凍結前に憲法と rules の承認欄・裁定 id を 1 字も変えない（digest が変わる）**。凍結後: 論点 A の残り（面の便の契約表・.vessel.toml の cargo 化・契約表は凍結の後に書く = contract-table の外部 schema が無く「まだ分からない」が増えて凍結を塞ぐため）。論点 C: 持ち主の裁定（2026-09-24T22:50Z）= 対話は最初から・menu 無しの独立窓を remote / local で。持ち主の裁定（2026-09-24T22:57Z）= **Rust + Chrome で click と入力の反映まで試作して検証してから決める**。作業場の contract-stage.md（v0.3）と target-app/ を置き、agent 2 つ（spike-stage-server・spike-window・opus）を起動。**席が 2026-09-24T23:3xZ に起こし直され（session 名 scribe3-13）、背景の agent 2 つは消えた（足場のみ残存）ので同名で再起動した（23:40Z）。scribe2-2a / folio2-dc も ListAgents に見えない = 相手も起こし直しの可能性・要るときに ListAgents で名を確かめる。scribe3 に台帳（bd）は無い（heartbeat の bd ready は no database）。**試作 2 つとも完了（2026-09-24T23:57Z）。結果は grill C §9 と設計ノート bakeoff-surface §8。生きた窓 = stage-server を tailnet IP の port 8803 で起動（pid は spikes/live/stage/pid・Chrome は子 process）・hub に card と命令の送り口を追加。持ち主の実測（2026-09-25T00:16Z）= tailnet 越しで遅い・日本語入力が二重 → **解決は必須（裁定）**。原因を code で特定し契約 v0.4（作業場 contract-stage-v04.md）を書いて agent 2 つを起動。**持ち主の本来の要件（2026-09-25T00:24Z）= 席が持ち主の端末の Chrome を起動して操作する → ssh + app mode + CDP tunnel で実証済み（grill C §11・ThinkPad に窓が開いている・tunnel は scratchpad/tunnel.pid）。画面配信の窓は端末に届かない周の代替。v0.4 の直し agent 2 つは続行中（代替の質を上げる）。持ち主の裁定（2026-09-25T00:28Z）「スムーズに動く。それで決めて良い」→ ADR-5 は accepted（user 2026-09-25T00:27Z・s2-07l.214）。論点 G: **持ち主の裁定（2026-09-25T00:39Z（名）/ 00:42Z（prefix））= 名は tsuzuri・CLI は短く・prefix は CLI と揃える → ADR-6 accepted・憲法の id を tsuzuri-constitution に改めた（凍結前・digest が変わるので folio2-f5 へ新 digest を送る）。**綴りの裁定（00:55Z）= CLI tz・prefix t3（3 は v3 の印）→ ADR-6 (3)(4) に記入済み。台帳は改名後の dir で `bd init --prefix t3`。**folio2-f5 の返事（00:55Z）: 便 133 は鍵 scribe3-constitution で本流 3269ff4 に着地済み（行き違い）→ 鍵を tsuzuri-constitution に改める便 134（digest 不変）を起草中・**始まりの凍結は便 134 の着地 commit が届くまで待つ**（1〜2 時間・口座 black3 が上限に近い）。床の裁定 id の形（英字 1 + 数字 1）は folio2 の控え f2-648.203 に起票済み・**prefix は t3 に決まり床の直しは不要（folio2-f5 へ送付済み）・repo の dir が tsuzuri に変わったら folio2-f5 へ 1 行送る（契約の出所の path 合わせ）**。改名は席を閉じてから docs/handoff/rename-to-tsuzuri.sh（RUN=1）で 1 度に行い、scribe2 席が席を登録し直して起こす（手順書 docs/handoff/2026-09-25-rename-to-tsuzuri.md）。新しい席は改名後の dir で再開する。**名が決まったら: ADR-6（名と可変の形）→ 改名の手順（repo dir・state dir・session・登録 row を scribe2 席と 1 度で）→ 台帳（prefix は名から）→ B。答えが来たら s3 席が bd init → epic と memo → commit → 以後の裁定は s3 の notes へ。次 = B（裁定面）。v0.4 の直し agent 2 つは続行（代替の窓）**。folio2-dc は便 133 で列の根の表に行を足す（数時間・着地の commit が届く）。持ち主は「移動はしない・討論を続ける」と言明（hook の表示は移動中のまま・持ち主の言を優先）。
- その後: 契約 v0.2（作業場 contract.md §7 の論点を決める）→ 論点 A（合流の形・init 1 発・scribe2 席の設計 doc の path を待つ）へ。生きた server 3 つ（port 8791〜8793）と hub（tailnet-serve）は動いたまま。止めるなら spikes/live/*/pid と ℹ 稼働中のサーバはありませんでした: /home/shuu5/projects/local-projects/scribe3/…/scribe3-spikes/hub。
- 論点 D: ADR-2（比較試作で決める）発効済み。試作 4 つ完了・結果は design-note/bakeoff-surface.yaml §6・作業場 ~/projects/local-projects/scribe3-spikes（自前 git）・hub は tailnet-serve（`tailnet-serve list` で URL・停止は `tailnet-serve stop --dir …/scribe3-spikes/hub`）・生きた server 3 つ（port 8791〜8793・pid は spikes/live/*/pid・止めるのは s3 席）。**本決定の 1 問（grill 記録 §8・推奨 = Leptos）を持ち主へ提示済み・答え待ち**。答えが来たら ADR-3 を書く。
- 以下は 13:1xZ 時点の記述（経緯として残す）。
- 論点 D（技術スタック）の 1 問を持ち主に提示済み・**答え待ち**。問いと推奨は docs/grill/2026-09-24-D-tech-stack.md §5（推奨 = (a) Rust の同期 server + build 無しの HTML/JS・撤退条件つき）。裁定はまだ無い＝ADR・vocabulary・srs には何も書いていない。
- 討論は「器の穴」の直しを待って止めていた。持ち主の指示: 討論の前に scribe2 側を直させる。

## 2. 器の穴（持ち主の裁定・2026-09-24T13:1xZ・逐語は scribe2 の台帳へ）
- 要点: 新しい置き場の立ち上げは `scribe2 init` 1 発で終わる簡便さが要る。今日は手作業 7 手（state dir・口座の symlink・host.toml の写し・起動行・git 設定・tmux・seat launch の長い引数）で、vessel init と .vessel.toml が抜けた。seat launch の引数の長さも同じ穴。
- 追加の裁定（13:2xZ・逐語は scribe2 の台帳へ）: 人間向けの説明が 0 で初見の人には知りようがない／人が打つ command は最小限にし、それで器が十全に動くようにする。s3 の読み = 穴は 3 つで 1 組（1 発でない・説明 0・command が多い）。目標は人が打つのは init と doctor の 2 つを上限に、残りは器が自分で行う。
- 報告先: scribe2 席（session 名 scribe2-2a）へ Claude Code の session 間メッセージで 3 通送付済み（欠落の実測 → 訂正: 器の穴として直す便に起票せよ → 説明 0・command 最小限の裁定）。scribe2 側が台帳に起票し ADR を書く。
- 起票済み（2026-09-24・scribe2 席の報告）: scribe2 の台帳 **s2-07l.609**（逐語と UTC の分つき・P1・open）。設計 doc と ADR は scribe2 側で書く。`bd --readonly show s2-07l.609` で読める。
- scribe3 側の直し（scribe2 席が実施・13:03Z）: `.vessel`（untracked・name=scribe2 version=2）と `.vessel.toml`（commit aeb81b1・allowed-commands=["git"]・common-verify=["git diff --quiet"]・requirements=design-intent/srs.yaml）。`scribe2 vessel show .` が stateDir を返すことを実測。
- v3 への持ち越し: 論点 A で「器の口として init を 1 本持つ」を要件候補に（grill 記録 §7）。

## 3. 起こし直した席が最初に確かめること
1. SessionStart の出力に `[scribe2/SessionStart]` の名乗りと orchestrator の指示文（決定はしごの行）が入っているか。無ければ state dir の inject.jsonl に session-start-header が無いことを実測して scribe2-2a へ 1 行返す。
2. `git status`（.vessel は untracked のままでよい・folio2 と同じ）。`folio check --dir design-intent` は違反 0・まだ分からない 2 が正常。
3. 持ち主の答えが来ていなければ、論点 D の 1 問（grill 記録 §5）を再掲して待つ。答えが来ていれば ADR-2（技術スタック）を design-intent/adr/ に起こし、語を vocabulary.yaml へ、要件を srs.yaml へ書く。

## 4. 決めごと（この席で確かめた運用）
- grill は 1 論点 1 問・推奨 1 つ・散文で問う（AskUserQuestion は使わない・grill-me skill）。
- 討論の記録は docs/grill/ の markdown、正本は design-intent/ の YAML。逐語は tracked に写さない（台帳へ）。
- この repo は remote 無し・master へ直接 commit（前席と同じ運用）。

## 5. v2 の残りの割り当て（2026-09-24・scribe2 席の送り物 §2 の 8 項目 → 論点）
- 論点 D（進行中）: 直接は無し。1（器が席の状態を構造で知る口）・7（run の終端 event）・8（init / doctor を GUI から撃つ）はどれも server 側の口なので、比較試作で「server と契約を固定し client だけ作り分ける」枠を補強する。
- 論点 A（合流の形・init）: 2（口座 × anchor の trust を器が持つ・serde_json は A3）・3（doctor は「何が無いか・次の 1 手」を必ず 1 行）・8（command は init と doctor の 2 つ上限）。
- 論点 C（表示面）: 1（描画でなく hook / SDK / 状態 file から席の状態を読む・literal の等値で dialog を塞ぐ形を持ち込まない）。
- 論点 B（GUI）: 8（init / doctor を GUI から撃てる）。
- 論点 E（憲法）: 5（scribe2 索引表の出所不一致 10 / 25 行と C7 要旨の古さを持ち込まない）・6（EARS の型は folio2 の門で最初から閉じる）。
- 論点 F（台帳）: 7（run の終端を event で持つ・札の不在で推さない）・§3 の open 便の扱い（.214 据え置き・.19 は送って close）。
- 要件書の受入: 4（folio v1 の SRS 生成器の 11 項目が folio2 で解けているかを srs.yaml の受入で確かめる）。

