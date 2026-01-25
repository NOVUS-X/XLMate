// "use client";
// import { createContext, useContext } from "react";
// import { Toast } from "primereact/toast";
// import { useRef } from "react";
// import { useRouter } from "next/navigation";
// // import {
// //   useConnect,
// //   useDisconnect,
// //   useAccount,
// //   useBalance,
// } from blockchain provider;
// // import type { Connector } from blockchain provider;
// import DotPulseLoader from "../components/ui/DotPulseLoader";
// import { STRK_TOKEN_ADDRESS } from "@/constants/tokens";

// interface AppContextType {
//   showToast: (
//     severity: "success" | "error" | "info",
//     summary: string,
//     detail: string
//   ) => void;
//   connectWallet: (connector: Connector) => Promise<void>;
//   disconnectWallet: () => Promise<void>;
//   address?: string;
//   status: string;
//   balance?: string | React.ReactNode;
//   contactAddress?: string;
// }

// const AppContext = createContext<AppContextType | undefined>(undefined);

// export function AppProvider({ children }: { children: React.ReactNode }) {
//   const router = useRouter();
//   const toast = useRef<Toast>(null);
//   const { connectAsync } = useConnect();
//   const { disconnectAsync } = useDisconnect();
//   const { address, status } = useAccount();
//   const { data, isLoading } = useBalance({
//     token: STRK_TOKEN_ADDRESS,
//     address: address as "0x",
//   });

//   const balance =
//     isLoading || !data ? (
//       <DotPulseLoader />
//     ) : (
//       `${parseFloat(data.formatted).toFixed(2)} STRK`
//     );

//   const showToast = (
//     severity: "success" | "error" | "info",
//     summary: string,
//     detail: string
//   ) => {
//     toast.current?.show({ severity, summary, detail });
//   };

//   const connectWallet = async (connector: Connector) => {
//     try {
//       await connectAsync({ connector });
//       localStorage.setItem("connector", connector.id);
//       showToast("success", "Success", "Wallet connected successfully");
//     } catch (error: unknown) {
//       localStorage.removeItem("connector");
//       let errorMessage = "Failed to connect wallet.";
//       if (error instanceof Error) {
//         if (error.message.includes("rejected")) {
//           errorMessage =
//             "Connection rejected. Please approve the connection request.";
//         } else if (error.message.includes("Connector not found")) {
//           errorMessage = `${connector.name} is not installed.`;
//         } else {
//           errorMessage = "Connection Failed";
//         }
//       }
//       showToast("error", "Connection Failed", errorMessage);
//     }
//   };

//   const disconnectWallet = async () => {
//     try {
//       await disconnectAsync();
//       localStorage.removeItem("connector");
//       showToast("success", "Success", "Wallet disconnected successfully");
//       setTimeout(() => {
//         router.push("/");
//       }, 1000);
//     } catch (error) {
//       console.log(error);
//       showToast("error", "Error", "Failed to disconnect wallet");
//     }
//   };

//   return (
//     <AppContext.Provider
//       value={{
//         showToast,
//         connectWallet,
//         disconnectWallet,

//         address,
//         status,
//         balance,
//       }}
//     >
//       <Toast ref={toast} />
//       {children}
//     </AppContext.Provider>
//   );
// }

// export const useAppContext = () => {
//   const context = useContext(AppContext);
//   if (!context) {
//     throw new Error("useAppContext must be used within an AppProvider");
//   }
//   return context;
// };

// Placeholder implementation while wallet context is being developed
export const useAppContext = () => {
  return {
    address: undefined,
    status: "disconnected",
    balance: undefined,
  };
};
