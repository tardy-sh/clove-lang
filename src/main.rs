use clap::{Parser as ClapParser, Subcommand};
use clove_lang::cli::{self, CheckOptions, CheckResult, CliError};
use std::io::{self, Read};

#[derive(ClapParser)]
#[command(name = "clove")]
#[command(about = "Clove - A JSON query language for filtering, transforming, and validating JSON")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate and execute a Clove query
    Check {
        /// The Clove query to execute
        query: String,

        /// JSON input (reads from stdin if not provided)
        #[arg(short, long)]
        input: Option<String>,

        /// Pretty-print the output
        #[arg(short, long)]
        pretty: bool,

        /// Only validate syntax, don't execute
        #[arg(long)]
        syntax_only: bool,
    },

    /// List documentation categories
    Docs,

    /// Show documentation for a specific category
    Doc {
        /// Category name (use 'clove docs' to list categories)
        category: String,
    },

    /// Interactive onboarding tutorial
    Onboard,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Check {
            query,
            input,
            pretty,
            syntax_only,
        } => run_check(query, input, pretty, syntax_only),
        Commands::Docs => {
            print!("{}", cli::get_docs_overview());
            Ok(())
        }
        Commands::Doc { category } => match cli::get_doc_category(&category) {
            Ok(content) => {
                print!("{}", content);
                Ok(())
            }
            Err(e) => Err(e),
        },
        Commands::Onboard => {
            print!("{}", cli::get_onboarding_content());
            Ok(())
        }
    };

    if let Err(e) = result {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

fn run_check(
    query: String,
    input: Option<String>,
    pretty: bool,
    syntax_only: bool,
) -> Result<(), CliError> {
    let input = match input {
        Some(s) => Some(s),
        None if !atty::is(atty::Stream::Stdin) => {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer).map_err(CliError::Io)?;
            Some(buffer)
        }
        None => None,
    };

    let options = CheckOptions {
        query,
        input,
        pretty,
        syntax_only,
    };

    match cli::execute_check(&options)? {
        CheckResult::SyntaxValid => println!("Syntax is valid"),
        CheckResult::Success(output) => {
            let json = if pretty {
                serde_json::to_string_pretty(&output)
            } else {
                serde_json::to_string(&output)
            }
            .unwrap();
            println!("{}", json);
        }
    }
    Ok(())
}
