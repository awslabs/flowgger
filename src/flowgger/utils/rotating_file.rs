use std::fs::OpenOptions;
use std::io::stderr;
use std::path::{Path, PathBuf};
use std::{
    fs::{self, File},
    io::{self, Write},
};

/// Writer providing a file rotating feature when a file reaches the configured size
pub struct RotatingFile {
    basename: PathBuf,
    max_size: usize,
    max_files: i32,

    current_file: Option<File>,
    current_size: usize,
}

impl RotatingFile {
    /// Create a new rotating file, which implements the Write trait
    /// Files are rotated when the data to write in the current file is going to reach the specified limit.
    /// During a rotation:
    /// - each existing file is renamed 'basename.{n}' -> 'basename.{n+1}', starting with n = maxfiles -2 up to 0.
    ///     The oldest 'basename.{maxfiles -1}' is therefore overwritten and the old data are lost
    /// - the current file is renamed 'basename' -> 'basename.0'
    /// - A new file 'basename' is created
    ///
    ///
    /// # Parameters
    /// - 'basename': Original file name and path.
    /// - 'max_size': Target size for rotating files. If a write will reach that limit, the file is rotated first.
    /// - 'max_files': Count of files that can be created in addition to the original file,
    ///             named 'basename.N' where'basename' is always the file being currently written
    ///             - 'basename.0' is always the most recent file that has been rotated
    ///             - 'basename.N' is always the oldest file
    ///
    /// # Example
    /// From parameters:
    ///     - basename = 'logs/syslog.log'
    ///     - max_files = 2
    ///
    /// The following files will be generated:
    ///     - Current file = 'logs/syslog.log'
    ///     - Older file = 'logs/syslog.log.0'
    ///     - Oldest file = 'logs/syslog.log.1'
    ///
    pub fn new<P: AsRef<Path>>(basepath: P, max_size: usize, max_files: i32) -> Self {
        let basename = basepath.as_ref().to_path_buf();
        Self {
            basename,
            max_size,
            max_files,

            current_file: None,
            current_size: 0,
        }
    }

