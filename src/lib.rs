use std::{
    fs::{self, File},
    io::BufReader,
    path::{Path, PathBuf},
};

use anyhow::Context;

pub fn get_test_case_file_text_content<P: AsRef<Path>>(path: P) -> anyhow::Result<String> {
    let path = path.as_ref();
    anyhow::ensure!(
        path.exists(),
        "The given path \"{}\" does not exist",
        path.display()
    );
    anyhow::ensure!(
        path.is_file(),
        "The given path \"{}\" is not a file",
        path.display()
    );

    let file =
        File::open(path).with_context(|| format!("Could not open \"{}\"", path.display()))?;
    let file_buffered = BufReader::new(file);

    // we put the width to some high number that we probably wont hit.
    const WIDTH: usize = 1000;
    let text_content = html2text::from_read(file_buffered, WIDTH)
        .with_context(|| format!("Could not read in file \"{}\" as text", path.display()))?;

    Ok(text_content)
}

pub fn get_file_paths_from_dir<P: AsRef<Path>>(path: P) -> anyhow::Result<Vec<PathBuf>> {
    let path = path.as_ref();
    anyhow::ensure!(
        path.exists(),
        "The given path \"{}\" does not exist",
        path.display()
    );
    anyhow::ensure!(
        path.is_dir(),
        "The given path \"{}\" is not a directory",
        path.display()
    );
    let dir = fs::read_dir(path)
        .with_context(|| format!("Could read from folder \"{}\"", path.display()))?;
    let files: Vec<PathBuf> = dir.map(|a| a.unwrap().path()).collect();

    Ok(files)
}

#[cfg(test)]
mod tests {
    use crate::{get_file_paths_from_dir, get_test_case_file_text_content};

    #[test]
    fn test_get_file_paths_from_dir_valid() {
        let files = get_file_paths_from_dir("Mozilla_TCs").unwrap();
        assert!(
            !files.is_empty(),
            "did not find any files in valid directory"
        );
    }
    #[test]
    fn test_get_file_paths_from_dir_valid_end_slash() {
        let files_slash = get_file_paths_from_dir("Mozilla_TCs/").unwrap();
        let files_no_slash = get_file_paths_from_dir("Mozilla_TCs").unwrap();
        assert_eq!(files_slash, files_no_slash);
    }

    #[test]
    fn test_get_file_paths_from_dir_missing_dir() {
        assert!(get_file_paths_from_dir("not_existing_dir").is_err());
    }
    #[test]
    fn test_get_file_paths_from_dir_path_not_dir() {
        assert!(get_file_paths_from_dir("Mozilla_TCs/TC1.html").is_err());
    }

    #[test]
    fn test_get_test_case_file_text_content_missing_file() {
        assert!(
            get_test_case_file_text_content("Mozilla_TCs/not_existing_file.html").is_err(),
            "missing files did not panic"
        );
    }
    #[test]
    fn test_get_test_case_file_text_content_path_is_not_file() {
        assert!(
            get_test_case_file_text_content("Mozilla_TCs/").is_err(),
            "when given a directory it did not panic"
        );
    }
    #[test]
    fn test_get_test_case_file_text_content_path_valid() {
        let text = get_test_case_file_text_content("Mozilla_TCs/TC1.html").unwrap();
        assert!(
            !text.is_empty(),
            "did not find any content in valid html file"
        )
    }
}
