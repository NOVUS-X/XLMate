"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import {
  CheatDetectionEngine,
  HeuristicResult,
} from "@/lib/cheatDetection";
import { type Move } from "chess.js";

interface UseCheatDetectionReturn {
  /** Analysis result for the opponent */
  opponentAnalysis: HeuristicResult;
  /** Analysis result for the current player */
  playerAnalysis: HeuristicResult;
  /** Record a move from either player */
  recordMove: (
    san: string,
    verbose: Move,
    fenBefore: string,
    color: "w" | "b",
    moveNumber: number,
  ) => void;
  /** Reset the engine state */
  reset: () => void;
  /** Is analysis active (enough moves collected) */
  isActive: boolean;
}

const EMPTY_RESULT: HeuristicResult = {
  score: 0,
  riskLevel: "low",
  summary: "Waiting for moves...",
  details: {
    timeConsistency: 0,
    accuracyScore: 0,
    complexitySpeed: 0,
    blunderAvoidance: 0,
    blunderCount: 0,
    moveCount: 0,
    avgThinkTime: 0,
    thinkTimeStdDev: 0,
    bestMoveRate: 0,
  },
};

/**
 * Hook that wraps CheatDetectionEngine for use in React components.
 *
 * Provides live analysis for both the player and their opponent.
 * Analysis is throttled to run at most once every 2 seconds to
 * keep CPU usage low.
 *
 * @param playerColor - Which colour the local player is playing as
 */
export function useCheatDetection(
  playerColor: "white" | "black",
): UseCheatDetectionReturn {
  const engineRef = useRef<CheatDetectionEngine>(
    new CheatDetectionEngine(),
  );
  const [opponentAnalysis, setOpponentAnalysis] =
    useState<HeuristicResult>(EMPTY_RESULT);
  const [playerAnalysis, setPlayerAnalysis] =
    useState<HeuristicResult>(EMPTY_RESULT);
  const lastAnalysisRef = useRef<number>(0);

  const runAnalysis = useCallback(() => {
    const now = Date.now();
    // Throttle: run at most once every 2 seconds
    if (now - lastAnalysisRef.current < 2000) return;
    lastAnalysisRef.current = now;

    const engine = engineRef.current;
    const colorMap =
      playerColor === "white"
        ? { player: "w" as const, opponent: "b" as const }
        : { player: "b" as const, opponent: "w" as const };
    setOpponentAnalysis(engine.analyse(colorMap.opponent));
    setPlayerAnalysis(engine.analyse(colorMap.player));
  }, [playerColor]);

  const recordMove = useCallback(
    (
      san: string,
      verbose: Move,
      fenBefore: string,
      color: "w" | "b",
      moveNumber: number,
    ) => {
      engineRef.current.recordMove({
        san,
        verbose,
        fenBefore,
        color,
        moveNumber,
      });
      runAnalysis();
    },
    [runAnalysis],
  );

  const reset = useCallback(() => {
    engineRef.current.reset();
    setOpponentAnalysis(EMPTY_RESULT);
    setPlayerAnalysis(EMPTY_RESULT);
  }, []);

  // Clean up on unmount
  useEffect(() => {
    const engine = engineRef.current;
    return () => {
      engine.reset();
    };
  }, []);

  const isActive =
    opponentAnalysis.details.moveCount >= 6 ||
    playerAnalysis.details.moveCount >= 6;

  return {
    opponentAnalysis,
    playerAnalysis,
    recordMove,
    reset,
    isActive,
  };
}
