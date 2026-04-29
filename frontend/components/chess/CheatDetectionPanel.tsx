"use client";

import React from "react";
import type { HeuristicResult, RiskLevel, HeuristicDetails } from "@/lib/cheatDetection";
import { FaShieldAlt, FaExclamationTriangle, FaCheckCircle } from "react-icons/fa";

interface CheatDetectionPanelProps {
  /** Analysis result for the opponent */
  opponentAnalysis: HeuristicResult;
  /** Analysis result for the current player */
  playerAnalysis: HeuristicResult;
  /** Whether analysis is active */
  isActive: boolean;
  /** Whether the panel is expanded */
  isExpanded: boolean;
  /** Toggle panel expansion */
  onToggle: () => void;
}

const RISK_CONFIG: Record<
  RiskLevel,
  { color: string; bg: string; border: string; label: string }
> = {
  low: {
    color: "text-emerald-400",
    bg: "bg-emerald-500/10",
    border: "border-emerald-500/30",
    label: "Low Risk",
  },
  moderate: {
    color: "text-yellow-400",
    bg: "bg-yellow-500/10",
    border: "border-yellow-500/30",
    label: "Moderate Risk",
  },
  high: {
    color: "text-orange-400",
    bg: "bg-orange-500/10",
    border: "border-orange-500/30",
    label: "High Risk",
  },
  critical: {
    color: "text-red-400",
    bg: "bg-red-500/10",
    border: "border-red-500/30",
    label: "Critical",
  },
};

/**
 * CheatDetectionPanel — Displays live cheat detection analysis
 * during a chess game.
 *
 * CPU-efficient: re-renders only when analysis results change.
 */
export function CheatDetectionPanel({
  opponentAnalysis,
  playerAnalysis,
  isActive,
  isExpanded,
  onToggle,
}: CheatDetectionPanelProps) {
  const oppRisk = RISK_CONFIG[opponentAnalysis.riskLevel];

  return (
    <div className="rounded-xl border border-gray-700/50 bg-gray-800/40 overflow-hidden">
      {/* Header — always visible */}
      <button
        onClick={onToggle}
        aria-expanded={isExpanded}
        aria-controls="cheat-detection-details"
        className="w-full flex items-center justify-between p-4 hover:bg-gray-700/20 transition-colors"
      >
        <div className="flex items-center gap-2">
          <FaShieldAlt
            className={`text-sm ${
              opponentAnalysis.riskLevel === "critical" ||
              opponentAnalysis.riskLevel === "high"
                ? "text-red-400"
                : "text-teal-400"
            }`}
          />
          <h3 className="text-sm font-semibold text-gray-300">
            Fair Play Monitor
          </h3>
        </div>
        <div className="flex items-center gap-2">
          {isActive && (
            <span
              className={`text-xs font-medium px-2 py-0.5 rounded-full ${oppRisk.bg} ${oppRisk.border} border ${oppRisk.color}`}
              aria-label={`Risk level: ${oppRisk.label}`}
            >
              {oppRisk.label}
            </span>
          )}
          <svg
            xmlns="http://www.w3.org/2000/svg"
            className={`h-4 w-4 text-gray-400 transition-transform duration-200 ${
              isExpanded ? "rotate-180" : ""
            }`}
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
            strokeWidth={2}
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              d="M19 9l-7 7-7-7"
            />
          </svg>
        </div>
      </button>

      {/* Expanded details */}
      {isExpanded && (
        <div className="px-4 pb-4 space-y-4 animate-slide-up" id="cheat-detection-details" role="region" aria-label="Fair play analysis details">
          {!isActive ? (
            <p className="text-xs text-gray-500 italic">
              Collecting moves for analysis... (minimum 6 moves needed)
            </p>
          ) : (
            <>
              {/* Opponent analysis */}
              <OpponentSection analysis={opponentAnalysis} />

              {/* Self analysis (subtle, for transparency) */}
              <SelfSection analysis={playerAnalysis} />
            </>
          )}
        </div>
      )}
    </div>
  );
}

// ── Sub-components ──────────────────────────────────────────────────────────

