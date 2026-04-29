"use client";

import React, { useState, useEffect, useMemo, useCallback, useRef } from "react";
import Image from "next/image";

import WhiteKing from "./chesspieces/white-king.svg";
import WhiteQueen from "./chesspieces/white-queen.svg";
import WhiteBishop from "./chesspieces/white-bishop.svg";
import WhiteKnight from "./chesspieces/white-knight.svg";
import WhiteRook from "./chesspieces/white-rook.svg";
import WhitePawn from "./chesspieces/white-pawn.svg";
import BlackKing from "./chesspieces/black-king.svg";
import BlackQueen from "./chesspieces/black-queen.svg";
import BlackBishop from "./chesspieces/black-bishop.svg";
import BlackKnight from "./chesspieces/black-knight.svg";
import BlackRook from "./chesspieces/black-rook.svg";
import BlackPawn from "./chesspieces/black-pawn.svg";

type BoardOrientation = "white" | "black";

interface ChessboardComponentProps {
  position: string;
  onDrop: (params: { sourceSquare: string; targetSquare: string }) => boolean;
  width?: number; // Optional fixed width
  orientation?: BoardOrientation;
}

// Parse FEN string to board state - memoized pure function
const parseFen = (fen: string): string[][] => {
  if (fen === "start") {
    return [
      ["bR", "bN", "bB", "bQ", "bK", "bB", "bN", "bR"],
      ["bP", "bP", "bP", "bP", "bP", "bP", "bP", "bP"],
      ["", "", "", "", "", "", "", ""],
      ["", "", "", "", "", "", "", ""],
      ["", "", "", "", "", "", "", ""],
      ["", "", "", "", "", "", "", ""],
      ["wP", "wP", "wP", "wP", "wP", "wP", "wP", "wP"],
      ["wR", "wN", "wB", "wQ", "wK", "wB", "wN", "wR"],
    ];
  }

  try {
    const fenParts = fen.split(" ");
    const rows = fenParts[0].split("/");
    const newBoard: string[][] = [];

    rows.forEach((row) => {
      const newRow: string[] = [];
      for (let i = 0; i < row.length; i++) {
        const char = row[i];
        if (isNaN(parseInt(char))) {
          const color = char === char.toUpperCase() ? "w" : "b";
          newRow.push(`${color}${char.toUpperCase()}`);
        } else {
          for (let j = 0; j < parseInt(char); j++) {
            newRow.push("");
          }
        }
      }
      newBoard.push(newRow);
    });

    return newBoard;
  } catch (e) {
    console.error("Error parsing FEN:", e);
    return Array.from({ length: 8 }, () => Array(8).fill(""));
  }
};

