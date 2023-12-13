use num;
use num_derive;
use serde::{Deserialize, Serialize};
use std::ops::{Add, Range, Sub};

use num_derive::FromPrimitive;

use crate::composer::render::{AdhocRenderer, RenderEngine, Renderer, Result};
use crate::composer::{context::CompositionContext, CompositionElement, CompositionSegment};

pub fn renderers() -> RenderEngine {
    RenderEngine::new() + Instrument::renderer()
}

/// Instruments defined according to [GM1 Sound Set](https://www.midi.org/specifications-old/item/gm-level-1-sound-set)
#[derive(Debug, Hash, FromPrimitive, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum Instrument {
    // Piano (0..=7)
    AcousticGrandPiano,
    BrightAcousticPiano,
    ElectricGrandPiano,
    HonkyTonkPiano,
    ElectricPiano1,
    ElectricPiano2,
    Harpsichord,
    Clavi,

    // Chromatic Percussion (8..=15)
    Celesta,
    Glockenspiel,
    MusicBox,
    Vibraphone,
    Marimba,
    Xylophone,
    TubularBells,
    Dulcimer,

    // Organ (16..=23)
    DrawbarOrgan,
    PercussiveOrgan,
    RockOrgan,
    ChurchOrgan,
    ReedOrgan,
    Accordion,
    Harmonica,
    TangoAccordion,

    // Guitar (24..=31)
    AcousticGuitarNylon,
    AcousticGuitarSteel,
    ElectricGuitarJazz,
    ElectricGuitarClean,
    ElectricGuitarMuted,
    OverdrivenGuitar,
    DistortionGuitar,
    GuitarHarmonics,

    // Bass (32..=39)
    AcousticBass,
    ElectricBassFinger,
    ElectricBassPick,
    FretlessBass,
    SlapBass1,
    SlapBass2,
    SynthBass1,
    SynthBass2,

    // Strings (40..=47)
    Violin,
    Viola,
    Cello,
    Contrabass,
    TremoloStrings,
    PizzicatoStrings,
    OrchestralHarp,
    Timpani,

    // Ensemble (48..=55)
    StringEnsemble1,
    StringEnsemble2,
    SynthStrings1,
    SynthStrings2,
    ChoirAahs,
    ChoirOohs,
    SynthVoice,
    OrchestraHit,

    // Brass (56..=63)
    Trumpet,
    Trombone,
    Tuba,
    MutedTrumpet,
    FrenchHorn,
    BrassSection,
    SynthBrass1,
    SynthBrass2,

    // Reed (64..=71)
    SopranoSax,
    AltoSax,
    TenorSax,
    BaritoneSax,
    Oboe,
    EnglishHorn,
    Bassoon,
    Clarinet,

    // Pipe (72..=79)
    Piccolo,
    Flute,
    Recorder,
    PanFlute,
    BlownBottle,
    Shakuhachi,
    Whistle,
    Ocarina,

    // Synth Lead (80..=87)
    LeadSquare,
    LeadSawtooth,
    LeadCalliope,
    LeadChiff,
    LeadCharang,
    LeadVoice,
    LeadFifths,
    LeadBassPlusLead,

    // Synth Pad (88..=95)
    PadNewAge,
    PadWarm,
    PadPolySynth,
    PadChoir,
    PadBowed,
    PadMetallic,
    PadHalo,
    PadSweep,

    // Synth Effects (96..=103)
    FXRain,
    FXSoundtrack,
    FXCrystal,
    FXAtmosphere,
    FXBrightness,
    FXGoblins,
    FXEchoes,
    FXSciFi,

    // Ethnic (104..=111)
    Sitar,
    Banjo,
    Shamisen,
    Koto,
    Kalimba,
    BagPipe,
    Fiddle,
    Shanai,

    // Percussive (112..=119)
    TinkleBell,
    Agogo,
    SteelDrums,
    Woodblock,
    TaikoDrum,
    MelodicTom,
    SynthDrum,
    ReverseCymbal,

    // Sound Effects (120..127)
    GuitarFretNoise,
    BreathNoise,
    Seashore,
    BirdTweet,
    TelephoneRing,
    Helicopter,
    Applause,
    Gunshot,
}

#[typetag::serde(name = "midi::Instrument")]
impl CompositionElement for Instrument {}

impl Instrument {
    pub fn renderer() -> impl Renderer<Item = Self> {
        AdhocRenderer::from(
            |segment: &Self, time_range: &Range<i32>, _context: &CompositionContext| {
                Result::Ok(vec![CompositionSegment::new(
                    crate::composer::Instrument {
                        program: (*segment).into(),
                    },
                    time_range.clone(),
                )])
            },
        )
    }
}

