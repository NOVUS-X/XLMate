"use client";

import type React from "react"
import Image from "next/image"
import Link from "next/link"
import { Button } from "@/components/ui/button"
import { Sheet, SheetContent, SheetTrigger } from "@/components/ui/sheet"
import { useState, useEffect } from "react"
import { WalletConnectModal } from "./WalletConnectModal"
import { SignInModal } from "./SignInModal"
import { useAppContext } from "@/context/walletContext"
import {
  ChessIcon,
  WatchIcon,
  NewsIcon,
  UserIcon,
  SettingsIcon,
  SupportIcon,
  WalletIcon,
  MenuIcon,
  CollapseIcon,
} from "@/components/icons"

interface SidebarProps {
  collapsed: boolean;
  setCollapsed: (collapsed: boolean) => void;
  isMobileView?: boolean;
}

export function GameSidebar({
  collapsed: propCollapsed,
  setCollapsed,
  isMobileView = false,
}: SidebarProps) {
  const [isHovered, setIsHovered] = useState(false);
  const [collapsed, setLocalCollapsed] = useState(propCollapsed);
  const [isWalletModalOpen, setIsWalletModalOpen] = useState(false);
  const [isSignInModalOpen, setIsSignInModalOpen] = useState(false);
  const { address, status } = useAppContext();

  useEffect(() => {
    setCollapsed(collapsed);
  }, [collapsed, setCollapsed]);
  
  useEffect(() => setLocalCollapsed(propCollapsed), [propCollapsed]);

  const truncateAddress = (addr: string) => {
    return `${addr.slice(0, 6)}...${addr.slice(-4)}`;
  };

  // For mobile view, we'll use a Sheet component
  if (isMobileView) {
    return (
      <>
        <Sheet>
          <SheetTrigger asChild>
            <Button variant="ghost" size="icon" className="md:hidden">
              <MenuIcon />
              <span className="sr-only">Toggle menu</span>
            </Button>
          </SheetTrigger>
          <SheetContent side="left" className="p-0 w-64 bg-gray-900 border-r border-gray-800">
            <MobileSidebar />
          </SheetContent>
        </Sheet>
        <WalletConnectModal isOpen={isWalletModalOpen} onClose={() => setIsWalletModalOpen(false)} />
        <SignInModal isOpen={isSignInModalOpen} onClose={() => setIsSignInModalOpen(false)} />
      </>
    );
  }

  return (
    <>
      <div
        className={`fixed left-0 top-0 h-full bg-gray-900/95 backdrop-blur-sm border-r border-gray-800/50 flex-col transition-all duration-500 ease-in-out hidden md:flex ${
          collapsed && !isHovered ? "w-16" : "w-64"
        } shadow-xl group z-[50]`}
        role="navigation"
        aria-label="Main Navigation"
        onMouseEnter={() => {
          setIsHovered(true);
          setLocalCollapsed(false);
        }}
        onMouseLeave={() => {
          setIsHovered(false);
          setLocalCollapsed(true);
        }}
      >
        <div className="p-4 flex items-center justify-center overflow-hidden">
          <div
            className={`transition-all duration-500 ease-in-out ${
              collapsed && !isHovered ? "w-16" : "w-full"
            }`}
          >
            <div className="w-16 h-16 relative transform hover:scale-105 transition-transform duration-300">
              <Image
                src="/images/XLMateLogo.png"
                alt="XLMate"
                fill
                className="object-contain drop-shadow-lg"
              />
            </div>
          </div>
        </div>

        <nav className="flex-1 px-2 overflow-hidden">
          <SidebarItem
            icon={<ChessIcon />}
            label="Play"
            href="/"
            collapsed={collapsed && !isHovered}
            active
          />
          <SidebarItem
            icon={<WatchIcon />}
            label="Watch"
            href="/watch"
            collapsed={collapsed && !isHovered}
          />
          <SidebarItem
            icon={<NewsIcon />}
            label="News"
            href="/news"
            collapsed={collapsed && !isHovered}
          />
          <SidebarItem
            icon={<UserIcon />}
            label="Profile"
            href="/profile"
            collapsed={collapsed && !isHovered}
          />
          <SidebarItem
            icon={<SettingsIcon />}
            label="Settings"
            href="/settings"
            collapsed={collapsed && !isHovered}
          />
          <SidebarItem
            icon={<SupportIcon />}
            label="Support"
            href="/support"
            collapsed={collapsed && !isHovered}
          />
        </nav>

        <div
          className={`p-4 space-y-3 overflow-hidden transition-all duration-500 ease-in-out ${
            collapsed && !isHovered ? "opacity-0" : "opacity-100"
          }`}
        >
          {status === "connected" && address ? (
            <div className="flex items-center space-x-3 p-2 rounded-lg bg-gray-800">
              <div className="flex-1 min-w-0">
                <p className="text-sm font-medium text-white truncate">
                  {truncateAddress(address)}
                </p>
              </div>
            </div>
          ) : (
            <Button 
              className="w-full bg-gradient-to-r from-teal-500 to-blue-700 hover:from-teal-600 hover:to-blue-800 text-white shadow-lg hover:shadow-teal-500/20 transition-all duration-300 rounded-lg"
              onClick={() => setIsWalletModalOpen(true)}
            >
              <div className="flex items-center">
                <div className="transform group-hover:scale-110 transition-transform duration-300">
                  <WalletIcon />
                </div>
                <span
                  className={`ml-2 transition-opacity duration-500 ${
                    collapsed && !isHovered ? "opacity-0" : "opacity-100"
                  }`}
                >
                  Connect Wallet
                </span>
              </div>
            </Button>
          )}
          <div
            className={`space-y-2 transition-all duration-500 ${
              collapsed && !isHovered ? "scale-0" : "scale-100"
            }`}
          >
            <Button 
              className="w-full bg-teal-600 hover:bg-teal-700 shadow-lg hover:shadow-teal-600/20 transition-all duration-300 rounded-lg"
              onClick={() => setIsSignInModalOpen(true)}
            >
              Sign Up
            </Button>
            <Button
              variant="outline"
              className="w-full border-gray-700 text-gray-300 hover:bg-gray-800/50 hover:border-teal-500/50 transition-all duration-300 rounded-lg"
              onClick={() => setIsSignInModalOpen(true)}
            >
              Log In
            </Button>
          </div>
        </div>

        <div className="border-t border-gray-800 p-2">
          <button
            onClick={() => setLocalCollapsed(!collapsed)}
            className="p-2 w-full flex items-center justify-center text-gray-400 hover:text-white hover:bg-gray-800 rounded-md transition-all duration-300 group"
          >
            <div className={`transform transition-transform duration-300 ${collapsed ? "rotate-180" : ""}`}>
              <CollapseIcon />
            </div>
            {!collapsed && (
              <span className="ml-2 transition-opacity duration-300 group-hover:text-teal-400">Collapse</span>
            )}
          </button>
        </div>
      </div>
      <WalletConnectModal isOpen={isWalletModalOpen} onClose={() => setIsWalletModalOpen(false)} />
      <SignInModal isOpen={isSignInModalOpen} onClose={() => setIsSignInModalOpen(false)} />
    </>
  )
}

