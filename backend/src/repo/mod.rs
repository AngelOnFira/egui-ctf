use std::{collections::HashMap, fs};

use git2::Repository;
use serde::{Serialize, Deserialize};

pub struct Repo {}

#[derive(Debug)]
pub struct RepoChallenge {
    challenge: Challenge,
    // Files
    files: HashMap<String, Vec<u8>>,
    // Dockerfile
    dockerfile: Option<String>,
    // Nomadfile
    nomadfile: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Challenge {
    // From challenge.json
    title: String,
    description: String,
    link: Option<String>,
    points: String,
    flag: String,
    active: bool,
}

impl Repo {
    pub fn clone_repo() {
        let url = "https://github.com/h4tt/H4TT-3.0.git";
        let repo = match Repository::clone(url, "./ctf") {
            Ok(repo) => repo,
            Err(e) => panic!("failed to clone: {}", e),
        };
    }

    /// A repo for the CTF will have the following structure:
    /// ctf/
    ///    - category-1/
    ///       - challenge-1/
    ///       - challenge-2/
    ///       - challenge-3/
    ///    - category-2/
    ///       - challenge-1/
    ///       - challenge-2/
    ///       - challenge-3/
    ///    ---
    ///    - [template]/
    ///    - servers/
    ///    - .git/
    ///
    /// The template, server, and .git folders can be ignored when parsing.
    ///
    /// Each of these challenges will contain the following structure
    /// challenge-1/
    ///    - challenge.json
    ///    - solution.txt
    ///    - Nomadfile [optional]
    ///    - Dockerfile [optional]
    ///    - files/ [optional]
    ///
    /// The challenge.json file will contain the following fields:
    ///
    /// {
    ///     "title": <Challenge title>,
    ///     "description": <Challenge description>,
    ///     "link": <Challenge link (optional)>,
    ///     "points": <Challenge points>,
    ///     "flag": <Challenge flag>,
    ///     "active": <true/false>,
    ///     "author": <Challenge author>,
    /// }
    ///
    ///
    pub fn parse_repo() {
        // Start by finding all the categories by getting all the folder names
        // in the ctf folder
        let mut categories = Vec::new();

        for entry in fs::read_dir("./ctf").unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                // Make sure it's not the template or servers folder
                if vec!["[template]", "servers", ".git"]
                    .iter()
                    .any(|&s| s == path.file_name().unwrap())
                {
                    continue;
                } else {
                    categories.push(path);
                }
            }
        }

        // Find all the challenges in each category
        let mut challenge_map = HashMap::new();
        for category in categories {
            let mut challenges = Vec::new();
            for entry in fs::read_dir(&category).unwrap() {
                let entry = entry.unwrap();
                let path = entry.path();
                if path.is_dir() {
                    
                    // Load the challenge.json file on this path
                    let challenge_json = fs::read_to_string(path.join("challenge.json")).unwrap();
                    
                    dbg!(&challenge_json);
                    
                    // Load it as a Challenge struct with serde
                    let mut challenge: Challenge = serde_json::from_str(&challenge_json).unwrap();

                    // If the challenge link is empty, set it to None
                    if challenge.link == Some("".to_string()) {
                        challenge.link = None;
                    }

                    // Create the repo challenge struct
                    let repo_challenge = RepoChallenge {
                        challenge,
                        files: HashMap::new(),
                        dockerfile: None,
                        nomadfile: None,
                    };

                    challenges.push(repo_challenge);
                    
                }
            }
            challenge_map.insert(category, challenges);
        }
    }
}
