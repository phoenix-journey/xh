use std::fs::File;
use std::io::prelude::*;

use indicatif::{HumanBytes, ProgressBar, ProgressStyle};
use regex::Regex;

use crate::utils::get_content_length;

// TODO: avoid name conflict unless `continue` flag is specified
fn get_file_name(response: &reqwest::Response) -> String {
    let fallback = response.url().path_segments().unwrap().last().unwrap();

    if let Some(value) = response.headers().get(reqwest::header::CONTENT_DISPOSITION) {
        let re = Regex::new("filename=\"(.*)\"").unwrap();
        if let Some(caps) = re.captures(value.to_str().unwrap()) {
            caps[1].to_string()
        } else {
            fallback.to_string()
        }
    } else {
        fallback.to_string()
    }
}

// TODO: support resumable downloads
pub async fn download_file(mut response: reqwest::Response, file_name: Option<String>, quiet: bool) {
    let file_name = file_name.unwrap_or(get_file_name(&response));
    let mut buffer = File::create(&file_name).unwrap();

    let pb =  if quiet {
        None
    } else {
        match get_content_length(&response.headers()) {
            Some(content_length) => {
                eprintln!(
                    "Downloading {} to \"{}\"",
                    HumanBytes(content_length),
                    file_name
                );
                let template = "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes} {bytes_per_sec} ETA {eta}";
                Some(
                    ProgressBar::new(content_length).with_style(
                        ProgressStyle::default_bar()
                            .template(template)
                            .progress_chars("#>-"))
                )
            }
            None => {
                eprintln!("Downloading to \"{}\"", file_name);
                Some(
                    ProgressBar::new_spinner().with_style(
                        ProgressStyle::default_bar()
                            .template("{spinner:.green} [{elapsed_precise}] {bytes} {bytes_per_sec} {msg}"))
                )
            }
        }
    };

    let mut downloaded = 0;
    while let Some(chunk) = response.chunk().await.unwrap() {
        buffer.write(&chunk).unwrap();
        downloaded += chunk.len() as u64;
        if let Some(pb) = &pb {
            pb.set_position(downloaded);
        }
    }

    if let Some(pb) = &pb {
        pb.finish_with_message("Done");
    }
}
