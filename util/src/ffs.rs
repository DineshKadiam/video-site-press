// SPDX-FileCopyrightText: 2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fmt::{Display, Formatter};
use std::fs;
use std::fs::{
    DirEntry,
    File,
    FileType,
    Metadata,
    Permissions
};
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum Error {
    Copy {
        error: io::Error,
        from: PathBuf,
        to: PathBuf
    },
    Create {
        error: io::Error,
        path: PathBuf
    },
    Read {
        error: io::Error,
        path: PathBuf
    },
    Remove {
        error: io::Error,
        path: PathBuf
    },
    Rename {
        error: io::Error,
        from: PathBuf,
        to: PathBuf
    },
    Write {
        error: io::Error,
        path: PathBuf
    }
}

/// A drop-in replacement for Path::canonicalize that returns a richer error type
/// including e.g. the path which was attempted to be canonicalized.
pub fn canonicalize(
    path: &Path
) -> Result<PathBuf, Error> {
    path.canonicalize()
        .map_err(|error| Error::read(error, path))
}

/// A drop-in replacement for fs::copy that returns a richer error type
/// including e.g. the path which was attempted to be copied.
pub fn copy(
    from: impl AsRef<Path>,
    to: impl AsRef<Path>,
) -> Result<u64, Error> {
    fs::copy(&from, &to)
        .map_err(|error| Error::copy(error, from.as_ref(), to.as_ref()))
}

/// A drop-in replacement for fs::create_dir that returns a richer error type
/// including e.g. the path which was attempted to be created.
pub fn create_dir(
    path: impl AsRef<Path>
) -> Result<(), Error> {
    fs::create_dir(&path)
        .map_err(|error| Error::create(error, path.as_ref()))
}

/// A drop-in replacement for fs::create_dir_all that returns a richer error type
/// including e.g. the path which was attempted to be created.
pub fn create_dir_all(
    path: impl AsRef<Path>
) -> Result<(), Error> {
    fs::create_dir_all(&path)
        .map_err(|error| Error::create(error, path.as_ref()))
}

/// A drop-in replacement for DirEntry::file_type that returns a richer error type
/// including e.g. the path which was attempted to be stat'ed.
pub fn file_type(
    dir_entry: &DirEntry
) -> Result<FileType, Error> {
    dir_entry.file_type()
        .map_err(|error| Error::read(error, dir_entry.path()))
}

/// A drop-in replacement for fs::metadata that returns a richer error type
/// including e.g. the path which was attempted to be read.
pub fn metadata(
    path: impl AsRef<Path>
) -> Result<Metadata, Error> {
    fs::metadata(&path)
        .map_err(|error| Error::read(error, path.as_ref()))
}

/// A drop-in replacement for File::open that returns a richer error type
/// including e.g. the path which was attempted to be opened
pub fn open(
    path: impl AsRef<Path>
) -> Result<File, Error> {
    File::open(&path)
        .map_err(|error| Error::read(error, path.as_ref()))
}

/// A drop-in replacement for fs::read_dir that returns a richer error type
/// including e.g. the path which was attempted to be read.
pub fn read_dir(
    path: impl AsRef<Path>
) -> Result<Vec<DirEntry>, Error> {
    match fs::read_dir(&path) {
        Ok(iter) => {
            iter
                .collect::<Result<_, _>>()
                .map_err(|error| Error::read(error, path.as_ref()))
        }
        Err(error) => Err(Error::read(error, path.as_ref()))
    }
}

/// A drop-in replacement for fs::read that returns a richer error type
/// including e.g. the path which was attempted to be read.
pub fn read(
    path: impl AsRef<Path>
) -> Result<Vec<u8>, Error> {
    fs::read(&path)
        .map_err(|error| Error::read(error, path.as_ref()))
}

