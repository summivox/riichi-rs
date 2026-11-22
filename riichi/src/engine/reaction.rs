use thiserror::Error;

use riichi_decomp::IrregularWait;
use riichi_elements::prelude::*;

use crate::{
    model::*,
    rules::Ruleset
};
use super::{
    agari::*,
    EngineCache,
    utils::*,
};

#[derive(Error, Debug)]
pub enum ReactionError {
    #[error("No action to react to.")]
    NoAction,

    #[error("The action is terminal; no reactions possible.")]
    TerminalAction,

    #[error("Cannot declare an open meld under Riichi.")]
    OpenMeldUnderRiichi,

    #[error("Cannot declare an open meld on the last draw")]
    MeldOnLastDraw,

    #[error("Tile {0} does not exist in the closed hand.")]
    TileNotExist(Tile),

    #[error("You can only call a discarded tile (is actually {0:?})")]
    NotDiscard(Action),

    #[error("Can only Chii on the previous player's discard.")]
    CanOnlyChiiPrevPlayer,

    #[error("Cannot Chii {2} with own {0}{1} (own may not exist).")]
    InvalidChii(Tile, Tile, Tile),

    #[error("Cannot Pon {2} with own {0}{1} (own may not exist).")]
    InvalidPon(Tile, Tile, Tile),

    #[error("Cannot Daiminkan.")]
    InvalidDaiminkan,

    #[error("No Ron when you are furiten: {0:?}")]
    Furiten(FuritenFlags),

    #[error("Cannot Ron (not waiting or no Yaku).")]
    CannotRonAgari,

    #[error("Cannot Ron over Ankan (kokushi excepted).")]
    CannotRonAgariOverAnkan,
}

pub fn check_reaction(
    begin: &RoundBegin,
    state: &State,
    action: Action,
    reactor: Player,
    reaction: Reaction,
    cache: &mut EngineCache,
) -> Result<(), ReactionError> {
    use ReactionError::*;

    if action.is_terminal() { return Err(TerminalAction); }

    let actor = state.core.actor;
    let reactor_i = reactor.to_usize();
    let hand = &state.closed_hands[reactor_i];

    match reaction {
        Reaction::Chii(own0, own1) => {
            if state.core.riichi[reactor_i].is_some() { return Err(OpenMeldUnderRiichi); }
            if is_last_draw(state) { return Err(MeldOnLastDraw); }
            if actor.succ() != reactor {
                return Err(CanOnlyChiiPrevPlayer);
            }
            if let Action::Discard(discard) = action {
                let called = discard.tile;
                if let Some(chii) = Chii::from_tiles(own0, own1, called)
                    && chii.is_in_hand(hand)
                {
                    cache.meld[reactor_i] = Some(Meld::Chii(chii));
                } else {
                    return Err(InvalidChii(own0, own1, called));
                }
            } else {
                return Err(NotDiscard(action));
            }
        }

        Reaction::Pon(own0, own1) => {
            if state.core.riichi[reactor_i].is_some() { return Err(OpenMeldUnderRiichi); }
            if is_last_draw(state) { return Err(MeldOnLastDraw); }
            if let Action::Discard(discard) = action {
                let called = discard.tile;
                let dir = actor.sub(reactor);
                if let Some(pon) = Pon::from_tiles_dir(own0, own1, called, dir)
                    && pon.is_in_hand(hand)
                {
                    cache.meld[reactor_i] = Some(Meld::Pon(pon));
                } else {
                    return Err(InvalidPon(own0, own1, called));
                }
            } else {
                return Err(NotDiscard(action));
            }
        }

        Reaction::Daiminkan => {
            if state.core.riichi[reactor_i].is_some() { return Err(OpenMeldUnderRiichi); }
            if is_last_draw(state) { return Err(MeldOnLastDraw); }
            if let Action::Discard(discard) = action {
                let called = discard.tile;
                let dir = actor.sub(reactor);
                if let Some(daiminkan) = Daiminkan::from_hand_dir(hand, called, dir) {
                    cache.meld[reactor_i] = Some(Meld::Daiminkan(daiminkan));
                } else {
                    return Err(InvalidDaiminkan);
                }
            } else {
                return Err(NotDiscard(action));
            }
        }

        Reaction::RonAgari => {
            if state.core.furiten[reactor_i].any() {
                return Err(Furiten(state.core.furiten[reactor_i]));
            }
            if matches!(action, Action::Ankan(_)) &&
                !matches!(cache.wait[reactor_i].irregular,
                     Some(IrregularWait::ThirteenOrphans(_)) |
                     Some(IrregularWait::ThirteenOrphansAll)) {
                return Err(CannotRonAgariOverAnkan);
            }
            let agari_input = AgariInput::new(
                begin.round_id,
                state,
                &cache.wait[reactor_i],
                action,
                reactor,
                actor,
            );
            let candidates = agari_candidates(&begin.ruleset, &agari_input);
            if candidates.is_empty() { return Err(CannotRonAgari); }
            cache.win[reactor_i] = candidates;
        }
    }
    Ok(())
}

