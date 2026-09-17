# 設計: 便 8 — `folio check` に版管理との照合と列の区間の消し込みを足す

- 要件: FR5（3 値・実行できなかった検査を合格にしない）/ NFR3（正本の内部の整合）
- 条: A-2.1（条文の改訂は判断の記録と承認を伴う）/ N-4.1（記録を欠く改訂を拒む）/ P-7.1（id を再利用・改番しない）/ P-4.1（検査が実行できなかった結果を「異常なし」として扱わない）/ N-1.1（管理下の対象を回復不能に削除する操作を拒む）
- 判断の記録: ADR-2（凍結 anchor の列・版管理との照合・区間の消し込み）。便 7（f2-648.17）の後に直列で置く。
- 裁定: 持ち主 2026-09-17「進めて良い」と前もっての確認（f2-648 notes）。
- 置き場: この文書は folio2 の設計ノート（M1 で YAML 正本へ移す）。契約表は末尾の区間。審査の材料は行 i が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は `+`）。fixture は増やさない（歯の中で一時 dir に写しを作る）。

## 1. 目的と中身

便 7 の `folio check`（凍結 anchor の欄・digest・索引・列・現行との一致）に、day-1 の床 `scripts/check_draft.py` の anchor の節の残り 2 つ、**版管理（git）との照合**と**列の区間（隣り合う anchor どうし）の amends の 1:1 の消し込み**を同じ式で足す。この便では床の script を触らない。`--emit-amends`・`--freeze-anchor`（書く側）は便 9。

(a) 版管理との照合。git は外部 crate でなく命令 `git` を子 process で撃つ（day-1 と同じ・環境変数のうち GIT_ で始まるものは渡さない＝GIT_DIR / GIT_WORK_TREE で照合先をすげ替えられない・各命令の待ち上限 20 秒）。手順と判定:
- `git -C <dir> rev-parse --show-toplevel` が失敗 → 測れない（pending・「版管理（git）が無いか読めない」）で (a) を終える。成功したときの根（実体に解いた path）が `<dir>` そのものなら 1 違反（anchor・design-intent 自体が版管理の根）、`<dir>` の上位でなければ 1 違反（anchor・照合先が違う）で (a) を終える。
- `rev-parse --verify -q HEAD` が失敗 → 測れない（pending・「commit が 1 つも無い」）で終える。`rev-parse --is-shallow-repository` が true で作業ツリーに anchor file が 1 本も無い → 測れない（pending・浅い写し）で終える。
- `check-ignore -q --no-index <anchors/ の相対 path>` と、作業ツリーに在る各 anchor file の相対 path について、終了コード 0（除外の規則が当たる）なら 1 違反ずつ（anchor・版管理から除外されている・追跡済みでも落とす）。
- `ls-tree -r --name-only HEAD -- <anchors/>` で HEAD にある anchor の file 名の集合（tracked）、`log --all --format=%H --name-status -- <anchors/>` で全ての参照の履歴に一度でも在った file 名の集合（ever・状態が D の行は入れない）と、状態が A・M・R・C で名前が constitution- で始まる各行について `show <commit>:<path>` の本文（hist）を集める（R と C は矢印の右の path）。どちらかの命令が失敗 → 測れない（pending・「版管理を読めない」）で終える。
- tracked に在って作業ツリーに無い file → 1 違反ずつ（anchor・HEAD にあるが作業ツリーに無い）。ever に在って tracked にも作業ツリーにも無い file → 1 違反ずつ（anchor・履歴に在ったが無い・削除を commit しても列の始め直しは認めない）。作業ツリーに在り hist にも在る各 file について、hist の各本文が今の本文と byte で違い、かつその本文を型付きで読めて**同じ形式**（欄の集合が床の file_keys と一致・digest_algo が床と同じ・projection の article_fields と statement_fields が床と同じ）なら 1 違反（anchor・履歴の同じ形式の anchor と中身が違う＝差し替え）で次の file へ（形式が違う古い anchor は移行の痕跡として見ない・読めない本文は飛ばす）。作業ツリーに在って tracked に無い anchor file（索引を除く）のうち、最新の版（索引の末尾）の file 名でないもの → 1 違反ずつ（anchor・追跡されていない・列の差し替え）。
- `<dir>/anchors/` が無い組（既存の fixture 15 組）では check-ignore・ls-tree・log の対象 path が存在しないが、それは違反にも測れないにもしない（tracked と ever が空・作業ツリーの anchor も無い＝照合するものが無いだけ・便 7 の (i) の pending が立つ）。命令自体の失敗（git が無い・読めない）だけを測れないにする。
- これらは便 7 の (f)〜(i) と同じ Report に積む（種別は文字列・pending は便 7 の測れないの一覧。Report の pending の口・yaml.rs の型付きの読み parse_typed と正規化 canonical・adr の記録 Adr・anchor の check_anchor と history_ids は便 7 が公開済み〔admin の実測 2026-09-17 main a1d40f4〕。anchor.rs の中の 写し（project）・正規化（canon）・digest（digest_of）・型付きの再読み（read_typed）・条の消失と改番（structural_diff）は非公開なので、便 8 は anchor.rs の中で crate の中へ可視性を広げて使う〔式は変えない・anchor.rs は write-set に在る〕）。

