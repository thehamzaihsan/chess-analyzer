import { el } from './dom';
import {
  fetchAllGamesForPlayer,
  ChessComPlayerNotFoundError,
  recentMonthlyPgnUrls,
} from '../chesscom';
import type { ChesscomInfo } from './state';
import type { SavedSessionSummary } from './storage';
import { logoSvg } from './logo';

const USERNAME_RE = /^[a-z0-9_-]{3,25}$/;

export interface InputScreenOptions {
  savedSession: SavedSessionSummary | null;
  onPgnLoaded: (pgnText: string, chesscom?: ChesscomInfo, variantGamesSkipped?: number) => void;
  onForget: () => void;
  onResume: () => void;
}

export function renderInputScreen(options: InputScreenOptions): HTMLElement {
  const container = el('div', { class: 'section' });

  const statusBox = el('div', { role: 'status', class: 'status-box' }, ['']);
  statusBox.style.display = 'none';

  // --- Username fetch ---
  const usernameInput = el('input', { type: 'text', placeholder: 'Chess.com username', 'aria-label': 'Chess.com username' });
  const fetchButton = el('button', {}, ['Fetch games']);

  let fetching = false;

  async function doFetch() {
    if (fetching) return; // ignore double-clicks / repeated Enter while a fetch is in flight
    const raw = usernameInput.value.trim().toLowerCase();
    if (!USERNAME_RE.test(raw)) {
      statusBox.textContent = 'Enter a valid Chess.com username (3-25 letters, numbers, - or _).';
      statusBox.style.display = 'block';
      return;
    }
    fetching = true;
    fetchButton.setAttribute('disabled', '');
    statusBox.style.display = 'block';
    statusBox.textContent = `Looking up ${raw}…`;
    try {
      const result = await fetchAllGamesForPlayer(raw, {
        onProgress: (p) => {
          statusBox.textContent = `Downloading games: month ${p.monthIndex} of ${p.totalMonths} (${p.monthLabel}), ${p.gamesSoFar} games so far…`;
        },
      });
      statusBox.textContent = `Loaded ${result.pgnText ? 'games' : 'no games'}. ${
        result.variantGamesSkipped ? `${result.variantGamesSkipped} variant games left out.` : ''
      }`;
      options.onPgnLoaded(
        result.pgnText,
        {
          username: raw,
          profile: result.profile,
          stats: result.stats,
          fetchedAt: result.fetchedAt,
        },
        result.variantGamesSkipped,
      );
    } catch (err) {
      if (err instanceof ChessComPlayerNotFoundError) {
        statusBox.textContent = `Chess.com has no player called ${raw}`;
      } else {
        renderFetchFailureHelp(statusBox, raw);
      }
    } finally {
      fetching = false;
      fetchButton.removeAttribute('disabled');
    }
  }

  fetchButton.addEventListener('click', doFetch);
  usernameInput.addEventListener('keydown', (e) => {
    if ((e as KeyboardEvent).key === 'Enter') doFetch();
  });

  usernameInput.className = 'search-input';
  fetchButton.className = 'btn btn-primary btn-lg';
  const searchBar = el('div', { class: 'search-bar' }, [
    el('span', { class: 'search-prefix', 'aria-hidden': 'true' }, ['chess.com/']),
    usernameInput,
    fetchButton,
  ]);

  // --- File picker + drag/drop ---
  const fileInput = el('input', { type: 'file', multiple: true, accept: '.pgn,text/plain', class: 'visually-hidden', id: 'pgn-file' });
  fileInput.addEventListener('change', async () => {
    const files = fileInput.files;
    if (!files || files.length === 0) return;
    const texts = await Promise.all([...files].map((f) => f.text()));
    options.onPgnLoaded(texts.join('\n\n'));
  });

  const filePanel = el(
    'label',
    {
      class: 'option-card dropzone',
      for: 'pgn-file',
      ondragover: (e: Event) => {
        e.preventDefault();
        (e.currentTarget as HTMLElement).classList.add('dragging');
      },
      ondragleave: (e: Event) => (e.currentTarget as HTMLElement).classList.remove('dragging'),
      ondrop: async (e: Event) => {
        e.preventDefault();
        (e.currentTarget as HTMLElement).classList.remove('dragging');
        const dt = (e as DragEvent).dataTransfer;
        if (!dt?.files?.length) return;
        const texts = await Promise.all([...dt.files].map((f) => f.text()));
        options.onPgnLoaded(texts.join('\n\n'));
      },
    },
    [
      el('span', { class: 'option-icon', 'aria-hidden': 'true' }, ['\u21E7']),
      el('strong', {}, ['Upload PGN files']),
      el('span', { class: 'muted' }, ['Drop files here or click to browse']),
      fileInput,
    ],
  );

  // --- Paste ---
  const textarea = el('textarea', { rows: 5, placeholder: '[Event "Live Chess"]\n[White "..."]\n...' }) as HTMLTextAreaElement;
  const pasteButton = el('button', { class: 'btn' }, ['Analyze pasted PGN']);
  pasteButton.addEventListener('click', () => {
    if (textarea.value.trim()) options.onPgnLoaded(textarea.value);
  });
  const pastePanel = el('div', { class: 'option-card' }, [
    el('strong', {}, ['Paste PGN text']),
    textarea,
    pasteButton,
  ]);

  const forgetButton = el('button', { class: 'btn btn-ghost btn-sm', onClick: () => options.onForget() }, [
    'Forget saved games',
  ]);

  const features = ['Results & rating', 'Openings', 'Engine-checked tactics', 'Endgames', 'Time management', 'Habits'];

  container.className = 'hero';
  container.append(
    el('div', { class: 'hero-inner' }, [
      el('div', { class: 'hero-logo' }, [logoSvg(72)]),
      el('span', { class: 'eyebrow' }, ['Private \u00b7 runs in your browser']),
      el('h1', {}, ['See what your chess history says about you']),
      el('p', { class: 'hero-lead' }, [
        'Enter a Chess.com username to analyze every game you have played \u2014 results, openings, engine-checked mistakes, endgames and habits.',
      ]),
      searchBar,
      statusBox,
      el('div', { class: 'feature-pills' }, features.map((f) => el('span', { class: 'pill' }, [f]))),
      el('div', { class: 'divider' }, [el('span', {}, ['or load a PGN'])]),
      el('div', { class: 'option-grid' }, [filePanel, pastePanel]),
      options.savedSession
        ? el('div', { class: 'resume-banner' }, [
            el('span', {}, [
              'Saved session: ',
              el('strong', {}, [options.savedSession.playerName]),
              ` \u00b7 ${options.savedSession.gameCount.toLocaleString()} games`,
            ]),
            el('div', { class: 'resume-actions' }, [
              el('button', { class: 'btn btn-sm', onClick: () => options.onResume() }, ['Continue']),
              forgetButton,
            ]),
          ])
        : '',
    ]),
  );

  return container;
}

function renderFetchFailureHelp(statusBox: HTMLElement, username: string): void {
  statusBox.textContent = '';
  const links = recentMonthlyPgnUrls(username, 24);
  const details = el('details', {}, [
    el('summary', {}, ['Monthly PGN download links (last 24 months)']),
    el(
      'ul',
      {},
      links.map((url) => el('li', {}, [el('a', { href: url, target: '_blank', rel: 'noreferrer' }, [url])])),
    ),
  ]);
  statusBox.append(
    el('p', {}, [
      `Could not reach Chess.com from here. You can browse the games directly: `,
      el('a', { href: `https://www.chess.com/games/archive/${username}`, target: '_blank', rel: 'noreferrer' }, [
        'chess.com/games/archive/' + username,
      ]),
    ]),
    details,
  );
}
