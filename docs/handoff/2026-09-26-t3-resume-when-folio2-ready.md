# t3 の再開の手引き — folio2 が道具として使えるようになったら、ここから動かす（2026-09-26・設計席）

持ち主の裁定（2026-09-26T14:43Z・逐語は t3-hub.1）: folio2 の「天井の空回り」と「致命的な使いづらさ」の修正が落ち着いて道具として現実的に使えるレベルになってから t3 の開発を進める。設計だけは先に揃えておく（同 15:0xZ）。
この file は「再開の合図が来たら何をどの順で撃つか」だけを持つ。設計の中身は正本（design-intent/）が持ち、ここには写さない。

## 0. 再開の合図（verified・folio2-6b の回答 2026-09-26 夜）
- folio2 の群 A の 8 便（持ち出しの食い違い: .189 init の骨格と面の生成器・.196 憲法の雛形・.191 id に焼かれた folio2 の名・.190 引用符付き id・.193 凍結の罠・.194 R-17 の無い置き場・.195 値域を狭める変更・.185 説明の字）が着地した時点で、folio2 席が tsuzuri 席へ SendMessage で知らせる（持ち主にも報告）。
- 「道具として使える」の判定点 = 外の repo で `folio init` → 正本を書く → `check` / `inject --check` / `schema --check` / `derive --check` / `build` / `ceiling --stamp` / `--gate` が人の手直しなしに通る。tsuzuri は init を使わず手で正本を書いたので、今でも check と derive は通る。

- **実測（2026-09-27T03:1xZ・folio binary = folio2 8472b5c 相当）**: 群 A の 8 本のうち 7 本 closed（残り .185 = 説明の字）。新しい置き場では init → commit → inject・schema・derive・graph・build が通る（凍結の前なので check は まだ分からない 2）。**tsuzuri の正本では folio build が止まる**（design-note/surface.yaml の承認欄 = 床は一覧・面の生成器は表で読む・folio2 の .243・便 163 は契約だけ）→ ceiling も止まる。再開の判定点は「tsuzuri で build と ceiling が通る」まで待つ。folio2-13 に最短の列と見込みを照会中。

- **folio2-13 の回答（2026-09-27T03:2xZ）**: 自己点検で「一括 30 の改訂 e / f は周を落とさないための磨きに手を広げていた」と認め、一括 30 を凍結して席の先頭を tsuzuri の道の便に切り替えた（持ち主の叱責 12:1x JST）。最短の列 = 便 163（.243 承認欄を面も一覧で読む）・165（.247 面が counts を数えて導く・promise と scope を任意に）・166（.248 要件の adrs を根拠の札に）。見込み 2〜3 時間（推測）。着地で tsuzuri 席へ知らせる（binary の更新 → schema --write → 5 か所 → ceiling の写しでの実測を添える）。48〜52 周目は字の上では 09-26 の裁定の範囲内（53 周目が 1 回残る）。(1) に要らない枝（一括 30・ADR-29・d162・d164・d168）は後へ回した。d162 が着地すると tsuzuri は schema --write で 4 file の書き直しが要る。
- **tsuzuri 側の 5 か所は 2026-09-27 に直した**: 要件書 v0.3（道具の登場人物 1 つ = tsuzuri・入れる側 4・出る側 4・要件の basis は憲法の条だけ）と、設計ノート surface の章 21 → 9（便の散文と契約表を surface-base〔便 a〜f・章 8〕と surface-board〔便 g〜k・章 7〕へ字を変えずに移した・surface-base と surface-board は持ち主の承認（2026-09-27T03:19Z「よい」）で effective）。導出物は contracts/surface-base.toml と surface-board.toml（旧 surface.toml は外した）。t3-hub.2 の pointer は contracts/surface-base.toml#a。写しで承認欄を表にして build を撃つと、次の止まり所は「srs.yaml.meta に counts が無い」= 便 165 の着地待ち。

