"use client";

import { motion } from "framer-motion";

const timelineEvents = [
  {
    year: "2007",
    title: "Mobile Money is Born",
    description: "M-Pesa revolutionizes African finance, proving that the continent can leapfrog traditional banking infrastructure.",
    color: "bg-blue-500",
  },
  {
    year: "2009",
    title: "The Genesis Block",
    description: "Satoshi Nakamoto mines the first Bitcoin block, creating a decentralized, censorship-resistant global currency.",
    color: "bg-[#F7931A]",
  },
  {
    year: "2018",
    title: "Lightning Network",
    description: "Layer 2 scaling solution launches, enabling instant, near-zero fee Bitcoin transactions globally.",
    color: "bg-purple-500",
  },
  {
    year: "2024",
    title: "Satbolt Bridges the Gap",
    description: "We combine the speed of Lightning with the familiarity of local fiat, giving African merchants the ultimate payment gateway.",
    color: "bg-[#22C55E]",
  },
];

export function Origins() {
  return (
    <section className="py-24 md:py-32 bg-[#050816] relative border-t border-white/5" id="origins">
      <div className="container mx-auto px-6 md:px-12 max-w-4xl">
        <div className="text-center mb-16 md:mb-24">
          <h2 className="text-4xl md:text-5xl font-bold text-white tracking-tight mb-6">
            The Evolution of <br className="hidden sm:block" />
            <span className="text-[#94A3B8]">African Commerce</span>
          </h2>
        </div>

        <div className="relative">
          {/* Vertical Line */}
          <div className="absolute left-8 md:left-1/2 top-0 bottom-0 w-px bg-white/10 -translate-x-1/2" />

          <div className="flex flex-col gap-12 md:gap-20">
            {timelineEvents.map((event, idx) => {
              const isEven = idx % 2 === 0;
              return (
                <div key={event.year} className="relative flex items-center md:justify-between w-full">
                  
                  {/* Marker */}
                  <div className="absolute left-8 md:left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 z-10 w-4 h-4 rounded-full bg-[#050816] border-2 border-white/20 shadow-[0_0_15px_rgba(255,255,255,0.1)] flex items-center justify-center">
                    <div className={`w-2 h-2 rounded-full ${event.color}`} />
                  </div>

                  {/* Content (Left or Right) */}
                  <motion.div
                    initial={{ opacity: 0, x: isEven ? -50 : 50, y: 20 }}
                    whileInView={{ opacity: 1, x: 0, y: 0 }}
                    viewport={{ once: true, margin: "-100px" }}
                    transition={{ duration: 0.6, delay: idx * 0.1 }}
                    className={`ml-20 md:ml-0 w-full md:w-5/12 ${isEven ? "md:text-right md:pr-12" : "md:ml-auto md:pl-12"}`}
                  >
                    <div className="inline-block px-4 py-1 rounded-full bg-white/5 border border-white/10 text-white font-bold mb-4">
                      {event.year}
                    </div>
                    <h3 className="text-2xl font-bold text-white mb-3">{event.title}</h3>
                    <p className="text-[#94A3B8] leading-relaxed">
                      {event.description}
                    </p>
                  </motion.div>
                </div>
              );
            })}
          </div>
        </div>
      </div>
    </section>
  );
}
