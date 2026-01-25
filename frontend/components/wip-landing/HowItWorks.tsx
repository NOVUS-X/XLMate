import React from "react";
import { Wallet, LayoutGrid, Users, Trophy } from "lucide-react";

const HowItWorks = () => {
  const steps = [
    {
      id: 1,
      title: "Connect Wallet",
      description:
        "Securely link your wallet to access decentralized gaming and manage your assets.",
      icon: <Wallet className="w-16 h-16 text-indigo-500" />,
    },
    {
      id: 2,
      title: "Choose Game Mode",
      description:
        "Select from various game modes optimized for blockchain interactions and Web3 experiences.",
      icon: <LayoutGrid className="w-16 h-16 text-purple-500" />,
    },
    {
      id: 3,
      title: "Match with Opponent",
      description:
        "Our decentralized matchmaking system pairs you with players of similar skill levels.",
      icon: <Users className="w-16 h-16 text-blue-500" />,
    },
    {
      id: 4,
      title: "Play and Earn",
      description:
        "Compete in games, complete objectives, and earn rewards stored directly on the blockchain.",
      icon: <Trophy className="w-16 h-16 text-amber-500" />,
    },
  ];

  return (
    <section className="py-20 bg-gradient-to-br from-gray-900 via-gray-800 to-gray-900 relative overflow-hidden">
      {/* Decorative elements */}
      <div className="absolute inset-0 opacity-10">
        <div className="absolute top-10 left-10 w-64 h-64 rounded-full bg-indigo-500 blur-3xl"></div>
        <div className="absolute bottom-10 right-10 w-64 h-64 rounded-full bg-blue-500 blur-3xl"></div>
        <div className="absolute top-1/2 left-1/2 w-96 h-96 -translate-x-1/2 -translate-y-1/2 rounded-full bg-purple-500 blur-3xl"></div>
      </div>

      {/* Network nodes decoration */}
      <div className="absolute inset-0 opacity-20">
        {Array.from({ length: 20 }).map((_, i) => (
          <div
            key={i}
            className="absolute h-2 w-2 bg-white rounded-full"
            style={{
              top: `${Math.random() * 100}%`,
              left: `${Math.random() * 100}%`,
              boxShadow: "0 0 10px 2px rgba(255, 255, 255, 0.8)",
            }}
          ></div>
        ))}
      </div>

      <div className="container mx-auto px-4 relative z-10">
        <div className="text-center mb-16">
          <h2 className="text-4xl md:text-5xl font-bold mb-4 text-transparent bg-clip-text bg-gradient-to-r from-indigo-400 via-purple-400 to-blue-400">
            How It Works
          </h2>
          <div className="h-1 w-24 bg-gradient-to-r from-indigo-500 to-purple-500 mx-auto mb-6"></div>
          <p className="text-gray-300 max-w-2xl mx-auto text-lg">
            Join the future of gaming with our simple step-by-step
            process
          </p>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-8">
          {steps.map((step, index) => (
            <div key={step.id} className="relative">
              <div className="bg-gray-800 bg-opacity-70 backdrop-blur-lg rounded-xl p-6 border border-gray-700 h-full transform transition-all duration-300 hover:scale-105 hover:border-indigo-500 group">
                <div className="absolute -top-6 left-1/2 transform -translate-x-1/2 bg-gray-900 rounded-full p-4 border-2 border-gray-700 group-hover:border-indigo-500 transition-all duration-300">
                  {step.icon}
                </div>

                {index < steps.length - 1 && (
                  <div className="hidden lg:block absolute top-1/2 left-full transform -translate-y-1/2 translate-x-4 w-8 h-0.5 bg-gradient-to-r from-indigo-500 to-purple-500 z-10"></div>
                )}

                <div className="mt-12 text-center">
                  <h3 className="text-2xl font-bold mb-3 text-white group-hover:text-indigo-400 transition-colors duration-300">
                    {step.title}
                  </h3>
                  <p className="text-gray-400 group-hover:text-gray-300 transition-colors duration-300">
                    {step.description}
                  </p>
                </div>

                <div className="absolute -bottom-3 left-1/2 transform -translate-x-1/2 bg-gray-900 rounded-full h-6 w-6 flex items-center justify-center text-indigo-500 font-bold border border-gray-700">
                  {step.id}
                </div>
              </div>
            </div>
          ))}
        </div>

        {/* Blockchain-themed button */}
        <div className="text-center mt-16">
          <button className="bg-gradient-to-r from-indigo-600 to-purple-600 hover:from-indigo-700 hover:to-purple-700 text-white font-bold py-3 px-8 rounded-full transition-all duration-300 hover:shadow-lg hover:shadow-indigo-500/30 transform hover:-translate-y-1">
            Start Playing Now
          </button>
        </div>
      </div>
    </section>
  );
};

export default HowItWorks;
