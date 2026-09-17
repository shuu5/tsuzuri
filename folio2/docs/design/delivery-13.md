# 設計: 便 13 — 部品目録を正本に型の一覧を導出し、`folio parts` で 3 面を部品目録と突き合わせる

- 要件: FR4（3 面を単一の生成器と単一の design token で出す）/ NFR2（見た目の材料は 1 か所から・部品目録に無い class は rules 行 R-3 の値 0）
- 条: P-2.3（design token は 1 か所）/ P-2.4（部品・図の型は閉じた一覧で持つ）/ P-5.1（型の一覧は型付きデータに置く）/ P-6.3・P-6.4（一方を正本とし他方は導出する・2 面を人が書いて検査で揃える設計を採らない）/ P-4.1・P-4.2（確かめられない検査を合格にしない）/ P-10.1・P-10.2（独立した凍結 anchor）
- 判断の記録: ADR-5（決定 (3) 部品目録の部品・図の型を閉じた一覧として型で持つ・帰結 2「正本は部品目録の側」）。便 12（f2-648.23）の後に直列で置く。
- 裁定: 持ち主 2026-09-17（f2-648 notes・組み立ての方法 = 手書き・外部の部品を足さない）と ADR-5 の発効の承認（f2-648.22 notes）。
- 置き場: この文書は folio2 の設計ノート（M1 で YAML 正本へ移す）。契約表は末尾の区間。審査の材料は行 n が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は `+`）。

## 1. 目的と中身

3 面（入口・憲法・要件書）の生成器を書く前に、生成器が使ってよい部品の一覧を型で閉じ、できた面を部品目録と突き合わせる検査（要件書の受入基準 AC2 の機構）を先に置く。部品目録の正本は今の置き場 `design-intent/preview/parts.json`（本便では中身も置き場も変えない）。Rust の型の一覧は人が書かず、組み立て時に部品目録から導出する（P-6.4）。本便は面を生成しない。検査の相手は、今 `design-intent/preview/` に在る見本 3 面（index.html・constitution.html・srs.html）と様式の定義 folio.css で、後続の便で生成器の出力に置き換わっても同じ検査が掛かる。

planner の実測（2026-09-17・main 96d4284）: 部品目録の components は 30 個（各部品は role と faces を必ず持ち、faces の値は index・constitution・srs のどれか）・figure_type_enum は 8 個・shelf_type_enum は 1 個・style_props_allowed は 4 個。見本 3 面の部品の名札（属性 data-component の値）は 3 面合わせて 30 種で、部品目録の 30 個と過不足なく一致し、面ごとの faces にも全部合う（違反 0）。見本 3 面の class は 3 面合わせて 193 種で、全部が folio.css の class の定義 212 種の中に在る（目録外 0）。見本 3 面の行内の様式（属性 style）は 12 か所で、性質の名は全部 style_props_allowed の 4 個の中に在る（違反 0）。見本 3 面に引用符なしの class・style・data-component は 0 で、3 つの属性の値に「&」を含むものも 0。二重引用符で囲んだ値の中に一重引用符を含む属性（link 要素の href）が各面に 1 つ在る＝属性の値は開いた引用符と同じ種類の引用符でだけ閉じる。属性の値の中に「>」を含むタグは 0。folio.css に url の取り込みは 0。外部 crate の yaml-rust2（既に使っている版）は部品目録の JSON をそのまま読める（小さな試しで components 30・figure_type_enum 8 を読めた）。admin 席の独立した実測（2026-09-17・main 96d4284）でも上の数は全部同じ値だった（components 30・図の型 8・棚の型 1・許す性質 4・class 193 種が 212 種の中・行内の様式 12 か所で違反 0・引用符なし 0・「&」0・部品の名札 30 種で目録外 0 と faces の違い 0）。

