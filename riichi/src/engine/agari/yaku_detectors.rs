use riichi_decomp::{RegularWait, IrregularWait, WaitingKind};
use riichi_elements::prelude::*;

use crate::{
    common::{sort3},
    engine::utils::*,
    model::*,
    rules::Ruleset,
    yaku::*,
};
use super::{
    AgariInput,
    HandCommon,
    RegularWaitCommon,
};

pub fn detect_yakus_for_regular(
    ruleset: &Ruleset,
    yaku_builder: &mut YakuBuilder,
    input: &AgariInput,
    hand_common: &HandCommon,
    regular_wait: &RegularWait,
    wait_common: &RegularWaitCommon,
) {
    detect_pinfu(ruleset, yaku_builder,
                 wait_common.extra_fu,
                 hand_common.is_closed);
    detect_riichi(ruleset, yaku_builder,
                  input.riichi);
    detect_mentsumo(ruleset, yaku_builder,
                    hand_common.agari_kind,
                    input.melds);
    detect_rinshan(ruleset, yaku_builder,
                   hand_common.agari_kind,
                   input.incoming_is_kan);
    detect_chankan(ruleset, yaku_builder,
                   input.action_is_kan,
                   hand_common.agari_kind);
    detect_last_draw(ruleset, yaku_builder,
                     hand_common.agari_kind,
                     input.is_last_draw);
    detect_first_chance(ruleset, yaku_builder,
                        input.winner,
                        input.contributor,
                        input.round_id.button(),
                        input.is_first_chance,
                        hand_common.agari_kind);
    detect_hand_only_yakus(ruleset, yaku_builder,
                           &hand_common.all_tiles,
                           hand_common.is_closed);
    detect_winds(ruleset, yaku_builder,
                 &hand_common.all_tiles,
                 input.round_id,
                 input.winner);
    detect_chuuren(ruleset, yaku_builder,
                   &hand_common.all_tiles_packed,  // TODO(summivox): replace with `all_tiles`
                   input.winning_tile,
                   input.melds);
    detect_ankou(ruleset, yaku_builder,
                 hand_common.agari_kind,
                 input.melds,
                 regular_wait,
                 wait_common.wait_group);
    detect_kan(ruleset, yaku_builder,
               input.melds);
    detect_toitoi(ruleset, yaku_builder,
                  input.melds,
                  regular_wait,
                  wait_common.wait_group);
    detect_shuntsu(ruleset, yaku_builder,
                   input.melds,
                   regular_wait,
                   wait_common.wait_group,
                   hand_common.is_closed);
    detect_sanshokudoukou(ruleset, yaku_builder,
                          input.melds,
                          regular_wait,
                          wait_common.wait_group);
    detect_chanta(ruleset, yaku_builder,
                  input.melds,
                  &hand_common.all_tiles,
                  regular_wait,
                  wait_common.wait_group,
                  hand_common.is_closed);
}

pub fn detect_yakus_for_irregular(
    ruleset: &Ruleset,
    yaku_builder: &mut YakuBuilder,
    input: &AgariInput,
    hand_common: &HandCommon,
    irregular: IrregularWait,
) {
    detect_irregular(ruleset, yaku_builder,
                     irregular);
    detect_riichi(ruleset, yaku_builder,
                  input.riichi);
    detect_mentsumo(ruleset, yaku_builder,
                    hand_common.agari_kind,
                    input.melds);
    detect_rinshan(ruleset, yaku_builder,
                   hand_common.agari_kind,
                   input.incoming_is_kan);
    detect_chankan(ruleset, yaku_builder,
                   input.action_is_kan,
                   hand_common.agari_kind);
    detect_last_draw(ruleset, yaku_builder,
                     hand_common.agari_kind,
                     input.is_last_draw);
    detect_first_chance(ruleset, yaku_builder,
                        input.winner,
                        input.contributor,
                        input.round_id.button(),
                        input.is_first_chance,
                        hand_common.agari_kind);
    detect_hand_only_yakus(ruleset, yaku_builder,
                           &hand_common.all_tiles,
                           hand_common.is_closed);
}

fn detect_pinfu(
    _ruleset: &Ruleset,
    yaku_builder: &mut YakuBuilder,
    extra_fu: u8,
    is_closed: bool,
) {
    // This is trivial; we keep it here anyway for uniformity.
    if extra_fu == 0 && is_closed {
        yaku_builder.add(Yaku::Pinfu, 1);
    }
}

