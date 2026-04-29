"use client";

import React from "react";
import {
  type TransactionRecord,
  type TxPhase,
  type TxType,
} from "@/context/transactionContext";
import { useTransactionContext } from "@/context/transactionContext";

/* ------------------------------------------------------------------ */
/*  Phase configuration                                                */
/* ------------------------------------------------------------------ */

const PHASE_CONFIG: Record<
  TxPhase,
  { icon: React.ReactNode; label: string; color: string; bg: string; animate: boolean }
> = {
  preparing: {
    icon: <svg className="h-4 w-4 animate-spin" viewBox="0 0 24 24"><circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" fill="none" /><path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" /></svg>,
    label: "Preparing",
    color: "text-blue-400",
    bg: "bg-blue-500/10 border-blue-500/30",
    animate: true,
  },
  signing: {
    icon: <svg className="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2}><path strokeLinecap="round" strokeLinejoin="round" d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z" /></svg>,
    label: "Awaiting Signature",
    color: "text-yellow-400",
    bg: "bg-yellow-500/10 border-yellow-500/30",
    animate: false,
  },
  submitting: {
    icon: <svg className="h-4 w-4 animate-spin" viewBox="0 0 24 24"><circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" fill="none" /><path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" /></svg>,
    label: "Submitting",
    color: "text-indigo-400",
    bg: "bg-indigo-500/10 border-indigo-500/30",
    animate: true,
  },
  confirming: {
    icon: <svg className="h-4 w-4 animate-pulse" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2}><path strokeLinecap="round" strokeLinejoin="round" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" /></svg>,
    label: "Confirming",
    color: "text-cyan-400",
    bg: "bg-cyan-500/10 border-cyan-500/30",
    animate: true,
  },
  confirmed: {
    icon: <svg className="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2}><path strokeLinecap="round" strokeLinejoin="round" d="M5 13l4 4L19 7" /></svg>,
    label: "Confirmed",
    color: "text-emerald-400",
    bg: "bg-emerald-500/10 border-emerald-500/30",
    animate: false,
  },
  failed: {
    icon: <svg className="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2}><path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" /></svg>,
    label: "Failed",
    color: "text-red-400",
    bg: "bg-red-500/10 border-red-500/30",
    animate: false,
  },
};

const TX_TYPE_ICON: Record<TxType, string> = {
  payment: "💸",
  contract: "📜",
  stake: "🔒",
  claim: "🎁",
};

/* ------------------------------------------------------------------ */
/*  Lifecycle stepper dots                                             */
/* ------------------------------------------------------------------ */

const PHASE_ORDER: TxPhase[] = [
  "preparing",
  "signing",
  "submitting",
  "confirming",
];

function LifecycleStepper({ phase }: { phase: TxPhase }) {
  const currentIdx = PHASE_ORDER.indexOf(phase);

  return (
    <div className="flex items-center gap-1 mt-2" role="group" aria-label="Transaction progress steps">
      {PHASE_ORDER.map((p, i) => {
        const isCompleted = i < currentIdx || phase === "confirmed";
        const isCurrent = i === currentIdx;
        return (
          <React.Fragment key={p}>
            {i > 0 && (
              <div
                className={`h-0.5 flex-1 rounded-full transition-all duration-300 ${
                  isCompleted ? "bg-emerald-500" : "bg-gray-700/50"
                }`}
              />
            )}
            <div
              className={`w-2 h-2 rounded-full transition-all duration-300 ${
                isCompleted
                  ? "bg-emerald-500"
                  : isCurrent
                    ? "bg-blue-400 animate-pulse"
                    : "bg-gray-700/50"
              }`}
              title={p}
              aria-label={`${p}${isCompleted ? " — completed" : isCurrent ? " — current" : ""}`}
            />
          </React.Fragment>
        );
      })}
    </div>
  );
}

/* ------------------------------------------------------------------ */
/*  Single transaction card                                            */
/* ------------------------------------------------------------------ */