function OpponentSection({ analysis }: { analysis: HeuristicResult }) {
  const cfg = RISK_CONFIG[analysis.riskLevel];

  return (
    <div className="space-y-3">
      <div className="flex items-center gap-2">
        <FaExclamationTriangle className={`text-xs ${cfg.color}`} />
        <span className="text-xs font-semibold text-gray-300 uppercase tracking-wide">
          Opponent Analysis
        </span>
      </div>

      {/* Score bar */}
      <div className="space-y-1">
        <div className="flex items-center justify-between text-xs">
          <span className="text-gray-400">Suspicion Score</span>
          <span className={`font-bold ${cfg.color}`}>{analysis.score}/100</span>
        </div>
        <div className="w-full h-2 rounded-full bg-gray-700/50 overflow-hidden"
          role="progressbar"
          aria-valuenow={analysis.score}
          aria-valuemin={0}
          aria-valuemax={100}
          aria-label={`Suspicion score: ${analysis.score} out of 100`}
        >
          <div
            className={`h-full rounded-full transition-all duration-500 ${
              analysis.score >= 70
                ? "bg-red-500"
                : analysis.score >= 50
                  ? "bg-orange-500"
                  : analysis.score >= 30
                    ? "bg-yellow-500"
                    : "bg-emerald-500"
            }`}
            style={{ width: `${analysis.score}%` }}
          />
        </div>
      </div>

      {/* Heuristic breakdown */}
      <HeuristicBreakdown details={analysis.details} />

      {/* Summary */}
      <p className="text-xs text-gray-400 leading-relaxed">
        {analysis.summary}
      </p>
    </div>
  );
}

function SelfSection({ analysis }: { analysis: HeuristicResult }) {
  return (
    <div className="space-y-2 pt-2 border-t border-gray-700/30">
      <div className="flex items-center gap-2">
        <FaCheckCircle className="text-xs text-gray-500" />
        <span className="text-xs font-semibold text-gray-400 uppercase tracking-wide">
          Your Stats
        </span>
      </div>
      <div className="grid grid-cols-2 gap-2 text-xs">
        <StatItem
          label="Moves"
          value={String(analysis.details.moveCount)}
        />
        <StatItem
          label="Avg Think"
          value={`${analysis.details.avgThinkTime.toFixed(1)}s`}
        />
        <StatItem
          label="Best Move %"
          value={`${(analysis.details.bestMoveRate * 100).toFixed(0)}%`}
        />
        <StatItem
          label="Blunders"
          value={String(analysis.details.blunderCount)}
        />
      </div>
    </div>
  );
}

function HeuristicBreakdown({ details }: { details: HeuristicDetails }) {
  const items = [
    { label: "Time Consistency", value: details.timeConsistency },
    { label: "Move Accuracy", value: details.accuracyScore },
    { label: "Complexity Speed", value: details.complexitySpeed },
    { label: "Blunder Avoidance", value: details.blunderAvoidance },
  ];

  return (
    <div className="space-y-1.5">
      {items.map((item) => (
        <div key={item.label} className="space-y-0.5">
          <div className="flex items-center justify-between text-xs">
            <span className="text-gray-400">{item.label}</span>
            <span
              className={`font-medium ${
                item.value >= 50
                  ? "text-red-400"
                  : item.value >= 30
                    ? "text-yellow-400"
                    : "text-gray-300"
              }`}
            >
              {item.value}%
            </span>
          </div>
          <div className="w-full h-1 rounded-full bg-gray-700/40"
            role="progressbar"
            aria-valuenow={item.value}
            aria-valuemin={0}
            aria-valuemax={100}
            aria-label={`${item.label}: ${item.value} percent`}
          >
            <div
              className={`h-full rounded-full transition-all duration-300 ${
                item.value >= 50
                  ? "bg-red-500/60"
                  : item.value >= 30
                    ? "bg-yellow-500/60"
                    : "bg-gray-500/40"
              }`}
              style={{ width: `${Math.min(100, item.value)}%` }}
            />
          </div>
        </div>
      ))}
    </div>
  );
}

function StatItem({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-center justify-between px-2 py-1 rounded bg-gray-700/20">
      <span className="text-gray-500">{label}</span>
      <span className="text-gray-300 font-medium">{value}</span>
    </div>
  );
}
