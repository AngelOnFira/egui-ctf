use std::{collections::HashMap, fs, path::Path, time::Duration};

use entity::entities::challenge::{self};
use git2::Repository;
use sea_orm::{ActiveModelTrait, ColumnTrait, Database, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};

pub struct Repo {
    challenges: HashMap<String, RepoChallenge>,
}

#[derive(Debug)]
pub struct RepoChallenge {
    challenge: Challenge,
    catgeory: String,
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
    points: i32,
    flag: String,
    active: bool,
    author: String,
}

impl Repo {
    pub fn clone_repo() {
        let url = "https://github.com/h4tt/H4TT-3.0.git";

        // Make sure that the ctf folder doesn't exist
        if Path::new("./ctf").exists() {
            fs::remove_dir_all("./ctf").unwrap();
        }

        // Clone a repository to a local directory
        let _repo = loop {
            match Repository::clone(url, "./ctf") {
                Ok(repo) => break repo,
                Err(e) => {
                    println!("failed to clone: {}", e);
                    std::thread::sleep(Duration::from_secs(5));
                }
            };
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
    pub fn parse_repo() -> Self {
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
        let mut challenge_map: HashMap<String, RepoChallenge> = HashMap::new();
        for category in categories {
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

                    let challenge_title = challenge.title.clone();

                    // Create the repo challenge struct
                    let repo_challenge = RepoChallenge {
                        challenge,
                        catgeory: category.file_name().unwrap().to_str().unwrap().to_string(),
                        files: HashMap::new(),
                        dockerfile: None,
                        nomadfile: None,
                    };

                    challenge_map.insert(challenge_title, repo_challenge);
                }
            }
        }

        Repo {
            challenges: challenge_map,
        }
    }

    /// Load all the challenges into the database
    pub async fn update_database(&self) -> Option<&RepoChallenge> {
        let db = Database::connect("postgres://postgres:postgres@localhost:5432/postgres")
            .await
            .unwrap();

        // Iterate over each challenge
        for challenge in self.challenges.values() {
            // Check the database to see if the challenge already exists
            let challenge_exists = challenge::Entity::find()
                .filter(challenge::Column::Title.eq(&challenge.challenge.title))
                .one(&db)
                .await
                .unwrap();

            match challenge_exists {
                // If the challenge exists, update it with the info from the
                // repo. This is helpful for things like changing point counts,
                // or changing the flag.
                Some(db_challenge) => {
                    let mut db_challenge: challenge::ActiveModel = db_challenge.into();

                    // Update all the fields
                    db_challenge.title = Set(challenge.challenge.title.clone());
                    db_challenge.description = Set(challenge.challenge.description.clone());

                    if let Some(link) = &challenge.challenge.link {
                        db_challenge.link = Set(link.to_owned());
                    }

                    db_challenge.points = Set(challenge.challenge.points);
                    db_challenge.flag = Set(challenge.challenge.flag.clone());
                    db_challenge.active = Set(challenge.challenge.active);
                    db_challenge.author = Set(challenge.challenge.author.clone());
                    db_challenge.category = Set(challenge.catgeory.clone());

                    // Save the challenge
                    db_challenge.update(&db).await.unwrap();
                }

                // If it's not already in the database, create a new challenge
                None => {
                    let new_challenge = challenge::ActiveModel {
                        title: Set(challenge.challenge.title.clone()),
                        description: Set(challenge.challenge.description.clone()),
                        link: {
                            if let Some(link) = &challenge.challenge.link {
                                Set(link.to_owned())
                            } else {
                                Set("".to_string())
                            }
                        },
                        points: Set(challenge.challenge.points),
                        flag: Set(challenge.challenge.flag.clone()),
                        active: Set(challenge.challenge.active),
                        author: Set(challenge.challenge.author.clone()),
                        category: Set(challenge.catgeory.clone()),
                        ..Default::default()
                    };

                    new_challenge.save(&db).await.unwrap();
                }
            }
        }

        None
    }
}
