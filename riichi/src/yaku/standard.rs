use once_cell::sync::Lazy;
use rustc_hash::FxHashSet as HashSet;

use super::Yaku::{self, *};

/// The set of _standard_ [`Yaku`]s, according to this crate.
/// Serves as the definition of "standard" for the allow-/block-lists in [`crate::rules::Ruleset`].
pub static STANDARD_YAKU: Lazy<HashSet<Yaku>> = Lazy::new(|| {
    HashSet::from_iter([
        Menzenchintsumohou,
        Riichi,
        Ippatsu,
        Chankan,
        Rinshankaihou,
        Haiteimouyue,
        Houteiraoyui,
        Pinfu,
        Tanyaochuu,
        Iipeikou,
        JikazehaiE,
        JikazehaiS,
        JikazehaiW,
        JikazehaiN,
        BakazehaiE,
        BakazehaiS,
        BakazehaiW,
        BakazehaiN,
        SangenpaiHaku,
        SangenpaiHatsu,
        SangenpaiChun,
        DoubleRiichi,
        Chiitoitsu,
        Honchantaiyaochuu,
        Ikkitsuukan,
        Sanshokudoujun,
        Sanshokudoukou,
        Sankantsu,
        Toitoihou,
        Sannankou,
        Shousangen,
        Honroutou,
        Ryanpeikou,
        Junchantaiyaochuu,
        Honniisou,
        Chinniisou,
        Tenhou,
        Chiihou,
        Daisangen,
        Suuankou,
        SuuankouTanki,
        Tsuuiisou,
        Ryuuiisou,
        Chinroutou,
        Chuurenpoutou,
        Junseichuurenpoutou,
        Kokushi,
        Kokushi13,
        Daisuushi,
        Shousuushi,
        Suukantsu,
    ])
});