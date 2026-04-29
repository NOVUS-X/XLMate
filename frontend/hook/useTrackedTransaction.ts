"use client";

import { useCallback, useRef } from "react";
import { useTransactionContext } from "@/context/transactionContext";
import type { TxType } from "@/context/transactionContext";
import { useToast } from "@/components/ui/toast";

interface UseTrackedTransactionOptions {
  /** Transaction type */
  type: TxType;
  /** Human-readable label */
  label: string;
  /** Amount for display */
  amount?: string;
  /** Destination for payments */
  destination?: string;
  /** Auto-dismiss after confirmation (ms). Default: 8000 */
  autoDismissMs?: number;
}

/**
 * Hook that wraps an async operation (e.g. sendXLM) with full
 * transaction lifecycle tracking and toast notifications.
 *
 * CPU-efficient: only creates state updates when the phase changes,
 * and auto-dismisses confirmed transactions after a configurable delay.
 */
export function useTrackedTransaction(opts: UseTrackedTransactionOptions) {
  const { startTransaction, updatePhase, dismissTransaction } =
    useTransactionContext();
  const { addToast } = useToast();
  const autoDismissMs = opts.autoDismissMs ?? 8000;
  const dismissTimerRef = useRef<Map<string, ReturnType<typeof setTimeout>>>(
    new Map(),
  );

  const execute = useCallback(
    async <T>(
      fn: (txId: string) => Promise<T>,
    ): Promise<T | undefined> => {
      // Phase 1: Preparing
      const txId = startTransaction({
        type: opts.type,
        phase: "preparing",
        label: opts.label,
        amount: opts.amount,
        destination: opts.destination,
      });

      try {
        // Phase 2: Signing (the fn should call Freighter sign)
        updatePhase(txId, "signing");

        const result = await fn(txId);

        // Phase 3: Submitting
        updatePhase(txId, "submitting");

        // Phase 4: Confirming (simulate — in production, poll Horizon for confirmation)
        updatePhase(txId, "confirming");

        // In production, we'd poll the Horizon API for the transaction result.
        // For now, we assume immediate confirmation after successful submission.
        // A real implementation would look like:
        //   const confirmed = await pollHorizonForConfirmation(hash);
        //   updatePhase(txId, "confirmed", { hash });

        updatePhase(txId, "confirmed");

        addToast({
          severity: "success",
          title: "Transaction Confirmed",
          detail: `${opts.label} completed successfully.`,
        });

        // Auto-dismiss after delay
        const timer = setTimeout(() => {
          dismissTransaction(txId);
          dismissTimerRef.current.delete(txId);
        }, autoDismissMs);
        dismissTimerRef.current.set(txId, timer);

        return result;
      } catch (err) {
        const message =
          err instanceof Error ? err.message : "Transaction failed.";

        updatePhase(txId, "failed", { error: message });

        addToast({
          severity: "error",
          title: "Transaction Failed",
          detail: message,
        });

        return undefined;
      }
    },
    [
      opts.type,
      opts.label,
      opts.amount,
      opts.destination,
      autoDismissMs,
      startTransaction,
      updatePhase,
      dismissTransaction,
      addToast,
    ],
  );

  return { execute };
}
