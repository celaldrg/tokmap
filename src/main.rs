use std::path::PathBuf;
use std::process::ExitCode;

use tokmap::{render_json, render_text, scan, thousands, ReportOptions};

const HELP: &str = "tokmap: see where your repo's tokens go before an AI agent reads it

USAGE:
    tokmap [PATH] [OPTIONS]

ARGS:
    PATH                 Folder or file to measure (default: current folder)

OPTIONS:
    -d, --depth <N>      Group by the first N path levels (default: 1)
    -n, --top <N>        Show the N heaviest entries (default: 15)
    -b, --budget <N>     Compare with a token budget; exit 1 when over it
        --json           Print JSON instead of the table
        --no-git         Walk the folder instead of asking git for the file list
    -h, --help           Show this help
    -V, --version        Show the version

Token counts are a fast heuristic estimate, not an exact tokenizer.";

fn parse_num(flag: &str, v: Option<String>) -> Result<u64, String> {
    let v = v.ok_or_else(|| format!("{flag} needs a value"))?;
    let cleaned: String = v.chars().filter(|c| *c != '_' && *c != ',').collect();
    let (digits, mult) = match cleaned.chars().last() {
        Some('k') | Some('K') => (&cleaned[..cleaned.len() - 1], 1_000),
        Some('m') | Some('M') => (&cleaned[..cleaned.len() - 1], 1_000_000),
        _ => (cleaned.as_str(), 1),
    };
    digits
        .parse::<u64>()
        .map(|n| n * mult)
        .map_err(|_| format!("{flag}: '{v}' is not a number"))
}

fn run() -> Result<ExitCode, String> {
    let mut args = std::env::args().skip(1);
    let mut path: Option<PathBuf> = None;
    let mut depth = 1u64;
    let mut top = 15u64;
    let mut budget = None;
    let mut json = false;
    let mut use_git = true;
    while let Some(a) = args.next() {
        match a.as_str() {
            "-h" | "--help" => {
                println!("{HELP}");
                return Ok(ExitCode::SUCCESS);
            }
            "-V" | "--version" => {
                println!("tokmap {}", env!("CARGO_PKG_VERSION"));
                return Ok(ExitCode::SUCCESS);
            }
            "-d" | "--depth" => depth = parse_num(&a, args.next())?,
            "-n" | "--top" => top = parse_num(&a, args.next())?,
            "-b" | "--budget" => budget = Some(parse_num(&a, args.next())?),
            "--json" => json = true,
            "--no-git" => use_git = false,
            s if s.starts_with('-') => return Err(format!("unknown option {s} (try --help)")),
            s => {
                if path.is_some() {
                    return Err("only one PATH is allowed".into());
                }
                path = Some(PathBuf::from(s));
            }
        }
    }
    let root = path.unwrap_or_else(|| PathBuf::from("."));
    if !root.exists() {
        return Err(format!("{} does not exist", root.display()));
    }
    let result = scan(&root, use_git).map_err(|e| format!("{}: {e}", root.display()))?;
    let opts = ReportOptions {
        label: root.display().to_string(),
        depth: depth as usize,
        top: top.max(1) as usize,
        budget,
    };
    if json {
        print!("{}", render_json(&result, &opts));
    } else {
        print!("{}", render_text(&result, &opts));
    }
    if let Some(b) = budget {
        let total = result.total_tokens();
        if total > b {
            if !json {
                eprintln!("\ntokmap: over budget by {} tokens", thousands(total - b));
            }
            return Ok(ExitCode::from(1));
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(e) => {
            eprintln!("tokmap: {e}");
            ExitCode::from(2)
        }
    }
}