fn detect_irregular(
    _ruleset: &Ruleset,
    yaku_builder: &mut YakuBuilder,
    irregular: IrregularWait,
) {
    match irregular {
        IrregularWait::SevenPairs(_) =>
            yaku_builder.add(Yaku::Chiitoitsu, 2),
        IrregularWait::ThirteenOrphans(_) =>
            yaku_builder.add(Yaku::Kokushi, -1),
        // TODO(summivox): ruleset (double yakuman)
        IrregularWait::ThirteenOrphansAll =>
            yaku_builder.add(Yaku::Kokushi13, -1),
    }
}

fn detect_riichi(
    _ruleset: &Ruleset,
    yaku_builder: &mut YakuBuilder,
    riichi: Option<Riichi>,
) {
    if let Some(riichi) = riichi {
        if riichi.is_double {
            yaku_builder.add(Yaku::DoubleRiichi, 2);
        } else {
            yaku_builder.add(Yaku::Riichi, 1);
        }
        if riichi.is_ippatsu {
            yaku_builder.add(Yaku::Ippatsu, 1);
        }
    }
}

fn detect_mentsumo(
    _ruleset: &Ruleset,
    yaku_builder: &mut YakuBuilder,
    agari_kind: AgariKind,
    melds: &[Meld],
) {
    if melds.iter().all(|m| m.is_closed()) && agari_kind == AgariKind::Tsumo {
        yaku_builder.add(Yaku::Menzenchintsumohou, 1);
    }
}

fn detect_rinshan(
    _ruleset: &Ruleset,
    yaku_builder: &mut YakuBuilder,
    agari_kind: AgariKind,
    incoming_is_kan: bool,
) {
    if incoming_is_kan && agari_kind == AgariKind::Tsumo {
        yaku_builder.add(Yaku::Rinshankaihou, 1);
    }
}

fn detect_chankan(
    _ruleset: &Ruleset,
    yaku_builder: &mut YakuBuilder,
    action_is_kan: bool,
    agari_kind: AgariKind,
) {
    // NOTE: The kokushi-ankan interaction is handled by `check_reaction`.
    if agari_kind == AgariKind::Ron && action_is_kan {
        yaku_builder.add(Yaku::Chankan, 1);
    }
}

fn detect_last_draw(
    _ruleset: &Ruleset,
    yaku_builder: &mut YakuBuilder,
    agari_kind: AgariKind,
    is_last_draw: bool,
) {
    // NOTE: rinshan will override haitei through blocked yakus
    if is_last_draw {
        match agari_kind {
            AgariKind::Tsumo => yaku_builder.add(Yaku::Haiteimouyue, 1),
            AgariKind::Ron => yaku_builder.add(Yaku::Houteiraoyui, 1),
        }
    }
}

fn detect_first_chance(
    _ruleset: &Ruleset,
    yaku_builder: &mut YakuBuilder,
    winner: Player,
    contributor: Player,
    button: Player,
    is_init_abortable: bool,
    agari_kind: AgariKind,
) {
    if is_init_abortable {
        match agari_kind {
            AgariKind::Ron => {
                if contributor.to_u8() < winner.to_u8() {
                    yaku_builder.add(Yaku::Renhou, 4);
                }
            }
            AgariKind::Tsumo => {
                if winner == button {
                    yaku_builder.add(Yaku::Tenhou, -1);
                } else {
                    yaku_builder.add(Yaku::Chiihou, -1);
                }
            }
        }
    }
}

fn detect_hand_only_yakus(
    ruleset: &Ruleset,
    yaku_builder: &mut YakuBuilder,
    all_tiles: &TileSet37,
    is_closed: bool,
) {
    let (num_m, num_p, num_s, num_z) =
        (m_count(all_tiles), p_count(all_tiles), s_count(all_tiles), z_count(all_tiles));
    let one_nine = pure_terminal_count(all_tiles);
    let num_tiles: u8 = num_m + num_p + num_s + num_z;

    // tile categories
    if green_count(all_tiles) == num_tiles {
        yaku_builder.add(Yaku::Ryuuiisou, -1);
    } else if num_z + one_nine == 0 {
        if is_closed || ruleset.yaku_allow_open_tanyao {
            yaku_builder.add(Yaku::Tanyaochuu, 1);
        }
    } else if num_z == num_tiles {
        yaku_builder.add(Yaku::Tsuuiisou, -1);
    } else if one_nine == num_tiles {
        yaku_builder.add(Yaku::Chinroutou, -1);
    } else if num_z + one_nine == num_tiles {
        yaku_builder.add(Yaku::Honroutou, 2);
    }

    // individual dragon groups
    if all_tiles[31] >= 3 { yaku_builder.add(Yaku::SangenpaiHaku, 1); }
    if all_tiles[32] >= 3 { yaku_builder.add(Yaku::SangenpaiHatsu, 1); }
    if all_tiles[33] >= 3 { yaku_builder.add(Yaku::SangenpaiChun, 1); }

    // all dragons, all winds
    let dragons = sort3(all_tiles[31], all_tiles[32], all_tiles[33]);
    if dragons.0 >= 3 {
        yaku_builder.add(Yaku::Daisangen, -1);
    } else if dragons.0 == 2 && dragons.1 >= 3 {
        yaku_builder.add(Yaku::Shousangen, 2);
    } else {
        let mut winds = [all_tiles[27], all_tiles[28], all_tiles[29], all_tiles[30]];
        winds.sort();
        if winds[0] >= 3 {
            yaku_builder.add(Yaku::Daisuushi, -1);
        } else if winds[0] == 2 && winds[1] >= 3 {
            yaku_builder.add(Yaku::Shousuushi, -1);
        }
    }

    // flushes
    let (_a, b, c) = sort3(num_m, num_p, num_s);
    if b == 0 && c > 0 {
        if num_z == 0 {
            yaku_builder.add(Yaku::Chinniisou, if is_closed { 6 } else { 5 })
        } else {
            yaku_builder.add(Yaku::Honniisou, if is_closed { 3 } else { 2 })
        }
    }
}