// ChessboardComponent with full memoization
const ChessboardComponent: React.FC<ChessboardComponentProps> = ({
  position,
  onDrop,
  width,
  orientation = "white",
}) => {
  const [mounted, setMounted] = useState(false);
  const [boardWidth, setBoardWidth] = useState(width || 560);
  const [selectedSquare, setSelectedSquare] = useState<string | null>(null);
  const [hoveredSquare, setHoveredSquare] = useState<string | null>(null);
  const touchStartSquare = useRef<string | null>(null);
  const boardRef = useRef<HTMLDivElement>(null);

  const boardState = useMemo(() => parseFen(position), [position]);

  const translateDisplayToActual = useCallback(
    (row: number, col: number) => {
      if (orientation === "black") {
        return [7 - row, 7 - col] as const;
      }
      return [row, col] as const;
    },
    [orientation],
  );

  const getSquareNotation = useCallback((row: number, col: number) => {
    return `${String.fromCharCode(97 + col)}${8 - row}`;
  }, []);

  const updateBoardSize = useCallback(
    (parentWidth?: number) => {
      if (typeof window === "undefined") return;
      const containerWidth =
        parentWidth ?? boardRef.current?.parentElement?.clientWidth ??
        window.innerWidth;
      if (!containerWidth) return;

      const maxSize = width || 560;
      const clampWidth = Math.min(containerWidth, maxSize);
      const responsiveWidth =
        window.innerWidth < 768
          ? Math.max(240, Math.min(clampWidth * 0.95, clampWidth))
          : clampWidth;

      setBoardWidth((current) => {
        const rounded = Math.round(responsiveWidth);
        return Math.abs(current - rounded) > 1 ? rounded : current;
      });
    },
    [width],
  );

  useEffect(() => {
    setMounted(true);
  }, []);

  useEffect(() => {
    if (!mounted || width) {
      if (width) {
        setBoardWidth(width);
      }
      return;
    }

    const parent = boardRef.current?.parentElement;
    if (!parent) return;

    updateBoardSize(parent.clientWidth);

    const resizeObserver =
      typeof ResizeObserver !== "undefined"
        ? new ResizeObserver((entries) => {
            for (const entry of entries) {
              updateBoardSize(entry.contentRect.width);
            }
          })
        : null;

    if (resizeObserver) {
      resizeObserver.observe(parent);
    }

    const onWindowResize = () => updateBoardSize();
    const onOrientationChange = () => updateBoardSize();

    window.addEventListener("resize", onWindowResize);
    window.addEventListener("orientationchange", onOrientationChange);

    return () => {
      window.removeEventListener("resize", onWindowResize);
      window.removeEventListener("orientationchange", onOrientationChange);
      if (resizeObserver) {
        resizeObserver.disconnect();
      }
    };
  }, [mounted, width, updateBoardSize]);

  const pieceImages: Record<string, string> = useMemo(
    () => ({
      wP: WhitePawn,
      wR: WhiteRook,
      wN: WhiteKnight,
      wB: WhiteBishop,
      wQ: WhiteQueen,
      wK: WhiteKing,
      bP: BlackPawn,
      bR: BlackRook,
      bN: BlackKnight,
      bB: BlackBishop,
      bQ: BlackQueen,
      bK: BlackKing,
    }),
    [],
  );

  const getPieceImage = useCallback(
    (piece: string) => {
      if (!piece) return null;
      const isWhite = piece.startsWith("w");
      return (
        <div
          className="piece-container group"
          style={{
            width: "100%",
            height: "100%",
            display: "flex",
            justifyContent: "center",
            alignItems: "center",
            position: "relative",
            userSelect: "none",
            cursor: "grab",
            transform: `scale(${boardWidth < 400 ? 0.7 : 0.9})`,
            transition: "all 0.2s ease",
          }}
        >
          <div
            style={{
              width: boardWidth < 400 ? "80%" : "90%",
              height: boardWidth < 400 ? "80%" : "90%",
              position: "relative",
              transform: "scale(1)",
              transition: "transform 0.2s ease",
              aspectRatio: "1/1",
              minHeight: "40px",
            }}
            className="group-hover:transform group-hover:scale-110"
          >
            <Image
              src={pieceImages[piece]}
              alt={piece}
              fill
              priority
              sizes="(max-width: 400px) 80vw, 90vw"
              style={{
                width: "100%",
                height: "100%",
                objectFit: "contain",
                filter: isWhite
                  ? "drop-shadow(2px 2px 2px rgba(0,0,0,0.5))"
                  : "drop-shadow(2px 2px 2px rgba(0,0,0,0.3))",
                transition: "filter 0.2s ease",
              }}
              className="group-hover:filter group-hover:brightness-110"
              onError={(e) => {
                console.error(`Failed to load chess piece: ${piece}`);
                const target = e.target;
                if (
                  target &&
                  typeof (target as Element).className !== "undefined"
                ) {
                  (target as HTMLElement).style.opacity = "0.5";
                }
              }}
            />
          </div>
        </div>
      );
    },
    [boardWidth, pieceImages],
  );

  const attemptMove = useCallback(
    (
      sourceRow: number,
      sourceCol: number,
      targetRow: number,
      targetCol: number,
    ): void => {
      const sourceSquare = `${String.fromCharCode(97 + sourceCol)}${
        8 - sourceRow
      }`;
      const targetSquare = `${String.fromCharCode(97 + targetCol)}${
        8 - targetRow
      }`;
      const moveSuccess = onDrop({ sourceSquare, targetSquare });
      if (moveSuccess) {
        setSelectedSquare(null);
      }
    },
    [onDrop],
  );

  const handleSquareClick = useCallback(
    (row: number, col: number) => {
      const [actualRow, actualCol] = translateDisplayToActual(row, col);
      const clickedSquare = `${actualRow},${actualCol}`;
      if (!selectedSquare && boardState[actualRow][actualCol]) {
        setSelectedSquare(clickedSquare);
        return;
      }
      if (selectedSquare === clickedSquare) {
        setSelectedSquare(null);
        return;
      }
      if (selectedSquare) {
        const [sourceRow, sourceCol] = selectedSquare.split(",").map(Number);
        attemptMove(sourceRow, sourceCol, actualRow, actualCol);
      }
    },
    [selectedSquare, boardState, attemptMove, translateDisplayToActual],
  );

  const handleDragStart = useCallback(
    (e: React.DragEvent, row: number, col: number) => {
      const [actualRow, actualCol] = translateDisplayToActual(row, col);
      e.dataTransfer.setData("text/plain", `${actualRow},${actualCol}`);
      const draggedElement = e.currentTarget as HTMLElement;
      if (draggedElement) {
        draggedElement.style.opacity = "0.6";
      }
    },
    [translateDisplayToActual],
  );

  const handleDragEnd = useCallback((e: React.DragEvent) => {
    const draggedElement = e.currentTarget as HTMLElement;
    if (draggedElement) {
      draggedElement.style.opacity = "1";
    }
  }, []);

  const handleDrop = useCallback(
    (e: React.DragEvent, row: number, col: number) => {
      e.preventDefault();
      const [targetRow, targetCol] = translateDisplayToActual(row, col);
      const data = e.dataTransfer.getData("text/plain");
      const [sourceRow, sourceCol] = data.split(",").map(Number);
      attemptMove(sourceRow, sourceCol, targetRow, targetCol);
    },
    [attemptMove, translateDisplayToActual],
  );

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
  }, []);

  const getSquareFromTouch = useCallback(
    (touch: React.Touch): [number, number] | null => {
      if (!boardRef.current) return null;
      const rect = boardRef.current.getBoundingClientRect();
      const x = touch.clientX - rect.left;
      const y = touch.clientY - rect.top;
      const col = Math.floor((x / rect.width) * 8);
      const row = Math.floor((y / rect.height) * 8);
      if (col < 0 || col > 7 || row < 0 || row > 7) return null;
      return [row, col];
    },
    [],
  );

  const handleTouchStart = useCallback(
    (e: React.TouchEvent, row: number, col: number) => {
      const [actualRow, actualCol] = translateDisplayToActual(row, col);
      if (!boardState[actualRow][actualCol]) return;
      touchStartSquare.current = `${actualRow},${actualCol}`;
      setSelectedSquare(`${actualRow},${actualCol}`);
    },
    [boardState, translateDisplayToActual],
  );

  const handleTouchMove = useCallback(
    (e: React.TouchEvent) => {
      if (!touchStartSquare.current) return;
      const sq = getSquareFromTouch(e.touches[0]);
      if (sq) {
        const [actualRow, actualCol] = translateDisplayToActual(sq[0], sq[1]);
        setHoveredSquare(`${actualRow},${actualCol}`);
      }
    },
    [getSquareFromTouch, translateDisplayToActual],
  );

  const handleTouchEnd = useCallback(
    (e: React.TouchEvent) => {
      if (!touchStartSquare.current) return;
      const sq = getSquareFromTouch(e.changedTouches[0]);
      if (sq) {
        const [srcRow, srcCol] = touchStartSquare.current.split(",").map(Number);
        const [targetRow, targetCol] = translateDisplayToActual(sq[0], sq[1]);
        attemptMove(srcRow, srcCol, targetRow, targetCol);
      }
      touchStartSquare.current = null;
      setHoveredSquare(null);
      setSelectedSquare(null);
    },
    [getSquareFromTouch, attemptMove, translateDisplayToActual],
  );

  if (!mounted) {
    return (
      <div className="w-full h-full flex items-center justify-center bg-gray-800 rounded-md">
        <div className="text-white">Initializing chessboard...</div>
      </div>
    );
  }
  return (
    <div
      ref={boardRef}
      className="chessboard-container w-full mx-auto relative"
      role="grid"
      aria-label="Chess Board"
      onTouchMove={handleTouchMove}
      onTouchEnd={handleTouchEnd}
      style={{
        width: "100%",
        maxWidth: `${boardWidth}px`,
        aspectRatio: "1/1",
        display: "grid",
        gridTemplateColumns: `repeat(8, minmax(0, 1fr))`,
        gridTemplateRows: `repeat(8, minmax(0, 1fr))`,
        border: "2px solid #005dad",
        borderRadius: "4px",
        boxShadow: "0 8px 16px rgba(0, 93, 173, 0.3)",
        overflow: "visible",
        touchAction: "none",
        margin: "0 auto",
        padding: "1%",
        transform: "scale(var(--board-scale, 1))",
        transformOrigin: "center center",
      }}
      aria-live="polite"
    >
      {boardState.map((row, rowIndex) =>
        row.map((_, colIndex) => {
          const [actualRow, actualCol] = translateDisplayToActual(
            rowIndex,
            colIndex,
          );
          const piece = boardState[actualRow][actualCol];
          const isLight = (rowIndex + colIndex) % 2 === 1;
          const squareKey = `${actualRow},${actualCol}`;
          const isSelected = selectedSquare === squareKey;
          const isHovered =
            hoveredSquare === squareKey && hoveredSquare !== selectedSquare;
          const squareLabel = `${getSquareNotation(actualRow, actualCol)}${
            piece ? " with " + piece : ""
          }`;
          return (
            <div
              key={`${rowIndex}-${colIndex}`}
              role="gridcell"
              aria-label={squareLabel}
              tabIndex={0}
              onKeyDown={(e) => {
                if (e.key === "Enter" || e.key === " ") {
                  handleSquareClick(rowIndex, colIndex);
                }
              }}
              style={{
                backgroundColor: isLight ? "#008e90" : "#ffffff",
                width: "100%",
                height: "100%",
                display: "flex",
                justifyContent: "center",
                alignItems: "center",
                cursor: piece ? "grab" : "default",
                position: "relative",
                boxShadow: isSelected
                  ? "inset 0 0 0 3px rgba(0, 93, 173, 0.75)"
                  : isHovered
                  ? "inset 0 0 0 3px rgba(0, 200, 170, 0.7)"
                  : "none",
                transition: "background-color 0.2s ease, box-shadow 0.1s ease",
              }}
              onClick={() => handleSquareClick(rowIndex, colIndex)}
              onTouchStart={(e) => handleTouchStart(e, rowIndex, colIndex)}
              draggable={!!piece}
              onDragStart={(e) => handleDragStart(e, rowIndex, colIndex)}
              onDragEnd={handleDragEnd}
              onDrop={(e) => handleDrop(e, rowIndex, colIndex)}
              onDragOver={handleDragOver}
            >
              {piece && (
                <div
                  style={{
                    transition: "transform 0.2s ease-out",
                    transform: `scale(${isSelected ? 1.1 : 1})`,
                  }}
                >
                  {getPieceImage(piece)}
                </div>
              )}
            </div>
          );
        }),
      )}
    </div>
  );
};

export default React.memo(ChessboardComponent);
