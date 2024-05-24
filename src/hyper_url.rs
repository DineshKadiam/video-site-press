use std::fmt::{Display, Formatter};

use url::Url;

/// Urls can be user-provided with or without trailing slash
/// ("https://example.com/subdir" "https://example.com/subdir/"), but for
/// further url construction through the [url] crate the trailing slash is
/// significant (https://docs.rs/url/latest/url/struct.Url.html#method.join),
/// hence we store both representation and a working copy of the parsed url.
#[derive(Clone, Debug)]
pub struct HyperUrl {
    pub representation: String,
    pub value: Url
}

impl HyperUrl {
    /// Delegates parsing to the url crate but beforehand ensures a trailing
    /// slash so further url construction is performed correctly.
    pub fn parse(url: &str) -> Result<HyperUrl, String> {
        let parsed_url = match url.ends_with('/') {
            true => Url::parse(url),
            false => Url::parse(&format!("{url}/"))
        };

        match parsed_url {
            Ok(value) => {
                Ok(HyperUrl {
                    representation: url.to_owned(),
                    value
                })
            }
            Err(err) => Err(err.to_string())
        }
    }
}

impl Display for HyperUrl {
    fn fmt(&self, formatter: &mut Formatter) -> std::fmt::Result {
        let representation = &self.representation;
        write!(formatter, "{representation}")
    }
}
