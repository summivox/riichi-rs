mod hand_common;
mod regular_wait_common;
mod yaku_detectors;

use itertools::Itertools;

use riichi_decomp::{IrregularWait, RegularWait, Wait, WaitSet};
use riichi_elements::prelude::*;

use crate::{
    engine::{
        utils::*,
    },
    model::*,
    rules::Ruleset,
    yaku::*,
};
use super::{
    scoring::*,
};
use self::{
    hand_common::*,
    regular_wait_common::*,
    yaku_detectors::*,
};

#[derive(Debug)]
pub struct AgariInput<'a> {
    pub round_id: RoundId,

    // from the winner
    pub winner: Player,
    pub closed_hand: &'a TileSet37,
    pub riichi: Option<Riichi>,
    pub melds: &'a [Meld],
    pub wait_set: &'a WaitSet,

    // from the contributor
    pub contributor: Player,
    pub incoming_is_kan: bool,
    pub action_is_kan: bool,
    pub winning_tile: Tile,

    // from the table
    pub is_first_chance: bool,
    pub is_last_draw: bool,
}

impl<'a> AgariInput<'a> {
    pub fn new(
        round_id: RoundId,
        state: &'a State,
        wait_set: &'a WaitSet,
        action: Action,
        winner: Player,
        contributor: Player,
    ) -> Self {
        let winner_i = winner.to_usize();
        AgariInput {
            round_id,

            winner,
            closed_hand: &state.closed_hands[winner_i],
            riichi: state.core.riichi[winner_i],
            melds: &state.melds[winner_i],
            wait_set,

            contributor,
            incoming_is_kan: state.core.incoming_meld.is_some_and(|m| m.is_kan()),
            action_is_kan: action.is_kan(),
            winning_tile: action.tile().unwrap(),  // assumed not NineKinds

            is_first_chance: is_first_chance(state),
            is_last_draw: is_last_draw(state),
        }
    }
}

pub fn agari_candidates(
    ruleset: &Ruleset,
    input: &AgariInput,
) -> Vec<AgariCandidate> {
    let hand_common = calc_hand_common(ruleset, input);

    let regular_waits = input.wait_set.regular.iter()
        .filter(|wait|
            wait.waiting_tile == input.winning_tile.to_normal())
        .map(|wait|
            (wait, calc_regular_wait_common(ruleset, input, &hand_common, wait)));

    let irregular_wait = input.wait_set.irregular.filter(|irregular|
        match irregular {
            IrregularWait::SevenPairs(t) | IrregularWait::ThirteenOrphans(t) =>
                *t == input.winning_tile.to_normal(),
            IrregularWait::ThirteenOrphansAll => true,
        });

    let mut candidates = regular_waits
        .filter_map(|(regular_wait, wait_common)|
            calc_regular_agari_candidate(ruleset, input, &hand_common, regular_wait, &wait_common))
        .collect_vec();
    candidates.extend(irregular_wait
        .and_then(|irregular|
            calc_irregular_agari_candidate(ruleset, input, &hand_common, irregular)));
    candidates
}

fn calc_regular_agari_candidate(
    ruleset: &Ruleset,
    input: &AgariInput,
    hand_common: &HandCommon,
    regular_wait: &RegularWait,
    wait_common: &RegularWaitCommon,
) -> Option<AgariCandidate> {
    let mut yaku_builder = YakuBuilder::new(ruleset);
    detect_yakus_for_regular(ruleset, &mut yaku_builder,
                             input, hand_common, regular_wait, wait_common);
    let yaku_values = yaku_builder.build();
    if yaku_values.is_empty() { return None; }
    let scoring = calc_scoring(ruleset,
                               &yaku_values,
                               Wait::Regular(*regular_wait),
                               DoraHits::default(),  // ignored for now
                               hand_common.agari_kind,
                               hand_common.is_closed,
                               wait_common.extra_fu);
    Some(AgariCandidate {
        wait: Wait::Regular(*regular_wait),
        yaku_values,
        scoring,
    })
}

fn calc_irregular_agari_candidate(
    ruleset: &Ruleset,
    input: &AgariInput,
    hand_common: &HandCommon,
    irregular: IrregularWait,
) -> Option<AgariCandidate> {
    let mut yaku_builder = YakuBuilder::new(ruleset);
    detect_yakus_for_irregular(ruleset, &mut yaku_builder,
                               input, hand_common, irregular);
    let yaku_values = yaku_builder.build();
    if yaku_values.is_empty() { return None; }
    let scoring = calc_scoring(ruleset,
                               &yaku_values,
                               Wait::Irregular( input.wait_set.irregular.unwrap()),
                               DoraHits::default(),  // ignored for now
                               hand_common.agari_kind,
                               hand_common.is_closed,
                               0);
    Some(AgariCandidate {
        wait: Wait::Irregular(irregular),
        yaku_values,
        scoring,
    })
}

#[cfg(test)]
mod tests;
