mod cmd;
use std::process::exit;

use anyhow::{anyhow, Result};
use console::{style, Style};
use shellfirm::{CmdExit, Config};

const DEFAULT_ERR_EXIT_CODE: i32 = 1;

fn main() {
    let app = cmd::default::command()
        .subcommand(cmd::command::command())
        .subcommand(cmd::config::command());

    let matches = app.clone().get_matches();

    let env = env_logger::Env::default().filter_or(
        "LOG",
        matches.value_of("log").unwrap_or(log::Level::Info.as_str()),
    );
    env_logger::init_from_env(env);

    // load configuration
    let config = match Config::new(None) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Loading config error: {}", err);
            exit(1)
        }
    };

    if let Some((command_name, subcommand_matches)) = matches.subcommand() {
        if command_name == "config" && subcommand_matches.subcommand_name() == Some("reset") {
            let c = cmd::config::run_reset(&config, None);
            shellfirm_exit(Ok(c));
        }
    };

    let settings = match config.get_settings_from_file() {
        Ok(c) => c,
        Err(e) => {
            eprintln!(
                "Could not load setting from file. Try resolving by running `{}`\nError: {}",
                style("shellfirm config reset").bold().italic().underlined(),
                e
            );
            exit(1)
        }
    };

    let checks = match settings.get_active_checks() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Could not load checks. err: Error: {}", e);
            exit(1)
        }
    };

    let res = matches.subcommand().map_or_else(
        || Err(anyhow!("command not found")),
        |tup| match tup {
            ("pre-command", subcommand_matches) => {
                cmd::command::run(subcommand_matches, &settings, &checks)
            }
            ("config", subcommand_matches) => {
                cmd::config::run(subcommand_matches, &config, &settings)
            }
            _ => unreachable!(),
        },
    );

    shellfirm_exit(res);
}

fn shellfirm_exit(res: Result<CmdExit>) {
    let exit_with = match res {
        Ok(cmd) => {
            if let Some(message) = cmd.message {
                let style = if exitcode::is_success(cmd.code) {
                    Style::new().green()
                } else {
                    Style::new().red()
                };
                eprintln!("{}", style.apply_to(message));
            }
            cmd.code
        }
        Err(e) => {
            log::debug!("{:?}", e);
            DEFAULT_ERR_EXIT_CODE
        }
    };
    exit(exit_with);
}