fn detect_winds(
    _ruleset: &Ruleset,
    yaku_builder: &mut YakuBuilder,
    all_tiles: &TileSet37,
    round_id: RoundId,
    winner: Player,
) {
    match round_id.prevailing_wind().to_u8() {
        0 if all_tiles[27] >= 3 => yaku_builder.add(Yaku::BakazehaiE, 1),
        1 if all_tiles[28] >= 3 => yaku_builder.add(Yaku::BakazehaiS, 1),
        2 if all_tiles[29] >= 3 => yaku_builder.add(Yaku::BakazehaiW, 1),
        3 if all_tiles[30] >= 3 => yaku_builder.add(Yaku::BakazehaiN, 1),
        _ => {}
    }
    match round_id.self_wind_for_player(winner).to_u8() {
        0 if all_tiles[27] >= 3 => yaku_builder.add(Yaku::JikazehaiE, 1),
        1 if all_tiles[28] >= 3 => yaku_builder.add(Yaku::JikazehaiS, 1),
        2 if all_tiles[29] >= 3 => yaku_builder.add(Yaku::JikazehaiW, 1),
        3 if all_tiles[30] >= 3 => yaku_builder.add(Yaku::JikazehaiN, 1),
        _ => {}
    }
}

fn detect_chuuren(
    _ruleset: &Ruleset,
    yaku_builder: &mut YakuBuilder,
    all_tiles_packed: &[u32; 4],
    winning_tile: Tile,
    melds: &[Meld],
) {
    // NOTE: This is more strict than just `is_closed` --- Ankan is not even allowed.
    if !melds.is_empty() || winning_tile.suit() == 3 { return }

    if let Some(r_pos) = chuuren_agari(all_tiles_packed[winning_tile.suit() as usize]) {
        if r_pos == winning_tile.normal_num() - 1 {
            // TODO(summivox): ruleset (double yakuman)
            yaku_builder.add(Yaku::Junseichuurenpoutou, -1);
        } else {
            yaku_builder.add(Yaku::Chuurenpoutou, -1);
        }
    }
}

fn detect_ankou(
    _ruleset: &Ruleset,
    yaku_builder: &mut YakuBuilder,
    agari_kind: AgariKind,
    melds: &[Meld],
    regular_wait: &RegularWait,
    wait_group: Option<HandGroup>,
) {
    let mut num_ankou_complete =
        regular_wait.groups().filter(|g| matches!(g, HandGroup::Koutsu(_))).count() +
        melds.iter().filter(|m| m.is_closed()).count();
    // closed waiting koutsu also counts
    if let Some(HandGroup::Koutsu(_)) = wait_group
        && agari_kind == AgariKind::Tsumo
    {
        num_ankou_complete += 1;
    }
    match num_ankou_complete {
        4 => {
            if regular_wait.waiting_kind == WaitingKind::Tanki {
                // TODO(summivox): ruleset (double yakuman)
                yaku_builder.add(Yaku::SuuankouTanki, -1);
            } else {
                yaku_builder.add(Yaku::Suuankou, -1);
            }
        }
        3 => yaku_builder.add(Yaku::Sannankou, 2),
        _ => {}
    }
}

fn detect_kan(
    _ruleset: &Ruleset,
    yaku_builder: &mut YakuBuilder,
    melds: &[Meld],
) {
    let num_kan = melds.iter().filter(|m| m.is_kan()).count();
    match num_kan {
        4 => yaku_builder.add(Yaku::Suukantsu, -1),
        3 => yaku_builder.add(Yaku::Sankantsu, 2),
        _ => {}
    }
}

