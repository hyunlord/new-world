//! Dependency-free command-line boundary validation.
use std::path::PathBuf;

/// Validated arguments for the headless measurement runner.
#[derive(Clone, Debug)]
pub struct ProbeArgs {
    /// Selected simulation fixture.
    pub(super) scene: String,
    /// Half-open seed range.
    pub(super) seeds: std::ops::Range<u64>,
    /// Positive simulation duration in days.
    pub(super) days: u64,
    /// Positive raw-sample interval in ticks.
    pub(super) sample_every: u64,
    /// Validated pool size, at most eight.
    pub(super) workers: usize,
    /// Raw sample and summary directory.
    pub(super) out: PathBuf,
    /// Sorted unique display minutes requested each day.
    pub(super) clock_samples: Vec<u32>,
    /// Whether to compare two independent engines per seed.
    pub(super) check_determinism: bool,
    /// Emit read-only per-world gate observation sidecars.
    pub(super) gate_observations: bool,
}

impl ProbeArgs {
    /// Pure shared opt-in parser; explicit CLI opt-in never hides an invalid environment value.
    pub(crate) fn gate_observations_enabled(
        cli_enabled: bool,
        setting: Option<&std::ffi::OsStr>,
    ) -> Result<bool, String> {
        match setting {
            None => Ok(cli_enabled),
            Some(value) if value == "1" => Ok(true),
            Some(_) => Err("WORLDSIM_GATE_OBSERVER must be absent or 1".into()),
        }
    }

    /// Parse argv excluding the executable; reject empty ranges and unsafe worker counts.
    pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let setting = std::env::var_os("WORLDSIM_GATE_OBSERVER");
        Self::parse_with_gate_setting(args, setting.as_deref())
    }

    fn parse_with_gate_setting(
        args: impl IntoIterator<Item = String>,
        setting: Option<&std::ffi::OsStr>,
    ) -> Result<Self, String> {
        let environment_enabled = super::ProbeArgs::gate_observations_enabled(false, setting)?;
        let mut parsed = Self {
            scene: "baseline".into(),
            seeds: 0..1,
            days: 10,
            sample_every: 120,
            workers: 4,
            out: PathBuf::from(".harness/evidence/revitalize/probe"),
            check_determinism: false,
            gate_observations: false,
            clock_samples: Vec::new(),
        };
        let mut args = args.into_iter();
        while let Some(flag) = args.next() {
            if flag == "--gate-observations" {
                if parsed.gate_observations {
                    return Err("duplicate --gate-observations".into());
                }
                parsed.gate_observations = true;
                continue;
            }
            if flag == "--check-determinism" {
                parsed.check_determinism = true;
                continue;
            }
            if flag == "--help" {
                return Err("usage: worldsim_probe --scene baseline|game_v26 --seeds a..b --days N --sample-every N --workers 1..8 --out DIR [--check-determinism] [--gate-observations] [--clock-samples HH:MM,HH:MM]".into());
            }
            let value = args
                .next()
                .ok_or_else(|| format!("missing value for {flag}"))?;
            match flag.as_str() {
                "--scene" => parsed.scene = value,
                "--clock-samples" => parsed.clock_samples = parse_clock_samples(&value)?,
                "--seeds" => {
                    let (a, b) = value
                        .split_once("..")
                        .ok_or("seeds must be a..b (half-open)")?;
                    parsed.seeds = a.parse().map_err(|_| "invalid seed start")?
                        ..b.parse().map_err(|_| "invalid seed end")?;
                }
                "--days" => parsed.days = value.parse().map_err(|_| "invalid days")?,
                "--sample-every" => {
                    parsed.sample_every = value.parse().map_err(|_| "invalid sample interval")?
                }
                "--workers" => parsed.workers = value.parse().map_err(|_| "invalid workers")?,
                "--out" => parsed.out = value.into(),
                _ => return Err(format!("unknown argument {flag}")),
            }
        }
        if !matches!(parsed.scene.as_str(), "baseline" | "game_v26") {
            return Err("scene must be baseline (game_v26 is accepted as its alias)".into());
        }
        if parsed.seeds.is_empty() {
            return Err("seeds must be a nonempty half-open range".into());
        }
        if !(1..=150).contains(&parsed.days) || parsed.sample_every == 0 {
            return Err("days must be 1..=150 and sample-every must be positive".into());
        }
        if !(1..=8).contains(&parsed.workers) {
            return Err("workers must be between 1 and 8".into());
        }
        parsed.gate_observations |= environment_enabled;
        Ok(parsed)
    }
}

fn parse_clock_samples(value: &str) -> Result<Vec<u32>, String> {
    let mut minutes = std::collections::BTreeSet::new();
    for time in value.split(',') {
        let (hour, minute) = time.split_once(':').ok_or("clock sample must be HH:MM")?;
        if hour.len() != 2
            || minute.len() != 2
            || !hour
                .bytes()
                .chain(minute.bytes())
                .all(|b| b.is_ascii_digit())
        {
            return Err("clock sample must be HH:MM".into());
        }
        let hour: u32 = hour.parse().map_err(|_| "invalid hour")?;
        let minute: u32 = minute.parse().map_err(|_| "invalid minute")?;
        if hour >= 24 || minute >= 60 {
            return Err("clock sample outside 00:00..23:59".into());
        }
        minutes.insert(hour * 60 + minute);
    }
    Ok(minutes.into_iter().collect())
}

