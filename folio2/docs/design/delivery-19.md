# 設計: 便 19 — 命令 `folio intake`（質問を散文で出し、回答から支度表 1 枚を書く・差分だけ問う）

- 要件: FR1（5 問以下の易しい質問を推奨回答つきで提示し、回答から文書の集合を写像して支度表 1 枚を生成する）/ FR2（回答が無ければ推奨回答で進み、推奨で進めた項目を別枠に出す）/ FR8（途中で足すときは差分だけを問い支度表を更新する）
- 条: P-1.1・P-1.2（生成と検査と知らせるところまで・採否の判断を代行しない）/ P-6.1・P-6.2（支度表の正本は YAML 1 file・生成物を手で直さない）/ P-4.1・P-4.2（黙って飛ばさない）/ P-10.1（独立した凍結 anchor）/ N-1.1（管理下の対象を回復不能に消さない）
- 判断の記録: ADR-6（決定 (2) 質問 5 つと行き先／決定 (3) 支度表は YAML 1 枚・欄は intake.yaml の sheet の節／決定 (4) 質問は対話面の散文で出し回答は旗（answers）の file で受ける・無ければ全項目を推奨回答で進め別枠に・答え済みの項目は問わない／決定 (5) 順序 = 床の便 → 本便）。便 18（f2-648.32）の後に直列で置く。
- 裁定: 持ち主 2026-09-18「承認するし質問も全て推奨で承認する」（3 つの問い）と ADR-6 の発効の承認「承認する」（f2-648.31 notes・main f54efaa）。rules 行 D-10（選択式の窓を使わない）。
- 置き場: この文書は folio2 の設計ノート（M1 で YAML 正本へ移す）。契約表は末尾の区間。審査の材料は行 t が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は `+`）。

## 1. 目的と中身

相談窓口の命令 `folio intake` を置く。命令は相談窓口の正本 `design-intent/intake.yaml`（v0.1・発効済み・便 18 で床に入る・本便では読むだけ）から質問 5 つと推奨回答を対話面の散文として標準出力へ書き（rules 行 D-10・選択式の窓を使わない）、回答を旗（answers）で渡す YAML の file から読み、回答から文書の集合を写像して支度表 1 枚（YAML・intake.yaml の sheet の節が欄を決める）を設計文書の置き場へ書く。回答の無い質問は推奨回答で進め、その全部を支度表の別枠（recommended）に出す（FR2）。既に支度表が在れば、その回答を答え済みとして引き継ぎ、問うのは答えの無い質問だけ（FR8）。採否の判断は道具の外（P-1.2）＝命令は「承認」を求めず、支度表の承認欄は空のまま書く（承認の記帳は持ち主の対話面と台帳・P-12.2）。

planner の実測（2026-09-18・main f54efaa）: intake.yaml は meta（id folio2-intake・version v0.1・status effective）・answers（values = はい・いいえ・default = recommend）・targets 5 行（constitution〔with = vocabulary・rules〕・srs・adr・design-note・inject）・questions 5 行（q1〜q5・ask・recommend = 全部 はい・why・yes = 順に constitution・srs・adr・design-note・inject・no = 全部空）・sheet（file = intake-sheet.yaml・title・explain・sections 5 行 = meta・documents・recommended・answers・approval）。正本の読み手は `face.rs` の load（型付きの読み・重複キーと空の文書は Err）と欄の読み X（f・ef）。YAML の書き手は `yaml.rs` の write（値の木と先頭の注釈から決定的な字面を書く・便 9 の freeze が使う）。入口の正本 index.yaml の intake の command は「folio intake」（本便の命令名と一致）。verify の filter 語 sheet を名に含む `#[test]` の関数は base に 0 本。

(a) 命令の形: `folio intake --dir <正本の置き場・既定 design-intent> [--answers <回答の file・相対なら --dir からの相対・絶対ならそのまま>] --print|--write`。`--print` と `--write` はどちらか 1 つを必ず指定する（便 14 と同じ ArgGroup の形）。読むのは intake.yaml と、在れば支度表 `<dir>/<sheet.file>`（sheet の file の値）と、旗が在れば回答の file。
- 回答の file の形: 最上位の欄 answers の表（質問の id → 値）。例:

```
answers: {q1: はい, q2: いいえ}
```

