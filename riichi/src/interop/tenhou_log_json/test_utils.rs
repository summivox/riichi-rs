use itertools::Itertools;
use rand::prelude::*;

use crate::prelude::*;
use crate::engine::utils::calc_pot_delta;
use super::*;

/// Fully simulate/replay a [`RecoveredRound`] through our [`Engine`], validating the states along
/// the way. Useful for cross-checking the implementation of our [`Engine`].
pub fn run_a_round(
    num_reds: [u8; 3],
    recovered: &RecoveredRound,
    end_info: &TenhouEndInfo
) -> RoundHistory {
    let lite = &recovered.history;
    println!("\n{:?}", lite.begin.round_id);

    let mut engine = Engine::new();

    let mut begin = lite.begin.clone();
    let mut missing_tiles = wall::get_missing_tiles_in_partial_wall(
        &recovered.known_wall, num_reds).iter_tiles().collect_vec();
    missing_tiles[..].shuffle(&mut rand::rng());
    begin.wall = wall::fill_missing_tiles_in_partial_wall(
        &recovered.known_wall, missing_tiles.into_iter());
    log::debug!("{}", begin.wall.display());

    let mut full = RoundHistory {
        begin: begin.clone(),
        steps: vec![],
        ron: lite.ron,
    };

    engine.begin_round(begin);
    let mut last_step = None;
    for (seq, action_reaction) in lite.action_reactions.iter().enumerate() {
        // println!("{}", engine.state().core);
        // println!("{}", action_reaction);
        assert_eq!(engine.state().core.seq, seq as u8);
        assert_eq!(engine.state().core.actor, action_reaction.actor);
        engine.register_action(action_reaction.action).unwrap();
        if let Some((reactor, reaction)) = action_reaction.reactor_reaction {
            engine.register_reaction(reactor, reaction).unwrap();
        }
        if seq == lite.action_reactions.len() - 1 {
            // handle multi-ron
            let ron = &mut full.ron;
            if recovered.final_result == ActionResult::Abort(AbortReason::TripleRon) {
                for p in other_players_after(action_reaction.actor) {
                    ron[p.to_usize()] = true;
                }
            }
            for i in 0..4 {
                if ron[i] {
                    engine.register_reaction(
                        Player::new(i as u8),
                        Reaction::RonAgari,
                    ).unwrap();
                }
            }
        }
        let step = engine.step();
        full.steps.push(step.clone());
        last_step = Some(step);
    }
    if let Some(step) = last_step {
        match step.action_result {
            ActionResult::Abort(abort_reason) => {
                log::info!("engine says: {:?}", abort_reason);
            }
            ActionResult::Agari(agari_kind) => {
                log::info!("engine says: {:?}", agari_kind);

                let end = engine.end().clone().unwrap();
                println!("{:?}", end.agari_result);

                // Deduct newly added pot from players under riichi.
                // They are not included anyway.
                let mut delta = end.points_delta;
                let pot_delta = calc_pot_delta(&engine.state().core.riichi);
                for i in 0..4 { delta[i] -= pot_delta[i]; }

                // Exclude cases where Pao / Liability apply.
                if end_info.agari.iter().all(|x| x.liable_player == x.winner) {
                    assert_eq!(delta, end_info.overall_delta);
                }
            }
            _ => {}
        }
        assert_eq!(step.action_result, recovered.final_result);
    }
    full
}
