mod jaccard;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    time::Instant,
};

use anyhow::Context;
use foldhash::HashMapExt;

use test_prioritization::*;

use crate::jaccard::jaccard_index;

fn write_distances_to_csv<P: AsRef<Path>>(
    csv_path: P,
    distances: &HashMap<(PathBuf, PathBuf), f32, foldhash::fast::RandomState>,
) -> anyhow::Result<()> {
    let csv_path = csv_path.as_ref();
    anyhow::ensure!(
        csv_path.exists()
            || csv_path
                .parent()
                .context("Could not find directory of csv path")?
                .is_dir(),
        "The given csv path is not valid. Path {}",
        csv_path.display()
    );

    let mut csv_file_writer = BufWriter::new(
        File::options()
            .write(true)
            .create(true)
            .truncate(true)
            .open(csv_path)
            .context("Could not create or open csv file")?,
    );

    for line in distances
        .iter()
        .map(|((p1, p2), distance)| format!("{},{},{}\n", p1.display(), p2.display(), distance))
    {
        csv_file_writer
            .write_all(line.as_bytes())
            .context("Could not write to csv file")?;
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    const DISTANCE_CSV_PATH: &str = "./distance.csv";
    const TEST_CASE_FOLDER: &str = "Mozilla_TCs";

    let now = Instant::now();
    let paths = get_file_paths_from_dir(TEST_CASE_FOLDER)?;
    let elapsed_time = now.elapsed();
    println!(
        "get_file_paths_from_dir took: {}",
        elapsed_time.as_secs_f32()
    );

    let now = Instant::now();
    let file_combos = get_file_combinations(&paths)?;
    let elapsed_time = now.elapsed();
    println!("get_file_combinations took: {}", elapsed_time.as_secs_f32());

    let mut distances: HashMap<(PathBuf, PathBuf), f32, foldhash::fast::RandomState> =
        foldhash::HashMap::new();
    let mut tc_file_cache: HashMap<PathBuf, anyhow::Result<String>, foldhash::fast::RandomState> =
        foldhash::HashMap::new();

    let now = Instant::now();
    for (p1, p2) in file_combos {
        let p1_text = tc_file_cache
            .entry(p1.clone())
            .or_insert_with(|| get_test_case_file_text_content(&p1))
            .as_ref()
            .unwrap()
            .clone();

        let p2_text = tc_file_cache
            .entry(p2.clone())
            .or_insert_with(|| get_test_case_file_text_content(&p2))
            .as_ref()
            .unwrap()
            .clone();

        let distance = 1. - jaccard_index(&p1_text, &p2_text, jaccard::Strategy::Word);
        distances.insert((p1.clone(), p2.clone()), distance);
    }
    let elapsed_time = now.elapsed();
    println!("calc distances took: {}", elapsed_time.as_secs_f32());

    write_distances_to_csv(DISTANCE_CSV_PATH, &distances)?;

    Ok(())
}
