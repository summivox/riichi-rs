use core::fmt::{Display, Formatter};
use core::ops::{Index, IndexMut};

use derive_more::{
    Constructor, From, Into, IntoIterator, Index, IndexMut,
};

use crate::tile::Tile;

/// Histogram for all 37 kinds of tiles (including reds).
/// Can be directly indexed with [`Tile`].
#[derive(Clone, Debug, Eq, PartialEq, Constructor, From, Into, IntoIterator, Index, IndexMut)]
pub struct TileSet37(pub [u8; 37]);

impl Index<Tile> for TileSet37 {
    type Output = u8;
    fn index(&self, tile: Tile) -> &Self::Output {
        &self.0[tile.encoding() as usize]
    }
}

impl IndexMut<Tile> for TileSet37 {
    fn index_mut(&mut self, tile: Tile) -> &mut Self::Output {
        &mut self.0[tile.encoding() as usize]
    }
}

impl Default for TileSet37 {
    fn default() -> Self { TileSet37([0; 37]) }
}

impl Display for TileSet37 {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        for xs in [
            &self.0[0..9],
            &self.0[9..18],
            &self.0[18..27],
            &self.0[27..34],
            &self.0[34..37],
        ] {
            for x in xs {
                write!(f, "{}", x)?;
            }
            write!(f, ",")?;
        }
        Ok(())
    }
}

impl FromIterator<Tile> for TileSet37 {
    fn from_iter<T: IntoIterator<Item=Tile>>(tiles: T) -> Self {
        let mut ts = Self::default();
        for tile in tiles {
            ts[tile] += 1;
        }
        ts
    }
}

impl TileSet37 {
    /// An empty tile set.
    pub const fn empty_set() -> Self { TileSet37([0; 37]) }

    /// The complete set of tiles in a game, given the number of red 5's in play.
    /// Each red 5 replaces its corresponding normal 5; the total number of tiles remains 136
    /// (34 x 4).
    pub const fn complete_set(num_reds: [u8; 3]) -> Self {
        let mut a = [4; 37];
        a[34] = num_reds[0];
        a[35] = num_reds[1];
        a[36] = num_reds[2];
        a[4] = 4 - num_reds[0];
        a[13] = 4 - num_reds[1];
        a[22] = 4 - num_reds[2];
        TileSet37(a)
    }

    /// Same as [`super::TileSet34::packed_34`], but collapsing red 5's into normal 5's.
    pub fn packed_34(&self) -> [u32; 4] {
        let mut packed = [0u32; 4];
        let h = &self.0;
        for i in (0..34).rev() {
            let s = i / 9;
            packed[s] = (packed[s] << 3) | (h[i] as u32);
        }
        packed[0] += (h[34] as u32) << (3 * 4);
        packed[1] += (h[35] as u32) << (3 * 4);
        packed[2] += (h[36] as u32) << (3 * 4);
        packed
    }

    /// Iterates through all tiles in this tile set, in encoding order (i.e. reds come after honors).
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
    fn tile_set_is_indexable_with_tile() {
        let mut h = TileSet37::from_iter(tiles_from_str("1112345678999m"));
        h[t!("9m")] -= 2;
        h[t!("7z")] += 2;
        assert_eq!(h, [
            3, 1, 1, 1, 1, 1, 1, 1, 1,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 2,
            0, 0, 0u8,
        ].into());
    }
}
