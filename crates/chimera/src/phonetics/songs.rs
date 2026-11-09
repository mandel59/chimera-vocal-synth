//! Demo melodies for Sing mode: [MIDI note, beats] per syllable, in the adult female register.

use serde::Serialize;

/// octave shift per body
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SongOctave {
    pub male: f64,
    pub female: f64,
    pub child: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DemoSong {
    pub name: &'static str,
    pub bpm: f64,
    pub lyrics: &'static str,
    pub notes: &'static [[f64; 2]],
    /// octave shift per body
    pub octave: SongOctave,
}

#[rustfmt::skip]
pub static DEMO_SONGS: &[DemoSong] = &[
    DemoSong {
        // an original tune for this instrument
        name: "Made of air",
        bpm: 84.0,
        lyrics: "Breathe in, breathe out, I am made of air. Tongue and lips and throat, singing anywhere.",
        notes: &[
            [64.0, 1.0], [60.0, 1.0], [62.0, 1.0], [57.0, 1.0],
            [57.0, 0.5], [60.0, 0.5], [64.0, 1.0], [62.0, 0.5], [60.0, 1.5],
            [65.0, 1.0], [64.0, 0.5], [62.0, 0.5], [60.0, 0.5], [62.0, 1.5],
            [64.0, 0.5], [62.0, 0.5], [60.0, 0.5], [59.0, 0.5], [57.0, 2.0],
        ],
        octave: SongOctave { male: -1.0, female: 0.0, child: 0.0 },
    },
    DemoSong {
        // traditional (public domain)
        name: "Twinkle, twinkle",
        bpm: 100.0,
        lyrics: "Twinkle twinkle little star, how I wonder what you are.",
        notes: &[
            [60.0, 1.0], [60.0, 1.0], [67.0, 1.0], [67.0, 1.0], [69.0, 1.0], [69.0, 1.0], [67.0, 2.0],
            [65.0, 1.0], [65.0, 1.0], [64.0, 1.0], [64.0, 1.0], [62.0, 1.0], [62.0, 1.0], [60.0, 2.0],
        ],
        octave: SongOctave { male: -1.0, female: 0.0, child: 0.0 },
    },
    DemoSong {
        // 荒城の月: music by Rentarō Taki (1879–1903), words by Bansui Doi (1871–1952), published
        // 1901; both in the public domain. The first verse, to the melody as it is usually sung
        // today, in A minor, 4/4. A breath after each half line (7 and 5 morae). こう and ろう are
        // sung as long お, so they are written こー and ろー (ー holds the vowel onto the next note).
        name: "荒城の月",
        bpm: 66.0,
        lyrics: "はるこーろーの、はなのえん。めぐるさかずき、かげさして。ちよのまつがえ、わけいでし。むかしのひかり、いまいずこ。",
        notes: &[
            [64.0, 1.0], [64.0, 1.0], [69.0, 1.0], [71.0, 1.0], [72.0, 1.5], [71.0, 0.5], [69.0, 2.0],
            [65.0, 1.0], [65.0, 1.0], [64.0, 1.0], [62.0, 1.0], [64.0, 4.0],
            [64.0, 1.0], [64.0, 1.0], [69.0, 1.0], [71.0, 1.0], [72.0, 1.5], [71.0, 0.5], [69.0, 2.0],
            [65.0, 1.0], [62.0, 1.0], [64.0, 1.5], [64.0, 0.5], [69.0, 4.0],
            [72.0, 1.0], [72.0, 1.0], [71.0, 1.0], [69.0, 1.0], [65.0, 1.0], [65.0, 1.0], [64.0, 2.0],
            [65.0, 1.0], [64.0, 1.0], [62.0, 1.0], [60.0, 1.0], [59.0, 4.0],
            [64.0, 1.0], [64.0, 1.0], [69.0, 1.0], [71.0, 1.0], [72.0, 1.5], [71.0, 0.5], [69.0, 2.0],
            [65.0, 1.0], [62.0, 1.0], [64.0, 1.5], [64.0, 0.5], [69.0, 4.0],
        ],
        octave: SongOctave { male: -1.0, female: 0.0, child: 0.0 },
    },
    DemoSong {
        // さくらさくら: a traditional song (古謡, published 1888), words and tune in the public
        // domain. The particle は is written as sung (わ); ー holds a vowel onto the next note.
        name: "さくらさくら",
        bpm: 76.0,
        lyrics: "さくら、さくら、やよいのそらーわ、みわたすかぎーり、かすみかくもーか、においぞいずーる、いざや、いざや、みにゆーかん。",
        notes: &[
            [69.0, 1.0], [69.0, 1.0], [71.0, 2.0],
            [69.0, 1.0], [69.0, 1.0], [71.0, 2.0],
            [69.0, 1.0], [71.0, 1.0], [72.0, 1.0], [71.0, 1.0], [69.0, 1.0], [71.0, 0.5], [69.0, 0.5], [65.0, 2.0],
            [64.0, 1.0], [60.0, 1.0], [64.0, 1.0], [65.0, 1.0], [64.0, 1.0], [64.0, 0.5], [60.0, 0.5], [59.0, 2.0],
            [69.0, 1.0], [71.0, 1.0], [72.0, 1.0], [71.0, 1.0], [69.0, 1.0], [71.0, 0.5], [69.0, 0.5], [65.0, 2.0],
            [64.0, 1.0], [60.0, 1.0], [64.0, 1.0], [65.0, 1.0], [64.0, 1.0], [64.0, 0.5], [60.0, 0.5], [59.0, 2.0],
            [69.0, 1.0], [69.0, 1.0], [71.0, 2.0],
            [69.0, 1.0], [69.0, 1.0], [71.0, 2.0],
            [64.0, 1.0], [65.0, 1.0], [71.0, 0.5], [69.0, 0.5], [65.0, 1.0], [64.0, 3.0],
        ],
        octave: SongOctave { male: -1.0, female: 0.0, child: 0.0 },
    },
];
