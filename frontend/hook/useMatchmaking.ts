import { useEffect, useRef, useState, useCallback } from "react";

export type MatchmakingStatus =
  | "idle"
  | "searching"
  | "match_found"
  | "connected"
  | "error";

interface MatchFoundPayload {
  gameId: string;
  color: "white" | "black";
  opponentId: string;
}

interface UseMatchmakingReturn {
  status: MatchmakingStatus;
  gameId: string | null;
  playerColor: "white" | "black" | null;
  error: string | null;
  joinMatchmaking: () => void;
  cancelMatchmaking: () => void;
  sendMove: (from: string, to: string, promotion?: string) => void;
  lastOpponentMove: { from: string; to: string; promotion?: string } | null;
}

const API_BASE = process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:8000";
const WS_BASE = API_BASE.replace(/^http/, "ws");

export function useMatchmaking(): UseMatchmakingReturn {
  const [status, setStatus] = useState<MatchmakingStatus>("idle");
  const [gameId, setGameId] = useState<string | null>(null);
  const [playerColor, setPlayerColor] = useState<"white" | "black" | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [lastOpponentMove, setLastOpponentMove] = useState<{
    from: string;
    to: string;
    promotion?: string;
  } | null>(null);

  const matchmakingWsRef = useRef<WebSocket | null>(null);
  const gameWsRef = useRef<WebSocket | null>(null);
  const sessionIdRef = useRef<string | null>(null);

  const cleanup = useCallback(() => {
    if (matchmakingWsRef.current) {
      matchmakingWsRef.current.close();
      matchmakingWsRef.current = null;
    }
    if (gameWsRef.current) {
      gameWsRef.current.close();
      gameWsRef.current = null;
    }
  }, []);

  const openGameSocket = useCallback((gId: string) => {
    const ws = new WebSocket(`${WS_BASE}/v1/games/${gId}/ws`);
    gameWsRef.current = ws;

    ws.onopen = () => setStatus("connected");

    ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        if (data.type === "move") {
          setLastOpponentMove({
            from: data.from,
            to: data.to,
            promotion: data.promotion,
          });
        }
      } catch {
        // ignore malformed messages
      }
    };

    ws.onerror = () => {
      setError("Game connection error.");
      setStatus("error");
    };

    ws.onclose = () => {
      if (status === "connected") setStatus("idle");
    };
  }, [status]);

  const joinMatchmaking = useCallback(async () => {
    setStatus("searching");
    setError(null);

    try {
      // POST to join matchmaking queue, receive a session token
      const res = await fetch(`${API_BASE}/v1/matchmaking/join`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        credentials: "include",
      });

      if (!res.ok) throw new Error(`Matchmaking failed: ${res.status}`);

      const { sessionId } = await res.json();
      sessionIdRef.current = sessionId;

      // Open WebSocket to listen for match_found event
      const ws = new WebSocket(
        `${WS_BASE}/v1/matchmaking/ws?session=${sessionId}`
      );
      matchmakingWsRef.current = ws;

      ws.onmessage = (event) => {
        try {
          const data: { type: string } & Partial<MatchFoundPayload> =
            JSON.parse(event.data);

          if (data.type === "match_found" && data.gameId && data.color) {
            setGameId(data.gameId);
            setPlayerColor(data.color);
            setStatus("match_found");
            ws.close();
            matchmakingWsRef.current = null;
            openGameSocket(data.gameId);
          }
        } catch {
          // ignore malformed messages
        }
      };

      ws.onerror = () => {
        setError("Matchmaking connection error.");
        setStatus("error");
      };

      ws.onclose = () => {
        if (status === "searching") {
          // closed without match — either cancelled or error
        }
      };
    } catch (err) {
      setError(err instanceof Error ? err.message : "Unknown error");
      setStatus("error");
    }
  }, [openGameSocket, status]);

  const cancelMatchmaking = useCallback(async () => {
    cleanup();
    if (sessionIdRef.current) {
      // Best-effort cancel; ignore errors
      fetch(`${API_BASE}/v1/matchmaking/leave`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ sessionId: sessionIdRef.current }),
        credentials: "include",
      }).catch(() => {});
      sessionIdRef.current = null;
    }
    setStatus("idle");
    setGameId(null);
    setPlayerColor(null);
    setError(null);
  }, [cleanup]);

  const sendMove = useCallback(
    (from: string, to: string, promotion = "q") => {
      if (gameWsRef.current?.readyState === WebSocket.OPEN && gameId) {
        gameWsRef.current.send(
          JSON.stringify({ type: "move", gameId, from, to, promotion })
        );
      }
    },
    [gameId]
  );

  // Cleanup on unmount
  useEffect(() => () => cleanup(), [cleanup]);

  return {
    status,
    gameId,
    playerColor,
    error,
    joinMatchmaking,
    cancelMatchmaking,
    sendMove,
    lastOpponentMove,
  };
}