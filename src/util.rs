use std::collections::HashMap;
use std::env;

use anyhow::Context;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct UploadValidator {
    frontend_url_max_length: Option<usize>,
    allowed_file_types: HashMap<String, Option<usize>>,
}

impl UploadValidator {
    pub fn from_env() -> anyhow::Result<Self> {
        let frontend_url_max_length: Option<usize> = match env::var("FRONTEND_URL_MAX_LENGTH").ok() {
            Some(s) => Some(s.parse::<usize>().context("Failed to parse FRONTEND_URL_MAX_LENGTH")?),
            None => None
        };

        let allowed_file_types: HashMap<String, Option<usize>> = match env::var("DISCORD_ALLOWED_FILE_EXTENSIONS").ok() {
            None => HashMap::with_capacity(0),
            Some(value) => {
                let mut map = HashMap::with_capacity(value.split(',').count());
                for file_type in value.split(',') {
                    match file_type.split_once('=') {
                        None => {
                            map.insert(file_type.to_string(), None);
                        },
                        Some((ext, length_str)) => {
                            let length = length_str.parse::<usize>().context("Failed to parse max length")?;
                            map.insert(ext.to_string(), Some(length));
                        }
                    }
                }
                map
            }
        };

        Ok(UploadValidator {
            frontend_url_max_length,
            allowed_file_types,
        })
    }

    pub fn check(&self, path: &str, original: &str) -> Result<(), &str> {
        let file_name = path.split('/').last().ok_or("Failed to get file name")?;
        let file_extension = file_name.split('.').last().ok_or("Invalid file name or extension")?;

        let original_file_name = original.split('/').last().ok_or("Failed to get attachment file name")?;
        let original_file_extension = original_file_name.split('.').last().ok_or("Invalid attachment file name or extension")?;

        if file_extension != original_file_extension {
            return Err("Target file type does not match attachment file type");
        }

        if let Some(max_length) = self.frontend_url_max_length {
            if path.len() > max_length {
                return Err("File path too long!");
            }
        }

        if !self.allowed_file_types.is_empty() {
            let file_ext_str = file_extension;

            match self.allowed_file_types.get(file_ext_str) {
                Some(opt) => {
                    if let Some(max_length) = opt {
                        if file_name.len() > *max_length {
                            return Err("File name too long!");
                        }
                    }
                }
                None => return Err("File type not allowed!"),
            }
        }

        Ok(())
    }
}