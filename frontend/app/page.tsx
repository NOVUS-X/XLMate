"use client";

import React, { useState, useEffect } from "react";
import ChessboardComponent from "@/components/chess/ChessboardComponent";
import { Chess } from "chess.js";
import GameModeButtons from "@/components/GameModeButtons";
import { useMatchmaking } from "@/hook/useMatchmaking";
import { FaUser } from "react-icons/fa";
import { RiAliensFill } from "react-icons/ri";

export default function Home() {
  const [game] = useState(new Chess());
  const [position, setPosition] = useState("start");
  const [gameMode, setGameMode] = useState<"online" | "bot" | null>(null);

  const {
    status,
    playerColor,
    error: matchmakingError,
    joinMatchmaking,
    cancelMatchmaking,
    sendMove,
    lastOpponentMove,
  } = useMatchmaking();

  // Kick off matchmaking when online mode is selected
  useEffect(() => {
    if (gameMode === "online") {
      joinMatchmaking();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [gameMode]);

  // Apply opponent's move to local chess state
  useEffect(() => {
    if (!lastOpponentMove) return;
    try {
      const move = game.move({
        from: lastOpponentMove.from,
        to: lastOpponentMove.to,
        promotion: lastOpponentMove.promotion ?? "q",
      });
      if (move) setPosition(game.fen());
    } catch {
      // illegal move from server — ignore
    }
  }, [lastOpponentMove, game]);

  const isMyTurn =
    gameMode !== "online" ||
    (status === "connected" &&
      ((playerColor === "white" && game.turn() === "w") ||
        (playerColor === "black" && game.turn() === "b")));

  const handleMove = ({
    sourceSquare,
    targetSquare,
  }: {
    sourceSquare: string;
    targetSquare: string;
  }) => {
    if (!isMyTurn) return false;

    try {
      const move = game.move({
        from: sourceSquare,
        to: targetSquare,
        promotion: "q",
      });
      if (move === null) return false;

      requestAnimationFrame(() => setPosition(game.fen()));

      // Forward move to server in online mode
      if (gameMode === "online") {
        sendMove(sourceSquare, targetSquare, "q");
      }

      return true;
    } catch {
      return false;
    }
  };

  const handleExit = () => {
    if (gameMode === "online") cancelMatchmaking();
    game.reset();
    setPosition("start");
    setGameMode(null);
  };

  const handleSetGameMode = (mode: "online" | "bot" | null) => {
    setGameMode(mode);
  };

  // Searching / waiting overlay label
  const onlineStatusLabel = () => {
    if (status === "searching") return "🔍 Searching for opponent…";
    if (status === "match_found") return "✅ Match found! Starting…";
    if (status === "connected") return `🟢 Online Match (you are ${playerColor})`;
    if (status === "error") return `❌ ${matchmakingError ?? "Connection error"}`;
    return "Online Match";
  };

  return (
    <div className="min-h-screen bg-gray-900 p-4 md:p-8">
      <div className="max-w-7xl mx-auto">
        <div className="flex flex-col md:flex-row gap-8 items-center justify-center">
          {/* Chessboard Section */}
          <div className="w-full max-w-[600px] order-2 md:order-1">
            <div className="w-full min-w-[320px]">
              <ChessboardComponent position={position} onDrop={handleMove} />
            </div>

            {gameMode && (
              <div className="mt-4 flex items-center justify-between bg-gradient-to-r from-gray-800/50 to-gray-900/50 p-4 rounded-xl border border-teal-500/20">
                <div className="flex items-center gap-4">
                  <div className="bg-gradient-to-br from-teal-400/30 to-blue-500/30 p-3 rounded-xl">
                    {gameMode === "online" ? (
                      <FaUser className="text-2xl text-white filter drop-shadow-md" />
                    ) : (
                      <RiAliensFill className="text-2xl text-white filter drop-shadow-md" />
                    )}
                  </div>
                  <h2 className="text-xl font-bold text-white tracking-wide">
                    {gameMode === "online"
                      ? onlineStatusLabel()
                      : "Playing vs Bot"}
                  </h2>
                </div>

                <button
                  onClick={handleExit}
                  className="px-4 py-2 bg-gradient-to-r from-red-500/20 to-red-600/20 hover:from-red-500/30 hover:to-red-600/30 
                  border border-red-500/30 hover:border-red-400/50 rounded-lg text-white font-medium transition-all duration-300 
                  flex items-center gap-2 group hover:scale-105 active:scale-95"
                >
                  <span>Exit Game</span>
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    className="h-5 w-5 transform transition-transform group-hover:translate-x-1"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2}
                      d="M17 8l4 4m0 0l-4 4m4-4H3"
                    />
                  </svg>
                </button>
              </div>
            )}

            {/* Searching spinner */}
            {gameMode === "online" && status === "searching" && (
              <div className="mt-3 flex items-center gap-2 text-teal-400 text-sm animate-pulse px-1">
                <div className="w-3 h-3 rounded-full border-2 border-teal-400 border-t-transparent animate-spin" />
                Waiting for an opponent to join…
              </div>
            )}
          </div>

          {/* Game Modes Section */}
          <div className="flex flex-col justify-center space-y-6 max-w-[500px] w-full order-1 md:order-2">
            {!gameMode && <GameModeButtons setGameMode={handleSetGameMode} />}
          </div>
        </div>
      </div>
    </div>
  );
}