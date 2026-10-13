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
}

impl ProbeArgs {
    /// Parse argv excluding the executable; reject empty ranges and unsafe worker counts.
    pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut parsed = Self {
            scene: "legacy".into(),
            seeds: 0..1,
            days: 10,
            sample_every: 120,
            workers: 4,
            out: PathBuf::from(".harness/evidence/revitalize/probe"),
            check_determinism: false,
            clock_samples: Vec::new(),
        };
        let mut args = args.into_iter();
        while let Some(flag) = args.next() {
            if flag == "--check-determinism" {
                parsed.check_determinism = true;
                continue;
            }
            if flag == "--help" {
                return Err("usage: worldsim_probe --scene legacy|game|game_v9|game_v10|game_v11|game_v12|game_v13|game_v14|game_v15|game_v16_neutral|game_v16|game_v17_nofission|game_v17 --seeds a..b --days N --sample-every N --workers 1..8 --out DIR [--check-determinism] [--clock-samples HH:MM,HH:MM]".into());
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
        if !matches!(parsed.scene.as_str(), "legacy" | "game" | "game_v9" | "game_v10" | "game_v11" | "game_v12" | "game_v13" | "game_v14" | "game_v15" | "game_v16_neutral" | "game_v16" | "game_v17_nofission" | "game_v17") {
            return Err("scene must be legacy, game, game_v9, game_v10, game_v11, game_v12, game_v13, game_v14, game_v15, game_v16_neutral, game_v16, game_v17_nofission or game_v17".into());
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
        ProbeArgs::parse(args.iter().map(|s| (*s).into()))
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
        assert!(parse(&["--days", "120", "--scene", "game_v9"]).is_ok());
        assert!(parse(&["--days", "150", "--scene", "game_v10"]).is_ok());
        assert!(parse(&["--days", "150", "--scene", "game_v11"]).is_ok());
        assert!(parse(&["--days", "150", "--scene", "game_v12"]).is_ok());
        assert!(parse(&["--days", "150", "--scene", "game_v13"]).is_ok());
        assert!(parse(&["--days", "150", "--scene", "game_v14"]).is_ok());
        assert!(parse(&["--days", "150", "--scene", "game_v15"]).is_ok());
        assert!(parse(&["--days", "150", "--scene", "game_v16_neutral"]).is_ok());
        assert!(parse(&["--days", "150", "--scene", "game_v16"]).is_ok());
        assert!(parse(&["--days", "150", "--scene", "game_v17_nofission"]).is_ok());
        assert!(parse(&["--days", "150", "--scene", "game_v17"]).is_ok());
        assert!(parse(&["--scene", "game_v18"]).is_err());
        assert!(parse(&["--days", "151"]).is_err());
    }
    #[test]
    fn validates_half_open_range_and_hard_worker_limit() {
        let args = parse(&["--seeds", "2..5", "--workers", "8"]).expect("valid");
        assert_eq!(args.seeds.collect::<Vec<_>>(), [2, 3, 4]);
        assert!(parse(&["--scene", "game"]).is_ok());
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
