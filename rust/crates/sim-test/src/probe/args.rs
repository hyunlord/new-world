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
        };
        let mut args = args.into_iter();
        while let Some(flag) = args.next() {
            if flag == "--check-determinism" {
                parsed.check_determinism = true;
                continue;
            }
            if flag == "--help" {
                return Err("usage: worldsim_probe --scene legacy|game --seeds a..b --days N --sample-every N --workers 1..8 --out DIR [--check-determinism]".into());
            }
            let value = args
                .next()
                .ok_or_else(|| format!("missing value for {flag}"))?;
            match flag.as_str() {
                "--scene" => parsed.scene = value,
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
        if parsed.scene == "game" {
            return Err("game scene is not implemented until Part 2".into());
        }
        if parsed.scene != "legacy" {
            return Err("scene must be legacy or game".into());
        }
        if parsed.seeds.is_empty() {
            return Err("seeds must be a nonempty half-open range".into());
        }
        if parsed.days == 0 || parsed.sample_every == 0 {
            return Err("days and sample-every must be positive".into());
        }
        if !(1..=8).contains(&parsed.workers) {
            return Err("workers must be between 1 and 8".into());
        }
        Ok(parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn parse(args: &[&str]) -> Result<ProbeArgs, String> {
        ProbeArgs::parse(args.iter().map(|s| (*s).into()))
    }
    #[test]
    fn validates_half_open_range_and_hard_worker_limit() {
        let args = parse(&["--seeds", "2..5", "--workers", "8"]).expect("valid");
        assert_eq!(args.seeds.collect::<Vec<_>>(), [2, 3, 4]);
        for args in [
            &["--workers", "9"][..],
            &["--workers", "0"],
            &["--seeds", "4..4"],
            &["--sample-every", "0"],
            &["--scene", "game"],
        ] {
            assert!(parse(args).is_err());
        }
    }
}
