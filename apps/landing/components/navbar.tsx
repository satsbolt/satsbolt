"use client";

import { useState, useEffect } from "react";
import Image from "next/image";
import Link from "next/link";
import { Button } from "@/components/ui/button";

export function Navbar() {
  const [scrolled, setScrolled] = useState(false);

  useEffect(() => {
    const handleScroll = () => {
      setScrolled(window.scrollY > 20);
    };
    window.addEventListener("scroll", handleScroll);
    return () => window.removeEventListener("scroll", handleScroll);
  }, []);

  return (
    <header
      className={`fixed top-0 left-0 right-0 z-50 transition-all duration-300 border-b ${
        scrolled
          ? "bg-[#050816]/80 backdrop-blur-md border-white/10 py-4"
          : "bg-transparent border-transparent py-6"
      }`}
    >
      <div className="container mx-auto px-6 md:px-12 flex items-center justify-between">
        <Link href="/" className="flex items-center relative z-10">
          <Image
            src="/assets/logo.png"
            alt="Satbolt Logo"
            width={280}
            height={80}
            priority
            className="h-14 md:h-16 w-auto object-contain drop-shadow-[0_2px_10px_rgba(255,255,255,0.1)]"
          />
        </Link>

        <nav className="hidden md:flex items-center gap-12">
          {["Features", "Merchants", "Developers", "About"].map((item) => (
            <Link
              key={item}
              href={`#${item.toLowerCase()}`}
              className="text-base md:text-lg font-bold text-white/90 hover:text-white transition-colors tracking-wide"
            >
              {item}
            </Link>
          ))}
        </nav>

        <div className="flex items-center gap-4">
          <Button
            className="bg-[#F7931A] text-black hover:bg-[#F7931A]/90 font-bold rounded-full px-8 h-12 text-base shadow-[0_0_20px_rgba(247,147,26,0.35)] hover:scale-105 transition-all"
          >
            Join Waitlist
          </Button>
        </div>
      </div>
    </header>
  );
}
