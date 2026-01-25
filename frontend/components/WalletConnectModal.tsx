// import { Button } from "@/components/ui/button";
// import { useAppContext } from "@/context/walletContext";
// Wallet connection modal - implementation pending
// import { useState } from "react";

// interface WalletConnectModalProps {
//   isOpen: boolean;
//   onClose: () => void;
// }

// export function WalletConnectModal({
//   isOpen,
//   onClose,
// }: WalletConnectModalProps) {
//   const { connectors } = useConnect();
//   const { connectWallet } = useAppContext();
//   const [isConnecting, setIsConnecting] = useState<string | null>(null);
//   if (!isOpen) return null;

//   return (
//     <div className="fixed inset-0 z-50 flex items-center justify-center">
//       <div className="relative bg-gray-900 rounded-lg border border-gray-800 p-6 w-full max-w-md mx-4">
//         <button
//           onClick={onClose}
//           className="absolute top-4 right-4 text-gray-400 hover:text-white"
//           aria-label="Close modal"
//         >
//           <svg
//             xmlns="http://www.w3.org/2000/svg"
//             width="24"
//             height="24"
//             viewBox="0 0 24 24"
//             fill="none"
//             stroke="currentColor"
//             strokeWidth="2"
//             strokeLinecap="round"
//             strokeLinejoin="round"
//           >
//             <line x1="18" y1="6" x2="6" y2="18"></line>
//             <line x1="6" y1="6" x2="18" y2="18"></line>
//           </svg>
//         </button>
//         <div className="space-y-4">
//           <div className="text-center">
//             <h2 className="text-xl font-bold text-white">Connect Wallet</h2>
//             <p className="text-gray-400 mt-2">
//               Choose your preferred wallet to connect to StarkMate
//             </p>
//           </div>
//           <div className="space-y-3">
//             {connectors.map((connector) => {
//               return (
//                 <Button
//                   key={connector.id}
//                   disabled={isConnecting === connector.id}
//                   className="w-full bg-gradient-to-r from-teal-500 to-blue-700 hover:from-teal-600 hover:to-blue-800"
//                   onClick={() => {
//                     setIsConnecting(connector.id);
//                     connectWallet(connector)
//                       .then(() => onClose())
//                       .catch(() => setIsConnecting(null))
//                       .finally(() => setIsConnecting(null));
//                   }}
//                 >
//                   {isConnecting === connector.id ? (
//                     <span className="flex items-center">
//                       <svg
//                         className="animate-spin -ml-1 mr-3 h-5 w-5 text-white"
//                         xmlns="http://www.w3.org/2000/svg"
//                         fill="none"
//                         viewBox="0 0 24 24"
//                       >
//                         <circle
//                           className="opacity-25"
//                           cx="12"
//                           cy="12"
//                           r="10"
//                           stroke="currentColor"
//                           strokeWidth="4"
//                         ></circle>
//                         <path
//                           className="opacity-75"
//                           fill="currentColor"
//                           d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
//                         ></path>
//                       </svg>
//                       Connecting...
//                     </span>
//                   ) : (
//                     connector.id.charAt(0).toUpperCase() + connector.id.slice(1)
//                   )}
//                 </Button>
//               );
//             })}
//           </div>
//         </div>
//       </div>
//     </div>
//   );
// }

// Placeholder export while WalletConnectModal is being implemented
export function WalletConnectModal({ isOpen, onClose }: { isOpen: boolean; onClose: () => void }) {
  if (!isOpen) return null;
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div className="relative bg-gray-900 rounded-lg border border-gray-800 p-6 w-full max-w-md mx-4">
        <button
          onClick={onClose}
          className="absolute top-4 right-4 text-gray-400 hover:text-white"
          aria-label="Close modal"
        >
          ✕
        </button>
        <h2 className="text-xl font-semibold text-white mb-4">Connect Wallet</h2>
        <p className="text-gray-400 mb-6">Wallet connection coming soon</p>
      </div>
    </div>
  );
}
