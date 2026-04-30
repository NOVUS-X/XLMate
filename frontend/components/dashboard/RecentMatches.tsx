"use client";

import { useState, useMemo } from "react";
import type { EloDataPoint } from "@/components/profile/EloRatingChart";
import { cn } from "@/lib/utils";
import { Search, Filter, X, ChevronDown, SearchX } from "lucide-react";

interface RecentMatchesProps {
  data: EloDataPoint[];
}

function getResult(change: number): "W" | "L" | "D" {
  if (change > 0) return "W";
  if (change < 0) return "L";
  return "D";
}

function formatOpponent(opponent: string): string {
  if (opponent.length <= 12) return opponent;
  return `${opponent.slice(0, 6)}...${opponent.slice(-4)}`;
}

const badgeClassMap = {
  W: "border-emerald-500/30 bg-emerald-500/15 text-emerald-400",
  L: "border-red-500/30 bg-red-500/15 text-red-400",
  D: "border-yellow-500/30 bg-yellow-500/15 text-yellow-400",
} as const;

export default function RecentMatches({ data }: RecentMatchesProps) {
  const [searchQuery, setSearchQuery] = useState("");
  const [showFilters, setShowFilters] = useState(false);
  const [resultFilter, setResultFilter] = useState<"ALL" | "W" | "L" | "D">("ALL");

  const filteredMatches = useMemo(() => {
    return [...data]
      .reverse()
      .filter((match) => {
        const matchesSearch = match.opponent.toLowerCase().includes(searchQuery.toLowerCase());
        const result = getResult(match.change);
        const matchesResult = resultFilter === "ALL" || result === resultFilter;
        return matchesSearch && matchesResult;
      })
      .slice(0, 10);
  }, [data, searchQuery, resultFilter]);

  return (
    <section
      role="region"
      aria-label="Recent matches history"
      className="group relative overflow-hidden rounded-2xl border border-gray-700/30 bg-gray-900/40 p-6 backdrop-blur-md transition-all duration-500 hover:border-teal-500/30"
    >
      <div className="absolute -right-24 -top-24 h-48 w-48 rounded-full bg-teal-500/5 blur-[80px] transition-all group-hover:bg-teal-500/10" />
      
      <div className="relative mb-8 flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h2 className="text-xl font-bold tracking-tight text-white">Historical Matches</h2>
          <p className="mt-1 text-sm text-gray-400">Search and analyze your past performance</p>
        </div>
        
        <div className="flex items-center gap-2">
          <div className="relative">
            <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-gray-500" />
            <input
              type="text"
              placeholder="Search opponent..."
              aria-label="Search opponents"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="w-full rounded-xl border border-gray-700/50 bg-gray-950/50 py-2 pl-10 pr-4 text-sm text-white outline-none transition-all focus:border-teal-500/50 focus:ring-1 focus:ring-teal-500/20 sm:w-64"
            />
            {searchQuery && (
              <button 
                onClick={() => setSearchQuery("")}
                className="absolute right-3 top-1/2 -translate-y-1/2 text-gray-500 hover:text-white"
              >
                <X className="h-3 w-3" />
              </button>
            )}
          </div>
          
          <button
            onClick={() => setShowFilters(!showFilters)}
            aria-label="Toggle filters"
            aria-expanded={showFilters}
            className={cn(
              "flex h-10 w-10 items-center justify-center rounded-xl border transition-all duration-300",
              showFilters 
                ? "border-teal-500/50 bg-teal-500/10 text-teal-400" 
                : "border-gray-700/50 bg-gray-800/50 text-gray-400 hover:border-gray-600 hover:text-white"
            )}
          >
            <Filter className="h-4 w-4" />
          </button>
        </div>
      </div>

      {showFilters && (
        <div className="mb-6 flex flex-wrap gap-2 animate-in fade-in slide-in-from-top-2 duration-300">
          {["ALL", "W", "L", "D"].map((r) => (
            <button
              key={r}
              onClick={() => setResultFilter(r as any)}
              className={cn(
                "rounded-lg px-4 py-1.5 text-xs font-bold tracking-wider transition-all",
                resultFilter === r
                  ? "bg-teal-500/20 text-teal-300 border border-teal-500/30"
                  : "bg-gray-800/30 text-gray-500 border border-transparent hover:bg-gray-800/60"
              )}
            >
              {r === "ALL" ? "SHOW ALL" : r === "W" ? "WINS" : r === "L" ? "LOSSES" : "DRAWS"}
            </button>
          ))}
        </div>
      )}

      <div className="overflow-x-auto rounded-xl border border-gray-800/50 bg-black/20">
        <table className="min-w-full text-left text-sm" aria-label="Recent matches table">
          <thead className="bg-gray-800/30">
            <tr className="text-xs uppercase tracking-[0.2em] text-gray-500">
              <th className="px-4 py-4 font-semibold">Timeline</th>
              <th className="px-4 py-4 font-semibold">Adversary</th>
              <th className="px-4 py-4 font-semibold">Outcome</th>
              <th className="px-4 py-4 font-semibold text-right">Rating Impact</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-800/50">
            {filteredMatches.length > 0 ? (
              filteredMatches.map((match, index) => {
                const result = getResult(match.change);
                return (
                  <tr
                    key={`${match.date}-${match.opponent}-${index}`}
                    className="group/row transition-all hover:bg-white/[0.02]"
                  >
                    <td className="px-4 py-4 text-gray-400">
                      {new Date(match.date).toLocaleDateString("en-US", {
                        month: "short",
                        day: "numeric",
                        year: "numeric",
                      })}
                    </td>
                    <td className="px-4 py-4">
                      <div className="flex items-center gap-2">
                        <div className="h-2 w-2 rounded-full bg-gray-700 transition-colors group-hover/row:bg-teal-500" />
                        <span className="font-mono text-white/90">{formatOpponent(match.opponent)}</span>
                      </div>
                    </td>
                    <td className="px-4 py-4">
                      <span className={cn(
                        "inline-flex h-7 min-w-[3rem] items-center justify-center rounded-md border text-[10px] font-black tracking-widest",
                        badgeClassMap[result]
                      )}>
                        {result === "W" ? "VICTORY" : result === "L" ? "DEFEAT" : "DRAW"}
                      </span>
                    </td>
                    <td className="px-4 py-4 text-right">
                      <div className="flex items-center justify-end gap-3">
                        <span className={cn(
                          "font-mono font-bold",
                          match.change >= 0 ? "text-emerald-400" : "text-red-400"
                        )}>
                          {match.change >= 0 ? "+" : ""}{match.change}
                        </span>
                        <div className="h-8 w-[1px] bg-gray-800" />
                        <span className="font-mono text-lg font-black text-white">{match.elo}</span>
                      </div>
                    </td>
                  </tr>
                );
              })
            ) : (
              <tr>
                <td colSpan={4} className="px-4 py-16 text-center">
                  <div className="flex flex-col items-center gap-4">
                    <div className="flex h-16 w-16 items-center justify-center rounded-full bg-gradient-to-br from-teal-400/20 to-blue-500/20 text-teal-300">
                      <SearchX className="h-8 w-8" />
                    </div>
                    <div className="space-y-1">
                      <p className="text-lg font-bold text-white">No matches found</p>
                      <p className="text-sm text-gray-400">Try adjusting your filters or search query</p>
                    </div>
                    <button 
                      onClick={() => {setSearchQuery(""); setResultFilter("ALL");}}
                      className="rounded-full border border-teal-500/30 bg-teal-500/10 px-4 py-1.5 text-xs font-bold text-teal-400 transition-all hover:bg-teal-500/20"
                    >
                      Reset all filters
                    </button>
                  </div>
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>
    </section>
  );
}
