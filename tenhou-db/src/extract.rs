use std::io::{Read, Seek};

use anyhow::{bail, Result};
use camino::Utf8Path;
use flate2::read::GzDecoder;
use indicatif::ProgressBar;
use itertools::Itertools;

use crate::parse::*;

/// ".html" reader => HTML string
///
/// This really is [`std::io::read_to_string`] (pending stabilization as of 2022-09-18).
pub fn read_html(mut html_reader: impl Read) -> Result<String> {
    let mut html_str = String::new();
    html_reader.read_to_string(&mut html_str)?;
    Ok(html_str)
}

/// ".html.gz" reader => HTML string
pub fn read_html_gz(html_gz_reader: impl Read) -> Result<String> {
    read_html(GzDecoder::new(html_gz_reader))
}

/// name + either ".html" or ".html.gz" reader => HTML string
pub fn read_html_maybe_gz(name: &Utf8Path, reader: impl Read) -> Result<String> {
    match name.extension() {
        Some("gz") if name.as_str().ends_with(".html.gz") =>
            read_html_gz(reader),
        Some("html") =>
            read_html(reader),
        Some(ext) =>
            bail!("unknown extension for name: {}; extension={:?}", name, ext),
        _ =>
            bail!("unknown extension for name: {}", name),
    }
}

/// name + either html or html.gz reader => all game log lines (id + metadata)
pub fn archive_lines_from_file(name: &Utf8Path, reader: impl Read)
                               -> Result<Vec<GameLogInfo>> {
    read_html_maybe_gz(name, reader).map(|html|
        html.split("\n").flat_map(parse_archive_line).collect_vec())
}

/// Calls the closure with a slice of id strings for each valid ".html.gz" sub-archive in the given
/// Tenhou yearly ".zip" meta-archive.
pub fn process_zip_file<F>(zip_reader: impl Read + Seek, mut f: F) -> Result<()>
    where
        F: for<'a> FnMut(&'a str, &'a [GameLogInfo]) {
    let mut archive = zip::ZipArchive::new(zip_reader)?;
    let len = archive.len();
    let total = archive.file_names()
        .filter(|name| name.as_ref().is_ok_and(|n| n.ends_with(".html.gz"))).count();
    let progress = ProgressBar::new(total as u64);
    let mut overall_result = Ok(());
    for file_index in 0..len {
        let zip_file = match archive.by_index(file_index) {
            Ok(file) => file,
            Err(e) => {
                log::warn!("cannot unzip file #{} (of {}) => {}", file_index, len, e);
                continue;
            }
        };
        let name = zip_file.name()?.into_owned();
        if !name.ends_with(".html.gz") { continue; }
        let file_result = archive_lines_from_file(&Utf8Path::new(&name), zip_file)
            .map(|entries| f(&name, &entries));
        overall_result = overall_result.and(file_result);
        progress.inc(1);
    }
    progress.finish();
    overall_result
}
