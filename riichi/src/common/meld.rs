//! Meld 副露
//!
//! A [`Meld`] is one of: [`Chii`], [`Pon`], [`Kakan`], [`Daiminkan`], [`Ankan`].
//!
//! ## Ref
//! - <https://riichi.wiki/Naki>
//! - <https://ja.wikipedia.org/wiki/%E5%89%AF%E9%9C%B2>

use std::fmt::{Display, Formatter};

use crate::{
    common::{
        HandGroup,
        Tile,
        TileSet37,
        typedefs::*
    }
};

mod chii;
mod pon;
mod kakan;
mod daiminkan;
mod ankan;
mod packed;
mod utils;

pub use chii::Chii;
pub use pon::Pon;
pub use kakan::Kakan;
pub use daiminkan::Daiminkan;
pub use ankan::Ankan;
pub use utils::*;

/// Sum type of all kinds of melds (副露).
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Meld {
    /// See [`Chii`].
    Chii(Chii),
    /// See [`Pon`].
    Pon(Pon),
    /// See [`Kakan`].
    Kakan(Kakan),
    /// See [`Daiminkan`].
    Daiminkan(Daiminkan),
    /// See [`Ankan`].
    Ankan(Ankan),
}

impl Meld {
    /// [`Ankan`]
    pub fn is_closed(&self) -> bool {
        matches!(self, Meld::Ankan(_))
    }

    /// [`Kakan`], [`Daiminkan`], or [`Ankan`]
    pub fn is_kan(&self) -> bool {
        matches!(self, Meld::Kakan(_) | Meld::Daiminkan(_) | Meld::Ankan(_))
    }

    pub fn called(&self) -> Option<Tile> {
        match self {
            Self::Chii(chii) => Some(chii.called),
            Self::Pon(pon) => Some(pon.called),
            Self::Daiminkan(daiminkan) => Some(daiminkan.called),
            Self::Kakan(kakan) => Some(kakan.pon.called),
            Self::Ankan(_) => None,
        }
    }

    pub fn dir(&self) -> Option<Player> {
        match self {
            Self::Chii(_) => Some(P3),
            Self::Pon(pon) => Some(pon.dir),
            Self::Daiminkan(daiminkan) => Some(daiminkan.dir),
            Self::Kakan(kakan) => Some(kakan.pon.dir),
            Self::Ankan(_) => None,
        }
    }

    /// Maps to the equivalent closed-hand group. Useful for e.g. winning condition calculations.
    /// - Chii => [`HandGroup::Shuntsu`]
    /// - Pon/Kan => [`HandGroup::Koutsu`] (ignoring the 4th tile)
    pub fn to_equivalent_group(&self) -> HandGroup {
        use HandGroup::*;
        match self {
            Meld::Chii(chii) => Shuntsu(chii.min),
            Meld::Pon(pon) => Koutsu(pon.called.to_normal()),
            Meld::Kakan(kakan) => Koutsu(kakan.added.to_normal()),
            Meld::Daiminkan(daiminkan) => Koutsu(daiminkan.called.to_normal()),
            Meld::Ankan(ankan) => Koutsu(ankan.own[0].to_normal()),
        }
    }

    pub fn consume_from_hand(&self, hand: &mut TileSet37) {
        match self {
            Meld::Chii(chii) => chii.consume_from_hand(hand),
            Meld::Pon(pon) => pon.consume_from_hand(hand),
            Meld::Daiminkan(daiminkan) => daiminkan.consume_from_hand(hand),
            Meld::Kakan(kakan) => kakan.consume_from_hand(hand),
            Meld::Ankan(ankan) => ankan.consume_from_hand(hand),
        }
    }
}

impl Display for Meld {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // Different melds' string representations are already distinct; simply pass through.
        match self {
            Meld::Chii(chii) => write!(f, "{}", chii),
            Meld::Pon(pon) => write!(f, "{}", pon),
            Meld::Kakan(kakan) => write!(f, "{}", kakan),
            Meld::Daiminkan(daiminkan) => write!(f, "{}", daiminkan),
            Meld::Ankan(ankan) => write!(f, "{}", ankan),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn chii_example() {
        let chii = Chii::from_tiles(
            "4s".parse().unwrap(),
            "6s".parse().unwrap(),
            "0s".parse().unwrap()).unwrap();
        let meld = Meld::Chii(chii);
        assert_eq!(Meld::from_packed(0x1155), Some(meld));
        assert_eq!(meld.packed(), 0x1155);
        assert_eq!(chii.to_string(), "C046s");
        assert_eq!(meld.to_string(), "C046s");
    }

    #[test]
    fn pon_example() {
        let pon = Pon::from_tiles_dir(
            "5p".parse().unwrap(),
            "0p".parse().unwrap(),
            "0p".parse().unwrap(),
            P2).unwrap();
        let meld = Meld::Pon(pon);
        assert_eq!(Meld::from_packed(0x258D), Some(meld));
        assert_eq!(meld.packed(), 0x258D);
        assert_eq!(pon.to_string(), "0P05p");
        assert_eq!(meld.to_string(), "0P05p");
    }

    #[test]
    fn null_example() {
        assert_eq!(Meld::from_packed(0), None);
    }

    #[test]
    fn sizeof() {
        println!("Meld={} (align={}), Option<Meld>={} (align={})",
                 std::mem::size_of::<Meld>(),
                 std::mem::align_of::<Meld>(),
                 std::mem::size_of::<Option<Meld>>(),
                 std::mem::align_of::<Option<Meld>>(),
        );
    }
}
