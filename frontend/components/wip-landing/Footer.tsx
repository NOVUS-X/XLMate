"use client";

import Link from "next/link";
import { FC } from "react";
import { Github, Twitter, Send, ArrowUp } from "lucide-react";

const Footer: FC = () => {
  const currentYear: number = new Date().getFullYear();

  const scrollToHero = () => {
    window.scrollTo({
      top: 0,
      behavior: "smooth",
    });
  };
  return (
    <footer
      id="footer"
      className="bg-gray-900 border-t border-cyan-500/20 pt-12 pb-6 relative"
    >
      {/* Glow effect */}
      <div className="absolute inset-0 pointer-events-none overflow-hidden">
        <div className="absolute -bottom-24 left-1/4 w-1/2 h-24 bg-cyan-500/20 blur-3xl rounded-full"></div>
      </div>

      <div className="container mx-auto px-4 relative">
        {/* Main footer content */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-8 mb-12">
          {/* About section */}
          <div className="space-y-4">
            <div className="flex items-center space-x-2">
              <div className="relative w-8 h-8 overflow-hidden">
                <div className="absolute inset-0 bg-gradient-to-br from-cyan-500 to-purple-500 rounded-md"></div>
                <div className="absolute inset-1 bg-gray-900 rounded-sm flex items-center justify-center">
                  <span className="text-cyan-500 font-bold text-lg">C</span>
                </div>
                <div className="absolute top-0 left-0 w-full h-full opacity-50 bg-gradient-to-t from-transparent to-white/30"></div>
              </div>
              <span className="text-white font-bold text-xl tracking-tight">
                XL<span className="text-cyan-500">Mate</span>
              </span>
            </div>
            <p className="text-gray-400 text-sm">
              The premier platform for competitive gaming on Stellar. Play,
              compete, and earn in a secure and decentralized environment.
            </p>
          </div>

          {/* Core Features */}
          <div className="space-y-4">
            <h3 className="text-white font-bold text-lg relative inline-block">
              Core Features
              <span className="absolute -bottom-1 left-0 w-12 h-0.5 bg-gradient-to-r from-cyan-500 to-purple-500"></span>
            </h3>
            <ul className="space-y-2">
              <li>
                <Link
                  href="/play"
                  aria-label="Competitive Gaming"
                  className="text-gray-400 hover:text-cyan-400 text-sm transition-colors"
                >
                  Competitive Gaming
                </Link>
              </li>
              <li>
                <Link
                  href="/nft-gallery"
                  aria-label="NFT Rewards"
                  className="text-gray-400 hover:text-cyan-400 text-sm transition-colors"
                >
                  NFT Rewards
                </Link>
              </li>
              <li>
                <Link
                  href="/leaderboard"
                  aria-label="Global Leaderboards"
                  className="text-gray-400 hover:text-cyan-400 text-sm transition-colors"
                >
                  Global Leaderboards
                </Link>
              </li>
              <li>
                <Link
                  href="/dao"
                  aria-label="DAO Governance"
                  className="text-gray-400 hover:text-cyan-400 text-sm transition-colors"
                >
                  DAO Governance
                </Link>
              </li>
            </ul>
          </div>

          {/* How It Works */}
          <div className="space-y-4">
            <h3 className="text-white font-bold text-lg relative inline-block">
              How It Works
              <span className="absolute -bottom-1 left-0 w-12 h-0.5 bg-gradient-to-r from-cyan-500 to-purple-500"></span>
            </h3>
            <ol className="space-y-2">
              <li className="text-gray-400 text-sm flex items-start">
                <span className="inline-flex items-center justify-center w-5 h-5 rounded-full bg-gradient-to-r from-cyan-500 to-purple-500 text-xs text-white mr-2 flex-shrink-0 mt-0.5">
                  1
                </span>
                <span>Connect your wallet to access the platform</span>
              </li>
              <li className="text-gray-400 text-sm flex items-start">
                <span className="inline-flex items-center justify-center w-5 h-5 rounded-full bg-gradient-to-r from-cyan-500 to-purple-500 text-xs text-white mr-2 flex-shrink-0 mt-0.5">
                  2
                </span>
                <span>Choose your game mode and find opponents</span>
              </li>
              <li className="text-gray-400 text-sm flex items-start">
                <span className="inline-flex items-center justify-center w-5 h-5 rounded-full bg-gradient-to-r from-cyan-500 to-purple-500 text-xs text-white mr-2 flex-shrink-0 mt-0.5">
                  3
                </span>
                <span>Win matches to climb the leaderboard</span>
              </li>
              <li className="text-gray-400 text-sm flex items-start">
                <span className="inline-flex items-center justify-center w-5 h-5 rounded-full bg-gradient-to-r from-cyan-500 to-purple-500 text-xs text-white mr-2 flex-shrink-0 mt-0.5">
                  4
                </span>
                <span>Earn rewards and exclusive NFTs</span>
              </li>
            </ol>
          </div>

          {/* Contact */}
          <div className="space-y-4">
            <h3 className="text-white font-bold text-lg relative inline-block">
              Connect With Us
              <span className="absolute -bottom-1 left-0 w-12 h-0.5 bg-gradient-to-r from-cyan-500 to-purple-500"></span>
            </h3>
            <div className="flex space-x-4">
              <Link
                href="https://github.com/xlmate"
                target="_blank"
                rel="noopener noreferrer"
                aria-label="GitHub"
                className="w-10 h-10 rounded-full bg-gray-800 hover:bg-gray-700 flex items-center justify-center group transition-all duration-300 hover:shadow-lg hover:shadow-cyan-500/20"
              >
                <Github className="w-5 h-5 text-gray-400 group-hover:text-cyan-400 transition-colors" />
              </Link>
              <Link
                href="https://t.me/xlmate"
                target="_blank"
                rel="noopener noreferrer"
                aria-label="Telegram"
                className="w-10 h-10 rounded-full bg-gray-800 hover:bg-gray-700 flex items-center justify-center group transition-all duration-300 hover:shadow-lg hover:shadow-cyan-500/20"
              >
                <Send className="w-5 h-5 text-gray-400 group-hover:text-cyan-400 transition-colors" />
              </Link>
              <Link
                href="https://twitter.com/xlmate"
                target="_blank"
                rel="noopener noreferrer"
                aria-label="Twitter"
                className="w-10 h-10 rounded-full bg-gray-800 hover:bg-gray-700 flex items-center justify-center group transition-all duration-300 hover:shadow-lg hover:shadow-cyan-500/20"
              >
                <Twitter className="w-5 h-5 text-gray-400 group-hover:text-cyan-400 transition-colors" />
              </Link>
            </div>
            <div className="pt-2">
              <p className="text-gray-400 text-sm">
                Join our community to stay updated on the latest tournaments and
                features.
              </p>
            </div>
          </div>
        </div>

        {/* Newsletter */}
        <div className="w-full max-w-3xl mx-auto mb-12">
          <div className="p-6 rounded-xl bg-gray-800/50 border border-gray-700 backdrop-blur-sm relative overflow-hidden">
            {/* Background glow */}
            <div className="absolute -inset-1 bg-gradient-to-r from-cyan-500/10 to-purple-500/10 blur-xl"></div>
            <div className="relative">
              <div className="text-center mb-4">
                <h3 className="text-white font-bold text-lg">
                  Stay in the loop
                </h3>
                <p className="text-gray-400 text-sm">
                  Subscribe to our newsletter for updates on tournaments and new
                  features
                </p>
              </div>
              <div className="flex flex-col sm:flex-row gap-2">
                <input
                  type="email"
                  placeholder="Enter your email"
                  className="flex-1 bg-gray-900 border border-gray-700 rounded-lg px-4 py-2 text-white focus:outline-none focus:ring-2 focus:ring-cyan-500/50"
                />
                <button className="bg-gradient-to-r from-cyan-500 to-purple-600 hover:from-cyan-600 hover:to-purple-700 text-white font-medium py-2 px-6 rounded-lg transition-all duration-300 hover:shadow-lg hover:shadow-cyan-500/20">
                  Subscribe
                </button>
              </div>
            </div>
          </div>
        </div>

        {/* Bottom bar */}
        <div className="border-t border-gray-800 pt-6 flex flex-col md:flex-row justify-between items-center">
          <p className="text-gray-500 text-sm mb-4 md:mb-0">
            &copy; {currentYear} XLMate. All rights reserved.
          </p>
          <div className="flex space-x-6">
            <Link
              href="/terms"
              aria-label="Terms of Service"
              className="text-gray-500 hover:text-gray-300 text-sm transition-colors"
            >
              Terms of Service
            </Link>
            <Link
              href="/privacy"
              aria-label="Privacy Policy"
              className="text-gray-500 hover:text-gray-300 text-sm transition-colors"
            >
              Privacy Policy
            </Link>
            <Link
              href="/faq"
              aria-label="FAQ"
              className="text-gray-500 hover:text-gray-300 text-sm transition-colors"
            >
              FAQ
            </Link>
          </div>
        </div>
        {/* Scroll to top arrow button - positioned at bottom-right */}
        <div
          onClick={scrollToHero}
          className="absolute bottom-8 right-8 w-12 h-12 rounded-full bg-gradient-to-r from-cyan-500 to-purple-600 flex items-center justify-center cursor-pointer shadow-lg hover:shadow-cyan-500/50 transition-all duration-300 group"
          aria-label="Scroll to top"
        >
          <ArrowUp className="animate-bounce w-5 h-5 text-white group-hover:scale-110 transition-transform" />
        </div>
      </div>
    </footer>
  );
};

export default Footer;
