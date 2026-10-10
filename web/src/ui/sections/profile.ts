import { el } from './shared';
import { formatDate } from '../format';
import type { SectionContext } from './shared';

function statGroup(stats: Record<string, unknown>, key: string): Record<string, unknown> | null {
  const group = stats[key];
  return group && typeof group === 'object' ? (group as Record<string, unknown>) : null;
}

function ratingRow(label: string, stats: Record<string, unknown> | null): HTMLElement | null {
  if (!stats) return null;
  const last = statGroup(stats, 'last');
  const best = statGroup(stats, 'best');
  const record = statGroup(stats, 'record');
  if (!last) return null;
  const rating = last.rating as number | undefined;
  const bestRating = best?.rating as number | undefined;
  const bestGameUrl = best?.game as string | undefined;
  const win = (record?.win as number) ?? 0;
  const loss = (record?.loss as number) ?? 0;
  const draw = (record?.draw as number) ?? 0;
  const total = win + loss + draw;
  const score = total ? (((win + 0.5 * draw) / total) * 100).toFixed(1) : '-';
  return el('tr', {}, [
    el('td', {}, [label]),
    el('td', {}, [rating != null ? String(rating) : '-']),
    el(
      'td',
      {},
      bestRating != null
        ? [bestGameUrl ? el('a', { href: bestGameUrl, target: '_blank', rel: 'noreferrer' }, [String(bestRating)]) : String(bestRating)]
        : ['-'],
    ),
    el('td', {}, [`${win}/${draw}/${loss}`]),
    el('td', {}, [`${score}%`]),
  ]);
}

export function renderProfileSection(ctx: SectionContext): HTMLElement | null {
  if (!ctx.chesscom) return null;
  const { profile, stats, username, fetchedAt } = ctx.chesscom;
  const section = el('div', { class: 'section', id: 'profile' }, [el('h2', {}, ['Chess.com profile'])]);

  const avatar = profile.avatar
    ? el('img', {
        class: 'profile-avatar',
        src: String(profile.avatar),
        alt: `${username} avatar`,
        onError: (e: Event) => (e.target as HTMLImageElement).remove(),
      })
    : null;

  const meta: string[] = [];
  if (profile.league) meta.push(`${String(profile.league)} league`);
  if (profile.joined) meta.push(`Joined ${formatDate(new Date(Number(profile.joined) * 1000))}`);
  if (profile.last_online) meta.push(`Last online ${formatDate(new Date(Number(profile.last_online) * 1000))}`);
  if (profile.followers != null) meta.push(`${profile.followers} followers`);

  section.append(
    el('div', { class: 'profile-head' }, [
      avatar ?? '',
      el('div', {}, [
        el('div', { class: 'profile-name' }, [
          el('a', { href: `https://www.chess.com/member/${username}`, target: '_blank', rel: 'noreferrer' }, [username]),
          profile.name ? el('span', { class: 'muted' }, [` \u00b7 ${String(profile.name)}`]) : '',
          profile.country ? el('span', { class: 'tag' }, [` ${String(profile.country).split('/').pop() ?? ''}`]) : '',
        ]),
        el('div', { class: 'profile-meta' }, [meta.join(' \u00b7 ')]),
        el('div', { class: 'kv-note' }, [`Fetched ${formatDate(fetchedAt)}`]),
      ]),
    ]),
  );

  if (stats) {
    const table = el('table', {}, [
      el('thead', {}, [
        el('tr', {}, [
          el('th', {}, ['Time control']),
          el('th', {}, ['Rating']),
          el('th', {}, ['Best']),
          el('th', {}, ['W/D/L']),
          el('th', {}, ['Score']),
        ]),
      ]),
      el(
        'tbody',
        {},
        [
          ratingRow('Rapid', statGroup(stats, 'chess_rapid')),
          ratingRow('Blitz', statGroup(stats, 'chess_blitz')),
          ratingRow('Bullet', statGroup(stats, 'chess_bullet')),
          ratingRow('Daily', statGroup(stats, 'chess_daily')),
          ratingRow('Daily 960', statGroup(stats, 'chess960_daily')),
        ].filter((r): r is HTMLElement => r != null),
      ),
    ]);
    section.append(el('div', { class: 'hscroll' }, [table]));

    const puzzle = statGroup(stats, 'tactics');
    const puzzleHighest = puzzle ? statGroup(puzzle, 'highest') : null;
    const puzzleRush = statGroup(stats, 'puzzle_rush');
    const puzzleRushBest = puzzleRush ? statGroup(puzzleRush, 'best') : null;
    if (puzzleHighest || puzzleRushBest) {
      section.append(
        el('p', { class: 'muted' }, [
          puzzleHighest ? `Puzzle rating: ${puzzleHighest.rating}. ` : '',
          puzzleRushBest ? `Puzzle Rush best: ${puzzleRushBest.score}.` : '',
        ]),
      );
    }
  }

  return section;
}