/// ##Example
/// ```rust
/// # use redact_composer::musical::midi::{Instrument, Instruments};
/// #
/// let pianos = Instrument::AcousticGrandPiano + Instrument::BrightAcousticPiano;
/// assert_eq!(pianos, Instruments { programs: vec![Instrument::AcousticGrandPiano, Instrument::BrightAcousticPiano] });
/// ```
impl Add for Instrument {
    type Output = Instruments;

    fn add(self, rhs: Self) -> Self::Output {
        Instruments {
            programs: vec![self, rhs],
        }
    }
}

/// ##Example
/// ```rust
/// # use redact_composer::musical::midi::{Instrument, Instruments};
/// #
/// let pianos = Instrument::AcousticGrandPiano
///                 + Instruments { programs: vec![Instrument::BrightAcousticPiano] };
/// assert_eq!(pianos, Instruments { programs: vec![Instrument::AcousticGrandPiano, Instrument::BrightAcousticPiano] });
/// ```
impl Add<Instruments> for Instrument {
    type Output = Instruments;

    fn add(self, rhs: Instruments) -> Self::Output {
        Instruments {
            programs: vec![self],
        } + rhs
    }
}

/// ##Example
/// ```rust
/// # use redact_composer::musical::midi::{Instrument};
/// #
/// assert_eq!(Instrument::from(0u8), Instrument::AcousticGrandPiano);
/// ```
impl From<u8> for Instrument {
    fn from(value: u8) -> Self {
        num::FromPrimitive::from_u8(value).unwrap()
    }
}

impl From<Instrument> for u8 {
    fn from(value: Instrument) -> Self {
        value as u8
    }
}

/// A thin wrapper around a [`Vec<Instrument>`] with Add/Subtract operations.
#[derive(Debug, Clone, PartialEq)]
pub struct Instruments {
    pub programs: Vec<Instrument>,
}

impl Instruments {
    pub fn all() -> Instruments {
        Instruments {
            programs: (0..128).map(Instrument::from).collect(),
        }
    }

    pub fn piano() -> Instruments {
        Instruments {
            programs: (0..8).map(Instrument::from).collect(),
        }
    }

    pub fn tonal_percussive() -> Instruments {
        Instruments {
            programs: (8..16).map(Instrument::from).collect(),
        }
    }

    pub fn organ() -> Instruments {
        Instruments {
            programs: (16..24).map(Instrument::from).collect(),
        }
    }

    pub fn guitar() -> Instruments {
        Instruments {
            programs: (24..32).map(Instrument::from).collect(),
        }
    }

    pub fn bass() -> Instruments {
        Instruments {
            programs: (32..40).map(Instrument::from).collect(),
        }
    }

    pub fn strings() -> Instruments {
        Instruments {
            programs: (40..48).map(Instrument::from).collect(),
        }
    }

    pub fn ensemble() -> Instruments {
        Instruments {
            programs: (48..56).map(Instrument::from).collect(),
        }
    }

    pub fn brass() -> Instruments {
        Instruments {
            programs: (56..64).map(Instrument::from).collect(),
        }
    }

    pub fn reed() -> Instruments {
        Instruments {
            programs: (64..72).map(Instrument::from).collect(),
        }
    }

    pub fn pipe() -> Instruments {
        Instruments {
            programs: (72..80).map(Instrument::from).collect(),
        }
    }

    pub fn synth_lead() -> Instruments {
        Instruments {
            programs: (80..88).map(Instrument::from).collect(),
        }
    }

    pub fn synth_pad() -> Instruments {
        Instruments {
            programs: (88..96).map(Instrument::from).collect(),
        }
    }

    pub fn synth_fx() -> Instruments {
        Instruments {
            programs: (96..104).map(Instrument::from).collect(),
        }
    }

    pub fn ethnic() -> Instruments {
        Instruments {
            programs: (104..112).map(Instrument::from).collect(),
        }
    }

    pub fn percussive() -> Instruments {
        Instruments {
            programs: (112..120).map(Instrument::from).collect(),
        }
    }

    pub fn sound_fx() -> Instruments {
        Instruments {
            programs: (120..128).map(Instrument::from).collect(),
        }
    }

    pub fn melodic() -> Instruments {
        Self::all()
            - Self::percussive()
            - Self::sound_fx()
            - Self::synth_fx()
            - Instrument::Timpani
            - Instrument::TubularBells
            - Instrument::PadBowed
            - Instrument::LeadFifths
            - Instrument::OrchestraHit
            - Instrument::Kalimba
            - Instrument::GuitarHarmonics
    }
}

impl IntoIterator for Instruments {
    type Item = Instrument;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.programs.into_iter()
    }
}

