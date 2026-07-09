use std::collections::{HashMap, HashSet};

use dioxus::prelude::*;
use serde::Serialize;

#[derive(Serialize)]
pub(super) struct FormatArgs {
    pub input: String,
    pub repair: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct JsonMatch {
    pub path: String,
    pub is_key: bool,
}

#[derive(Props, Clone, PartialEq)]
pub(crate) struct HighlightProps {
    pub text: String,
    pub match_idx: Option<usize>,
    pub active_global_idx: usize,
    pub query: String,
    pub default_class: String,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct SearchResultMap {
    pub list: Vec<JsonMatch>,
    pub path_to_match: HashMap<String, (usize, bool)>,
    pub active_ancestor_paths: HashSet<String>,
}

#[derive(Clone, Copy)]
pub(super) struct SearchContext {
    pub query: Signal<String>,
    pub search_results: Memo<SearchResultMap>,
    pub active_index: Signal<usize>,
}
