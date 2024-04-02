use crate::{Key, Note, PitchClass, PitchClassCollection};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "redact-composer")]
use redact_composer_core::derive::Element;

/// Musical note name.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "redact-composer", derive(Element))]
#[allow(missing_docs)]
pub enum NoteName {
    Abb,
    Ab,
    A,
    As,
    Ass,
    Bbb,
    Bb,
    B,
    Bs,
    Bss,
    Cbb,
    Cb,
    C,
    Cs,
    Css,
    Dbb,
    Db,
    D,
    Ds,
    Dss,
    Ebb,
    Eb,
    E,
    Es,
    Ess,
    Fbb,
    Fb,
    F,
    Fs,
    Fss,
    Gbb,
    Gb,
    G,
    Gs,
    Gss,
}

impl NoteName {
    /// Returns this note name as a [`Note`] in a given octave.
    /// ```
    /// use redact_composer_musical::{Note, NoteName::C};
    /// assert_eq!(C.in_octave(4), Note(60));
    /// ```
    pub fn in_octave(&self, octave: i8) -> Note {
        (*self, octave).into()
    }
}

impl From<NoteName> for u8 {
    fn from(note: NoteName) -> Self {
        use NoteName::*;
        match note {
            C | Bs | Dbb => 0,
            Cs | Db | Bss => 1,
            D | Css | Ebb => 2,
            Ds | Eb | Fbb => 3,
            E | Fb | Dss => 4,
            F | Es | Gbb => 5,
            Fs | Gb | Ess => 6,
            G | Fss | Abb => 7,
            Gs | Ab => 8,
            A | Gss | Bbb => 9,
            As | Bb | Cbb => 10,
            B | Cb | Ass => 11,
        }
    }
}

impl PartialEq<PitchClass> for NoteName {
    /// ```
    /// use redact_composer_musical::{NoteName, PitchClass};
    /// assert!(NoteName::C.eq(&PitchClass(0)));
    /// ```
    fn eq(&self, pitch_class: &PitchClass) -> bool {
        PitchClass::from(*self) == *pitch_class
    }
}

#[allow(dead_code)]
impl NoteName {
    /// Strips any accidental (for example, [`NoteName::Ab`] will return [`NoteName::A`]).
    pub(crate) fn letter(&self) -> NoteName {
        use NoteName::*;
        match self {
            Abb | Ab | A | As | Ass => A,
            Bbb | Bb | B | Bs | Bss => B,
            Cbb | Cb | C | Cs | Css => C,
            Dbb | Db | D | Ds | Dss => D,
            Ebb | Eb | E | Es | Ess => E,
            Fbb | Fb | F | Fs | Fss => F,
            Gbb | Gb | G | Gs | Gss => G,
        }
    }

    pub(crate) fn next_letter(&self) -> NoteName {
        use NoteName::*;
        match self.letter() {
            A => B,
            B => C,
            C => D,
            D => E,
            E => F,
            F => G,
            G => A,
            _ => unreachable!(),
        }
    }

    pub(crate) fn prev_letter(&self) -> NoteName {
        use NoteName::*;
        match self.letter() {
            A => G,
            B => A,
            C => B,
            D => C,
            E => D,
            F => E,
            G => F,
            _ => unreachable!(),
        }
    }

    pub(crate) fn has_sharp(&self) -> bool {
        use NoteName::*;
        matches!(
            self,
            As | Bs | Cs | Ds | Es | Fs | Gs | Ass | Bss | Css | Dss | Ess | Fss | Gss
        )
    }

    pub(crate) fn has_flat(&self) -> bool {
        use NoteName::*;
        matches!(
            self,
            Ab | Bb | Cb | Db | Eb | Fb | Gb | Abb | Bbb | Cbb | Dbb | Ebb | Fbb | Gbb
        )
    }

    pub(crate) fn has_double_sharp(&self) -> bool {
        use NoteName::*;
        matches!(self, Ass | Bss | Css | Dss | Ess | Fss | Gss)
    }

