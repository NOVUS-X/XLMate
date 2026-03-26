"use client";

import React from "react";
import { useAppContext } from "@/context/walletContext";
import { Button } from "./ui/button";
import { Loader2, LogOut, AlertCircle } from "lucide-react";

export function ConnectWalletButton() {
  const { address, status, connectWallet, disconnectWallet } = useAppContext();
  const [isLoading, setIsLoading] = React.useState(false);

  const handleConnect = async () => {
    setIsLoading(true);
    try {
      await connectWallet();
    } catch (err) {
      console.error("Failed to connect:", err);
    } finally {
      setIsLoading(false);
    }
  };

  const handleDisconnect = async () => {
    setIsLoading(true);
    try {
      await disconnectWallet();
    } finally {
      setIsLoading(false);
    }
  };

  // State: Disconnected (idle)
  if (status === "disconnected") {
    return (
      <Button
        onClick={handleConnect}
        disabled={isLoading}
        className="bg-blue-600 hover:bg-blue-700 text-white px-4"
      >
        {isLoading ? (
          <>
            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
            Connecting...
          </>
        ) : (
          "Connect Wallet"
        )}
      </Button>
    );
  }

  // State: Connecting
  if (status === "connecting") {
    return (
      <Button disabled className="bg-gray-400 text-white px-4">
        <Loader2 className="mr-2 h-4 w-4 animate-spin" />
        Connecting...
      </Button>
    );
  }

  // State: Connected
  if (status === "connected" && address) {
    const truncatedAddress = `${address.slice(0, 6)}...${address.slice(-6)}`;
    return (
      <div className="flex items-center gap-2">
        <Button
          variant="outline"
          disabled
          className="bg-green-50 dark:bg-green-950 border-green-200 dark:border-green-800 text-green-900 dark:text-green-100 px-3 cursor-default"
        >
          <div className="w-2 h-2 bg-green-600 rounded-full mr-2"></div>
          {truncatedAddress}
        </Button>
        <Button
          onClick={handleDisconnect}
          disabled={isLoading}
          variant="ghost"
          size="sm"
          className="text-gray-500 hover:text-red-600"
          title="Disconnect wallet"
        >
          {isLoading ? (
            <Loader2 className="h-4 w-4 animate-spin" />
          ) : (
            <LogOut className="h-4 w-4" />
          )}
        </Button>
      </div>
    );
  }

  // State: Network Mismatch
  if (status === "network_mismatch") {
    return (
      <div className="flex items-center gap-2">
        <Button
          disabled
          className="bg-yellow-50 dark:bg-yellow-950 border border-yellow-200 dark:border-yellow-800 text-yellow-900 dark:text-yellow-100 px-4 cursor-default"
        >
          <AlertCircle className="mr-2 h-4 w-4" />
          Wrong Network
        </Button>
        <Button
          onClick={handleDisconnect}
          disabled={isLoading}
          variant="ghost"
          size="sm"
          className="text-gray-500 hover:text-red-600"
          title="Disconnect wallet"
        >
          {isLoading ? (
            <Loader2 className="h-4 w-4 animate-spin" />
          ) : (
            <LogOut className="h-4 w-4" />
          )}
        </Button>
      </div>
    );
  }

  // State: Error
  if (status === "error") {
    return (
      <Button
        onClick={handleConnect}
        disabled={isLoading}
        className="bg-red-600 hover:bg-red-700 text-white px-4"
      >
        {isLoading ? (
          <>
            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
            Retrying...
          </>
        ) : (
          <>
            <AlertCircle className="mr-2 h-4 w-4" />
            Connection Failed
          </>
        )}
      </Button>
    );
  }

  return null;
}
