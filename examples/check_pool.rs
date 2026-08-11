//! Prints which crossref rate-limit pool the configured client lands in.
//!
//! Run with `cargo run --example check_pool`.
use crossref_client::{Crossref, WorksQuery};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let polite = Crossref::builder().polite("emil@lindfors.no").build()?;
    let anon = Crossref::builder().build()?;

    for (label, client) in [("polite", &polite), ("anonymous", &anon)] {
        // the pool and the budget are only known once a response has come back
        client.works(WorksQuery::empty()).await?;

        let limit = client.rate_limit();
        println!(
            "{label:10} pool={} limit={}/{:?}",
            client.api_pool().as_deref().unwrap_or("-"),
            limit.limit,
            limit.interval,
        );
    }
    Ok(())
}