    pub(crate) fn has_double_flat(&self) -> bool {
        use NoteName::*;
        matches!(self, Abb | Bbb | Cbb | Dbb | Ebb | Fbb | Gbb)
    }

    pub(crate) fn complexity(&self) -> u8 {
        use NoteName::*;
        match self {
            A | B | C | D | E | F | G => 0,
            As | Bs | Cs | Ds | Es | Fs | Gs => 1,
            Ab | Bb | Cb | Db | Eb | Fb | Gb => 1,
            Ass | Bss | Css | Dss | Ess | Fss | Gss => 2,
            Abb | Bbb | Cbb | Dbb | Ebb | Fbb | Gbb => 2,
        }
    }
}

impl PitchClass {
    /// Provides the note names matching this pitch class.
    pub fn names(&self) -> Vec<NoteName> {
        use NoteName::*;
        match self.0 {
            0 => vec![C, Bs, Dbb],
            1 => vec![Cs, Db, Bss],
            2 => vec![D, Css, Ebb],
            3 => vec![Ds, Eb, Fbb],
            4 => vec![E, Fb, Dss],
            5 => vec![F, Es, Gbb],
            6 => vec![Fs, Gb, Ess],
            7 => vec![G, Fss, Abb],
            8 => vec![Gs, Ab],
            9 => vec![A, Gss, Bbb],
            10 => vec![As, Bb, Cbb],
            11 => vec![B, Cb, Ass],
            _ => panic!(),
        }
    }

    /// Returns this pitch class's name within the context of a [`Key`]. Pitch classes not in the
    /// given key will return some variation of a name equating to the pitch class, but exactly
    /// which is subject to change.
    pub fn name_in_key(&self, key: &Key) -> NoteName {
        let pitch_names = self.names();
        let key_note_names = key.note_names();

        let in_key_note = key_note_names
            .iter()
            .find(|n| &PitchClass::from(**n) == self);

        if let Some(note) = in_key_note {
            return *note;
        }

        let letter_matching_key_note = key_note_names
            .iter()
            .find(|n| &PitchClass::from(n.letter()) == self);

        if let Some(key_note) = letter_matching_key_note {
            return *key_note;
        }

        let naturalized_note = pitch_names.iter().find(|n| !n.has_sharp() && !n.has_flat());

        if let Some(note) = naturalized_note {
            return *note;
        }

        let key_sharps = key_note_names.iter().filter(|n| n.has_sharp()).count();
        let key_flats = key_note_names.iter().filter(|n| n.has_flat()).count();

        if key_sharps >= key_flats {
            if let Some(note) = pitch_names.iter().find(|n| n.has_sharp()) {
                *note
            } else if let Some(note) = pitch_names.iter().find(|n| n.has_flat()) {
                *note
            } else {
                pitch_names[0]
            }
        } else if let Some(note) = pitch_names.iter().find(|n| n.has_flat()) {
            *note
        } else if let Some(note) = pitch_names.iter().find(|n| n.has_sharp()) {
            *note
        } else {
            pitch_names[0]
        }
    }
}

impl Key {
    /// Returns the [`NoteName`] of the key's tonic pitch class.
    pub fn root_name(&self) -> NoteName {
        match self.name_pref {
            Some(n) => Self::simplify_root(n),
            None => self
                .root
                .names()
                .into_iter()
                .min_by_key(NoteName::complexity)
                .expect("Key should be nameable for any NoteName"),
        }
    }

    pub(crate) fn simplify_root(name: NoteName) -> NoteName {
        if name.complexity() > 1 {
            PitchClass::from(name)
                .names()
                .into_iter()
                .min_by_key(NoteName::complexity)
                .unwrap()
        } else {
            name
        }
    }

