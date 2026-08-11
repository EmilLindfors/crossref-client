//! Prints which crossref rate-limit pool the configured client lands in.
use crossref_client::Crossref;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let polite = Crossref::builder().polite("emil@lindfors.no").build()?;
    let anon = Crossref::builder().build()?;

    for (label, c) in [("polite", &polite), ("anonymous", &anon)] {
        let resp = c
            .client
            .get("https://api.crossref.org/works?rows=1")
            .send()
            .await?;
        let hdr = |n: &str| {
            resp.headers()
                .get(n)
                .and_then(|v| v.to_str().ok())
                .unwrap_or("-")
                .to_string()
        };
        println!(
            "{label:10} pool={} limit={}/{}",
            hdr("x-api-pool"),
            hdr("x-rate-limit-limit"),
            hdr("x-rate-limit-interval")
        );
    }
    Ok(())
}
