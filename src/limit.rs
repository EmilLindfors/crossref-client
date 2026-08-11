//! Client side rate limiting.
//!
//! Crossref publishes the budget it grants a client on every response
//! (`x-rate-limit-limit` and `x-rate-limit-interval`) and
//! [expects clients to stay inside it](https://api.crossref.org/swagger-ui/index.html#/Etiquette);
//! going over earns a `429`. [`Limiter`] spaces requests evenly across the
//! reported interval and re-reads the budget from every response, so a client
//! that moves between the anonymous and the polite pool adapts on its own.

use reqwest::header::HeaderMap;
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::Duration;
use tokio::time::Instant;

/// The request budget crossref grants a client.
///
/// Read back from a live client with [`Crossref::rate_limit`](crate::Crossref::rate_limit).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RateLimit {
    /// how many requests are allowed within `interval`
    pub limit: u32,
    /// the window that `limit` applies to
    pub interval: Duration,
}

impl RateLimit {
    /// The budget assumed until crossref has reported one.
    ///
    /// Deliberately below what either pool actually grants. The first response
    /// carries the real figure, so guessing low costs nothing but a slightly
    /// slower first handful of requests, whereas guessing high costs a `429`.
    pub const CONSERVATIVE: Self = Self {
        limit: 5,
        interval: Duration::from_secs(1),
    };

    /// The gap to leave between two requests to spend the budget evenly.
    pub fn spacing(&self) -> Duration {
        self.interval / self.limit.max(1)
    }

    /// Reads the budget crossref reported on a response.
    ///
    /// Returns [`None`] unless both headers are present and understood.
    pub fn from_headers(headers: &HeaderMap) -> Option<Self> {
        let limit = header(headers, "x-rate-limit-limit")?.trim().parse().ok()?;
        let interval = parse_interval(header(headers, "x-rate-limit-interval")?)?;
        Some(Self { limit, interval })
    }
}

impl Default for RateLimit {
    fn default() -> Self {
        Self::CONSERVATIVE
    }
}

/// Reads a header as a `str`, if it is present and not binary.
fn header<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name)?.to_str().ok()
}

/// Parses the `1s` form crossref uses for `x-rate-limit-interval`.
fn parse_interval(raw: &str) -> Option<Duration> {
    let raw = raw.trim();
    let unit_at = raw.find(|c: char| !c.is_ascii_digit())?;
    let (amount, unit) = raw.split_at(unit_at);
    let amount: u64 = amount.parse().ok()?;
    match unit {
        "ms" => Some(Duration::from_millis(amount)),
        "s" => Some(Duration::from_secs(amount)),
        "m" => Some(Duration::from_secs(amount.checked_mul(60)?)),
        "h" => Some(Duration::from_secs(amount.checked_mul(3_600)?)),
        _ => None,
    }
}

/// How long crossref asked us to wait, from a `retry-after` header.
///
/// Only the delay-seconds form is understood; the HTTP-date form yields
/// [`None`] and the caller falls back to its own backoff.
pub(crate) fn retry_after(headers: &HeaderMap) -> Option<Duration> {
    let seconds = header(headers, "retry-after")?.trim().parse().ok()?;
    Some(Duration::from_secs(seconds))
}

/// Paces requests so a client stays inside the budget crossref granted it.
///
/// Shared between every clone of a [`Crossref`](crate::Crossref), so concurrent
/// callers queue against one another rather than each getting a full budget.
#[derive(Debug)]
pub(crate) struct Limiter {
    state: Mutex<State>,
}

#[derive(Debug)]
struct State {
    /// the budget, as last reported by crossref
    rate: RateLimit,
    /// the earliest instant at which the next request may leave
    next_slot: Option<Instant>,
    /// the pool crossref sorted the last request into, from `x-api-pool`
    pool: Option<String>,
}

impl Limiter {
    /// A limiter that assumes `rate` until crossref reports otherwise.
    pub(crate) fn new(rate: RateLimit) -> Self {
        Self {
            state: Mutex::new(State {
                rate,
                next_slot: None,
                pool: None,
            }),
        }
    }

    /// Waits until the next request fits inside the budget.
    ///
    /// Claims the slot before sleeping, so concurrent callers spread out
    /// instead of all waking into the same one.
    pub(crate) async fn acquire(&self) {
        let slot = {
            let mut state = self.lock();
            let now = Instant::now();
            let slot = state.next_slot.map_or(now, |slot| slot.max(now));
            state.next_slot = Some(slot + state.rate.spacing());
            slot
        };
        tokio::time::sleep_until(slot).await;
    }

    /// Re-reads the budget and the pool from a response.
    pub(crate) fn observe(&self, headers: &HeaderMap) {
        let rate = RateLimit::from_headers(headers);
        let pool = header(headers, "x-api-pool").map(str::to_owned);

        let mut state = self.lock();
        if let Some(rate) = rate {
            state.rate = rate;
        }
        if pool.is_some() {
            state.pool = pool;
        }
    }

