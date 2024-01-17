/// General MIDI Level 1 types.
use num;
use num_derive;
use num_derive::FromPrimitive;
use std::ops::{Add, Sub};

use crate::elements::Program;
use redact_composer_core::{derive::Element, IntoCompositionSegment};
use redact_composer_core::{
    elements::PlayNote,
    render::{AdhocRenderer, RenderEngine, Renderer, Result},
    Segment,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Default renderers used for GM [`Element`](redact_composer_core::Element)s
/// ([`Instrument`] and [`DrumHit`]).
pub fn renderers() -> RenderEngine {
    RenderEngine::new() + Instrument::renderer() + DrumHit::renderer()
}

/// Types implementing [`Element`](redact_composer_core::Element).
pub mod elements {
    pub use super::{DrumHit, Instrument};
}

/// Instruments defined according to
/// [GM1 Sound Set](https://www.midi.org/specifications-old/item/gm-level-1-sound-set)
#[derive(Element, Debug, Hash, FromPrimitive, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[allow(missing_docs)]
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

impl From<Instrument> for Program {
    fn from(value: Instrument) -> Program {
        Program(value as u8)
    }
}

impl Instrument {
    /// Renderer that render an [`Instrument`] segment as a [`Program`] with the same timing.
    pub fn renderer() -> impl Renderer<Element = Self> {
        AdhocRenderer::<Self>::new(|segment, _| {
            Result::Ok(vec![
                Program::from(*segment.element).into_segment(segment.timing)
            ])
        })
    }
}

/// ##Example
/// ```rust
/// # use redact_composer_midi::gm::elements::Instrument;
/// # use redact_composer_midi::gm::Instruments;
/// #
/// let pianos = Instrument::AcousticGrandPiano + Instrument::BrightAcousticPiano;
/// assert_eq!(pianos, Instruments { instruments: vec![Instrument::AcousticGrandPiano, Instrument::BrightAcousticPiano] });
/// ```
impl Add for Instrument {
    type Output = Instruments;

    fn add(self, rhs: Self) -> Self::Output {
        Instruments {
            instruments: vec![self, rhs],
        }
    }
}

/// ##Example
/// ```rust
/// # use redact_composer_midi::gm::elements::Instrument;
/// # use redact_composer_midi::gm::Instruments;
/// #
/// let pianos = Instrument::AcousticGrandPiano
///                 + Instruments { instruments: vec![Instrument::BrightAcousticPiano] };
/// assert_eq!(pianos, Instruments { instruments: vec![Instrument::AcousticGrandPiano, Instrument::BrightAcousticPiano] });
/// ```
impl Add<Instruments> for Instrument {
    type Output = Instruments;

    fn add(self, rhs: Instruments) -> Self::Output {
        Instruments {
            instruments: vec![self],
        } + rhs
    }
}

/// ##Example
/// ```rust
/// # use redact_composer_midi::gm::elements::Instrument;
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
    /// A list of instruments.
    pub instruments: Vec<Instrument>,
}

impl Instruments {
    /// All instruments.
    pub fn all() -> Instruments {
        Instruments {
            instruments: (0..128).map(Instrument::from).collect(),
        }
    }

    /// GM1 piano instruments.
    pub fn piano() -> Instruments {
        Instruments {
            instruments: (0..8).map(Instrument::from).collect(),
        }
    }

    /// GM1 tonal percussive instruments.
    pub fn tonal_percussive() -> Instruments {
        Instruments {
            instruments: (8..16).map(Instrument::from).collect(),
        }
    }

    /// GM1 organ instruments.
    pub fn organ() -> Instruments {
        Instruments {
            instruments: (16..24).map(Instrument::from).collect(),
        }
    }

    /// GM1 guitar instruments.
    pub fn guitar() -> Instruments {
        Instruments {
            instruments: (24..32).map(Instrument::from).collect(),
        }
    }

    /// GM1 bass instruments.
    pub fn bass() -> Instruments {
        Instruments {
            instruments: (32..40).map(Instrument::from).collect(),
        }
    }

    /// GM1 string instruments.
    pub fn strings() -> Instruments {
        Instruments {
            instruments: (40..48).map(Instrument::from).collect(),
        }
    }

    /// GM1 ensemble instruments.
    pub fn ensemble() -> Instruments {
        Instruments {
            instruments: (48..56).map(Instrument::from).collect(),
        }
    }

    /// GM1 brass instruments.
    pub fn brass() -> Instruments {
        Instruments {
            instruments: (56..64).map(Instrument::from).collect(),
        }
    }

    /// GM1 reed instruments.
    pub fn reed() -> Instruments {
        Instruments {
            instruments: (64..72).map(Instrument::from).collect(),
        }
    }

    /// GM1 pipe instruments.
    pub fn pipe() -> Instruments {
        Instruments {
            instruments: (72..80).map(Instrument::from).collect(),
        }
    }

    /// GM1 synth lead instruments.
    pub fn synth_lead() -> Instruments {
        Instruments {
            instruments: (80..88).map(Instrument::from).collect(),
        }
    }

    /// GM1 synth pad instruments.
    pub fn synth_pad() -> Instruments {
        Instruments {
            instruments: (88..96).map(Instrument::from).collect(),
        }
    }

    /// GM1 synth FX instruments.
    pub fn synth_fx() -> Instruments {
        Instruments {
            instruments: (96..104).map(Instrument::from).collect(),
        }
    }