- **再開の合図が来た（folio2-d5・2026-09-27T05:0xZ）**: 便 163・165・166・168 が着地し target/debug/folio は 62111b1。tsuzuri の正本（380cf8b）で席が撃って確かめた（verified）: check 合格 0/0・schema --check 一致（書き直し不要）・derive --check 一致・**build --write rc 0（17 file）**・graph --digest 節点 223・ceiling --write rc 0（4 観点）・--gate は実装だけの write-set で「通す」。inject --check は rc 2（folio の inject が上限の行を id R-2 で決め打ち）だが **tsuzuri は inject を使わない**（ADR-8 決定 (4)）ので判定の外と folio2 に回答。**天井の周は tsuzuri では回さない**: folio2 の構造の直し（A = 発効した判断の記録を凍結された来歴に・B = 天井が運用の規則を読まない・C = 周は止めるの確かめと M の出口だけ・門は直していない支持の止めるだけを見る）が着地するまで（folio2 の勧め・持ち主が 09-27 に承認した直し・着地で知らせが来る）。A の後は ADR-7 / 8 の改訂の欄の扱いが変わる見込み。**残る手番 = 持ち主の再開の裁定 → t3-hub.2 の defer を戻す → tsuzuri の dispatch の周を 1 回起こす**（席は launch の権能が無い＝scribe2 席か持ち主が撃つ・runner と lens の行の雛形は ~/.cache/scribe2-admin/mk-run.sh と同じ形）。不確か（uncertain）: 便 a が common-verify を cargo の行に変えた後、次の便の受付が入口の flip の宣言（entrance-flip = unmeasured のまま）を求める見込み（scribe2 pipeline.md §5 の NoEntranceRed）＝便 b の前に確かめる。

## 1. 揃えてある設計（正本の置き場・全部 folio2 の形・`folio check` 合格）
| 何 | 置き場 | 状態 |
|---|---|---|
| 判断の記録 ADR-1〜8（名・面の技術・合流の順・表示面・裁定面・tz の 1 binary） | design-intent/adr/ | accepted（承認欄に逐語と裁定 id） |
| 憲法 v1.0（凍結）・規則の表 R-1〜R-17 + 面の便の行・語彙表（面と器の語） | design-intent/ | 凍結 anchor 合格 |
| 要件書 v0.2（FR1〜FR17・NFR1〜3・AC1〜14） | design-intent/srs.yaml | draft（面の便の要件） |
| 設計ノート surface（部品・口・歯・便 a〜k の契約表・節点 20・辺 31・不変条件 12・契約の型・器に頼む口） | design-intent/design-note/surface.yaml | effective（持ち主 14:37Z「よい」） |
| 器が読む導出物 | contracts/schema.toml（scribe2 fd39667 の写し）・contracts/surface.toml（`folio derive` の出力・差分 0） | 着地済み |
| 見本（本番ではない） | ~/projects/local-projects/tsuzuri-site/mock3/（配信 http://100.127.217.108:8101/mock3/）・受入の測定 script は席の作業場 fix1/ fix8/ fix11/ | 確定（直し 11 まで） |
| 討論の記録 | docs/grill/2026-09-25-B-ruling-surface.md（裁定面）・-C-（表示面）・-A-（合流の形） | 経緯つき |

## 2. 再開の手順（上から順・各段は 1 つの命令か 1 つの問い）
0. **remote** = 済（2026-09-26T14:56Z 持ち主「１．よい」→ GitHub の private repo shuu5/tsuzuri を origin として作り main を push・.vessel.toml に remote = "origin"）。以後 commit は origin へ push する。
1. **folio2 の binary を取り直す**: `cd ~/projects/local-projects/folio2 && git pull && cargo build`（席の権能で撃てなければ folio2 席に頼む）。
2. **床を撃ち直す**: `folio check --dir design-intent`・`folio schema --write --dir design-intent`（生成区間の差があれば書き直して同じ commit）・`folio derive --dir design-intent --out ../contracts --check`。落ちたら直してから先へ（合格 = 出口 ①②）。
3. **folio2 側の字の変更を写すか決める**: 要件書の自己規定（meta.effective 等）が変わっていたら、tsuzuri の srs.yaml を揃えるかは設計席の裁量（床は字を検査しない）。
4. **器の側の前提（§3）を消し込む**: 未回答が残っていれば scribe2 席へ再送。
5. **便 a を器の列に戻す**: `bdw update t3-hub.2 --status open`（defer を戻す）→ notes に「再開（合図 = folio2 の群 A 着地 <日付>）」の 1 行。
6. **t3 の dispatch の周を起こす**（§3 の 2・管理席か持ち主の手）→ 列に t3-hub.2 が載る（`[DISPATCH] bead=t3-hub.2 …`）ことを確認。受付で断られたら理由（Refuse の名）を grill B §18 に記録して直す。
7. **便 a の着地の後**: 便 b〜k の bead を契約表の depends のとおり順に起票（`--deps blocks:<前の bead>`・pointer = `contracts/surface-base.toml#<行 id>（便 g〜k は surface-board.toml）`）。1 本ずつ。
8. **code の持ち込み（ADR-4 決定 (3)）**: 便 b の後、scribe2 と folio2 の code を履歴つきで持ち込む段は別の判断の記録（crate の名の扱い = ADR-8 決定 (6)）。便 c 以降はその後。

