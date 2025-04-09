use clap::Parser;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{fs, io};

pub struct FileData {
    progress_path: PathBuf,
    pub progress: Progress,
    pub story: String,
}

impl FileData {
    pub fn load() -> io::Result<Self> {
        // Parse the args
        let cli_args = Cli::parse();
        let story = fs::read_to_string(&cli_args.story)?
            .replace("\n", "↩")
            .replace("—", "-")
            .replace("—", "-")
            .replace("’", "'")
            .replace("“", "\"")
            .replace("”", "\"");
        let progress_path = cli_args.story.with_extension("progress.json");
        // Load the progress file
        let progress = Progress::load(&progress_path)?;
        // Build the persistant state
        Ok(FileData {
            progress_path,
            progress,
            story,
        })
    }
    pub fn save(&self) -> io::Result<()> {
        self.progress.save(&self.progress_path)
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct Progress {
    pub chars: usize,
}

impl Progress {
    fn load(path: &Path) -> io::Result<Self> {
        // Ensure the file exists
        if !path.exists() {
            let mut file = std::fs::File::create_new(path)?;
            let content = serde_json::to_string_pretty(&Progress::default()).unwrap();
            file.write_all(content.as_bytes())?;
        }
        // Read the config file
        Ok(serde_json::from_reader(fs::File::open(path)?).unwrap())
    }
    fn save(&self, path: &Path) -> io::Result<()> {
        // Overwrite the file
        let mut file = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(path)?;
        // With the current progress
        file.write_all(serde_json::to_string_pretty(&self).unwrap().as_bytes())
    }
}

#[derive(Parser)]
struct Cli {
    #[arg(long)]
    progress: Option<PathBuf>,
    story: PathBuf,
}
