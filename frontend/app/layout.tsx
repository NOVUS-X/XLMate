import type { Metadata } from "next";
import "./globals.css";
import ClientRoot from "@/components/ClientRoot";
import { AppProvider } from "@/context/walletContext";
import { MatchmakingProvider } from "@/context/matchmakingContext";
import "primereact/resources/themes/lara-light-cyan/theme.css";
// Provider import - wallet setup pending
// import { Providers } from "./provider";

export const metadata: Metadata = {
  title: "XLMate",
  description: "XLMate — Chess on Stellar",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body className="antialiased font-rowdies bg-background text-foreground">
        {/* <Provider> */}
          {/* <AppProvider> */}
            {/* <Providers> */}
            <AppProvider>
              <MatchmakingProvider>
                <ClientRoot>{children}</ClientRoot>
              </MatchmakingProvider>
            </AppProvider>
            {/* </Providers> */}
          {/* </AppProvider> */}
        {/* </Provider> */}
      </body>
    </html>
  );
}
