//! host の負荷と書きの読みの歯（接頭辞 hload_・持ち主の裁定 t3-hub.77.28）。
//! kernel の file の字・smartctl の JSON・host の面の書きの測りの表の読みと、注意の線（規則の行 R-42）の判じと電文の組み。
#![cfg(test)]

use std::path::Path;

use tsuzuri_contract::board::Reading;
use tsuzuri_contract::host::{Gauge, Load, Memory, Pressure, ScopeMemory, Stall, Wear};
use tsuzuri_core::hostload::{
    BudgetRow, DATA_UNIT_BYTES, DeviceTexts, HostTexts, IO_FULL_WARN, LOAD_WARN, MEMORY_FULL_WARN,
    SCOPES_SHOWN, ScopeTexts, budget_rows, cores, host_doc, hundredths, load, memory, over, rate,
    scopes, sectors_written, stall, wear,
};

const LOADAVG: &str = "25.36 19.68 18.47 3/2345 123456\n";
const PSI_CPU: &str = "some avg10=32.90 avg60=30.80 avg300=21.80 total=1\nfull avg10=0.00 avg60=0.00 avg300=0.00 total=0\n";
const PSI_MEM: &str = "some avg10=0.40 avg60=1.23 avg300=1.22 total=9\nfull avg10=0.40 avg60=1.21 avg300=1.21 total=8\n";
const MEMINFO: &str = "MemTotal:       131710000 kB\nMemFree:         1000 kB\nMemAvailable:   86200000 kB\nSwapTotal:       8388604 kB\nSwapFree:            204 kB\n";
const STAT: &str = "  1 2 3 4 5 6 89003451946 8 9 10 11\n";
const SMART: &str = r#"{"local_time":{"time_t":1791180420},"nvme_smart_health_information_log":{"percentage_used":6,"data_units_written":163085937,"available_spare":100,"media_errors":0}}"#;

fn known<T>(v: T) -> Reading<T> {
    Reading::Known(v)
}

fn st(avg10: u32, avg60: u32) -> Stall {
    Stall { avg10, avg60 }
}

fn calm() -> Pressure {
    Pressure {
        cpu_some: known(st(9_999, 9_999)),
        memory_full: known(st(0, 0)),
        io_full: known(st(0, 0)),
    }
}

fn ld(one: u32, cores: u32) -> Reading<Load> {
    known(Load {
        one,
        five: 0,
        fifteen: 0,
        cores,
    })
}

fn scope(name: &str, current: Option<&str>, max: Option<&str>) -> ScopeTexts {
    ScopeTexts {
        name: name.into(),
        current: current.map(Into::into),
        max: max.map(Into::into),
    }
}

/// 規則の行 R-42 の value の字のうち、`head` の直後から `tail` の前までの数の 100 倍。
fn rule_number(row: &str, head: &str, tail: &str) -> u32 {
    let (_, after) = row
        .split_once(head)
        .unwrap_or_else(|| panic!("R-42 に {head} が無い: {row}"));
    let (number, _) = after
        .split_once(tail)
        .unwrap_or_else(|| panic!("R-42 に {tail} が無い: {row}"));
    hundredths(number).unwrap_or_else(|| panic!("R-42 の数 {number} が読めない"))
}

#[test]
fn hload_hundredths_and_cores() {
    for (text, want) in [
        ("25.36", Some(2_536)),
        ("1.0", Some(100)),
        ("5", Some(500)),
        ("0.05", Some(5)),
        ("0.129", Some(12)),
        ("32.9", Some(3_290)),
    ] {
        assert_eq!(hundredths(text), want, "{text}");
    }
    for text in ["", "-1", "1.2.3", "a", ".5", "1.-2"] {
        assert_eq!(hundredths(text), None, "{text}");
    }
    for (text, want) in [("0-31\n", Some(32)), ("0", Some(1)), ("0-3,5,7-9", Some(8))] {
        assert_eq!(cores(text), want, "{text}");
    }
    for text in ["", "0-x", "3-1", "0-3,"] {
        assert_eq!(cores(text), None, "{text}");
    }
}

