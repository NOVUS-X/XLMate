"use client";

import React, {
  createContext,
  useCallback,
  useContext,
  useState,
  useRef,
  type ReactNode,
} from "react";

/* ------------------------------------------------------------------ */
/*  Types                                                              */
/* ------------------------------------------------------------------ */

/**
 * The lifecycle stages of a Stellar/Soroban transaction.
 *
 * preparing → signing → submitting → confirming → confirmed
 *                                                      ↘ failed
 */
export type TxPhase =
  | "preparing"
  | "signing"
  | "submitting"
  | "confirming"
  | "confirmed"
  | "failed";

export type TxType = "payment" | "contract" | "stake" | "claim";

export interface TransactionRecord {
  id: string;
  /** What kind of transaction */
  type: TxType;
  /** Current lifecycle phase */
  phase: TxPhase;
  /** Human-readable label (e.g. "Send 5 XLM") */
  label: string;
  /** When the tx was created */
  createdAt: number;
  /** When the tx reached a terminal state (confirmed/failed) */
  resolvedAt?: number;
  /** Stellar transaction hash (set after submission) */
  hash?: string;
  /** Error message if failed */
  error?: string;
  /** Amount involved (for display) */
  amount?: string;
  /** Destination address (for payments) */
  destination?: string;
}

interface TransactionContextValue {
  /** All tracked transactions (most recent first) */
  transactions: TransactionRecord[];
  /** Track a new transaction, returns the tx ID */
  startTransaction: (tx: Omit<TransactionRecord, "id" | "createdAt" | "resolvedAt" | "hash" | "error">) => string;
  /** Update the phase of a transaction */
  updatePhase: (id: string, phase: TxPhase, extras?: Partial<Pick<TransactionRecord, "hash" | "error" | "resolvedAt">>) => void;
  /** Remove a transaction from tracking */
  dismissTransaction: (id: string) => void;
  /** Clear all resolved transactions */
  clearResolved: () => void;
  /** Get a specific transaction by ID */
  getTransaction: (id: string) => TransactionRecord | undefined;
  /** Number of active (non-terminal) transactions */
  activeCount: number;
}

/* ------------------------------------------------------------------ */
/*  Context                                                            */
/* ------------------------------------------------------------------ */

const TransactionContext = createContext<TransactionContextValue | undefined>(
  undefined,
);

let txCounter = 0;

export function TransactionProvider({ children }: { children: ReactNode }) {
  const [transactions, setTransactions] = useState<TransactionRecord[]>([]);
  const txMap = useRef<Map<string, TransactionRecord>>(new Map());

  const startTransaction = useCallback(
    (tx: Omit<TransactionRecord, "id" | "createdAt" | "resolvedAt" | "hash" | "error">) => {
      const id = `tx-${++txCounter}`;
      const record: TransactionRecord = {
        ...tx,
        id,
        createdAt: Date.now(),
      };
      txMap.current.set(id, record);
      setTransactions((prev) => [record, ...prev]);
      return id;
    },
    [],
  );

  const updatePhase = useCallback(
    (id: string, phase: TxPhase, extras?: Partial<Pick<TransactionRecord, "hash" | "error" | "resolvedAt">>) => {
      const existing = txMap.current.get(id);
      if (!existing) return;

      const updated: TransactionRecord = {
        ...existing,
        phase,
        ...(extras ?? {}),
        ...(phase === "confirmed" || phase === "failed"
          ? { resolvedAt: Date.now() }
          : {}),
      };
      txMap.current.set(id, updated);
      setTransactions((prev) => prev.map((t) => (t.id === id ? updated : t)));
    },
    [],
  );

  const dismissTransaction = useCallback((id: string) => {
    txMap.current.delete(id);
    setTransactions((prev) => prev.filter((t) => t.id !== id));
  }, []);

  const clearResolved = useCallback(() => {
    const resolved = new Set<string>();
    for (const t of txMap.current.values()) {
      if (t.phase === "confirmed" || t.phase === "failed") {
        resolved.add(t.id);
      }
    }
    for (const id of resolved) {
      txMap.current.delete(id);
    }
    setTransactions((prev) =>
      prev.filter((t) => t.phase !== "confirmed" && t.phase !== "failed"),
    );
  }, []);

  const getTransaction = useCallback(
    (id: string) => txMap.current.get(id),
    [],
  );

  const activeCount = transactions.filter(
    (t) => t.phase !== "confirmed" && t.phase !== "failed",
  ).length;

  return (
    <TransactionContext.Provider
      value={{
        transactions,
        startTransaction,
        updatePhase,
        dismissTransaction,
        clearResolved,
        getTransaction,
        activeCount,
      }}
    >
      {children}
    </TransactionContext.Provider>
  );
}

export function useTransactionContext() {
  const ctx = useContext(TransactionContext);
  if (!ctx)
    throw new Error(
      "useTransactionContext must be used within TransactionProvider",
    );
  return ctx;
}
