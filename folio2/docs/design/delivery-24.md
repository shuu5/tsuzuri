# 設計: 便 24 — 散文の門（設計ノートの散文のうち規範の印を持つ文は参照 id を持ち、数と単位を持たない・rules 行 R-16）

- 要件: FR12（規範の印を持つ文が同じ文の中に参照 id〔条 id・rules 行 id・要件 id・契約 id〕を持ち数と単位を持たないことを床で数える・記述の文は数えない）/ FR5（3 値）
- 条: N-2.1・N-2.2（散文にしか無い規則は規則でなく、検査がそれを落とす）/ P-5.1・P-5.2（印と単位の一覧は型付きデータ = rules 行 R-16 に置き、文書からは id で参照する）/ P-10.1（独立した凍結 anchor）/ P-4.2
- 判断の記録: ADR-3（決定 (7) 散文の門は 1 形・器 scribe2 が M1 まで同じ式を暫定で持ち、M1 が引き取ったら撤去する）。欄の決まり design-note/schema.yaml の section.by_type.prose.prose_gate_rules_row = R-16。便 23（f2-648.37）の後に直列で置く。
- 裁定: 持ち主 2026-09-18「進んで良い」（M1 の着手）。R-16 の値は ADR-3 の発効承認（2026-09-16「いずれも承認する」）で発効済み・式の出所は持ち主の裁定 2026-09-13（s2-07l.197 notes・G-e = (b)）。
- 置き場: この文書は folio2 の設計ノート（M1 で YAML 正本へ移す）。契約表は末尾の区間。審査の材料は行 y が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は `+`）。

## 1. 目的と中身

便 23 で床に入った設計ノートの正本の、型 prose の節の body（散文）に散文の門を掛ける。数えるのは「規範の印を持つ文」だけで、記述の文は自由（FR12）。印を持つ文は (1) 同じ文の中に参照 id を 1 つ以上持ち、(2) 参照 id の外に「数字列 + 単位」を持たない。どちらかを欠けば種別「prose-gate」の違反。印・「禁止」の直後の文字・単位の一覧は rules 行 R-16 の value（型付きデータ）から検査のたびに読み、生成器にも散文にも写さない（P-5.1・R-16 の note「値の一覧は M1 の実装で型付きデータから読む」）。式は器 scribe2 の暫定の床（scribe2 の xtask の散文の門の module・印 5・直後の文字 5・単位 13・文の区切り・pointer の外の数 + 単位）と同じにする（ADR-3 決定 (7)）。

planner の実測（2026-09-18・main f5ea833・scribe2 main 12e64cc）: R-16 の value = marks 5（しなければならない・してはならない・してはいけない・SHALL・MUST）・prohibition = {word: 禁止, clause_ends: [。・）・・・、・null]}・units 13（秒・分・時間・日・件・本・行・byte・KB・MB・%・s・ms）。population（R-16 の欄）= 設計ノートの散文（code fence の内側・行頭が縦線の表の行・注釈の行を除く）・文の区切りは「。」か行末。note = 印は部分一致で英語は否定形も含む（SHALL NOT は SHALL を含む）・副詞「必ず」は数えない・「禁止」は直後の文字が clause_ends のどれか（null = 行末）のときだけ印・単位は数字列の直後。scribe2 の暫定の床の式（同じ値の宣言順・文の区切り・pointer の byte 範囲を左から重ならず取り、その外で「数字列 + 空白 1 つまで + 単位」を探す・英字の単位は直後が英字でないときだけ・理由は no-pointer が先で number-with-unit が後・1 文に 2 つ当たれば先の理由で 1 件）を読んで確かめた。設計ノートの見本 example.yaml の節 1 の散文は印を持つ文が 1 つ（「書き出してはならない（FR15）」）で、参照 id を持ち数と単位を持たない = 違反 0。verify の filter 語 prose を名に含む `#[test]` の関数は base の tests/note.rs に 2 本（便 23 の note の歯）あるが、verify は `--test prose` の scope なので歯の置き場も期待も本便の外（tests/note.rs は触らない）。