pub fn resolve_reaction(
    ruleset: &Ruleset,
    state: &State,
    action: Action,
    reactions: &[Option<Reaction>; 4],
) -> (ActionResult, Option<(Player, Reaction)>) {
    let actor = state.core.actor;

    // Handle in-turn voluntary termination.
    match action {
        Action::TsumoAgari(_) => return (ActionResult::Agari(AgariKind::Tsumo), None),
        Action::AbortNineKinds => return (ActionResult::Abort(AbortReason::NineKinds), None),
        _ => {}
    }

    // Find the highest priority reaction.
    let reactor_reaction = other_players_after(actor).into_iter()
        .flat_map(|reactor|
            reactions[reactor.to_usize()].map(|reaction| (reactor, reaction)))
        .max_by_key(|(_reactor, reaction)| *reaction);

    let result = match reactor_reaction {
        // Meld can be preempted by:
        // - four riichi
        // - four kan
        // - wall exhausted (see <https://riichi.wiki/Haitei_raoyue_and_houtei_raoyui>)
        //
        // Meld does not conflict with:
        // - four wind: 4th wind cannot be called
        Some((reactor, Reaction::Chii(_, _))) |
        Some((reactor, Reaction::Pon(_, _))) |
        Some((reactor, Reaction::Daiminkan)) =>
            (ActionResult::CalledBy(reactor), reactor_reaction),

        // Ron takes precedence over everything else at this point.
        Some((_reactor, Reaction::RonAgari)) => {
            let num_rons = reactions.iter().filter(|&&reaction|
                reaction == Some(Reaction::RonAgari)).count();
            if num_rons <= ruleset.ron_max_num_players as usize {
                return (ActionResult::Agari(AgariKind::Ron), reactor_reaction);
            } else {
                let reason = match num_rons {
                    3 => AbortReason::TripleRon,
                    2 => AbortReason::DoubleRon,
                    _ => panic!("ruleset is invalid")
                };
                return (ActionResult::Abort(reason), reactor_reaction);
            }
        }

        None => (ActionResult::Pass, None),
    };

    if is_aborted_four_wind(state, action) {
        return (ActionResult::Abort(AbortReason::FourWind), None);
    }
    if is_aborted_four_riichi(state, action) {
        return (ActionResult::Abort(AbortReason::FourRiichi), None);
    }
    // TODO(summivox): ruleset (4-kan judgment point)
    if is_aborted_four_kan(
        state, action, reactor_reaction.map(|(_reactor, reaction)| reaction)) {
        return (ActionResult::Abort(AbortReason::FourKan), None);
    }
    if result.0 == ActionResult::Pass && is_last_draw(state) {
        for player in other_players_after(actor) {
            if is_nagashi_mangan(state, player) {
                return (ActionResult::Abort(AbortReason::NagashiMangan), None);
            }
        }
        if is_nagashi_mangan(state, actor) {
            // The last discard has not been committed to the river, but we still need to take it
            // into consideration!
            if let Action::Discard(Discard { tile, .. }) = action
                && !tile.is_terminal()
            {
                return (ActionResult::Abort(AbortReason::WallExhausted), None);
            }
            return (ActionResult::Abort(AbortReason::NagashiMangan), None);
        }
        return (ActionResult::Abort(AbortReason::WallExhausted), None);
    }

    result
}
