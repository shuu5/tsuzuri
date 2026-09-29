# 設計: 便 9 — `folio check` に `--emit-amends`（差分の印字）と `--freeze-anchor`（凍結）を足す

- 要件: FR5（3 値・実行できなかった検査を合格にしない）/ NFR3（正本の内部の整合）/ FR6 の隣（正本から生成物を決定的に導く・P-6）
- 条: A-2.1（条文の改訂は判断の記録と承認を伴う）/ N-4.1（記録を欠く改訂を拒む）/ P-6.1（正本 YAML から逐語で生成する）/ P-10.1（検査は独立した凍結 anchor を持つ）/ N-1.1（管理下の対象を回復不能に削除しない＝同じ版は上書きしない・anchor と索引は消さない）
- 判断の記録: ADR-2（凍結 anchor の生成は全検査 0 違反のときだけ・索引は追記のみ・--emit-amends は判断の記録を書く補助）。便 8（f2-648.18）の後に直列で置く。
- 裁定: 持ち主 2026-09-17「進めて良い」と前もっての確認（f2-648 notes）。
- 置き場: この文書は folio2 の設計ノート（M1 で YAML 正本へ移す）。契約表は末尾の区間。審査の材料は行 j が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は `+`）。fixture は増やさない（歯の中で一時 dir に写しを作る）。

## 1. 目的と中身

便 8 の `folio check` に、day-1 の床 `scripts/check_draft.py` の 2 つの旗を同じ式で足す。`--emit-amends` は最新 anchor と現行の欄単位の差分を判断の記録の amends にそのまま貼れる形で印字する読み取り専用の補助、`--freeze-anchor` は全検査が 0 違反で測れないも無いときだけ現行の写しを新しい版の anchor として書き索引に追記する。これで憲法の改訂の手順（版上げ → 判断の記録 → 承認 → 凍結）が folio だけで回る（ADR-1 の帰結「道具の移管」の終端）。この便では床の script を触らない（床の削除は別の便・A-1）。

(a) 旗。`folio check` に `--emit-amends` と `--freeze-anchor` を足す（両方同時は引数の断り・終了コード 2）。旗が無いときの振る舞いは便 8 のまま。

(b) `--emit-amends`。便 8 までの全検査を回した後、最新 anchor（索引の末尾・読めているもの）の content と便 7 の (e) の現行の写しを便 8 の (b) の欄単位の差分で比べ、標準出力へ次を書く: 最新 anchor が無いか読めなければ 1 行「# 比較元の anchor が無い（最初の版か、列が切れている）」。在れば 1 行目「# 比較元 anchor <最新の版> → 現行（版 <現行 meta.version>）の欄単位の差分。amends にそのまま貼る（version は現行 meta.version の値＝上げる版）」、続けて差分を 対象の字面の昇順・欄の道の字面の昇順 に「- 」+ json の表 1 つ（欄の順は target・field・version・previous_text・new_text の固定・version は現行 meta.version・値は便 8 の (b) の値の表現の字面）で 1 行ずつ。json の表の字面は day-1 の json.dumps の既定の形（二重引用符で囲んだキー + 「: 」+ 値・項目は「, 」で区切る・非 ASCII はそのまま・二重引用符と逆斜線は逆斜線を前置・制御文字は便 7 の (c) と同じ）。この旗のときは違反の行を標準出力でなく標準エラーへ書く（標準出力を貼れる形だけにする・day-1 と同じ）。終了コードは旗の無い check と同じ。day-1 の床の実測（admin・2026-09-17・main 2adc1f2・P-1 の title を「判断する道具を作らない（改訂）」に変え meta.version を v1.1 にした写し）: 終了コード 1・標準出力は 2 行ちょうど＝見出し「# 比較元 anchor v1.0 → 現行（版 v1.1）の欄単位の差分。amends にそのまま貼る（version は現行 meta.version の値＝上げる版）」と 「- 」+ 表 {target: P-1, field: title, version: v1.1, previous_text: 判断する道具を作らない, new_text: 判断する道具を作らない（改訂）}（キーと文字列の値は二重引用符で囲み、キーと値の間は「: 」・項目の間は「, 」・末尾に改行 1 つ）。folio の字面はこれに byte で一致させる。変異なしなら終了コード 0・標準出力は見出し 1 行だけ（「現行（版 v1.0）」）。

