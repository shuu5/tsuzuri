# 設計: 便 45 — 欄の決まりの file の生成区間を床の定数から導出する folio schema（--write / --check）と、判断の記録の欄の決まりの説明の注を床の定数の側へ（ADR-9・FR19 / AC17）

- 要件: FR19（欄の決まりの file の決まりの部分を床の定数から導出する）/ FR6（憲法の生成区間 = 同じ型）/ FR5（3 値）
- 条: P-6.2・P-6.3・P-6.4（正本は 1 つ・ほかは導出・生成物を手で直さない）/ P-14.2（生成区間と正本の差分を検出する型）/ P-5.5（裁定の正本は設計文書 = 生成区間の外は書き換えない）/ P-10.1（凍結 anchor は生成側から独立）/ N-3.1（data 側で緩められない）
- 判断の記録: ADR-9（発効 2026-09-19・決定 = schema 節だけを生成区間に・folio check の置き場ごとの検査は現行のまま・byte 一致は folio schema --check）。本便は 2 便のうち 1 本目（生成器と判断の記録の側）。2 本目（便 46）= 設計ノートの側。
- 裁定: 持ち主 2026-09-19「どちらも承認する」（ADR-9 の発効）・「承認する」（要件書 v1.7 = FR19 / AC17）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 at が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は + の接頭辞・縮む file は無い）。

## 1. 目的と中身

欄の決まりの file 2 本（design-intent/adr/schema.yaml・design-intent/design-note/schema.yaml）は、planner の取り込みで schema 節が生成区間の体裁に書き直してある（印 2 本で挟んだ・体裁は本節 (c) の規則・planner が folio を使わず独立の Python で組んだ = 凍結 anchor の元）。本便は (1) 同じ byte を床の定数から導出する命令 folio schema（--write / --check）を足し、(2) 判断の記録の欄の決まりの説明の注（名が _note で終わる欄 22 本）を床の定数（adr.rs の FLOOR）の側へ移し、(3) 2 本の file に重複している床の機械を新しい file に寄せる。folio check の置き場ごとの検査（欄単位の突き合わせ）と床の出す文言は 1 字も変えない。設計ノートの側（note.rs・design-note/schema.yaml）は便 46 で運び、本便では触らない。

planner の実測（2026-09-19・main d44c893 + 本文書の取り込み）: crates/folio/src/adr.rs（正規化 1,050・余地 450）と note.rs（1,349・余地 151）は、床の木の型 Floor（adr.rs 18〜29 行は 4 変種 Val / Num / Strs / Map・note.rs 34〜47 行はそれに Seq を足した 5 変種・どちらも非公開）・マクロ keys_floor（adr.rs 105〜112 行）・関数 strip_notes（adr.rs 337〜349 行）・関数 floor_diff（adr.rs 351〜405 行）を別々に持つ。FLOOR（adr.rs 114〜204 行）は schema 節の値の欄と 1 対 1 で、_note の欄は 1 本も無い。床の違反の文は adr.rs 270〜275 行の「adr/schema.yaml schema.<道> が床の定数と違う（欄の決まりの閾値・値域・置き場は床の定数の写し＝data 側で動かせない・N-3.1）」で、道の末尾の飾りは「（未知の欄＝機械が読まない欄は *_note で終える）」「（欠落）」「（欄の表でない）」の 3 種。憲法の生成区間の実装 inject.rs（正規化 264）は 印 2 本・region_of（印がそれぞれちょうど 1 本で begin が先のときだけ区間を返す）・導出 1 関数を --write / --check が共有する型で、verdict.rs の終了コードは 合格 0 / 不合格 1 / まだ分からない 2。main.rs（正規化 516・余地 984）は mod の宣言 34 本と副命令 11 個。凍結 fixture の写し tests/fixtures/ の下の adr/schema.yaml 16 本は最小の手書き（45 行・_note なし・印なし）で、tests/floor_cases.yaml の場合は file を丸ごと書き直すので印が消える = folio check は印を見てはいけない。

(a) 命令の口（main.rs）: folio schema --dir <設計文書の置き場・既定 design-intent> に --write か --check のどちらか 1 つ（排他・必須・inject と同じ ArgGroup の形）。対象の file は本便では adr/schema.yaml の 1 本（便 46 で design-note/schema.yaml を足せるよう、対象は schema.rs の中の一覧 1 つで持つ）。標準出力は 1 行、理由は標準エラーに「folio schema: <理由>」。
- file が無い・読めない・symlink → 2「<file>: 読めない」。
- 印が 1 対でない（begin か end が 0 本・2 本以上・逆順）→ 2「<file>: 印が 1 対でない」。
- --check: 生成区間の byte が導出と同じ → 0「folio schema: 一致（<file>・<byte> byte）」／違う → 1「<file>: 生成区間 <数> byte ≠ 導出 <数> byte」。
- --write: 生成区間を導出で置き換えて書く → 0「folio schema: 書いた（<file>・<byte> byte）」／既に同じなら書かずに 0「folio schema: 変わらない（<file>・<byte> byte）」。生成区間の外の byte は 1 つも変えない。

