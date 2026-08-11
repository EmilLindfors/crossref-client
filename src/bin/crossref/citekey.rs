//! Reading a bibtex citation key back into the work it stands for.
//!
//! A key like `@FloysandEtAl2021` carries three things and no more: the
//! surnames of the first authors, in the order they were credited, whether the
//! list was cut short, and the year the work was cited by. It carries no
//! title, so nothing here can identify a work on its own. What it can do is
//! answer, of a work crossref returned, whether the key vouches for it -- and
//! that is the half that matters, because crossref answers every query with
//! something.

use crossref_client::{Contributor, PartialDate, Work};
use serde::{Serialize, Serializer};
use std::fmt;
use std::str::FromStr;

/// The marks a latin letter is written with, ordered by how often a name
/// carries them, both per letter and within a letter.
///
/// The table is read in both directions. [`fold`] reads it right to left, to
/// compare a key written in ascii against metadata that is not. `Fløysand` is
/// how crossref spells the name and `Floysand` is all a bibtex key can hold.
///
/// [`CiteKey::spellings`] reads it left to right, to guess the spelling back:
/// crossref folds nothing, so `query.author=Floysand` matches the works of a
/// different person entirely and `query.author=Fløysand` is the only way to
/// reach these. The order is what keeps that guessing cheap -- `ø` before `ö`
/// before `ó` -- since the first guess is usually the right one.
const MARKS: &[(&str, &str)] = &[
    ("o", "øöóòôõő"),
    ("a", "åäáàâãā"),
    ("e", "éèêëěę"),
    ("u", "üúùûů"),
    ("i", "íìîï"),
    ("n", "ñńň"),
    ("c", "çčć"),
    ("s", "šśş"),
    ("z", "žźż"),
    ("y", "ýÿ"),
    ("l", "ł"),
    ("d", "đð"),
    ("r", "ř"),
    ("t", "ť"),
    ("g", "ğ"),
    ("ae", "æ"),
    ("oe", "œ"),
    ("ss", "ß"),
    ("th", "þ"),
];

/// The ascii skeleton of a name: lower case, letters and digits only, every
/// mark in [`MARKS`] folded away and everything else dropped.
///
/// `Fløysand`, `FLØYSAND` and `Floysand` all fold to `floysand`, and
/// `van Dijk`, `Van Dijk` and `VanDijk` all fold to `vandijk`. Two names are
/// the same name here when they are written the same way once the typography
/// is gone, which is the most a citation key can be held to.
pub fn fold(name: &str) -> String {
    let mut folded = String::with_capacity(name.len());
    for letter in name.chars().flat_map(char::to_lowercase) {
        if letter.is_ascii_alphanumeric() {
            folded.push(letter);
        } else if let Some((ascii, _)) = MARKS.iter().find(|(_, marked)| marked.contains(letter)) {
            folded.push_str(ascii);
        }
        // spaces, hyphens and apostrophes carry no identity of their own
    }
    folded
}

/// A citation key, read for the little it says: `@FloysandEtAl2021` is the
/// surname `Floysand`, an unnamed rest, and the year 2021.
///
/// Parsed rather than validated, so everything downstream holds a key that
/// named someone and gave a year.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CiteKey {
    key: String,
    surnames: Vec<String>,
    year: i32,
    et_al: bool,
}

impl CiteKey {
    /// the key as it was written, without its leading `@`
    pub fn key(&self) -> &str {
        &self.key
    }

    /// the surnames the key names, in the order it names them
    pub fn surnames(&self) -> &[String] {
        &self.surnames
    }

    /// the year the work was cited by, which is not always the year crossref
    /// issued it under -- a work published online in one year and in an issue
    /// the next is cited by either
    pub fn year(&self) -> i32 {
        self.year
    }

    /// whether the key cut the author list short with an `et al`
    pub fn et_al(&self) -> bool {
        self.et_al
    }

