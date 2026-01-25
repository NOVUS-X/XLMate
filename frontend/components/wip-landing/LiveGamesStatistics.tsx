"use client";
import React from "react";
import {
  Activity,
  Trophy,
  Coins,
  Zap,
  Users,
  BarChart3,
  Clock,
  CheckCircle,
} from "lucide-react";

// Define types for the data
interface Stat {
  activeGames: number;
  totalGamesPlayed: number;
  tokenWagered: number;
  tokenDistributed: number;
}

interface LiveGame {
  id: number;
  game: string;
  players: string;
  wager: number;
  timeLeft: string;
}

interface RecentWinner {
  id: number;
  player: string;
  game: string;
  prize: number;
  time: string;
}

const LiveGamesStatistics: React.FC = () => {
  // Sample data - in a real app, this would come from your API or blockchain
  const stats: Stat = {
    activeGames: 328,
    totalGamesPlayed: 152847,
    tokenWagered: 1825433,
    tokenDistributed: 1734161,
  };

  const liveGames: LiveGame[] = [
    {
      id: 1,
      game: "Crypto Clash",
      players: "VitalikFan99 vs StarkWizard",
      wager: 250,
      timeLeft: "3:24",
    },
    {
      id: 2,
      game: "Chain Racers",
      players: "Satoshi2023 vs BlockMaster",
      wager: 500,
      timeLeft: "8:42",
    },
    {
      id: 3,
      game: "NFT Battles",
      players: "CryptoQueen vs DeFiChamp",
      wager: 150,
      timeLeft: "1:15",
    },
    {
      id: 4,
      game: "Tournament Finale",
      players: "ZkPlayer vs L2Warrior",
      wager: 350,
      timeLeft: "5:36",
    },
  ];

  const recentWinners: RecentWinner[] = [
    {
      id: 1,
      player: "CryptoKing",
      game: "Crypto Clash",
      prize: 420,
      time: "2 min ago",
    },
    {
      id: 2,
      player: "ChessProXL",
      game: "NFT Battles",
      prize: 275,
      time: "5 min ago",
    },
    {
      id: 3,
      player: "BlockchainBeast",
      game: "Chain Racers",
      prize: 650,
      time: "12 min ago",
    },
    {
      id: 4,
      player: "ZkRollup",
      game: "Starknet Arena",
      prize: 390,
      time: "18 min ago",
    },
  ];

  return (
    <section className="py-16 bg-gradient-to-br from-gray-900 via-black to-gray-900 relative overflow-hidden">
      {/* Animated background elements */}
      <div className="absolute inset-0 opacity-30">
        <div className="absolute h-full w-full">
          {Array.from({ length: 30 }).map((_, i) => (
            <div
              key={i}
              className="absolute h-px opacity-20 bg-gradient-to-r from-transparent via-indigo-500 to-transparent"
              style={{
                top: `${Math.random() * 100}%`,
                left: 0,
                right: 0,
                animation: `pulse ${5 + Math.random() * 10}s infinite`,
                animationDelay: `${Math.random() * 5}s`,
              }}
            ></div>
          ))}
        </div>
      </div>

      {/* Hexagonal grid pattern */}
      <div className="absolute inset-0 opacity-10">
        <div className="absolute h-full w-full bg-[url('data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSI4NiIgaGVpZ2h0PSI0OSI+PHBhdGggZmlsbD0ibm9uZSIgc3Ryb2tlPSIjZmZmIiBzdHJva2Utd2lkdGg9IjEuNSIgZD0iTTEgMTdoMjFsMTAgMTcgMTAtMTdoMjFsLTEwIDE3IDEwIDE3SDQybC0xMC0xNy0xMCAxN0gxbDEwLTE3eiIvPjwvc3ZnPg==')]"></div>
      </div>

      <div className="container mx-auto px-4 relative z-10">
        <div className="text-center mb-12">
          <h2 className="text-4xl md:text-5xl font-bold mb-4 text-transparent bg-clip-text bg-gradient-to-r from-blue-400 via-purple-400 to-indigo-400">
            Live Games & Statistics
          </h2>
          <div className="h-1 w-24 bg-gradient-to-r from-blue-500 to-purple-500 mx-auto mb-6"></div>
          <p className="text-gray-300 max-w-2xl mx-auto text-lg">
            Real-time blockchain gaming metrics and ongoing matches on the
            XLMate ecosystem
          </p>
        </div>

        {/* Key statistics */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-12">
          <div className="bg-gray-800 bg-opacity-60 backdrop-blur-lg rounded-xl p-6 border border-gray-700 transition-all duration-300 hover:border-purple-500 group">
            <div className="flex items-center mb-4">
              <div className="mr-4 p-3 bg-blue-500 bg-opacity-20 rounded-lg">
                <Activity className="w-6 h-6 text-blue-400" />
              </div>
              <h3 className="text-xl font-bold text-white">Active Games</h3>
            </div>
            <p className="text-3xl font-bold text-transparent bg-clip-text bg-gradient-to-r from-blue-400 to-purple-400">
              {stats.activeGames}
            </p>
            <p className="text-gray-400 mt-2 text-sm">
              Live games being played right now
            </p>
          </div>

          <div className="bg-gray-800 bg-opacity-60 backdrop-blur-lg rounded-xl p-6 border border-gray-700 transition-all duration-300 hover:border-purple-500 group">
            <div className="flex items-center mb-4">
              <div className="mr-4 p-3 bg-purple-500 bg-opacity-20 rounded-lg">
                <BarChart3 className="w-6 h-6 text-purple-400" />
              </div>
              <h3 className="text-xl font-bold text-white">Total Games</h3>
            </div>
            <p className="text-3xl font-bold text-transparent bg-clip-text bg-gradient-to-r from-purple-400 to-indigo-400">
              {stats.totalGamesPlayed.toLocaleString()}
            </p>
            <p className="text-gray-400 mt-2 text-sm">
              Games played on-chain since launch
            </p>
          </div>

          <div className="bg-gray-800 bg-opacity-60 backdrop-blur-lg rounded-xl p-6 border border-gray-700 transition-all duration-300 hover:border-purple-500 group">
            <div className="flex items-center mb-4">
              <div className="mr-4 p-3 bg-indigo-500 bg-opacity-20 rounded-lg">
                <Coins className="w-6 h-6 text-indigo-400" />
              </div>
              <h3 className="text-xl font-bold text-white">Token Wagered</h3>
            </div>
            <p className="text-3xl font-bold text-transparent bg-clip-text bg-gradient-to-r from-indigo-400 to-blue-400">
              {stats.tokenWagered.toLocaleString()}
            </p>
            <p className="text-gray-400 mt-2 text-sm">
              Total tokens wagered on the platform
            </p>
          </div>

          <div className="bg-gray-800 bg-opacity-60 backdrop-blur-lg rounded-xl p-6 border border-gray-700 transition-all duration-300 hover:border-purple-500 group">
            <div className="flex items-center mb-4">
              <div className="mr-4 p-3 bg-pink-500 bg-opacity-20 rounded-lg">
                <Zap className="w-6 h-6 text-pink-400" />
              </div>
              <h3 className="text-xl font-bold text-white">Distributed</h3>
            </div>
            <p className="text-3xl font-bold text-transparent bg-clip-text bg-gradient-to-r from-pink-400 to-purple-400">
              {stats.tokenDistributed.toLocaleString()}
            </p>
            <p className="text-gray-400 mt-2 text-sm">
              Tokens distributed to winners
            </p>
          </div>
        </div>

        {/* Live games and recent winners */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
          {/* Live games */}
          <div className="bg-gray-800 bg-opacity-50 backdrop-blur-lg rounded-xl border border-gray-700 overflow-hidden">
            <div className="p-6 border-b border-gray-700 flex items-center justify-between">
              <div className="flex items-center">
                <div className="mr-4 p-2 bg-green-500 bg-opacity-20 rounded-lg">
                  <Users className="w-5 h-5 text-green-400" />
                </div>
                <h3 className="text-xl font-bold text-white">Live Games</h3>
              </div>
              <div className="flex items-center bg-gray-900 rounded-full px-3 py-1">
                <div className="w-2 h-2 rounded-full bg-green-500 mr-2 animate-pulse"></div>
                <span className="text-sm text-green-400">Live</span>
              </div>
            </div>
            <div className="overflow-hidden">
              <ul className="divide-y divide-gray-700">
                {liveGames.map((game) => (
                  <li
                    key={game.id}
                    className="p-4 hover:bg-gray-700 transition-colors duration-200"
                  >
                    <div className="flex justify-between items-center">
                      <div className="flex-1">
                        <h4 className="font-semibold text-white">
                          {game.game}
                        </h4>
                        <p className="text-gray-400 text-sm">{game.players}</p>
                      </div>
                      <div className="flex items-center">
                        <div className="mr-4 text-right">
                          <span className="text-xs text-gray-400">Wager</span>
                          <p className="font-semibold text-indigo-400">
                            {game.wager} STARK
                          </p>
                        </div>
                        <div className="flex items-center bg-gray-900 rounded-lg px-3 py-1">
                          <Clock className="w-4 h-4 text-yellow-400 mr-1" />
                          <span className="text-sm text-yellow-400">
                            {game.timeLeft}
                          </span>
                        </div>
                      </div>
                    </div>
                  </li>
                ))}
              </ul>
            </div>
            <div className="p-4 border-t border-gray-700 text-center">
              <button className="text-blue-400 hover:text-blue-300 transition-colors duration-200 text-sm font-medium">
                View All Live Games
              </button>
            </div>
          </div>

          {/* Recent winners */}
          <div className="bg-gray-800 bg-opacity-50 backdrop-blur-lg rounded-xl border border-gray-700 overflow-hidden">
            <div className="p-6 border-b border-gray-700 flex items-center justify-between">
              <div className="flex items-center">
                <div className="mr-4 p-2 bg-amber-500 bg-opacity-20 rounded-lg">
                  <Trophy className="w-5 h-5 text-amber-400" />
                </div>
                <h3 className="text-xl font-bold text-white">Recent Winners</h3>
              </div>
              <div className="flex items-center bg-gray-900 rounded-full px-3 py-1">
                <CheckCircle className="w-4 h-4 text-amber-400 mr-1" />
                <span className="text-sm text-amber-400">Verified</span>
              </div>
            </div>
            <div className="overflow-hidden">
              <ul className="divide-y divide-gray-700">
                {recentWinners.map((winner) => (
                  <li
                    key={winner.id}
                    className="p-4 hover:bg-gray-700 transition-colors duration-200"
                  >
                    <div className="flex justify-between items-center">
                      <div className="flex-1">
                        <h4 className="font-semibold text-white">
                          {winner.player}
                        </h4>
                        <p className="text-gray-400 text-sm">{winner.game}</p>
                      </div>
                      <div className="flex items-center">
                        <div className="mr-4 text-right">
                          <span className="text-xs text-gray-400">Prize</span>
                          <p className="font-semibold text-green-400">
                            {winner.prize} STARK
                          </p>
                        </div>
                        <div className="text-gray-500 text-sm">
                          {winner.time}
                        </div>
                      </div>
                    </div>
                  </li>
                ))}
              </ul>
            </div>
            <div className="p-4 border-t border-gray-700 text-center">
              <button className="text-blue-400 hover:text-blue-300 transition-colors duration-200 text-sm font-medium">
                View All Winners
              </button>
            </div>
          </div>
        </div>

        {/* View all games CTA */}
        <div className="mt-12 text-center">
          <button className="bg-gradient-to-r from-blue-600 to-purple-600 hover:from-blue-700 hover:to-purple-700 text-white font-bold py-3 px-8 rounded-full transition-all duration-300 hover:shadow-lg hover:shadow-blue-500/30 transform hover:-translate-y-1">
            Explore All Games
          </button>
        </div>
      </div>

      {/* Add custom style for the animation */}
      <style jsx>{`
        @keyframes pulse {
          0%,
          100% {
            opacity: 0.1;
          }
          50% {
            opacity: 0.5;
          }
        }
      `}</style>
    </section>
  );
};

export default LiveGamesStatistics;
