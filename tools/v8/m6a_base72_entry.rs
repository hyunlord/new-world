//! Distinct M6a BASE observer adapter; links the unchanged published BASE library.
fn main() -> std::process::ExitCode {
    let result = sim_test::content_boot::from_environment().and_then(|()| {
        let parsed = parse_colony_base72(std::env::args().skip(1).collect())?;
        if !parsed.out.is_absolute() || parsed.out.exists() {
            return Err("BASE72 output must be a new absolute directory".into());
        }
        sim_test::probe::colony::run(&parsed)
    });
    match result {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            std::process::ExitCode::FAILURE
        }
    }
}

/// Extend only the explicitly requested M6a observer horizon, after BASE validation.
fn parse_colony_base72(
    mut args: Vec<String>,
) -> Result<sim_test::probe::colony::ColonyArgs, String> {
    let qualification = args
        .first()
        .is_some_and(|arg| arg == "--base72-qualification-48");
    if qualification {
        args.remove(0);
    }
    let days = if qualification { "48" } else { "72" };
    if !args
        .windows(2)
        .any(|pair| pair[0] == "--days" && pair[1] == days)
        || !args.iter().any(|arg| arg == "--mux2-observe")
    {
        return Err("BASE72 requires explicit72 observer or explicit48 qualification".into());
    }
    // A narrow observer-only grammar prevents conflicting duplicate options or
    // save/player/B4/ascii side channels from entering the extended protocol.
    let mut seen = std::collections::BTreeSet::new();
    let mut cursor = 0;
    let mut day_value = None;
    while cursor < args.len() {
        let flag = args[cursor].as_str();
        if !seen.insert(flag) {
            return Err(format!("BASE72 duplicate option: {flag}"));
        }
        match flag {
            "--mux2-observe" | "--check-determinism" => cursor += 1,
            "--scene"
            | "--seeds"
            | "--days"
            | "--workers"
            | "--out"
            | "--steward"
            | "--defence"
            | "--story"
            | "--neighbours"
            | "--mux2-policy-sha256" => {
                let value = args
                    .get(cursor + 1)
                    .ok_or_else(|| format!("BASE72 missing value for {flag}"))?;
                if value.starts_with("--") {
                    return Err(format!("BASE72 missing value for {flag}"));
                }
                if flag == "--days" {
                    day_value = Some(cursor + 1);
                }
                cursor += 2;
            }
            _ => return Err(format!("BASE72 unsupported option: {flag}")),
        }
    }
    for required in [
        "--scene",
        "--seeds",
        "--days",
        "--workers",
        "--out",
        "--steward",
        "--defence",
        "--story",
        "--neighbours",
        "--mux2-policy-sha256",
    ] {
        if !seen.contains(required) {
            return Err(format!("BASE72 requires explicit {required}"));
        }
    }
    let position = day_value.ok_or("BASE72 missing --days")?;
    let mut validation_args = args.clone();
    validation_args[position] = "48".into();
    let mut parsed = sim_test::probe::colony::ColonyArgs::parse(validation_args)?;
    // Original parser has now checked policy hash, explicit defence, steward,
    // story/neighbours on, no commands, seed/worker bounds and all option values.
    if parsed.workers != 1
        || parsed.seeds.end - parsed.seeds.start != 1
        || parsed.defence != Some(true)
        || parsed.mux2_policy.as_deref()
            != Some("7a6c90ac89db9cfb08acac6a8e14c91d1a1690d9fa40c3798c52323ead43eae8")
    {
        return Err("BASE72 requires one seed/worker, defence on and authentic BASE policy".into());
    }
    if qualification && parsed.seeds.start != 0 {
        return Err("BASE72 qualification requires seed0".into());
    }
    parsed.days = if qualification { 48 } else { 72 };
    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::parse_colony_base72;
    fn args() -> Vec<String> {
        [
            "--scene",
            "colony",
            "--seeds",
            "0..1",
            "--days",
            "72",
            "--workers",
            "1",
            "--out",
            "/unused",
            "--steward",
            "on",
            "--defence",
            "on",
            "--story",
            "on",
            "--neighbours",
            "on",
            "--mux2-observe",
            "--mux2-policy-sha256",
            "7a6c90ac89db9cfb08acac6a8e14c91d1a1690d9fa40c3798c52323ead43eae8",
        ]
        .into_iter()
        .map(String::from)
        .collect()
    }
    #[test]
    fn original48_and_extended72_share_parser() {
        assert!(sim_test::probe::colony::ColonyArgs::parse(args()).is_err());
        let actual = parse_colony_base72(args()).expect("72");
        assert_eq!(actual.days, 72);
        assert!(actual.commands.is_none() && actual.save_at.is_none() && actual.b4.is_none());
        let mut a = args();
        a[5] = "48".into();
        assert!(parse_colony_base72(a.clone()).is_err());
        a.insert(0, "--base72-qualification-48".into());
        assert_eq!(parse_colony_base72(a).expect("qualification").days, 48);
    }
    #[test]
    fn reject_changed_inputs() {
        for (index, value) in [
            (3, "0..2"),
            (5, "47"),
            (7, "2"),
            (11, "off"),
            (13, "off"),
            (15, "off"),
            (17, "off"),
            (20, "bad"),
        ] {
            let mut a = args();
            a[index] = value.into();
            assert!(parse_colony_base72(a).is_err(), "{index}");
        }
    }
    #[test]
    fn reject_duplicates_and_side_channels() {
        for flag in [
            "--days",
            "--commands",
            "--save-at",
            "--save-out",
            "--ascii",
            "--b4-observe",
            "--chronicle-out",
            "--unknown",
        ] {
            let mut a = args();
            a.extend([flag.into(), "72".into()]);
            assert!(parse_colony_base72(a).is_err(), "{flag}");
        }
    }
}
