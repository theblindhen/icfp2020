use serde::Deserialize;
use serde_json::Value;

// This `derive` requires the `serde` dependency.
#[derive(Deserialize, Debug)]
struct Repo {
    name: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Sending request...");

    // Demonstrates sending an HTTP request and decoding the response as JSON.
    let repos : Value =
        ureq::get("http://jsrn.dk/advanced.json")
            .timeout(std::time::Duration::from_secs(30))
            .call()
            .into_json_deserialize().unwrap();
    println!("{:#?}", repos);
    Ok(())
}
