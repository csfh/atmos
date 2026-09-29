use std::env;
use std::io::{self, Read};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    let mut stdin = String::new();
    if wants_request_stdin(&args) {
        let _ = io::stdin().read_to_string(&mut stdin);
    }
    let code = ratmos::run(&args, &stdin, &mut io::stdout(), &mut io::stderr());
    ExitCode::from(u8::try_from(code).unwrap_or(1))
}

fn wants_request_stdin(args: &[String]) -> bool {
    for arg in args.iter().skip(1) {
        if arg == "--" {
            break;
        }
        if arg == "request" {
            return true;
        }
    }
    false
}
