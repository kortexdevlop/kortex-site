use crate::{api::Query, models::ToolList};
use std::collections::{BTreeSet, HashSet};
#[derive(Clone, Default)]
pub struct Store {
    pub query: Query,
    pub page: Option<ToolList>,
    seen: HashSet<String>,
    cursors: HashSet<String>,
    pub loaded: usize,
    pub known_categories: BTreeSet<String>,
}
impl Store {
    pub fn reset(&mut self, query: Query) {
        let known_categories = std::mem::take(&mut self.known_categories);
        *self = Self {
            known_categories,
            query,
            ..Self::default()
        };
    }
    pub fn accept(&mut self, page: ToolList) -> Result<(), String> {
        let mut ids = HashSet::new();
        if page
            .items
            .iter()
            .any(|t| self.seen.contains(&t.id) || !ids.insert(t.id.clone()))
            || self.seen.len() + page.items.len() > 10000
            || page
                .next_cursor
                .as_ref()
                .is_some_and(|c| self.cursors.contains(c))
        {
            return Err("Pagination incohérente ; relancez la recherche.".into());
        }
        self.known_categories
            .extend(page.items.iter().map(|t| t.category.clone()));
        self.seen.extend(ids);
        if let Some(c) = &page.next_cursor {
            self.cursors.insert(c.clone());
        }
        self.loaded += page.items.len();
        self.page = Some(page);
        Ok(())
    }
}
