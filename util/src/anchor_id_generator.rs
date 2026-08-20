// SPDX-FileCopyrightText: 2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::collections::HashSet;

use slug::slugify;

use super::deduplicate_id;

/// A stateful anchor id generator, computing guaranteed unique ids from slugs
/// and headings (etc.) for use within a page.
pub struct AnchorIdGenerator {
    previous_ids: HashSet<String>
}

impl AnchorIdGenerator {
    /// Generates a new, unique id, starting with an arbitrary, unescaped
    /// string as input. In most cases this will return just a sluggified
    /// version of the input, but in case of conflict it will deduplicate the
    /// id further by appending a number suffix.
    pub fn generate_for_arbitrary(&mut self, input: &str) -> String {
        let mut id = slugify(input);

        while self.previous_ids.contains(&id) {
            id = deduplicate_id(&id);
        }

        self.previous_ids.insert(id.clone());

        id
    }

    /// Generates a new, unique id, starting with an already escaped slug as
    /// input. In most cases this will return the input 1:1, but in case of
    /// conflict it will deduplicate the input by appending a number suffix.
    pub fn generate_for_slug(&mut self, slug: &str) -> String {
        let mut id = slug.to_string();

        while self.previous_ids.contains(&id) {
            id = deduplicate_id(&id);
        }

        self.previous_ids.insert(id.clone());

        id
    }

    pub fn new() -> AnchorIdGenerator {
        AnchorIdGenerator {
            previous_ids: HashSet::new()
        }
    }
}