#[test]
fn hload_reads_kernel_texts() {
    assert_eq!(
        load(LOADAVG, "0-31\n"),
        Some(Load {
            one: 2_536,
            five: 1_968,
            fifteen: 1_847,
            cores: 32
        })
    );
    assert_eq!(load("25.36 19.68\n", "0-31"), None);
    assert_eq!(load(LOADAVG, "garbage"), None);
    assert_eq!(stall(PSI_CPU, "some"), Some(st(3_290, 3_080)));
    assert_eq!(stall(PSI_MEM, "full"), Some(st(40, 121)));
    assert_eq!(stall("some avg10=1.00 avg300=2.00 total=1\n", "some"), None);
    assert_eq!(stall(PSI_CPU, "none"), None);
    assert_eq!(
        memory(MEMINFO),
        Some(Memory {
            total: 131_710_000 * 1024,
            available: 86_200_000 * 1024,
            swap_total: 8_388_604 * 1024,
            swap_free: 204 * 1024,
        })
    );
    assert_eq!(memory(&MEMINFO.replace("SwapFree", "SwapGone")), None);
    assert_eq!(sectors_written(STAT), Some(89_003_451_946));
    assert_eq!(sectors_written("1 2 3 4 5 6"), None);
}

/// 線は 100 倍の整数で、規則の行 R-42 の value の字の数と同じ。越えた時だけ注意（ちょうどは付けない）・読めない測りは付けない。
#[test]
fn hload_lines_are_rule_r42() {
    assert_eq!((LOAD_WARN, IO_FULL_WARN, MEMORY_FULL_WARN), (100, 500, 100));
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../design-intent/rules.yaml");
    let rules = std::fs::read_to_string(path).expect("rules の file を読む");
    let row = rules
        .lines()
        .find(|l| l.contains("id: R-42,"))
        .expect("rules の file に行 R-42 が在る");
    assert_eq!(
        rule_number(row, "core 比が ", " を越え"),
        LOAD_WARN,
        "{row}"
    );
    assert_eq!(
        rule_number(row, "io の詰まり（PSI の full の avg60）が ", "% を越え"),
        IO_FULL_WARN
    );
    assert_eq!(
        rule_number(
            row,
            "memory の詰まり（PSI の full の avg60）が ",
            "% を越え"
        ),
        MEMORY_FULL_WARN
    );
}

#[test]
fn hload_over_follows_the_lines() {
    assert_eq!(over(&ld(3_200, 32), &calm()), Vec::<Gauge>::new());
    assert_eq!(over(&ld(3_201, 32), &calm()), [Gauge::Load]);
    assert_eq!(over(&ld(3_231, 32), &calm()), [Gauge::Load]);
    assert_eq!(over(&ld(u32::MAX, u32::MAX), &calm()), Vec::<Gauge>::new());
    let mut p = calm();
    p.io_full = known(st(9_999, 500));
    assert_eq!(over(&ld(0, 1), &p), Vec::<Gauge>::new());
    p.io_full = known(st(0, 501));
    assert_eq!(over(&ld(0, 1), &p), [Gauge::Io]);
    p.memory_full = known(st(0, 101));
    assert_eq!(
        over(&ld(101, 1), &p),
        [Gauge::Load, Gauge::Io, Gauge::Memory]
    );
    p.memory_full = known(st(0, 100));
    assert_eq!(over(&ld(0, 1), &p), [Gauge::Io]);
    let unknown = Pressure {
        cpu_some: Reading::Unknown,
        memory_full: Reading::Unknown,
        io_full: Reading::Unknown,
    };
    assert_eq!(over(&Reading::Unknown, &unknown), Vec::<Gauge>::new());
}

/// 上限の無い（max）・上限 0・今の量が読めない scope は落とし、埋まりの多い順・同じ埋まりは名の順に `SCOPES_SHOWN` まで。
#[test]
fn hload_scopes_rank_the_bounded() {
    let raw = vec![
        scope("free", Some("10"), Some("max")),
        scope("zero", Some("10"), Some("0")),
        scope("lost", None, Some("100")),
        scope("half-b", Some("50"), Some("100")),
        scope("half-a", Some("16"), Some("32\n")),
        scope("most", Some("26"), Some("32")),
        scope("low", Some("1"), Some("100")),
        scope("tiny", Some("0"), Some("100")),
        scope("mid", Some("30"), Some("100")),
    ];
    let names: Vec<String> = scopes(&raw).into_iter().map(|s| s.name).collect();
    assert_eq!(SCOPES_SHOWN, 5);
    assert_eq!(names, ["most", "half-a", "half-b", "mid", "low"]);
    assert_eq!(
        scopes(&raw[5..6]),
        [ScopeMemory {
            name: "most".into(),
            current: 26,
            max: 32
        }]
    );
}

