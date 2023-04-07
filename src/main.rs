use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use clap::{Arg, Command};
use std::env;

#[derive(Debug, Deserialize, Serialize)]
struct Repository {
    name: String,
    description: Option<String>,
    language: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Repositories {
    items: Vec<Repository>,
}

async fn search_github_repositories(query: &str, access_token: &str) -> Result<Repositories, reqwest::Error> {
    let client = Client::new();
    let url = format!("https://api.github.com/search/repositories?q={}&per_page=100", query);

    let user_agent = format!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

    let request = client
        .get(url)
        .header(header::ACCEPT, "application/vnd.github+json")
        .header(header::AUTHORIZATION, format!("Bearer {}", access_token))
        .header(header::USER_AGENT, user_agent)
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await?;

    let repositories: Repositories = request.json().await?;
    Ok(repositories)
}

fn filter_repositories(
    repositories: Repositories,
    title: Option<&str>,
    description: Option<&str>,
    language: Option<&str>,
) -> Vec<Repository> {
    repositories
        .items
        .into_iter()
        .filter(|repo| {
            title.map_or(true, |title| {
                repo.name.to_lowercase().contains(&title.to_lowercase())
            })
        })
        .filter(|repo| {
            description.map_or(true, |description| {
                repo.description
                    .as_ref()
                    .map_or(false, |repo_description| {
                        repo_description
                            .to_lowercase()
                            .contains(&description.to_lowercase())
                    })
            })
        })
        .filter(|repo| {
            language.map_or(true, |language| {
                repo.language
                    .as_ref()
                    .map_or(false, |repo_language| {
                        repo_language.to_lowercase() == language.to_lowercase()
                    })
            })
        })
        .collect()
}

fn print_repo(repo: Repository) {
    let description = repo.description.unwrap_or_else(|| "No description".to_string());
    let language = repo.language.unwrap_or_else(|| "No language specified".to_string());

    println!(
        "Repository Name: {}\nDescription: {}\nLanguage: {}\n---",
        repo.name, description, language
    );
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let matches = Command::new("GitHub Repository Search")
        .arg(
            Arg::new("username")
                .short('u')
                .long("username")
                .value_name("USERNAME")
                .help("GitHub username")
                .required(true),
        )
        .arg(
            Arg::new("repositories")
                .short('r')
                .long("repositories")
                .value_name("REPOSITORIES")
                .help("Filter by the specified repository name"),
        )
        .arg(
            Arg::new("title")
                .short('t')
                .long("title")
                .value_name("TITLE")
                .help("Filter by the specified title"),
        )
        .arg(
            Arg::new("description")
                .short('d')
                .long("description")
                .value_name("DESCRIPTION")
                .help("Filter by the specified repository description"),
        )
        .arg(
            Arg::new("language")
                .short('l')
                .long("language")
                .value_name("LANGUAGE")
                .help("Filter by the specified programming language"),
        )
        .get_matches();

    let access_token = env::var("GITHUB_ACCESS_TOKEN").expect("GITHUB_ACCESS_TOKEN must be set");
    let github_username = matches.get_one::<String>("username").unwrap();

    let title = matches.get_one::<String>("title").map(String::as_str);
    let description = matches.get_one::<String>("description").map(String::as_str);
    let language = matches.get_one::<String>("language").map(String::as_str);

    let search_query = format!("user:{}", github_username);

    let repositories = search_github_repositories(&search_query, &access_token).await?;

    let filtered_repos = filter_repositories(repositories, title, description, language);

    for repo in filtered_repos {
        print_repo(repo);
    }

    Ok(())
}