    /// Open the base file and ready for logging and set it as current file
    /// Should typically be called after object creation in order to be able to start writing
    ///
    /// # Returns
    /// - 'Ok': The file has successfully been open
    /// - 'Err': The file system could not open the file
    ///
    pub fn open(&mut self) -> io::Result<()> {
        match RotatingFile::open_file(self.basename.as_path()) {
            Ok(file) => {
                let metadata = file.metadata()?;
                self.current_size = metadata.len() as usize;

                self.current_file = Some(file);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    /// Static method to open a file
    ///
    /// # Parameters
    /// 'basename': Path of the file to open
    ///
    /// # Returns
    /// Operation result, Ok(file) or IO Err()
    ///
    pub fn open_file<P: AsRef<Path>>(basename: P) -> io::Result<File> {
        OpenOptions::new().create(true).append(true).open(basename)
    }

    /// Build a file path with the specified file number as externsion, on the model:
    /// 'basename.N'. If the index is negative, the basename is returned
    ///
    /// # Parameters
    /// - 'file_num':  File number (file extension)
    ///
    /// # Returns
    /// Path of the new file
    ///
    fn build_file_path(&self, file_num: i32) -> PathBuf {
        if file_num < 0 {
            self.basename.clone()
        } else {
            let mut path = self.basename.clone();
            path.set_extension(file_num.to_string());
            path
        }
    }

    /// Execute a log file rotation
    /// Starting from the file n=(self.max_files -1):
    /// - each existing file is renamed 'basename.{n}' -> 'basename.{n+1}'
    /// - the current file is renamed 'basename' -> 'basename.0'
    /// A new file 'basename' is created
    ///
    /// # Returns
    /// - 'Ok':   when the rotation has been done
    /// - 'Err':  when the new file could not be open
    ///
    fn rotate(&mut self) -> io::Result<()> {
        let _ = writeln!(
            stderr(),
            "File {} reached size limit {}, rotating",
            self.basename.to_string_lossy(),
            self.max_size
        );

        // Make sure that file is not gonna be used anymore
        let _ = self.current_file.take();

        // Shift all existing files extension by 1
        let mut dest_pathbuf = self.build_file_path(self.max_files - 1);
        let mut src_pathbuf;
        for file_num in (0..self.max_files).rev() {
            src_pathbuf = self.build_file_path(file_num - 1);
            let _ = fs::rename(src_pathbuf.as_path(), dest_pathbuf.as_path());
            dest_pathbuf = src_pathbuf;
        }

        // Create new logfile, fail if we can't
        self.open()?;
        self.current_size = 0;

        Ok(())
    }
}

/// Implementation of the Write trait to allow the Rotating file object to be used as data writer
/// Refer to https://doc.rust-lang.org/std/io/trait.Write.html for trait description
impl Write for RotatingFile {
    /// Writes will always give an event to write, or a set in case of bufferred writing.
    /// We don't split the data to write as we don't want to cut in the middle of an event
    /// So we always write the full data block
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let written = buf.len();

        // If the current file can't hold the whole data block, rotate
        if self.current_size + written > self.max_size {
            self.rotate()?;
        }

        // Write the whole data block
        self.current_size += written;
        if let Some(Err(err)) = self.current_file.as_mut().map(|file| file.write(buf)) {
            return Err(err);
        }

        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        if let Some(Err(err)) = self.current_file.as_mut().map(|file| file.flush()) {
            Err(err)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate tempdir;
    use tempdir::TempDir;

    fn build_pattern_list(count: u32, length: usize) -> Vec<String> {
        let mut pattern_list = Vec::new();
        for i in 0..count {
            let mut pattern_str = std::iter::repeat(i.to_string())
                .take(length)
                .collect::<String>();
            pattern_str.push('\n');
            pattern_list.push(pattern_str);
        }
        pattern_list
    }

    #[test]
    fn test_rotation_2files() -> Result<(), io::Error> {
        let tmp_dir = TempDir::new("test_rotation")?;
        let file_base = tmp_dir.path().join("test_log.log");
        let file_rotated = tmp_dir.path().join("test_log.0");
        let file_rotated2 = tmp_dir.path().join("test_log.1");

        let test_patterns = build_pattern_list(7, 6);

        let mut rotating_file = RotatingFile::new(&file_base, 16, 2);
        let result = rotating_file.open();
        if result.is_err() {
            println!("Error opening log file {}: {}", file_base.to_string_lossy(), result.unwrap_err());
            fs::write("d.d", "test")?;
        }
        assert!(rotating_file.open().is_ok());

        // No rotation yet
        let _ = rotating_file.write(test_patterns[0].as_bytes());
        let _ = rotating_file.write(test_patterns[1].as_bytes());
        assert_eq!(
            fs::read_to_string(file_base.as_path()).unwrap(),
            format!("{}{}", test_patterns[0], test_patterns[1])
        );

        // First rotation
        let _ = rotating_file.write(test_patterns[2].as_bytes());
        assert_eq!(
            fs::read_to_string(file_rotated.as_path()).unwrap(),
            format!("{}{}", test_patterns[0], test_patterns[1])
        );
        assert_eq!(fs::read_to_string(file_base.as_path()).unwrap(), test_patterns[2]);

        // Second rotation
        let _ = rotating_file.write(test_patterns[3].as_bytes());
        let _ = rotating_file.write(test_patterns[4].as_bytes());
        assert_eq!(
            fs::read_to_string(file_rotated2.as_path()).unwrap(),
            format!("{}{}", test_patterns[0], test_patterns[1])
        );
        assert_eq!(
            fs::read_to_string(file_rotated.as_path()).unwrap(),
            format!("{}{}", test_patterns[2], test_patterns[3])
        );
        assert_eq!(fs::read_to_string(file_base.as_path()).unwrap(), test_patterns[4]);

        // Oldest log overwritten
        let _ = rotating_file.write(test_patterns[5].as_bytes());
        let _ = rotating_file.write(test_patterns[6].as_bytes());
        assert_eq!(
            fs::read_to_string(file_rotated2.as_path()).unwrap(),
            format!("{}{}", test_patterns[2], test_patterns[3])
        );
        assert_eq!(
            fs::read_to_string(file_rotated.as_path()).unwrap(),
            format!("{}{}", test_patterns[4], test_patterns[5])
        );
        assert_eq!(fs::read_to_string(file_base).unwrap(), test_patterns[6]);

        let _ = rotating_file.flush();

        Ok(())
    }

    #[test]
    fn test_file_invalid_path() {
        let file_base = "/some/crazy/path/test_log.log";

        let mut rotating_file = RotatingFile::new(file_base, 16, 2);
        assert!(rotating_file.open().is_err());
    }
}
