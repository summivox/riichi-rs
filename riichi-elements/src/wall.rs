//! The wall of tiles (牌山/山/壁牌) and utils.
//!
//! ## How we represent the wall
//!
//! ```ascii_art
//!                          _______________      ______________
//!                         <--- TAIL (CCW) |    / HEAD (CW) --->
//!  self draws |  dora indicators  |rinshan|   |            initial deal             | self draws
//!      118 120|122 124 126 128 130|132 134|   | 0   2   4   6   8  10        48  50 |52  54
//! ... +---+---*---+---+---+---+###*---+---+   +---+---+---+---+---+---+ ... +---+---*---+---+ ...
//!     |#66|#68| D4| D3| D2| D1| D0|RS2|RS0|   |E0 |E2 |S0 |S2 |W0 |W2 |     |E12|W12|#00|#02|      TOP
//! ... +===+===*===+===+===+===+===*===+===+   +===+===+===+===+===+===+ ... +===+===*===+===+ ...
//!     |#67|#69|UD4|UD3|UD2|UD1|UD0|RS3|RS1|   |E1 |E3 |S1 |S3 |W1 |W3 |     |S12|N12|#01|#03|      BOTTOM
//! ... +---+---*---+---+---+---+---*---+---+   +---+---+---+---+---+---+ ... +---+---*---+---+ ...
//!      119 121|123 125 127 129 131|133 135|   | 1   3   5   7   9  11        49  51 |53  55
//!  self draws |ura-dora indicators|rinshan|   |            initial deal             | self draws
//! ```
//!
//! In a physical game, the following procedure is used to prepare the wall:
//!
//! 1.  Shuffle: 136 tiles => 4 sides x 17 stacks x 2 tiles per stack, treated as a ring.
//!
//! 2.  Randomly decide the splitting point on this ring (rules vary on this).
//!
//! 3.  From the splitting point: clockwise => head, counterclockwise => tail.
//!     Now the ring can be treated as a linear 68 x 2 array of tiles.
//!
//! 4.  Reveal the top tile of the 3rd stack from tail; this is the first Dora Indicator.
//!     (figure: `###`)
//!
//! 5.  Initial deal: Players take turns (E->S->W->N->E->...) to draw 2 stacks (= 4 tiles) from the
//!     head until everyone has 12. Each player then draws one more tile.
//!     (figure: `E0`~`E3`, `S0`~`S3`, (...), `W8`~`W11`, `N8`~`N11`; `E12`, `S12`, `W12`, `N12`)
//!
//! 6.  The button player takes his first self-draw and the round starts.
//!     (figure: `#00`)
//!
//! 7.  Additional draw after each Kan is taken from the tail.
//!     (figure: `RS0`, `RS1`, `RS2`, `RS3`)
//!
//! 8.  Additional Dora Indicators are flipped further from tail since the initial one.
//!     (figure: `D1`, `D2`, `D3`, `D4`)
//!
//! In this crate, we assign an index to each of the 136 tiles in the "linear" wall after splitting.
//! The split wall is indexed head-to-tail (major), top-to-bottom (minor). In the figure, this index
//! is annotated as numbers next to the boxes (above/below).
//!
//! ## Ref
//!
//! - <https://ja.wikipedia.org/wiki/配牌>
//! - <https://ja.wikipedia.org/wiki/壁牌>
//! - <https://riichi.wiki/Yama>

use core::fmt::{Display, Formatter};

use crate::{
    tile::Tile,
    tile_set::*,
    player::*,
};

/// The wall of tiles.
/// See [mod-level docs](self).
pub type Wall = [Tile; 136];

/// Wall with some tiles unknown.
pub type PartialWall = [Option<Tile>; 136];

/// Constructor for an obviously invalid wall. Useful for mutating it later.
pub const fn make_dummy_wall() -> Wall { [Tile::MIN; 136] }

