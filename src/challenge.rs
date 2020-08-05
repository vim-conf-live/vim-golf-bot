use serde::{Deserialize, Serialize};

use std::fs::File;
use std::str::FromStr;

use glob::glob;

#[derive(Serialize, Deserialize)]
pub struct Challenge {
    pub title: String,
    pub input: Vec<String>,
    timestamp: i64,
    pub id: String,
    pub output: Vec<String>,
    pub scores: Vec<Submission>,
}

#[derive(Serialize, Deserialize)]
pub struct Submission {
    pub author: String,
    pub score: usize,
    pub keys: String,
}

impl Challenge {
    pub fn new(title: String, input: Vec<String>, output: Vec<String>, id: String, timestamp: i64) -> Self {
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

    pub fn filename(id: &str) -> String {
        format!("challenges/{}.chal", id)
    }

    pub fn all() -> glob::Paths {
        glob("challenges/*.chal").unwrap()
    }

    pub fn last() -> Option<Self> {
        Self::all().filter_map(|res| {
            if let Ok(path) = res {
                let fname = path.file_stem()?.to_str()?;
                fname.parse::<Challenge>().ok()
            } else {
                None
            }
        }).max_by_key(|chall| chall.timestamp)
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