    /// The accented spellings this key could have been written down from, the
    /// likeliest first, each differing from the key in a single letter.
    ///
    /// Only ascii surnames are guessed at: a key that kept its diacritics
    /// needs no guessing. One letter at a time is a deliberate floor rather
    /// than a limit of the idea -- every further letter multiplies the
    /// requests, and a name that lost two marks is rare next to one that lost
    /// one.
    pub fn spellings(&self) -> Vec<Vec<String>> {
        let depth = MARKS
            .iter()
            .map(|(_, marked)| marked.chars().count())
            .max()
            .unwrap_or_default();
        let mut spellings = Vec::new();

        for rank in 0..depth {
            for (ascii, marked) in MARKS {
                let Some(mark) = marked.chars().nth(rank) else {
                    continue;
                };
                for (which, surname) in self.surnames.iter().enumerate() {
                    if !surname.is_ascii() {
                        continue;
                    }
                    // the offsets hold in the original because it is ascii
                    for (at, matched) in surname.to_lowercase().match_indices(ascii) {
                        let mut spelling = self.surnames.clone();
                        spelling[which] = respell(surname, at, matched.len(), mark);
                        spellings.push(spelling);
                    }
                }
            }
        }
        spellings
    }

    /// Why this work is not the one the key cites, or nothing at all when it
    /// is. `within` is how many years either side of the key's year still
    /// count as the same publication.
    ///
    /// Every reason is reported rather than the first, because a near miss is
    /// worth reading: a work off by a year and nothing else is a different
    /// answer than a work by different people.
    pub fn mismatches(&self, work: &Work, within: i32) -> Vec<Mismatch> {
        let mut mismatches = Vec::new();

        match published_year(work) {
            Some(year) if (year - self.year).abs() <= within => (),
            Some(year) => mismatches.push(Mismatch::Year {
                found: year,
                cited: self.year,
            }),
            None => mismatches.push(Mismatch::Undated),
        }

        let families = families(work);
        mismatches.extend(self.unmatched_authors(&families));

        if self.et_al && families.len() <= self.surnames.len() {
            mismatches.push(Mismatch::TooFewAuthors {
                found: families.len(),
            });
        }
        mismatches
    }

    /// Walks the key's surnames along the work's authors: they have to appear
    /// in the order the key gives them, starting at the first author, which is
    /// the one thing every citation key style agrees on.
    ///
    /// An author may answer for more than one of the key's surnames, because
    /// camel case cannot say whether `VanDijk` is one name or two. `van Dijk`
    /// takes both; two authors named `Le` and `Nguyen` take one each.
    fn unmatched_authors(&self, families: &[&str]) -> Vec<Mismatch> {
        let folded: Vec<String> = self.surnames.iter().map(|name| fold(name)).collect();
        let credited: Vec<String> = families.iter().map(|family| fold(family)).collect();
        let Some(first) = families.first() else {
            return vec![Mismatch::NoAuthors];
        };

        let mut surname = 0;
        let mut family = 0;
        while surname < folded.len() {
            let found = (family..credited.len())
                .find_map(|at| credits(&folded[surname..], &credited[at]).map(|took| (at, took)));

            match found {
                // the first surname has to be the first author, not merely
                // somewhere in the list
                Some((at, _)) if surname == 0 && at != 0 => {
                    return vec![Mismatch::NotFirstAuthor {
                        expected: self.surnames[0].clone(),
                        found: first.to_string(),
                    }];
                }
                Some((at, took)) => {
                    surname += took;
                    family = at + 1;
                }
                None => {
                    return vec![Mismatch::MissingAuthor {
                        expected: self.surnames[surname].clone(),
                    }];
                }
            }
        }
        Vec::new()
    }
}

/// How many of `surnames`, from the front, this author answers for, or [`None`]
/// if the author is not the next one the key names.
fn credits(surnames: &[String], family: &str) -> Option<usize> {
    (1..=surnames.len()).find(|took| surnames[..*took].concat() == family)
}