(a) 導出: 新しい組み立ての script `crates/folio/build.rs` が、`CARGO_MANIFEST_DIR` から見た `../../design-intent/preview/parts.json` を yaml-rust2 で読み、Rust の source 1 本を `OUT_DIR` に書く。`crates/folio/src/parts.rs` がそれを取り込む。導出するもの: 部品の閉じた一覧（enum・30 個・並びは部品目録の順）・図の型の閉じた一覧（8 個）・棚の型の閉じた一覧（1 個）・行内の様式に許す性質の名の一覧（4 個）。名の付け方は機械的: 部品目録の名を「-」で割り、各片の先頭を大文字にして繋ぐ（pipeline-rail は PipelineRail）。各一覧は 全部を並べた定数・目録の名を返す関数・名から引く関数 を持ち、部品はさらに faces を返す関数を持つ。導出できない（部品目録が無い・読めない・components が表でない・部品が faces の一覧を持たない・名が ASCII の英小文字と数字と「-」以外を含む・先頭が数字・2 つの名が同じ型の名に潰れる）ときは組み立てを失敗させる（黙って空の一覧にしない）。build の script は部品目録の path を再実行の条件に宣言する。`crates/folio/Cargo.toml` には build の依存として yaml-rust2 を今と同じ版の指定で足す（新しい外部 crate は増えない）。`Cargo.lock` は変わらない（admin 席の実測: main の写しに build の依存の節と yaml-rust2 を使う最小の build の script を置いて offline で組み立て → 成功・`Cargo.lock` は byte 不変）。write-set に載せてあるのは、万一 cargo が書き換えたときにその差分を含められるようにするためで、新しい package の行が増えたら止めて問う。

(b) 副命令 `folio parts`（`crates/folio/src/main.rs` に枝を足し、本体は `crates/folio/src/parts.rs`）。旗は `--check` と `--print` のどちらか 1 つを必ず指定する（便 2・便 11 と同じ ArgGroup の形）。
- `--print`: 導出した一覧を JSON の 1 行で標準出力に出して終了 0。形は「{」components（部品の名 → faces の一覧の表・部品目録の順）・figure_types・shelf_types・style_props の 4 つのキーをこの順に「}」。文字列の直列化は `yaml.rs` の既存の関数 json_str（crate の中から呼べる）を使う。区切りは「,」と「:」だけで空白を入れず、末尾に改行 1 つ。
- `--check`: 選べる引数は `--dir <正本の置き場・既定 design-intent>`・`--css <様式の定義・既定は dir の下の preview/folio.css>`・`--page <面の名>=<path>`（何度でも・1 つも無ければ既定の 3 面 = dir の下の preview/ の index.html・constitution.html・srs.html をそれぞれ面 index・constitution・srs として読む）。面の名が index・constitution・srs のどれでもなければ clap の使い方の誤りではなく「まだ分からない」（終了 2）。
- 報告と終了コードは `folio check` と同じ型（`verdict.rs` の Report と Verdict）で、違反は「[種別] 文言」を標準出力へ、まだ分からないは「# まだ分からない: 文言」を標準エラーへ、最後に「folio parts: 判定（違反 N・まだ分からない M）」を標準出力へ出す。
- 数えるもの:
  1. 部品目録と組み立て時の写しの一致: 実行時に dir の下の preview/parts.json を `yaml.rs` の読み手で読み、components の名と faces・figure_type_enum・shelf_type_enum・style_props_allowed が、組み立て時に導出した一覧と過不足なく同じ順で一致すること。違えば「まだ分からない」（文言「parts.json: 組み立て時の部品目録と違う（組み立て直す）」）で、以下の 2〜4 は数えない（P-10.3 と同じ考え方）。部品目録が無い・読めない・重複キーが在る も「まだ分からない」。
  2. class（種別「R-3」・文言「面の file 名: 部品目録に無い class「名」」）: 面の全部の開始タグの属性 class の値を空白で割った各語が、様式の定義の class の集合に在ること。1 つの面で同じ class が何度出ても違反は 1 件。
  3. 部品の名札（種別「parts」）: 面の全部の属性 data-component の値が部品の一覧に在ること（文言「面の file 名: 部品目録に無い部品「名」」）と、その部品の faces がその面の名を含むこと（文言「面の file 名: 部品「名」はこの面に置けない」）。
  4. 行内の様式（種別「parts」・文言「面の file 名: 許されない行内の様式「性質の名」」）: 面の全部の属性 style の値を「;」で割り、各片の最初の「:」より前を前後の空白を除いて性質の名とし、それが許す一覧に在ること。「:」の無い空でない片は違反。
