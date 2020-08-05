use serde::{Deserialize, Serialize};

use std::fs::File;
use std::path::PathBuf;
use std::str::FromStr;
use std::str::Lines;

use glob::glob;

pub trait FromLines: Sized {
    type Error;

    fn from_lines(lines: &mut Lines) -> Result<Self, Self::Error>;
}

#[derive(Serialize, Deserialize)]
pub struct TextBlock {
    pub lang: Option<String>,
    pub content: Vec<String>,
}

impl FromLines for TextBlock {
    type Error = String;
    fn from_lines(lines: &mut Lines) -> Result<Self, Self::Error> {
        let mut lang: Option<String> = None;
        let mut content: Vec<String> = Vec::new();
        let mut is_filling = false;

        for line in lines {
            if line.starts_with("```") {
                if is_filling {
                    return Ok(Self::new(lang, content));
                } else {

                    // Starting to read the block, we need to extract the lang too
                    let line = line.strip_prefix("```").unwrap();
                    if !line.is_empty() {
                        lang = Some(line.to_owned());
                    }

                    is_filling = true;
                }
            } else if is_filling {
                content.push(line.to_owned());
            }
        }

        Err(String::from("Failed to parse TextBlock, reached EOF."))
    }
}

impl TextBlock {
    pub fn new(lang: Option<String>, content: Vec<String>) -> Self {
        Self { lang, content }
    }

    pub fn as_markdown(&self) -> String {
        let mut block;
        if let Some(inner) = &self.lang {
            block = format!("```{}\n", inner);
        } else {
            block = String::from("```\n");
        }
        block.push_str(&self.content.join("\n"));
        block.push_str("\n```");

        block
    }

    pub fn len(&self) -> usize {
        self.content.len()
    }

    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }
}

#[derive(Serialize, Deserialize)]
pub struct Submission {
    pub author: String,
    pub score: usize,
    pub keys: String,
}

#[derive(Serialize, Deserialize)]
pub struct Challenge {
    pub id: String,
    pub title: String,
    timestamp: i64,
    pub input: TextBlock,
    pub output: TextBlock,
    pub scores: Vec<Submission>,
}

impl Challenge {
    pub const DIR: &'static str = "challenges";
    pub fn new(
        title: String,
        input: TextBlock,
        output: TextBlock,
        id: String,
        timestamp: i64,
    ) -> Self {
        Challenge {
            title,
            id,
            timestamp,
            input,
            output,
            scores: Vec::new(),
        }
    }

    pub fn add_submission(&mut self, author: String, keys: String, score: usize) -> usize {
        let sub = Submission {
            score,
            author,
            keys,
        };

        self.scores.push(sub);

        score
    }

    pub fn filename(id: &str) -> PathBuf {
        let mut buf = PathBuf::new();
        buf.push(Self::DIR);
        buf.push(id);
        buf.set_extension("chal");

        buf
    }

    pub fn create_dir() -> std::io::Result<()> {
        let mut buf = PathBuf::new();
        buf.push(Self::DIR);

        if std::fs::metadata(&buf).is_err() {
            std::fs::create_dir(buf)?;
        }

        Ok(())
    }

    pub fn all() -> glob::Paths {
        glob(&format!("{}/*.chal", Self::DIR)).unwrap()
    }

    pub fn last() -> Option<Self> {
        Self::all()
            .filter_map(|res| {
                if let Ok(path) = res {
                    let fname = path.file_stem()?.to_str()?;
                    fname.parse::<Challenge>().ok()
                } else {
                    None
                }
            })
            .max_by_key(|chall| chall.timestamp)
    }
}

impl FromStr for Challenge {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.strip_prefix("`").unwrap_or(s);
        let s = s.strip_suffix("`").unwrap_or(s);

        let file = File::open(Challenge::filename(s)).map_err(|e| e.to_string())?;
        ron::de::from_reader(file).map_err(|e| e.to_string())
    }
}
