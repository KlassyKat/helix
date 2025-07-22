#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(unused)]
// From insert_newline we ask the question are we between a pair?
// This file answers with the ranges of the closest surrounding pair
// In insert_newline we then ask Is the end of the start range 1 less than the start of the end range
// If yes, we do the bracket pair behavior of opening an indented line and putting the closing pair on the next line matching the indent of the open pair

// Query doc pairs

use std::collections::HashMap;

use ropey::RopeSlice;
use tree_house::{
    tree_sitter::{Capture, InactiveQueryCursor, Node, Query, RopeInput},
    TREE_SITTER_MATCH_LIMIT,
};

use crate::Syntax;

struct PairQuery {
    query: Query,
    pair_open_capture: Option<Capture>,
    pair_close_capture: Option<Capture>,
}

struct PairCapture {
    open: usize,
    close: usize,
}

// Find closest surrounding pair -> (Range, Range)

// Find treeitter pairs -> HashMap<NodeID, PairCapture>
fn query_treesitter_pairs<'a>(
    query: &PairQuery,
    syntax: &Syntax,
    range: std::ops::Range<u32>,
    text: RopeSlice<'a>,
) {
    let mut pair_captures: HashMap<usize, PairCapture> = HashMap::new();

    let mut cursor = InactiveQueryCursor::new(range, TREE_SITTER_MATCH_LIMIT).execute_query(
        &query.query,
        &syntax.tree().root_node(),
        RopeInput::new(text),
    );

    while let Some(m) = cursor.next_match() {
        let mut pair_open: Option<&Node> = None;
        let mut pair_close: Option<&Node> = None;

        for matched_node in m.matched_nodes() {
            let capture = Some(matched_node.capture);

            if capture == query.pair_open_capture {
                if pair_open.is_some() {
                    log::error!("Invalid pair capture: Encountered more than one @pair.open in the same match.");
                } else {
                    pair_open = Some(&matched_node.node);
                }
            } else if capture == query.pair_close_capture {
                if pair_close.is_some() {
                    log::error!("Invalid pair capture: Encountered more than one @pair.close in the same match.");
                } else {
                    pair_close = Some(&matched_node.node);
                }
            } else {
                continue;
            }
        }

        match (pair_open, pair_close) {
            (None, None) => continue,
            (None, Some(_)) => log::error!("Invalid pair capture: @pair.close requires an accompanying @pair.open."),
            (Some(_), None) => log::error!("Invalid pair capture: @pair.open requires an accompanying @pair.close."),
            (Some(open), Some(close)) => {
                if let Some(existing) = pair_captures.insert(open.id(), PairCapture { open: (), close: () })
            },
        }
    }
}
