//! [`Action`]s, [`Reaction`]s, and the [result](`ActionResult`) of an action-reaction cycle.

use crate::common::*;
use super::Discard;

/// Action by the in-turn player.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Action {
    /// Discard a tile. See [`Discard`].
    /// The `called_by` field is implied and can be safely ignored here.
    Discard(Discard),
    /// Declare an [`Ankan`] (4 in closed hand).
    Ankan(Tile),
    /// Declare a [`Kakan`] (1 in closed hand, 3 in pon).
    Kakan(Tile),
    /// Win by self-draw. See [`ActionResult::TsumoAgari`].
    TsumoAgari(Tile),
    /// Abort by Nine Kinds of Terminals. See [`ActionResult::AbortNineKinds`].
    AbortNineKinds,
}

impl Action {
    pub fn tile(self) -> Option<Tile> {
        match self {
            Action::Discard(discard) => Some(discard.tile),
            Action::Ankan(tile) => Some(tile),
            Action::Kakan(tile) => Some(tile),
            Action::TsumoAgari(tile) => Some(tile),
            Action::AbortNineKinds => None,
        }
    }

    pub fn is_terminal(self) -> bool {
        match self {
            Self::TsumoAgari(_) | Self::AbortNineKinds => true,
            _ => false,
        }
    }
}

/// Reaction from an out-of-turn player.
/// The lack of reaction / "pass" / unknown reaction can be represented by `Option<Reaction>`.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Reaction {
    // NOTE: Variant order matters --- used by `derive(Ord)` to comprare priority.
    Chii(Tile, Tile),
    Pon(Tile, Tile),
    Daiminkan,
    // tile is implicit
    RonAgari,
}

/// Conclusion of an action-reaction cycle.
/// Unknown state can be represented by `Option<PostReactionState>`, just like `Reaction`.
/// However, an explicit `Pass` is included to represent "nothing has happened; move on".
#[derive(Copy, Clone, Debug, num_enum::Default, Eq, PartialEq)]
#[repr(u8)]
pub enum ActionResult {
    #[num_enum(default)]
    /// The action successfully took place without any reaction.
    Pass = 0,

    /// A [`crate::Chii`] has been called (チー).
    Chii,
    /// A [`crate::Pon`] has been called (ポン).
    Pon,
    /// A [`crate::Daiminkan`] has been called (大明槓).
    Daiminkan,

    /// At least one player has won by steal (ロン和ガリ).
    /// Multiple players (but not too many) may call Ron on the same tile (discard/kakan/ankan).
    RonAgari,

    /// The player in action has won by self-draw (ツモ和ガリ).
    ///
    /// Resolution:
    /// - Determined by in-turn action.
    /// - No reaction allowed.
    TsumoAgari,

    /// The round has been aborted due to the player in action declaring "nine kinds of terminals"
    /// (九種九牌).
    ///
    /// Resolution:
    /// - Determined by in-turn action.
    /// - No reaction allowed.
    ///
    /// <https://riichi.wiki/Tochuu_ryuukyoku#Kyuushu_kyuuhai>
    AbortNineKinds,

    /// The round has ended because no more tiles can be drawn from the wall (荒牌).
    /// Penalties payments may apply (不聴罰符), including sub-type [`Self::AbortNagashiMangan`].
    ///
    /// Resolution:
    /// - Determined by end-of-turn resolution.
    /// - Can be preempted by [`Self::RonAgari`] and all other aborts.
    ///
    /// <https://riichi.wiki/Ryuukyoku>
    AbortWallExhausted,

    /// Special case of [`Self::AbortWallExhausted`] (流し満貫).
    /// Treated as penalties payments.
    ///
    /// <https://riichi.wiki/Nagashi_mangan>
    AbortNagashiMangan,

    /// Four Kan's (四開槓).
    /// - A player has attemped to call the 5th Kan of the round; all 5 are by the same player.
    /// - A player has attempted to call the 4th Kan of the round; all 4 are NOT by the same player.
    ///
    /// Resolution:
    /// - Determined by end-of-turn resolution.
    /// - Can be preempted by [`Self::RonAgari`].
    ///
    /// Note that kakan and ankan may also be preempted due to Chankan (搶槓).
    ///
    /// <https://riichi.wiki/Tochuu_ryuukyoku#Suukaikan>
    AbortFourKan,

    /// The round has been aborted because four of the same kind of wind tile has been discarded
    /// consecutively since the game starts (四風連打).
    ///
    /// Resolution:
    /// - Determined by end-of-turn resolution.
    /// - Cannot be preempted.
    ///
    /// <https://riichi.wiki/Tochuu_ryuukyoku#Suufon_renda>
    AbortFourWind,

    /// The round has been aborted because all four players are under active riichi (四家立直).
    ///
    /// Resolution:
    /// - Determined by end-of-turn resolution.
    /// - Can be preempted by [`Self::RonAgari`].
    ///
    /// <https://riichi.wiki/Tochuu_ryuukyoku#Suucha_riichi>
    AbortFourRiichi,

    /// The round has been aborted because too many players (usually 3) called Ron on the same tile.
    ///
    /// Resolution:
    /// - Determined by end-of-turn resolution.
    /// - Pre-empts all others.
    ///
    /// <https://riichi.wiki/Tochuu_ryuukyoku#Sanchahou>
    AbortMultiRon,
}

impl ActionResult {
    pub const fn is_meld(self) -> bool {
        use ActionResult::*;
        match self {
            Chii | Pon | Daiminkan => true,
            _ => false,
        }
    }
    pub const fn is_agari(self) -> bool {
        use ActionResult::*;
        match self {
            TsumoAgari | RonAgari => true,
            _ => false,
        }
    }
    pub const fn is_abort(self) -> bool {
        use ActionResult::*;
        match self {
            AbortNineKinds | AbortWallExhausted | AbortNagashiMangan |
            AbortFourKan | AbortFourWind | AbortFourRiichi | AbortMultiRon => true,
            _ => false,
        }
    }
    pub const fn is_terminal(self) -> bool { self.is_agari() || self.is_abort() }
}