- 面の読み方（外部 crate と正規表現を使わない手書きの走査）: 注釈（「<!--」から「-->」）と、script 要素・style 要素の中身は読み飛ばす。開始タグ（「<」の直後が英字）ごとに属性を 名 = 値 の形で読む。値は二重引用符か一重引用符で囲んだ形を受ける。class・style・data-component の 3 つの属性の値が引用符で囲まれていなければ、その面は「まだ分からない」。閉じない注釈・閉じないタグ・閉じない引用符も「まだ分からない」。文字参照（「&amp;」の形）は解かない（3 つの属性の値に「&」が在れば「まだ分からない」）。面の file が無い・読めない・UTF-8 でない も「まだ分からない」。
- 様式の定義の読み方: 注釈（「/*」から「*/」）と、二重引用符か一重引用符の文字列の中を読み飛ばし、残りの中で「.」の直後が英字か「_」か（「-」とその次が英字か「_」か「-」）で始まり、英数字・「_」・「-」が続く限りを 1 つの class の名とする。「.」の直後が数字のもの（小数の値）は class ではない。様式の定義が無い・読めない・閉じない注釈や文字列 は「まだ分からない」。この読み方は値の中の「.」で始まる語も class に数える側に倒れる（集合が広がる向き）ので、見落としの向きは「目録外を見逃す」側である＝実測では folio.css に url の取り込みも、文字列の外の「.」で始まる値の語も無く、集合は 212 種で planner の別の道具の数えと一致した。

(c) 歯 `crates/folio/tests/parts.rs`（binary 経由・歯の関数名はすべて parts を含める＝verify の filter 語・base の歯に parts を含む名は 0 本）。
- 見本 3 面で合格: repo の `design-intent/` をそのまま `--dir` に渡して `--check` = 終了 0 ∧ 標準出力に「違反 0」。
- 目録外の class で不合格（AC2 の RED の文）: `design-intent/preview/` の index.html を一時 dir へ写し、どれかの開始タグの class に folio.css に無い語を 1 つ足して `--page index=その写し` で `--check` = 終了 1 ∧「部品目録に無い class」∧ 違反 1。
- 目録外の部品・面に置けない部品・許されない行内の様式: 同じ作りで 1 つずつ変異を当て、どれも終了 1 ∧ それぞれの文言 ∧ 違反 1。面に置けない部品の変異は、入口の面の写しに憲法と要件書だけに置ける部品 doc-cover-band の名札を足す。
- 読めない入力 = 終了 2: 面の path が無い／class の値の引用符を外す／閉じない注釈を足す／`--css` に無い path／面の名が 3 つのどれでもない／部品目録を写して components から 1 つ消した dir を `--dir` に渡す（文言「組み立て時の部品目録と違う」）。
- 独立した凍結 anchor（P-10.1・生成器からも見本からも独立した最小の手書き）: 新しい置き場 `tests/fixtures/floor/` に 4 本。mini.css（class の定義を 3 つ・注釈の中と文字列の中に「.」で始まる語を 1 つずつ・小数の値を 1 つ）・mini-ok.html（面 index として読む・class は mini.css の 3 つだけ・部品の名札は入口の面に置ける hub-cover を 1 つ・行内の様式は許す性質を 1 つ・注釈の中と script 要素の中に目録外の class を持つタグの字面を 1 つずつ）・mini-extra.html（mini-ok.html に目録外の class を 1 つ足しただけ）・parts-catalog.json（`--print` の期待の 1 行・手書き）。歯: `--css mini.css --page index=mini-ok.html` = 終了 0／`--page index=mini-extra.html` = 終了 1 ∧ 違反 1／`--print` の出力が parts-catalog.json と byte 一致。parts-catalog.json の中身は、部品目録の今の 30 個の部品の名と faces・図の型 8 個・棚の型 1 個・許す性質 4 個を部品目録の順に写したもの（部品目録を判断の記録つきで変えるときに、この期待も同じ取り込みで直す）。
- unit の歯（`src/parts.rs` の中・名に parts を含む）: 様式の定義の読み方（注釈・文字列・小数・「-」始まり）と、面の読み方（注釈・script 要素・一重引用符・引用符なしは読めない）と、行内の様式の割り方。

