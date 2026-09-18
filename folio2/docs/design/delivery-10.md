# 設計: 便 10 — 凍結 fixture `tests/floor_cases.yaml` の 134 case を folio の歯で回し、床との残差を無くす

- 要件: FR5（3 値・実行できなかった検査を合格にしない）/ NFR3（正本の内部の整合）
- 条: P-10.1（検査は生成側からも検査側からも独立した凍結 anchor を 1 本以上持つ）/ P-10.2（生成物どうしの突き合わせを唯一の合格判定にしない）/ P-7.1（id を再利用・改番しない）/ P-3.1（機械で決定的に検査できる項目は床に置く）
- 判断の記録: ADR-1（帰結「M0 で folio が同じ検査を出せた時点で床の script を退役」）/ ADR-2（凍結 anchor）。便 9（f2-648.19）の後に直列で置く。
- 裁定: 持ち主 2026-09-17「進めて良い」と前もっての確認（f2-648 notes）。床の script の削除は本便に含めない（A-1・別に問う）。
- 置き場: この文書は folio2 の設計ノート（M1 で YAML 正本へ移す）。契約表は末尾の区間。審査の材料は行 k が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は `+`）。`tests/floor_cases.yaml` は凍結 fixture で触らない（write-set に無い）。

## 1. 目的と中身

day-1 の床 `scripts/check_draft.py` が RED になる凍結 fixture `tests/floor_cases.yaml`（134 case・f2-648.2 の受入・生成側からも検査側からも独立した anchor・P-10.1）を、Python の runner `tests/run_floor_cases.py` と同じ規則で回す Rust の歯 `crates/folio/tests/floor_cases.rs` を置き、`folio check` で 134 case すべてを通す。便 9 までの `folio check` に fixture を当てた実測（planner・2026-09-17・main 0a3b321・runner の呼び先を folio に替えた写し）は 125 / 134 で、残る 9 case の差は 4 種。本便でその 4 種を folio 側で直す（fixture は触らない）。