/// A drop-in replacement for fs::read_to_string that returns a richer error type
/// including e.g. the path which was attempted to be read.
pub fn read_to_string(
    path: impl AsRef<Path>
) -> Result<String, Error> {
    fs::read_to_string(&path)
        .map_err(|error| Error::read(error, path.as_ref()))
}

/// A drop-in replacement for fs::remove_dir_all that returns a richer error type
/// including e.g. the path which was attempted to be removed.
pub fn remove_dir_all(
    path: impl AsRef<Path>
) -> Result<(), Error> {
    fs::remove_dir_all(&path)
        .map_err(|error| Error::remove(error, path.as_ref()))
}

/// A drop-in replacement for fs::remove_file that returns a richer error type
/// including e.g. the path which was attempted to be removed.
pub fn remove_file(
    path: impl AsRef<Path>
) -> Result<(), Error> {
    fs::remove_file(&path)
        .map_err(|error| Error::remove(error, path.as_ref()))
}

/// A drop-in replacement for fs::rename that returns a richer error type
/// including e.g. the path which was attempted to be renamed.
pub fn rename(
    from: impl AsRef<Path>,
    to: impl AsRef<Path>,
) -> Result<(), Error> {
    fs::rename(&from, &to)
        .map_err(|error| Error::rename(error, from.as_ref(), to.as_ref()))
}

/// A drop-in replacement for fs::set_permissions that returns a richer error
/// type including e.g. the path to which was attempted to be written.
pub fn set_permissions(
    path: impl AsRef<Path>,
    permissions: Permissions
) -> Result<(), Error> {
    fs::set_permissions(&path, permissions)
        .map_err(|error| Error::write(error, path.as_ref()))
}

/// A drop-in replacement for fs::write that returns a richer error type
/// including e.g. the path to which was attempted to be written.
pub fn write(
    path: impl AsRef<Path>,
    contents: impl AsRef<[u8]>
) -> Result<(), Error> {
    fs::write(&path, &contents)
        .map_err(|error| Error::write(error, path.as_ref()))
}

impl Error {
    pub fn copy(
        error: io::Error,
        from: impl Into<PathBuf>,
        to: impl Into<PathBuf>
    ) -> Error {
        Error::Copy {
            error,
            from: from.into(),
            to: to.into()
        }
    }

    pub fn create(
        error: io::Error,
        path: impl Into<PathBuf>
    ) -> Error {
        Error::Create {
            error,
            path: path.into()
        }
    }

    pub fn read(
        error: io::Error,
        path: impl Into<PathBuf>
    ) -> Error {
        Error::Read {
            error,
            path: path.into()
        }
    }

    pub fn rename(
        error: io::Error,
        from: impl Into<PathBuf>,
        to: impl Into<PathBuf>
    ) -> Error {
        Error::Rename {
            error,
            from: from.into(),
            to: to.into()
        }
    }

    pub fn remove(
        error: io::Error,
        path: impl Into<PathBuf>
    ) -> Error {
        Error::Remove {
            error,
            path: path.into()
        }
    }

    pub fn write(
        error: io::Error,
        path: impl Into<PathBuf>
    ) -> Error {
        Error::Write {
            error,
            path: path.into()
        }
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter) -> std::fmt::Result {
        match self {
            Error::Copy { error, from, to } => {
                let from = from.display();
                let to = to.display();

                write!(formatter, "{error} (copy:{from} -> {to})")
            }
            Error::Create { error, path } => {
                let path = path.display();

                write!(formatter, "{error} (create:{path})")
            }
            Error::Read { error, path } => {
                let path = path.display();

                write!(formatter, "{error} (read:{path})")
            }
            Error::Remove { error, path } => {
                let path = path.display();

                write!(formatter, "{error} (remove:{path})")
            }
            Error::Rename { error, from, to } => {
                let from = from.display();
                let to = to.display();

                write!(formatter, "{error} (rename:{from} -> {to})")
            }
            Error::Write { error, path } => {
                let path = path.display();

                write!(formatter, "{error} (write:{path})")
            }
        }
    }
}
