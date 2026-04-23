// ELO K-factor: 32 for standard play, matching the contract's i32 rating type
const K = 32;

export interface EloResult {
  newRatingA: number;
  newRatingB: number;
  deltaA: number;
  deltaB: number;
}

/**
 * outcome: 1 = A wins, 0 = B wins, 0.5 = draw
 */
export function calculateElo(
  ratingA: number,
  ratingB: number,
  outcome: 1 | 0 | 0.5,
): EloResult {
  const expectedA = 1 / (1 + Math.pow(10, (ratingB - ratingA) / 400));
  const expectedB = 1 - expectedA;

  const deltaA = Math.round(K * (outcome - expectedA));
  const deltaB = Math.round(K * ((1 - outcome) - expectedB));

  return {
    newRatingA: ratingA + deltaA,
    newRatingB: ratingB + deltaB,
    deltaA,
    deltaB,
  };
}