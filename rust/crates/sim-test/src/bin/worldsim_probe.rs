//! Headless WorldSim probe command.
fn main() -> std::process::ExitCode {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("usage: worldsim_probe --scene legacy|game --seeds a..b --days N --sample-every N --workers 1..8 --out DIR [--check-determinism] [--clock-samples HH:MM,...] [--agent-trace]");
        return std::process::ExitCode::SUCCESS;
    }
    // Pass 16: `--agent-trace` is split off before `ProbeArgs::parse`.
    let (args, options) = sim_test::probe::ProbeOptions::split(args);
    match sim_test::probe::ProbeArgs::parse(args).and_then(|args| sim_test::probe::run_with(&args, &options)) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            std::process::ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn cli_rejects_unknown_flags() {
        assert!(sim_test::probe::ProbeArgs::parse(["--unknown".into(), "value".into()]).is_err());
    }
}
