use std::{
    cmp::Ordering,
    fmt::{Display, Formatter},
};

use itertools::Itertools;

use riichi_elements::prelude::*;
use riichi_decomp_table::{
    WaitingKind
};

pub(crate) type RegularWaitGroups = nanovec::NanoDequeBit<u32, u8, 8>;

/// A regular waiting pattern and hand decomposition of a waiting hand.
///
/// For a regular `3N+1` hand, this includes:
///
/// - Exactly `N` hand groups, including at most one incomplete (waiting) hand group.
///   The complete hand groups can be iterated using [`RegularWait::groups()`].
/// - A complete pair (雀頭) or a [Tanki (単騎)][Tanki] waiting pattern (= incomplete pair).
///
/// Note that there is exactly one waiting pattern, either a [Tanki] or an incomplete group.
///
/// [Tanki]: WaitingKind::Tanki
///
/// ## Optional `serde` support
///
/// Custom format, serialization only.
/// Example:
/// ```json
/// {
///     "groups": [
///         {"type": "Koutsu", "tile": "1m"},
///         {"type": "Koutsu", "tile": "2m"},
///         {"type": "Shuntsu", "tile": "7m"}
///     ],
///     "pair": "6z",
///     "kind": "Shanpon",
///     "pattern": "7z",
///     "waiting": "7z"
/// }
/// ```
/// 
#[derive(Copy, Clone, Debug)]
pub struct RegularWait {
    /// complete groups in this hand decomposition.
    pub(crate) raw_groups: RegularWaitGroups,

    /// The complete pair (excluding Tanki).
    pub pair: Option<Tile>,

    /// The detailed kind of the waiting pattern.
    pub waiting_kind: WaitingKind,

    /// The smallest tile in the waiting pattern.
    ///
    /// Examples:
    /// - 12m wait 3m => 1m
    /// - 34m wait 2m => 3m
    /// - 79p wait 8p => 7p
    /// - 3s wait 3s => 3s
    pub pattern_tile: Tile,

    /// The waiting tile (duh).
    pub waiting_tile: Tile,
}

impl RegularWait {
    /// Construct from components. This is only used for testing purposes.
    #[cfg(test)]
    pub fn new(groups: &[HandGroup], pair: Option<Tile>,
               waiting_kind: WaitingKind, pattern_tile: Tile, waiting_tile: Tile) -> Self {
        Self {
            raw_groups: groups.iter().map(|g| g.packed()).collect(),
            pair,
            waiting_kind,
            pattern_tile,
            waiting_tile,
        }
    }

    /// Iterate all complete groups in this hand decomposition.
    pub fn groups(&self) -> impl Iterator<Item = HandGroup> + use<> {
        self.raw_groups.map(|x| HandGroup::from_packed(x).unwrap())
    }

    /// Since groups are unordered, comparison must be applied to sorted groups.
    /// Here we don't need to convert back to `HandGroup` --- raw (packed) is sufficient.
    fn sorted_raw_groups(&self) -> [u8; 4] {
        let mut result = self.raw_groups.packed().to_le_bytes();
        sortnet::sortnet4(&mut result);
        result
    }

    /// Returns whether this waiting pattern has a pair (complete or incomplete).
    pub fn has_pair_or_tanki(&self) -> bool {
        self.pair.is_some() || self.waiting_kind == WaitingKind::Tanki
    }

    /// Returns the tile of the pair (complete or incomplete).
    pub fn pair_or_tanki(&self) -> Option<Tile> {
        self.pair.or_else(||
            (self.waiting_kind == WaitingKind::Tanki).then_some(self.waiting_tile))
    }

    /// Returns whether this waiting pattern is part of a double-sided wait, i.e.
    /// 45m waits 3m or 6m (両面). This mostly affects scoring and the Pinfu Yaku.
    ///
    /// The reason this is separate is because we overloaded the "Ryanmen" term in [`WaitingKind`]
    /// to broaden its scope to any 2 consecutive numerals, which simplified handling.
    ///
    /// For game rule purposes, this method should be used.
    pub fn is_true_ryanmen(&self) -> bool {
        matches!((self.waiting_kind, self.pattern_tile.normal_num()),
            (WaitingKind::RyanmenLow, 2..=7) |  // (1)23 ..= (6)78 ; excluding (7)89
            (WaitingKind::RyanmenHigh, 2..=7) |  // 23(4) ..= 78(9) ; excluding 12(3)
            (WaitingKind::RyanmenBoth, _)
        )
    }
}

