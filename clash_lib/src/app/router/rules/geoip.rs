use tracing::debug;

use super::RuleMatcher;
use crate::{common::mmdb::MmdbLookup, session::Session};

#[derive(Clone)]
pub struct GeoIP {
    pub target: String,
    pub country_code: String,
    pub no_resolve: bool,
    pub mmdb: MmdbLookup,
}

impl std::fmt::Display for GeoIP {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GeoIP({} - {})", self.target, self.country_code)
    }
}

impl RuleMatcher for GeoIP {
    fn apply(&self, sess: &Session) -> bool {
        let ip = sess.resolved_ip.or(sess.destination.ip());

        if let Some(ip) = ip {
            match self.mmdb.lookup_country(ip) {
                Ok(country) => country.country_code == self.country_code,
                Err(e) => {
                    debug!("GeoIP lookup failed: {}", e);
                    false
                }
            }
        } else {
            false
        }
    }

    fn target(&self) -> &str {
        self.target.as_str()
    }

    fn payload(&self) -> String {
        self.country_code.clone()
    }

    fn type_name(&self) -> &str {
        "GeoIP"
    }

    fn should_resolve_ip(&self) -> bool {
        !self.no_resolve
    }
}
