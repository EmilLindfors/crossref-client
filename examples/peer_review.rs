//! Reconstructs the open peer review history of a paper.
//!
//! Journals that publish their reviews register each one as a work in its own
//! right: type `peer-review`, a `review` block saying which round it belongs to
//! and what the referee recommended, and an `is-review-of` relation pointing
//! back at the paper. So the history is one query -- every work that reviews
//! this DOI -- and the reviews carry enough metadata to put themselves back in
//! order.
//!
//! ```text
//! cargo run --example peer_review -- 10.5194/egusphere-2026-890
//! ```
use crossref_client::{
    Crossref, Order, ResultControl, Review, Sort, WorkResultControl, WorksFilter, WorksQuery,
};
use std::collections::BTreeMap;

/// Copernicus runs its journals on open review, so this one has a public
/// history to show if no DOI is given on the command line.
const DEFAULT_DOI: &str = "10.5194/egusphere-2026-890";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let doi = std::env::args().nth(1).unwrap_or(DEFAULT_DOI.to_string());

    let client = Crossref::builder()
        // whoever runs this, so crossref can get in touch if it misbehaves
        .polite(std::env::var("CROSSREF_MAILTO").ok().as_deref())
        .build()?;

    let paper = client.work(&doi).await?;
    println!(
        "{}\n{}\n",
        paper.title.first().map_or("(untitled)", String::as_str),
        paper.doi
    );

    // every work that declares itself a review of this DOI, oldest first
    let reviews = client
        .works(
            WorksQuery::empty()
                .filter(WorksFilter::RelationType("is-review-of".to_string()))
                .filter(WorksFilter::RelationObject(doi))
                .sort(Sort::Created)
                .order(Order::Asc)
                .result_control(WorkResultControl::Standard(ResultControl::Rows(100))),
        )
        .await?;

    if reviews.items.is_empty() {
        println!("no public reviews -- this journal does not deposit them");
        return Ok(());
    }

    // `revision-round` is what orders a history, but plenty of members leave it
    // out and only number the reports, so fall back to a single round
    let mut rounds: BTreeMap<String, Vec<Review>> = BTreeMap::new();
    for work in reviews.items {
        let Some(review) = work.review else { continue };
        let round = review.revision_round.clone().unwrap_or_default();
        rounds.entry(round).or_default().push(review);
    }

    for (round, mut reviews) in rounds {
        reviews.sort_by(|a, b| a.running_number.cmp(&b.running_number));

        match round.as_str() {
            "" => println!("reviews"),
            round => println!("round {round}"),
        }
        for review in reviews {
            println!(
                "  {:<6} {:<16} {:<18} {}",
                review.running_number.as_deref().unwrap_or("-"),
                review.type_,
                review.stage.as_deref().unwrap_or("-"),
                review
                    .recommendation
                    .as_deref()
                    .unwrap_or("no recommendation"),
            );
            if let Some(statement) = &review.competing_interest_statement {
                println!("         competing interests: {statement}");
            }
        }
    }

    Ok(())
}
