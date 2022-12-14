use core::fmt::{Display, Formatter};

use crate::{
    tile::Tile,
    tile_set::*,
    utils::{sort2, sort3},
};

use super::packed::{PackedMeld, PackedMeldKind};

/// An open group of 3 consecutive tiles (チー / 明順).
/// The called tile may only come from the previous player's discard.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct Chii {
    /// The calling player's own 2 tiles.
    pub own: [Tile; 2],

    /// The called tile.
    pub called: Tile,

    /// The smallest tile (ignoring red) in the group.
    pub min: Tile,
}

impl Chii {
    pub const fn dir(self) -> u8 { self.called.normal_num() - self.min.num() }
    pub const fn suit(self) -> u8 { self.called.suit() }

    /// Construct from own tiles and the called tile.
    pub fn from_tiles(own0: Tile, own1: Tile, called: Tile) -> Option<Self> {
        let suit = called.suit();
        if own0.suit() != suit || own1.suit() != suit { return None; }
        let (own0, own1) = sort2(own0, own1);
        let (a, b, c) = sort3(
            own0.to_normal(),
            own1.to_normal(),
            called.to_normal());
        if !(b == a.succ().unwrap() && c == b.succ().unwrap()) { return None; }
        Some(Chii { own: [own0, own1], called, min: a })
    }

    /// Checks whether own tiles exist in player's closed hand.
    pub fn is_in_hand(self, hand: &TileSet37) -> bool {
        hand[self.own[0]] >= 1 && hand[self.own[1]] >= 1
    }

    /// Removes own tiles from player's closed hand (assuming they exist).
    pub fn consume_from_hand(self, hand: &mut TileSet37) {
        hand[self.own[0]] -= 1;
        hand[self.own[1]] -= 1;
    }
}

impl Display for Chii {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "C{}{}{}{}",
               self.called.num(),
               self.own[0].num(),
               self.own[1].num(),
               self.called.suit_char())
    }
}

impl TryFrom<PackedMeld> for Chii {
    type Error = ();

    fn try_from(raw: PackedMeld) -> Result<Self, Self::Error> {
        if raw.kind() != PackedMeldKind::Chii as u8 { return Err(()); }
        let mut a = raw.get_tile().ok_or(())?;
        let mut b = a.succ().unwrap();
        let mut c = b.succ().unwrap();
        if raw.red() > 0 {
            a = a.to_red();
            b = b.to_red();
            c = c.to_red();
        }
        match raw.dir() {
            0 => Chii::from_tiles(b, c, a),
            1 => Chii::from_tiles(a, c, b),
            2 => Chii::from_tiles(a, b, c),
            _ => return Err(()),
        }.ok_or(())
    }
}

impl From<Chii> for PackedMeld {
    fn from(chii: Chii) -> Self {
        let [own0, own1] = chii.own;
        let red = own0.is_red() || own1.is_red() || chii.called.is_red();
        PackedMeld::new()
            .with_tile(chii.min.encoding())
            .with_dir(chii.dir())
            .with_red(red as u8)
            .with_kind(PackedMeldKind::Chii as u8)
    }
}
