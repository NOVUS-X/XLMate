import { calculateElo } from './elo.service';

describe('calculateElo', () => {
  it('winner gains rating, loser loses rating', () => {
    const result = calculateElo(1200, 1200, 1);
    expect(result.deltaA).toBe(16);
    expect(result.deltaB).toBe(-16);
  });

  it('draw between equal players changes nothing significant', () => {
    const result = calculateElo(1200, 1200, 0.5);
    expect(result.deltaA).toBe(0);
    expect(result.deltaB).toBe(0);
  });

  it('upset: lower-rated player wins gains more', () => {
    const result = calculateElo(1000, 1400, 1);
    expect(result.deltaA).toBeGreaterThan(16);
    expect(result.deltaB).toBeLessThan(-16);
  });

  it('ratings are conserved (zero-sum)', () => {
    const result = calculateElo(1500, 1300, 0);
    expect(result.deltaA + result.deltaB).toBe(0);
  });
});