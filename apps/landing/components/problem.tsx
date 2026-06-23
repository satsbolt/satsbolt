"use client";

import { motion } from "framer-motion";
import { Clock, AlertTriangle, TrendingDown } from "lucide-react";

export function Problem() {
  const problems = [
    {
      title: "Bank Delays",
      description: "Traditional rails take days to settle. Cash flow stalls while you wait.",
      icon: Clock,
      color: "text-red-400",
      glow: "group-hover:shadow-[0_0_30px_rgba(248,113,113,0.15)]",
      border: "group-hover:border-red-500/30",
    },
    {
      title: "Failed Payments",
      description: "High failure rates at checkout cost merchants up to 30% of their revenue.",
      icon: AlertTriangle,
      color: "text-orange-400",
      glow: "group-hover:shadow-[0_0_30px_rgba(251,146,60,0.15)]",
      border: "group-hover:border-orange-500/30",
    },
    {
      title: "Merchant Losses",
      description: "Network downtime leads to abandoned carts and frustrated customers.",
      icon: TrendingDown,
      color: "text-[#F7931A]",
      glow: "group-hover:shadow-[0_0_30px_rgba(247,147,26,0.15)]",
      border: "group-hover:border-[#F7931A]/30",
    },
  ];

  return (
    <section className="py-24 md:py-32 bg-[#050816] relative overflow-hidden" id="problem">
      {/* Background glow to signify the 'problem' area */}
      <div className="absolute top-0 left-1/2 -translate-x-1/2 w-full max-w-3xl h-[400px] bg-red-500/5 blur-[120px] rounded-full pointer-events-none" />

      <div className="container mx-auto px-6 md:px-12 relative z-10">
        <div className="text-center max-w-3xl mx-auto mb-16 md:mb-24">
          <h2 className="text-4xl md:text-6xl font-bold text-white tracking-tight mb-6">
            Commerce Stops When <br className="hidden md:block" />
            <span className="text-[#94A3B8]">Networks Fail.</span>
          </h2>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-6 md:gap-10">
          {problems.map((problem, idx) => {
            const Icon = problem.icon;
            return (
              <motion.div
                key={problem.title}
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.5, delay: idx * 0.1 }}
                className={`group relative p-8 rounded-3xl bg-white/[0.02] border border-white/5 backdrop-blur-sm transition-all duration-500 ${problem.border} ${problem.glow}`}
              >
                <div className={`w-14 h-14 rounded-2xl bg-white/[0.03] flex items-center justify-center mb-6 border border-white/5 transition-colors group-hover:bg-white/[0.05]`}>
                  <Icon className={`w-7 h-7 ${problem.color}`} />
                </div>
                <h3 className="text-2xl font-bold text-white mb-4">{problem.title}</h3>
                <p className="text-[#94A3B8] leading-relaxed text-lg">
                  {problem.description}
                </p>
              </motion.div>
            );
          })}
        </div>
      </div>
    </section>
  );
}