(b) 列の区間の消し込み。索引の entries の隣り合う 2 つ（前の版 pv・後の版 cv）で両方の anchor が読めているものについて、前の anchor の content と後の anchor の content を**欄単位**で比べ、その版（cv）を名指す発効した判断の amends と 1 対 1 に消し込む。欄単位の比べ方（day-1 の diff_targets と同じ）:
- 対象 = 範囲（各 anchor の projection.scope）の各節（articles 以外）と各条 id。節は「欄の道 → 値」に平らにする（表は道を「.」で伸ばし、一覧は丸ごと 1 値・節が無い側は空の表・節の値が表でないなら道 value に 1 値）。条は title・tier・binds の 3 欄と、各規範文の text・pattern・strength を statements.<規範文 id>.<欄> の道で持つ。
- 値の表現（render_val）: 文字列は そのまま、ただし空文字は印（空）、json として読める字面の文字列（null・true・false・数・二重引用符で始まる文字列・「[」か「{」で始まる字面）は json の文字列として二重引用符で囲む（便 7 の (c) の Str の形）。文字列以外（一覧・表・数・真偽・null・日付）は便 7 の (c) の正規化の字面（日付は文字列として）。「[」か「{」で始まる文字列が json として読めるかの判定は便 8 では行わず、その字面を持つ文字列が対象の中に在れば 測れない（pending・正規化できない値・欄の道）に落とす（fail-closed・正本と anchor v1.0 の実測 2026-09-17: 該当 0）。
- 差分 = 前後で値が違う (対象, 欄の道) の全件。片側に無い欄は、前に無ければ 前の値を印（新設）、今に無ければ 今の値を印（削除）とする。加えて条の並び（両側に在る条 id だけを前後の順で「・」で繋いだ字面が違えば 対象 articles・欄の道 order）と、各条の規範文の並び（同じく statements.order）を差分に入れる。
- 記録に依らない検査: 前に在って今に無い条 id → 1 違反ずつ（P-7・条は消さない）。前で消えた規範文（今に無い規範文 id）の text の集合と、今で新設の規範文（前に無い id）の text の集合が交われば、交わる text ごとに 1 違反（P-7・改番）。
- 消し込み: cv を version に持つ amends を持つ発効した判断（ver_adrs）の各 amends 項を (target, field) → (判断の id, 項) に集め、同じ (target, field) が 2 つの判断に在れば 1 違反（A-2・重複して記録）。差分の各 (対象, 欄の道)（消えた条の分は除く）について記録が無ければ 1 違反（N-4・記録が無い・文言に前後の値の先頭 40 字）、在れば previous_text が前の値と・new_text が今の値と完全一致しなければ 1 違反（A-2）。余った記録は、対象が消えた条なら 1 違反（A-2・条は消さない）、そうでなければ 1 違反（A-2・差分が無い）。差分の対象が現行の条 id なら、その条の amended_by に ver_adrs のどれかを adr に持つ項が無ければ 1 違反（N-4）。
- 便 7 の (i) の「最新の版が現行と違う」ときの条の消失・改番は便 7 のまま（この (b) は列の区間だけ・現行との消し込みは便 9 の凍結の時に同じ関数を回す）。

(c) 判定は便 7 の 3 値。正本（main）では違反 0・測れない 0（版管理の根 = folio2 の repo・HEAD あり・浅くない・anchors/ は除外されず tracked・履歴の anchor は今の本文と同じ・列は v1.0 の 1 本で区間なし。day-1 の床の実測 2026-09-17）。

既存の fixture 16 組（`tests/fixtures/` の check 3・refs 3・vocab 2・adr 3・link 3・anchor 2）は folio2 の repo の中に在るので (a) の照合先は folio2 の repo で、HEAD あり・浅くない・anchors/ は 2 組（anchor/root-digest-drift/ だけが持つ）とも除外されず tracked・履歴の本文と同じ・索引の末尾の版の file だけ → (a) の違反 0・測れない 0。列の区間は無い（最大 1 本）→ (b) は動かない。よって各歯（`crates/folio/tests/check.rs`・`refs.rs`・`vocab.rs`・`adr.rs`・`link.rs`・`anchor.rs`）の期待は変わらない。これら 6 本の歯は verify で回すために write-set に在るが触らない（本文も期待も変えない）。fixture 16 組も触らない。

`folio check` 自身の歯（`crates/folio/tests/gitcheck.rs`・binary 経由・fixture は増やさず歯の中で一時 dir に写しを作る）: (1) `tests/fixtures/anchor/no-anchor/` を一時 dir に写し git を作らずに掛ける → 終了コード 2・違反 0・標準エラーに「版管理（git）が無いか読めない」を含む行／(2) `tests/fixtures/anchor/root-digest-drift/` を一時 dir に写し `git init` + `add -A` + 1 commit の後に `anchors/constitution-v1.0.yaml` を消して掛ける → 終了コード 1・違反に「HEAD にあるが作業ツリーに無い」を含む行が 1 つ在り、便 7 の根の digest の違反も残る（違反は 2 件以上・終了コード 1）／(3) 同じ写しで commit の後に `.gitignore` に anchors/ を書いて掛ける → 違反に「版管理から除外」を含む行が在る／(4) 同じ写しで commit の後に anchor の content の 1 字を変えて掛ける（commit しない）→ 違反に「履歴の同じ形式の anchor と中身が違う」を含む行が在る。列の区間の消し込み (b) の歯は `crates/folio/src/lineage.rs` の unit の歯（型付きの木を 2 つ作って差分・記録の消し込み・重複・余り・条の消失・改番の各 1 件を見る・fixture を使わない）。

突き合わせの歯（`crates/folio/tests/parity.rs`・便 7 の 27 入力に足す・写し全部 + git 1 commit + 変異 1 つ・床と folio の終了コードの一致）: (28) 写しの `anchors/` を丸ごと消す → 1（HEAD にあるが無い）／(29) 写しの `anchors/constitution-v1.0.yaml` だけを消す（索引は残す）→ 1／(30) 写しの版管理の根に `.gitignore` を作り anchors/ を書く → 1／(31) 写しの `.git` を消す → 2（版管理が無い＝測れない・違反 0）。既存の 27 入力は変えない。

便 7 が置いた形との接続: `crates/folio/src/main.rs` に `mod gitcheck; mod lineage;` を足す。実装は新規 `crates/folio/src/gitcheck.rs`（(a)・`std::process::Command` で git を撃つ・環境変数の遮断・20 秒の待ち上限）と `crates/folio/src/lineage.rs`（(b)・便 7 の型付きの木と正規化を使う）。便 6 の `crates/folio/src/link.rs` は unit の歯の名（adr_ids_follow_the_floor_pattern）に adr を含み verify の filter adr で歯の file として解けるので、write-set に在るが触らない（本文も期待も変えない）。`anchor.rs` から (a) を anchors/ の読みの後に、(b) を列の検査の後に呼ぶ（便 7 の検査の式は変えない）。判断の記録（発効・amends・approval）は便 5・便 6 の `adr.rs` が読んだものを使う。外部 crate は増やさない（clap と yaml-rust2 のまま・`Cargo.toml` と `Cargo.lock` は触らない）。正規表現は使わない。便 2・便 3 の歯と fixture は触らない。

## 2. 範囲

- 入れる: 版管理との照合（根・HEAD・浅い写し・除外・HEAD と履歴の anchor・同じ形式の差し替え・未追跡）・列の区間の消し込み（欄単位の差分・記録との 1:1・重複・余り・条の消失・改番）・歯 `gitcheck.rs`（一時 dir 4 件）と `lineage.rs` の unit の歯・parity の 4 入力。
- 入れない: `--emit-amends`（最新 anchor と現行の差分を貼れる形で印字）・`--freeze-anchor`（凍結と索引への追記・現行との消し込み）・`scripts/check_draft.py` の変更。すべて便 9。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| git | 版管理との照合 | 根・HEAD・浅さ・除外・tracked / ever / hist の集合と作業ツリーの照合 |
| flatten | 欄単位 | 節と条を「欄の道 → 値の字面」に平らにし、並びの差分も持つ |
| reconcile | 消し込み | 差分と発効した判断の amends を 1:1 に消し、余りと不足を落とす |

## 4. 検査（歯）

§1 のとおり（gitcheck の歯 4 件・lineage の unit の歯・便 0〜便 7 の歯そのまま・parity 31 入力）。共通の検証は `.vessel.toml` の common-verify。

## 5. 依存

外部 crate は増やさない（clap / yaml-rust2 のまま）。git は命令として子 process で使う（CI と worker の環境に在る）。parity の歯は CI が既に持つ Python 3 と pyyaml を使う。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "i"
title = "folio check に版管理との照合と列の区間の消し込みを足す"
req = ["FR5", "NFR3"]
section = "1"
write-set = ["+crates/folio/src/gitcheck.rs", "+crates/folio/src/lineage.rs", "crates/folio/src/anchor.rs", "crates/folio/src/main.rs", "crates/folio/src/link.rs", "+crates/folio/tests/gitcheck.rs", "crates/folio/tests/parity.rs", "crates/folio/tests/check.rs", "crates/folio/tests/refs.rs", "crates/folio/tests/vocab.rs", "crates/folio/tests/adr.rs", "crates/folio/tests/link.rs", "crates/folio/tests/anchor.rs"]
verify = ["cargo nextest run -p folio gitcheck", "cargo nextest run -p folio lineage", "cargo nextest run -p folio anchor", "cargo nextest run -p folio check", "cargo nextest run -p folio refs", "cargo nextest run -p folio vocab", "cargo nextest run -p folio adr", "cargo nextest run -p folio link", "cargo nextest run -p folio parity", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "M"
done = "gitcheck の歯 4 件が §1 のとおり緑、lineage の unit の歯が緑、便 7 の歯 anchor が期待不変で緑、便 0 の歯 check が期待不変で緑、便 1 の歯 refs が期待不変で緑、便 4 の歯 vocab が期待不変で緑、便 5 の歯 adr が期待不変で緑、便 6 の歯 link が期待不変で緑、parity の 31 入力が床と一致して緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