    /// Returns the ordered list of [`NoteName`]'s associated with the key's pitch classes.
    /// ```
    /// use redact_composer_musical::{Key, NoteName::*, Scale::Major};
    /// let key = Key::from((D, Major));
    /// assert_eq!(key.note_names(), [D, E, Fs, G, A, B, Cs])
    /// ```
    pub fn note_names(&self) -> Vec<NoteName> {
        let first = self.root_name();

        let mut next_letter = first.next_letter();
        let mut sharp_pref: Option<bool> = if first.has_sharp() || first.has_flat() {
            Some(first.has_sharp())
        } else {
            None
        };

        let mut names = vec![first];
        for pitch in self.pitch_classes().into_iter().skip(1) {
            // First try finding a name that matches the sharp/flat preference
            let name_options = pitch.names().into_iter().collect::<Vec<_>>();
            let maybe_name = name_options
                .iter()
                .filter(|n| n.letter() == next_letter)
                .filter(|n| {
                    if let Some(pref_sharp) = sharp_pref {
                        if pref_sharp {
                            n.has_sharp()
                        } else {
                            n.has_flat()
                        }
                    } else {
                        true
                    }
                })
                .min_by_key(|n| n.complexity())
                .copied();

            let name = if let Some(name) = maybe_name {
                name
            } else {
                // If no match found for the sharp/flat preference, remove the restriction
                name_options
                    .iter()
                    .filter(|n| n.letter() == next_letter)
                    .min_by_key(|n| n.complexity())
                    .copied()
                    .expect("Bug: Every Key note name should have a following note name.")
            };

            names.push(name);

            if sharp_pref.is_none() && (name.has_sharp() || name.has_flat()) {
                sharp_pref = Some(name.has_sharp());
            }

            next_letter = name.next_letter();
        }

        names
    }
}

#[cfg(test)]
mod test {
    mod scale_note_names {
        use crate::{Key, NoteName::*, Scale::*};

        #[test]
        fn ab_scales() {
            assert_eq!(
                Key::from((Ab, Major)).note_names(),
                [Ab, Bb, C, Db, Eb, F, G]
            );
            assert_eq!(
                Key::from((Ab, NaturalMinor)).note_names(),
                [Ab, Bb, Cb, Db, Eb, Fb, Gb]
            );
            assert_eq!(
                Key::from((Ab, MelodicMinor)).note_names(),
                [Ab, Bb, Cb, Db, Eb, F, G]
            );
            assert_eq!(
                Key::from((Ab, HarmonicMinor)).note_names(),
                [Ab, Bb, Cb, Db, Eb, Fb, G]
            );
        }

        #[test]
        fn a_scales() {
            assert_eq!(Key::from((A, Major)).note_names(), [A, B, Cs, D, E, Fs, Gs]);
            assert_eq!(
                Key::from((A, NaturalMinor)).note_names(),
                [A, B, C, D, E, F, G]
            );
            assert_eq!(
                Key::from((A, MelodicMinor)).note_names(),
                [A, B, C, D, E, Fs, Gs]
            );
            assert_eq!(
                Key::from((A, HarmonicMinor)).note_names(),
                [A, B, C, D, E, F, Gs]
            );
        }

        #[test]
        fn as_scales() {
            assert_eq!(
                Key::from((As, Major)).note_names(),
                [As, Bs, Css, Ds, Es, Fss, Gss]
            );
            assert_eq!(
                Key::from((As, NaturalMinor)).note_names(),
                [As, Bs, Cs, Ds, Es, Fs, Gs]
            );
            assert_eq!(
                Key::from((As, MelodicMinor)).note_names(),
                [As, Bs, Cs, Ds, Es, Fss, Gss]
            );
            assert_eq!(
                Key::from((As, HarmonicMinor)).note_names(),
                [As, Bs, Cs, Ds, Es, Fs, Gss]
            );
        }

