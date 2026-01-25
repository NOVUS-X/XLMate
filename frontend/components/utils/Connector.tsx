"use client";
import React, { useState } from "react";
import { WalletConnectModal } from "../WalletConnectModal";

const Connector: React.FC = () => {
  const [isModalOpen, setIsModalOpen] = useState(false);

  return (
    <div className="p-6 text-center">
      <div>
        <button
          onClick={() => setIsModalOpen(true)}
          className="px-4 py-2 rounded-full bg-gradient-to-r from-teal-500 to-blue-700 hover:from-teal-600 hover:to-blue-800 text-white font-medium"
        >
          Connect Wallet
        </button>
        <WalletConnectModal 
          isOpen={isModalOpen} 
          onClose={() => setIsModalOpen(false)} 
        />
      </div>
    </div>
  );
};

export default Connector;