/// Make a sorted wall of the standard 136-tile set, including specified number of red-5's for each
/// (numeral) suit.
pub fn make_sorted_wall(num_reds: [u8; 3]) -> Wall {
    let mut wall = [Tile::MIN; 136];
    for encoding in 0u8..34u8 {
        let tile = Tile::from_encoding(encoding).unwrap();
        let suit = tile.suit();
        let num = tile.num();
        if num == 5 && suit <= 2 {
            for i in 0..num_reds[suit as usize] {
                wall[(encoding * 4 + i) as usize] = tile.to_red();
            }
            for i in num_reds[suit as usize]..4 {
                wall[(encoding * 4 + i) as usize] = tile;
            }
        } else {
            for i in 0..4 {
                wall[(encoding * 4 + i) as usize] = tile;
            }
        }
    }
    wall
}

/// Make sure that a wall is valid --- 34 kinds x 4 each = 136
pub fn is_valid_wall(wall: Wall) -> bool {
    TileSet34::from_iter(wall).into_iter().all(|n| n == 4)
}

/// For each player starting from the button player, which wall tiles to take as the initial draw?
pub const DEAL_INDEX: [[usize; 13]; 4] = [
    [0x00, 0x01, 0x02, 0x03, 0x10, 0x11, 0x12, 0x13, 0x20, 0x21, 0x22, 0x23, 0x30],
    [0x04, 0x05, 0x06, 0x07, 0x14, 0x15, 0x16, 0x17, 0x24, 0x25, 0x26, 0x27, 0x31],
    [0x08, 0x09, 0x0a, 0x0b, 0x18, 0x19, 0x1a, 0x1b, 0x28, 0x29, 0x2a, 0x2b, 0x32],
    [0x0c, 0x0d, 0x0e, 0x0f, 0x1c, 0x1d, 0x1e, 0x1f, 0x2c, 0x2d, 0x2e, 0x2f, 0x33],
];
/// Index of dora indicators in the wall, by their order of revealing first-to-last.
pub const DORA_INDICATOR_INDEX: [usize; 5] = [130, 128, 126, 124, 122];
/// Index of ura-dora indicators in the wall; order corresponding to dora indicators.
pub const URA_DORA_INDICATOR_INDEX: [usize; 5] = [131, 129, 127, 125, 123];
/// Index of kan draws in the wall, first-to-last.
pub const KAN_DRAW_INDEX: [usize; 4] = [134, 135, 132, 133];

/// Total number of draws (front + back) cannot exceed this.
pub const MAX_NUM_DRAWS: u8 = 136 - 14;

/// Draws the initial 13 tiles for each of the 4 players, according to standard rules.
/// See [module-level docs](self).
pub fn deal(wall: &Wall, button: Player) -> [TileSet37; 4] {
    let mut hists = [
        TileSet37::default(),
        TileSet37::default(),
        TileSet37::default(),
        TileSet37::default(),
    ];
    for i in 0..4 {
        for wall_index in DEAL_INDEX[i] {
            let p = button.add(Player::new(i as u8));
            hists[p.to_usize()][wall[wall_index].encoding() as usize] += 1;
        }
    }
    hists
}

/// Returns the indexed (0..=4) dora indicator.
pub fn dora_indicator(wall: &Wall, i: usize) -> Tile {
    wall[DORA_INDICATOR_INDEX[i]]
}

/// Returns the indexed (0..=4) ura-dora indicator.
pub fn ura_dora_indicator(wall: &Wall, i: usize) -> Tile {
    wall[URA_DORA_INDICATOR_INDEX[i]]
}

/// Returns the entire dora indicator section of the wall as an array.
/// Note that this does not handle the gradual revealing of dora indicators.
pub fn dora_indicators(wall: &Wall) -> [Tile; 5] {
    DORA_INDICATOR_INDEX.map(|i| wall[i])
}

/// Returns the entire ura-dora indicator section of the wall as an array.
/// Note that this does not handle the final revealing of ura-dora indicators.
pub fn ura_dora_indicators(wall: &Wall) -> [Tile; 5] {
    URA_DORA_INDICATOR_INDEX.map(|i| wall[i])
}

/// Returns the indexed (0..=3) Kan draw.
pub fn kan_draw(wall: &Wall, i: usize) -> Tile {
    wall[KAN_DRAW_INDEX[i]]
}

