//! 接頭辞 `vrmiss_`。
//! 審査の材料の欠けの印を器が判じて lens を撃たずに INCONCLUSIVE にする歯（契約の審査と行の審査）。

use super::*;

/// 偽 lens（撃たれた時に `marker` を置いて PASS を返す）で `design` を intake する（引数は `intake_raw` と同じで `--lens` だけが違う）。
fn vrmiss_intake(repo: &Path, state: &Path, marker: &Path) -> Output {
    run_pipe(&[
        "intake", "--design", "docs/design/toy.md#a", "--bead", "s2-a",
        "--repo", &repo.display().to_string(), "--state-dir", &state.display().to_string(),
        "--rules", &ceiling_rules(state), "--lens", &fake_lens(marker, &lens_verdict("PASS")),
    ])
}

/// 欠けの印の evidence の頭（全角のコロンで本文の行に繋ぐ）。
const VRMISS_HEAD: &str = "審査の材料が欠ける：";

/// 1 つ目と 2 つ目の項: 裸の yaml の id は lens を撃たず、rc 3・Reviewed の detail・review.json・材料の requirements.txt が欠けの印を持つ。
#[test]
fn vrmiss_bare_yaml_id_stops_before_the_lens() {
    let (repo, state) = faced_repo_with_req("spec/reqs.yaml", "requirements:\n  - FR1\n  - FR2\n", &["FR1"]);
    let marker = state.join("vrmiss-lens-ran");
    let out = vrmiss_intake(&repo, &state, &marker);
    assert_eq!(out.status.code(), Some(i32::from(RC_INCONCLUSIVE)), "{} / {}", stdout_of(&out), stderr_of(&out));
    assert!(stdout_of(&out).contains("stage=Reviewed verdict=INCONCLUSIVE"), "{}", stdout_of(&out));
    let id = run_id_of(&out);
    assert_eq!(reviewed_detail(&state, &id), "verdict:INCONCLUSIVE kind:section-material-missing");
    assert!(!marker.exists(), "偽 lens は撃たれない");
    let missing = "（要件面 spec/reqs.yaml の FR1 に本文が無い）";
    let pairs = review_pairs(&state, &id);
    assert_eq!(value_of(&pairs, "kind"), "section-material-missing");
    assert_eq!(value_of(&pairs, "evidence"), format!("{VRMISS_HEAD}FR1 {missing}"));
    for key in ["at", "tree", "quality"] {
        assert!(!review_has(&state, &id, key), "{key} を持たない");
    }
    let body = fs::read_to_string(review_dir(&state, &id).join("requirements.txt")).unwrap_or_default();
    assert_eq!(body, format!("FR1: {missing}\n"), "材料の file は全部置く");
    clean(&[&repo, &state]);
}

/// 3 つ目の項: md の空の見出しは欠けた id だけを名指し、本文を持つ隣の id の字を evidence に載せない。
#[test]
fn vrmiss_empty_md_heading_names_only_the_missing_id() {
    let md = "# 要件\n\n## FR1\n\n\n## FR2 審査\n\n審査する。\n";
    let (repo, state) = faced_repo_with_req("spec/reqs.md", md, &["FR1", "FR2"]);
    let marker = state.join("vrmiss-lens-ran");
    let out = vrmiss_intake(&repo, &state, &marker);
    assert_eq!(out.status.code(), Some(i32::from(RC_INCONCLUSIVE)), "{} / {}", stdout_of(&out), stderr_of(&out));
    let id = run_id_of(&out);
    let evidence = value_of(&review_pairs(&state, &id), "evidence");
    assert_eq!(evidence, format!("{VRMISS_HEAD}FR1 （要件面 spec/reqs.md の FR1 に本文が無い）"));
    assert!(!evidence.contains("審査する"), "本文の在る id は欠けの印に載せない: {evidence}");
    assert!(!marker.exists(), "偽 lens は撃たれない");
    clean(&[&repo, &state]);
}

/// 4 つ目の項（否定の見本）: 同じ yaml の形で FR1 が text を持てば lens を撃ち、PASS で rc 0。
#[test]
fn vrmiss_requirement_with_a_body_fires_the_lens() {
    let yaml = "requirements:\n  - id: FR1\n    text: 便を起こす\n  - FR2\n";
    let (repo, state) = faced_repo_with_req("spec/reqs.yaml", yaml, &["FR1"]);
    let marker = state.join("vrmiss-lens-ran");
    let out = vrmiss_intake(&repo, &state, &marker);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "{} / {}", stdout_of(&out), stderr_of(&out));
    assert!(marker.exists(), "偽 lens は撃たれる");
    assert_eq!(reviewed_detail(&state, &run_id_of(&out)), "verdict:PASS");
    clean(&[&repo, &state]);
}

/// 5 つ目の項: 行の審査も欠けの印の行を撃たず、INCONCLUSIVE・section-material-missing の記録を書く。
#[test]
fn vrmiss_row_review_does_not_fire_on_a_missing_material() {
    let place_rows = [rv_row("x", 1, &[("req", "[\"FR1\"]")])];
    let reqs = "# 要件\n\n## FR1\n\n\n## FR2 審査\n\n審査する。\n";
    let place = rv_place(&["本文 1"], &place_rows, &[("reqs.md", reqs.to_owned())]);
    rv_ledger(&place, &[prelens_issue("s2-rv.1", "open", "x", &[])], None);
    let edited = [rv_row("x", 1, &[("req", "[\"FR1\"]"), ("done", "\"done を変えた\"")])];
    let sha = rv_commit(&place, None, &[(DESIGN_FILE, rv_doc(&["本文 1"], &edited))]);
    let out = rv_review(&place, &sha);
    assert!(rv_log(&place).is_empty(), "偽 lens の呼びの log は空");
    let row = rv_row_out(&out, "x");
    assert_eq!(row.get("verdict").map(String::as_str), Some("INCONCLUSIVE"), "{} / {}", stdout_of(&out), stderr_of(&out));
    let record = rv_record(&place, row.get("record").map_or("", String::as_str));
    assert_eq!(record.get("kind").map(String::as_str), Some("section-material-missing"), "{record:?}");
    clean(&[&place.repo, &place.state]);
}
