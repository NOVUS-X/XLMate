"use client";
import React, { useEffect, useState } from "react";
import { FaArrowDown } from "react-icons/fa"; // Import the arrow icon

import Link from "next/link";
import Chessboard from "./Chessboard";

const HeroSection = () => {
  const [isAnimating, setIsAnimating] = useState(false);

  useEffect(() => {
    setIsAnimating(true);

    // Animation cycling
    const interval = setInterval(() => {
      setIsAnimating(false);
      setTimeout(() => setIsAnimating(true), 100);
    }, 10000);

    return () => clearInterval(interval);
  }, []);

  const scrollToFooter = () => {
    const footer = document.getElementById("footer");
    if (footer) {
      window.scrollTo({
        top: footer.offsetTop,
        behavior: "smooth",
      });
    }
  };

  // Chess piece types for our background elements
  const pieceTypes = ["pawn", "knight", "bishop", "rook", "queen", "king"];

  return (
    <div className="relative min-h-screen overflow-hidden bg-gradient-to-br from-gray-900 via-indigo-950 to-purple-900">
      {/* Blockchain grid background */}
      <div className="absolute inset-0 opacity-20 grid grid-cols-12 grid-rows-12">
        {Array.from({ length: 144 }, (_, i) => (
          <div
            key={i}
            className="jsx-51939d8b18ab707d border border-cyan-500/30 bg-cyan-500/10"
          />
        ))}
      </div>

      {/* Chess piece background elements */}
      <div className="absolute inset-0 z-0 overflow-hidden">
        {Array(15)
          .fill(null)
          .map((_, i) => {
            const piece =
              pieceTypes[Math.floor(Math.random() * pieceTypes.length)];
            const size = `${Math.random() * 0.5 + 0.5}rem`;
            const opacity = Math.random() * 0.3 + 0.2;
            const duration = `${Math.random() * 10 + 10}s`;
            const delay = `${Math.random() * 5}s`;

            return (
              <div
                key={i}
                className={`absolute ${isAnimating ? "animate-float" : ""}`}
                style={{
                  left: `${Math.random() * 100}%`,
                  top: `${Math.random() * 100}%`,
                  width: size,
                  height: size,
                  opacity: opacity,
                  animationDelay: delay,
                  animationDuration: duration,
                }}
              >
                <svg viewBox="0 0 50 50" className="w-full h-full">
                  <path
                    d={getPiecePath(piece)}
                    fill="currentColor"
                    className="text-cyan-400/50"
                  />
                </svg>
              </div>
            );
          })}
      </div>

      {/* Content container */}
      <div className="relative z-10 container mx-auto px-4 h-screen flex flex-col md:flex-row items-center justify-center md:justify-between">
        {/* Text content */}
        <div className="w-full md:w-1/2 text-center md:text-left mb-12 md:mb-0">
          <h1 className="text-4xl md:text-6xl font-bold text-transparent bg-clip-text bg-gradient-to-r from-cyan-400 to-purple-400 mb-6 tracking-tight">
            <span className="">XL</span>
            <span className="">MATE</span>
          </h1>

          <p className="text-gray-300 text-xl md:text-2xl mb-8 max-w-xl">
            Play. Compete. Collect.{" "}
            <span className="text-cyan-400">On-Chain</span>.
            <br />
            <span className="text-lg">Every move secured by Stellar.</span>
          </p>

          <div className="flex flex-col sm:flex-row items-center justify-center md:justify-start space-y-4 sm:space-y-0 sm:space-x-4">
            <Link href="/play">
              <button className="px-8 py-3 rounded-full bg-gradient-to-r from-cyan-500 to-blue-600 text-white font-semibold text-lg hover:from-cyan-600 hover:to-blue-700 transition-all duration-300 shadow-lg hover:shadow-cyan-500/50 w-48">
                Play Now
              </button>
            </Link>

            <button className=" py-3 rounded-full bg-transparent border-2 border-purple-500 text-white font-semibold text-lg hover:bg-purple-500/20 transition-all duration-300 w-48">
              Connect Wallet
            </button>
          </div>
        </div>

        {/* Visual content */}
        <div className="w-full md:w-1/2 flex justify-center ">
          <div className="relative w-72 h-72 md:w-96 md:h-96">
            {/* Chess board with blockchain elements */}
            <div className="absolute inset-0 rotate-45 ">
              <Chessboard />
            </div>

            {/* Circle of small blockchain nodes */}
            {Array(12)
              .fill(null)
              .map((_, i) => (
                <div
                  key={i}
                  className="absolute w-3 h-3 md:w-4 md:h-4 bg-purple-500 rounded-full shadow-lg shadow-purple-500/50"
                  style={{
                    left: `${50 + 45 * Math.cos((i * Math.PI) / 6)}%`,
                    top: `${50 + 45 * Math.sin((i * Math.PI) / 6)}%`,
                    animation: `pulse ${2 + (i % 3)}s infinite ${i * 0.2}s`,
                  }}
                />
              ))}

            {/* Connection lines */}
            <svg
              className="absolute inset-0 w-full h-full"
              viewBox="0 0 100 100"
            >
              <g className="opacity-60">
                {Array(6)
                  .fill(null)
                  .map((_, i) => (
                    <line
                      key={i}
                      x1={50 + 45 * Math.cos((i * Math.PI) / 3)}
                      y1={50 + 45 * Math.sin((i * Math.PI) / 3)}
                      x2={50 + 45 * Math.cos(((i + 3) * Math.PI) / 3)}
                      y2={50 + 45 * Math.sin(((i + 3) * Math.PI) / 3)}
                      stroke="rgb(139, 92, 246)"
                      strokeWidth="0.5"
                      className={isAnimating ? "animate-pulse" : ""}
                    />
                  ))}
              </g>
            </svg>
          </div>
        </div>
        {/* Scroll indicator */}
        <div
          onClick={scrollToFooter}
          className="absolute bottom-8 right-8 w-12 h-12 rounded-full bg-gradient-to-r from-cyan-500 to-purple-600 flex items-center justify-center cursor-pointer shadow-lg hover:shadow-cyan-500/50 transition-all duration-300 group"
          aria-label="Scroll to top"
        >
          <FaArrowDown className=" animate-bounce w-5 h-5 text-white group-hover:scale-110 transition-transform" />
        </div>
      </div>
    </div>
  );
};

