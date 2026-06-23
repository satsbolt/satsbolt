"use client";

import Image from "next/image";
import { Button } from "@/components/ui/button";
import { motion } from "framer-motion";

export function Hero() {
  return (
    <section className="relative pt-32 pb-20 md:pt-48 md:pb-32 overflow-hidden min-h-[90vh] flex items-center" id="hero">
      {/* Background Image with Overlay */}
      <div className="absolute inset-0 z-0">
        <Image
          src="/assets/hero_bg.png"
          alt="Satbolt Background"
          fill
          className="object-cover object-center opacity-40"
          priority
        />
        <div className="absolute inset-0 bg-gradient-to-b from-[#050816]/80 via-[#050816]/60 to-[#050816]" />
      </div>

      <div className="container mx-auto px-6 md:px-12 relative z-10 flex flex-col items-center">
        
        {/* Centered Copy */}
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8, ease: "easeOut" }}
          className="flex flex-col items-center w-full"
        >
          <h1 className="text-5xl md:text-7xl lg:text-[90px] font-bold tracking-tight leading-[1.15] mb-12">
            <span className="text-white">Payments </span>
            <span className="text-[#22C55E]">That </span>
            <span className="text-[#F7931A] drop-shadow-[0_0_40px_rgba(247,147,26,0.2)]">Never</span>
            <br />
            <span className="text-[#94A3B8]">Fail Anywhere in Africa</span>
          </h1>
        </motion.div>

        {/* Centered Visual (Phone Mockup) */}
        <motion.div
          initial={{ opacity: 0, y: 50 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 1, ease: "easeOut", delay: 0.3 }}
          className="relative flex justify-center w-full"
        >
          <motion.div
            animate={{ y: [0, -15, 0] }}
            transition={{ repeat: Infinity, duration: 6, ease: "easeInOut" }}
            className="relative w-full max-w-[500px] md:max-w-[750px] aspect-square mx-auto mt-4"
          >
            {/* Soft backdrop behind the phone to blend it into the dark background */}
            <div className="absolute inset-0 bg-gradient-to-t from-[#050816] via-transparent to-transparent z-10 pointer-events-none" />
            <Image
              src="/assets/hero_mockup.png"
              alt="Satbolt Payment Success App Screen"
              fill
              priority
              className="object-contain drop-shadow-[0_20px_50px_rgba(0,0,0,0.5)] relative z-0 mix-blend-lighten"
            />
          </motion.div>
        </motion.div>

      </div>
    </section>
  );
}
