"use client";

import React from "react";
import { useAppContext } from "@/context/walletContext";
import { useTransactionContext } from "@/context/transactionContext";

/**
 * Web3StatusBar — A compact indicator shown at the top of the main content area.
 * Displays wallet connection status, network, and a truncated address when connected.
 * CPU-efficient: re-renders only when wallet context values change.
 */
export function Web3StatusBar() {
  const { address, status } = useAppContext();
  const { activeCount } = useTransactionContext();

  const statusConfig: Record<
    string,
    { dot: string; label: string; color: string }
  > = {
    connected: {
      dot: "bg-emerald-400 animate-pulse-glow",
      label: "Connected",
      color: "text-emerald-400",
    },
    connecting: {
      dot: "bg-yellow-400 animate-pulse",
      label: "Connecting",
      color: "text-yellow-400",
    },
    disconnected: {
      dot: "bg-gray-500",
      label: "Not Connected",
      color: "text-gray-400",
    },
    error: {
      dot: "bg-red-400",
      label: "Error",
      color: "text-red-400",
    },
  };

  const cfg = statusConfig[status] ?? statusConfig.disconnected;

  const truncateAddress = (addr: string) =>
    `${addr.slice(0, 6)}...${addr.slice(-4)}`;

  return (
    <div className="flex items-center gap-3 px-4 py-2 rounded-xl bg-gray-800/40 border border-gray-700/30 text-sm" role="status" aria-label="Wallet connection status">
      <span className={`w-2 h-2 rounded-full ${cfg.dot}`} aria-hidden="true" />
      <span className={`${cfg.color} font-medium`}>{cfg.label}</span>
      {address && (
        <>
          <span className="text-gray-600">|</span>
          <span className="font-mono text-gray-300 text-xs">
            {truncateAddress(address)}
          </span>
        </>
      )}
      <span className="text-gray-600">|</span>
      <span className="text-xs text-gray-500 flex items-center gap-1">
        <span className="w-1.5 h-1.5 rounded-full bg-yellow-500" aria-hidden="true" />
        Testnet
      </span>
      {activeCount > 0 && (
        <>
          <span className="text-gray-600">|</span>
          <span className="text-xs text-blue-400 flex items-center gap-1">
            <span className="w-1.5 h-1.5 rounded-full bg-blue-400 animate-pulse" aria-hidden="true" />
            {activeCount} tx
          </span>
        </>
      )}
    </div>
  );
}
