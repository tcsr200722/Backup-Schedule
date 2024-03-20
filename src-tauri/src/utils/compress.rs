pub mod compressor {
    //! The `compressor` module is able to create archieves for file or directory.
    //! The following compression formats are supported: zip, tar.gz, tar.bz2, tar.xz, 7z.
    //! 
    //! # Install Dependencies
    //! 
    //! ```
    //! $ cargo add zip tar flate2 bzip2 xz2 sevenz-rust walkdir
    //! ```

    /// Prepares packing for `from` path and `to path.
    /// Returns the new `from` path
    /// 
    /// # Arguments
    /// 
    /// * `from` - A string slice that holds the source path
    /// * `to` - A string slice that holds the save path
    /// 
    /// # Examples
    /// 
    /// ```
    /// use compress::compressor::prepare_packing;
    /// 
    /// let file = "path\\for\\target\\file";
    /// let dir = "path\\for\\target\\dir";
    /// let save_file = "path\\for\\save\\file.format";
    /// let save_dir = "path\\for\\save\\dir.format";
    /// 
    /// let packing_file = prepare_packing(file, save_file).except("some error occurs");
    /// let packing_dir = prepare_packing(dir, save_dir).except("some error occurs"); 
    /// assert_eq!(packing_file, "path\\for\\save\\packing\\file.format".to_string());
    /// assert_eq!(packing_dir, "path\\for\\target\\dir".to_string());
    /// ```
    fn prepare_packing(from: &str, to: &str) -> Result<String, std::io::Error> {
        use std::io::{Error, ErrorKind};
        use std::path::Path;
        use std::fs::{File, copy, create_dir_all};

        // Check whether `from` path exists
        if !Path::new(from).exists() {
            return Err(Error::from(ErrorKind::NotFound));
        }

        let src_path = Path::new(from);
        let save_path = Path::new(to);
        if let Some(prefix) = save_path.parent() {
            // Create the `to` file.
            create_dir_all(prefix)?;
            File::create(save_path)?;

            // If `from` path is file, creates a directory `packing` at the parent path of `to` path.
            // Copy `from` to `packing` and returns the new `from` path inside `packing` directory.
            if src_path.is_file() {
                let _path = prefix.join("packing");
                let packing_dir = _path.as_path();
                let file_name = src_path.file_name();
                match file_name {
                    Some(name) => {
                        create_dir_all(packing_dir)?;
                        copy(from, packing_dir.join(name.to_str().unwrap()))?;             
                        return Ok(packing_dir.display().to_string());
                    },
                    None => {
                        return Err(Error::from(ErrorKind::NotFound));
                    }
                }
            }
        } 

        Ok(from.to_string())
    }

    /// Finishes packing for `from` path and `to path.
    /// 
    /// # Arguments
    /// 
    /// * `from` - A string slice that holds the source path
    /// * `to` - A string slice that holds the save path
    /// 
    /// # Examples
    /// 
    /// ```
    /// use compress::compressor::finish_packing;
    /// 
    /// let file = "path\\for\\target\\file";
    /// let save_file = "path\\for\\save\\file.format";
    /// 
    /// let _ = finish_packing(file, save_file).except("some error occurs");
    /// assert_eq!(Path::new("path\\for\\save\\packing").exists(), false);
    /// ```
    fn finish_packing(from: &str, to: &str) -> Result<(), std::io::Error> {
        use std::path::Path;
        use std::fs::remove_dir_all;
    
        // If `from` path is file, delete the `packing` directory.
        if Path::new(from).is_file() {
            if let Some(prefix) = Path::new(to).parent() {
                remove_dir_all(prefix.join("packing"))?;        
            }
        }
    
        Ok(())
    }
    

    /// Creates zip archive for directory.
    /// 
    /// # Arguments
    /// 
    /// * `it` - Iterator to walk directory
    /// * `prefix` - A string slice that hold the directory path
    /// * `writer` - Write target that stores the archive data
    /// * `method` - The zip compression method
    /// 
    /// # Examples
    /// 
    /// ```
    /// use compress::compressor::prepare_packing;
    /// use std::io::{Error, ErrorKind};
    /// use std::fs::OpenOptions;
    /// 
    /// let dir = "path\\for\\target\\dir";
    /// let save_dir = "path\\for\\save\\dir.format";
    /// 
    /// let walkdir = walkdir::WalkDir::new(dir);
    /// let iter = walkdir.into_iter();
    /// let saver = OpenOptions::new().write(true).open(save_dir)?;
    /// if let Err(_) = zip_dir(&mut iter.filter_map(|e| e.ok()), &dir, saver, zip::CompressionMethod::Deflated) {
    ///     return Err(Error::from(ErrorKind::Other));
    /// }
    /// ```
    fn zip_dir<T>(
        it: &mut dyn Iterator<Item = walkdir::DirEntry>,
        prefix: &str,
        writer: T,
        method: zip::CompressionMethod,
    ) -> zip::result::ZipResult<()>
    where
        T: std::io::Write + std::io::Seek,
    {
        use std::io::{Read, Write};
        use std::path::Path;
        use std::fs::File;
        let mut zip = zip::ZipWriter::new(writer);
        let options = zip::write::FileOptions::default()
            .compression_method(method)
            .unix_permissions(0o755);
    
        let mut buffer = Vec::new();
        for entry in it {
            let path = entry.path();
            let name = path.strip_prefix(Path::new(prefix)).unwrap();
    
            // Write file or directory explicitly
            // Some unzip tools unzip files with directory paths correctly, some do not!
            if path.is_file() {
                // info!("adding file {:?} as {:?} ...", path, name);
                #[allow(deprecated)]
                zip.start_file_from_path(name, options)?;
                let mut f = File::open(path)?;
    
                f.read_to_end(&mut buffer)?;
                zip.write_all(&*buffer)?;
                buffer.clear();
            } else if !name.as_os_str().is_empty() {
                // Only if not root! Avoids path spec / warning
                // and mapname conversion failed error on unzip
                // info!("adding dir {:?} as {:?} ...", path, name);
                #[allow(deprecated)]
                zip.add_directory_from_path(name, options)?;
            }
        }
        zip.finish()?;
        Result::Ok(())
    }    

    /// Creates zip archive for file or directory.
    /// 
    /// # Arguments
    /// 
    /// * `from` - A string slice that holds the source path
    /// * `to` - A string slice that holds the save path
    /// 
    /// # Examples
    /// 
    /// ```
    /// use compress::compressor::create_zip_archive;
    /// 
    /// let src = "path\\for\\target";
    /// let dst = "path\\for\\save.zip";
    /// 
    /// let _ = create_zip_archive(src, dst)?;
    /// assert_eq!(Path::new("path\\for\\save.zip").exists(), true);
    /// ```
    fn create_zip_archive(from: &str, to: &str) -> Result<(), std::io::Error> {
        use std::io::{Error, ErrorKind};
        use std::fs::OpenOptions;
        if let Ok(src) = prepare_packing(from, to) {
            let prefix = src.clone();
            let walkdir = walkdir::WalkDir::new(src);
            let iter = walkdir.into_iter();
    
            let save_file = OpenOptions::new().write(true).open(to)?;
            if let Err(_) = zip_dir(&mut iter.filter_map(|e| e.ok()), &prefix, save_file, zip::CompressionMethod::Deflated) {
                return Err(Error::from(ErrorKind::Other));
            }
        }
    
        finish_packing(from, to)?;
    
        Ok(())
    }

    /// Creates tar package for file or directory.
    /// 
    /// # Arguments
    /// 
    /// * `from` - A string slice that holds the source path
    /// * `to` - A string slice that holds the save path
    /// 
    /// # Examples
    /// 
    /// ```
    /// use compress::compressor::create_tar_archive;
    /// 
    /// let src = "path\\for\\target";
    /// let dst = "path\\for\\save.tar";
    /// 
    /// let _ = create_tar_archive(src, dst)?;
    /// assert_eq!(Path::new("path\\for\\save.tar").exists(), true);
    /// ```
    fn create_tar_package(from: &str, to: &str) -> Result<(), std::io::Error> {
        use std::io::{Error, ErrorKind};
        use std::fs::OpenOptions;
        if let Ok(src) = prepare_packing(from, to) {
            let save_file = OpenOptions::new().write(true).open(to)?;
            let mut tar_builder = tar::Builder::new(save_file);

            if let Err(_) = tar_builder.append_dir_all("", src) {
                return Err(Error::from(ErrorKind::Other));
            }
        }

        finish_packing(from, to)?;

        Ok(())
    }

    /// Creates tar.gz archive for file or directory.
    /// 
    /// # Arguments
    /// 
    /// * `from` - A string slice that holds the source path
    /// * `to` - A string slice that holds the save path
    /// 
    /// # Examples
    /// 
    /// ```
    /// use compress::compressor::create_tar_gz_archive;
    /// 
    /// let src = "path\\for\\target";
    /// let dst = "path\\for\\save.tar.gz";
    /// 
    /// let _ = create_tar_gz_archive(src, dst)?;
    /// assert_eq!(Path::new("path\\for\\save.tar.gz").exists(), true);
    /// ```
    fn create_tar_gz_archive(from: &str, to: &str) -> Result<(), std::io::Error> {
        use std::io::{Error, ErrorKind};
        use std::fs::OpenOptions;
        use flate2::write::GzEncoder;

        if let Ok(src) = prepare_packing(from, to) {
            let save_file = OpenOptions::new().write(true).open(to)?;
            let encoder = GzEncoder::new(save_file, flate2::Compression::default());
            let mut tar_builder = tar::Builder::new(encoder);

            if let Err(_) = tar_builder.append_dir_all("", src) {
                return Err(Error::from(ErrorKind::Other));
            }
        }

        finish_packing(from, to)?;
        
        Ok(())
    }

    /// Creates tar.bz2 archive for file or directory.
    /// 
    /// # Arguments
    /// 
    /// * `from` - A string slice that holds the source path
    /// * `to` - A string slice that holds the save path
    /// 
    /// # Examples
    /// 
    /// ```
    /// use compress::compressor::create_tar_bz2_archive;
    /// 
    /// let src = "path\\for\\target";
    /// let dst = "path\\for\\save.tar.gz";
    /// 
    /// let _ = create_tar_bz2_archive(src, dst)?;
    /// assert_eq!(Path::new("path\\for\\save.tar.bz2").exists(), true);
    /// ```
    fn create_tar_bz2_archive(from: &str, to: &str) -> Result<(), std::io::Error> {
        use std::io::{Error, ErrorKind};
        use std::fs::OpenOptions;
        use bzip2::write::BzEncoder;

        if let Ok(src) = prepare_packing(from, to) {
            let save_file = OpenOptions::new().write(true).open(to)?;
            let encoder = BzEncoder::new(save_file, bzip2::Compression::best());
            let mut tar_builder = tar::Builder::new(encoder);

            if let Err(_) = tar_builder.append_dir_all("", src) {
                return Err(Error::from(ErrorKind::Other));
            }
        }

        finish_packing(from, to)?;

        Ok(())
    }

    /// Creates tar.xz archive for file or directory.
    /// 
    /// # Arguments
    /// 
    /// * `from` - A string slice that holds the source path
    /// * `to` - A string slice that holds the save path
    /// 
    /// # Examples
    /// 
    /// ```
    /// use compress::compressor::create_tar_xz_archive;
    /// 
    /// let src = "path\\for\\target";
    /// let dst = "path\\for\\save.tar.xz";
    /// 
    /// let _ = create_tar_xz_archive(src, dst)?;
    /// assert_eq!(Path::new("path\\for\\save.tar.xz").exists(), true);
    /// ```
    fn create_tar_xz_archive(from: &str, to: &str) -> Result<(), std::io::Error> {
        use std::io::{Error, ErrorKind};
        use std::fs::OpenOptions;
        use xz2::write::XzEncoder;

        if let Ok(src) = prepare_packing(from, to) {
            let save_file = OpenOptions::new().write(true).open(to)?;
            let encoder = XzEncoder::new(save_file, 6);
            let mut tar_builder = tar::Builder::new(encoder);

            if let Err(_) = tar_builder.append_dir_all("", src) {
                return Err(Error::from(ErrorKind::Other));
            }
        }

        finish_packing(from, to)?;

        Ok(())
    }

    /// Creates 7z archive for file or directory.
    /// 
    /// # Arguments
    /// 
    /// * `from` - A string slice that holds the source path
    /// * `to` - A string slice that holds the save path
    /// 
    /// # Examples
    /// 
    /// ```
    /// use compress::compressor::create_7zip_archive;
    /// 
    /// let src = "path\\for\\target";
    /// let dst = "path\\for\\save.7z";
    /// 
    /// let _ = create_7zip_archive(src, dst)?;
    /// assert_eq!(Path::new("path\\for\\save.7z").exists(), true);
    /// ```
    fn create_7zip_archive(from: &str, to: &str) -> Result<(), std::io::Error> {
        use std::io::{Error, ErrorKind};

        if let Err(_) = sevenz_rust::compress_to_path(from, to) {
            return Err(Error::from(ErrorKind::Other));
        }
        
        Ok(())
    }

    /// Creates archive for file or directory.
    /// 
    /// # Arguments
    /// 
    /// * `from` - A string slice that holds the source path
    /// * `to` - A string slice that holds the save path
    /// 
    /// # Examples
    /// 
    /// ```
    /// use compress::compressor::create_archive;
    /// 
    /// let src = "path\\for\\target";
    /// let dst = "path\\for\\save.zip";
    /// 
    /// let _ = create_archive(src, dst)?;
    /// assert_eq!(Path::new("path\\for\\save.zip").exists(), true);
    /// ```
    pub fn create_archive(from: &str, to: &str) -> Result<(), std::io::Error> {
        use std::io::{Error, ErrorKind};
        use std::path::Path;

        if !Path::new(from).exists() {
            return Err(Error::from(ErrorKind::NotFound));
        }

        let mut save_format: Option<&str> = None;
        let support_formats = Vec::from(["zip", "tar.gz", "tar.bz2", "tar.xz", "7z"]);
        for item in &support_formats {
            if to.ends_with(item) {
                save_format = Some(item);
                break;
            }
        }        

        if let Some(format) = save_format {
            match format {
                "zip" => {
                    create_zip_archive(from, to)?;
                },
                "tar.gz" => {
                    create_tar_gz_archive(from, to)?;
                },
                "tar.bz2" => {
                    create_tar_bz2_archive(from, to)?;
                },
                "tar.xz" => {
                    create_tar_xz_archive(from, to)?;
                },
                "7z" => {
                    create_7zip_archive(from, to)?;
                },
                _ => {
                    return Err(Error::from(ErrorKind::Unsupported));
                }       
            }
        }

        Ok(())
    }
}