(b) 印と生成区間: 印は行頭からの注の行 2 本。begin =「# folio:schema:begin — 生成区間・手で直さない・正本は実装の定数（folio schema --write が書く）」、end =「# folio:schema:end」（どちらも行の全部がこの字面・前後に空白なし）。生成区間 = begin の行の改行の次の byte から、end の行の先頭の byte の手前まで。中身は「schema:」の行 + (c) の規則で組んだ本体の各行で、各行の末尾は改行 1 つ（最後の行も）。

(c) 導出の体裁の規則（W = 100・字数は Unicode の字の数・行は字下げ込みで数える・2 本の file に共通・schema.rs の関数 1 つ）:
1. 表を block で書くとき、子は「<字下げ><キー>: <値>」。字下げは半角の空白で深さ × 2（schema の直下が 2）。キーの順は FLOOR に書いた順。
2. 文字列と数の一覧: flow の 1 行「<字下げ><キー>: [a, b]」（区切りは半角の読点 + 空白）が W 以内なら flow、超えれば block（キーの行の下に各項「<字下げ + 2>- <値>」）。空は [] 。
3. 表: 子が全部「文字列・数・真偽」か「その一覧」で、flow の 1 行「<字下げ><キー>: {k: v, k: [a, b]}」が W 以内なら flow、さもなくば block。空の表は {} 。
4. 表の一覧（便 46 の設計ノートの側だけに在る）: 各項を 3 の flow で「<字下げ + 2>- {…}」、W を超える項は block（1 つ目のキーを「- 」の後ろに、残りのキーを字下げ + 4 に）。
5. 値の字面: 数と真偽（true / false）は裸。文字列は次のどれかに当たれば単引用符で囲み（中の単引用符は 2 つ重ねる）、さもなくば裸 = 空の文字列／先頭か末尾が空白／先頭の字が 角括弧・波括弧・# ・& ・* ・! ・| ・> ・単引用符・二重引用符・% ・@ ・逆引用符 のどれか／先頭が「- 」「? 」「: 」／「: 」（半角コロン + 空白）か「 #」（空白 + #）を含む／末尾が半角コロン／flow の中でだけ 半角の読点・角括弧・波括弧 のどれかを含む。
実の file で単引用符が付くのは adr/schema.yaml の ruling_pattern の 1 か所だけ（先頭が角括弧）。二重引用符を含む値が 1 つ在る（amends_note の value の中）が、先頭でないので裸のまま = .rs の側では生の文字列の字面で持つ。

(d) 凍結 anchor（P-10.1・生成側から独立）: main の design-intent/adr/schema.yaml の生成区間は planner が独立の Python（(c) の規則の別実装・PyYAML で読んで書き、読み直した木が元と等しいことと 2 度かけて byte が変わらないことを確かめた）で組んだ。生成区間 = 117 行・22,182 byte・sha256 = 8c53aa96fad9bddf83fb04d33113124b79bac036aea3fa2a12249e37f65ce998（begin の次の行から end の手前の行まで・sha256sum で実測）。作業者は FLOOR に _note の欄を file に在る順と字面のまま足し、folio schema --check が実の file に対して 0 を返すまで合わせる。食い違ったら (c) の規則か FLOOR の字面が本節と違うということで、design-intent/adr/schema.yaml の側を書き換えて合わせてはいけない（変えるなら planner へ問う）。

(e) 床の機械を寄せる（新しい file crates/folio/src/schema.rs・crate の中へ公開）: 床の木の型 Floor（5 変種 Val / Num / Strs / Map / Seq）・マクロ keys_floor・strip_notes・floor_diff・印の定数・区間を取る関数・(c) の導出・run（--write / --check）を置き、adr.rs は自分の Floor / keys_floor / strip_notes / floor_diff を消して schema.rs のものを使う（差し引き約 +80 行）。note.rs は本便では触らず自分の写しを使い続ける（便 46 で寄せる）。FLOOR に _note の欄を足すと、floor_diff のキーの和集合に FLOOR 側の _note の欄が入り「（欠落）」が 22 件立つので、floor_diff は FLOOR 側の名が _note で終わるキーを和集合から除く（data 側は strip_notes 済み）。床の違反の文と飾り 3 種は字面のまま運ぶ。adr.rs が crate の中へ公開している読み口（floor_strs / floor_val / floor_num・link.rs と anchor.rs が使う）の名と振る舞いは変えない（link.rs・anchor.rs は write-set の外で、Floor の型を名指していない = 読み口の戻り値は文字列と数だけ）。

