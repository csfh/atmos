use std::env;
use std::io::{self, Read};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    // Linked as `gum`, this binary answers confirm prompts (see tasks::user::gum_job).
    if args
        .first()
        .and_then(|a| std::path::Path::new(a).file_name())
        .is_some_and(|n| n == "gum")
    {
        atmosd::tasks::user::gum_stub(&args[1..]);
    }
    if wants(&args, "serve") {
        return ExitCode::from(u8::try_from(atmosd::run_serve(&args)).unwrap_or(1));
    }
    let mut stdin = String::new();
    if wants(&args, "request") {
        let _ = io::stdin().read_to_string(&mut stdin);
    }
    let code = atmosd::run(&args, &stdin, &mut io::stdout(), &mut io::stderr());
    ExitCode::from(u8::try_from(code).unwrap_or(1))
}

fn wants(args: &[String], command: &str) -> bool {
    for arg in args.iter().skip(1) {
        if arg == "--" {
            break;
        }
        if arg == command {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argv(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn wants_finds_a_command_after_the_program_name() {
        assert!(wants(&argv(&["atmosd", "request"]), "request"));
        assert!(wants(
            &argv(&["atmosd", "--root", "/x", "request"]),
            "request"
        ));
    }

    #[test]
    fn wants_ignores_the_program_name_and_other_commands() {
        assert!(!wants(&argv(&["request"]), "request"));
        assert!(!wants(&argv(&["atmosd", "snapshot"]), "request"));
        assert!(!wants(&argv(&["atmosd"]), "request"));
        assert!(!wants(&[], "request"));
    }

    #[test]
    fn wants_stops_at_the_double_dash() {
        assert!(!wants(&argv(&["atmosd", "--", "request"]), "request"));
        assert!(wants(&argv(&["atmosd", "request", "--", "x"]), "request"));
    }
}
