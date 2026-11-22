use std::{
    fmt::Formatter,
};

use itertools::Itertools;
use serde::{de::{Deserialize, Deserializer, SeqAccess, Visitor, Error}, Serialize, Serializer};
use serde::ser::SerializeSeq;
use serde_json::Value;

use riichi_elements::prelude::*;

use crate::{
    model::*,
};
use super::{
    scoring::*,
    strings::*,
};

#[derive(Clone, Debug, Default)]
#[cfg_attr(test, derive(Eq, PartialEq))]  // No need to compare other than in tests.
pub struct TenhouEndInfo {
    pub result: ActionResult,
    pub overall_delta: [GamePoints; 4],
    pub agari: Vec<TenhouAgariResult>,
}

#[derive(Clone, Debug)]
#[cfg_attr(test, derive(Eq, PartialEq))]  // No need to compare other than in tests.
pub struct TenhouAgariResult {
    pub winner: Player,
    pub contributor: Player,
    pub liable_player: Player,
    pub points_delta_after_pot: [GamePoints; 4],
    pub scoring: TenhouScoring,
    pub details: Vec<YakuOrDora>,
}

impl Serialize for TenhouEndInfo {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error> where S: Serializer {
        match self.result {
            ActionResult::Agari(_) => {
                let mut seq = s.serialize_seq(Some(1 + 2 * self.agari.len()))?;
                seq.serialize_element(action_result_to_str(self.result))?;
                for agari in self.agari.iter() {
                    seq.serialize_element(&agari.points_delta_after_pot)?;
                    seq.serialize_element(agari)?;
                }
                seq.end()
            }
            ActionResult::Abort(AbortReason::WallExhausted) |
            ActionResult::Abort(AbortReason::NagashiMangan) => {
                let mut seq = s.serialize_seq(Some(1))?;
                seq.serialize_element(action_result_to_str(self.result))?;
                seq.serialize_element(&self.overall_delta)?;
                seq.end()
            }
            _ => {
                let mut seq = s.serialize_seq(Some(1))?;
                seq.serialize_element(action_result_to_str(self.result))?;
                seq.end()
            }
        }
    }
}

impl Serialize for TenhouAgariResult {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut seq = s.serialize_seq(Some(4 + self.details.len()))?;
        seq.serialize_element(&self.winner.to_u8())?;
        seq.serialize_element(&self.contributor.to_u8())?;
        seq.serialize_element(&self.liable_player.to_u8())?;
        seq.serialize_element(&self.scoring.to_string())?;
        for yaku_or_dora in self.details.iter() {
            seq.serialize_element(&yaku_or_dora.to_string())?;
        }
        seq.end()
    }
}

impl<'de> Deserialize<'de> for TenhouEndInfo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        deserializer.deserialize_seq(EndInfoVisitor)
    }
}

struct EndInfoVisitor;