(f) 歯（新しい file crates/folio/tests/schema.rs・関数名は全部 schema で始める・folio は実行 file の crate なので歯は命令を撃つ）:
1. 実の正本: design-intent の写しに --check → 0 ∧ 標準出力に「一致」∧「22182 byte」。生成区間を sha256sum で測り直して (d) の値と同じ。
2. ずれ: 写しの生成区間の 1 byte を書き換えて --check → 1 ∧「生成区間」∧「≠ 導出」。
3. 印: begin を消す → 2 ∧「印が 1 対でない」／end を 2 本に → 2／begin と end を入れ替える → 2。
4. 書き直し: 歯 2 の写しに --write → 0 ∧「書いた」∧ file 全体が元の file と byte 一致（生成区間の外も不変）。もう 1 度 --write → 0 ∧「変わらない」。
5. 旗: --write と --check の両方 → 2／どちらも無し → 2。
6. file が無い置き場 → 2 ∧「読めない」。
7. 床は印を見ない: 写しの印 2 本を消して folio check → 写しに git が無いこと以外の違反 0（歯は git の初期化を済ませた写しで 合格 を見る・tests/ceiling.rs の Work と同じ作り方）。
既存の歯は期待不変で回帰に回す: tests/adr.rs（8 本・fixture adr/schema-drift の違反の文が 1 字も変わらないこと）・tests/floor_cases.rs（1 本・134 の場合・うち adr/schema.yaml を path で変える 16 件）。どちらも本文は変えず、verify の置き場として write-set に載せる。

(g) 便 44 までの形との接続: 新規 = src/schema.rs・tests/schema.rs の 2 本。既存 = adr.rs（FLOOR に注 22 本 + 機械を寄せる）・main.rs（mod の宣言 1 本と副命令 Schema と分岐・約 +40 行）。正本 design-intent/adr/schema.yaml は write-set の外（planner が書いた byte が anchor・便は変えない）。fixture の写し 16 本・floor_cases.yaml・note.rs・design-note/schema.yaml・CI の yml・.vessel.toml は触らない（CI の門は歯 1 が担う）。size M = 既存 file 1 本あたりの増分の見積は adr.rs が最大で約 +170 行（注 22 本の字面・葉 59）− 88 行（寄せる機械）。外部 crate は増やさない（sha256 は歯の中で sha256sum を撃つ・便 38 と同じ）。

## 2. 範囲

- 入れる: 命令 folio schema（--write / --check）・schema.rs（床の機械の共有 + 導出）・adr.rs の FLOOR に説明の注 22 本・歯 7 群。
- 入れない: 設計ノートの側（便 46）・folio check の検査と文言の変更・正本と fixture の変更・CI の yml の変更。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| cli | 口 | main.rs の副命令 Schema（--write / --check 排他） |
| emit | 導出 | schema.rs（体裁の規則 5 つ・印・区間・run） |
| floor | 床の機械 | schema.rs へ寄せた Floor / keys_floor / strip_notes / floor_diff |
| notes | 説明の注 | adr.rs の FLOOR に 22 本（file の順と字面のまま） |
| anchor | 凍結 | main の adr/schema.yaml の生成区間（22,182 byte・sha256 8c53aa96…） |
| teeth | 歯 | tests/schema.rs 7 群・adr 8 本と floor_cases 1 本は期待不変 |

## 4. 検査（歯）

§1 (f) のとおり。共通の検証は .vessel.toml の common-verify。

## 5. 依存

外部 crate は増やさない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "at"
title = "欄の決まりの file の生成区間を床の定数から導出する folio schema（--write / --check・印 2 本・体裁の規則 5 つ・終了 0 / 1 / 2）と、判断の記録の欄の決まりの説明の注 22 本を adr.rs の FLOOR へ・床の機械を schema.rs に寄せる（folio check の検査と文言は不変）"
req = ["FR19", "FR6", "FR5"]
section = "1"
write-set = ["+crates/folio/src/schema.rs", "+crates/folio/tests/schema.rs", "crates/folio/src/adr.rs", "crates/folio/src/main.rs", "crates/folio/tests/adr.rs", "crates/folio/tests/floor_cases.rs"]
verify = ["cargo nextest run -p folio --test schema schema", "cargo nextest run -p folio --test adr adr", "cargo nextest run -p folio --test floor_cases floor", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "M"
done = "schema の歯（実の正本の一致と要約値・ずれ・印 3 通り・書き直しと冪等・旗 2 通り・file なし・床は印を見ない）が緑、便 5 以降の歯 adr 8 本と floor_cases の 134 の場合が期待不変で緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
