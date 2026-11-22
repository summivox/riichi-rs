use core::fmt::{Display, Formatter};
use core::ops::{Index, IndexMut};

use super::TileSet37;

use derive_more::{
    Constructor, From, Into, IntoIterator, Index, IndexMut,
};

use crate::tile::Tile;

/// Histogram for all 34 kinds of normal tiles (red 5's are treated as normal 5's).
/// Can be directly indexed with [`Tile`].
#[derive(Clone, Debug, Eq, PartialEq, Constructor, From, Into, IntoIterator, Index, IndexMut)]
pub struct TileSet34(pub [u8; 34]);

impl Index<Tile> for TileSet34 {
    type Output = u8;
    fn index(&self, tile: Tile) -> &Self::Output {
        &self.0[tile.normal_encoding() as usize]  // NOTE: different
    }
}

impl IndexMut<Tile> for TileSet34 {
    fn index_mut(&mut self, tile: Tile) -> &mut Self::Output {
        &mut self.0[tile.normal_encoding() as usize]  // NOTE: different
    }
}

impl Default for TileSet34 {
    fn default() -> Self { TileSet34([0u8; 34]) }
}

impl Display for TileSet34 {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        for xs in self.0.chunks(9) {
            for x in xs {
                write!(f, "{}", x)?;
            }
            write!(f, ",")?;
        }
        Ok(())
    }
}

// Conversion is one-way from 37 to 34 (count of red is lost).
impl From<&TileSet37> for TileSet34 {
    fn from(original: &TileSet37) -> Self {
        let mut result: [u8; 34] = (original[..34]).try_into().unwrap();
        result[4] += original[34];
        result[13] += original[35];
        result[22] += original[36];
        Self(result)
    }
}

impl FromIterator<Tile> for TileSet34 {
    fn from_iter<T: IntoIterator<Item=Tile>>(tiles: T) -> Self {
        let mut ts = Self::default();
        for tile in tiles {
            ts[tile.to_normal()] += 1;
        }
        ts
    }
}

impl TileSet34 {
    /// An empty tile set.
    pub const fn empty_set() -> Self { TileSet34([0; 34]) }

    /// The complete set of tiles in a game, without differentiating red 5's from normal 5's.
    /// The total number of tiles is 136 (34 x 4).
    pub const fn complete_set() -> Self { TileSet34([4; 34]) }

    /// Reconstructs the histogram from its [packed](Self::packed_34) representation.
    pub fn from_packed(packed: [u32; 4]) -> Self {
        let mut ts34 = Self::default();
        let mut i = 0;
        for s in 0..3 {
            let mut m = packed[s];
            for _ in 0..9 {
                ts34[i] = (m & 0o7) as u8;
                i += 1;
                m >>= 3;
            }
        }
        let mut m = packed[3];
        for _ in 0..7 {
            ts34[i] = (m & 0o7) as u8;
            i += 1;
            m >>= 3;
        }
        ts34
    }

    /// Compresses the histogram so that each element takes 3 bits (valid range `0..=4`).
    /// This results in 4 x 27-bit integers, one for each suit.
    ///
    /// Conveniently, this is 1 digit per element in octal.
    pub fn packed_34(&self) -> [u32; 4] {
        let mut packed = [0u32; 4];
        let h = &self.0;
        for i in (0..34).rev() {
            let s = i / 9;
            packed[s] = (packed[s] << 3) | (h[i] as u32);
        }
        packed
    }

    /// Iterates through all tiles in this tile set, in encoding order.
    pub fn iter_tiles(&self) -> impl Iterator<Item=Tile> + use<> {
        self.0.into_iter().enumerate().flat_map(|(encoding, count)|
            itertools::repeat_n(
                Tile::from_encoding(encoding as u8).unwrap(),
                count as usize))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::*;

    #[test]
    fn ts34_treats_red_as_normal() {
        let mut h = TileSet34::default();
        h[t!("5m")] = 1;
        h[t!("0p")] = 2;
        h[t!("6s")] = 3;
        assert_eq!(h, [
            0, 0, 0, 0, 1, 0, 0, 0, 0,
            0, 0, 0, 0, 2, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 3, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
        ].into());
    }

    #[test]
    fn ts34_packs_correctly() {
        let h = TileSet34::from_iter(tiles_from_str("147m258p369s77z"));
        assert_eq!(h.packed_34(), [
            0o001001001,
            0o010010010,
            0o100100100,
            0o2000000,
        ]);
        assert_eq!(TileSet34::from_packed(h.packed_34()), h);
    }
}
