mod placeholders;

use clap::{Parser, Subcommand};
use placeholders::collect_placeholders_from_environment_variable;
// use std::env;

pub async fn run(cli: Cli) {
    // println!("Hello, from PGMT!");

    // match env::current_dir() {
    //     Ok(path) => println!("Current working directory: {}", path.display()),
    //     Err(e) => eprintln!("Error getting current directory: {}", e),
    // }

    // match cli.command {
    //     Commands::Migrate {
    //         url, directories, ..
    //     } => {
    //         println!("URL: {}", url);
    //         for dir in directories {
    //             println!("Directory: {}", dir);
    //         }
    //     }
    // }
    match cli.command {
        Commands::Migrate { url, directories } => {
            let placeholders = collect_placeholders_from_environment_variable();
            println!("URL: {}", url);
            println!("URL: {}", url);
            for dir in directories.clone() {
                println!("Directory: {}", dir);
            }
            pgmt_core::migration_dirs(directories, url, placeholders)
                .await
                .unwrap();
        }
    }
}

#[derive(Parser)]
#[command(name = "pgmt")]
#[command(about = "PostgreSQL Migration Tool")]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run database migrations from one or more directories
    Migrate {
        /// Database URL
        #[arg(short = 'u', long)]
        url: String,

        /// Directories containing migrations
        #[arg(required = true)]
        directories: Vec<String>,
    },
}

// fn main() {
//     let cli = Cli::parse();
//
//     match cli.command {
//         Commands::Migrate { url, directories } => {
//             println!("URL: {}", url);
//             for dir in directories {
//                 println!("Directory: {}", dir);
//             }
//         }
//     }
// }
