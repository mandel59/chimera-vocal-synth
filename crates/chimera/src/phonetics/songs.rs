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
        // 1901; both in the public domain. The first verse to Taki's original melody, in D minor,
        // 4/4, ♩=88, one mora per note, a breath after each half line (7 and 5 morae).
        // Source of the notes: the LilyPond source of "Kojo no tsuki re mineur.svg" by Cdang,
        // Wikimedia Commons, 2026, CC0:
        // https://commons.wikimedia.org/wiki/File:Kojo_no_tsuki_re_mineur.svg
        // Its rest at each line's end (a2. r4) is held into the note before (a demo song has no
        // rests); its slurs are phrasing and are left out. こう and ろう are sung as long お, so they
        // are written こー and ろー (ー holds the vowel onto the next note).
        name: "荒城の月",
        bpm: 88.0,
        lyrics: "はるこーろーの、はなのえん。めぐるさかずき、かげさして。ちよのまつがえ、わけいでし。むかしのひかり、いまいずこ。",
        notes: &[
            [69.0, 1.0], [69.0, 1.0], [74.0, 1.0], [76.0, 1.0], [77.0, 1.0], [76.0, 1.0], [74.0, 2.0],
            [70.0, 1.0], [70.0, 1.0], [69.0, 1.0], [68.0, 1.0], [69.0, 4.0],
            [69.0, 1.0], [69.0, 1.0], [74.0, 1.0], [76.0, 1.0], [77.0, 1.0], [76.0, 1.0], [74.0, 2.0],
            [70.0, 1.0], [67.0, 1.0], [69.0, 1.5], [69.0, 0.5], [62.0, 4.0],
            [65.0, 1.5], [65.0, 0.5], [64.0, 1.0], [62.0, 1.0], [70.0, 1.0], [70.0, 1.0], [69.0, 2.0],
            [67.0, 1.0], [69.0, 1.0], [70.0, 1.5], [70.0, 0.5], [69.0, 4.0],
            [69.0, 1.0], [69.0, 1.0], [74.0, 1.0], [76.0, 1.0], [77.0, 1.0], [76.0, 1.0], [74.0, 2.0],
            [70.0, 1.0], [67.0, 1.0], [69.0, 1.5], [69.0, 0.5], [62.0, 4.0],
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
    DemoSong {
        // Traditional warabeuta; words and melody are in the public domain. Our transcription
        // of the unaccompanied melody, checked against Recreation Association of Japan,
        // Recrew 689 (2020), p. 37 (no arrangement, chords or score image reproduced):
        // https://mem.recreation.or.jp/wpmem/wp-content/uploads/2021/09/recrew_689_2020_02-2.pdf
        // Dotted-eighth / sixteenth rhythm. As in 荒城の月, rests extend the preceding note;
        // punctuation supplies a breath. ぽう, やい and ちょい subdivide the same pitch.
        // First あん and せん share a note (𛅧); the second せ・ん has separate notes.
        name: "あんたがたどこさ",
        bpm: 100.0,
        lyrics: "あ𛅧たがたどこさ、ひごさ、ひごどこさ、くまもとさ、くまもとどこさ、せ𛅧ばさ、せんばやまにわ、たぬきがおってさ、それをりょーしが、てっぽーでうってさ、にてさ、やいてさ、くってさ、それをこのはで、ちょいとかぶせ。",
        notes: &[
            [62.0, 0.75], [62.0, 0.25], [62.0, 0.75], [65.0, 0.25],
            [67.0, 0.75], [65.0, 0.25], [67.0, 1.0],
            [69.0, 0.75], [65.0, 0.25], [67.0, 1.0],
            [69.0, 0.75], [69.0, 0.25], [67.0, 0.75], [65.0, 0.25], [62.0, 1.0],
            [62.0, 0.75], [65.0, 0.25], [65.0, 0.75], [65.0, 0.25], [67.0, 1.0],
            [65.0, 0.75], [67.0, 0.25], [69.0, 0.75], [69.0, 0.25], [67.0, 0.75], [65.0, 0.25], [62.0, 1.0],
            [67.0, 0.75], [65.0, 0.25], [62.0, 1.75],
            [62.0, 0.25], [69.0, 0.75], [69.0, 0.25],
            [69.0, 0.75], [69.0, 0.25], [65.0, 0.75], [65.0, 0.25],
            [67.0, 0.75], [67.0, 0.25], [69.0, 0.75], [69.0, 0.25],
            [67.0, 0.75], [65.0, 0.25], [62.0, 1.75],
            [62.0, 0.25], [69.0, 0.75], [69.0, 0.25],
            [69.0, 0.75], [69.0, 0.25], [65.0, 0.75], [65.0, 0.25],
            [67.0, 1.0], [69.0, 0.5], [69.0, 0.25], [69.0, 0.25],
            [67.0, 0.75], [65.0, 0.25], [62.0, 1.0],
            [62.0, 0.75], [65.0, 0.25], [67.0, 1.0],
            [62.0, 0.5], [62.0, 0.25], [65.0, 0.25], [67.0, 1.0],
            [69.0, 0.75], [65.0, 0.25], [67.0, 1.75],
            [62.0, 0.25], [69.0, 0.75], [69.0, 0.25],
            [69.0, 0.75], [69.0, 0.25], [65.0, 0.75], [65.0, 0.25],
            [67.0, 0.5], [67.0, 0.25], [65.0, 0.25], [62.0, 1.0], [65.0, 1.0],
            [67.0, 2.0],
        ],
        octave: SongOctave { male: -1.0, female: 0.0, child: 0.0 },
    },
];
