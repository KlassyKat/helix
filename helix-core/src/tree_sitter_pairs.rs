#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(unused)]
// From insert_newline we ask the question are we between a pair?
// This file answers with the ranges of the closest surrounding pair
// In insert_newline we then ask Is the end of the start range 1 less than the start of the end range
// If yes, we do the bracket pair behavior of opening an indented line and putting the closing pair on the next line matching the indent of the open pair

// Query doc pairs
// NOTE: Could use this feature to pair match '<' and '|' in contexts where they are used as surrounding blocks and type them both in like how brackets are done

use std::{collections::HashMap, ops::Range};

use ropey::RopeSlice;
use tree_house::{
    tree_sitter::{Capture, InactiveQueryCursor, Node, Query, RopeInput},
    TREE_SITTER_MATCH_LIMIT,
};

use crate::Syntax;

// TODO:
type PairOpen = Range<u32>;
type PairClose = Range<u32>;

struct PairQuery {
    query: Query,
    pair_open_capture: Option<Capture>,
    pair_close_capture: Option<Capture>,
}

struct PairCapture<'a> {
    open: Node<'a>,
    close: Node<'a>,
}

// Are we between a pair?
fn is_between_ts_pair(cursor: usize, range: (PairOpen, PairClose)) {
    range.0.end < cursor && range.1.start
}

// What is the closest pair?
fn find_surrounding_pair(
    cursor: usize,
    pair_ranges: &[(PairOpen, PairClose)],
) -> Option<(PairOpen, PairClose)> {
    pair_ranges
        .iter()
        .filter(|(open, close)| (open.end as usize) < cursor && (close.start as usize) >= cursor)
        .min_by_key(|(open, close)| (open.end as usize) - cursor)
        .cloned()
}

// Where are the pairs?
fn get_pair_ranges(pairs: &HashMap<usize, PairCapture>) -> Vec<(PairOpen, PairClose)> {
    pairs
        .values()
        .map(|v| (v.open.byte_range(), v.close.byte_range()))
        .collect::<Vec<(PairOpen, PairClose)>>()
}

// What are the pairs?
// Find treeitter pairs -> HashMap<NodeID, PairCapture>
pub fn query_treesitter_pairs<'a>(
    query: &PairQuery,
    syntax: &'a Syntax,
    range: std::ops::Range<u32>,
    text: RopeSlice<'_>,
) -> HashMap<usize, PairCapture<'a>> {
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
            (None, Some(_)) => log::error!(
                "Invalid pair capture: @pair.close requires an accompanying @pair.open."
            ),
            (Some(_), None) => log::error!(
                "Invalid pair capture: @pair.open requires an accompanying @pair.close."
            ),
            (Some(open), Some(close)) => {
                let replaced = pair_captures.insert(
                    open.id(),
                    PairCapture {
                        open: open.clone(),
                        close: close.clone(),
                    },
                );
                // TODO: Remove this
                if replaced.is_some() {
                    log::debug!("Tree sitter pair overwritten. This should not occur.");
                }
            }
        }
    }
    pair_captures
}

struct TsPairBuilder {}
