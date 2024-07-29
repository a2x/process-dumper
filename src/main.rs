use std::str::FromStr;

use anyhow::Result;

use clap::{ArgAction, Parser};

use log::LevelFilter;

use memflow::prelude::v1::*;

use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};

mod dumper;

#[derive(Debug, Parser)]
#[command(author, version)]
struct Args {
    /// The name of the memflow connector to use.
    #[arg(short, long)]
    connector: Option<String>,

    /// Additional arguments to pass to the memflow connector.
    #[arg(short = 'a', long)]
    connector_args: Option<String>,

    /// The name of the process to dump.
    #[arg(short, long)]
    process_name: String,

    /// The name of the file to write the process buffer to.
    ///
    /// If not specified, the name of the process will instead be used.
    #[arg(short, long)]
    file_name: Option<String>,

    /// Increase logging verbosity. Can be specified multiple times.
    #[arg(short, action = ArgAction::Count)]
    verbose: u8,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let level_filter = match args.verbose {
        0 => LevelFilter::Error,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        3 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    TermLogger::init(
        level_filter,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .unwrap();

    let conn_args = args
        .connector_args
        .map(|s| ConnectorArgs::from_str(&s).expect("unable to parse connector arguments"))
        .unwrap_or_default();

    let os = match args.connector {
        Some(conn) => {
            let inventory = Inventory::scan();

            inventory
                .builder()
                .connector(&conn)
                .args(conn_args)
                .os("win32")
                .build()?
        }
        None => {
            #[cfg(windows)]
            {
                memflow_native::create_os(&OsArgs::default(), LibArc::default())?
            }
            #[cfg(not(windows))]
            {
                panic!("no connector specified")
            }
        }
    };

    let mut process = os.into_process_by_name(&args.process_name)?;

    dumper::dump_process(&mut process, args.file_name)?;

    Ok(())
}
