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
        ureq::get("https://api.github.com/orgs/theblindhen/repos")
            .set("User-Agent", "Reqwest")
            .timeout(std::time::Duration::from_secs(30))
            .call()
            .into_json_deserialize().unwrap();
    println!("{:#?}", repos);
    Ok(())
}
