import React from "react";
import { render, screen } from "@testing-library/react";
import ChessboardComponent from "@/components/chess/ChessboardComponent";

vi.mock("next/image", () => ({
  __esModule: true,
  default: (props: { alt: string }) => React.createElement("img", {
    alt: props.alt,
  }),
}));

describe("ChessboardComponent", () => {
  it("renders a full 8x8 chessboard", () => {
    render(
      React.createElement(ChessboardComponent, {
        position: "start",
        onDrop: () => false,
      }),
    );

    const cells = screen.getAllByRole("gridcell");
    expect(cells).toHaveLength(64);
    expect(cells[0]).toHaveAttribute("aria-label", "a8 with bR");
    expect(cells[63]).toHaveAttribute("aria-label", "h1 with wR");
  });

  it("rotates the board when orientation is set to black", () => {
    render(
      React.createElement(ChessboardComponent, {
        position: "start",
        onDrop: () => false,
        orientation: "black",
      }),
    );

    const cells = screen.getAllByRole("gridcell");
    expect(cells[0]).toHaveAttribute("aria-label", "h1 with wR");
    expect(cells[63]).toHaveAttribute("aria-label", "a8 with bR");
  });
});
