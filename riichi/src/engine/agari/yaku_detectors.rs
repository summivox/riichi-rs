use riichi_decomp_table::WaitingKind;
use crate::{
    analysis::RegularWait,
    common::*,
    engine::utils::*,
    model::*,
    Rules
};
use crate::analysis::IrregularWait;
use super::{
    AgariInput,
    HandCommon,
    RegularWaitCommon,
};

pub fn detect_yakus_for_regular(
    rules: &Rules,
    yaku_builder: &mut YakuBuilder,
    input: &AgariInput,
    hand_common: &HandCommon,
    regular_wait: &RegularWait,
    wait_common: &RegularWaitCommon,
) {
    detect_pinfu(rules, yaku_builder, wait_common.extra_fu);
    detect_riichi(rules, yaku_builder,
                  &input.riichi_flags);
    detect_mentsumo(rules, yaku_builder,
                    hand_common.agari_kind,
                    input.melds);
    detect_rinshan(rules, yaku_builder,
                   hand_common.agari_kind,
                   input.incoming_meld);
    detect_chankan(rules, yaku_builder,
                   input.action,
                   hand_common.agari_kind);
    detect_last_chance(rules, yaku_builder,
                       input.num_draws,
                       hand_common.agari_kind);
    detect_first_chance(rules, yaku_builder,
                        input.winner,
                        input.begin.round_id.button(),
                        input.is_init_abortable,
                        hand_common.agari_kind);
    detect_hand_only_yakus(rules, yaku_builder,
                           &hand_common.all_tiles,
                           hand_common.is_closed);
    detect_winds(rules, yaku_builder,
                 &hand_common.all_tiles,
                 input.begin.round_id,
                 input.winner);
    detect_chuuren(rules, yaku_builder,
                   &hand_common.all_tiles_packed,
                   hand_common.winning_tile,
                   hand_common.is_closed);
    detect_ankou(rules, yaku_builder,
                 hand_common.agari_kind,
                 regular_wait,
                 wait_common.wait_group);
    detect_kan(rules, yaku_builder,
               input.melds);
    detect_toitoi(rules, yaku_builder,
                  input.melds,
                  regular_wait,
                  wait_common.wait_group);
    detect_shuntsu(rules, yaku_builder,
                   input.melds,
                   regular_wait,
                   wait_common.wait_group,
                   hand_common.is_closed);
    detect_sanshokudoukou(rules, yaku_builder,
                          input.melds,
                          regular_wait,
                          wait_common.wait_group);
    detect_chanta(rules, yaku_builder,
                  input.melds,
                  &hand_common.all_tiles,
                  regular_wait,
                  wait_common.wait_group,
                  hand_common.is_closed);
}

pub fn detect_yakus_for_irregular(
    rules: &Rules,
    yaku_builder: &mut YakuBuilder,
    input: &AgariInput,
    hand_common: &HandCommon,
    irregular: IrregularWait,
) {
    detect_irregular(rules, yaku_builder,
                     irregular);
    detect_riichi(rules, yaku_builder,
                  &input.riichi_flags);
    detect_mentsumo(rules, yaku_builder,
                    hand_common.agari_kind,
                    input.melds);
    detect_rinshan(rules, yaku_builder,
                   hand_common.agari_kind,
                   input.incoming_meld);
    detect_chankan(rules, yaku_builder,
                   input.action,
                   hand_common.agari_kind);
    detect_last_chance(rules, yaku_builder,
                       input.num_draws,
                       hand_common.agari_kind);
    detect_first_chance(rules, yaku_builder,
                        input.winner,
                        input.begin.round_id.button(),
                        input.is_init_abortable,
                        hand_common.agari_kind);
    detect_hand_only_yakus(rules, yaku_builder,
                           &hand_common.all_tiles,
                           hand_common.is_closed);
}

fn detect_pinfu(
    _rules: &Rules,
    yaku_builder: &mut YakuBuilder,
    extra_fu: u8,
) {
    // This is trivial; we keep it here anyway for uniformity.
    if extra_fu == 0 {
        yaku_builder.add(Yaku::Pinfu, 1);
    }
}

fn detect_irregular(
    _rules: &Rules,
    yaku_builder: &mut YakuBuilder,
    irregular: IrregularWait,
) {
    match irregular {
        IrregularWait::SevenPairs(_) =>
            yaku_builder.add(Yaku::Chiitoitsu, 2),
        IrregularWait::ThirteenOrphans(_) =>
            yaku_builder.add(Yaku::Kokushi, -1),
        // TODO(summivox): rules (double yakuman)
        IrregularWait::ThirteenOrphansAll =>
            yaku_builder.add(Yaku::Kokushi13, -1),
    }
}

fn detect_riichi(
    _rules: &Rules,
    yaku_builder: &mut YakuBuilder,
    riichi_flags: &RiichiFlags,
) {
    if riichi_flags.is_active {
        if riichi_flags.is_double {
            yaku_builder.add(Yaku::DoubleRiichi, 2);
        } else {
            yaku_builder.add(Yaku::Riichi, 1);
        }
        if riichi_flags.is_ippatsu {
            yaku_builder.add(Yaku::Ippatsu, 1);
        }
    }
}

