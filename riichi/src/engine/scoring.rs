use std::cmp::min;

use riichi_decomp::{IrregularWait, Wait};
use riichi_elements::prelude::*;

use crate::{
    model::*,
    rules::Ruleset,
    yaku::*,
};

pub fn calc_scoring(
    ruleset: &Ruleset,
    yaku_values: &YakuValues,
    wait: Wait,
    dora_hits: DoraHits,
    agari_kind: AgariKind,
    is_closed: bool,
    extra_fu: u8,
) -> Scoring {
    let value_sum = yaku_values.values().sum::<i8>();
    if value_sum < 0 {
        Scoring {
            yakuman_total_value: (-value_sum) as u8,
            yaku_total_value: 0,
            dora_hits,
            fu: 0,
        }
    } else if value_sum > 0 {
        Scoring {
            yakuman_total_value: 0,
            yaku_total_value: value_sum as u8,
            dora_hits,
            fu: match wait {
                Wait::Irregular(IrregularWait::SevenPairs(_)) => 25,
                _ => calc_regular_fu(ruleset, agari_kind, is_closed, extra_fu),
            },
        }
    } else { panic!() }
}

/// Calculates the points gains and losses for each player given a win.
///
/// Tsumo:
/// - Button gets 2x from each of the 3 non-button players.
/// - Non-button gets 1x from each of the 2 non-button players and 2x from the button player.
/// - Honba payout is 100 per honba per player (total 3 players => 300 per honba total).
///
/// Ron:
/// - Button gets 6x from the contributor
/// - Non-button gets 4x from the contributor
/// - Honba payout is 300 per honba.
///
/// Each transaction between two players is separately rounded up to the nearest 100 points.
/// Note that this is the _only_ rounding step for points --- basic points are _not_ rounded.
pub fn distribute_points(
    _ruleset: &Ruleset,
    round_id: RoundId,
    take_pot: bool,
    winner: Player,
    contributor: Player,
    basic_points: GamePoints,
) -> [GamePoints; 4] {
    let button = round_id.button();
    // TODO(summivox): ruleset (atama-hane), ruleset (basengo)
    let honba = if take_pot { round_id.honba as GamePoints } else { 0 };
    let k_honba = 100;

    let mut delta = [0; 4];
    if winner == contributor {
        // Tsumo
        let (k_non_button, k_button) = if winner == button { (2, 0) } else { (1, 2) };
        for player in other_players_after(winner) {
            let k = if player == button { k_button } else { k_non_button };
            let points = round_points_up(k * basic_points + k_honba * honba);
            delta[winner.to_usize()] += points;
            delta[player.to_usize()] -= points;
        }
    } else {
        // Ron
        let k = if winner == button { 6 } else { 4 };
        let points = round_points_up(k * basic_points + 3 * k_honba * honba);
        delta[winner.to_usize()] += points;
        delta[contributor.to_usize()] -= points;
    }
    delta
}

impl Scoring {
    pub fn basic_points(&self) -> GamePoints {
        if self.yakuman_total_value > 0 {
            return 8000 * self.yakuman_total_value as GamePoints
        }
        match self.han() {
            0 => 0,
            1..=5 => min(2000,
                         fu_han_formula(self.fu, self.han())),  // mangan or less
            6..=7 => 3000,  // haneman (1.5x mangan)
            8..=10 => 4000,  // baiman (2x mangan)
            11..=12 => 6000,  // sanbaiman (3x mangan)
            _ => 8000,  // kazoe-yakuman (4x mangan)
        }
    }

    pub fn basic_points_aotenjou(&self) -> GamePoints {
        fu_han_formula(self.fu, self.yakuman_total_value * 13 + self.han())
    }
}

fn fu_han_formula(fu: u8, han: u8) -> GamePoints {
    fu as GamePoints * (1 << (2 + han as GamePoints))
}

fn round_fu_up(fu: u8) -> u8 { fu.div_ceil(10) * 10 }

fn round_points_up(points: GamePoints) -> GamePoints { (points + 99) / 100 * 100 }

/// See: <https://riichi.wiki/Fu>
fn calc_regular_fu(
    _ruleset: &Ruleset,
    agari_kind: AgariKind,
    is_closed: bool,
    extra_fu: u8,
) -> u8 {
    use AgariKind::*;
    static TABLE: [[[u8; 2]; 2]; 2] = [
        // [open ron, closed ron], [open tsumo, closed tsumo]
        [  [30,       30        ], [30,         20          ]],  // pinfu-style
        [  [20,       30        ], [22,         22          ]],  // not pinfu
    ];
    let fu_before_rounding = extra_fu + TABLE
        [match extra_fu { 0 => 0, _ => 1 }]
        [match agari_kind { Ron => 0, Tsumo => 1 }]
        [is_closed as usize];
    round_fu_up(fu_before_rounding)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distribute_points_examples() {
        let ruleset = Ruleset::default();
        let basic_points = fu_han_formula(30, 4);
        assert_eq!(basic_points, 1920);
        
        assert_eq!(
            distribute_points(&ruleset, RoundId { kyoku: 0, honba: 0 },
                              true, P1, P1, basic_points),
            [-3900, 7900, -2000, -2000]);
        assert_eq!(
            distribute_points(&ruleset, RoundId { kyoku: 0, honba: 2 },
                              true, P1, P1, basic_points),
            [-4100, 8500, -2200, -2200]);

        assert_eq!(
            distribute_points(&ruleset, RoundId { kyoku: 0, honba: 0 },
                              true, P1, P2, basic_points),
            [0, 7700, -7700, 0]);
        assert_eq!(
            distribute_points(&ruleset, RoundId { kyoku: 0, honba: 1 },
                              true, P1, P2, basic_points),
            [0, 8000, -8000, 0]);
        assert_eq!(
            distribute_points(&ruleset, RoundId { kyoku: 0, honba: 0 },
                              true, P1, P0, basic_points),
            [-7700, 7700, 0, 0]);

        assert_eq!(
            distribute_points(&ruleset, RoundId { kyoku: 2, honba: 0 },
                              true, P2, P2, basic_points),
            [-3900, -3900, 11700, -3900]);
        assert_eq!(
            distribute_points(&ruleset, RoundId { kyoku: 2, honba: 0 },
                              true, P2, P3, basic_points),
            [0, 0, 11600, -11600]);
    }
}