/// Convert one-based display day and minute to elapsed tick; omit pre-start times.
pub(super) fn clock_tick(day: u64, minute: u32, ticks_per_day: u64, offset: u64) -> Option<u64> {
    day.checked_sub(1)?
        .checked_mul(ticks_per_day)?
        .checked_add(u64::from(minute))?
        .checked_sub(offset)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn parse(args: &[&str]) -> Result<ProbeArgs, String> {
        ProbeArgs::parse_with_gate_setting(args.iter().map(|s| (*s).into()), None)
    }
    #[test]
    fn clock_minutes_validate_sort_deduplicate_and_convert_with_offset() {
        assert_eq!(
            super::parse_clock_samples("19:30,02:00,12:00,02:00").expect("clock"),
            vec![120, 720, 1170]
        );
        for bad in ["", "24:00", "23:60", "2:00", "aa:00", "02:00,"] {
            assert!(super::parse_clock_samples(bad).is_err());
        }
        assert_eq!(super::clock_tick(1, 120, 1440, 480), None);
        assert_eq!(super::clock_tick(2, 120, 1440, 480), Some(1080));
        assert_eq!(super::clock_tick(1, 720, 1440, 480), Some(240));
        // V8-0: only the baseline scene (and its alias game_v26) is accepted;
        // the removed per-version scenes are rejected, and the messages name
        // the baseline.
        assert!(parse(&["--days", "150", "--scene", "baseline"]).is_ok());
        assert!(parse(&["--scene", "game_v26"]).is_ok());
        assert_eq!(parse(&[]).expect("default").scene, "baseline");
        for old in [
            "game",
            "legacy",
            "game_v9",
            "game_v19",
            "game_v20_inert",
            "game_v25",
            "game_v27",
        ] {
            assert!(parse(&["--scene", old]).is_err(), "{old}");
        }
        let usage = parse(&["--help"]).expect_err("usage");
        assert!(usage.contains("--scene baseline|game_v26"), "{usage}");
        let unknown = parse(&["--scene", "game_v99"]).expect_err("unknown scene");
        assert!(unknown.contains("baseline"), "{unknown}");
        assert!(parse(&["--days", "151"]).is_err());
    }
    #[test]
    fn observations_are_opt_in_and_duplicates_rejected() {
        assert!(!parse(&[]).expect("default").gate_observations);
        assert!(
            parse(&["--gate-observations"])
                .expect("observer")
                .gate_observations
        );
        assert!(parse(&["--gate-observations", "--gate-observations"]).is_err());
    }
    #[test]
    fn validates_half_open_range_and_hard_worker_limit() {
        let args = parse(&["--seeds", "2..5", "--workers", "8"]).expect("valid");
        assert_eq!(args.seeds.collect::<Vec<_>>(), [2, 3, 4]);
        assert!(parse(&["--scene", "baseline"]).is_ok());
        for args in [
            &["--workers", "9"][..],
            &["--workers", "0"],
            &["--seeds", "4..4"],
            &["--sample-every", "0"],
            &["--scene", "unknown"],
        ] {
            assert!(parse(args).is_err());
        }
    }
}

#[cfg(test)]
mod gate_environment_tests {
    use super::*;
    use std::ffi::OsStr;

    #[test]
    fn environment_and_flag_are_independent_opt_ins() {
        for setting in [None, Some(OsStr::new("1"))] {
            for cli in [false, true] {
                let args = if cli {
                    vec!["--gate-observations".to_string()]
                } else {
                    Vec::new()
                };
                let parsed =
                    ProbeArgs::parse_with_gate_setting(args, setting).expect("valid setting");
                assert_eq!(parsed.gate_observations, cli || setting.is_some());
            }
            let duplicate = [
                "--gate-observations".to_string(),
                "--gate-observations".to_string(),
            ];
            assert!(ProbeArgs::parse_with_gate_setting(duplicate, setting)
                .expect_err("duplicate flag")
                .contains("duplicate"));
        }
        for value in ["", "0", "true", " 1", "1 "] {
            for cli in [false, true] {
                let args = if cli {
                    vec!["--gate-observations".to_string()]
                } else {
                    Vec::new()
                };
                assert_eq!(
                    ProbeArgs::parse_with_gate_setting(args, Some(OsStr::new(value)))
                        .expect_err("invalid environment"),
                    "WORLDSIM_GATE_OBSERVER must be absent or 1"
                );
            }
        }
    }
    #[cfg(unix)]
    #[test]
    fn non_unicode_environment_is_invalid_even_with_cli_opt_in() {
        use std::os::unix::ffi::OsStringExt;
        let setting = std::ffi::OsString::from_vec(vec![0xff]);
        assert_eq!(
            ProbeArgs::gate_observations_enabled(true, Some(&setting))
                .expect_err("invalid setting"),
            "WORLDSIM_GATE_OBSERVER must be absent or 1"
        );
    }
}
