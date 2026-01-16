"use client";
import React, { useState } from "react";

const WaitlistSection = () => {
  const [email, setEmail] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [isSubmitted, setIsSubmitted] = useState(false);
  const [error, setError] = useState("");

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    
    // Trim the email to remove any accidental spaces
    const trimmedEmail = email.trim();
    
    if (!trimmedEmail) {
      setError("We need your email to keep you updated");
      return;
    }
    
    // Check for common email mistakes
    if (!trimmedEmail.includes('@')) {
      setError("Don't forget the @ symbol in your email");
      return;
    }
    
    if (trimmedEmail.includes('@') && !trimmedEmail.includes('.')) {
      setError("Your email needs a domain (like .com or .io)");
      return;
    }
    
    if (!/^\S+@\S+\.\S+$/.test(trimmedEmail)) {
      setError("That email format doesn't look quite right");
      return;
    }
    
    setError("");
    setIsSubmitting(true);
    
    // Here you would typically send the email to your backend
    // This is a mock implementation
    try {
      // Simulate API call
      await new Promise(resolve => setTimeout(resolve, 1500));
      setIsSubmitted(true);
      // Store email in localStorage to remember the user signed up
      localStorage.setItem('waitlistEmail', trimmedEmail);
    } catch (err) {
      setError("Connection issue. Please try again.");
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div className="relative py-20 overflow-hidden bg-gradient-to-br from-gray-900 via-indigo-950 to-purple-900">
      {/* Blockchain grid background - similar to HeroSection for consistency */}
      <div className="absolute inset-0 z-0">
        <div className="grid grid-cols-12 grid-rows-12 h-full w-full opacity-20">
          {Array(144)
            .fill(0)
            .map((_, i) => (
              <div
                key={i}
                className={`border border-cyan-500/30 ${
                  Math.random() > 0.92 ? "bg-cyan-500/20" : ""
                }`}
              />
            ))}
        </div>
      </div>

      {/* Content container */}
      <div className="relative z-10 container mx-auto px-4">
        <div className="max-w-4xl mx-auto text-center">
          <h2 className="text-3xl md:text-5xl font-bold mb-6 animate-fadeIn">
            <span className="relative inline-block group">
              <span className="text-transparent bg-clip-text bg-gradient-to-r from-cyan-400 to-purple-400 transition-all duration-300 group-hover:scale-105">
                Join the Waitlist
              </span>
              <span className="absolute -inset-x-2 -inset-y-1 bg-gradient-to-r from-cyan-400/10 to-purple-400/10 blur-md opacity-30 -z-10 rounded-lg transition-all duration-300 group-hover:opacity-50 group-hover:blur-lg"></span>
            </span>
          </h2>
          
          <p className="text-gray-300 text-lg mb-8 max-w-2xl mx-auto">
            Be among the first to experience XLMate when we launch. 
            Get early access, exclusive NFTs, and updates on our progress.
          </p>

          {!isSubmitted ? (
            <form onSubmit={handleSubmit} className="max-w-md mx-auto">
              <div className="flex flex-col sm:flex-row gap-4">
                <input
                  value={email}
                  onChange={(e) => {
                    setEmail(e.target.value);
                    // Clear error as user types
                    if (error) setError("");
                  }}
                  onBlur={() => {
                    // Validate on blur for better UX
                    if (email && !email.includes('@')) {
                      setError("Don't forget the @ symbol");
                    }
                  }}
                  placeholder="Enter your email address"
                  className={`flex-grow px-4 py-3 rounded-full bg-gray-800/50 border ${error ? 'border-red-500/50' : 'border-cyan-500/30'} text-white placeholder-gray-400 focus:outline-none focus:ring-2 ${error ? 'focus:ring-red-500/50' : 'focus:ring-cyan-500/50'} transition-all duration-300`}
                  disabled={isSubmitting}
                  aria-label="Email address for waitlist"
                  autoComplete="email"
                />
                <button
                  type="submit"
                  disabled={isSubmitting}
                  className="px-6 py-3 rounded-full bg-gradient-to-r from-cyan-500 to-blue-600 text-white font-semibold hover:from-cyan-600 hover:to-blue-700 transition-all duration-300 shadow-lg hover:shadow-cyan-500/50 disabled:opacity-70 disabled:cursor-not-allowed"
                  aria-label="Submit email to join waitlist"
                >
                  {isSubmitting ? (
                    <span className="flex items-center justify-center">
                      <svg className="animate-spin -ml-1 mr-2 h-4 w-4 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                        <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                        <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                      </svg>
                      Joining...
                    </span>
                  ) : (
                    "Join Waitlist"
                  )}
                </button>
              </div>
              {error && (
                <div className="mt-3 flex items-center p-2 bg-red-500/20 border border-red-500/30 rounded-lg animate-pulse-once">
                  <svg className="w-5 h-5 text-red-400 mr-2 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
                  </svg>
                  <p className="text-red-400 text-sm">{error}</p>
                </div>
              )}
            </form>
          ) : (
            <div className="bg-gray-800/50 border border-cyan-500/30 rounded-xl p-6 max-w-md mx-auto animate-pulse-once">
              <div className="flex items-center justify-center mb-4">
                <div className="rounded-full bg-gradient-to-r from-green-500/30 to-cyan-500/30 p-3 shadow-lg shadow-cyan-500/20">
                  <svg className="w-8 h-8 text-cyan-400" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M5 13l4 4L19 7"></path>
                  </svg>
                </div>
              </div>
              <h3 className="text-2xl font-bold text-transparent bg-clip-text bg-gradient-to-r from-cyan-400 to-purple-400 mb-3">Welcome to XLMate!</h3>
              <p className="text-gray-300 mb-4">
                You're now part of our exclusive community. We'll send you updates about early access and special NFT drops.
              </p>
              <div className="flex justify-center">
                <div className="inline-flex items-center px-4 py-2 rounded-full bg-purple-500/20 border border-purple-500/30">
                  <svg className="w-4 h-4 text-purple-400 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M13 10V3L4 14h7v7l9-11h-7z"></path>
                  </svg>
                  <span className="text-purple-300 text-sm">Early access confirmed</span>
                </div>
              </div>
            </div>
          )}

          <div className="mt-12 flex flex-wrap justify-center gap-8">
            <div className="flex items-center space-x-2 hover:scale-105 transition-transform duration-300">
              <div className="w-10 h-10 rounded-full bg-cyan-500/20 flex items-center justify-center shadow-lg shadow-cyan-500/20">
                <svg className="w-5 h-5 text-cyan-400" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"></path>
                </svg>
              </div>
              <span className="text-gray-300 group-hover:text-white">Secure & Private</span>
            </div>
            <div className="flex items-center space-x-2 hover:scale-105 transition-transform duration-300">
              <div className="w-10 h-10 rounded-full bg-purple-500/20 flex items-center justify-center shadow-lg shadow-purple-500/20">
                <svg className="w-5 h-5 text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M11 5.882V19.24a1.76 1.76 0 01-3.417.592l-2.147-6.15M18 13a3 3 0 100-6M5.436 13.683A4.001 4.001 0 017 6h1.832c4.1 0 7.625-1.234 9.168-3v14c-1.543-1.766-5.067-3-9.168-3H7a3.988 3.988 0 01-1.564-.317z"></path>
                </svg>
              </div>
              <span className="text-gray-300 group-hover:text-white">Early Updates</span>
            </div>
            <div className="flex items-center space-x-2 hover:scale-105 transition-transform duration-300">
              <div className="w-10 h-10 rounded-full bg-blue-500/20 flex items-center justify-center shadow-lg shadow-blue-500/20">
                <svg className="w-5 h-5 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4"></path>
                </svg>
              </div>
              <span className="text-gray-300 group-hover:text-white">Exclusive NFTs</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default WaitlistSection;