(a) 母集団: 設計ノート 1 本の sections のうち type = prose の節の body（複数行の文字列）。行のうち code fence（行頭が 3 つの逆引用符・開きから閉じまで）の内側・行頭が縦線の行・行頭が `<!--` の行は除く。残りの行を「。」で文に切り、行末でも切る（行を跨ぐ文は行末で切れる）。前後の空白を落として空の文は捨てる。他の型の節の欄（role・why・red_when・done 等）と note・title は数えない（R-16 の population = 散文）。

(b) 印の判定（文ごと）: marks のどれかを部分一致で含む、または word（禁止）の直後の文字が clause_ends のどれか（null は「禁止」が文の末尾）である文 = 印を持つ文。それ以外は記述の文（数えない）。

(c) 参照 id（pointer）の形（FR12 の 4 種・folio2 の id 空間）: 条 id `P-<数>`・`A-<数>`・`N-<数>`（枝番 `.<数>` は任意）／rules 行 id `R-<数>`・`D-<数>`／要件 id `FR<数>`・`NFR<数>`・`AC<数>`・`CON<数>`・`GOAL<数>`／契約 id `<doc id>#<row id>`（doc id と row id の形は欄の決まりの id_pattern と row_id.pattern）。英字で始まる形は直前が ASCII の英数字でない位置でだけ読む（識別子の途中を pointer にしない）。左から重ならずに取る。実在の解決は本便では行わない（形だけ・実在は便 23 の参照の解決の母集団に足す = 印を持つ文の pointer を既知の id に解き、無ければ種別「note」の違反）。

(d) 数と単位: pointer の byte 範囲の外で、数字列（ASCII の数字 1 つ以上）の直後に空白 1 つまでを挟んで units のどれかが在る = 数と単位を持つ。英字の単位（byte・KB・MB・s・ms）は直後が ASCII の英字でないときだけ単位と読む（`4 skills` を `4 s` にしない）。「%」は空白無しでも可（他と同じ規則で扱う）。

(e) 違反（種別「prose-gate」・1 文につき最大 1 件・理由は no-pointer が先）: 文言「design-note/<file>: 節 <n> 行 <k>: <理由の名> <文の先頭 60 字>」（理由の名 = no-pointer／number-with-unit・k = body の中の 1 始まりの行番号）。R-16 の value が読めない・欄が無い・型が違う = 「まだ分からない」（文言「rules.yaml: R-16 の value が読めない: <理由>」・合格にしない）。

(f) 実装 `crates/folio/src/prose.rs`（新規・pure な判定 = 本文と R-16 の値から違反の一覧を返す関数 + rules 行 R-16 の読み手）。便 23 の module（note.rs）から prose の節ごとに呼ぶ（呼び出し 1 か所を足す）。正規表現は使わない。外部 crate は足さない。

