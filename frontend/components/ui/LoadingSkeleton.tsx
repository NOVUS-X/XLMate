"use client";

import React from "react";

interface SkeletonProps {
  className?: string;
  count?: number;
}

/**
 * LoadingSkeleton — A shimmer-based skeleton loader.
 * Uses the shimmer-bg animation defined in globals.css for a premium feel.
 * CPU-efficient: pure CSS animation, no JS-driven animation frames.
 */
export function LoadingSkeleton({ className = "", count = 1 }: SkeletonProps) {
  return (
    <>
      {Array.from({ length: count }).map((_, i) => (
        <div
          key={i}
          className={`shimmer-bg rounded-lg ${className}`}
          aria-hidden="true"
        />
      ))}
    </>
  );
}

/**
 * BoardSkeleton — A chessboard-sized skeleton placeholder.
 */
export function BoardSkeleton() {
  return (
    <div className="w-full max-w-[560px] min-w-[320px] aspect-square rounded-md border-2 border-gray-700/50 p-1">
      <div className="grid grid-cols-8 grid-rows-8 gap-0 w-full h-full">
        {Array.from({ length: 64 }).map((_, i) => (
          <div
            key={i}
            className={`${
              (Math.floor(i / 8) + (i % 8)) % 2 === 0
                ? "bg-gray-700/30"
                : "bg-gray-600/20"
            } rounded-sm`}
          />
        ))}
      </div>
    </div>
  );
}
