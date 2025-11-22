mod jaccard;
use std::{path::PathBuf, str::FromStr};

use test_prioritization::*;

fn main() -> anyhow::Result<()> {
    let paths = vec![
        PathBuf::from_str("Mozilla_TCs/TC1.html").unwrap(),
        PathBuf::from_str("Mozilla_TCs/TC2.html").unwrap(),
        PathBuf::from_str("Mozilla_TCs/TC3.html").unwrap(),
        PathBuf::from_str("Mozilla_TCs/TC4.html").unwrap(),
    ];
    let file_combos = get_file_combinations(&paths)?;

    dbg!(file_combos);

    Ok(())
}