(g) 歯 `crates/folio/tests/prose.rs`（binary 経由・関数名はすべて prose を含める＝verify の filter 語）。歯の置き場のため既存の歯 `crates/folio/tests/check.rs` を write-set に載せるが本文も期待も変えない。入力は便 23 の歯と同じ作り（`design-intent/` と `contracts/schema.toml` を一時 dir へ写し git の 1 commit にしてから設計ノートを置いて `folio check --dir` を回す）。
- AC10（凍結 anchor `tests/fixtures/prose-gate/`・手書きの設計ノート 2 本）: `violations.yaml`（prose の節に 印を持つ文 2 つ = 参照 id の無い文・`30 秒` を持つ文〔参照 id 付き〕+ 記述の文 2 つ）を置く = 1 ∧ 違反 2 ∧ 出力に no-pointer と number-with-unit ∧ 記述の文は出ない／`clean.yaml`（記述の文だけ + 参照 id を持ち数と単位の無い印付きの文 1 つ + 表の行と code fence の中の印付きの文 + 契約表の節 1 つ〔行 id a・欄は器の導出 file の必須欄を最小で〕）を置く = 0。
- 式の枝（`clean.yaml` の写しに変異 1 つずつ・各 1 で違反 1、または 0）: 「禁止。」= 1／「禁止事項の一覧」（直後が事項）= 0／「SHALL NOT …（FR12）」= 0（pointer あり）・pointer を消す = 1／「必ず …。」（副詞だけ）= 0／`4 skills` を含む印付きの文 = 0・`4 s` = 1／`100%` = 1／行を跨ぐ印付きの文で pointer が次の行 = 1／pointer が識別子の途中（`xFR12y`）= 1／契約 id `clean#a` = 0（clean.yaml が持つ契約表の節の行 id a・便 23 の note.rs は同じ文書の契約表の行 id を `<doc id>#<row id>` と裸の形で既知の id に足す＝doc id は file の stem なので clean に解ける。他の文書の契約 id は解けず種別 note の違反になる＝本便の枝には置かない）。
- 「まだ分からない」: 写しの rules.yaml の R-16 の value を文字列にする = 2 ∧「R-16 の value が読めない」。
- 便 23 の歯と便 0〜22 の歯は期待不変（example.yaml は違反 0）。unit は置かない。

(h) 便 23 までの形との接続: 新規は `crates/folio/src/prose.rs`・歯 `crates/folio/tests/prose.rs`・fixture 2 本（新規 dir `tests/fixtures/prose-gate/` = 要件書 AC10 の red_test が名指す path）。便 23 の module（note.rs）は prose の節で門を呼ぶ 1 か所と、印を持つ文の pointer を参照の解決の母集団に足す 1 か所。`crates/folio/src/main.rs` は `mod prose;` の 1 行。他の src・`design-intent/`・`contracts/`・`.github/workflows/` は触らない。外部 crate は増えない。size は S = 中身を変える既存の file 1 本あたりの増分（note.rs 20 行未満・main.rs 1 行）。

## 2. 範囲

- 入れる: 散文の門（母集団・印・pointer の形・数と単位・違反の文言・R-16 の値を rules から読む）・凍結 anchor・歯。
- 入れない: R-16 の値の変更・他の型の節の欄への門・憲法や要件書の規範文への門（R-9 / R-12 の領分）・scribe2 側の暫定の床の撤去（M1 が引き取った後に s2 へ知らせる・別途）・導出物（FR11）。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| population | 母集団 | prose の body から fence・表・注釈を除き「。」と行末で文に切る |
| mark | 印 | marks の部分一致・禁止 + 直後の文字 |
| pointer | 参照 id | 4 種の形を左から重ならず取る |
| unit | 数と単位 | pointer の外の数字列 + 単位（英字は語の境界） |
| rule | R-16 の読み | rules.yaml の value（型付き）を検査のたびに読む |

## 4. 検査（歯）

§1 (g) のとおり。共通の検証は `.vessel.toml` の common-verify（便 0〜便 23 の歯は期待不変で全部回る）。

## 5. 依存

外部 crate は増やさない（clap / yaml-rust2 のまま）。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "y"
title = "散文の門 — 設計ノートの prose の節で規範の印を持つ文は参照 id を持ち数と単位を持たない（rules 行 R-16）"
req = ["FR12", "FR5"]
section = "1"
write-set = ["+crates/folio/src/prose.rs", "+crates/folio/tests/prose.rs", "crates/folio/src/note.rs", "crates/folio/src/main.rs", "crates/folio/tests/check.rs", "+tests/fixtures/prose-gate/violations.yaml", "+tests/fixtures/prose-gate/clean.yaml"]
verify = ["cargo nextest run -p folio --test prose prose", "cargo nextest run -p folio --test check check", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "prose の歯（AC10 の凍結 anchor 2 本・式の枝 11・まだ分からない 1）が緑、便 0 の歯 check が期待不変で緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
