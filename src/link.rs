use url::Url;

#[derive(Clone, Debug)]
pub struct Link {
    /// Used in conjunction with rel="me" linking, when we want the link to be present
    /// to verify identity, but not display it.
    pub hidden: bool,
    pub label: Option<String>,
    /// Indicates rel="me" linking (https://microformats.org/wiki/rel-me)
    pub rel_me: bool,
    pub url: Url
}

impl Link {
    pub fn new(
        hidden: bool,
        label: Option<String>,
        rel_me: bool,
        url: Url
    ) -> Link {
        Link {
            hidden,
            label,
            rel_me,
            url
        }
    }

    /// Serializes the link as an eno field, ready to be written to a collection,
    /// playlist or video manifest.
    pub fn to_eno(&self) -> String {
        let mut field = String::from("link:\n");

        if let Some(label) = &self.label {
            field.push_str(&format!("label = {}\n", label));
        }

        field.push_str(&format!("url = {}\n", self.url));

        if self.rel_me {
            if self.hidden {
                field.push_str(&format!("verification = rel-me-hidden\n"));
            } else {
                field.push_str(&format!("verification = rel-me\n"));
            }
        }

        field
    }
}