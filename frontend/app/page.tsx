"use client";

import React, { useState, useEffect, useCallback } from "react";
import dynamic from "next/dynamic";
const ChessboardComponent = dynamic(
  () => import("@/components/chess/ChessboardComponent"),
  { ssr: false },
);
import { Chess } from "chess.js";
const GameModeButtons = dynamic(() => import("@/components/GameModeButtons"), {
  ssr: false,
});
import { FaUser } from "react-icons/fa";
import { RiAliensFill } from "react-icons/ri";
import { useChessSocket } from "@/hook/useChessSocket";
import { useMatchmaking } from "@/hook/useMatchmaking";
import { useRouter } from "next/navigation";

export default function Home() {
  const [game] = useState(new Chess());
  const [position, setPosition] = useState("start");
  const [gameMode, setGameMode] = useState<"online" | "bot" | null>(null);
  const router = useRouter();
  const MOCK_PLAYER_COUNT = 847; // replace with real web socket player count

  const {
    status: matchmakingStatus,
    playerColor,
    error: matchmakingError,
    joinMatchmaking,
    cancelMatchmaking,
    sendMove: matchmakingSendMove,
    lastOpponentMove,
    gameId,
  } = useMatchmaking();

  const {
    status: socketStatus,
    sendMove: socketSendMove,
    disconnect: disconnectSocket,
    reconnect: reconnectSocket,
  } = useChessSocket(gameId);

  // Choose which sendMove to use based on game state
  const sendMove = useCallback(
    (from: string, to: string, promotion?: string) => {
      if (gameId) {
        socketSendMove({ from, to, promotion: promotion || "q" });
      } else {
        matchmakingSendMove(from, to, promotion);
      }
    },
    [gameId, socketSendMove, matchmakingSendMove],
  );

  // Kick off matchmaking when online mode is selected
  useEffect(() => {
    if (gameMode === "online") {
      joinMatchmaking();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [gameMode]);

  useEffect(() => {
    if (matchmakingStatus === "match_found" && gameId) {
      router.push(`/play/${gameId}`);
    }
  }, [matchmakingStatus, gameId, router]);

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
    (socketStatus === "connected" &&
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
    if (gameMode === "online") {
      cancelMatchmaking();
      disconnectSocket();
    }
    game.reset();
    setPosition("start");
    setGameMode(null);
  };

  const handleSetGameMode = (mode: "online" | "bot" | null) => {
    if (mode === "online") {
      // Show personality selection before joining the queue
      setGameMode(mode);
      setIsPersonalityModalOpen(true);
    } else {
      setGameMode(mode);
    }
  };

  const handlePersonalityConfirm = () => {
    setIsPersonalityModalOpen(false);
    joinMatchmaking(aiPersonality);
  };

  const handlePersonalityClose = () => {
    setIsPersonalityModalOpen(false);
    setGameMode(null);
  };

  // Searching / waiting overlay label
  const onlineStatusLabel = () => {
    if (socketStatus === "reconnecting") return "🔄 Reconnecting...";
    if (matchmakingStatus === "match_found") return "✅ Match found! Starting…";
    if (socketStatus === "connected")
      return `🟢 Online Match (you are ${playerColor})`;
    if (matchmakingStatus === "error" || socketStatus === "error")
      return `❌ ${matchmakingError ?? "Connection error"}`;
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
            {gameMode === "online" && matchmakingStatus === "searching" && (
              <div className="fixed inset-0 bg-black/80 backdrop-blur-sm z-50 flex items-center justify-center">
                <div className="bg-gray-800 p-6 rounded-xl border border-yellow-500/30 text-center">
                  <div className="flex flex-col items-center gap-4">
                    <h3 className="text-xl font-bold text-yellow-400">
                      Looking for opponent...
                    </h3>
                    <span className="relative flex h-8 w-8">
                      <span className="animate-ping absolute inline-flex h-full w-full rounded-full border-2 border-yellow-400 opacity-75 [animation-duration:0.8s]"></span>
                      <span className="relative inline-flex rounded-full h-8 w-8 border-2 border-yellow-500 bg-yellow-500/10"></span>
                    </span>

                    <p className="text-gray-300 text-sm">
                      {MOCK_PLAYER_COUNT} Players online
                    </p>

                    <button
                      onClick={handleExit}
                      className="px-4 py-2 bg-gradient-to-r from-red-500/20 to-red-600/20 hover:from-red-500/30 hover:to-red-600/30 
                  border border-red-500/30 hover:border-red-400/50 rounded-lg text-white font-medium transition-all duration-300 
                  flex items-center gap-2 group hover:scale-105 active:scale-95"
                    >
                      <span>Cancel Search</span>
                      <svg
                        xmlns="http://www.w3.org/2000/svg"
                        className="h-5 w-5"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke="currentColor"
                      >
                        <path
                          strokeLinecap="round"
                          strokeLinejoin="round"
                          strokeWidth={2}
                          d="M6 18L18 6M6 6l12 12"
                        />
                      </svg>
                    </button>
                  </div>
                </div>
              </div>
            )}

            {/* Reconnection overlay */}
            {gameMode === "online" && socketStatus === "reconnecting" && (
              <div className="fixed inset-0 bg-black/80 backdrop-blur-sm z-50 flex items-center justify-center">
                <div className="bg-gray-800 p-6 rounded-xl border border-yellow-500/30 text-center">
                  <div className="flex flex-col items-center gap-4">
                    <div className="w-8 h-8 rounded-full border-2 border-yellow-500 border-t-transparent animate-spin" />
                    <h3 className="text-xl font-bold text-yellow-400">
                      Reconnecting...
                    </h3>
                    <p className="text-gray-300 text-sm">
                      Attempting to restore connection
                    </p>
                    <button
                      onClick={reconnectSocket}
                      className="mt-2 px-4 py-2 bg-yellow-500/20 hover:bg-yellow-500/30 border border-yellow-500/50 rounded-lg text-yellow-400 font-medium transition-all duration-300"
                    >
                      Reconnect Now
                    </button>
                  </div>
                </div>
              </div>
            )}
          </div>

          {/* Game Modes Section */}
          <div className="flex flex-col justify-center space-y-6 max-w-[500px] w-full order-1 md:order-2">
            {!gameMode && <GameModeButtons setGameMode={handleSetGameMode} />}
          </div>
        </div>
      </div>

      {/* AI Personality Selection Modal */}
      <AiPersonalityModal
        isOpen={isPersonalityModalOpen}
        onClose={handlePersonalityClose}
        onConfirm={handlePersonalityConfirm}
      />
    </div>
  );
}