    /// Holds every request back for at least `delay`.
    ///
    /// A `429` means the budget we believed in was wrong, so the pause has to
    /// apply to everything sharing this limiter, not only to the request that
    /// happened to be rejected.
    pub(crate) fn back_off(&self, delay: Duration) {
        let mut state = self.lock();
        let resume = Instant::now() + delay;
        state.next_slot = Some(state.next_slot.map_or(resume, |slot| slot.max(resume)));
    }

    /// The budget crossref last reported.
    pub(crate) fn rate(&self) -> RateLimit {
        self.lock().rate
    }

    /// The pool crossref last sorted a request into.
    pub(crate) fn pool(&self) -> Option<String> {
        self.lock().pool.clone()
    }

    /// The lock is only ever held across arithmetic, so a panic cannot leave
    /// the state half-updated and poisoning carries no information.
    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::{HeaderMap, HeaderValue};

    fn headers(pairs: &[(&'static str, &str)]) -> HeaderMap {
        let mut headers = HeaderMap::new();
        for (name, value) in pairs {
            headers.insert(*name, HeaderValue::from_str(value).unwrap());
        }
        headers
    }

    #[test]
    fn the_budget_is_read_from_the_headers_crossref_sends() {
        let rate = RateLimit::from_headers(&headers(&[
            ("x-rate-limit-limit", "50"),
            ("x-rate-limit-interval", "1s"),
        ]));

        assert_eq!(
            Some(RateLimit {
                limit: 50,
                interval: Duration::from_secs(1)
            }),
            rate
        );
    }

    #[test]
    fn a_half_reported_budget_is_no_budget() {
        assert_eq!(
            None,
            RateLimit::from_headers(&headers(&[("x-rate-limit-limit", "50")]))
        );
        assert_eq!(
            None,
            RateLimit::from_headers(&headers(&[
                ("x-rate-limit-limit", "50"),
                ("x-rate-limit-interval", "1 fortnight"),
            ]))
        );
    }

    #[test]
    fn every_interval_unit_crossref_might_use_parses() {
        assert_eq!(Some(Duration::from_millis(500)), parse_interval("500ms"));
        assert_eq!(Some(Duration::from_secs(1)), parse_interval("1s"));
        assert_eq!(Some(Duration::from_secs(120)), parse_interval("2m"));
        assert_eq!(Some(Duration::from_secs(3_600)), parse_interval("1h"));
        assert_eq!(None, parse_interval("s"));
        assert_eq!(None, parse_interval("12"));
    }

    #[test]
    fn spacing_divides_the_interval_across_the_budget() {
        let rate = RateLimit {
            limit: 50,
            interval: Duration::from_secs(1),
        };
        assert_eq!(Duration::from_millis(20), rate.spacing());
    }

    #[test]
    fn a_zero_budget_does_not_divide_by_zero() {
        let rate = RateLimit {
            limit: 0,
            interval: Duration::from_secs(1),
        };
        assert_eq!(Duration::from_secs(1), rate.spacing());
    }

    #[test]
    fn retry_after_seconds_are_understood_and_dates_are_not() {
        assert_eq!(
            Some(Duration::from_secs(30)),
            retry_after(&headers(&[("retry-after", "30")]))
        );
        assert_eq!(
            None,
            retry_after(&headers(&[("retry-after", "Wed, 21 Oct 2015 07:28:00 GMT")]))
        );
    }

    #[tokio::test(start_paused = true)]
    async fn requests_are_spaced_across_the_interval() {
        let limiter = Limiter::new(RateLimit {
            limit: 4,
            interval: Duration::from_secs(1),
        });
        let start = Instant::now();

        for _ in 0..4 {
            limiter.acquire().await;
        }

        // the first request leaves immediately, the other three are spaced 250ms
        assert_eq!(Duration::from_millis(750), start.elapsed());
    }

    #[tokio::test(start_paused = true)]
    async fn a_backoff_holds_back_the_next_request() {
        let limiter = Limiter::new(RateLimit::CONSERVATIVE);
        limiter.acquire().await;
        let start = Instant::now();

        limiter.back_off(Duration::from_secs(5));
        limiter.acquire().await;

        assert_eq!(Duration::from_secs(5), start.elapsed());
    }

    #[test]
    fn observing_a_response_replaces_the_assumed_budget() {
        let limiter = Limiter::new(RateLimit::CONSERVATIVE);

        limiter.observe(&headers(&[
            ("x-rate-limit-limit", "50"),
            ("x-rate-limit-interval", "1s"),
            ("x-api-pool", "polite"),
        ]));

        assert_eq!(50, limiter.rate().limit);
        assert_eq!(Some("polite".to_string()), limiter.pool());

        // a response without the headers leaves what we already know alone
        limiter.observe(&HeaderMap::new());

        assert_eq!(50, limiter.rate().limit);
        assert_eq!(Some("polite".to_string()), limiter.pool());
    }
}
