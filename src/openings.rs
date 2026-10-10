/// Opening book and classification using longest-prefix match.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpeningEntry {
    pub moves: &'static [&'static str],
    pub eco: &'static str,
    pub name: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct OpeningMatch {
    pub eco: String,
    pub name: String,
}

pub static OPENING_BOOK: &[OpeningEntry] = &[
    // ---- 1.e4 e5 lines ----
    OpeningEntry { moves: &["e4", "e5"], eco: "C20", name: "King's Pawn Game" },
    OpeningEntry { moves: &["e4", "e5", "Nf3"], eco: "C40", name: "King's Knight Opening" },
    OpeningEntry { moves: &["e4", "e5", "Nf3", "Nc6"], eco: "C44", name: "Normal Variation" },
    OpeningEntry { moves: &["e4", "e5", "Nf3", "Nc6", "Bc4"], eco: "C50", name: "Italian Game" },
    OpeningEntry { moves: &["e4", "e5", "Nf3", "Nc6", "Bc4", "Bc5"], eco: "C50", name: "Giuoco Piano" },
    OpeningEntry { moves: &["e4", "e5", "Nf3", "Nc6", "Bc4", "Nf6"], eco: "C55", name: "Two Knights Defense" },
    OpeningEntry { moves: &["e4", "e5", "Nf3", "Nc6", "Bb5"], eco: "C60", name: "Ruy Lopez" },
    OpeningEntry { moves: &["e4", "e5", "Nf3", "Nc6", "d4"], eco: "C44", name: "Scotch Game" },
    OpeningEntry { moves: &["e4", "e5", "Nf3", "Nc6", "Nc3"], eco: "C46", name: "Three Knights Opening" },
    OpeningEntry { moves: &["e4", "e5", "Nf3", "Nc6", "Nc3", "Nf6"], eco: "C47", name: "Four Knights Game" },
    OpeningEntry { moves: &["e4", "e5", "Nf3", "Nf6"], eco: "C42", name: "Petrov's Defense" },
    OpeningEntry { moves: &["e4", "e5", "Nf3", "d6"], eco: "C41", name: "Philidor Defense" },
    OpeningEntry { moves: &["e4", "e5", "Nf3", "Qf6"], eco: "C40", name: "McConnell Defense" },
    OpeningEntry { moves: &["e4", "e5", "Nf3", "Qe7"], eco: "C40", name: "Gunderam Defense" },
    OpeningEntry { moves: &["e4", "e5", "Nf3", "f6"], eco: "C40", name: "Damiano Defense" },
    OpeningEntry { moves: &["e4", "e5", "Nf3", "f5"], eco: "C40", name: "Latvian Gambit" },
    OpeningEntry { moves: &["e4", "e5", "Nc3"], eco: "C25", name: "Vienna Game" },
    OpeningEntry { moves: &["e4", "e5", "Bc4"], eco: "C23", name: "Bishop's Opening" },
    OpeningEntry { moves: &["e4", "e5", "Qh5"], eco: "C20", name: "Wayward Queen Attack" },
    OpeningEntry { moves: &["e4", "e5", "f4"], eco: "C30", name: "King's Gambit" },
    OpeningEntry { moves: &["e4", "e5", "d4"], eco: "C21", name: "Center Game" },
    OpeningEntry { moves: &["e4", "e5", "d4", "exd4"], eco: "C21", name: "Center Game Accepted" },
    OpeningEntry { moves: &["e4", "e5", "c3"], eco: "C20", name: "Leonardis Variation" },
    OpeningEntry { moves: &["e4", "e5", "Ne2"], eco: "C20", name: "Napoleon Attack" },
    OpeningEntry { moves: &["e4", "e5", "Ke2"], eco: "C20", name: "Bongcloud Attack" },
    OpeningEntry { moves: &["e4", "Nc6"], eco: "B00", name: "Nimzowitsch Defense" },
    OpeningEntry { moves: &["e4", "e5", "Nf3", "d5"], eco: "C40", name: "Elephant Gambit" },

    // ---- other 1.e4 lines ----
    OpeningEntry { moves: &["e4", "c5"], eco: "B20", name: "Sicilian Defense" },
    OpeningEntry { moves: &["e4", "c5", "Nc3"], eco: "B23", name: "Closed Sicilian" },
    OpeningEntry { moves: &["e4", "c5", "Nf3"], eco: "B27", name: "Sicilian Defense: Open" },
    OpeningEntry { moves: &["e4", "c5", "Nf3", "d6", "d4", "cxd4", "Nxd4", "Nf6", "Nc3", "a6"], eco: "B90", name: "Sicilian: Najdorf" },
    OpeningEntry { moves: &["e4", "c5", "Nf3", "Nc6"], eco: "B30", name: "Sicilian: Old Sicilian" },
    OpeningEntry { moves: &["e4", "c5", "c3"], eco: "B22", name: "Alapin Sicilian" },
    OpeningEntry { moves: &["e4", "c6"], eco: "B10", name: "Caro-Kann Defense" },
    OpeningEntry { moves: &["e4", "c6", "d4", "d5"], eco: "B12", name: "Caro-Kann: Classical" },
    OpeningEntry { moves: &["e4", "e6"], eco: "C00", name: "French Defense" },
    OpeningEntry { moves: &["e4", "e6", "d4", "d5"], eco: "C01", name: "French: Main Line" },
    OpeningEntry { moves: &["e4", "d5"], eco: "B01", name: "Scandinavian Defense" },
    OpeningEntry { moves: &["e4", "d5", "exd5", "Qxd5"], eco: "B01", name: "Scandinavian: Mieses-Kotroc" },
    OpeningEntry { moves: &["e4", "Nf6"], eco: "B02", name: "Alekhine's Defense" },
    OpeningEntry { moves: &["e4", "d6"], eco: "B07", name: "Pirc Defense" },
    OpeningEntry { moves: &["e4", "g6"], eco: "B06", name: "Modern Defense" },

    // ---- 1.d4 lines ----
    OpeningEntry { moves: &["d4", "d5"], eco: "D00", name: "Queen's Pawn Game" },
    OpeningEntry { moves: &["d4", "d5", "c4"], eco: "D06", name: "Queen's Gambit" },
    OpeningEntry { moves: &["d4", "d5", "c4", "e6"], eco: "D30", name: "Queen's Gambit Declined" },
    OpeningEntry { moves: &["d4", "d5", "c4", "c6"], eco: "D10", name: "Slav Defense" },
    OpeningEntry { moves: &["d4", "d5", "c4", "dxc4"], eco: "D20", name: "Queen's Gambit Accepted" },
    OpeningEntry { moves: &["d4", "d5", "Nf3"], eco: "D02", name: "Queen's Pawn: Krause" },
    OpeningEntry { moves: &["d4", "d5", "Bf4"], eco: "D00", name: "London System" },
    OpeningEntry { moves: &["d4", "d5", "Nf3", "Nf6", "Bf4"], eco: "D02", name: "London System" },
    OpeningEntry { moves: &["d4", "Nf6"], eco: "A45", name: "Indian Defense" },
    OpeningEntry { moves: &["d4", "Nf6", "c4", "e6", "Nc3", "Bb4"], eco: "E20", name: "Nimzo-Indian Defense" },
    OpeningEntry { moves: &["d4", "Nf6", "c4", "g6"], eco: "E60", name: "King's Indian / Grünfeld" },
    OpeningEntry { moves: &["d4", "Nf6", "c4", "g6", "Nc3", "d5"], eco: "D80", name: "Grünfeld Defense" },
    OpeningEntry { moves: &["d4", "Nf6", "c4", "g6", "Nc3", "Bg7"], eco: "E60", name: "King's Indian Defense" },
    OpeningEntry { moves: &["d4", "Nf6", "c4", "c5"], eco: "A56", name: "Benoni Defense" },
    OpeningEntry { moves: &["d4", "Nf6", "c4", "e5"], eco: "A51", name: "Budapest Gambit" },
    OpeningEntry { moves: &["d4", "f5"], eco: "A80", name: "Dutch Defense" },

    // ---- Flank & other openings ----
    OpeningEntry { moves: &["c4"], eco: "A10", name: "English Opening" },
    OpeningEntry { moves: &["c4", "e5"], eco: "A20", name: "English: King's English" },
    OpeningEntry { moves: &["c4", "c5"], eco: "A30", name: "English: Symmetrical" },
    OpeningEntry { moves: &["Nf3"], eco: "A04", name: "Réti Opening" },
    OpeningEntry { moves: &["Nf3", "d5", "g3"], eco: "A07", name: "King's Indian Attack" },
    OpeningEntry { moves: &["f4"], eco: "A02", name: "Bird's Opening" },
    OpeningEntry { moves: &["b3"], eco: "A01", name: "Nimzo-Larsen Attack" },
    OpeningEntry { moves: &["g3"], eco: "A00", name: "Hungarian Opening" },
    OpeningEntry { moves: &["b4"], eco: "A00", name: "Polish (Sokolsky) Opening" },
    OpeningEntry { moves: &["g4"], eco: "A00", name: "Grob Opening" },
    OpeningEntry { moves: &["a3"], eco: "A00", name: "Anderssen's Opening" },
    OpeningEntry { moves: &["Nc3"], eco: "A00", name: "Dunst Opening" },
];

/// Match a sequence of SAN moves against the opening book using longest prefix matching.
pub fn identify_opening(san_moves: &[String]) -> OpeningMatch {
    let mut best_match: Option<&OpeningEntry> = None;
    let mut max_len = 0;

    for entry in OPENING_BOOK {
        let entry_len = entry.moves.len();
        if entry_len <= san_moves.len() {
            let matches = entry.moves.iter().enumerate().all(|(idx, &m)| san_moves[idx] == m);
            if matches && entry_len > max_len {
                max_len = entry_len;
                best_match = Some(entry);
            }
        }
    }

    if let Some(entry) = best_match {
        OpeningMatch {
            eco: entry.eco.to_string(),
            name: entry.name.to_string(),
        }
    } else {
        OpeningMatch {
            eco: "A00".to_string(),
            name: "Unclassified Opening".to_string(),
        }
    }
}
