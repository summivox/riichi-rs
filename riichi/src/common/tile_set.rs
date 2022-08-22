//! Unordered multi-sets of tiles, represented as histograms.

use std::ops::{Index, IndexMut};

use derive_more::{Constructor, From, Into, Index, IndexMut};

use crate::Tile;

/// Histogram for all 37 kinds of tiles (including red)
#[derive(Copy, Clone, Debug, Eq, PartialEq, Constructor, From, Into, Index, IndexMut)]
pub struct TileSet37([u8; 37]);

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
    fn default() -> Self { TileSet37([0u8; 37]) }
}

impl From<&[Tile]> for TileSet37 {
    fn from(tiles: &[Tile]) -> Self {
        let mut histogram = TileSet37::default();
        for &tile in tiles {
            histogram[tile] += 1;
        }
        histogram
    }
}

impl TileSet37 {
    fn to_sorted_vec(&self) -> Vec<Tile> {
        let mut tiles: Vec<Tile> = vec![];
        tiles.reserve_exact(self.0.into_iter().sum::<u8>() as usize);
        for (encoding, count) in self.0.into_iter().enumerate() {
            for _ in 0..count {
                tiles.push(Tile::from_encoding(encoding as u8).unwrap());
            }
        }
        tiles
    }
}

/// Histogram for all 34 kinds of normal tiles (red 5's are treated as normal 5's)
#[derive(Copy, Clone, Debug, Eq, PartialEq, Constructor, From, Into, Index, IndexMut)]
pub struct TileSet34([u8; 34]);

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

// Conversion is one-way from 37 to 34 (count of red is lost).
impl From<TileSet37> for TileSet34 {
    fn from(original: TileSet37) -> Self {
        let mut result: [u8; 34] = (&original[..34]).try_into().unwrap();
        result[4] += original[34];
        result[13] += original[35];
        result[22] += original[36];
        Self(result)
    }
}

impl From<&[Tile]> for TileSet34 {
    fn from(tiles: &[Tile]) -> Self {
        let mut histogram = TileSet34::default();
        for &tile in tiles {
            histogram[tile] += 1;
        }
        histogram
    }
}

impl TileSet34 {
    pub fn to_sorted_vec(&self) -> Vec<Tile> {
        let mut tiles: Vec<Tile> = vec![];
        tiles.reserve_exact(self.0.into_iter().sum::<u8>() as usize);
        for (encoding, count) in self.0.into_iter().enumerate() {
            for _ in 0..count {
                tiles.push(Tile::from_encoding(encoding as u8).unwrap());
            }
        }
        tiles
    }

    /// Compress the histogram so that each element takes 3 bits (valid range `0..=4`).
    /// This results in 4 x 27-bit integers, one for each suit.
    ///
    /// Conveniently this is 1 digit per element in octal.
    pub fn packed(&self) -> [u32; 4] {
        let mut packed = [0u32; 4];
        let h = &self.0;
        for i in (0..34).rev() {
            let s = i / 9;
            packed[s] = (packed[s] << 3) | (h[i] as u32);
        }
        packed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use crate::tiles_from_str;

    #[test]
    fn histogram_can_be_indexed_with_tile() {
        let mut h = TileSet37::from(&tiles_from_str("1112345678999m")[..]);
        h[Tile::from_str("9m").unwrap()] -= 2;
        h[Tile::from_str("7z").unwrap()] += 2;
        assert_eq!(h, [
            3, 1, 1, 1, 1, 1, 1, 1, 1,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 2,
            0, 0, 0u8,
        ].into());
    }

    #[test]
    fn ts34_treats_red_as_normal() {
        let mut h = TileSet34::default();
        h[Tile::from_str("5m").unwrap()] = 1;
        h[Tile::from_str("0p").unwrap()] = 2;
        h[Tile::from_str("6s").unwrap()] = 3;
        assert_eq!(h, [
            0, 0, 0, 0, 1, 0, 0, 0, 0,
            0, 0, 0, 0, 2, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 3, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
        ].into());
    }

    #[test]
    fn ts34_packs_correctly() {
        let mut h = TileSet34::from(&tiles_from_str("147m258p369s77z")[..]);
        assert_eq!(h.packed(), [
            0o001001001,
            0o010010010,
            0o100100100,
            0o2000000,
        ]);
    }
}