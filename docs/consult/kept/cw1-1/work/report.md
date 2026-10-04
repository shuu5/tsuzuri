# 相談の窓 cw1 の試し — 窓の仕組みが話す窓で設計のとおり効くか

- 日: 2026-10-05（UTC）
- 窓: cw1（form=talk・starter=chat・model=fable・effort=xhigh・uttered=20261004T2207Z）
- 束: 3766ac7797cdf78f
- 問い: bundle/question.md は「# 題 テスト」の 1 行だけ。最初の相談の窓なので、窓の仕組み（判断の記録 ADR-29 決定 (7)(8)・要件 FR19）の煙の試しと読んだ。
- 根拠の読み: ADR-29 決定 (7) の末尾「対話の窓での効きは作る行で撃って確かめ、確かめられなければ起動の形を問い直す」。grill では「対話の窓でも制限の形と囲いの設定は同じに効く」が推し量り（inferred・撃っていない）に留まっていた。この窓は対話の形（話す窓）なので、ここで撃った結果がその確かめになる。

## 手順

1. 囲いの探り `work/probe.sh` を書いて `bash work/probe.sh` で撃ち、出力を `work/probe-result.txt` に置いた。探るのは 書き（作業場・私用の temp・repo・fleet・pipe・席の drafts・home・共有の /tmp）、読み（repo・fleet・pipe・台帳の写し・資格の file 7 つ）、殻のネットワーク（配り元 3 つと github.com・example.com）、環境（TZ_CONSULT_ID・temp・package の cache 4 つ）。
2. 資格の dir と file がどう覆われているかを `findmnt -T` と `stat -c %F` と `ls -A | wc -l` で見た。
3. 守りの hook を `tz consult list`（席の口）で撃った。
4. 窓の中から `tz consult bundle cw1` で束を組み直した。
5. web の道具 WebFetch で https://example.com/ を読んだ。
6. package の入れを `uv venv .venv/probe` と `uv pip install six` で撃った。
7. 所見の草稿 `drafts/test-1.json` を書き、`tz consult answer` で出した（この report が席に届いていれば通っている）。

## 結果

| 探り | 見立て | 結果 |
|---|---|---|
| 書き: 作業場の下・私用の temp（/tmp/tzc-42c569/claude-1001） | 書ける | 書けた |
| 書き: repo・fleet・pipe・席の drafts・home・共有の /tmp | 書けない | 書けなかった |
| 読み: repo・fleet・pipe・bundle/ledger.json | 読める | 読めた |
| 読み: ~/.ssh・~/.config/gh・state dir の accounts・/tmp/claude-1001 | 見えない | 空の tmpfs で覆われ、項目 0（masked） |
| 読み: ~/.cld-env・repo/.beads/.env | 見えない | /dev/null の文字装置で覆われる（masked） |
| 読み: ~/.git-credentials | 見えない | この host に無い（absent） |
| 殻のネットワーク: registry.npmjs.org・pypi.org・index.crates.io | 出られる | 出られた |
| 殻のネットワーク: github.com・example.com | 断られる | 断られた（sandbox_violations に user denied） |
| web の道具: WebFetch https://example.com/ | 読める | 読めた（殻の許しの外で動く） |
| 守りの hook: tz consult list | 断る | 「相談の窓の守り: 窓から tz consult list は撃てない（所見は answer・束は bundle）」で断られた |
| 束の組み直し: tz consult bundle cw1 | 通る・要約値は不変 | 「束 3766ac7797cdf78f」・bundle/digest と同じ |
| package の cache の環境変数 4 つ | 作業場の下 | 全部作業場の下（node_modules/.npm-cache・.venv/pip-cache・target/cargo-home・.venv/uv-cache） |
| package の入れ: uv venv + uv pip install six | 作業場に入る | six 1.17.0 が .venv/probe の下に入り、cache は .venv/uv-cache に置かれた（.venv は 320K） |
| 所見の口: tz consult answer drafts/test-1.json | findings/cw1-1.json に置かれる | （席がこの所見を読めていれば通った） |

探りの 1 回目は、空の tmpfs で覆われた dir を `ls` が通るので「読めた」と誤って出した（4 行 unexpected）。覆いの形を findmnt と stat で読む形に直して 2 回目を撃ち、全部が見立てに合った（work/probe-result.txt は 2 回目）。

## 器（Claude Code）の dontAsk の形で殻の命令が黙って断られる観察

囲い（OS）の断りではなく、器の許可の形の断り。断りの字は「Permission to use Bash has been denied because Claude Code is running in don't ask mode」で、守りの hook の字（「相談の窓の守り: …」）と別。

- 断られた: `env | grep … | sed …; find …/.claude …; cat …/.claude/settings.json`／`for … "$(command -v … 2>/dev/null || echo -)" …`／`tz consult bundle cw1; echo "rc=$?"; head …; ls …`／`echo "rc=$?"`（単独）／`python3 - <<'EOF' …`（草稿の欄を検める短い script・同じ形の heredoc が直前に 1 回通っている）。
- 通った: `echo $(echo x)`（命令の置き換えは引き金でない）／`printf … "$TZ_CONSULT_ID"`／`bash work/probe.sh 2>&1 | tee …`／`tz consult bundle cw1`（単独）／`uv venv … && uv pip install … && …`／`python3 - <<'EOF' …`（台帳の写しを読む script）。
- 読み: 引き金は器の命令の分類（許可の prompt が要ると判じた物を dontAsk が断る）で、似た命令でも結果が揺れる。tsuzuri の起動の設定では変えられない。複雑な殻の命令は work/ の script に書いて `bash work/x.sh` で撃つと通る。束の手引き brief.md にこの 1 行を足すと、窓の無駄な断りが減る（席の判断）。

## memo t3-hub.78 との関わり

台帳の写しの memo t3-hub.78（相談の窓の残りの 5 行）の最後の記帳（席 2026-10-04T21:39Z）は「持ち主が相談の窓を開ける確かめは記帳が無い。持ち主に 1 度開いてもらい（/tsuzuri:consult・話す窓）、開けたらこの memo に昇格の行を書いて閉じる」と書く。この窓 cw1 は持ち主のチャット（発話 20261004T2207Z・その記帳の後）で開いた話す窓なので、この試しがその確かめに当たる。所見の題はそのため t3-hub.78 にした（所見の行がその memo に置かれる）。問う窓（claude -p）の分の確かめは、窓からは起こせない（席の口）ので席の手で残る。

## 結び

話す窓（対話の形）でも、ADR-29 決定 (7) の囲い（作業場の外へ書けない・資格の file が見えない・殻のネットワークは配り元だけ・web の道具は別）と守りの hook と束の組み直しと package の入れが設計のとおり効いた。grill で推し量りに留まっていた「対話の窓でも同じに効く」は、この結果で撃って確かめた事になる。囲いの穴は見つからなかった。使い勝手の引っ掛かりは器の dontAsk の黙った断りだけで、script 経由の書き方で回れる。