(d) 便 12 までの形との接続: 新規は `crates/folio/build.rs`・`crates/folio/src/parts.rs`・歯 `crates/folio/tests/parts.rs`・fixture 4 本。`crates/folio/src/main.rs` で変えるのは `mod parts;` の 1 行と、副命令 parts の枝（引数の型と、報告を出して終了コードを返す部分）だけで、他の副命令の枝は 1 字も変えない。`crates/folio/Cargo.toml` で変えるのは build の依存の節を足すことだけ。使う既存の口は `yaml.rs` の読み手と json_str・`verdict.rs` の Report と Verdict で、どれも既に crate の中から呼べるので、`yaml.rs` と `verdict.rs` の中身は変えない。`check.rs`・`entrance.rs`・`render.rs`・他の src は中身を変えない（`folio check` に部品目録の検査を組み込まない＝凍結 fixture の 134 case と既存の fixture 17 組は preview/ を持たないので、組み込むと期待が化ける）。既存の歯 `crates/folio/tests/check.rs` は、受付の門が歯の置き場を解くために write-set に在るが 1 字も変えない。本便は既存の歯の fixture を触らない（新しい fixture を新しい置き場に足すだけ）ので、既存の歯の回帰は共通の検証（workspace の全歯）が捕まえる。`design-intent/` の下（部品目録・見本 3 面・folio.css を含む）・`scripts/`・`.github/workflows/` は触らない。正規表現は使わない。契約表の size は S = 中身を変える既存の file（main.rs・Cargo.toml）1 本あたりの増分の見積で、新規の file は増分に数えない。

## 2. 範囲

- 入れる: 部品目録からの型の一覧の導出（組み立て時）・副命令 `folio parts`（check / print）・3 面の class と部品の名札と行内の様式の検査・歯・独立した最小の凍結 fixture。
- 入れない: 3 面の生成（HTML）・部品目録の中身や置き場の変更・密度 profile の一覧（部品目録に値域がまだ無い）・色や字の大きさが design token 由来である割合の検査（rules 行 R-3 の 2 つ目の値・様式の定義の中身の検査として別の便）・折り返し禁止の許可の一覧と読み手の区別（audience）の検査・図の道具（archify）の図の本体の class の検査（M1）・`folio check` への組み込み・CI の段の変更。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| derive | 導出 | 組み立て時に部品目録から閉じた一覧（部品・図の型・棚の型・許す性質）を作る |
| scan-html | 面の読み手 | 開始タグの属性 class・data-component・style を手書きの走査で読む |
| scan-css | 様式の定義の読み手 | class の定義の集合を作る |
| verdict | 判定 | 4 つの検査を 3 値で出す・print は導出した一覧を 1 行で出す |

## 4. 検査（歯）

§1 (c) のとおり。共通の検証は `.vessel.toml` の common-verify（便 0〜便 12 の歯は期待不変で全部回る）。

## 5. 依存

外部 crate は増やさない（clap / yaml-rust2 のまま）。yaml-rust2 を build の依存にも宣言する（同じ版・新しい package は増えない）。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "n"
title = "部品目録を正本に型の一覧を導出し、folio parts で 3 面を部品目録と突き合わせる"
req = ["FR4", "NFR2"]
section = "1"
write-set = ["+crates/folio/build.rs", "+crates/folio/src/parts.rs", "+crates/folio/tests/parts.rs", "crates/folio/src/main.rs", "crates/folio/tests/check.rs", "crates/folio/Cargo.toml", "Cargo.lock", "+tests/fixtures/floor/mini.css", "+tests/fixtures/floor/mini-ok.html", "+tests/fixtures/floor/mini-extra.html", "+tests/fixtures/floor/parts-catalog.json"]
verify = ["cargo nextest run -p folio parts", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "parts の歯（見本 3 面で合格・目録外の class で不合格・目録外の部品と面に置けない部品と許されない行内の様式で不合格・読めない入力 6 つで終了 2・独立した凍結 fixture の合格と不合格と print の一致・unit）が緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
