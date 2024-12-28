use clap::{Parser, Subcommand};
use jsrmx::{
    input::{InputDirectory, JsonReaderInput, JsonSourceInput},
    output::{JsonAppendableOutput, JsonWritableOutput},
    processor::{json, NdjsonBundler, NdjsonUnbundler},
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
        output: JsonAppendableOutput,
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
        output: JsonWritableOutput,
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
        output: JsonAppendableOutput,
        /// String-escaped nested JSON fields to escape
        #[arg(short, long, value_delimiter = ',')]
        escape: Option<Vec<String>>,
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
        output: JsonWritableOutput,
        /// List of field names to read for filename, uses first non-null value
        #[arg(short, long, value_delimiter = ',')]
        name: Option<Vec<String>>,
        /// Field name to append before the file extension
        #[arg(short, long)]
        r#type: Option<String>,
        /// Pretty-print output objects
        #[arg(short, long, default_value_t = true)]
        pretty: bool,
        /// String-escaped nested JSON fields to unescape
        #[arg(short, long, value_delimiter = ',')]
        unescape: Option<Vec<String>>,
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
            output,
            filter,
            pretty,
            sort,
        } => {
            let entries = input.get_entries(sort);
            let merged_object = json::merge(entries, filter);
            if pretty && !compact {
                output.write().unwrap().set_pretty(true);
            }
            output
                .read()
                .unwrap()
                .append(merged_object)
                .unwrap_or_else(|e| log::error!("Error writing to output: {e}"));
        }
        Commands::Split {
            compact,
            input,
            output,
            filter,
            pretty,
        } => {
            if pretty && !compact {
                output.write().unwrap().set_pretty(true);
            };
            let object = input.get_object().expect("Error reading input: {input:?}");
            let entries = json::split(object, filter);
            output
                .read()
                .unwrap()
                .write_entries(entries)
                .unwrap_or_else(|e| {
                    log::error!("Error splitting: {e}");
                });
        }
        Commands::Bundle {
            dir,
            escape,
            output,
        } => NdjsonBundler::new(dir, output.0)
            .bundle(escape)
            .unwrap_or_else(|e| {
                log::error!("Error bundling: {e}");
            }),
        Commands::Unbundle {
            compact,
            input,
            name,
            output,
            pretty,
            r#type,
            unescape,
        } => {
            if pretty && !compact {
                output.write().unwrap().set_pretty(true);
            }
            NdjsonUnbundler::new(input, output.0, unescape)
                .unbundle(name, r#type)
                .unwrap_or_else(|e| {
                    log::error!("Error unbundling: {e}");
                })
        }
    }
}