#[test]
fn hload_rate_from_sectors() {
    assert_eq!(
        rate(Some((1_000, 10_000)), (1_000 + 14_080_000, 20_000)),
        known(720_896_000)
    );
    assert_eq!(
        rate(Some((1_000, 10_000)), (3_000, 10_500)),
        known(2_048_000)
    );
    assert_eq!(rate(None, (3_000, 10_500)), Reading::Unknown);
    assert_eq!(
        rate(Some((3_000, 10_000)), (2_999, 20_000)),
        Reading::Unknown
    );
    assert_eq!(
        rate(Some((1_000, 10_000)), (3_000, 10_000)),
        Reading::Unknown
    );
    assert_eq!(
        rate(Some((1_000, 10_000)), (3_000, 9_999)),
        Reading::Unknown
    );
    assert_eq!(rate(Some((3_000, 0)), (2_999, 1_000_000)), Reading::Unknown);
}

#[test]
fn hload_wear_reads_smartctl_json() {
    assert_eq!(DATA_UNIT_BYTES, 512_000);
    assert_eq!(
        wear(SMART),
        Some(Wear {
            read_at: 1_791_180_420,
            used_pct: 6,
            written: 83_499_999_744_000,
            spare_pct: 100,
            media_errors: 0,
        })
    );
    for key in [
        "time_t",
        "percentage_used",
        "data_units_written",
        "available_spare",
        "media_errors",
    ] {
        assert_eq!(wear(&SMART.replace(key, "gone")), None, "{key}");
    }
    assert_eq!(
        wear(&SMART.replace("\"percentage_used\":6", "\"percentage_used\":300")),
        None
    );
    assert_eq!(wear("not json"), None);
}

/// 書きの測りの表の行だけを読み、name と stat の両方が読める行を宣言の順に返す（wear は無くてよい）。
#[test]
fn hload_budget_rows_read_the_face() {
    let face = "[[device]]\nname = \"pc\"\nstat = \"/x\"\n\n[[write-budget]]\nname = \"sys\"\nstat = \"/sys/block/a/stat\"\nwear = \"/var/lib/w/a.json\" # 日に 1 度\n\n[[write-budget]] # 注\nname = \"bak\"\nstat = \"/sys/block/b/stat\"\n\n[[write-budget]]\nname = \"half\"\n\n[tick]\nname = \"x\"\nstat = \"/y\"\n";
    assert_eq!(
        budget_rows(face),
        [
            BudgetRow {
                name: "sys".into(),
                stat: "/sys/block/a/stat".into(),
                wear: Some("/var/lib/w/a.json".into())
            },
            BudgetRow {
                name: "bak".into(),
                stat: "/sys/block/b/stat".into(),
                wear: None
            },
        ]
    );
    assert!(budget_rows("name = \"x\"\nstat = \"/y\"\n").is_empty());
}

#[test]
fn hload_doc_assembles_the_texts() {
    let texts = HostTexts {
        loadavg: Some(LOADAVG.into()),
        online: Some("0-15".into()),
        psi_cpu: Some(PSI_CPU.into()),
        psi_memory: Some(PSI_MEM.into()),
        psi_io: None,
        meminfo: Some(MEMINFO.into()),
        scopes: Some(vec![scope("s", Some("26"), Some("32"))]),
        devices: vec![
            DeviceTexts {
                name: "sys".into(),
                rate: known(72_000_000),
                wear: Some(Some(SMART.into())),
            },
            DeviceTexts {
                name: "bak".into(),
                rate: Reading::Unknown,
                wear: Some(None),
            },
            DeviceTexts {
                name: "plain".into(),
                rate: Reading::Unknown,
                wear: None,
            },
        ],
    };
    let doc = host_doc(&texts, 77);
    assert_eq!(doc.at, 77);
    assert_eq!(doc.over, [Gauge::Load, Gauge::Memory]);
    assert_eq!(doc.pressure.io_full, Reading::Unknown);
    assert_eq!(doc.pressure.cpu_some, known(st(3_290, 3_080)));
    assert!(matches!(doc.memory, Reading::Known(m) if m.swap_free == 204 * 1024));
    assert!(matches!(&doc.scopes, Reading::Known(s) if s.len() == 1));
    let wears: Vec<_> = doc.devices.iter().map(|d| d.wear.clone()).collect();
    assert_eq!(
        wears,
        [
            Some(known(wear(SMART).unwrap())),
            Some(Reading::Unknown),
            None
        ]
    );
    assert_eq!(doc.devices[0].rate, known(72_000_000));
    let bare = host_doc(&HostTexts::default(), 5);
    assert_eq!(
        (bare.load, bare.memory, bare.scopes),
        (Reading::Unknown, Reading::Unknown, Reading::Unknown)
    );
    assert!(bare.over.is_empty() && bare.devices.is_empty());
}
