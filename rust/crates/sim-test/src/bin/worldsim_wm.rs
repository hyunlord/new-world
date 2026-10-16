//! Headless WorldSim world recorder (Pass 7): one directory of NPY arrays per
//! episode, byte-identical for the same arguments whatever the worker count.
fn main() -> std::process::ExitCode {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("{}", sim_test::worldmodel::export::USAGE);
        return std::process::ExitCode::SUCCESS;
    }
    let started = std::time::Instant::now();
    match sim_test::worldmodel::export::WmArgs::parse(args).and_then(|args| sim_test::worldmodel::export::run(&args)) {
        Ok(done) => {
            eprintln!("DONE episodes={} wall_seconds={:.1}", done.len(), started.elapsed().as_secs_f64());
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
    fn cli_rejects_unknown_flags() {
        assert!(sim_test::worldmodel::export::WmArgs::parse(["--unknown".into(), "value".into()]).is_err());
    }
}
