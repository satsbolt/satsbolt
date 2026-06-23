"use client";

import { motion } from "framer-motion";
import { Button } from "@/components/ui/button";

export function CTA() {
  return (
    <section className="py-24 md:py-40 bg-[#050816] relative overflow-hidden" id="cta">
      {/* Intense Center Glow */}
      <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-full max-w-5xl h-[500px] bg-[#F7931A]/10 blur-[150px] rounded-full pointer-events-none" />

      <div className="container mx-auto px-6 md:px-12 relative z-10 text-center">
        <motion.div
          initial={{ opacity: 0, scale: 0.95 }}
          whileInView={{ opacity: 1, scale: 1 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6, ease: "easeOut" }}
          className="max-w-4xl mx-auto"
        >
          <h2 className="text-5xl md:text-7xl lg:text-8xl font-bold text-white tracking-tight leading-[1.1] mb-8">
            Ready to Accept Payments <br className="hidden md:block" />
            <span className="text-[#F7931A] drop-shadow-[0_0_30px_rgba(247,147,26,0.3)]">Without Delays?</span>
          </h2>
          
          <p className="text-xl md:text-2xl text-[#94A3B8] mb-12 max-w-2xl mx-auto">
            Join the elite businesses powering their payments with Lightning. Early access is closing soon.
          </p>

          <form className="flex flex-col sm:flex-row items-center justify-center gap-4 max-w-xl mx-auto" onSubmit={(e) => e.preventDefault()}>
            <input
              type="email"
              placeholder="Enter your email address"
              className="w-full sm:w-[350px] h-14 rounded-full bg-white/5 border border-white/20 px-6 text-white placeholder:text-white/40 focus:outline-none focus:ring-2 focus:ring-[#F7931A]/50 transition-all text-lg"
              required
            />
            <Button
              size="lg"
              className="w-full sm:w-auto bg-[#F7931A] text-black hover:bg-[#F7931A]/90 font-bold rounded-full px-10 h-14 text-lg shadow-[0_0_20px_rgba(247,147,26,0.35)] hover:scale-105 transition-all"
            >
              Join Waitlist
            </Button>
          </form>
          <p className="text-sm text-[#94A3B8]/60 mt-6 font-medium">
            No spam. We will only contact you about early access.
          </p>
        </motion.div>
      </div>
    </section>
  );
}
