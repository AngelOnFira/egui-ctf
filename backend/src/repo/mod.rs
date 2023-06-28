use std::fs;

use git2::Repository;

pub struct Repo {}

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

        dbg!(&categories);
    }
}