/// Deduces the set of unknown tiles from the given partially-known wall, and the known total number
/// of red tiles.
/// 
/// Panics when the partially-known wall is inconsistent with the assumed complete set of tiles. 
pub fn get_missing_tiles_in_partial_wall(partial_wall: &PartialWall, num_reds: [u8; 3]) -> TileSet37 {
    let mut missing = TileSet37::complete_set(num_reds);
    for tile_or_hole in partial_wall {
        if let &Some(tile) = tile_or_hole {
            if missing[tile] == 0 {
                panic!("More {} in the partial wall than expected.", tile)
            }
            missing[tile] -= 1;
        }
    }
    missing
}

/// Combine the partially-known wall and (reordered) unknown tiles to form a fully-known wall.
/// 
/// Panics when there are not enough tiles in `missing_tiles` to fill the "holes" in `partial_wall`.
/// 
/// Does not check validity of the resulting wall.
pub fn fill_missing_tiles_in_partial_wall(
    partial_wall: &PartialWall, missing_tiles: impl IntoIterator<Item=Tile>) -> Wall {
    let mut missing_iter = missing_tiles.into_iter();
    partial_wall.map(|tile_or_hole|
        tile_or_hole.or_else(|| missing_iter.next()).unwrap())
}

// Hack to impl `Display` for both `Wall` and `PartialWall`

pub struct WallDisplay<'a>(&'a Wall);
pub trait WallDisplayMethod {
    fn display(&self) -> WallDisplay<'_>;
}
impl WallDisplayMethod for Wall {
    fn display(&self) -> WallDisplay<'_> { WallDisplay(self) }
}
impl<'a> Display for WallDisplay<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        for (i, tile) in self.0.iter().enumerate() {
            write!(f, "{} ", tile)?;
            if i % 8 == 7 {
                writeln!(f)?;
            }
        }
        writeln!(f)
    }
}

pub struct PartialWallDisplay<'a>(&'a PartialWall);
pub trait PartialWallDisplayMethod {
    fn display(&self) -> PartialWallDisplay<'_>;
}
impl PartialWallDisplayMethod for PartialWall {
    fn display(&self) -> PartialWallDisplay<'_> { PartialWallDisplay(self) }
}
impl<'a> Display for PartialWallDisplay<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        for (i, maybe_tile) in self.0.iter().enumerate() {
            if let Some(tile) = maybe_tile {
                write!(f, "{} ", tile)?;
            } else {
                write!(f, "?? ")?;
            }
            if i % 8 == 7 {
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use crate::tile::tiles_from_str;

    #[test]
    fn sorted_wall_is_correct() {
        let ans = concat!(
            "111122223333444405556666777788889999m",
            "111122223333444400556666777788889999p",
            "111122223333444455556666777788889999s",
            "1111222233334444555566667777z");
        let wall = make_sorted_wall([1, 2, 0]);
        itertools::assert_equal(wall, tiles_from_str(ans));
        assert!(is_valid_wall(wall));
    }

    #[test]
    fn sorted_wall_deals_correctly() {
        let wall = make_sorted_wall([1, 1, 1]);
        assert_eq!(deal(&wall, P1), [
            TileSet37::new([
                0, 0, 0, 4, 0, 0, 0, 4, 0,
                0, 0, 4, 1, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0,
                0, 0, 0,
            ]),  // N: 4444m 8888m 3333p 4p
            TileSet37::new([
                4, 0, 0, 0, 3, 0, 0, 0, 4,
                0, 0, 0, 1, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0,
                1, 0, 0,
            ]),  // E: 1111m 0555m 9999m 4p
            TileSet37::new([
                0, 4, 0, 0, 0, 4, 0, 0, 0,
                4, 0, 0, 1, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0,
                0, 0, 0,
            ]),  // S: 2222m 6666m 1111p 4p
            TileSet37::new([
                0, 0, 4, 0, 0, 0, 4, 0, 0,
                0, 4, 0, 1, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0,
                0, 0, 0,
            ]),  // W: 3333m 7777m 2222p 4p
        ]);
    }
}
