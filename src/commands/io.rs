use std::{fs, path::{PathBuf, Path}};

use crate::models::MDPError;

pub trait FileReader {
    fn read(&self, paths: Vec<PathBuf>) -> Result<String, MDPError>;
}

pub struct MarkdownFileReader {}

impl FileReader for MarkdownFileReader {
    fn read(&self, paths: Vec<PathBuf>) -> Result<String, MDPError> {
        let mut s = String::new();

        for path in all_md_files(paths)? {
            s = format!("{}\n\n{}", s, fs::read_to_string(path.as_path()).map_err(|e| {
                MDPError::IOReadError{
                    path,
                    details: e.to_string(),
                }
            })?);
        }

        Ok(s)
    }

}

/// Returns all markdown files, i.e. find all markdown files in provided directories.
fn all_md_files(paths: Vec<PathBuf>) -> Result<Vec<PathBuf>, MDPError> {
    let mut res: Vec<PathBuf> = vec![];

    for path in paths {
        if path.is_dir() {
            let dir_iter_err = MDPError::IOError(
                format!("error while traversing the directory {}", path.to_string_lossy().into_owned())
            );
            for entry in fs::read_dir(path).map_err(|_| dir_iter_err.clone())? {
                let entry = entry.map_err(|_| dir_iter_err.clone())?;
                let p = entry.path();
                if is_md_file(&p) {
                    res.push(p);
                }
            }
        } else {
            res.push(path);
        }
    }

    Ok(res)
}

fn is_md_file<P: AsRef<Path>>(path: &P) -> bool {
    let path = path.as_ref();
    path.is_file() && path.extension().map_or(false, |ext| ext == "md")
}


pub trait OutputWriter {
    fn write_output(&self, output: &str) -> Result<(), MDPError>;
}

pub struct StdoutWriter {}

impl OutputWriter for StdoutWriter {
    fn write_output(&self, output: &str) -> Result<(), MDPError> {
        println!("{}", output);
        Ok(())
    }
}

pub struct FileWriter {
    pub path: PathBuf,
}

impl OutputWriter for FileWriter {
    fn write_output(&self, output: &str) -> Result<(), MDPError> {
        if self.file_exists() {
            self.delete_file()?;
        }

        fs::write(self.path.clone(), output)
            .map_err(|e| MDPError::IOReadError { path: self.path.clone(), details: e.to_string()})?;

        self.make_read_only()?;

        Ok(())
    }
}

impl FileWriter {
    #[cfg(unix)]
    /// Set file permissions to read-only.
    fn make_read_only(&self) -> Result<(), MDPError> {
        use std::os::unix::fs::PermissionsExt;

        let metadata = fs::metadata(self.path.clone()).map_err(|_|
            MDPError::IOError("could not set file read only".to_string())
        )?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o444); // Read-only for owner, group, and others
        fs::set_permissions(self.path.clone(), permissions).map_err(|_|
            MDPError::IOError("could not set file read only".to_string())
        )
    }

    #[cfg(windows)]
    /// Set file permissions to read-only.
    fn make_read_only(&self) -> Result<(), MDPError> {
        let err = MDPError::IOError("could not remove read only flag from file".to_string());
        let metadata = fs::metadata(self.path.as_path()).map_err(|_| err.clone())?;
        let mut permissions = metadata.permissions();
        permissions.set_readonly(true);
        fs::set_permissions(self.path.as_path(), permissions).map_err(|_| err)
    }

    #[cfg(unix)]
    fn delete_file(&self) -> Result<(), MDPError> {
        fs::remove_file(&self.path).map_err(|_| MDPError::IOError("could not delete file".to_string()))
    }

    #[cfg(windows)]
    fn delete_file(&self) -> Result<(), MDPError> {
        let err = MDPError::IOError("could not remove read only flag from file".to_string());
        let metadata = fs::metadata(self.path.as_path()).map_err(|_| err.clone())?;
        let mut permissions = metadata.permissions();
        permissions.set_readonly(false);
        fs::set_permissions(self.path.as_path(), permissions).map_err(|_| err)?;

        fs::remove_file(&self.path).map_err(|_| MDPError::IOError("could not delete file".to_string()))
    }

    fn file_exists(&self) -> bool {
        Path::new(&self.path).exists()
    }
}