interface SidebarItemProps {
  icon: React.ReactNode;
  label: string;
  href: string;
  collapsed?: boolean;
  active?: boolean;
}

function SidebarItem({
  icon,
  label,
  href,
  collapsed,
  active,
}: SidebarItemProps) {
  return (
    <Link
      href={href}
      className={`flex items-center gap-4 px-4 py-3 rounded-lg transition-all duration-300 ${
        active
          ? "bg-gray-800/50 text-white"
          : "text-gray-400 hover:text-white hover:bg-gray-800/30"
      }`}
    >
      <span
        className={`${
          active ? "text-teal-400" : "text-gray-300"
        } transition-all duration-300 group-hover:text-teal-400 transform group-hover:scale-110 min-w-[24px]`}
      >
        {icon}
      </span>
      <span
        className={`ml-3 text-sm font-medium text-gray-300 group-hover:text-white transition-all duration-500 ${
          collapsed ? "opacity-0 w-0" : "opacity-100 w-auto"
        } whitespace-nowrap`}
      >
        {label}
      </span>
    </Link>
  );
}

interface MobileSidebarProps {
  className?: string;
}

function MobileSidebar({ className = "" }: MobileSidebarProps) {
  return (
    <div className={`flex flex-col h-full bg-gray-900 ${className}`}>
      <div className="p-4 flex items-center space-x-2">
        <div className="w-16 h-16 relative">
          <Image
            src="/images/XLMateLogo.png"
            alt="XLMate"
            fill
            className="object-contain"
          />
        </div>
      </div>
      <nav className="flex-1">
        <MobileSidebarItem icon={<ChessIcon />} label="Play" href="/" active />
        <MobileSidebarItem icon={<WatchIcon />} label="Watch" href="/watch" />
        <MobileSidebarItem icon={<NewsIcon />} label="News" href="/news" />
        <MobileSidebarItem icon={<UserIcon />} label="Profile" href="/profile" />
        <MobileSidebarItem icon={<SettingsIcon />} label="Settings" href="/settings" />
        <MobileSidebarItem icon={<SupportIcon />} label="Support" href="/support" />
      </nav>
      <div className="p-4 space-y-2">
        <Button className="w-full bg-gradient-to-r from-teal-500 to-blue-700 hover:from-teal-600 hover:to-blue-800 text-white">
          <div className="flex items-center">
            <WalletIcon />
            <span className="ml-2">Connect Wallet</span>
          </div>
        </Button>
        <Button className="w-full bg-teal-600 hover:bg-teal-700">Sign Up</Button>
        <Button variant="outline" className="w-full border-gray-700 text-gray-300">
          Log In
        </Button>
      </div>
    </div>
  );
}

function MobileSidebarItem({ icon, label, href, active }: SidebarItemProps) {
  return (
    <Link
      href={href}
      className={`flex items-center p-3 px-4 hover:bg-gray-800/50 transition-all duration-300 rounded-lg mb-1 group ${
        active ? "bg-gray-800/50 shadow-lg" : ""
      }`}
    >
      <span
        className={`${
          active ? "text-teal-400" : "text-gray-300"
        } transition-all duration-300 group-hover:text-teal-400 transform group-hover:scale-110`}
      >
        {icon}
      </span>
      <span className="ml-3 text-sm font-medium text-gray-300 group-hover:text-white transition-colors duration-300">
        {label}
      </span>
    </Link>
  );
}
