use std::path::PathBuf;

use clap::Parser;
#[derive(Parser, Debug)]
pub struct Cli {
    #[arg(short, long)]
    pub test_case_folder: PathBuf,

    #[arg(short, long)]
    pub distance_csv_output_path: PathBuf,

    #[arg(short, long)]
    pub prio_csv_output_path: PathBuf,
}
