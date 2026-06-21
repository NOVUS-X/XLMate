import React from "react";
import { fireEvent, render, screen } from "@testing-library/react";
import { AiPersonalityModal } from "@/app/components/matchmaking/AiPersonalityModal";

const setAiPersonality = vi.fn();

vi.mock("@/context/matchmakingContext", () => ({
  useMatchmakingContext: () => ({
    aiPersonality: "defensive",
    setAiPersonality,
    chessVariant: "standard",
  }),
}));

vi.mock("@/lib/chessVariants", () => ({
  getChessVariantById: () => ({
    label: "Standard",
    description: "Classic chess rules.",
    averageGameTime: "10 min",
  }),
}));

describe("AiPersonalityModal accessibility", () => {
  beforeEach(() => {
    setAiPersonality.mockClear();
  });

  it("renders a labelled modal dialog with descriptive copy", () => {
    render(
      <AiPersonalityModal
        isOpen
        onClose={() => undefined}
        onConfirm={() => undefined}
      />,
    );

    const dialog = screen.getByRole("dialog", { name: "Finalize Match Setup" });

    expect(dialog).toHaveAttribute("aria-modal", "true");
    expect(dialog).toHaveAccessibleDescription(
      "Lock in your preferred chess format and AI co-pilot before matchmaking starts.",
    );
  });

  it("exposes personality choices as pressed buttons", () => {
    render(
      <AiPersonalityModal
        isOpen
        onClose={() => undefined}
        onConfirm={() => undefined}
      />,
    );

    expect(screen.getByRole("button", { name: /^select defensive/i })).toHaveAttribute(
      "aria-pressed",
      "true",
    );
    expect(screen.getByRole("button", { name: /^select aggressive/i })).toHaveAttribute(
      "aria-pressed",
      "false",
    );

    fireEvent.click(screen.getByRole("button", { name: /^select sacrificial/i }));

    expect(setAiPersonality).toHaveBeenCalledWith("sacrificial");
  });

  it("closes with the Escape key", () => {
    const onClose = vi.fn();

    render(
      <AiPersonalityModal
        isOpen
        onClose={onClose}
        onConfirm={() => undefined}
      />,
    );

    fireEvent.keyDown(window, { key: "Escape" });

    expect(onClose).toHaveBeenCalledTimes(1);
  });
});