/// `surname` with the `width` bytes at `at` replaced by `mark`, keeping the
/// case that was there: `Overa` guessed at `o` gives `Øvera`, not `øvera`.
fn respell(surname: &str, at: usize, width: usize, mark: char) -> String {
    let replaced = if surname[at..at + width].starts_with(char::is_uppercase) {
        mark.to_uppercase().to_string()
    } else {
        mark.to_string()
    };

    let mut spelling = String::with_capacity(surname.len() + replaced.len());
    spelling.push_str(&surname[..at]);
    spelling.push_str(&replaced);
    spelling.push_str(&surname[at + width..]);
    spelling
}

/// The surnames a citation would name a work by: its authors, or the editors of
/// one deposited without any, which is how crossref carries edited volumes.
///
/// Organisations are credited under a single `name` rather than a family name,
/// and are taken as they stand -- a key can name one as readily as a person.
pub fn families(work: &Work) -> Vec<&str> {
    [&work.author, &work.editor]
        .into_iter()
        .flatten()
        .find(|credited| !credited.is_empty())
        .map(|credited| credited.iter().filter_map(name).collect())
        .unwrap_or_default()
}

/// What to hold a contributor to: the family name of a person, the whole name
/// of an organisation.
fn name(contributor: &Contributor) -> Option<&str> {
    contributor
        .family
        .as_deref()
        .or(contributor.name.as_deref())
}

/// The year to hold a work to: the earliest date it was published under, since
/// that is the one a bibliography is built from.
pub fn published_year(work: &Work) -> Option<i32> {
    [
        &work.issued,
        &work.published_online,
        &work.published_print,
        &work.posted,
    ]
    .into_iter()
    .flatten()
    .filter_map(PartialDate::as_date_field)
    .map(|date| date.year())
    .min()
}

/// Why a work is not the one a [`CiteKey`] cites.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mismatch {
    /// the work carries no author or editor to check the key against
    NoAuthors,
    /// the key names someone else first
    NotFirstAuthor {
        /// the surname the key opens with
        expected: String,
        /// the work's first author
        found: String,
    },
    /// a surname the key names is credited nowhere after the ones before it
    MissingAuthor {
        /// the surname that is missing
        expected: String,
    },
    /// the key said `et al` of an author list with no one left over
    TooFewAuthors {
        /// how many the work credits
        found: usize,
    },
    /// the work is published too far from the year the key cites
    Year {
        /// the year the work was published
        found: i32,
        /// the year the key cites
        cited: i32,
    },
    /// the work carries no publication date at all
    Undated,
}

impl fmt::Display for Mismatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Mismatch::NoAuthors => write!(f, "credits nobody"),
            Mismatch::NotFirstAuthor { expected, found } => {
                write!(f, "opens with {found}, not {expected}")
            }
            Mismatch::MissingAuthor { expected } => write!(f, "does not credit {expected}"),
            Mismatch::TooFewAuthors { found } => {
                write!(f, "credits {found}, too few for an `et al`")
            }
            Mismatch::Year { found, cited } => write!(f, "published {found}, cited as {cited}"),
            Mismatch::Undated => write!(f, "carries no publication date"),
        }
    }
}

impl Serialize for Mismatch {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl FromStr for CiteKey {
    type Err = String;

