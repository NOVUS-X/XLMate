import { useEffect, useRef, useState, useCallback } from "react";

export type EnhancedSocketStatus =
  | "idle"
  | "connecting"
  | "connected"
  | "reconnecting"
  | "disconnected"
  | "error";

interface ChessMove {
  from: string;
  to: string;
  promotion?: string;
}

interface WebSocketMessage {
  type: string;
  from?: string;
  to?: string;
  san?: string;
  fen?: string;
  promotion?: string;
  white?: number;
  black?: number;
  result?: string;
  final_fen?: string;
  code?: number;
  message?: string;
  token?: string;
  expires_in?: number;
}

interface UseEnhancedChessSocketReturn {
  status: EnhancedSocketStatus;
  gameId: string | null;
  lastOpponentMove: ChessMove | null;
  connectionInfo: {
    instanceId: string | null;
    reconnectAttempts: number;
    latency: number | null;
  };
  sendMove: (move: ChessMove) => void;
  disconnect: () => void;
  reconnect: () => void;
}

const API_BASE = process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:8080";
const WS_BASE = API_BASE.replace(/^http/, "ws");

// Exponential backoff configuration
const MAX_RECONNECT_ATTEMPTS = 10;
const INITIAL_RECONNECT_DELAY = 1000; // 1 second
const MAX_RECONNECT_DELAY = 30000; // 30 seconds

/**
 * Enhanced WebSocket hook with support for:
 * - Reconnection tokens for seamless horizontal scaling
 * - Multi-instance connection tracking
 * - Improved latency monitoring
 * - Better error recovery
 */