- 回答の解き方（決定的・順序は questions の順）: 質問ごとに 値 = 回答の file の値 → 無ければ 既存の支度表の answers のうち source が answered の行の値 → 無ければ recommend（推奨回答）。既存の支度表の source が recommended の行は値を持たないものとして扱う（＝また問う・FR8）。出所（source）= answered（file か既存の支度表の answered の行から）／recommended（推奨で進めた）。
- 写像: 質問ごとに 値が はい なら yes の一覧・いいえ なら no の一覧 の行き先を、questions の順・一覧の順に集め、同じ id は最初の 1 回だけ持つ（from = その質問の id）。行き先の type と with は targets の行から写す。
- `--print`: 答えの無い質問（回答の file も既存の支度表の source answered の行も値を持たない質問）だけを、質問 1 つにつき 1 行「<id>. <ask>（おすすめ: <recommend>）— <why>」で標準出力へ書く。先頭に 1 行「folio intake: 質問 <数> つ（答えは旗 answers の YAML の file で渡す・答えなくてもおすすめで進む）」。答えの無い質問が 0 なら 1 行「folio intake: 問う質問は無い（支度表 <file> が全部答え済み）」。終了 0。file は書かない。
- `--write`: 上の解き方で支度表を組み、`<dir>/<sheet.file>` へ書く（在れば上書き＝生成物・P-6.2）。終了 0・標準出力「folio intake: 支度表を書いた（持つ文書 <数>・推奨で進めた項目 <数>・<path>）」。支度表の形（yaml.rs の write の字面・先頭の注釈は固定の 1 行「# folio2 支度表（folio intake の生成物・手で直さない・承認は対話面と台帳で）」・数と日付は書かない）:

```
meta: {id: folio2-intake-sheet, version: <intake.yaml の meta.version>, status: draft}
documents:
  - {id: constitution, type: 憲法, with: [vocabulary, rules], from: q1}
recommended:
  - {q: q2, ask: <ask>, recommend: はい}
answers:
  - {q: q1, value: はい, source: answered}
approval: []
```

  documents は写像の結果（0 行なら空の一覧）。recommended は出所が recommended の質問（0 行なら空の一覧）。answers は全質問（questions の順）。approval は空の一覧（承認の記帳は持ち主の対話面と台帳の後に planner が書く・本便の外）。meta の status は draft 固定（承認の後に変えるのは人・本便の外）。
- 導出できない = 2・標準エラー「folio intake: まだ分からない: <理由>」・`--write` でも 1 byte も書かない: intake.yaml が読めない・欄が無い・型が違う（meta の version／answers の values と default／targets の各行の id・type・with／questions の各行の id・ask・recommend・why・yes・no／sheet の file）／questions の id の重複／recommend が values に無い／yes か no の行き先が targets に無い／回答の file が読めない・最上位が表でない・answers が表でない・質問の id が questions に無い・値が values に無い／既存の支度表が在るのに読めない・answers が一覧でない・行の q が questions に無い・value が values に無い／`--write` の出力先の親 dir が無い。
- 3 値: 本便の命令に不合格（1）は無い（判定ではなく生成）。

(b) 実装 `crates/folio/src/sheet.rs`（新規・命令の口と解き方と書き手）。`crates/folio/src/main.rs` に `mod sheet;` と Command の variant（Intake・dir・answers・print・write）とその分岐。正規表現は使わない。外部 crate は足さない。`face.rs` の load と X はそのまま使う（変えない）。