fn detect_mentsumo(
    _rules: &Rules,
    yaku_builder: &mut YakuBuilder,
    agari_kind: AgariKind,
    melds: &[Meld],
) {
    if melds.iter().all(|m| m.is_closed()) && agari_kind == AgariKind::Tsumo {
        yaku_builder.add(Yaku::Menzenchintsumohou, 1);
    }
}

fn detect_rinshan(
    _rules: &Rules,
    yaku_builder: &mut YakuBuilder,
    agari_kind: AgariKind,
    incoming_meld: Option<Meld>,
) {
    if let Some(meld) = incoming_meld {
        if meld.is_kan() && agari_kind == AgariKind::Tsumo {
            yaku_builder.add(Yaku::Rinshankaihou, 1);
        }
    }
}

fn detect_chankan(
    _rules: &Rules,
    yaku_builder: &mut YakuBuilder,
    action: Action,
    agari_kind: AgariKind,
) {
    // NOTE: The kokushi-ankan interaction is handled by `check_reaction`.
    if agari_kind == AgariKind::Ron && action.is_kan() {
        yaku_builder.add(Yaku::Chankan, 1);
    }
}

fn detect_last_chance(
    _rules: &Rules,
    yaku_builder: &mut YakuBuilder,
    num_draws: u8,
    agari_kind: AgariKind,
) {
    // NOTE: rinshan will override haitei
    if num_draws == wall::MAX_NUM_DRAWS {
        match agari_kind {
            AgariKind::Tsumo => yaku_builder.add(Yaku::Haiteiraoyue, 1),
            AgariKind::Ron => yaku_builder.add(Yaku::Houteiraoyui, 1),
        }
    }
}

fn detect_first_chance(
    _rules: &Rules,
    yaku_builder: &mut YakuBuilder,
    winner: Player,
    button: Player,
    is_init_abortable: bool,
    agari_kind: AgariKind,
) {
    if is_init_abortable {
        match agari_kind {
            AgariKind::Ron => {
                // TODO(summivox): rules (renhou)
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
    _rules: &Rules,
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
        // TODO(summivox): rules (kui-tan)
        yaku_builder.add(Yaku::Tanyaochuu, 1);
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
    _rules: &Rules,
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
        _ => panic!()
    }
    match round_id.self_wind_for_player(winner).to_u8() {
        0 if all_tiles[27] >= 3 => yaku_builder.add(Yaku::JikazehaiE, 1),
        1 if all_tiles[28] >= 3 => yaku_builder.add(Yaku::JikazehaiS, 1),
        2 if all_tiles[29] >= 3 => yaku_builder.add(Yaku::JikazehaiW, 1),
        3 if all_tiles[30] >= 3 => yaku_builder.add(Yaku::JikazehaiN, 1),
        _ => panic!()
    }
}

fn detect_chuuren(
    _rules: &Rules,
    yaku_builder: &mut YakuBuilder,
    all_tiles_packed: &[u32; 4],
    winning_tile: Tile,
    is_closed: bool,
) {
    if !is_closed || winning_tile.suit() == 3 { return }
    let h = all_tiles_packed[winning_tile.suit() as usize];
    // check h is at least 0o311111113 (all bins must apply)
    if (h + 0o133333331) & 0o444444444 != 0o444444444 { return }
    // subtract 0o311111113; now only 1 shall remain (full closed hand, n == 14)
    let r = h - 0o311111113;
    assert!(r.is_power_of_two());
    let r_pos = r.trailing_zeros() as u8 / 3;
    if r_pos == winning_tile.normal_num() - 1 {
        // TODO(summivox): rules (double yakuman)
        yaku_builder.add(Yaku::Junseichuurenpoutou, -1);
    } else {
        yaku_builder.add(Yaku::Chuurenpoutou, -1);
    }
}

fn detect_ankou(
    _rules: &Rules,
    yaku_builder: &mut YakuBuilder,
    agari_kind: AgariKind,
    regular_wait: &RegularWait,
    wait_group: Option<HandGroup>,
) {
    let mut num_ankou_complete =
        regular_wait.groups().filter(|g| matches!(g, HandGroup::Koutsu(_))).count();
    // closed waiting koutsu also counts
    // TODO(summivox): if-let-chain
    if let Some(HandGroup::Koutsu(_)) = wait_group {
        if agari_kind == AgariKind::Tsumo {
            num_ankou_complete += 1;
        }
    }
    match num_ankou_complete {
        4 => {
            if regular_wait.waiting_kind == WaitingKind::Tanki {
                // TODO(summivox): rules (double yakuman)
                yaku_builder.add(Yaku::SuuankouTanki, -1);
            } else {
                yaku_builder.add(Yaku::Suuankou, -1);
            }
        }
        3 => yaku_builder.add(Yaku::Sannankou, -1),
        _ => {}
    }
}

fn detect_kan(
    _rules: &Rules,
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
    _rules: &Rules,
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
    _rules: &Rules,
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
    _rules: &Rules,
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
    _rules: &Rules,
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
        if let Some(g) = wait_group { is_chanta(g) } else { false };
    if meld_chanta && closed_chanta && waiting_chanta {
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
        HandGroup::Koutsu(t) => t.num() == 1 || t.num() == 9,
        HandGroup::Shuntsu(t) => t.num() == 1 || t.num() == 7,
    }
}