(a) 残差の是正（day-1 と同じ字面・同じ終了コードに揃える。検査の式は変えない）:
- 文言 1（4 case: chain-restart-same-version・anchors-all-deleted-tracked・older-anchor-deleted-tracked・git-env-redirect）: `crates/folio/src/gitcheck.rs` の「anchors/<名> は版管理の HEAD にあるが作業ツリーに無い（anchor と索引は消さない）」を床の字面「anchors/<名> は版管理（HEAD）にあるが作業ツリーに無い（anchor と索引は消さない）」に改める。便 8 の歯 `crates/folio/tests/gitcheck.rs` の (2) はこの文言を「HEAD にあるが作業ツリーに無い」（HEAD の後が半角空白）で名指しており新しい字面に含まれない（admin の実測 2026-09-17 main 0a3b321）ので、その needle 1 か所を「にあるが作業ツリーに無い」に改める（期待の終了コードと件数は不変・tests/gitcheck.rs は write-set に在る）。
- 文言 2（freeze-refuses-broken）: 憲法の条の plain が空のとき、`crates/folio/src/check.rs` の欄の非空は「条 P-1 の plain が空」と出すが、床は種別 R-10 で「P-1: plain が無い」と出す。folio は条の plain だけ、欄の非空の違反に加えて（違反の数は 1 のまま・置き換える）種別 R-10・文言「<条 id>: plain が無い」で出す。他の欄の非空の文言は変えない。
- 文言 3（statement-id-duplicate）: 規範文 id の重複を `check.rs` は種別 重複キー・「constitution.yaml: 行 id「P-8.1」が重複」と出すが、床は種別 schema・「規範文 id が重複（同じ id の規範文が 2 本以上＝欄単位の消し込みが id で潰れる・P-7.1）: [P-8.1]」。folio は規範文の重複だけ床の字面（種別 schema・重複した id の一覧を末尾に）に改める。条の重複・語彙・要件書の行 id の重複の文言は変えない。
- 検査の漏れ（article-id-shadows-section）: 床の schema の節にある「条 id の形」の検査が folio に無い。憲法の各条の id が P・A・N のどれか + 「-」+ 数字列 の全体一致でなければ 1 違反（種別 schema・文言「<id>: 条 id の形（P-n / A-n / N-n）でない」）を `check.rs` の憲法の検査に足す（便 6 の「条 id が改訂の範囲の節名と同じ」はそのまま・両方出る）。fixture の 16 組の条 id は P-1・P-2 だけ（実測）なので他の歯の期待は変わらない。
- 判断の記録の重複キー（adr-duplicate-key）: 床は判断の記録の file を重複キーを拒む読み手で読み、重複があれば「読めない」で終了コード 2。folio の `adr.rs` は重複キーを違反として続けている。judgement を床に揃え、判断の記録の file（schema.yaml を含む）に重複キーがあれば「まだ分からない（読めない側・文言に file 名と行）」にする。正本 4 file の重複キー（便 0・違反 1）は変えない（床も 4 file は違反で数える）。
- 正規化できない値（type-change-unrecorded）: 便 8 の (b) が「[」か「{」で始まる文字列を fail-closed で測れないにしていたのを、床と同じ判定に改める。床は文字列を json として読めるかを試し（json.loads）、読めれば json の文字列として二重引用符で囲み、読めなければそのまま。folio は json の受理器を持つ（正規表現なし・値 = null / true / false / 数〔任意の「-」+ 0 か 1〜9 で始まる数字列 + 任意の「.」数字列 + 任意の e/E 指数〕/ 二重引用符の文字列〔逆斜線の escape と \u 4 桁を含む〕/ 一覧〔[ ]・「,」区切り〕/ 表〔{ }・二重引用符のキー + 「:」〕・要素間の空白は許す・先頭と末尾の空白も許す・全体が 1 値で終わること）。受理すれば json の文字列として囲み、しなければそのまま。実測の case の値「[reject, build-check, human-review, none]」は json でない（裸の語）ので そのまま になり、床と同じく差分として現れる。

(b) 歯 `crates/folio/tests/floor_cases.rs`（binary 経由・`tests/floor_cases.yaml` を読む・1 case = 1 `#[test]` でなく 1 本の歯が全 case を回して落ちた case を名指す）。runner の規則を写す:
- 読み: 先頭の欄 expected_cases（134）と cases で始まる各節の一覧を繋いだものが case。件数が違えば歯を落とす。各 case の欄 = id・why・expect_rc・mutate（一覧・要素は 1 段か 1 段の一覧〔alias〕）・任意で expect_msg・expect_stderr・expect_not_msg・expect_n・expect_file・expect_no_file・no_git。expect_rc が 0 でない case は expect_msg か expect_stderr を持つ（無ければ歯を落とす）。
- 写し: case ごとに一時 dir に `design-intent/` を丸ごと写し（adr/・anchors/ 込み）、no_git でなければ一時 dir の根で git init と add -A と 1 commit（user.email と user.name は `-c` で与える）。
- 段の適用（順に・mutate の各段は次のどれか 1 つ）: freeze_anchor（`folio check --dir <写し> --freeze-anchor` を撃ち、最後の段でなく終了コードが freeze_rc〔既定 0〕と違えば失敗の印・最後の段でなく 0 で commit〔既定 true〕なら git commit）／emit_amends_into: <adr file>（`--emit-amends` を撃ち、終了コードが 2 なら失敗の印、そうでなければ標準出力から「#」で始まらない行を YAML の一覧として読み、その判断の記録の amends に置く）／git_snapshot・git_commit（.git が無ければ init・add -A・commit。commit は runner と同じく `--allow-empty`〔木が変わらない段でも通す〕・凍結後の commit も同じ）／git_ignore_anchors（根の .gitignore に design-intent/anchors/ を書き、`git rm -r --cached design-intent/anchors`・commit）／git_nested（写しの design-intent 自体で init・commit）／git_ignore_anchors_keep_tracked（.gitignore だけ書いて commit）／git_ignore_pattern: <pattern>（.gitignore にその 1 行・commit）／git_reinit_no_commit（.git を消して init だけ）／git_orphan_drop_anchors（`checkout --orphan clean`・anchors を cached から外し dir も消す・commit）／git_shallow_clone（`git clone --depth 1 file://<根> <根>/shallow` し、以後の写し = shallow/design-intent）／env_git_dir_empty（根の下に空の repo を init し、以後の folio の起動に GIT_DIR = その .git・GIT_WORK_TREE = 根 を環境変数で渡す）／refreeze_in_fresh_repo（写しを根/fresh/design-intent へ写し anchors/ を消し init・commit・そこで `--freeze-anchor`〔終了コード 0 なら失敗の印〕・できた anchors/*.yaml を元の写しの anchors/ へ写す）／anchor_forge: {file, path, value, add}（その anchor を型付きで読み path の欄を置き換え、digest を便 7 の正規化 + sha256 で計算し直し〔digest を除く全欄の正規化の sha256 hex・runner の digest_of と同じ〕、索引の同じ版の digest も書き換える）／symlink_dir・symlink_file（対象を根/outside へ動かし symlink を置く）／delete_dir: dir（dir を消す）／delete: file（file を消す）／create: <木>（file を書く・親 dir を作る）／write_text: <文字列>（そのまま書く）／swap: [i, j]（path の一覧の 2 要素を入れ替える）／raw_key: <鍵> + value（path の表に生の鍵で置く）／pop（path の一覧の末尾を消す）／それ以外 = {file, path, value, add}（path の欄を置き換える・欄が無ければ add のときだけ新設・一覧の末尾の次への add は追加）。path の文法 = 「.」区切りの欄名・「[<数>]」の添字・「[<欄>=<値>]」（一覧から欄の字面が値に等しい要素を選ぶ）。木の読み書きは便 7 の型付きの読みと便 9 の書き手（値の型を保つ・日付は plain）。
- 実行と判定: 最後の段が freeze なら その終了が結果、さらに 0 なら素の `folio check` も回して 0 でなければ失敗の印・結果はそちら。そうでなければ素の `folio check`。判定 = 終了コードが expect_rc と同じ ∧ expect_msg が標準出力 + 標準エラーに含まれる ∧ expect_stderr が標準エラーに含まれる ∧ expect_not_msg が含まれない ∧ expect_n が在れば標準出力の「[」で始まる行の数が同じ ∧ expect_file の各 path が写しに在る ∧ expect_no_file の各 path が無い ∧ 失敗の印が無い。落ちた case は id・終了コード（期待）・why・出力の先頭 10 行を歯の失敗の文言に入れる。
- 段の適用で例外（欄が無い・path が解けない・命令が失敗）は その case の失敗。
- 歯は tests/floor_cases.yaml を読むだけで書かない。写しは一時 dir（歯の終わりに消す）。
- 歯からの型付きの読み書きと digest: folio は bin だけで統合の歯から src の関数を呼べないので、歯の先頭で path 属性（`#[path]`・値は ../src/sha256.rs と ../src/yaml.rs）と `#[allow(dead_code)]` を付けた `mod sha256;` と `mod yaml;` の形で source を取り込む（sha256.rs は crate の参照 0・yaml.rs の crate の参照は unit の歯の中の sha256 の 1 つだけで、両方を歯の crate の根に置けば解ける・yaml_rust2 は通常の依存で歯からも使える・admin の実測）。使う口 = yaml の parse_typed・canonical・write と sha256 の hex（すべて pub・可視性の変更は要らない）。`#[allow(dead_code)]` は必須（取り込んだ module の未使用の関数が clippy の -D warnings で落ちる）。lib 化や digest の副命令は構造の判断になるので採らない。

(c) 期待: 134 / 134 が通る（(a) の是正の後・planner の実測は 125 / 134 で、残り 9 が (a) の 4 種に対応: 文言 1 = 5 case〔chain-restart-same-version・anchors-all-deleted-tracked・older-anchor-deleted-tracked・git-env-redirect の 4 は同じ文言・anchors-deleted-committed は便 9 で通った〕・文言 2 = freeze-refuses-broken・文言 3 = statement-id-duplicate・漏れ = article-id-shadows-section・重複キー = adr-duplicate-key・正規化 = type-change-unrecorded）。既存の歯 17 組と parity 34 入力の期待は、上の tests/gitcheck.rs の needle 1 か所を除いて変わらない（文言の変更は他の歯が名指す文言に掛からない: `tests/check.rs` は「[重複キー] rules.yaml」「[欄の非空] vocabulary.yaml」の先頭だけ・`tests/adr.rs` は schema-drift と adr/schema.yaml・`tests/vocab.rs` は場所「P-1 plain」・`tests/refs.rs` 等は R-4 / adr / N-4 の文言・admin と planner の実測）。

便 9 が置いた形との接続: 実装の変更は `check.rs`（条 id の形・plain の文言・規範文 id の重複の文言）・`adr.rs`（重複キーを読めないに）・`gitcheck.rs`（文言）・`lineage.rs`（json の受理器・「[」「{」の fail-closed を外す）。新規は歯 `crates/folio/tests/floor_cases.rs` だけ。`crates/folio/src/link.rs` は unit の歯の名（adr_ids_follow_the_floor_pattern）に adr を含み verify の filter adr で解け、`crates/folio/src/yaml.rs` は便 9 の unit の歯の名（yaml_writer_round_trips_the_main_anchor）に anchor を含み filter anchor で解けるので、どちらも write-set に在るが触らない（本文も期待も変えない）。verify の filter floor_cases は base の歯の名に当たらない（floor を含む名は在るが floor_cases は無い・実測）。外部 crate は増やさない（clap と yaml-rust2 のまま・`Cargo.toml` と `Cargo.lock` は触らない）。正規表現は使わない。便 2・便 3 の歯と fixture は触らない。`tests/run_floor_cases.py` と `scripts/check_draft.py` は触らない。

## 2. 範囲

- 入れる: 残差 4 種の是正（文言 3 か所・条 id の形・判断の記録の重複キー・json の受理器）・歯 `floor_cases.rs`（runner の規則の写し・134 case）。
- 入れない: `tests/floor_cases.yaml` の変更（凍結）・`scripts/check_draft.py`・`tests/run_floor_cases.py`・`scripts/render_preview.py` の削除や変更（床の退役は A-1・別に問う）・parity の歯の撤去。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| align | 残差の是正 | 文言 3 か所・条 id の形・重複キー・json の受理器 |
| runner | case の実行 | 写し・段の適用・実行・判定を runner と同じ規則で |
| mutate | 段の適用 | path の文法・型付きの木の置き換え・git・symlink・凍結・差分の貼り付け |

## 4. 検査（歯）

§1 のとおり（floor_cases の歯 = 134 case・便 0〜便 9 の歯そのまま・parity 34 入力）。共通の検証は `.vessel.toml` の common-verify。

## 5. 依存

外部 crate は増やさない（clap / yaml-rust2 のまま）。git は命令として子 process で使う。parity の歯は CI が既に持つ Python 3 と pyyaml を使う（本便では変えない）。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "k"
title = "凍結 fixture の 134 case を folio の歯で回し、床との残差を無くす"
req = ["FR5", "NFR3"]
section = "1"
write-set = ["+crates/folio/tests/floor_cases.rs", "crates/folio/src/check.rs", "crates/folio/src/adr.rs", "crates/folio/src/gitcheck.rs", "crates/folio/src/lineage.rs", "crates/folio/src/link.rs", "crates/folio/src/yaml.rs", "~crates/folio/tests/parity.rs", "crates/folio/tests/check.rs", "crates/folio/tests/refs.rs", "crates/folio/tests/vocab.rs", "crates/folio/tests/adr.rs", "crates/folio/tests/link.rs", "crates/folio/tests/anchor.rs", "crates/folio/tests/gitcheck.rs", "crates/folio/tests/freeze.rs"]
verify = ["cargo nextest run -p folio floor_cases", "cargo nextest run -p folio check", "cargo nextest run -p folio adr", "cargo nextest run -p folio gitcheck", "cargo nextest run -p folio lineage", "cargo nextest run -p folio refs", "cargo nextest run -p folio vocab", "cargo nextest run -p folio link", "cargo nextest run -p folio anchor", "cargo nextest run -p folio freeze", "cargo nextest run -p folio parity", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "M"
done = "floor_cases の歯が 134 case すべてで緑、便 0 の歯 check が期待不変で緑、便 5 の歯 adr が期待不変で緑、便 8 の歯 gitcheck と lineage が期待不変で緑、便 1 の歯 refs が期待不変で緑、便 4 の歯 vocab が期待不変で緑、便 6 の歯 link が期待不変で緑、便 7 の歯 anchor が期待不変で緑、便 9 の歯 freeze が期待不変で緑、parity の 34 入力が床と一致して緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
