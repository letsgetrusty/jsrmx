use clap::{Parser, Subcommand};
use jsrmx::{
    input::{InputDirectory, JsonReaderInput, JsonSourceInput},
    output::Output,
    processor::{json, ndjson},
};

#[derive(Parser)]
#[command(name = "jsrmx")]
#[command(about = "A tool to break apart or combine large JSON and NDJSON files.", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Merges multiple single-object <dir>/${key}.json files into one json object.
    Merge {
        /// Compact JSON output
        #[arg(short, long, conflicts_with = "pretty", default_value_t = false)]
        compact: bool,
        /// Target input directory
        input: JsonSourceInput,
        /// Output filename or `-` for stdout
        #[arg(default_value = "-")]
        output: Output,
        /// Only split keys matching regex filter
        #[arg(short, long)]
        filter: Option<String>,
        /// Pretty-print output files
        #[arg(short, long, default_value_t = true)]
        pretty: bool,
        /// Alphabetically sort object keys
        #[arg(short, long, default_value_t = false)]
        sort: bool,
    },
    /// Splits single JSON object into multiple json objects.
    Split {
        /// Compact JSON output
        #[arg(short, long, conflicts_with = "pretty", default_value_t = false)]
        compact: bool,
        /// Input filename or `-` for stdin
        #[arg(default_value = "-")]
        input: JsonReaderInput,
        /// Target output directory or `-` for stdout
        #[arg(default_value = "-")]
        output: Output,
        /// Only split keys matching regex filter
        #[arg(short, long)]
        filter: Option<String>,
        /// Pretty-print output files
        #[arg(short, long, default_value_t = true)]
        pretty: bool,
    },
    /// Bundles multiple <dir>/*.json files into one ndjson file
    Bundle {
        /// Target input directory
        dir: InputDirectory,
        /// Output filename or `-` for stdout
        #[arg(default_value = "-")]
        output: Output,
    },
    /// Unbundle single [input] file into multiple json objects
    Unbundle {
        /// Compact JSON output
        #[arg(short, long, conflicts_with = "pretty", default_value_t = false)]
        compact: bool,
        /// Input filename or `-` for stdin
        #[arg(default_value = "-")]
        input: JsonReaderInput,
        /// Target output directory or `-` for stdout
        #[arg(default_value = "-")]
        output: Output,
        /// Output filename prefix
        #[arg(short, long)]
        name: Option<String>,
        /// Pretty-print output objects
        #[arg(short, long, default_value_t = true)]
        pretty: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    let env = env_logger::Env::default().filter_or("LOG_LEVEL", "warn");
    env_logger::Builder::from_env(env)
        .format_timestamp_millis()
        .init();

    std::panic::set_hook(Box::new(|panic| {
        // Use the error level to log the panic
        log::debug!("{:?}", panic);
        log::error!("{}", panic);
    }));

    match cli.command {
        Commands::Merge {
            compact,
            input,
            mut output,
            filter,
            pretty,
            sort,
        } => {
            let entries = input.0.get_entries(sort);
            let merged_object = json::merge(entries, filter);
            if pretty && !compact {
                output.set_pretty();
            }
            output
                .append(merged_object)
                .unwrap_or_else(|e| log::error!("Error writing to output: {e}"));
        }
        Commands::Split {
            compact,
            input,
            mut output,
            filter,
            pretty,
        } => {
            if pretty && !compact {
                output.set_pretty();
            };
            let object = input
                .0
                .get_object()
                .expect("Error reading input: {input:?}");
            let entries = json::split(object, filter);
            output.write_entries(entries).unwrap_or_else(|e| {
                log::error!("Error splitting: {e}");
            });
        }
        Commands::Bundle { dir, output } => ndjson::bundle(&dir, &output).unwrap_or_else(|e| {
            log::error!("Error bundling: {e}");
        }),
        Commands::Unbundle {
            compact,
            input,
            mut output,
            name,
            pretty,
        } => {
            if pretty && !compact {
                output.set_pretty();
            }
            ndjson::unbundle(&input, &output, name.as_deref()).unwrap_or_else(|e| {
                log::error!("Error unbundling: {e}");
            })
        }
    }
}
