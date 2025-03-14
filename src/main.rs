mod data;
mod neocities;
mod state;
mod sync;

use std::{env, path::PathBuf, process::exit};

use clap::{Parser, Subcommand};
use colored::*;
use data::Data;
use neocities::Neocities;
use sync::{sync, SyncError};

#[derive(Parser, Debug)]
#[command(
    version,
    about = r#"
|\---/|
| x_x |   neocities-sync
 \_-_/

Sync files to neocities while doing the least amount of API requests."#
)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Login to neocities.
    Login,
    /// Logout from neocities.
    Logout {
        #[arg(short, long)]
        username: Option<String>,
    },
    /// Sync a directory to neocities.
    Sync {
        #[arg(short, long)]
        username: Option<String>,
        /// The directory to sync.
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Path to the state file. Used to keep track of the last sync.
        #[arg(short, long)]
        state: Option<PathBuf>,
        /// Use this if you are NOT a supporter.
        #[arg(short, long)]
        ignore_disallowed_file_types: bool,
    },
}

fn get_username(username: Option<String>, data: &Data) -> (String, bool) {
    let (username, is_default) = match username {
        Some(username) => (username, false),
        None => match data.get_default_username() {
            Some(username) => (username.to_string(), true),
            None => {
                eprintln!(
                    "{} Use {} to login first.",
                    "Not logged in.".bright_red(),
                    "neocities-sync login".bright_cyan()
                );
                exit(1);
            }
        },
    };
    (username, is_default)
}

async fn login_cmd(mut data: Data) {
    let Ok(username) = inquire::Text::new("Enter your username:").prompt() else {
        exit(1);
    };
    let Ok(password) =
        inquire::Password::new("Enter your password:").without_confirmation().prompt()
    else {
        exit(1);
    };
    let mut neocities = Neocities::new();
    if !neocities.login(&username, &password).await.unwrap() {
        eprintln!("{}", "Username or password is incorrect.".bright_red());
        exit(1);
    }
    let entry = keyring::Entry::new("neocities-sync", &username).unwrap();
    entry.set_password(&neocities.api_key.unwrap()).unwrap();
    eprintln!("{}", "Login successful.".bright_green());
    if data.get_default_username().is_none() {
        data.set_default_username(username);
    }
}

async fn logout_cmd(mut data: Data, username: Option<String>) {
    let (username, is_default) = get_username(username, &data);
    let entry = match keyring::Entry::new("neocities-sync", &username) {
        Ok(entry) => entry,
        Err(keyring::Error::NoEntry) => {
            eprintln!(
                "{} Use {} to login first.",
                "That username is not logged in.".bright_red(),
                "neocities-sync login".bright_cyan()
            );
            if is_default {
                data.remove_default_username();
            }
            exit(1);
        }
        Err(err) => panic!("{:#?}", err),
    };
    entry.delete_password().unwrap();
    eprintln!("{}", "Logout successful.".bright_green());
    if is_default {
        data.remove_default_username();
    }
}

async fn sync_cmd(
    data: Data,
    username: Option<String>,
    path: PathBuf,
    state: Option<PathBuf>,
    ignore_disallowed_file_types: bool,
) {
    let api_key = env::var("NEOCITIES_API_KEY").unwrap_or_else(|_| {
        let (username, _) = get_username(username, &data);
        let entry = keyring::Entry::new("neocities-sync", &username).unwrap();
        match entry.get_password() {
            Ok(api_key) => api_key,
            Err(keyring::Error::NoEntry) => {
                eprintln!(
                    "{} Use {} to login first.",
                    "Not logged in.".bright_red(),
                    "neocities-sync login".bright_cyan()
                );
                exit(1);
            }
            Err(err) => panic!("{:#?}", err),
        }
    });
    let state = state.unwrap_or_else(|| path.join(".state"));
    let mut neocities = Neocities::new();
    neocities.api_key = Some(api_key);
    let stats = match sync(&neocities, path, state, ignore_disallowed_file_types).await
    {
        Ok(stats) => stats,
        Err(error) => match error {
            SyncError::InvalidAuth => {
                eprintln!(
                    "{} Use {} to login again.",
                    "Invalid session.".bright_red(),
                    "neocities-sync login".bright_cyan()
                );
                exit(1);
            }
            SyncError::InvalidFileType => {
                eprintln!(
                    "{} Use {} to ignore such files.",
                    "Invalid file type.".bright_red(),
                    "--ignore-disallowed-file-types".bright_cyan()
                );
                exit(1);
            }
            SyncError::MissingFiles => {
                eprintln!(
                    "{} Re-run the sync command after deleting your state file.",
                    "Out of sync.".bright_red(),
                );
                exit(1);
            }
            SyncError::ReqwestError(error) => panic!("{:#?}", error),
            SyncError::IOError(error) => panic!("{:#?}", error),
        },
    };
    eprintln!(
        "{} {}, {} {}",
        "uploaded".bright_green(),
        stats.uploaded,
        "deleted".bright_red(),
        stats.deleted
    );
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    std::panic::set_hook(Box::new(|info| {
        eprintln!(
            "{} {}\n\nCreate an issue at {}",
            "Panic!".bold().bright_red(),
            info,
            "https://github.com/aspizu/neocities-sync/issues".bright_cyan()
        );
        exit(1);
    }));
    let args = Args::parse();
    let data = Data::new();
    match args.command {
        Commands::Login => login_cmd(data).await,
        Commands::Logout { username } => logout_cmd(data, username).await,
        Commands::Sync { username, path, state, ignore_disallowed_file_types } => {
            sync_cmd(data, username, path, state, ignore_disallowed_file_types).await
        }
    }
    Ok(())
}
