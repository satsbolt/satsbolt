"use client";

import { motion } from "framer-motion";
import { Shield, RefreshCw, Code2, CalendarClock } from "lucide-react";

export function MerchantSection() {
  const features = [
    {
      title: "Zero Chargebacks",
      description: "Lightning transactions are final and cryptographically secure. Say goodbye to fraud and costly chargeback disputes.",
      icon: Shield,
    },
    {
      title: "Auto-Conversion",
      description: "Get paid in Naira instantly. We handle the Lightning network volatility so you don't have to.",
      icon: RefreshCw,
    },
    {
      title: "Easy API",
      description: "Developer-first documentation. Integrate our seamless checkout experience in just 10 lines of code.",
      icon: Code2,
    },
    {
      title: "24/7 Settlements",
      description: "No weekends. No bank holidays. Your money settles into your account the second the transaction completes.",
      icon: CalendarClock,
    },
  ];

  return (
    <section className="py-24 md:py-32 bg-[#050816] relative border-t border-white/5" id="merchants">
      <div className="container mx-auto px-6 md:px-12">
        <div className="flex flex-col md:flex-row md:items-end justify-between mb-16 md:mb-24 gap-8">
          <div className="max-w-2xl">
            <h2 className="text-4xl md:text-6xl font-bold text-white tracking-tight mb-6">
              Everything you need <br className="hidden sm:block" />
              <span className="text-[#94A3B8]">to scale your business.</span>
            </h2>
            <p className="text-[#94A3B8] text-lg md:text-xl">
              Built specifically for the demands of modern African commerce.
            </p>
          </div>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-6 md:gap-10">
          {features.map((feature, idx) => {
            const Icon = feature.icon;
            return (
              <motion.div
                key={feature.title}
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.5, delay: idx * 0.1 }}
                className="group p-8 md:p-10 rounded-3xl bg-white/[0.02] border border-white/5 hover:bg-white/[0.04] transition-all duration-300"
              >
                <div className="w-14 h-14 rounded-2xl bg-[#22C55E]/10 flex items-center justify-center mb-6 text-[#22C55E] group-hover:scale-110 transition-transform duration-300">
                  <Icon className="w-7 h-7" />
                </div>
                <h3 className="text-2xl font-bold text-white mb-4">{feature.title}</h3>
                <p className="text-[#94A3B8] leading-relaxed text-lg">
                  {feature.description}
                </p>
              </motion.div>
            );
          })}
        </div>
      </div>
    </section>
  );
}
