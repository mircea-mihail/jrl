use std::fs;
use std::io;

pub fn get_day_file_name(days_before: i64) -> String {
    (chrono::offset::Local::now() - chrono::Duration::days(days_before))
        .format("%Y-%m-%d.txt")
        .to_string()
}

pub fn get_jrl_files(jrl_dir_path: &std::path::PathBuf) -> io::Result<Vec<std::path::PathBuf>> {
    let dir_files = fs::read_dir(jrl_dir_path)?;
    let mut journal_paths: Vec<std::path::PathBuf> = Vec::new();

    for file in dir_files {
        let path = file?.path();

        if let Some(stem) = path.file_stem() {
            let string_split_stem = stem.to_string_lossy();
            let stem_vec: Vec<&str> = string_split_stem.split("-").collect();
            if stem_vec.len() == 3
                && stem_vec[0].len() == 4
                && stem_vec[1].len() == 2
                && stem_vec[2].len() == 2
            {
                journal_paths.push(path);
            }
        }
    }
    journal_paths.sort();

    Ok(journal_paths)
}
