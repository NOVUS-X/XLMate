
"use client";
import React, { useState } from "react";
import {
  ChevronLeft,
  ChevronRight,
  Zap,
  Award,
  Coins,
  Gamepad2,
} from "lucide-react";

const KeyFeatures = () => {
  const [activeFeature, setActiveFeature] = useState(0);

  const features = [
    {
      id: 1,
      title: "On-Chain Moves",
      description:
        "Every chess move is recorded on blockchain with verifiable proof, ensuring complete transparency and immutability.",
      icon: <Zap className="h-12 w-12 text-purple-500" />,
      image: "https://via.placeholder.com/800x600",
      color: "from-purple-500 to-blue-500",
    },
    {
      id: 2,
      title: "NFT Rewards",
      description:
        "Earn exclusive NFTs for tournament victories, brilliant moves, and achieving milestones. Each NFT contains your game history and achievements.",
      icon: <Award className="h-12 w-12 text-amber-500" />,
      image: "https://via.placeholder.com/800x600",
      color: "from-amber-500 to-orange-500",
    },
    {
      id: 3,
      title: "Token Wagering",
      description:
        "Stake your tokens on matches and earn rewards based on your performance. Participate in prize pools and climb the leaderboards.",
      icon: <Coins className="h-12 w-12 text-emerald-500" />,
      image: "https://via.placeholder.com/800x600",
      color: "from-emerald-500 to-teal-500",
    },
    {
      id: 4,
      title: "Game Modes",
      description:
        "Choose from multiple game modes: Standard, Blitz, and Bullet. Each mode has its own ranking system and tournament structure.",
      icon: <Gamepad2 className="h-12 w-12 text-rose-500" />,
      image: "https://via.placeholder.com/800x600",
      color: "from-rose-500 to-pink-500",
    },
  ];

  const nextFeature = () => {
    setActiveFeature((prev) => (prev === features.length - 1 ? 0 : prev + 1));
  };

  const prevFeature = () => {
    setActiveFeature((prev) => (prev === 0 ? features.length - 1 : prev - 1));
  };

  return (
    <section className="py-16 bg-gradient-to-br from-gray-900 to-black text-white relative overflow-hidden">
      {/* Hexagon Background Pattern */}
      <div className="absolute inset-0 opacity-10 z-0">
        <div className="absolute top-0 left-0 w-full h-full bg-[url('/hexagon-pattern.svg')] bg-repeat"></div>
      </div>

      {/* Circuit-like Lines */}
      <div className="absolute inset-0 z-0">
        <div className="absolute top-1/4 left-0 w-full h-px bg-gradient-to-r from-transparent via-blue-500 to-transparent"></div>
        <div className="absolute top-3/4 left-0 w-full h-px bg-gradient-to-r from-transparent via-purple-500 to-transparent"></div>
        <div className="absolute top-0 left-1/4 w-px h-full bg-gradient-to-b from-transparent via-green-500 to-transparent"></div>
        <div className="absolute top-0 left-3/4 w-px h-full bg-gradient-to-b from-transparent via-pink-500 to-transparent"></div>
      </div>

      {/* Glowing Orbs */}
      <div className="absolute top-1/4 right-1/4 w-64 h-64 bg-blue-500 rounded-full blur-3xl opacity-20"></div>
      <div className="absolute bottom-1/4 left-1/4 w-64 h-64 bg-purple-500 rounded-full blur-3xl opacity-20"></div>

      <div className="container mx-auto px-4 relative z-10">
        <div className="text-center mb-16">
          <h2 className="text-4xl md:text-5xl font-bold mb-4 bg-clip-text text-transparent bg-gradient-to-r from-blue-500 via-purple-500 to-pink-500">
            Key Features
          </h2>
          <p className="text-gray-400 max-w-2xl mx-auto">
            Experience chess like never before with our Web3-powered platform on
            XLMate, combining traditional gameplay with blockchain innovation.
          </p>
        </div>

        <div className="flex flex-col lg:flex-row items-center justify-between gap-8">
          {/* Feature Navigation (Mobile) */}
          <div className="flex items-center justify-center gap-4 lg:hidden w-full mb-8">
            <button
              onClick={prevFeature}
              className="p-2 rounded-full bg-gray-800 hover:bg-gray-700 transition-colors"
            >
              <ChevronLeft className="w-6 h-6" />
            </button>

            <div className="flex gap-2">
              {features.map((_, index) => (
                <button
                  key={index}
                  onClick={() => setActiveFeature(index)}
                  className={`w-3 h-3 rounded-full transition-colors ${index === activeFeature ? "bg-blue-500" : "bg-gray-700"
                    }`}
                />
              ))}
            </div>

            <button
              onClick={nextFeature}
              className="p-2 rounded-full bg-gray-800 hover:bg-gray-700 transition-colors"
            >
              <ChevronRight className="w-6 h-6" />
            </button>
          </div>

          {/* Feature Cards (Desktop) */}
          <div className="hidden lg:flex flex-col gap-4 w-1/3">
            {features.map((feature, index) => (
              <div
                key={feature.id}
                className={`p-6 rounded-2xl cursor-pointer transition-all duration-300 border border-gray-800 hover:border-gray-700 ${activeFeature === index
                    ? `bg-gradient-to-r ${feature.color} shadow-lg shadow-${feature.color.split("-")[1]
                    }-500/20`
                    : "bg-gray-900 hover:bg-gray-800"
                  }`}
                onClick={() => setActiveFeature(index)}
              >
                <div className="flex items-center gap-4">
                  <div
                    className={`p-3 rounded-xl bg-gray-800 ${activeFeature === index ? "bg-opacity-30" : ""
                      }`}
                  >
                    {feature.icon}
                  </div>
                  <div>
                    <h3 className="text-xl font-bold">{feature.title}</h3>
                    {activeFeature === index && (
                      <p className="text-sm text-gray-300 mt-2">
                        {feature.description}
                      </p>
                    )}
                  </div>
                </div>
              </div>
            ))}
          </div>

          {/* Feature Showcase */}
          <div className="w-full lg:w-2/3 rounded-3xl bg-gray-900 border border-gray-800 overflow-hidden">
            <div className="relative aspect-video">
              {/* Replace with actual feature visualization */}
              <div
                className={`absolute inset-0 bg-gradient-to-br ${features[activeFeature].color} opacity-10`}
              ></div>

              {/* Placeholder for feature visualization */}
              <div className="absolute inset-0 flex items-center justify-center">
                {activeFeature === 0 && (
                  <div className="w-full h-full p-8 flex flex-col items-center justify-center">
                    <div className="grid grid-cols-8 grid-rows-8 gap-1 max-w-md">
                      {Array(64)
                        .fill(0)
                        .map((_, i) => {
                          const row = Math.floor(i / 8);
                          const col = i % 8;
                          const isBlack = (row + col) % 2 === 1;
                          return (
                            <div
                              key={i}
                              className={`aspect-square ${isBlack ? "bg-gray-700" : "bg-gray-600"
                                } rounded relative`}
                            >
                              {[12, 21, 33, 42].includes(i) && (
                                <div className="absolute inset-0 flex items-center justify-center">
                                  <div className="w-3 h-3 bg-blue-500 rounded-full animate-pulse"></div>
                                </div>
                              )}
                            </div>
                          );
                        })}
                    </div>
                    <div className="mt-6 text-center">
                      <p className="text-blue-500 font-mono text-xs">
                        ✓ Move e4 verified on Starknet • Block #2498753 •
                        0x8fc...
                      </p>
                    </div>
                  </div>
                )}

                {activeFeature === 1 && (
                  <div className="w-full h-full p-8 flex items-center justify-center">
                    <div className="relative w-64 h-64">
                      <div className="absolute inset-0 rounded-full border-4 border-amber-500 animate-spin-slow"></div>
                      <div className="absolute inset-4 rounded-full border-2 border-dashed border-amber-400 animate-spin-slow-reverse"></div>
                      <div className="absolute inset-8 rounded-full bg-gradient-to-br from-amber-400 to-orange-500 flex items-center justify-center">
                        <div className="text-center">
                          <Award className="h-12 w-12 text-white mx-auto" />
                          <p className="text-white font-bold mt-2">
                            Grandmaster NFT
                          </p>
                          <p className="text-xs text-white/80">
                            Rarity: Legendary
                          </p>
                        </div>
                      </div>
                    </div>
                  </div>
                )}

                {activeFeature === 2 && (
                  <div className="w-full h-full p-8 flex items-center justify-center">
                    <div className="bg-gray-800 p-6 rounded-xl w-full max-w-md">
                      <div className="flex justify-between items-center mb-6">
                        <div>
                          <p className="text-sm text-gray-400">Current Stake</p>
                          <p className="text-2xl font-bold">250 STARK</p>
                        </div>
                        <div>
                          <p className="text-sm text-gray-400">
                            Potential Reward
                          </p>
                          <p className="text-2xl font-bold text-emerald-500">
                            +375 STARK
                          </p>
                        </div>
                      </div>
                      <div className="flex gap-4">
                        <button className="flex-1 bg-emerald-500 hover:bg-emerald-600 text-white p-2 rounded-lg">
                          Increase Stake
                        </button>
                        <button className="flex-1 bg-gray-700 hover:bg-gray-600 text-white p-2 rounded-lg">
                          Withdraw
                        </button>
                      </div>
                    </div>
                  </div>
                )}

                {activeFeature === 3 && (
                  <div className="w-full h-full p-8 flex items-center justify-center">
                    <div className="grid grid-cols-1 md:grid-cols-3 gap-4 w-full max-w-2xl">
                      <div className="bg-gray-800 p-4 rounded-xl border border-blue-500/30">
                        <div className="flex items-center justify-between mb-2">
                          <h4 className="font-bold">Standard</h4>
                          <p className="text-xs text-gray-400">30 min</p>
                        </div>
                        <div className="mt-4 text-center">
                          <p className="text-xs text-gray-400">
                            Players online
                          </p>
                          <p className="text-xl font-bold">1,254</p>
                        </div>
                      </div>
                      <div className="bg-gradient-to-b from-rose-500 to-pink-500 p-4 rounded-xl transform scale-110 shadow-lg">
                        <div className="flex items-center justify-between mb-2">
                          <h4 className="font-bold">Blitz</h4>
                          <p className="text-xs">5 min</p>
                        </div>
                        <div className="mt-4 text-center">
                          <p className="text-xs text-white/80">
                            Players online
                          </p>
                          <p className="text-xl font-bold">3,987</p>
                        </div>
                      </div>
                      <div className="bg-gray-800 p-4 rounded-xl border border-purple-500/30">
                        <div className="flex items-center justify-between mb-2">
                          <h4 className="font-bold">Bullet</h4>
                          <p className="text-xs text-gray-400">1 min</p>
                        </div>
                        <div className="mt-4 text-center">
                          <p className="text-xs text-gray-400">
                            Players online
                          </p>
                          <p className="text-xl font-bold">2,546</p>
                        </div>
                      </div>
                    </div>
                  </div>
                )}
              </div>
            </div>

            {/* Mobile Feature Description */}
            <div className="lg:hidden p-6 bg-gray-800">
              <h3 className="text-xl font-bold mb-2 flex items-center gap-2">
                {features[activeFeature].icon}
                {features[activeFeature].title}
              </h3>
              <p className="text-gray-300">
                {features[activeFeature].description}
              </p>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
};

export default KeyFeatures;
