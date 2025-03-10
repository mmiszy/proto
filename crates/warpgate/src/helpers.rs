use crate::error::WarpgateError;
use miette::IntoDiagnostic;
use reqwest::Url;
use starbase_utils::fs::{self, FsError};
use std::io;
use std::path::{Path, PathBuf};

pub fn extract_prefix_from_slug(slug: &str) -> &str {
    slug.split('/').next().expect("Expected an owner scope!")
}

pub fn extract_suffix_from_slug(slug: &str) -> &str {
    slug.split('/')
        .nth(1)
        .expect("Expected a package or repository name!")
}

pub fn determine_cache_extension(value: &str) -> &str {
    for ext in [".toml", ".json", ".yaml", ".yml"] {
        if value.ends_with(ext) {
            return ext;
        }
    }

    ".wasm"
}

pub fn create_wasm_file_stem(name: &str) -> String {
    let mut name = name.to_lowercase().replace('-', "_");

    if !name.ends_with("_plugin") {
        name.push_str("_plugin");
    }

    name
}

pub async fn download_url_to_temp(raw_url: &str, temp_dir: &Path) -> miette::Result<PathBuf> {
    let url = Url::parse(raw_url).into_diagnostic()?;
    let filename = url.path_segments().unwrap().last().unwrap().to_owned();

    // Fetch the file from the HTTP source
    let response = reqwest::get(url)
        .await
        .map_err(|error| WarpgateError::Http { error })?;
    let status = response.status();

    if status.as_u16() == 404 {
        return Err(WarpgateError::DownloadNotFound {
            url: raw_url.to_owned(),
        }
        .into());
    }

    if !status.is_success() {
        return Err(WarpgateError::DownloadFailed {
            url: raw_url.to_owned(),
            status: status.to_string(),
        }
        .into());
    }

    // Write the bytes to our temporary file
    let mut contents = io::Cursor::new(
        response
            .bytes()
            .await
            .map_err(|error| WarpgateError::Http { error })?,
    );

    let temp_path = temp_dir.join(filename);
    let mut file = fs::create_file(&temp_path)?;

    io::copy(&mut contents, &mut file).map_err(|error| FsError::Create {
        path: temp_path.to_path_buf(),
        error,
    })?;

    Ok(temp_path)
}

pub fn move_or_unpack_download(temp_path: &Path, dest_path: &Path) -> miette::Result<()> {
    let ext = temp_path.extension().map(|e| e.to_str().unwrap());

    match ext {
        // Move these files as-is
        Some("wasm" | "toml" | "json" | "yaml" | "yml") => {
            fs::rename(temp_path, dest_path)?;
        }

        // Unpack archives to temp and move the wasm file
        Some("tar" | "gz" | "xz" | "tgz" | "txz" | "zip") => {
            unimplemented!();
        }

        Some(x) => {
            return Err(miette::miette!(
                "Unsupported file extension `{}` for downloaded plugin.",
                x
            ));
        }

        None => {
            return Err(miette::miette!(
                "Unsure how to handle downloaded plugin as no file extension/type could be derived."
            ));
        }
    };

    Ok(())
}