impl<'de> Visitor<'de> for EndInfoVisitor {
    type Value = TenhouEndInfo;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        write!(formatter,
            "a tenhou6 result array with at least 1 string element describing the overall result\
            of the round"
        )
    }

    //noinspection ALL
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error> where A: SeqAccess<'de> {
        let result_str: String = seq.next_element()?.ok_or_else(|| Error::custom("no result str"))?;
        if result_str != AGARI_STR {
            // This has to be an abort result.
            let abort_reason = abort_from_str(&result_str)
                .ok_or_else(|| Error::custom("unrecognized result"))?;
            match abort_reason {
                AbortReason::WallExhausted | AbortReason::NagashiMangan => {
                    if let Some(Value::Array(delta_arr)) = seq.next_element::<Value>()?
                        && delta_arr.len() == 4
                    {
                        return Ok(TenhouEndInfo {
                            result: ActionResult::Abort(abort_reason),
                            overall_delta: [
                                delta_arr[0].as_i64().unwrap_or(0) as GamePoints,
                                delta_arr[1].as_i64().unwrap_or(0) as GamePoints,
                                delta_arr[2].as_i64().unwrap_or(0) as GamePoints,
                                delta_arr[3].as_i64().unwrap_or(0) as GamePoints,
                            ],
                            agari: vec![],
                        });
                    }
                    if result_str == NONE_WAITING || result_str == ALL_WAITING {
                        return Ok(TenhouEndInfo {
                            result: ActionResult::Abort(abort_reason),
                            overall_delta: [0; 4],
                            agari: vec![],
                        });
                    }
                    return Err(Error::custom("invalid delta"));
                }
                _ => return Ok(TenhouEndInfo {
                    result: ActionResult::Abort(abort_reason),
                    overall_delta: [0; 4],
                    agari: vec![],
                }),
            }
        }
        let mut agari_results: Vec<TenhouAgariResult> = vec![];
        while let (
            Some(Value::Array(delta_arr)),
            Some(Value::Array(details_arr)),
        ) =
        (
            seq.next_element::<Value>()?,
            seq.next_element::<Value>()?
        ) {
            // TODO(summivox): separate visitor for each TenhouAgariResult, (use RawValue?)
            let mut delta = [0; 4];
            if delta_arr.len() == 4 {
                for i in 0..4 {
                    delta[i] = delta_arr[i].as_i64().unwrap_or(0) as GamePoints;
                }
            }
            if details_arr.len() >= 4 {
                agari_results.push(TenhouAgariResult {
                    winner: Player::new(details_arr[0].as_i64().unwrap_or(0) as u8),
                    contributor: Player::new(details_arr[1].as_i64().unwrap_or(0) as u8),
                    liable_player: Player::new(details_arr[2].as_i64().unwrap_or(0) as u8),

                    points_delta_after_pot: delta,

                    scoring: details_arr[3].as_str()
                        .ok_or_else(|| Error::custom("invalid score"))?
                        .parse::<TenhouScoring>()
                        .map_err(|_| Error::custom("invalid score"))?,

                    details: details_arr[4..].iter().filter_map(|value|
                        value.as_str().and_then(parse_yaku_or_dora)).collect_vec(),
                })
            }
        }
        let agari_kind = if agari_results[0].winner == agari_results[0].contributor {
            AgariKind::Tsumo
        } else {
            AgariKind::Ron
        };
        Ok(TenhouEndInfo {
            result: ActionResult::Agari(agari_kind),
            overall_delta: agari_results.iter().fold(
                [0, 0, 0, 0],
                |mut delta, b| {
                    for i in 0..4 { delta[i] += b.points_delta_after_pot[i]; }
                    delta
                }),
            agari: agari_results,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::yaku::Yaku;
    use super::*;

    #[test]
    fn abort_example() {
        let end_info_struct = TenhouEndInfo {
            result: ActionResult::Abort(AbortReason::FourRiichi),
            overall_delta: [0; 4],
            agari: vec![],
        };
        let end_info_json = serde_json::json!(["四家立直"]);
        let serialized = serde_json::to_value(&end_info_struct).unwrap();
        let deserialized: TenhouEndInfo = serde_json::from_value(end_info_json.clone()).unwrap();
        assert_eq!(end_info_json, serialized);
        assert_eq!(end_info_struct, deserialized);
    }

    #[test]
    fn exhaust_example() {
        let end_info_struct = TenhouEndInfo {
            result: ActionResult::Abort(AbortReason::NagashiMangan),
            overall_delta: [-4000, -4000, 12000, -4000],
            agari: vec![],
        };
        let end_info_json = serde_json::json!(["流し満貫", [-4000, -4000, 12000, -4000]]);
        let serialized = serde_json::to_value(&end_info_struct).unwrap();
        let deserialized: TenhouEndInfo = serde_json::from_value(end_info_json.clone()).unwrap();
        assert_eq!(end_info_json, serialized);
        assert_eq!(end_info_struct, deserialized);
    }

    #[test]
    fn ron_2_example() {
        let end_info_struct = TenhouEndInfo {
            result: ActionResult::Agari(AgariKind::Ron),
            overall_delta: [13000, 0, 2000, -14000],
            agari: vec![
                TenhouAgariResult {
                    winner: P0,
                    contributor: P3,
                    liable_player: P0,
                    points_delta_after_pot: [13000, 0, 0, -12000],
                    scoring: TenhouScoring {
                        kind: TenhouScoringKind::Haneman,
                        payout: TenhouPayout::Ron(12000),
                    },
                    details: vec![
                        YakuOrDora::Yaku(Yaku::Riichi, 1),
                        YakuOrDora::Dora(3),
                        YakuOrDora::AkaDora(2),
                    ],
                },
                TenhouAgariResult {
                    winner: P2,
                    contributor: P3,
                    liable_player: P2,
                    points_delta_after_pot: [0, 0, 2000, -2000],
                    scoring: TenhouScoring {
                        kind: TenhouScoringKind::HanFu { han: 2, fu: 30 },
                        payout: TenhouPayout::Ron(2000),
                    },
                    details: vec![
                        YakuOrDora::Yaku(Yaku::SangenpaiHatsu, 1),
                        YakuOrDora::Dora(1),
                    ],
                },
            ],
        };
        let end_info_json = serde_json::json!([
            "和了",
            [13000, 0, 0, -12000],
            [0, 3, 0, "跳満12000点", "立直(1飜)", "ドラ(3飜)", "赤ドラ(2飜)"],
            [0, 0, 2000, -2000],
            [2, 3, 2, "30符2飜2000点", "役牌 發(1飜)", "ドラ(1飜)"]
        ]);
        let serialized = serde_json::to_value(&end_info_struct).unwrap();
        let deserialized: TenhouEndInfo = serde_json::from_value(end_info_json.clone()).unwrap();
        assert_eq!(end_info_json, serialized);
        assert_eq!(end_info_struct, deserialized);
    }
}
