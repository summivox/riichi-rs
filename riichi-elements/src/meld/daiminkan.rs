use core::fmt::{Display, Formatter};

use crate::{
    player::*,
    tile::Tile,
    tile_set::*,
    utils::{pack4, unpack4, sort3},
};

use super::{
    packed::{PackedMeld, PackedMeldKind, normalize_daiminkan},
    utils::{count_for_kan, daiminkan_tiles},
};


/// "Big Open Kan" formed by calling 1 with 3 of the same kind in the closed hand (大明槓).
/// Similar to [Pon](super::Pon), may be called from any other player's discard.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct Daiminkan {
    /// The calling player's own 3 tiles.
    pub own: [Tile; 3],

    /// The called tile.
    pub called: Tile,

    /// (discarding player - self) mod 4
    pub dir: Player,
}

impl Daiminkan {
    pub const fn num(self) -> u8 { self.called.normal_num() }
    pub const fn suit(self) -> u8 { self.called.suit() }

    /// Constructs from own tiles, the called tile, and the relative pos of the discarding player.
    pub fn from_tiles_dir(own: [Tile; 3], called: Tile, dir: Player) -> Option<Self> {
        if own[0].to_normal() != called.to_normal() ||
            own[1].to_normal() != called.to_normal() ||
            own[2].to_normal() != called.to_normal() ||
            dir.to_u8() == 0 { return None; }
        let (own0, own1, own2) = sort3(own[0], own[1], own[2]);
        Some(Daiminkan { own: [own0, own1, own2], called, dir })
    }

    /// Constructs from the closed hand for the called tile and
    /// the relative pos of the discarding player.
    pub fn from_hand_dir(hand: &TileSet37, called: Tile, dir: Player) -> Option<Self> {
        let normal = called.to_normal();
        let (num_normal, num_red) = count_for_kan(hand, normal);
        if num_normal + num_red != 3 { return None; }
        Self::from_tiles_dir(daiminkan_tiles(normal, num_red), called, dir)
    }

    /// Removes all own tiles from the hand (where this was constructed from).
    pub fn consume_from_hand(self, hand: &mut TileSet37) {
        hand[self.own[0].to_normal()] = 0;
        hand[self.own[0].to_red()] = 0;
    }
}

impl Display for Daiminkan {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let (n0, n1, n2, nc, s) = (
            self.own[0].num(),
            self.own[1].num(),
            self.own[2].num(),
            self.called.num(),
            self.called.suit_char(),
        );
        match self.dir.to_u8() {
            1 => write!(f, "{}{}{}D{}{}", n0, n1, n2, nc, s),
            2 => write!(f, "{}D{}{}{}{}", n0, nc, n1, n2, s),
            3 => write!(f, "D{}{}{}{}{}", nc, n0, n1, n2, s),
            _ => Err(core::fmt::Error::default()),
        }
    }
}

impl TryFrom<PackedMeld> for Daiminkan {
    type Error = ();

    fn try_from(raw: PackedMeld) -> Result<Self, Self::Error> {
        if raw.kind() != PackedMeldKind::Daiminkan as u8 { return Err(()); }
        let t = raw.get_tile().ok_or(())?;
        let (mut own0, mut own1, mut own2, mut called) = (t, t, t, t);
        let (r0, r1, r2, r3) = unpack4(normalize_daiminkan(raw.red()));
        if r0 { own0 = own0.to_red(); }
        if r1 { own1 = own1.to_red(); }
        if r2 { own2 = own2.to_red(); }
        if r3 { called = called.to_red(); }
        Daiminkan::from_tiles_dir(
            [own0, own1, own2], called, Player::new(raw.dir())).ok_or(())
    }
}

impl From<Daiminkan> for PackedMeld {
    fn from(daiminkan: Daiminkan) -> Self {
        let [own0, own1, own2] = daiminkan.own;
        PackedMeld::new()
            .with_tile(own0.normal_encoding())
            .with_dir(daiminkan.dir.to_u8())
            .with_red(pack4(own0.is_red(),
                            own1.is_red(),
                            own2.is_red(),
                            daiminkan.called.is_red()))
            .with_kind(PackedMeldKind::Daiminkan as u8)
    }
}