    /// GM1 ethnic instruments.
    pub fn ethnic() -> Instruments {
        Instruments {
            instruments: (104..112).map(Instrument::from).collect(),
        }
    }

    /// GM1 percussive instruments.
    pub fn percussive() -> Instruments {
        Instruments {
            instruments: (112..120).map(Instrument::from).collect(),
        }
    }

    /// GM1 sound FX instruments.
    pub fn sound_fx() -> Instruments {
        Instruments {
            instruments: (120..128).map(Instrument::from).collect(),
        }
    }

    /// Returns "melodic" instruments which have a clear tone, and are not overly percussive.
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
        self.instruments.into_iter()
    }
}

/// ##Example
/// ```rust
/// # use redact_composer_midi::gm::elements::Instrument;
/// # use redact_composer_midi::gm::Instruments;
/// #
/// let pianos = Instruments { instruments: vec![Instrument::AcousticGrandPiano] }
///                         + Instruments { instruments: vec![Instrument::BrightAcousticPiano] };
/// assert_eq!(pianos, Instruments { instruments: vec![Instrument::AcousticGrandPiano, Instrument::BrightAcousticPiano] });
/// ```
impl Add for Instruments {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Instruments {
            instruments: self.into_iter().chain(rhs).collect(),
        }
    }
}

/// ##Example
/// ```rust
/// # use redact_composer_midi::gm::elements::Instrument;
/// # use redact_composer_midi::gm::Instruments;
/// #
/// let pianos = Instruments { instruments: vec![Instrument::AcousticGrandPiano] }
///                         + Instrument::BrightAcousticPiano;
/// assert_eq!(pianos, Instruments { instruments: vec![Instrument::AcousticGrandPiano, Instrument::BrightAcousticPiano] });
/// ```
impl Add<Instrument> for Instruments {
    type Output = Self;

    fn add(self, rhs: Instrument) -> Self::Output {
        Instruments {
            instruments: self.into_iter().chain(vec![rhs]).collect(),
        }
    }
}

/// ##Example
/// ```rust
/// # use redact_composer_midi::gm::elements::Instrument;
/// # use redact_composer_midi::gm::Instruments;
/// #
/// let no_violins = Instruments { instruments: vec![Instrument::Violin, Instrument::Cello] }
///                         - Instruments { instruments: vec![Instrument::Violin] };
/// assert_eq!(no_violins, Instruments { instruments: vec![Instrument::Cello] });
/// ```
impl Sub for Instruments {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Instruments {
            instruments: self
                .into_iter()
                .filter(|i| !rhs.instruments.contains(i))
                .collect(),
        }
    }
}

/// ##Example
/// ```rust
/// # use redact_composer_midi::gm::elements::Instrument;
/// # use redact_composer_midi::gm::Instruments;
/// #
/// let no_violins = Instruments { instruments: vec![Instrument::Violin, Instrument::Cello] }
///                         - Instrument::Violin;
/// assert_eq!(no_violins, Instruments { instruments: vec![Instrument::Cello] });
///
/// let instruments = Instruments { instruments: vec![Instrument::AcousticGrandPiano, Instrument::BrightAcousticPiano] };
/// ```
impl Sub<Instrument> for Instruments {
    type Output = Self;

    fn sub(self, rhs: Instrument) -> Self::Output {
        Instruments {
            instruments: self.into_iter().filter(|i| *i != rhs).collect(),
        }
    }
}

/// ##Example
/// ```rust
/// # use redact_composer_midi::gm::elements::Instrument;
/// # use redact_composer_midi::gm::Instruments;
/// #
/// let instruments = Instruments { instruments: vec![Instrument::AcousticGrandPiano, Instrument::BrightAcousticPiano] };
/// let vec_instruments: Vec<Instrument> = instruments.into();
/// assert_eq!(
///     vec_instruments,
///     vec![Instrument::AcousticGrandPiano, Instrument::BrightAcousticPiano]
/// );
/// ```
impl From<Instruments> for Vec<Instrument> {
    fn from(value: Instruments) -> Self {
        value.instruments
    }
}

/// Represents a drum hit. (Similar to [`PlayNote`]).
#[derive(Element, Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DrumHit {
    /// The drum hit sound type.
    pub hit: DrumHitType,
    /// The strength of attack.
    pub velocity: u8,
}

impl DrumHit {
    /// Renderer that renders a [`DrumHit`] as a [`PlayNote`] over the same timing.
    pub fn renderer() -> impl Renderer<Element = Self> {
        AdhocRenderer::<Self>::new(|segment, _| {
            Result::Ok(vec![Segment::new(
                PlayNote {
                    note: segment.element.hit.into(),
                    velocity: segment.element.velocity,
                },
                segment.timing,
            )])
        })
    }
}

/// Percussion key map defined according to
/// [GM1 Sound Set](https://www.midi.org/specifications-old/item/gm-level-1-sound-set)
#[derive(Debug, Hash, FromPrimitive, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[allow(missing_docs)]
pub enum DrumHitType {
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

impl From<u8> for DrumHitType {
    fn from(value: u8) -> Self {
        num::FromPrimitive::from_u8(value).unwrap()
    }
}

impl From<DrumHitType> for u8 {
    fn from(value: DrumHitType) -> Self {
        value as u8
    }
}
