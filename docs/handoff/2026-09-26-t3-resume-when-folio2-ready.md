# t3 の再開の手引き — folio2 が道具として使えるようになったら、ここから動かす（2026-09-26・設計席）

持ち主の裁定（2026-09-26T14:43Z・逐語は t3-hub.1）: folio2 の「天井の空回り」と「致命的な使いづらさ」の修正が落ち着いて道具として現実的に使えるレベルになってから t3 の開発を進める。設計だけは先に揃えておく（同 15:0xZ）。
この file は「再開の合図が来たら何をどの順で撃つか」だけを持つ。設計の中身は正本（design-intent/）が持ち、ここには写さない。

## 0. 再開の合図（verified・folio2-6b の回答 2026-09-26 夜）
- folio2 の群 A の 8 便（持ち出しの食い違い: .189 init の骨格と面の生成器・.196 憲法の雛形・.191 id に焼かれた folio2 の名・.190 引用符付き id・.193 凍結の罠・.194 R-17 の無い置き場・.195 値域を狭める変更・.185 説明の字）が着地した時点で、folio2 席が tsuzuri 席へ SendMessage で知らせる（持ち主にも報告）。
- 「道具として使える」の判定点 = 外の repo で `folio init` → 正本を書く → `check` / `inject --check` / `schema --check` / `derive --check` / `build` / `ceiling --stamp` / `--gate` が人の手直しなしに通る。tsuzuri は init を使わず手で正本を書いたので、今でも check と derive は通る。

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
1. **folio2 の binary を取り直す**: `cd ~/projects/local-projects/folio2 && git pull && cargo build`（席の権能で撃てなければ folio2 席に頼む）。
2. **床を撃ち直す**: `folio check --dir design-intent`・`folio schema --write --dir design-intent`（生成区間の差があれば書き直して同じ commit）・`folio derive --dir design-intent --out ../contracts --check`。落ちたら直してから先へ（合格 = 出口 ①②）。
3. **folio2 側の字の変更を写すか決める**: 要件書の自己規定（meta.effective 等）が変わっていたら、tsuzuri の srs.yaml を揃えるかは設計席の裁量（床は字を検査しない）。
4. **器の側の前提（§3）を消し込む**: 未回答が残っていれば scribe2 席へ再送。
5. **便 a を器の列に戻す**: `bdw update t3-hub.2 --status open`（defer を戻す）→ notes に「再開（合図 = folio2 の群 A 着地 <日付>）」の 1 行。
6. **器が拾うのを見る**: t3 の dispatch の列に t3-hub.2 が載る（`[DISPATCH] bead=t3-hub.2 …`）ことを scribe2 席に確認。受付で断られたら理由（Refuse の名）を grill B §18 に記録して直す。
7. **便 a の着地の後**: 便 b〜k の bead を契約表の depends のとおり順に起票（`--deps blocks:<前の bead>`・pointer = `contracts/surface.toml#<行 id>`）。1 本ずつ。
8. **code の持ち込み（ADR-4 決定 (3)）**: 便 b の後、scribe2 と folio2 の code を履歴つきで持ち込む段は別の判断の記録（crate の名の扱い = ADR-8 決定 (6)）。便 c 以降はその後。

## 3. 器の側で未確定のもの（scribe2 席に照会中・2026-09-26T15:0xZ・回答が来たらここを更新）
1. remote の無い repo（tsuzuri）を便の対象にできるか（land と審査の材料の取り方）。だめなら remote の形（private repo か local の bare）。
2. t3 の dispatcher の列が動いているか・動かすのは誰の手番か。
3. 端末の一覧の行（host.toml の device 行）の起票の有無（ADR-5 決定 (2)・ADR-7 決定 (14)・要件 FR16）。
4. 席の停止の切り替えの command 名と、待ちの席へ 1 行を送る新設の口の起票の有無（要件 FR9・FR12）。

## 4. 待ちの間に設計席が進めてよいもの（実装は起こさない）
- 面の便の契約表の磨き（行の write-set と verify の名の見直し・歯の fixture の置き場の規約）。
- ADR-5 の端末の一覧の行の中身（§3 の 3 の回答の後）。
- 論点 A の残り（trunk を xtask の下に = 便 a の中・.vessel.toml の cargo 化 = 済）。
- 規則の行の値の凍結の材料（面の便の最初の実測で書く・not_frozen）。

## 5. 読む順（起こし直された席）
docs/handoff/2026-09-24-s3-design-seat-resume.md §0 → この file → design-intent/design-note/surface.yaml → docs/grill/2026-09-25-B-ruling-surface.md §18（末尾から）。
