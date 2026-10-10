import type { AnalyzedGame } from './ratings';
import { computeAdvantageCapitalizationSkill, computeResourcefulnessSkill } from './skills';

export interface AdvantageResourcefulnessReport {
  advantageCapitalization: ReturnType<typeof computeAdvantageCapitalizationSkill>;
  resourcefulness: ReturnType<typeof computeResourcefulnessSkill>;
  blownLeads: AnalyzedGame[]; // material-heuristic blown leads (not engine)
  comebacks: AnalyzedGame[]; // material-heuristic comebacks (not engine)
}

export function computeAdvantageResourcefulnessReport(
  games: AnalyzedGame[],
): AdvantageResourcefulnessReport {
  return {
    advantageCapitalization: computeAdvantageCapitalizationSkill(games),
    resourcefulness: computeResourcefulnessSkill(games),
    blownLeads: games.filter((g) => g.blownLead),
    comebacks: games.filter((g) => g.comeback),
  };
}
