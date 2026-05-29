import { calculateResourceSweepElcc } from '../src/lib/resourceSweepElcc.js';

function assertClose(actual: number | undefined, expected: number, label: string): void {
  if (actual === undefined || Math.abs(actual - expected) > 1e-9) {
    throw new Error(`${label}: expected ${expected}, got ${actual}`);
  }
}

function assertEqual(actual: unknown, expected: unknown, label: string): void {
  if (actual !== expected) {
    throw new Error(`${label}: expected ${expected}, got ${actual}`);
  }
}

function testFirstIntervalAverageEqualsMarginal(): void {
  const values = calculateResourceSweepElcc([
    { capacity: 0, peakGas: 100 },
    { capacity: 100, peakGas: 78 },
    { capacity: 200, peakGas: 70 },
  ]);

  assertClose(values[0].avg_elcc, 0, 'zero-capacity average');
  assertClose(values[0].marginal_elcc, 0, 'zero-capacity marginal');
  assertClose(values[1].avg_elcc, 22, 'first interval average');
  assertClose(values[1].marginal_elcc, 22, 'first interval marginal');
  assertClose(values[2].avg_elcc, 15, 'second point cumulative average');
  assertClose(values[2].marginal_elcc, 8, 'second interval marginal');
}

function testSaturatedResourceHasZeroMarginalAndDecliningAverage(): void {
  const values = calculateResourceSweepElcc([
    { capacity: 0, peakGas: 100 },
    { capacity: 100, peakGas: 80 },
    { capacity: 200, peakGas: 80 },
  ]);

  assertClose(values[1].avg_elcc, 20, 'pre-saturation average');
  assertClose(values[1].marginal_elcc, 20, 'pre-saturation marginal');
  assertClose(values[2].avg_elcc, 10, 'post-saturation average');
  assertClose(values[2].marginal_elcc, 0, 'post-saturation marginal');
}

function testNegativeMarginalIsNotClampedAway(): void {
  const values = calculateResourceSweepElcc([
    { capacity: 0, peakGas: 100 },
    { capacity: 100, peakGas: 95 },
    { capacity: 200, peakGas: 96 },
  ]);

  assertClose(values[2].avg_elcc, 2, 'negative interval cumulative average');
  assertClose(values[2].marginal_elcc, -1, 'negative interval marginal');
}

function testEmptyInput(): void {
  assertEqual(calculateResourceSweepElcc([]).length, 0, 'empty input length');
}

testFirstIntervalAverageEqualsMarginal();
testSaturatedResourceHasZeroMarginalAndDecliningAverage();
testNegativeMarginalIsNotClampedAway();
testEmptyInput();

console.log('resourceSweepElcc tests passed');