        #[test]
        fn bb_scales() {
            assert_eq!(Key::from((Bb, Major)).note_names(), [Bb, C, D, Eb, F, G, A]);
            assert_eq!(
                Key::from((Bb, NaturalMinor)).note_names(),
                [Bb, C, Db, Eb, F, Gb, Ab]
            );
            assert_eq!(
                Key::from((Bb, MelodicMinor)).note_names(),
                [Bb, C, Db, Eb, F, G, A]
            );
            assert_eq!(
                Key::from((Bb, HarmonicMinor)).note_names(),
                [Bb, C, Db, Eb, F, Gb, A]
            );
        }

        #[test]
        fn b_scales() {
            assert_eq!(
                Key::from((B, Major)).note_names(),
                [B, Cs, Ds, E, Fs, Gs, As]
            );
            assert_eq!(
                Key::from((B, NaturalMinor)).note_names(),
                [B, Cs, D, E, Fs, G, A]
            );
            assert_eq!(
                Key::from((B, MelodicMinor)).note_names(),
                [B, Cs, D, E, Fs, Gs, As]
            );
            assert_eq!(
                Key::from((B, HarmonicMinor)).note_names(),
                [B, Cs, D, E, Fs, G, As]
            );
        }

        #[test]
        fn bs_scales() {
            assert_eq!(
                Key::from((Bs, Major)).note_names(),
                [Bs, Css, Dss, Es, Fss, Gss, Ass]
            );
            assert_eq!(
                Key::from((Bs, NaturalMinor)).note_names(),
                [Bs, Css, Ds, Es, Fss, Gs, As]
            );
            assert_eq!(
                Key::from((Bs, MelodicMinor)).note_names(),
                [Bs, Css, Ds, Es, Fss, Gss, Ass]
            );
            assert_eq!(
                Key::from((Bs, HarmonicMinor)).note_names(),
                [Bs, Css, Ds, Es, Fss, Gs, Ass]
            );
        }

        #[test]
        fn cb_scales() {
            assert_eq!(
                Key::from((Cb, Major)).note_names(),
                [Cb, Db, Eb, Fb, Gb, Ab, Bb]
            );
            assert_eq!(
                Key::from((Cb, NaturalMinor)).note_names(),
                [Cb, Db, Ebb, Fb, Gb, Abb, Bbb]
            );
            assert_eq!(
                Key::from((Cb, MelodicMinor)).note_names(),
                [Cb, Db, Ebb, Fb, Gb, Ab, Bb]
            );
            assert_eq!(
                Key::from((Cb, HarmonicMinor)).note_names(),
                [Cb, Db, Ebb, Fb, Gb, Abb, Bb]
            );
        }

        #[test]
        fn c_scales() {
            assert_eq!(Key::from((C, Major)).note_names(), [C, D, E, F, G, A, B]);
            assert_eq!(
                Key::from((C, NaturalMinor)).note_names(),
                [C, D, Eb, F, G, Ab, Bb]
            );
            assert_eq!(
                Key::from((C, MelodicMinor)).note_names(),
                [C, D, Eb, F, G, A, B]
            );
            assert_eq!(
                Key::from((C, HarmonicMinor)).note_names(),
                [C, D, Eb, F, G, Ab, B]
            );
        }

        #[test]
        fn cs_scales() {
            assert_eq!(
                Key::from((Cs, Major)).note_names(),
                [Cs, Ds, Es, Fs, Gs, As, Bs]
            );
            assert_eq!(
                Key::from((Cs, NaturalMinor)).note_names(),
                [Cs, Ds, E, Fs, Gs, A, B]
            );
            assert_eq!(
                Key::from((Cs, MelodicMinor)).note_names(),
                [Cs, Ds, E, Fs, Gs, As, Bs]
            );
            assert_eq!(
                Key::from((Cs, HarmonicMinor)).note_names(),
                [Cs, Ds, E, Fs, Gs, A, Bs]
            );
        }