// Helper function to get SVG paths for chess pieces
function getPiecePath(piece: string): string {
  const pieces: { [key: string]: string } = {
    pawn: "M 20,42 L 20,35 C 20,33 18,31 18,29 C 18,27 22,27 22,23 C 22,21 20,19 20,19 C 20,19 25,18 25,14 C 25,11 22,10 22,10 L 28,10 C 28,10 25,11 25,14 C 25,18 30,19 30,19 C 30,19 28,21 28,23 C 28,27 32,27 32,29 C 32,31 30,33 30,35 L 30,42 L 20,42",
    rook: "M 18,42 L 18,35 L 16,35 L 16,31 L 14,31 L 14,25 L 16,25 L 16,21 L 14,21 L 14,15 L 36,15 L 36,21 L 34,21 L 34,25 L 36,25 L 36,31 L 34,31 L 34,35 L 32,35 L 32,42 L 18,42",
    knight:
      "M 20,42 L 20,35 C 20,35 18,34 18,32 C 18,30 18,28 19,27 C 20,26 21,23 21,21 C 21,19 20,18 20,16 C 20,14 22,13 22,13 C 25,13 28,15 30,17 C 32,19 34,21 34,25 C 34,29 31,31 31,31 L 31,35 L 29,35 L 29,42 L 20,42",
    bishop:
      "M 20,42 L 20,35 C 20,33 18,31 18,29 C 18,27 22,27 22,23 C 22,21 20,19 20,19 C 20,19 25,18 25,14 C 25,11 22,10 22,10 L 28,10 C 28,10 25,11 25,14 C 25,18 30,19 30,19 C 30,19 28,21 28,23 C 28,27 32,27 32,29 C 32,31 30,33 30,35 L 30,42 L 20,42",
    queen:
      "M 20,42 L 20,35 C 20,35 15,32 15,28 C 15,24 20,24 20,20 C 20,16 15,16 15,12 C 15,8 20,8 20,8 L 30,8 C 30,8 35,8 35,12 C 35,16 30,16 30,20 C 30,24 35,24 35,28 C 35,32 30,35 30,35 L 30,42 L 20,42",
    king: "M 22.5,8 L 22.5,12 L 17.5,12 L 17.5,17 L 22.5,17 L 22.5,42 L 27.5,42 L 27.5,17 L 32.5,17 L 32.5,12 L 27.5,12 L 27.5,8 L 22.5,8",
  };
  return pieces[piece] || pieces.pawn;
}

export default HeroSection;