    fn from_str(key: &str) -> Result<Self, Self::Err> {
        let key = key.trim().trim_start_matches('@');
        let tokens = tokens(key);

        let Some(at) = tokens.iter().rposition(|token| is_year(token)) else {
            return Err(format!("`{key}` carries no four digit year"));
        };
        // whatever follows the year disambiguates it -- the `a` of `Lindfors2022a`
        let year = tokens[at]
            .parse()
            .map_err(|_| format!("`{key}` has no year"))?;

        let mut surnames: Vec<String> = Vec::new();
        let mut et_al = false;
        let mut names = tokens[..at].iter().peekable();
        while let Some(name) = names.next() {
            match name.to_lowercase().as_str() {
                "etal" => et_al = true,
                "et" if names
                    .peek()
                    .is_some_and(|next| next.eq_ignore_ascii_case("al")) =>
                {
                    names.next();
                    et_al = true;
                }
                // initials and the odd stray letter name nobody
                _ if name.chars().count() < 2 => (),
                _ => surnames.push(name.clone()),
            }
        }

        if surnames.is_empty() {
            return Err(format!("`{key}` names no author"));
        }
        Ok(CiteKey {
            key: key.to_string(),
            surnames,
            year,
            et_al,
        })
    }
}

/// Whether a token is a year a work could have been published in, which is the
/// only way to tell one from a volume number or a page range.
fn is_year(token: &str) -> bool {
    matches!(token.parse::<i32>(), Ok(year) if (1400..=2200).contains(&year))
        && token.chars().count() == 4
}

/// Splits a key into the words it was built from, on the three boundaries a
/// key can have: a separator, a change between letters and digits, and the
/// camel case hump that is the only separator `LindforsJakobsen2022` has.
fn tokens(key: &str) -> Vec<String> {
    let mut tokens: Vec<String> = Vec::new();
    let mut previous: Option<char> = None;

    for letter in key.chars() {
        if !letter.is_alphanumeric() {
            previous = None;
            continue;
        }
        let starts_word = match previous {
            None => true,
            Some(before) => {
                before.is_ascii_digit() != letter.is_ascii_digit()
                    || (letter.is_uppercase() && before.is_lowercase())
            }
        };
        if starts_word {
            tokens.push(String::new());
        }
        tokens
            .last_mut()
            .expect("a word was started before it was pushed to")
            .push(letter);
        previous = Some(letter);
    }
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(key: &str) -> CiteKey {
        key.parse().unwrap_or_else(|err| panic!("`{key}`: {err}"))
    }

    #[test]
    fn a_key_is_read_as_surnames_and_a_year() {
        let parsed = key("@LindforsJakobsen2022");

        assert_eq!(["Lindfors", "Jakobsen"], parsed.surnames());
        assert_eq!(2022, parsed.year());
        assert!(!parsed.et_al());
        assert_eq!("LindforsJakobsen2022", parsed.key());
    }

    #[test]
    fn the_leading_at_is_optional() {
        assert_eq!(key("@Lindfors2022"), key("Lindfors2022"));
    }

    #[test]
    fn an_et_al_is_read_off_the_key_rather_than_taken_for_a_surname() {
        for written in [
            "@FloysandEtAl2021",
            "@Floysand_etal_2021",
            "@floysand:et:al:2021",
        ] {
            let parsed = key(written);
            assert_eq!(1, parsed.surnames().len(), "{written}");
            assert!(parsed.et_al(), "{written}");
            assert_eq!(2021, parsed.year(), "{written}");
        }
    }

    #[test]
    fn the_letter_that_disambiguates_two_keys_is_not_a_name() {
        let parsed = key("@Lindfors2022a");

        assert_eq!(["Lindfors"], parsed.surnames());
        assert_eq!(2022, parsed.year());
    }

    #[test]
    fn separators_split_a_key_the_way_camel_case_does() {
        assert_eq!(
            key("@lindfors_jakobsen_2022").surnames(),
            ["lindfors", "jakobsen"]
        );
        assert_eq!(key("@lindfors:2022:salmon").surnames(), ["lindfors"]);
    }

    #[test]
    fn a_key_that_names_nobody_or_no_year_is_rejected() {
        assert!(
            "@2022"
                .parse::<CiteKey>()
                .unwrap_err()
                .contains("names no author")
        );
        assert!(
            "@Lindfors"
                .parse::<CiteKey>()
                .unwrap_err()
                .contains("four digit year")
        );
        assert!("@".parse::<CiteKey>().is_err());
        assert!("@Lindfors123".parse::<CiteKey>().is_err());
        assert!("@Lindfors12345".parse::<CiteKey>().is_err());
    }

    #[test]
    fn folding_leaves_two_spellings_of_a_name_the_same_name() {
        assert_eq!("floysand", fold("Fløysand"));
        assert_eq!("floysand", fold("Floysand"));
        assert_eq!("floysand", fold("FLØYSAND"));
        assert_eq!("vandijk", fold("van Dijk"));
        assert_eq!("obrien", fold("O'Brien"));
        assert_eq!("gansser", fold("Gansser"));
        assert_eq!("grossmann", fold("Großmann"));
        assert_eq!("nino", fold("Niño"));
    }

    #[test]
    fn the_likeliest_spelling_is_guessed_first() {
        let spellings = key("@Floysand2021").spellings();

        assert_eq!(vec!["Fløysand".to_string()], spellings[0]);
        assert!(spellings.contains(&vec!["Floysånd".to_string()]));
        assert!(spellings.iter().all(|spelling| spelling.len() == 1));
    }

    #[test]
    fn a_guessed_spelling_keeps_the_case_it_replaced() {
        let spellings = key("@Overa2021").spellings();

        assert_eq!(vec!["Øvera".to_string()], spellings[0]);
        assert!(spellings.contains(&vec!["Overå".to_string()]));
    }

    #[test]
    fn every_guess_changes_one_letter_of_one_surname() {
        let spellings = key("@LindforsJakobsen2022").spellings();

        for spelling in &spellings {
            assert_eq!(2, spelling.len());
            let changed = spelling
                .iter()
                .zip(["Lindfors", "Jakobsen"])
                .filter(|(guess, written)| guess.as_str() != *written)
                .count();
            assert_eq!(1, changed, "{spelling:?}");
        }
        assert!(spellings.contains(&vec!["Lindfors".to_string(), "Jakøbsen".to_string()]));
    }

    #[test]
    fn a_surname_that_kept_its_diacritics_is_not_guessed_at() {
        assert!(key("@Fløysand2021").spellings().is_empty());
    }

    /// The verification half, against works assembled the way crossref sends
    /// them.
    mod against {
        use super::*;
        use crossref_client::DateParts;

        fn work(authors: &[&str], year: i32) -> Work {
            let json = serde_json::json!({
                "DOI": "10.0000/test",
                "author": authors
                    .iter()
                    .enumerate()
                    .map(|(at, family)| serde_json::json!({
                        "family": family,
                        "affiliation": [],
                        "sequence": if at == 0 { "first" } else { "additional" },
                    }))
                    .collect::<Vec<_>>(),
                "issued": { "date-parts": [[year]] },
            });
            serde_json::from_value(json).expect("a work built the way crossref sends one")
        }

        #[test]
        fn a_key_vouches_for_the_work_it_names() {
            let mismatches =
                key("@LindforsJakobsen2022").mismatches(&work(&["Lindfors", "Jakobsen"], 2022), 1);

            assert_eq!(Vec::<Mismatch>::new(), mismatches);
        }

        #[test]
        fn a_key_written_without_diacritics_still_vouches_for_the_work() {
            let mismatches = key("@FloysandEtAl2021")
                .mismatches(&work(&["Fløysand", "Hidle", "Overå"], 2021), 1);

            assert_eq!(Vec::<Mismatch>::new(), mismatches);
        }

        #[test]
        fn the_key_may_name_fewer_authors_than_the_work_credits() {
            let mismatches =
                key("@Lindfors2022").mismatches(&work(&["Lindfors", "Jakobsen"], 2022), 1);

            assert_eq!(Vec::<Mismatch>::new(), mismatches);
        }

        #[test]
        fn the_surnames_have_to_be_credited_in_the_order_the_key_gives_them() {
            let mismatches =
                key("@LindforsJakobsen2022").mismatches(&work(&["Jakobsen", "Lindfors"], 2022), 1);

            assert_eq!(
                vec![Mismatch::NotFirstAuthor {
                    expected: "Lindfors".to_string(),
                    found: "Jakobsen".to_string(),
                }],
                mismatches
            );
        }

        #[test]
        fn a_name_the_work_does_not_credit_is_reported_by_name() {
            let mismatches =
                key("@LindforsFeynman2022").mismatches(&work(&["Lindfors", "Jakobsen"], 2022), 1);

            assert_eq!(
                vec![Mismatch::MissingAuthor {
                    expected: "Feynman".to_string()
                }],
                mismatches
            );
        }

        #[test]
        fn an_author_may_answer_for_a_surname_camel_case_could_not_keep_together() {
            let mismatches = key("@VanDijk2020").mismatches(&work(&["van Dijk"], 2020), 1);
            assert_eq!(Vec::<Mismatch>::new(), mismatches);

            let mismatches = key("@McDonald2020").mismatches(&work(&["McDonald"], 2020), 1);
            assert_eq!(Vec::<Mismatch>::new(), mismatches);

            // ... and the same shape read the other way round still works
            let mismatches = key("@LeNguyen2020").mismatches(&work(&["Le", "Nguyen"], 2020), 1);
            assert_eq!(Vec::<Mismatch>::new(), mismatches);
        }

        #[test]
        fn an_et_al_wants_someone_left_over() {
            let mismatches = key("@FloysandEtAl2021").mismatches(&work(&["Fløysand"], 2021), 1);

            assert_eq!(vec![Mismatch::TooFewAuthors { found: 1 }], mismatches);
        }

        #[test]
        fn a_year_either_side_is_the_same_publication_and_two_is_not() {
            let key = key("@Lindfors2022");

            assert!(key.mismatches(&work(&["Lindfors"], 2021), 1).is_empty());
            assert!(key.mismatches(&work(&["Lindfors"], 2023), 1).is_empty());
            assert_eq!(
                vec![Mismatch::Year {
                    found: 2024,
                    cited: 2022
                }],
                key.mismatches(&work(&["Lindfors"], 2024), 1)
            );
            // and the window is the caller's to widen
            assert!(key.mismatches(&work(&["Lindfors"], 2024), 2).is_empty());
        }

        #[test]
        fn every_reason_a_work_is_wrong_is_reported_at_once() {
            let mismatches = key("@LindforsFeynman2022").mismatches(&work(&["Lindfors"], 2019), 1);

            assert_eq!(
                vec![
                    Mismatch::Year {
                        found: 2019,
                        cited: 2022
                    },
                    Mismatch::MissingAuthor {
                        expected: "Feynman".to_string()
                    },
                ],
                mismatches
            );
        }

        #[test]
        fn a_work_crossref_carries_no_names_or_dates_for_is_a_mismatch_not_a_match() {
            let mut bare = work(&["Lindfors"], 2022);
            bare.author = None;
            bare.issued = None;

            let mismatches = key("@Lindfors2022").mismatches(&bare, 1);

            assert!(mismatches.contains(&Mismatch::Undated), "{mismatches:?}");
            assert!(mismatches.contains(&Mismatch::NoAuthors), "{mismatches:?}");
        }

        #[test]
        fn an_edited_volume_is_checked_against_its_editors() {
            let mut volume = work(&["Fløysand"], 2021);
            volume.editor = volume.author.take();

            assert!(key("@Floysand2021").mismatches(&volume, 1).is_empty());
        }

        /// [`Work::citekey`] writes the key this reads, so a key it wrote has
        /// to lead back to the work it was written for -- including the `&`
        /// it joins two authors with and the `EtAl` it cuts three off at.
        #[test]
        fn a_key_the_library_writes_reads_back_to_the_work_it_was_written_for() {
            for (authors, expected) in [
                (&["Lindfors"][..], "Lindfors2022"),
                (&["Lindfors", "Jakobsen"], "Lindfors&Jakobsen2022"),
                (&["Fløysand", "Hidle", "Overå"], "FløysandEtAl2022"),
            ] {
                let work = work(authors, 2022);
                let written = work.citekey().expect("a work with authors and a year");

                assert_eq!(expected, written);
                assert_eq!(Vec::<Mismatch>::new(), key(&written).mismatches(&work, 0));
            }
        }

        #[test]
        fn the_earliest_date_a_work_carries_is_the_one_it_is_cited_by() {
            let mut online_first = work(&["Lindfors"], 2022);
            online_first.issued = None;
            online_first.published_online = Some(PartialDate {
                date_parts: DateParts(vec![vec![Some(2021)]]),
            });
            online_first.published_print = Some(PartialDate {
                date_parts: DateParts(vec![vec![Some(2022)]]),
            });

            assert!(key("@Lindfors2021").mismatches(&online_first, 0).is_empty());
        }
    }
}