// NOTE: Comparison of two waiting patterns is non-trivial because `groups` is logically an
// unordered collection. Fortunately we only need to trivially sort.

impl PartialEq<Self> for RegularWait {
    fn eq(&self, other: &Self) -> bool {
        self.sorted_raw_groups() == other.sorted_raw_groups()
            && self.pair == other.pair
            && self.waiting_kind == other.waiting_kind
            && self.pattern_tile == other.pattern_tile
            && self.waiting_tile == other.waiting_tile
    }
}

impl Eq for RegularWait {}

impl PartialOrd<Self> for RegularWait {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RegularWait {
    fn cmp(&self, other: &Self) -> Ordering {
        let o =
            self.sorted_raw_groups().cmp(&other.sorted_raw_groups());
        if o != Ordering::Equal { return o; }
        let o = self.pair.cmp(&other.pair);
        if o != Ordering::Equal { return o; }
        let o = self.waiting_kind.cmp(&other.waiting_kind);
        if o != Ordering::Equal { return o; }
        let o = self.pattern_tile.cmp(&other.pattern_tile);
        if o != Ordering::Equal { return o; }
        self.waiting_tile.cmp(&other.waiting_tile)
    }
}

impl Display for RegularWait {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use WaitingKind::*;
        write!(f, "{}", self.groups().sorted().map(|g| g.to_string()).join(" "))?;
        if let Some(pair) = self.pair {
            write!(f, " {}{}", pair.num(), pair)?;
        }
        let p = self.pattern_tile;
        let t = self.waiting_tile;
        match self.waiting_kind {
            Tanki =>
                write!(f, " {}+{}", p.num(), t),
            Shanpon =>
                write!(f, " {}{}+{}", p.num(), p.num(), t),
            Kanchan =>
                write!(f, " {}{}+{}", p.num(), p.succ2().unwrap().num(), t),
            RyanmenHigh | RyanmenLow | RyanmenBoth =>
                write!(f, " {}{}+{}", p.num(), p.succ().unwrap().num(), t),
        }
    }
}

// End of test-only comparisons.

#[cfg(feature = "serde")]
mod regular_wait_serde {
    use serde::ser::SerializeStruct;
    use serde::Serializer;
    use super::*;
    impl serde::Serialize for RegularWait {
        fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error> where S: Serializer {
            let mut st = s.serialize_struct("RegularWait", 5)?;
            st.serialize_field("groups", &self.groups().sorted().collect_vec())?;
            st.serialize_field("pair", &self.pair)?;
            st.serialize_field("kind", self.waiting_kind.as_ref())?;
            st.serialize_field("pattern", &self.pattern_tile)?;
            st.serialize_field("waiting", &self.waiting_tile)?;
            st.end()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use HandGroup::{Koutsu, Shuntsu};

    #[allow(unused)]
    fn k(str: &str) -> HandGroup { Koutsu(t!(str)) }
    #[allow(unused)]
    fn s(str: &str) -> HandGroup { Shuntsu(t!(str)) }

    #[cfg(feature = "serde")]
    mod serde_tests {
        use assert_json_diff::assert_json_eq;
        use WaitingKind::*;
        use RegularWait as W;
        use super::*;

        #[test]
        fn serialize_regular_wait() {
            let w = W::new(&[k("1m"), k("2m"), s("7m")], Some(t!("6z")),
                           Shanpon, t!("7z"), t!("7z"));
            let json = serde_json::json!({
                "groups": [
                    {"type": "Koutsu", "tile": "1m"},
                    {"type": "Koutsu", "tile": "2m"},
                    {"type": "Shuntsu", "tile": "7m"}
                ],
                "pair": "6z",
                "kind": "Shanpon",
                "pattern": "7z",
                "waiting": "7z"
            });
            let serialized = serde_json::to_value(w).unwrap();
            assert_json_eq!(serialized, json);
        }
    }
}
