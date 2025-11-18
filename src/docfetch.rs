use anyhow::Result;

/// Fetch documentation HTML from docs.rs for a given crate, version, and symbol
pub fn fetch_docs(crate_name: &str, version: &str, symbol: &str) -> Result<String> {
    // Construct docs.rs URL with search parameter
    let url = format!(
        "https://docs.rs/{}/{}/{}/?search={}",
        crate_name, version, crate_name, symbol
    );

    println!("Searching docs.rs...");
    println!("URL: {}", url);

    // Fetch the docs.rs page
    let mut response = ureq::get(&url).call()?;
    let status = response.status();

    println!("Status: {}", status);

    if status == 200 {
        let body = response.body_mut().read_to_string()?;
        Ok(body)
    } else {
        anyhow::bail!("Failed to fetch documentation (status: {})", status)
    }
}
