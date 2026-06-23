"use client";

import Image from "next/image";
import { motion } from "framer-motion";
import { CheckCircle2 } from "lucide-react";

export function Solution() {
  const features = [
    {
      title: "Scan & Pay instantly.",
      description: "Fast, seamless QR payments powered by the Lightning Network. No waiting for confirmations.",
      image: "/assets/qr_scanner.png",
      reverse: false,
    },
    {
      title: "Instant Settlement.",
      description: "Receive funds directly into your bank account without volatility risks. Settle instantly in Naira.",
      image: "/assets/notification.png",
      reverse: true,
    },
    {
      title: "Global Reach.",
      description: "Accept payments from anyone, anywhere. Connect your African business to the global economy.",
      image: "/assets/global_map.png",
      reverse: false,
    },
  ];

  return (
    <section className="py-24 md:py-32 bg-[#050816] relative overflow-hidden" id="features">
      <div className="container mx-auto px-6 md:px-12 relative z-10">
        <div className="text-center max-w-3xl mx-auto mb-20 md:mb-32">
          <h2 className="text-4xl md:text-6xl font-bold text-white tracking-tight mb-6">
            Satbolt Keeps <br className="hidden md:block" />
            <span className="text-[#22C55E]">Commerce Moving.</span>
          </h2>
          <p className="text-xl text-[#94A3B8]">
            Settle instantly in Naira. No volatility. No waiting.
          </p>
        </div>

        <div className="flex flex-col gap-24 md:gap-40">
          {features.map((feature, idx) => (
            <div
              key={feature.title}
              className={`flex flex-col ${
                feature.reverse ? "md:flex-row-reverse" : "md:flex-row"
              } items-center gap-12 md:gap-24`}
            >
              {/* Image Side */}
              <motion.div
                initial={{ opacity: 0, x: feature.reverse ? 50 : -50 }}
                whileInView={{ opacity: 1, x: 0 }}
                viewport={{ once: true, margin: "-100px" }}
                transition={{ duration: 0.8, ease: "easeOut" }}
                className="w-full md:w-1/2"
              >
                <div className="relative w-full aspect-square md:aspect-[4/3] rounded-3xl overflow-hidden border border-white/10 shadow-[0_0_50px_rgba(34,197,94,0.05)]">
                  <Image
                    src={feature.image}
                    alt={feature.title}
                    fill
                    className="object-cover"
                  />
                </div>
              </motion.div>

              {/* Text Side */}
              <motion.div
                initial={{ opacity: 0, y: 30 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true, margin: "-100px" }}
                transition={{ duration: 0.8, ease: "easeOut", delay: 0.2 }}
                className="w-full md:w-1/2 flex flex-col justify-center"
              >
                <div className="inline-flex items-center gap-2 px-4 py-2 rounded-full bg-white/5 border border-white/10 w-fit mb-6">
                  <CheckCircle2 className="w-5 h-5 text-[#22C55E]" />
                  <span className="text-sm font-semibold text-white uppercase tracking-wider">Step 0{idx + 1}</span>
                </div>
                <h3 className="text-4xl md:text-5xl font-bold text-white mb-6 leading-tight">
                  {feature.title}
                </h3>
                <p className="text-[#94A3B8] text-lg md:text-xl leading-relaxed">
                  {feature.description}
                </p>
              </motion.div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