export function useEnhancedChessSocket(
  gameId: string | null,
  accessToken?: string | null
): UseEnhancedChessSocketReturn {
  const [status, setStatus] = useState<EnhancedSocketStatus>("idle");
  const [lastOpponentMove, setLastOpponentMove] = useState<ChessMove | null>(null);
  const [connectionInfo, setConnectionInfo] = useState({
    instanceId: null as string | null,
    reconnectAttempts: 0,
    latency: null as number | null,
  });

  const wsRef = useRef<WebSocket | null>(null);
  const reconnectAttemptsRef = useRef(0);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const moveQueueRef = useRef<ChessMove[]>([]);
  const isManualDisconnectRef = useRef(false);
  const reconnectTokenRef = useRef<string | null>(null);
  const pingTimestampRef = useRef<number | null>(null);

  const clearReconnectTimeout = useCallback(() => {
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
      reconnectTimeoutRef.current = null;
    }
  }, []);

  const calculateReconnectDelay = useCallback((attempt: number): number => {
    const baseDelay = INITIAL_RECONNECT_DELAY * Math.pow(2, attempt);
    const jitter = Math.random() * 0.3 * baseDelay; // Add 0-30% jitter
    const delay = baseDelay + jitter;
    return Math.min(delay, MAX_RECONNECT_DELAY);
  }, []);

  const createWebSocket = useCallback((attemptReconnect = false): WebSocket | null => {
    if (!gameId) return null;

    try {
      // Build WebSocket URL with reconnection token if available
      let wsUrl = `${WS_BASE}/v1/ws/game/${gameId}`;
      const params = new URLSearchParams();

      if (reconnectTokenRef.current && attemptReconnect) {
        params.set("reconnect", reconnectTokenRef.current);
      }

      if (params.toString()) {
        wsUrl += `?${params.toString()}`;
      }

      const ws = new WebSocket(wsUrl);
      wsRef.current = ws;

      if (attemptReconnect) {
        setStatus("reconnecting");
        console.log(`[EnhancedWebSocket] Reconnecting to game ${gameId} (attempt ${reconnectAttemptsRef.current + 1})`);
      } else {
        setStatus("connecting");
        console.log(`[EnhancedWebSocket] Connecting to game ${gameId}`);
      }

      ws.onopen = () => {
        console.log(`[EnhancedWebSocket] ${attemptReconnect ? 'Reconnected' : 'Connected'} to game ${gameId}`);
        setStatus("connected");
        reconnectAttemptsRef.current = 0;
        setConnectionInfo(prev => ({ ...prev, reconnectAttempts: 0 }));

        // Send queued moves immediately upon reconnection
        if (moveQueueRef.current.length > 0) {
          console.log(`[EnhancedWebSocket] Dispatching ${moveQueueRef.current.length} queued moves`);
          moveQueueRef.current.forEach(move => {
            ws.send(JSON.stringify({
              type: "move",
              from: move.from,
              to: move.to,
              promotion: move.promotion,
            }));
          });
          moveQueueRef.current = [];
        }

        // Start latency monitoring
        startLatencyMonitoring(ws);
      };

      ws.onmessage = (event) => {
        try {
          const data: WebSocketMessage = JSON.parse(event.data);

          switch (data.type) {
            case "move":
              console.log(`[EnhancedWebSocket] Received opponent move:`, data);
              setLastOpponentMove({
                from: data.from!,
                to: data.to!,
                promotion: data.promotion,
              });
              break;

            case "clock":
              console.log(`[EnhancedWebSocket] Clock update:`, data);
              // Handle clock updates
              break;

            case "end":
              console.log(`[EnhancedWebSocket] Game ended:`, data);
              // Handle game end
              break;

            case "reconnect_token":
              // Store reconnection token for seamless reconnection
              if (data.token) {
                reconnectTokenRef.current = data.token;
                console.log(`[EnhancedWebSocket] Received reconnection token (expires in ${data.expires_in}s)`);
              }
              break;

            case "error":
              console.error(`[EnhancedWebSocket] Server error:`, data.message);
              break;

            case "pong":
              // Calculate latency
              if (pingTimestampRef.current) {
                const latency = Date.now() - pingTimestampRef.current;
                setConnectionInfo(prev => ({ ...prev, latency }));
              }
              break;

            default:
              console.log(`[EnhancedWebSocket] Unknown message type:`, data);
          }
        } catch (error) {
          console.error("[EnhancedWebSocket] Failed to parse message:", error);
        }
      };

      ws.onerror = (error) => {
        console.error("[EnhancedWebSocket] Error:", error);
        setStatus("error");
      };

      ws.onclose = (event) => {
        console.log(`[EnhancedWebSocket] Closed for game ${gameId}. Code: ${event.code}, Reason: ${event.reason}`);

        if (!isManualDisconnectRef.current) {
          setStatus("disconnected");

          // Attempt reconnection with exponential backoff
          if (reconnectAttemptsRef.current < MAX_RECONNECT_ATTEMPTS) {
            const delay = calculateReconnectDelay(reconnectAttemptsRef.current);
            console.log(`[EnhancedWebSocket] Reconnecting in ${Math.round(delay)}ms`);

            reconnectTimeoutRef.current = setTimeout(() => {
              reconnectAttemptsRef.current++;
              setConnectionInfo(prev => ({
                ...prev,
                reconnectAttempts: reconnectAttemptsRef.current,
              }));
              createWebSocket(true);
            }, delay);
          } else {
            console.log("[EnhancedWebSocket] Max reconnection attempts reached");
            setStatus("error");
          }
        } else {
          setStatus("idle");
          isManualDisconnectRef.current = false;
        }
      };

      return ws;
    } catch (error) {
      console.error("[EnhancedWebSocket] Failed to create:", error);
      setStatus("error");
      return null;
    }
  }, [gameId, calculateReconnectDelay]);

  const startLatencyMonitoring = useCallback((ws: WebSocket) => {
    // Send ping every 10 seconds to monitor latency
    const interval = setInterval(() => {
      if (ws.readyState === WebSocket.OPEN) {
        pingTimestampRef.current = Date.now();
        ws.send(JSON.stringify({ type: "ping" }));
      } else {
        clearInterval(interval);
      }
    }, 10000);

    // Cleanup interval on WebSocket close
    ws.addEventListener("close", () => clearInterval(interval));
  }, []);

  const sendMove = useCallback((move: ChessMove) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({
        type: "move",
        from: move.from,
        to: move.to,
        promotion: move.promotion,
      }));
      console.log("[EnhancedWebSocket] Move sent:", move);
    } else {
      moveQueueRef.current.push(move);
      console.log("[EnhancedWebSocket] Move queued:", move);

      if (status === "disconnected" || status === "idle") {
        reconnectAttemptsRef.current = 0;
        createWebSocket(true);
      }
    }
  }, [status, createWebSocket]);

  const disconnect = useCallback(() => {
    isManualDisconnectRef.current = true;
    clearReconnectTimeout();

    if (wsRef.current) {
      wsRef.current.close(1000, "Manual disconnect");
      wsRef.current = null;
    }

    moveQueueRef.current = [];
    reconnectAttemptsRef.current = 0;
    console.log("[EnhancedWebSocket] Manually disconnected");
  }, [clearReconnectTimeout]);

  const reconnect = useCallback(() => {
    isManualDisconnectRef.current = false;
    clearReconnectTimeout();

    if (wsRef.current) {
      wsRef.current.close();
      wsRef.current = null;
    }

    reconnectAttemptsRef.current = 0;
    console.log("[EnhancedWebSocket] Manual reconnection initiated");
    createWebSocket(true);
  }, [clearReconnectTimeout, createWebSocket]);

  useEffect(() => {
    if (gameId) {
      const ws = createWebSocket();
      return () => {
        isManualDisconnectRef.current = true;
        clearReconnectTimeout();
        if (ws) {
          ws.close();
        }
      };
    } else {
      setStatus("idle");
      setLastOpponentMove(null);
    }
  }, [gameId, createWebSocket, clearReconnectTimeout]);

  useEffect(() => {
    return () => {
      clearReconnectTimeout();
      if (wsRef.current) {
        wsRef.current.close();
      }
    };
  }, [clearReconnectTimeout]);

  return {
    status,
    gameId,
    lastOpponentMove,
    connectionInfo,
    sendMove,
    disconnect,
    reconnect,
  };
}