/// ##Example
/// ```rust
/// # use redact_composer::musical::midi::{Instrument, Instruments};
/// #
/// let pianos = Instruments { programs: vec![Instrument::AcousticGrandPiano] }
///                         + Instruments { programs: vec![Instrument::BrightAcousticPiano] };
/// assert_eq!(pianos, Instruments { programs: vec![Instrument::AcousticGrandPiano, Instrument::BrightAcousticPiano] });
/// ```
impl Add for Instruments {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Instruments {
            programs: self.into_iter().chain(rhs.into_iter()).collect(),
        }
    }
}

/// ##Example
/// ```rust
/// # use redact_composer::musical::midi::{Instrument, Instruments};
/// #
/// let pianos = Instruments { programs: vec![Instrument::AcousticGrandPiano] }
///                         + Instrument::BrightAcousticPiano;
/// assert_eq!(pianos, Instruments { programs: vec![Instrument::AcousticGrandPiano, Instrument::BrightAcousticPiano] });
/// ```
impl Add<Instrument> for Instruments {
    type Output = Self;

    fn add(self, rhs: Instrument) -> Self::Output {
        Instruments {
            programs: self.into_iter().chain(vec![rhs]).collect(),
        }
    }
}

/// ##Example
/// ```rust
/// # use redact_composer::musical::midi::{Instrument, Instruments};
/// #
/// let no_violins = Instruments { programs: vec![Instrument::Violin, Instrument::Cello] }
///                         - Instruments { programs: vec![Instrument::Violin] };
/// assert_eq!(no_violins, Instruments { programs: vec![Instrument::Cello] });
/// ```
impl Sub for Instruments {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Instruments {
            programs: self
                .into_iter()
                .filter(|i| !rhs.programs.contains(i))
                .collect(),
        }
    }
}

/// ##Example
/// ```rust
/// # use redact_composer::musical::midi::{Instrument, Instruments};
/// #
/// let no_violins = Instruments { programs: vec![Instrument::Violin, Instrument::Cello] }
///                         - Instrument::Violin;
/// assert_eq!(no_violins, Instruments { programs: vec![Instrument::Cello] });
///
/// let instruments = Instruments { programs: vec![Instrument::AcousticGrandPiano, Instrument::BrightAcousticPiano] };
/// ```
impl Sub<Instrument> for Instruments {
    type Output = Self;

    fn sub(self, rhs: Instrument) -> Self::Output {
        Instruments {
            programs: self.into_iter().filter(|i| *i != rhs).collect(),
        }
    }
}

/// ##Example
/// ```rust
/// # use redact_composer::musical::midi::{Instrument, Instruments};
/// #
/// let instruments = Instruments { programs: vec![Instrument::AcousticGrandPiano, Instrument::BrightAcousticPiano] };
/// let vec_instruments: Vec<Instrument> = instruments.into();
/// assert_eq!(
///     vec_instruments,
///     vec![Instrument::AcousticGrandPiano, Instrument::BrightAcousticPiano]
/// );
/// ```
impl From<Instruments> for Vec<Instrument> {
    fn from(value: Instruments) -> Self {
        value.programs
    }
}

#[derive(Debug, Hash, FromPrimitive, PartialEq, Clone, Copy)]
pub enum DrumHit {
    AcousticBassDrum = 35,
    BassDrum,
    SideStick,
    AcousticSnare,
    HandClap,
    ElectricSnare,
    LowFloorTom,
    ClosedHiHat,
    HighFloorTom,
    PedalHiHat,
    LowTom,
    OpenHiHat,
    LowMidTom,
    HighMidTom,
    CrashCymbal1,
    HighTom,
    RideCymbal1,
    ChineseCymbal,
    RideBell,
    Tambourine,
    SplashCymbal,
    Cowbell,
    CrashCymbal2,
    Vibraslap,
    RideCymbal2,
    HighBongo,
    LowBongo,
    MuteHighConga,
    OpenHighConga,
    LowConga,
    HighTimbale,
    LowTimbale,
    HighAgogo,
    LowAgogo,
    Cabasa,
    Maracas,
    ShortWhistle,
    LongWhistle,
    ShortGuiro,
    LongGuiro,
    Claves,
    HighWoodblock,
    LowWoodblock,
    MuteCuica,
    OpenCuica,
    MuteTriangle,
    OpenTriangle,
}

impl From<u8> for DrumHit {
    fn from(value: u8) -> Self {
        num::FromPrimitive::from_u8(value).unwrap()
    }
}

impl From<DrumHit> for u8 {
    fn from(value: DrumHit) -> Self {
        value as u8
    }
}