        #[test]
        fn db_scales() {
            assert_eq!(
                Key::from((Db, Major)).note_names(),
                [Db, Eb, F, Gb, Ab, Bb, C]
            );
            assert_eq!(
                Key::from((Db, NaturalMinor)).note_names(),
                [Db, Eb, Fb, Gb, Ab, Bbb, Cb]
            );
            assert_eq!(
                Key::from((Db, MelodicMinor)).note_names(),
                [Db, Eb, Fb, Gb, Ab, Bb, C]
            );
            assert_eq!(
                Key::from((Db, HarmonicMinor)).note_names(),
                [Db, Eb, Fb, Gb, Ab, Bbb, C]
            );
        }

        #[test]
        fn d_scales() {
            assert_eq!(Key::from((D, Major)).note_names(), [D, E, Fs, G, A, B, Cs]);
            assert_eq!(
                Key::from((D, NaturalMinor)).note_names(),
                [D, E, F, G, A, Bb, C]
            );
            assert_eq!(
                Key::from((D, MelodicMinor)).note_names(),
                [D, E, F, G, A, B, Cs]
            );
            assert_eq!(
                Key::from((D, HarmonicMinor)).note_names(),
                [D, E, F, G, A, Bb, Cs]
            );
        }

        #[test]
        fn ds_scales() {
            assert_eq!(
                Key::from((Ds, Major)).note_names(),
                [Ds, Es, Fss, Gs, As, Bs, Css]
            );
            assert_eq!(
                Key::from((Ds, NaturalMinor)).note_names(),
                [Ds, Es, Fs, Gs, As, B, Cs]
            );
            assert_eq!(
                Key::from((Ds, MelodicMinor)).note_names(),
                [Ds, Es, Fs, Gs, As, Bs, Css]
            );
            assert_eq!(
                Key::from((Ds, HarmonicMinor)).note_names(),
                [Ds, Es, Fs, Gs, As, B, Css]
            );
        }

        #[test]
        fn eb_scales() {
            assert_eq!(
                Key::from((Eb, Major)).note_names(),
                [Eb, F, G, Ab, Bb, C, D]
            );
            assert_eq!(
                Key::from((Eb, NaturalMinor)).note_names(),
                [Eb, F, Gb, Ab, Bb, Cb, Db]
            );
            assert_eq!(
                Key::from((Eb, MelodicMinor)).note_names(),
                [Eb, F, Gb, Ab, Bb, C, D]
            );
            assert_eq!(
                Key::from((Eb, HarmonicMinor)).note_names(),
                [Eb, F, Gb, Ab, Bb, Cb, D]
            );
        }

        #[test]
        fn e_scales() {
            assert_eq!(
                Key::from((E, Major)).note_names(),
                [E, Fs, Gs, A, B, Cs, Ds]
            );
            assert_eq!(
                Key::from((E, NaturalMinor)).note_names(),
                [E, Fs, G, A, B, C, D]
            );
            assert_eq!(
                Key::from((E, MelodicMinor)).note_names(),
                [E, Fs, G, A, B, Cs, Ds]
            );
            assert_eq!(
                Key::from((E, HarmonicMinor)).note_names(),
                [E, Fs, G, A, B, C, Ds]
            );
        }

        #[test]
        fn es_scales() {
            assert_eq!(
                Key::from((Es, Major)).note_names(),
                [Es, Fss, Gss, As, Bs, Css, Dss]
            );
            assert_eq!(
                Key::from((Es, NaturalMinor)).note_names(),
                [Es, Fss, Gs, As, Bs, Cs, Ds]
            );
            assert_eq!(
                Key::from((Es, MelodicMinor)).note_names(),
                [Es, Fss, Gs, As, Bs, Css, Dss]
            );
            assert_eq!(
                Key::from((Es, HarmonicMinor)).note_names(),
                [Es, Fss, Gs, As, Bs, Cs, Dss]
            );
        }