(c) `--freeze-anchor`。順に: (1) 最新 anchor が在り、その版が現行 meta.version より新しいか同じ（版の比べ方 = 「v」を除き「.」で分けた数の列の辞書順）なら、標準エラーに「版 <現行> は最新 anchor <最新> より新しくない（同じ版は上書きしない・版を上げてから）」を書き終了コード 1 で終える（他の検査より先・書かない）。(2) 最新 anchor が無いのに現行の版が床の first_version（v1.0）でなければ 1 違反（anchor）。最新 anchor が無いのに 版管理の HEAD か履歴に anchor が在った（便 8 の (a) の tracked / ever）か改訂の記録が在れば 1 違反（anchor・列の始め直しは認めない）。(3) 最新 anchor が在れば、便 8 の (b) の消し込みを 前 = 最新 anchor・今 = 現行の写し・版 = 現行 meta.version・場所の名 = 現行 で回す（差分の全件が現行の版を名指す発効した判断の amends と 1:1・変わった条の amended_by・条の消失・改番）。差分が 0 なら 1 違反（A-2・意味の無い版上げ）。現行の版を amends に持つ発効した判断が 0 本なら 1 違反（N-4）。(4) 凍結する木を作る: kind = constitution-anchor・digest_algo = 床の値・version = 現行 meta.version・previous = 最新の版（無ければ null）・projection = {scope: 憲法 schema.amendment_scope の列, article_fields: 床の 5 欄, statement_fields: 床の 4 欄}・meta_approval = 憲法 meta.approval の型付きの写し・approvals = 最新 anchor が在れば その版を名指す発効した各判断について {adr: 判断の id, who, date, ruling, verbatim, surface}（判断の id の字面の昇順）、無ければ [{adr: null, who, date, ruling, verbatim, surface}]（値は meta.approval から）・content = 現行の写し。digest = digest を除く全欄の便 7 の (c) の正規化の sha256。列の根（最新 anchor も無く previous が null）の凍結で digest が床の root_digest と違えば 1 違反（anchor・根は 1 度きり）。day-1 の床の実測（同上）: 変異なしで `--freeze-anchor` → 終了コード 1・標準エラー「版 v1.0 は最新 anchor v1.0 より新しくない（同じ版は上書きしない・版を上げてから）」・anchors/ は不変。合成した改訂（下の歯 (4) の形）で → 終了コード 0・標準エラー「凍結した: <path>/anchors/constitution-v1.1.yaml（条 27・previous v1.0・承認 1 件）・索引 index.yaml に追記」・索引の entries は 2 項で 2 項目 = {version: v1.1, previous: v1.0, digest: 3614e8fb で始まる値}・anchor の approvals = [{adr: ADR-5, who: 持ち主, date, ruling, verbatim, surface}]・meta_approval は憲法 meta.approval の写し（surface の長い字面もそのまま）。amends の new_text を 1 字違えると → 終了コード 1・「凍結しない — 違反 1 件・まだ分からない 0 件を直してから --freeze-anchor」・v1.1 の anchor は作られない。(5) 便 8 までの全検査と (2)〜(4) を合わせて、違反が 1 つでも在るか測れないが 1 つでも在れば、標準エラーに「凍結しない — 違反 N 件・まだ分からない M 件を直してから --freeze-anchor」を書き、何も書かず、終了コードは旗の無い check と同じ。どちらも無ければ `anchors/constitution-<版>.yaml` を書き（既に在れば書かずに 1 違反 anchor・N-1.1）、`anchors/index.yaml` を entries に {version, previous, digest} を 1 項足した全文で書き直し（他の項は変えない・追記のみ）、標準エラーに「凍結した: <path>（条 N・previous <版>・承認 N 件）・索引 index.yaml に追記」を書き、終了コード 0。

