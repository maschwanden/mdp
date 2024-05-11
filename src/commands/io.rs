use std::{fs, path::{PathBuf, Path}};

use crate::models::MDPError;

pub trait FileReader {
    fn read_file(&self, path: PathBuf) -> Result<String, MDPError>;
}

pub struct MarkdownFileReader {}

impl FileReader for MarkdownFileReader {
    fn read_file(&self, path: PathBuf) -> Result<String, MDPError> {
        fs::read_to_string(path.as_path()).map_err(|_| MDPError::IOReadError(path))
    }
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
            .map_err(|e| {
                dbg!(e);
                MDPError::IOReadError(self.path.clone())
            })?;

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
        let metadata = fs::metadata(self.path.as_path()).map_err(|_|
            MDPError::IOError("could not set file read only".to_string())
        )?;
        let mut permissions = metadata.permissions();
        permissions.set_readonly(true);
        fs::set_permissions(self.path.as_path(), permissions).map_err(|_|
            MDPError::IOError("could not set file read only".to_string())
        )
    }

    fn delete_file(&self) -> Result<(), MDPError> {
        fs::remove_file(&self.path).map_err(|_| MDPError::IOError("could not delete file".to_string()))
    }

    fn file_exists(&self) -> bool {
        Path::new(&self.path).exists()
    }
}
