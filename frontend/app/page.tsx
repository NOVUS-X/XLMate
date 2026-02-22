"use client";

import React, { useState } from "react";
import dynamic from "next/dynamic";
const ChessboardComponent = dynamic(() => import("@/components/chess/ChessboardComponent"), { ssr: false });
import { Chess } from "chess.js";
const GameModeButtons = dynamic(() => import("@/components/GameModeButtons"), { ssr: false });
import { FaUser } from "react-icons/fa";
import { RiAliensFill } from "react-icons/ri";

export default function Home() {
  const [game] = useState(new Chess());
  const [position, setPosition] = useState("start");
  const [gameMode, setGameMode] = useState<"online" | "bot" | null>(null);

  const handleMove = ({
    sourceSquare,
    targetSquare,
  }: {
    sourceSquare: string;
    targetSquare: string;
  }) => {
    try {
      const move = game.move({
        from: sourceSquare,
        to: targetSquare,
        promotion: "q",
      });

      if (move === null) return false;
      requestAnimationFrame(() => {
        setPosition(game.fen());
      });
      return true;
    } catch {
      return false;
    }
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
                    {gameMode === "online" ? "Online Match" : "Playing vs Bot"}
                  </h2>
                </div>
                <button
                  onClick={() => {
                    game.reset();
                    setPosition("start");
                    setGameMode(null);
                  }}
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
          </div>

          {/* Game Modes Section */}
          <div className="flex flex-col justify-center space-y-6 max-w-[500px] w-full order-1 md:order-2">
            {!gameMode && <GameModeButtons setGameMode={setGameMode} />}
          </div>
        </div>
      </div>
    </div>
  );
}