(d) yaml の書き手。凍結の木と索引は folio 自身の書き手で決定的に書く（外部 crate を足さない）。字面は day-1 の PyYAML の書き手と同じでなくてよいが、**PyYAML の既定の読み手と便 7 の型付きの読み手の両方で同じ型付きの木に読めること**を要件とする: 表は各欄を 1 行「キー: 値」（入れ子は 2 空白の字下げ）・一覧の各要素は「- 」（表の要素は 1 行目を「- キー: 値」で始め残りを揃える）・空の一覧は「[]」・空の表は「{}」・Null は null・Bool は true / false・Int は数字列・Float は便 7 の (c) の字面・Date はその字面のまま（PyYAML が日付に読む・便 7 の読み手も日付に読む）・Str は**必ず二重引用符で囲む**（中の二重引用符と逆斜線は逆斜線を前置・改行は \n・他の制御文字は \u + 4 桁・非 ASCII はそのまま＝PyYAML も便 7 の読み手も文字列に読み、真偽や数や日付の字面の文字列が型に化けない）。キーは文字列で、同じ規則で二重引用符で囲む。file の 1 行目は day-1 と同じ注釈（anchor: 「# folio2 憲法 <版> の凍結 anchor（ADR-2）。写し（範囲 <scope>・条は <5 欄>・規範文は id・<3 欄>）+ 発効の承認の写し + この版の承認一覧 + digest。手で直さない・消さない・同じ版は上書きしない（folio check --freeze-anchor が全検査 0 違反のときだけ作る）。」／索引: 「# folio2 凍結 anchor の索引（追記のみ・ADR-2）。列 = entries の順。手で直さない・消さない・空にしない。」）。書き手の unit の歯: main の anchor v1.0 を型付きで読み、書き手で書き、便 7 の読み手で読み直して型付きの木が等しく digest も等しいこと（往復）・二重引用符・逆斜線・改行・真偽の字面の文字列を含む木の往復。

(e) 判定は便 7 の 3 値。旗の無い check の振る舞いは変えない（既存の歯 16 組の期待は不変）。

`folio check` 自身の歯（`crates/folio/tests/freeze.rs`・binary 経由・fixture は増やさず歯の中で design-intent の写し全部 + git 1 commit を一時 dir に作る・便 0 の parity と同じ作り）: (1) 写しの P-1 の title を変え meta.version を v1.1 にし `--emit-amends` → 終了コード 1（未凍結の A-2）・標準出力は 1 行目の見出しと「- 」で始まる 1 行（target P-1・field title・version v1.1・previous_text 元の題・new_text 新しい題）の 2 行だけ・違反の行は標準エラー／(2) 変異なしで `--emit-amends` → 0・標準出力は見出し 1 行だけ／(3) 変異なしで `--freeze-anchor` → 1・標準エラーに「新しくない」・anchors/ は不変／(4) **合成した改訂**（欄集合は admin の床の実測で凍結が通った形そのもの）: 写しの P-1 の title を「判断する道具を作らない（改訂）」に変え meta.version を v1.1 にし、`adr/ADR-5.yaml`（id ADR-5・title・status accepted・date・context・decision・options 2 つで adopted 1 つ〔各 id・name・text・verdict・reason〕・basis [A-2]・retreat {kind: ruling, condition}・plain・approval {who: 持ち主, date, ruling: 台帳 id を含む字面, verbatim, surface: R-8}・grill {when, who, where, summary}・amends [{target: P-1, field: title, version: v1.1, previous_text: 判断する道具を作らない, new_text: 判断する道具を作らない（改訂）}]・本文は日本語だけ）を置き、P-1 に amended_by [{adr: ADR-5, date, approved_by: 持ち主, ruling: 台帳 id を含む字面, previous_text: 判断する道具を作らない, rationale}] を足して commit し `--freeze-anchor` → 0・`anchors/constitution-v1.1.yaml` が在り・索引の entries が 2 項で 2 項目が {version: v1.1, previous: v1.0, digest: 書いた anchor の digest}・続けて旗の無い check → 0／(5) (4) と同じ写しで amends の new_text を 1 字違えて `--freeze-anchor` → 1・「凍結しない」・v1.1 の anchor は作られない。

突き合わせの歯（crates/folio/tests/parity.rs（2026-09-18 に scripts/retired/ へ退役）・便 8 の 31 入力に足す・写し全部 + git 1 commit + 変異 1 つ）: (32) 上の (1) と同じ変異で `check_draft.py --emit-amends` と `folio check --emit-amends` を掛け、終了コードが 1 で一致し標準出力が byte で一致する／(33) 上の (4) と同じ合成した改訂を 2 つの写しに当て、片方に `check_draft.py --freeze-anchor`・もう片方に `folio check --freeze-anchor` を掛け、終了コードが 0 で一致し、書かれた `anchors/constitution-v1.1.yaml` を便 7 の型付きの読み手で読んだ木が等しく digest の値も等しく、索引の entries も型付きで等しい（file の字面は比べない・書き手が違う）／(34) (33) の後に両方の写しで `check_draft.py` と `folio check` を掛け 0 で一致する。既存の 31 入力は変えない。