## 3. 器の側の前提（scribe2-aa の回答 2026-09-26T15:2xZ・verified = scribe2 main 8198b92）
1. **remote と branch**: `.vessel.toml` の remote は任意（無ければ land は push を省き `remote=none`）。ただし器は base を `refs/remotes/origin/main` の名で読むので、**branch `main` と `origin` の名の remote が要る**（local の bare でも可・実績は GitHub の private repo・CI の gate〔ci-cmd〕を使うなら GitHub）。→ branch は 2026-09-26 に master から main へ改名済み。remote は持ち主の承認（14:56Z）で GitHub の private repo shuu5/tsuzuri を origin として作った（§2 の 0）。
2. **t3 の dispatcher**: tsuzuri の列は動いていない。動かすのは tsuzuri 側の手番で、`scribe2 pipe dispatch --state-dir <tsuzuri の state dir> --repo <tsuzuri> --runner "<runner の行>" --lens "<lens の行>"` を 1 回撃てば以後は便の終端の周が次を自動で撃つ（host の面の宣言は不要）。runner / lens の行の雛形は scribe2 の admin の道具（~/.cache/scribe2-admin/mk-run.sh の出力 runner.cmd / lens.cmd）と同じ形で binary と claude の包みの path を tsuzuri 用に置く。**設計席（orchestrator）は launch の権能を持たないので、撃つのは管理席か持ち主**。
3. **端末の行（device）**: scribe2 側に memo は無い。tsuzuri の要件 FR16 として設計だけ置く扱いで合意。scribe2 の台帳に memo **s2-07l.658**（起票済み 2026-09-26・昇格条件 = 持ち主の裁定・host の面の表が増えるので ADR 条件）。
4. **器の CLI**: `scribe2 seat heartbeat off|on|status --state-dir S --target T` は在る。待ちの席へ 1 行を送る口 **`scribe2 seat deliver --state-dir S --target T --ruling <記帳 id>` も着地した**（scribe2 main bed08a9・memo s2-07l.659 の昇格・持ち主経由の連絡 2026-09-27）。送る行は器が固定した 1 行（「scribe2 seat: 裁定 <id> が届いた（在りかは裁定面の記帳）」）、記帳 id は ASCII の英数字と . - _ : の 64 byte 以下、送るのは登録 row が在り最終の打刻が Idle で入力欄が空の席だけ。rc 0 = 届いた・rc 1 = 断りか未確認と理由の 13 語・rc 2 = event log が読めない。→ 設計ノート surface v0.2 の §20（記帳 id の形）と §21（口の行）に反映済み。**器には席どうしの連絡の経路が無いので、器の側への連絡は持ち主経由**（scribe2 orchestrator の言）。
- 群の訂正: tsuzuri は 2026-09-26T09:40Z の群の再編で Tier1 → **Tier2**（Tier1 は scribe2 と folio2）。14:37Z に Tier2 の記録が black5 になり t3:orchestrator は black5 で起こし直された。

## 4. 待ちの間に設計席が進めてよいもの（実装は起こさない）
- 面の便の契約表の磨き（行の write-set と verify の名の見直し・歯の fixture の置き場の規約）。
- ADR-5 の端末の一覧の行の中身（§3 の 3 の回答の後）。
- 論点 A の残り（trunk を xtask の下に = 便 a の中・.vessel.toml の cargo 化 = 済）。
- 規則の行の値の凍結の材料（面の便の最初の実測で書く・not_frozen）。

## 5. 読む順（起こし直された席）
docs/handoff/2026-09-24-s3-design-seat-resume.md §0 → この file → design-intent/design-note/surface.yaml → docs/grill/2026-09-25-B-ruling-surface.md §18（末尾から）。