function TransactionCard({
  tx,
  onDismiss,
}: {
  tx: TransactionRecord;
  onDismiss: (id: string) => void;
}) {
  const cfg = PHASE_CONFIG[tx.phase];
  const isTerminal = tx.phase === "confirmed" || tx.phase === "failed";
  const elapsed = tx.resolvedAt
    ? ((tx.resolvedAt - tx.createdAt) / 1000).toFixed(1)
    : null;

  return (
    <div
      className={`rounded-xl border p-3 ${cfg.bg} animate-slide-up transition-all duration-300`}
      role="status"
      aria-label={`Transaction: ${tx.label} — ${cfg.label}`}
    >
      <div className="flex items-start gap-3">
        {/* Type icon */}
        <span className="text-base leading-none mt-0.5" aria-hidden="true">
          {TX_TYPE_ICON[tx.type]}
        </span>

        {/* Details */}
        <div className="flex-1 min-w-0">
          <div className="flex items-center justify-between">
            <p className="text-sm font-medium text-white truncate">
              {tx.label}
            </p>
            {isTerminal && (
              <button
                onClick={() => onDismiss(tx.id)}
                className="text-gray-500 hover:text-white transition-colors text-xs ml-2"
                aria-label="Dismiss transaction"
              >
                ✕
              </button>
            )}
          </div>

          <div className="flex items-center gap-2 mt-1">
            <span className={`flex items-center gap-1 ${cfg.color} text-xs font-medium`}>
              {cfg.icon}
              {cfg.label}
            </span>
            {elapsed && (
              <span className="text-xs text-gray-500">
                {elapsed}s
              </span>
            )}
          </div>

          {/* Lifecycle stepper for active transactions */}
          {!isTerminal && <LifecycleStepper phase={tx.phase} />}

          {/* Hash link */}
          {tx.hash && (
            <p className="text-xs text-gray-500 font-mono mt-1 truncate">
              {tx.hash.slice(0, 16)}...
            </p>
          )}

          {/* Error message */}
          {tx.error && (
            <p className="text-xs text-red-400 mt-1">{tx.error}</p>
          )}
        </div>
      </div>
    </div>
  );
}

/* ------------------------------------------------------------------ */
/*  Main component: TransactionStatusIndicator                         */
/* ------------------------------------------------------------------ */

export function TransactionStatusIndicator() {
  const { transactions, dismissTransaction, clearResolved, activeCount } =
    useTransactionContext();

  if (transactions.length === 0) return null;

  return (
    <div
      className="fixed bottom-4 left-4 z-[90] w-80 space-y-2"
      role="region"
      aria-label="Transaction status"
    >
      {/* Active counter badge */}
      {activeCount > 0 && (
        <div className="flex items-center gap-2 px-3 py-2 rounded-xl bg-gray-900/90 backdrop-blur-md border border-blue-500/30 shadow-lg">
          <span className="w-2 h-2 rounded-full bg-blue-400 animate-pulse" aria-hidden="true" />
          <span className="text-xs text-blue-400 font-medium">
            {activeCount} {activeCount === 1 ? "transaction" : "transactions"} in progress
          </span>
        </div>
      )}

      {/* Transaction cards */}
      <div className="max-h-[400px] overflow-y-auto space-y-2 custom-scrollbar">
        {transactions.slice(0, 5).map((tx) => (
          <TransactionCard
            key={tx.id}
            tx={tx}
            onDismiss={dismissTransaction}
          />
        ))}
      </div>

      {/* Clear resolved button */}
      {transactions.some(
        (t) => t.phase === "confirmed" || t.phase === "failed",
      ) && (
        <button
          onClick={clearResolved}
          className="w-full text-xs text-gray-500 hover:text-gray-300 transition-colors py-1"
          aria-label="Clear completed transactions"
        >
          Clear completed
        </button>
      )}
    </div>
  );
}