(c) 歯 `crates/folio/tests/sheet.rs`（binary 経由・関数名はすべて sheet を含める＝verify の filter 語・verify の 1 行目は旗無しの filter で integration と src の unit〔module の道 sheet::tests〕の両方を回す・folio は bin crate で lib の的が無い）。歯の置き場のため既存の歯 `crates/folio/tests/vocab.rs` を write-set に載せるが本文も期待も変えない（便 17 と同じ形・filter 語 vocab は `tests/vocab.rs` の 2 本に解ける）。入力は `design-intent/` を丸ごと一時 dir へ写す（写しの intake-sheet.yaml は歯の中でだけ生まれる・版管理の design-intent には書かない）。
- AC1（凍結 anchor・P-10.1）: 固定の回答 5 つ `tests/fixtures/intake/answers-5.yaml`（q1〜q5 = はい・いいえ・はい・はい・いいえ）で `--write` = 0 ∧ 書いた支度表が `tests/fixtures/intake/expected-sheet.yaml` と byte 一致（documents = constitution・adr・design-note の 3 行・recommended 0 行・answers 5 行 source answered）。回答の file 無しで `--write` = 0 ∧ `tests/fixtures/intake/expected-sheet-recommended.yaml` と byte 一致（documents 5 行・recommended 5 行・answers 5 行 source recommended）。期待の file は最初の 1 回は生成物を写して置いてよく、置いた後は歯が固定する。
- FR8（差分）: 回答 3 つ `tests/fixtures/intake/answers-3.yaml`（q1・q3・q4 = はい）で `--write` → `--print` の出力が q2 と q5 の 2 行 + 先頭の 1 行だけ ∧ q1 の ask を含まない → q2・q5 の回答（いいえ・いいえ）だけを持つ file で `--write` → 支度表が `expected-sheet.yaml` と byte 一致（引き継ぎ + 上書きで同じ結果）。
- `--print`（支度表なし）: 先頭の 1 行 + 5 行 ∧ 各行に ask と「おすすめ: はい」∧ file は書かれない。全部答え済みの後の `--print` = 「問う質問は無い」の 1 行。
- 導出できない 6 つ（どれも 2 で支度表が出来ていない・在った支度表は変わらない）: 回答の質問の id が無い（q9）／値が values に無い／回答の file の最上位が一覧／intake.yaml の questions の recommend を values に無い値に変異／yes の行き先を targets に無い id に変異／既存の支度表の answers の value を values に無い値に変異。
- 出力先の親 dir が無い `--dir` = 2。`--print` と `--write` の同時指定と両方無しは clap の使い方の誤り（終了 2）。
- unit（`src/sheet.rs` の中・名に sheet を含む）: 回答の解き方（file → 既存の answered の行 → 推奨 の順と source・既存の recommended の行は引き継がず値なしと扱う case を 1 つ）・写像の重複の畳み（同じ id は最初の 1 回・from は最初の質問）。

(d) 便 18 までの形との接続: 新規は `crates/folio/src/sheet.rs`・歯 `crates/folio/tests/sheet.rs`・fixture 4 本（新規 dir `tests/fixtures/intake/`・要件書 AC1 の red_test が名指す path）。`crates/folio/src/main.rs` は `mod sheet;` と variant と分岐だけ。`face.rs`・便 18 の床の module（intake.rs）・`yaml.rs`・他の src・`build.rs`・`Cargo.toml`・`Cargo.lock`・`scripts/`・`.github/workflows/`・`design-intent/`・`.gitignore` は触らない（生成した支度表を版管理に置くかは支度表の承認の便で決める）。size は S = 中身を変える既存の file 1 本あたりの増分の見積（`main.rs` の増分は variant と分岐で 60 行未満・新規と fixture は増分に数えない）。

## 2. 範囲

- 入れる: 命令 `folio intake`（散文の質問・回答の file・支度表の生成・差分だけ問う）・歯・AC1 の凍結 anchor。
- 入れない: 支度表の形の検査（床・別便）・入口の面の支度表の節（便 20）・気づかせる 1 行（FR3・便 21）・支度表の承認の記帳（人・対話面と台帳）・版管理への支度表の追加・質問の文の変更。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| ask | 質問の提示 | 答えの無い質問だけを散文で標準出力へ |
| resolve | 回答の解き方 | file → 既存の支度表 → 推奨回答 の順・出所を持つ |
| map | 写像 | はい／いいえ の行き先を targets から写し、重複を畳む |
| sheet | 支度表の書き手 | yaml.rs の write で決定的に書く・承認欄は空 |

## 4. 検査（歯）

§1 (c) のとおり。共通の検証は `.vessel.toml` の common-verify（便 0〜便 18 の歯は期待不変で全部回る）。

## 5. 依存

外部 crate は増やさない（clap / yaml-rust2 のまま）。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "t"
title = "命令 folio intake — 質問を散文で出し、回答から支度表 1 枚を書く（差分だけ問う）"
req = ["FR1", "FR2", "FR8"]
section = "1"
write-set = ["+crates/folio/src/sheet.rs", "crates/folio/src/main.rs", "+crates/folio/tests/sheet.rs", "crates/folio/tests/vocab.rs", "+tests/fixtures/intake/answers-5.yaml", "+tests/fixtures/intake/answers-3.yaml", "+tests/fixtures/intake/expected-sheet.yaml", "+tests/fixtures/intake/expected-sheet-recommended.yaml"]
verify = ["cargo nextest run -p folio sheet", "cargo nextest run -p folio --test vocab vocab", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "sheet の歯（AC1 の凍結 anchor 2 本と byte 一致・FR8 の差分と引き継ぎ・print の 3 形・導出できない 6 つ・親 dir・unit）が緑、vocab の歯が期待不変で緑（置き場として write-set に在るだけ）、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
