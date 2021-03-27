use std::process::Command;

use regex::Regex;

mod enum_;
mod parser;

use crate::enum_::{ArgsNumType, OptionToken};
use parser::parse_line;

const MAIN: &str = "catkin";

const VERBS: [&'static str; 9] = [
    "build", "clean", "config", "create", "env", "init", "list", "locate", "profile",
];

pub fn generate_each_option(
    main_command: &str,
    sub_command: Option<&str>,
    result: Vec<(OptionToken, ArgsNumType)>,
    description: &str,
) {
    let mut short = None;
    let mut longs = Vec::new();
    let mut olds = Vec::new();

    for (item, _) in result {
        match item {
            OptionToken::ShortOption(c) => {
                assert!(short.is_none());
                short = Some(c);
            }
            OptionToken::LongOption(long) => longs.push(long),
            OptionToken::OldOption(old) => olds.push(old),
        }
    }

    let sub_command_option = if let Some(sub_command) = sub_command {
        std::format!("-n '__fish_seen_subcommand_from {}'", sub_command)
    } else {
        "-n __fish_use_subcommand".to_string()
    };

    let short_option = short.map_or("".to_string(), |c| std::format!("-s {}", c));

    for long in longs {
        println!(
            r#"complete -c {} {} {} -l {} -d "{}""#,
            &main_command, &sub_command_option, &short_option, &long, &description
        );
    }
    for old in olds {
        println!(
            r#"complete -c {} {} {} -o {} -d "{}""#,
            &main_command, &sub_command_option, &short_option, &old, &description
        );
    }
}

fn generate_each_subcommand(verb: Option<&str>) {
    let args = if let Some(verb) = verb {
        println!("");
        println!("# {}", &verb);
        println!("complete -c {} -n __fish_use_subcommand -a {}", MAIN, &verb);
        vec![verb, "--help"]
    } else {
        vec!["--help"]
    };
    let output = Command::new(MAIN)
        .args(args)
        .output()
        .expect("failed to execute process");
    let string = std::str::from_utf8(&output.stdout).unwrap();

    let re = Regex::new(r"\n {8,}").unwrap();
    let string = re.replace_all(string, " ");

    for line in string.lines() {
        let result = parse_line(line);
        if result.is_ok() {
            let result = result.unwrap();
            generate_each_option(MAIN, verb, result.1, result.0);
        }
    }
}

fn main() {
    generate_each_subcommand(None);
    for verb in VERBS.iter() {
        generate_each_subcommand(Some(verb));
    }
}