fn detect_toitoi(
    _ruleset: &Ruleset,
    yaku_builder: &mut YakuBuilder,
    melds: &[Meld],
    regular_wait: &RegularWait,
    wait_group: Option<HandGroup>,
) {
    if melds.iter().all(|m| !matches!(m, Meld::Chii(_))) &&
        regular_wait.groups().all(|g| matches!(g, HandGroup::Koutsu(_))) &&
        !matches!(wait_group, Some(HandGroup::Shuntsu(_))) {

        yaku_builder.add(Yaku::Toitoihou, 2);
    }
}

fn detect_shuntsu(
    _ruleset: &Ruleset,
    yaku_builder: &mut YakuBuilder,
    melds: &[Meld],
    regular_wait: &RegularWait,
    wait_group: Option<HandGroup>,
    is_closed: bool,
) {
    let mut mask = TileMask34::default();
    let mut peikou_mask = TileMask34::default();
    let mut num_peikou = 0;

    let mut update = |t: Tile| {
        if peikou_mask.has(t) {
            peikou_mask.clear(t);
            num_peikou += 1;
        } else {
            peikou_mask.set(t);
        }
        mask.set(t);
    };

    for m in melds.iter() {
        if let HandGroup::Shuntsu(t) = m.to_equivalent_group() {
            update(t);
        }
    }
    for g in regular_wait.groups() {
        if let HandGroup::Shuntsu(t) = g {
            update(t);
        }
    }
    if let Some(HandGroup::Shuntsu(t)) = wait_group {
        update(t);
    }

    if is_closed {
        match num_peikou {
            1 => yaku_builder.add(Yaku::Iipeikou, 1),
            2 => yaku_builder.add(Yaku::Ryanpeikou, 3),
            _ => {}
        }
    }
    if mask.0 & 0b001001001 == 0b001001001 ||
        mask.0 & (0b001001001 << 9) == (0b001001001 << 9) ||
        mask.0 & (0b001001001 << 18) == (0b001001001 << 18) {
        yaku_builder.add(Yaku::Ikkitsuukan, if is_closed { 2 } else { 1 });
    }
    let sanshoku =
        (mask.0 & 0b111111111) &
            ((mask.0 >> 9) & 0b111111111) &
            ((mask.0 >> 18) & 0b111111111);
    if sanshoku.is_power_of_two() {
        yaku_builder.add(Yaku::Sanshokudoujun, if is_closed { 2 } else { 1 });
    }
}

fn detect_sanshokudoukou(
    _ruleset: &Ruleset,
    yaku_builder: &mut YakuBuilder,
    melds: &[Meld],
    regular_wait: &RegularWait,
    wait_group: Option<HandGroup>,
) {
    let mut mask = TileMask34::default();
    for m in melds.iter() {
        if let HandGroup::Koutsu(t) = m.to_equivalent_group() {
            mask.set(t);
        }
    }
    for g in regular_wait.groups() {
        if let HandGroup::Koutsu(t) = g {
            mask.set(t);
        }
    }
    if let Some(HandGroup::Koutsu(t)) = wait_group {
        mask.set(t);
    }
    let sanshoku =
        (mask.0 & 0b111111111) &
            ((mask.0 >> 9) & 0b111111111) &
            ((mask.0 >> 18) & 0b111111111);
    if sanshoku.is_power_of_two() {
        yaku_builder.add(Yaku::Sanshokudoukou, 2);
    }
}

fn detect_chanta(
    _ruleset: &Ruleset,
    yaku_builder: &mut YakuBuilder,
    melds: &[Meld],
    all_tiles: &TileSet37,
    regular_wait: &RegularWait,
    wait_group: Option<HandGroup>,
    is_closed: bool,
) {

    let meld_chanta =
        melds.iter().map(|m| m.to_equivalent_group()).all(is_chanta);
    let closed_chanta =
        regular_wait.groups().all(is_chanta);
    let waiting_chanta =
        if let Some(g) = wait_group { is_chanta(g) } else { true };
    let pair_chanta =
        if let Some(t) = regular_wait.pair_or_tanki() { t.is_terminal() } else { true };

    if meld_chanta && closed_chanta && waiting_chanta && pair_chanta {
        if honor_count(all_tiles) == 0 {
            yaku_builder.add(Yaku::Junchantaiyaochuu,
                             if is_closed { 3 } else { 2 });
        } else {
            yaku_builder.add(Yaku::Honchantaiyaochuu,
                             if is_closed { 2 } else { 1 });
        }
    }
}

fn is_chanta(hand_group: HandGroup) -> bool {
    match hand_group {
        HandGroup::Koutsu(t) => t.is_terminal(),
        HandGroup::Shuntsu(t) => t.num() == 1 || t.num() == 7,
    }
}
