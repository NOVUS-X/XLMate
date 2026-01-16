"use client"

import React from 'react'
import Link from 'next/link'
import { MenuIcon, UserIcon } from './icons'
import { Button } from './ui/button'
import { useTheme } from 'next-themes'
import { Sheet, SheetContent, SheetTrigger } from './ui/sheet'
import { GameSidebar } from './GameSidebar'
import { MoonIcon, SunIcon } from 'lucide-react'

export function Header() {
    const { theme, setTheme } = useTheme()

    return (
        <header className="h-16 border-b border-border bg-card/80 backdrop-blur-sm fixed top-0 left-0 right-0 z-50">
            <div className="container h-full flex items-center justify-between">
                <div className="flex items-center gap-4">
                    <Sheet>
                        <SheetTrigger asChild>
                            <Button variant="ghost" size="icon" className="md:hidden">
                                <MenuIcon className="h-6 w-6" />
                            </Button>
                        </SheetTrigger>
                        <SheetContent side="left" className="p-0 w-64">
                            <GameSidebar />
                        </SheetContent>
                    </Sheet>

                    <Link href="/" className="flex items-center space-x-2">
                        <div className="relative h-8 w-8">
                            <div className="absolute inset-0 bg-blue-500 rounded-full blur-sm opacity-30 animate-pulse"></div>
                            <svg viewBox="0 0 32 32" fill="none" xmlns="http://www.w3.org/2000/svg" className="relative z-10">
                                <path d="M16 3L22 7L28 5L24 12L28 19L22 17L16 21L16 14L10 17L4 19L8 12L4 5L10 7L16 3Z" fill="#3AA0CD" />
                                <path d="M16 3L10 7L4 5L8 12L4 19L10 17L16 21L16 14L16 3Z" fill="#2C8CB5" />
                                <circle cx="16" cy="12" r="2" fill="white" />
                            </svg>
                        </div>
                        <span className="font-bold text-xl bg-clip-text text-transparent bg-gradient-to-r from-blue-400 to-blue-600">
                            XLMATE
                        </span>
                    </Link>
                </div>

                <div className="flex items-center space-x-2">
                    <Button variant="ghost" size="icon" onClick={() => setTheme(theme === 'dark' ? 'light' : 'dark')}>
                        {theme === 'dark' ? <SunIcon className="h-5 w-5" /> : <MoonIcon className="h-5 w-5" />}
                    </Button>

                    <Button variant="ghost" size="icon" className="md:hidden">
                        <UserIcon className="h-6 w-6" />
                    </Button>
                </div>
            </div>
        </header>
    )
}