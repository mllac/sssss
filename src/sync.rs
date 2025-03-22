use std::{
    fs::{
        OpenOptions,
        self,
        File,
    },
    io::{
        BufReader,
        BufWriter,
        SeekFrom,
        BufRead,
        Write,
        self,
        Read,
        Seek,
    },
    path::{
        PathBuf,
        Path,
    },
    env,
};

use rayon::iter::{ParallelBridge, ParallelIterator};
use git2::Repository;
use thiserror::Error;

use crate::git;

// check if `path` exists under ~/.config/
pub fn in_config(path: impl AsRef<Path>) -> bool {
    env::home_dir()
        .map(|x| x.join(".config"))
        .map(|x| x.join(path).exists())
        .unwrap_or_default()
}

// check if `path` exists in /tmp
pub fn in_tmp(path: impl AsRef<Path>) -> bool {
    let tmp = env::temp_dir()
        .join(path);

    tmp.is_file() | tmp.is_dir()
}

// check if `path` exists in /bin
pub fn in_bin(path: impl AsRef<Path>) -> bool {
    let bin = PathBuf::from("/bin");

    fs::exists(bin.join(path))
        .unwrap_or_default()
}

#[derive(
    Debug,
    Error,
)]

pub enum StoreError {
    #[error("an io error occurred storing the path: {0}")]
    Io(#[from] io::Error),
    #[error("an error occurred syncing with git: {0}")]
    Git(#[from] git2::Error),
    #[error("path does not exist in the filesystem")]
    NotExists,
    #[error("path already exists in the store")]
    Duplicate,
    #[error(transparent)]
    Eyre(#[from] color_eyre::Report),
}

pub struct Store {
    reader: BufReader<File>,
    writer: BufWriter<File>,
}

impl Store {
    pub fn sync(
        &mut self,
        remote: String,
    ) -> Result<(), StoreError> {
        self.paths()?
            .par_bridge()
            .try_for_each(|x| {
                git::update(x, &remote)
            })?;

        Ok(())
    }

    pub fn insert(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<(), StoreError> {
        if !fs::exists(&path)? {
            return Err(StoreError::NotExists);
        }

        if self.contains(&path) {
            return Err(StoreError::Duplicate);
        }

        let path = path.as_ref()
            .to_string_lossy();

        let l = format!("{}\n", path);

        self.writer.seek(SeekFrom::End(0))?;

        self.writer.write_all(l.as_ref())?;

        self.writer.flush()?;

        Ok(())
    }

    pub fn paths(
        &mut self,
    ) -> Result<
        impl Iterator<Item = Repository>,
        StoreError,
    > {
        self.reader.rewind()?;

        Ok(
            self.reader
                .by_ref()
                .lines()
                .map_while(Result::ok)
                .map(PathBuf::from)
                .map(Repository::init)
                .map_while(Result::ok)
        )
    }

    pub fn contains(
        &mut self,
        path: impl AsRef<Path>,
    ) -> bool {
        self.reader
            .by_ref()
            .lines()
            .map_while(Result::ok)
            .any(|x| {
                let x: &Path = x.as_ref();
                x == path.as_ref()
            })
    }

    pub fn new(
        path: impl AsRef<Path>,
    )-> Result<Self, StoreError> {
        let path = env::temp_dir()
            .join(path);

        let f1 = OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .read(true)
            .open(path)?;

        let f2 =
            f1.try_clone()?;

        let reader =
            BufReader::new(f1);

        let writer =
            BufWriter::new(f2);

        Ok(
            Self {
                reader,
                writer,
            },
        )
    }
}
