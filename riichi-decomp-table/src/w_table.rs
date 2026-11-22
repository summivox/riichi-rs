use nanovec::NanoStackRadix;
use rustc_hash::FxHashMap as HashMap;

use crate::utils::*;
use details::*;

/*
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct WEntry {
    pub big: u64,
    pub tenpai_type: u8,
    pub tenpai_num: u8,
    pub anchor_num: u8,
    pub has_pair: bool,
}
// pub type WTable = HashMap<u32, Vec<WEntry>>;
 */

pub type WTable = HashMap<u32, WAlts>;
pub type WTableStatic = phf::Map<u32, u64>;

pub type WAlts = NanoStackRadix<u64, 55>;

/// Since the WTable is statically determinable, we can check its number of keys to make sure that
/// we have generated the correct table.
pub const W_TABLE_NUM_KEYS: usize = 66913;

pub fn make_w_table(c_table: &super::c_table::CTable) -> WTable {
    let mut w_table = WTable::with_capacity_and_hasher(
        W_TABLE_NUM_KEYS, Default::default());
    for key in c_table.keys() {
        make_waiting_for_c_entry(&mut w_table, *key);
    }
    w_table
}

fn make_waiting_for_c_entry(w_table: &mut WTable, key: u32) {
    let num_tiles = key_sum(key);
    let num_complete_groups = num_tiles / 3;
    let has_pair = (num_tiles % 3) == 2;

    let mut push = |new_key, pos: i8, waiting_kind: WaitingKind, _has_pair: bool| {
        w_table.entry(new_key).or_default().push(pack_alt(waiting_kind, pos) as u64)
    };

    if !has_pair {
        for pos in 0..=8 {
            if let Some(new_key) = check_pattern(key, 0o1, pos, 0) {
                push(new_key, pos, WaitingKind::Tanki, true);
            }
        }
    }
    if num_complete_groups <= 3 {
        // only 3 or less mentsu in complete part
        // try add mentsu-based tenpai pattern
        for pos in 0..=8 {
            if let Some(new_key) = check_pattern(key, 0o2, pos, 0) {
                push(new_key, pos, WaitingKind::Shanpon, has_pair);
            }
        }
        for pos in 0..=6 {
            if let Some(new_key) = check_pattern(key, 0o101, pos, 1) {
                push(new_key, pos, WaitingKind::Kanchan, has_pair);
            }
        }
        for pos in 0..=7 {
            let key_low = check_pattern(key, 0o11, pos, -1);
            let key_high = check_pattern(key, 0o11, pos, 2);
            if key_low.is_some() && key_high.is_some() {
                push(key_low.unwrap(), pos, WaitingKind::RyanmenBoth, has_pair);
            } else if let Some(key) = key_low {
                push(key, pos, WaitingKind::RyanmenLow, has_pair);
            } else if let Some(key) = key_high {
                push(key, pos, WaitingKind::RyanmenHigh, has_pair);
            }
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct WaitingPattern {
    pub complete_key: u32,
    pub waiting_kind: WaitingKind,
    pub pattern_pos: u8,
}

#[derive(
    Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd,
    strum::FromRepr, strum::AsRefStr, strum::EnumString,
)]
#[repr(u8)]
pub enum WaitingKind {
    #[default]
    Tanki = 0,   // e.g. 1222333444555z wait 1z
    Shanpon,     // e.g. 4477s wait 4s/7s
    Kanchan,     // e.g. 13m wait 2m
    RyanmenHigh, // e.g. 12m wait 3m, 12333345m77z wait 6m (because 3m is used up)
    RyanmenLow,  // e.g. 89m wait 7m, ditto
    RyanmenBoth, // e.g. 34m wait 2m/5m
}

impl WaitingKind {
    pub const fn is_shuntsu(self) -> bool {
        use WaitingKind::*;
        matches!(self, Kanchan | RyanmenHigh | RyanmenLow | RyanmenBoth)
    }

    pub const fn pattern(self) -> u32 {
        use WaitingKind::*;
        match self {
            Tanki => 0o1,
            Shanpon => 0o2,
            Kanchan => 0o101,
            RyanmenHigh | RyanmenLow | RyanmenBoth => 0o11,
        }
    }

    pub const fn pattern_at(self, pos: u8) -> u32 {
        self.pattern() << ((pos as u32) * 3)
    }
}

pub fn w_entry_iter(key: u32, packed_alts: u64) -> impl Iterator<Item = WaitingPattern> {
    w_entry_iter_alts(key, WAlts::from_packed(packed_alts))
}

pub fn w_entry_iter_alts(key: u32, alts: WAlts) -> impl Iterator<Item = WaitingPattern> {
    alts
        .map(|packed| unpack_alt(packed as u8))
        .map(move |(waiting_kind, pos)| {
            let complete_key = key - waiting_kind.pattern_at(pos);
            WaitingPattern{
                complete_key,
                waiting_kind,
                pattern_pos: pos,
            }
        })
}

pub(crate) mod details {
    
    use super::*;

    /// Attempts to place a waiting pattern at the position, and see if we are not attempting to use
    /// more than 4 tiles of each kind, including the waiting tile. If this attempt is valid,
    /// return `Some(new_key)`; otherwise `None`.
    pub fn check_pattern(key: u32, pattern: u8, pos: i8, waiting_offset: i8) -> Option<u32> {
        let new_key = key + ((pattern as u32) << ((pos as u32) * 3));
        let waiting_pos = pos + waiting_offset;
        if (0..=8).contains(&waiting_pos) &&
            !key_is_overflow(new_key) &&
            ((new_key >> ((waiting_pos as u32) * 3)) & 0o7) < 4 {
            Some(new_key)
        } else {
            None
        }
    }

    /// Encode (waiting_kind, pos) first into a "waiting alt" (1..55)
    ///
    /// Explained:
    /// - `waiting kind`: 0..6
    /// - `pos`: 0..9
    /// Total combinations: 6*9 = 54
    pub fn pack_alt(waiting_kind: WaitingKind, pos: i8) -> u8 {
        9 * (waiting_kind as u8) + (pos as u8) + 1
    }

    /// Decode (waiting_kind, pos) from a packed "waiting alt" (1..55)
    ///
    /// See [`pack_alt`].
    pub fn unpack_alt(packed: u8) -> (WaitingKind, u8) {
        let x = packed - 1;
        (WaitingKind::from_repr(x / 9).unwrap(), x % 9)
    }
}
