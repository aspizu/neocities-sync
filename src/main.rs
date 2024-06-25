mod neocities;
mod state;
mod sync;

use std::{path::PathBuf, process::exit};

use clap::{Parser, Subcommand};
use colored::*;
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
    Logout,
    /// Sync a directory to neocities.
    Sync {
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

async fn login_cmd() {
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
    let entry = keyring::Entry::new("neocities-sync", "default").unwrap();
    entry.set_password(&neocities.api_key.unwrap()).unwrap();
    eprintln!("{}", "Login successful.".bright_green());
}

async fn sync_cmd(
    path: PathBuf,
    state: Option<PathBuf>,
    ignore_disallowed_file_types: bool,
) {
    let entry = keyring::Entry::new("neocities-sync", "default").unwrap();
    let api_key = match entry.get_password() {
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
    };
    let state = state.unwrap_or_else(|| {
        let mut path = path.clone();
        path.push(".state");
        path
    });
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
    match args.command {
        Commands::Login => login_cmd().await,
        Commands::Logout => {
            let entry = keyring::Entry::new("neocities-sync", "default").unwrap();
            entry.delete_password().unwrap();
            eprintln!("{}", "Logout successful.".bright_green());
        }
        Commands::Sync { path, state, ignore_disallowed_file_types } => {
            sync_cmd(path, state, ignore_disallowed_file_types).await
        }
    }
    Ok(())
}
