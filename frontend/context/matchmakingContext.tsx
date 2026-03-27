"use client";
import React, { createContext, useContext, useState } from "react";

export type AiPersonality = "aggressive" | "defensive" | "sacrificial";

type MatchmakingContextType = {
  aiPersonality: AiPersonality;
  setAiPersonality: (personality: AiPersonality) => void;
};

const MatchmakingContext = createContext<MatchmakingContextType | undefined>(undefined);

export function MatchmakingProvider({ children }: { children: React.ReactNode }) {
  const [aiPersonality, setAiPersonality] = useState<AiPersonality>("aggressive");

  return (
    <MatchmakingContext.Provider value={{ aiPersonality, setAiPersonality }}>
      {children}
    </MatchmakingContext.Provider>
  );
}

export const useMatchmakingContext = () => {
  const ctx = useContext(MatchmakingContext);
  if (!ctx) throw new Error("useMatchmakingContext must be used within MatchmakingProvider");
  return ctx;
};
