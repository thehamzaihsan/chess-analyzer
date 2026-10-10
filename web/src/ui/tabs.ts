export type TabId = 'overview' | 'openings' | 'tactics' | 'habits' | 'games';

export const TABS: { id: TabId; label: string }[] = [
  { id: 'overview', label: 'Overview' },
  { id: 'openings', label: 'Openings' },
  { id: 'tactics', label: 'Tactics & endgame' },
  { id: 'habits', label: 'Time & habits' },
  { id: 'games', label: 'Games & opponents' },
];

export const SECTION_TABS: Record<string, TabId> = {
  summary: 'overview',
  profile: 'overview',
  skills: 'overview',
  takeaways: 'overview',
  'win-rate': 'overview',
  'rating-over-time': 'overview',
  opening: 'openings',
  'top-openings': 'openings',
  tactics: 'tactics',
  'advantage-resourcefulness': 'tactics',
  endgame: 'tactics',
  'time-management': 'habits',
  'board-habits': 'habits',
  'when-you-play': 'habits',
  games: 'games',
  results: 'games',
  opponents: 'games',
  'result-by-rating': 'games',
};
