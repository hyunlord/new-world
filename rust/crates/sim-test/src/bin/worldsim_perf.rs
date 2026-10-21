//! Revitalize Pass 15 benchmark: per-system tick timings of the natural game
//! world and of the stress world (`sim_test::perf`).
fn main() -> std::process::ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() || args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("{}", sim_test::perf::USAGE);
        return std::process::ExitCode::SUCCESS;
    }
    // M1.5: the standing benchmark has its own parser (`sim_test::bench`).
    if matches!(args[0].as_str(), "bench" | "bench-one") {
        return match sim_test::bench::main(&args) {
            Ok(()) => std::process::ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("{error}");
                std::process::ExitCode::FAILURE
            }
        };
    }
    match sim_test::perf::PerfArgs::parse(&args).and_then(|args| args.execute()) {
        Ok(written) => {
            eprintln!("worldsim_perf: {written} run(s) written");
            std::process::ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn cli_rejects_unknown_subcommands() {
        assert!(
            sim_test::perf::PerfArgs::parse(&["bogus".into(), "--out".into(), "x".into()]).is_err()
        );
    }
}
