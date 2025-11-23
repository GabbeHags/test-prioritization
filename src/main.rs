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

use indexmap::IndexMap;
use test_prioritization::*;

use crate::jaccard::jaccard_index;

fn write_distances_to_csv<P: AsRef<Path>>(
    csv_path: P,
    distances: &IndexMap<(PathBuf, PathBuf), f32, foldhash::fast::RandomState>,
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
fn write_prio_list_to_csv<P: AsRef<Path>>(
    csv_path: P,
    prio_list: &[PathBuf],
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

    for line in prio_list.iter().enumerate().map(|(rank, path)| {
        let tc_id = path.file_stem().unwrap().display();
        let test_text_content = get_test_case_file_text_content(path).unwrap();
        let mut test_description = String::new();
        for l in test_text_content.lines() {
            let start_pat = "# Test Case for ";
            if let Some(start) = l.find(start_pat) {
                test_description = l[start_pat.len() + start..].trim().to_string();
                break;
            }
            let start_pat = "# Testcase for ";
            if let Some(start) = l.find(start_pat) {
                test_description = l[start_pat.len() + start..].trim().to_string();
                break;
            }
            let start_pat = "# Test Case Description for ";
            if let Some(start) = l.find(start_pat) {
                test_description = l[start_pat.len() + start..].trim().to_string();
                break;
            }
        }
        let trim_char = '*';
        if test_description.starts_with(trim_char) && test_description.ends_with(trim_char) {
            test_description = test_description.trim_matches(trim_char).to_string();
        }
        format!("{},{},{}\n", rank, tc_id, test_description)
    }) {
        csv_file_writer
            .write_all(line.as_bytes())
            .context("Could not write to csv file")?;
    }

    Ok(())
}

fn calculate_distances<P: AsRef<Path>>(
    files: &[P],
) -> anyhow::Result<IndexMap<(PathBuf, PathBuf), f32, foldhash::fast::RandomState>> {
    let now = Instant::now();
    let file_combos = get_file_combinations(files)?;
    let elapsed_time = now.elapsed();
    println!("get_file_combinations took: {}", elapsed_time.as_secs_f32());

    let mut distances: IndexMap<(PathBuf, PathBuf), f32, foldhash::fast::RandomState> =
        IndexMap::with_hasher(foldhash::fast::RandomState::default());
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
    println!("distances calculation took: {}", elapsed_time.as_secs_f32());

    Ok(distances)
}

fn prioritize_distances(
    distances: &IndexMap<(PathBuf, PathBuf), f32, foldhash::fast::RandomState>,
) -> anyhow::Result<Vec<PathBuf>> {
    let now = Instant::now();
    let max = distances
        .values()
        .cloned()
        .enumerate()
        .reduce(|(ia, a), (ib, b)| match a.total_cmp(&b) {
            std::cmp::Ordering::Less => (ib, b),
            std::cmp::Ordering::Equal => (ia, a),
            std::cmp::Ordering::Greater => (ia, a),
        })
        .unwrap();

    let max_kv = distances.get_index(max.0).unwrap();
    let elapsed_time = now.elapsed();
    println!("find biggest distance took: {}", elapsed_time.as_secs_f32());

    let now = Instant::now();
    let mut min_dist_tmp_map: HashMap<PathBuf, Option<f32>, foldhash::fast::RandomState> =
        foldhash::HashMap::new();

    let mut prio_vec: Vec<PathBuf> = vec![max_kv.0.0.to_path_buf(), max_kv.0.1.to_path_buf()];

    min_dist_tmp_map.insert(max_kv.0.0.to_path_buf(), None);
    min_dist_tmp_map.insert(max_kv.0.1.to_path_buf(), None);

    loop {
        for ((p1, p2), d) in distances {
            match (min_dist_tmp_map.get(p1), min_dist_tmp_map.get(p2)) {
                (None, Some(None)) => min_dist_tmp_map.insert(p1.to_path_buf(), Some(*d)),
                (Some(None), None) => min_dist_tmp_map.insert(p2.to_path_buf(), Some(*d)),
                (_, Some(Some(d2))) => min_dist_tmp_map.insert(p2.to_path_buf(), Some(d2.min(*d))),
                (Some(Some(d2)), _) => min_dist_tmp_map.insert(p1.to_path_buf(), Some(d2.min(*d))),
                _ => continue,
            };
        }

        // find max val in min dist
        let prio = min_dist_tmp_map
            .iter()
            .filter(|(_, v)| v.is_some())
            .reduce(|(pa, a), (pb, b)| {
                match a
                    .expect("filtered to only be some")
                    .total_cmp(&b.expect("filtered to only be some"))
                {
                    std::cmp::Ordering::Less => (pb, b),
                    std::cmp::Ordering::Equal => (pa, a),
                    std::cmp::Ordering::Greater => (pa, a),
                }
            })
            .unwrap();

        prio_vec.push(prio.0.to_path_buf());
        min_dist_tmp_map.insert(prio.0.to_path_buf(), None);

        if min_dist_tmp_map.values().all(|val| val.is_none()) {
            break;
        }
    }
    let elapsed_time = now.elapsed();
    println!(
        "create prio distance list took: {}",
        elapsed_time.as_secs_f32()
    );

    Ok(prio_vec)
}

fn main() -> anyhow::Result<()> {
    const DISTANCE_CSV_PATH: &str = "./distance.csv";
    const PRIO_CSV_PATH: &str = "./prio.csv";
    const TEST_CASE_FOLDER: &str = "Mozilla_TCs";

    let now = Instant::now();
    let paths = get_file_paths_from_dir(TEST_CASE_FOLDER)?;
    let elapsed_time = now.elapsed();
    println!(
        "get_file_paths_from_dir took: {}",
        elapsed_time.as_secs_f32()
    );

    let distances = calculate_distances(&paths)?;

    write_distances_to_csv(DISTANCE_CSV_PATH, &distances)?;

    let prio_list = prioritize_distances(&distances)?;

    write_prio_list_to_csv(PRIO_CSV_PATH, &prio_list)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, str::FromStr};

    use indexmap::IndexMap;

    use crate::prioritize_distances;

    #[test]
    fn prioritize_distances_valid() {
        let mut distances: IndexMap<(PathBuf, PathBuf), f32, foldhash::fast::RandomState> =
            IndexMap::with_hasher(foldhash::fast::RandomState::default());

        distances.insert(
            (
                PathBuf::from_str("tc1").unwrap(),
                PathBuf::from_str("tc2").unwrap(),
            ),
            0.57,
        );
        distances.insert(
            (
                PathBuf::from_str("tc1").unwrap(),
                PathBuf::from_str("tc3").unwrap(),
            ),
            0.78,
        );
        distances.insert(
            (
                PathBuf::from_str("tc1").unwrap(),
                PathBuf::from_str("tc4").unwrap(),
            ),
            0.8,
        );
        distances.insert(
            (
                PathBuf::from_str("tc1").unwrap(),
                PathBuf::from_str("tc5").unwrap(),
            ),
            0.78,
        );
        distances.insert(
            (
                PathBuf::from_str("tc2").unwrap(),
                PathBuf::from_str("tc3").unwrap(),
            ),
            0.75,
        );
        distances.insert(
            (
                PathBuf::from_str("tc2").unwrap(),
                PathBuf::from_str("tc4").unwrap(),
            ),
            0.77,
        );
        distances.insert(
            (
                PathBuf::from_str("tc2").unwrap(),
                PathBuf::from_str("tc5").unwrap(),
            ),
            0.75,
        );
        distances.insert(
            (
                PathBuf::from_str("tc3").unwrap(),
                PathBuf::from_str("tc4").unwrap(),
            ),
            0.37,
        );
        distances.insert(
            (
                PathBuf::from_str("tc3").unwrap(),
                PathBuf::from_str("tc5").unwrap(),
            ),
            0.69,
        );
        distances.insert(
            (
                PathBuf::from_str("tc4").unwrap(),
                PathBuf::from_str("tc5").unwrap(),
            ),
            0.8,
        );
        let prio = prioritize_distances(&distances).unwrap();

        assert_eq!(
            prio,
            ["tc1", "tc4", "tc5", "tc2", "tc3"].map(|x| PathBuf::from_str(x).unwrap())
        )
    }
}
