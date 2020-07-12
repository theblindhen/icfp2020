use serde::Deserialize;

// This `derive` requires the `serde` dependency.
#[derive(Deserialize, Debug)]
struct Repo {
    name: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Sending request...");

    // Demonstrates sending an HTTP request and decoding the response as JSON. The GitHub API
    // requires setting a User-Agent header, which means we can't use the `reqwest::blocking::get`
    // convenience function.
    let repos : Vec<Repo> =
        reqwest::blocking::Client::builder()
            .user_agent("Reqwest")
            .build()?
            .get("https://api.github.com/orgs/theblindhen/repos")
            .send()?
            .json().unwrap();
    println!("{:#?}", repos);
    Ok(())
}