便 8 が置いた形との接続: `crates/folio/src/main.rs` の Check に 2 つの旗を足し、出力の切り替え（違反を標準エラーへ）と凍結の呼び出しを置く。実装は新規 `crates/folio/src/freeze.rs`（(b)(c)）と `yaml.rs` の書き手（(d)）。`check.rs` の `check_dir` は旗の値を受けて便 8 の結果（最新 anchor・現行の写し・発効した判断・tracked / ever / 記録の有無）を `freeze.rs` へ渡す形に広げる（検査の式は変えない）。`anchor.rs`・`lineage.rs`・`gitcheck.rs` は読み口の可視性を広げるだけで検査の式は変えない。便 6 の `crates/folio/src/link.rs` は unit の歯の名（adr_ids_follow_the_floor_pattern）に adr を含み verify の filter adr で歯の file として解けるので、write-set に在るが触らない（本文も期待も変えない）。verify の filter freeze と yaml は base の歯の名に当たらない（admin の実測）。外部 crate は増やさない（clap と yaml-rust2 のまま・`Cargo.toml` と `Cargo.lock` は触らない）。正規表現は使わない。便 2・便 3 の歯と fixture は触らない。

## 2. 範囲

- 入れる: `--emit-amends`・`--freeze-anchor`・yaml の書き手と往復の歯・freeze の歯 5 件・parity の 3 入力。
- 入れない: 床の script `scripts/check_draft.py` と `tests/run_floor_cases.py` の削除・置換（別の便・A-1・持ち主の確認）・anchor の形式の移行・憲法 v1.1 そのものの改訂。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| emit | 差分の印字 | 最新 anchor と現行の欄単位の差分を貼れる形で出す |
| freeze | 凍結 | 前提の検査・木の組み立て・digest・書き込み・索引の追記 |
| writer | yaml の書き手 | 型付きの木を PyYAML と folio の両方で同じ型に読める字面で書く |

## 4. 検査（歯）

§1 のとおり（writer の往復の歯・freeze の歯 5 件・便 0〜便 8 の歯そのまま・parity 34 入力）。共通の検証は `.vessel.toml` の common-verify。

## 5. 依存

外部 crate は増やさない（clap / yaml-rust2 のまま・書き手は手書き）。parity の歯は CI が既に持つ Python 3 と pyyaml を使う。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "j"
title = "folio check に --emit-amends と --freeze-anchor を足す"
req = ["FR5", "NFR3"]
section = "1"
write-set = ["+crates/folio/src/freeze.rs", "crates/folio/src/yaml.rs", "crates/folio/src/main.rs", "crates/folio/src/check.rs", "crates/folio/src/anchor.rs", "crates/folio/src/lineage.rs", "crates/folio/src/gitcheck.rs", "crates/folio/src/link.rs", "+crates/folio/tests/freeze.rs", "~crates/folio/tests/parity.rs", "crates/folio/tests/check.rs", "crates/folio/tests/refs.rs", "crates/folio/tests/vocab.rs", "crates/folio/tests/adr.rs", "crates/folio/tests/link.rs", "crates/folio/tests/anchor.rs", "crates/folio/tests/gitcheck.rs"]
verify = ["cargo nextest run -p folio freeze", "cargo nextest run -p folio yaml", "cargo nextest run -p folio anchor", "cargo nextest run -p folio gitcheck", "cargo nextest run -p folio lineage", "cargo nextest run -p folio check", "cargo nextest run -p folio refs", "cargo nextest run -p folio vocab", "cargo nextest run -p folio adr", "cargo nextest run -p folio link", "cargo nextest run -p folio parity", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "M"
done = "freeze の歯 5 件が §1 のとおり緑、yaml の書き手の往復の歯が緑、便 7 の歯 anchor が期待不変で緑、便 8 の歯 gitcheck と lineage が期待不変で緑、便 0 の歯 check が期待不変で緑、便 1 の歯 refs が期待不変で緑、便 4 の歯 vocab が期待不変で緑、便 5 の歯 adr が期待不変で緑、便 6 の歯 link が期待不変で緑、parity の 34 入力が床と一致して緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
