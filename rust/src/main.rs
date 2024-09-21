use std::{error::Error, fs};

use clap::Parser;
use regex::Regex;
use reqwest;
use url::Url;

#[derive(Parser, Debug)]
#[command(version, about, long_about = r#"Include a capture group named "out" to match next expressions against only that captured string.
Text is split by lines at the first non-multiline expression."#)]
struct Cli {
    /// File or URL; by default files are split line-by-line and URLs are not split
    #[arg()]
    file_or_url: String,

    /// Regular expression(s) to match against text; use `--help` for more information
    #[arg(short = 'e', long)]
    regex: Vec<String>,

    /// Flips text split behavior; multiline for files and line-by-line for URLs
    #[arg(short, long)]
    multiline: bool,
}

const CAPTURE_GROUP: &str = "out";

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();

    let mut regexes = Vec::with_capacity(args.regex.len());
    for r in args.regex {
        match Regex::new(&r) {
            Ok(re) => {
                let has_output_group = re.capture_names().any(|n| {
                    match n {
                        Some(n) => n == CAPTURE_GROUP,
                        None => false,
                    }
                });
                regexes.push((
                    re,
                    has_output_group,
                ))
            },
            Err(err) => return Err(Box::new(err)),
        }
    }

    let (text, multiline) = match Url::parse(&args.file_or_url) {
        Ok(_) => (
            reqwest::blocking::get(&args.file_or_url)?.text()?,
            !args.multiline,
        ),
        Err(_) => (
            fs::read_to_string(&args.file_or_url)?,
            args.multiline,
        )
    };

    if regexes.len() > 0 {
        if multiline {
            if let Some(out) = match_all_re(&regexes, &text) {
                println!("{}", out);
            }
        } else {
            text.split('\n')
                .filter_map(|line| match_all_re(&regexes, line))
                .for_each(|line| println!("{}", line));
        }
    } else {
        println!("{}", text);
    }

    Ok(())
}

fn match_all_re<'a>(regexes: &Vec<(Regex, bool)>, text: &'a str) -> Option<&'a str> {
    let mut haystack = text;
    for re in regexes {
        let cap = match re.0.captures(haystack) {
            Some(c) => c,
            None => return None,
        };
        if re.1 {
            haystack = match cap.name(CAPTURE_GROUP) {
                Some(c) => c.as_str(),
                None => return None,
            }
        }
    }
    Some(haystack)
}
