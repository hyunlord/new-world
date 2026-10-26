//! Headless WorldSim probe command.
fn main() -> std::process::ExitCode {
    let args: Vec<_> = std::env::args().skip(1).collect();
    #[cfg(feature = "mind_shadow")]
    if args.first().is_some_and(|arg| arg == "mind-shadow") {
        if args.iter().any(|arg| arg == "--help" || arg == "-h") {
            println!("usage: worldsim_probe mind-shadow --seeds a..b --days N --out NEW_EXTERNAL_DIR [--workers 1..8] [--steward on|off] [--defence on|off] [--story on|off|no-beasts] [--neighbours on|off] [--observe on|off]");
            return std::process::ExitCode::SUCCESS;
        }
        return match sim_test::content_boot::from_environment()
            .and_then(|()| sim_test::mind_shadow::run(&args[1..]))
        {
            Ok(()) => std::process::ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("{error}");
                std::process::ExitCode::FAILURE
            }
        };
    }
    if args.first().is_some_and(|arg| arg == "story-yield") {
        if args.iter().any(|arg| arg == "--help" || arg == "-h") {
            println!("usage: worldsim_probe story-yield --seeds a..b --days N --out NEW_EXTERNAL_DIR [--workers 1..8] [--steward on|off] [--defence on|off] [--story on|off|no-beasts] [--neighbours on|off]");
            return std::process::ExitCode::SUCCESS;
        }
        return match sim_test::content_boot::from_environment()
            .and_then(|()| sim_test::story_yield::run(&args[1..]))
        {
            Ok(()) => std::process::ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("{error}");
                std::process::ExitCode::FAILURE
            }
        };
    }
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("usage: worldsim_probe --scene colony --seeds a..b --days N --workers 1..8 --out DIR [--commands FILE] [--commands-out DIR] [--check-determinism]");
        println!("M-UX2: --steward on --defence on|off --days 48 --mux2-observe --mux2-policy-sha256 HEX (no player commands; external receipt verification required)");
        println!("colony observation: --b4-observe --b4-manifest FILE --b4-manifest-sha256 HEX (all three required; external SHA256 verified by runner/scorer)");
        println!("usage: worldsim_probe --scene colony --seeds S..S+1 --ascii --tick T [--rect x,y,w,h] [--commands FILE] [--ascii-legend] [--out FILE] [--people-json FILE]");
        println!("usage: worldsim_probe --scene baseline --seeds a..b --days N --sample-every N --workers 1..8 --out DIR [--check-determinism] [--clock-samples HH:MM,...] [--agent-trace] [--chronicle-out DIR]");
        return std::process::ExitCode::SUCCESS;
    }
    if let Err(error) = sim_test::content_boot::from_environment() {
        eprintln!("{error}");
        return std::process::ExitCode::FAILURE;
    }
    let colony = args
        .windows(2)
        .any(|w| w[0] == "--scene" && w[1] == "colony");
    // M1.5: the text renderer (goal 8.1) is a colony run that prints a grid.
    if colony && args.iter().any(|arg| arg == "--ascii") {
        return match sim_test::probe::ascii::AsciiArgs::parse(args)
            .and_then(|a| sim_test::probe::ascii::run(&a))
        {
            Ok(()) => std::process::ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("{error}");
                std::process::ExitCode::FAILURE
            }
        };
    }
    // M1: the colony scene has its own runner (commands, U-bars).
    if colony {
        return match sim_test::probe::colony::ColonyArgs::parse(args)
            .and_then(|a| sim_test::probe::colony::run(&a))
        {
            Ok(()) => std::process::ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("{error}");
                std::process::ExitCode::FAILURE
            }
        };
    }
    // Pass 16: `--agent-trace` is split off before `ProbeArgs::parse`.
    let (args, options) = sim_test::probe::ProbeOptions::split(args);
    // Pass 19: `--chronicle-out DIR` likewise.
    let parsed = sim_test::probe::chronicle_out::split(args).and_then(|(args, chronicle_out)| {
        sim_test::probe::ProbeArgs::parse(args).map(|args| (args, chronicle_out))
    });
    match parsed.and_then(|(args, chronicle_out)| {
        sim_test::probe::run_with_chronicle(&args, &options, chronicle_out.as_deref())
    }) {
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
