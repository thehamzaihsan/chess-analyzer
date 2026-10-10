export interface OpeningEntry {
  moves: string[];
  eco: string;
  name: string;
}

export interface OpeningMatch {
  eco: string;
  name: string;
}

function line(sanSequence: string, eco: string, name: string): OpeningEntry {
  return { moves: sanSequence.split(' '), eco, name };
}

// Longest-prefix match wins. Order within the list does not matter for
// correctness (we always pick the longest match), only for readability.
export const OPENING_BOOK: OpeningEntry[] = [
  // ---- 1.e4 e5 lines ----
  line('e4 e5', 'C20', "King's Pawn Game"),
  line('e4 e5 Nf3', 'C40', "King's Knight Opening"),
  line('e4 e5 Nf3 Nc6', 'C44', 'Normal Variation'),
  line('e4 e5 Nf3 Nc6 Bc4', 'C50', 'Italian Game'),
  line('e4 e5 Nf3 Nc6 Bc4 Bc5', 'C50', 'Giuoco Piano'),
  line('e4 e5 Nf3 Nc6 Bc4 Nf6', 'C55', 'Two Knights Defense'),
  line('e4 e5 Nf3 Nc6 Bb5', 'C60', 'Ruy Lopez'),
  line('e4 e5 Nf3 Nc6 d4', 'C44', 'Scotch Game'),
  line('e4 e5 Nf3 Nc6 Nc3', 'C46', 'Three Knights Opening'),
  line('e4 e5 Nf3 Nc6 Nc3 Nf6', 'C47', 'Four Knights Game'),
  line('e4 e5 Nf3 Nf6', 'C42', "Petrov's Defense"),
  line('e4 e5 Nf3 d6', 'C41', 'Philidor Defense'),
  line('e4 e5 Nf3 Qf6', 'C40', 'McConnell Defense'),
  line('e4 e5 Nf3 Qe7', 'C40', 'Gunderam Defense'),
  line('e4 e5 Nf3 f6', 'C40', 'Damiano Defense'),
  line('e4 e5 Nf3 f5', 'C40', 'Latvian Gambit'),
  line('e4 e5 Nc3', 'C25', 'Vienna Game'),
  line('e4 e5 Bc4', 'C23', "Bishop's Opening"),
  line('e4 e5 Qh5', 'C20', 'Wayward Queen Attack'),
  line('e4 e5 f4', 'C30', "King's Gambit"),
  line('e4 e5 d4', 'C21', 'Center Game'),
  line('e4 e5 d4 exd4', 'C21', 'Center Game'),
  line('e4 e5 c3', 'C20', 'Leonardis Variation'),
  line('e4 e5 Ne2', 'C20', 'Napoleon Attack'),
  line('e4 e5 Ke2', 'C20', 'Bongcloud Attack'),
  line('e4 Nc6', 'C20', 'Elephant Gambit Opening'),
  line('e4 e5 Nf3 d5', 'C40', 'Elephant Gambit'),

  // ---- other 1.e4 lines ----
  line('e4 c5', 'B20', 'Sicilian Defense'),
  line('e4 c5 Nc3', 'B23', 'Closed Sicilian'),
  line('e4 c5 Nf3', 'B27', 'Sicilian Defense'),
  line('e4 e6', 'C00', 'French Defense'),
  line('e4 c6', 'B10', 'Caro-Kann Defense'),
  line('e4 d5', 'B01', 'Scandinavian Defense'),
  line('e4 Nf6', 'B02', "Alekhine's Defense"),
  line('e4 Nf6 Nc3 d5', 'B02', "Alekhine's Scandinavian Variation"),
  line('e4 d6', 'B07', 'Pirc Defense'),
  line('e4 g6', 'B06', 'Modern Defense'),
  line('e4 Nc6 Nf3', 'B00', 'Nimzowitsch Defense'),
  line('e4 a6', 'B00', 'St. George Defense'),
  line('e4 a5', 'B00', 'Barnes Defense'),
  line('e4 b6', 'B00', 'Owen Defense'),
  line('e4', 'B00', "King's Pawn Opening"),

  // ---- 1.d4 lines ----
  line('d4 d5', 'D00', "Queen's Pawn Game"),
  line('d4 d5 c4', 'D06', "Queen's Gambit"),
  line('d4 d5 c4 e6', 'D30', "Queen's Gambit Declined"),
  line('d4 d5 c4 dxc4', 'D20', "Queen's Gambit Accepted"),
  line('d4 d5 Nf3 Nf6 Bf4', 'D02', 'London System'),
  line('d4 Nf6 Nf3 g6 Bf4', 'A45', 'London System'),
  line('d4 d5 Bf4', 'D00', 'London System'),
  line('d4 c6', 'D10', 'Slav Defense'),
  line('d4 d5 c4 c6', 'D10', 'Slav Defense'),
  line('d4 Nf6', 'A45', 'Indian Defense'),
  line('d4 Nf6 c4 g6', 'E60', "King's Indian Defense"),
  line('d4 f5', 'A80', 'Dutch Defense'),
  line('d4 e5', 'A40', 'Englund Gambit'),
  line('d4 e6', 'A40', 'Horwitz Defense'),
  line('d4 Nc6', 'A40', 'Mikenas Defense'),
  line('d4', 'A40', "Queen's Pawn Opening"),

  // ---- other first moves ----
  line('c4', 'A10', 'English Opening'),
  line('Nf3 d5 c4', 'A04', 'Zukertort Opening'),
  line('Nf3', 'A04', 'Zukertort Opening'),
  line('b3', 'A01', 'Nimzo-Larsen Attack'),
  line('f4', 'A02', 'Bird Opening'),
  line('Nc3', 'A00', 'Van Geet Opening'),
  line('e3', 'A00', "Van 't Kruijs Opening"),
  line('d3', 'A00', 'Mieses Opening'),
  line('a3', 'A00', 'Anderssen Opening'),
  line('h3', 'A00', 'Clemenz Opening'),
  line('b4', 'A00', 'Polish Opening'),
  line('a4', 'A00', 'Ware Opening'),
  line('h4', 'A00', 'Kadas Opening'),
  line('Nh3', 'A00', 'Barnes Opening'),
  line('c3', 'A00', 'Saragossa Opening'),
  line('Nc3 d5 Nf3', 'A00', 'Durkin Opening'),
  line('g3', 'A00', 'Hungarian Opening'),
  line('g4', 'A00', 'Grob Opening'),
  line('a3 e5', 'A00', 'Amar Opening'),
];

function normalizeSan(san: string): string {
  return san.replace(/[+#]/g, '');
}

const UNCOMMON: OpeningMatch = { eco: 'A00', name: 'Uncommon opening' };

export function matchOpening(sanMoves: string[]): OpeningMatch {
  const normalized = sanMoves.map(normalizeSan);
  let best: OpeningEntry | null = null;

  for (const entry of OPENING_BOOK) {
    if (entry.moves.length > normalized.length) continue;
    let matches = true;
    for (let i = 0; i < entry.moves.length; i++) {
      if (entry.moves[i] !== normalized[i]) {
        matches = false;
        break;
      }
    }
    if (matches && (!best || entry.moves.length > best.moves.length)) {
      best = entry;
    }
  }

  if (!best) return UNCOMMON;
  return { eco: best.eco, name: best.name };
}
