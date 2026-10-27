//! Host-side embed configuration. Parent origins are fixed by deployment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EmbedConfig {
    pub parent_origin: String,
    pub client_version: String,
}
impl EmbedConfig {
    pub fn validate(&self, host_origin: &str) -> bool {
        url::Url::parse(&self.parent_origin).is_ok_and(|u| {
            u.origin().ascii_serialization() == self.parent_origin
                && self.parent_origin != host_origin
                && (u.scheme() == "https"
                    || u.scheme() == "http"
                        && matches!(u.host_str(), Some("localhost" | "127.0.0.1")))
        }) && self.client_version == "1.0"
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn same_origin_is_refused() {
        let c = EmbedConfig {
            parent_origin: "https://run.example.test".into(),
            client_version: "1.0".into(),
        };
        assert!(!c.validate("https://run.example.test"));
        assert!(c.validate("https://catalog.example.test"));
    }
}