        #[test]
        fn fb_scales() {
            assert_eq!(
                Key::from((Fb, Major)).note_names(),
                [Fb, Gb, Ab, Bbb, Cb, Db, Eb]
            );
            assert_eq!(
                Key::from((Fb, NaturalMinor)).note_names(),
                [Fb, Gb, Abb, Bbb, Cb, Dbb, Ebb]
            );
            assert_eq!(
                Key::from((Fb, MelodicMinor)).note_names(),
                [Fb, Gb, Abb, Bbb, Cb, Db, Eb]
            );
            assert_eq!(
                Key::from((Fb, HarmonicMinor)).note_names(),
                [Fb, Gb, Abb, Bbb, Cb, Dbb, Eb]
            );
        }

        #[test]
        fn f_scales() {
            assert_eq!(Key::from((F, Major)).note_names(), [F, G, A, Bb, C, D, E]);
            assert_eq!(
                Key::from((F, NaturalMinor)).note_names(),
                [F, G, Ab, Bb, C, Db, Eb]
            );
            assert_eq!(
                Key::from((F, MelodicMinor)).note_names(),
                [F, G, Ab, Bb, C, D, E]
            );
            assert_eq!(
                Key::from((F, HarmonicMinor)).note_names(),
                [F, G, Ab, Bb, C, Db, E]
            );
        }

        #[test]
        fn fs_scales() {
            assert_eq!(
                Key::from((Fs, Major)).note_names(),
                [Fs, Gs, As, B, Cs, Ds, Es]
            );
            assert_eq!(
                Key::from((Fs, NaturalMinor)).note_names(),
                [Fs, Gs, A, B, Cs, D, E]
            );
            assert_eq!(
                Key::from((Fs, MelodicMinor)).note_names(),
                [Fs, Gs, A, B, Cs, Ds, Es]
            );
            assert_eq!(
                Key::from((Fs, HarmonicMinor)).note_names(),
                [Fs, Gs, A, B, Cs, D, Es]
            );
        }

        #[test]
        fn gb_scales() {
            assert_eq!(
                Key::from((Gb, Major)).note_names(),
                [Gb, Ab, Bb, Cb, Db, Eb, F]
            );
            assert_eq!(
                Key::from((Gb, NaturalMinor)).note_names(),
                [Gb, Ab, Bbb, Cb, Db, Ebb, Fb]
            );
            assert_eq!(
                Key::from((Gb, MelodicMinor)).note_names(),
                [Gb, Ab, Bbb, Cb, Db, Eb, F]
            );
            assert_eq!(
                Key::from((Gb, HarmonicMinor)).note_names(),
                [Gb, Ab, Bbb, Cb, Db, Ebb, F]
            );
        }

        #[test]
        fn g_scales() {
            assert_eq!(Key::from((G, Major)).note_names(), [G, A, B, C, D, E, Fs]);
            assert_eq!(
                Key::from((G, NaturalMinor)).note_names(),
                [G, A, Bb, C, D, Eb, F]
            );
            assert_eq!(
                Key::from((G, MelodicMinor)).note_names(),
                [G, A, Bb, C, D, E, Fs]
            );
            assert_eq!(
                Key::from((G, HarmonicMinor)).note_names(),
                [G, A, Bb, C, D, Eb, Fs]
            );
        }

        #[test]
        fn gs_scales() {
            assert_eq!(
                Key::from((Gs, Major)).note_names(),
                [Gs, As, Bs, Cs, Ds, Es, Fss]
            );
            assert_eq!(
                Key::from((Gs, NaturalMinor)).note_names(),
                [Gs, As, B, Cs, Ds, E, Fs]
            );
            assert_eq!(
                Key::from((Gs, MelodicMinor)).note_names(),
                [Gs, As, B, Cs, Ds, Es, Fss]
            );
            assert_eq!(
                Key::from((Gs, HarmonicMinor)).note_names(),
                [Gs, As, B, Cs, Ds, E, Fss]
            );
        }
    }
}
