//! Retired live shadow entrypoint; accepted evidence uses the frozen external executable.

/// Current baseline code cannot reproduce the archived old colony brain after isolation.
pub const RETIREMENT_NOTICE: &str = "mind-shadow live runner retired: use the verified frozen external binary and source/build manifest (tools/v8/core_shadow_freeze.py); rebuilding this checkout cannot reproduce the old colony brain";

/// Refuse before parsing, content bootstrap, filesystem writes or world construction.
pub fn run(_raw: &[String]) -> Result<(), String> {
    Err(RETIREMENT_NOTICE.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retired_runner_refuses_even_empty_or_malformed_requests() {
        for args in [vec![], vec!["--unknown".to_owned()]] {
            assert_eq!(run(&args), Err(RETIREMENT_NOTICE.to_owned()));
        }
    }
}
