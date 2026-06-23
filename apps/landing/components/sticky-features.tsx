"use client";

import { useEffect, useRef, useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import Image from "next/image";
import { CheckCircle2 } from "lucide-react";

const features = [
  {
    id: "scan",
    title: "Scan & Pay Instantly",
    description: "Scan any Lightning or Satbolt QR code. Payment settles within milliseconds.",
    image: "/assets/qr_scanner.png",
  },
  {
    id: "settlement",
    title: "Instant Naira Settlement",
    description: "No more waiting for T+2. Funds drop directly into your local account immediately.",
    image: "/assets/notification.png",
  },
  {
    id: "fees",
    title: "Absolute Zero Fees",
    description: "Keep 100% of your revenue. We make money when you scale, not by taxing your transactions.",
    image: "/assets/dashboard.png",
  },
];

export function StickyFeatures() {
  const [activeFeature, setActiveFeature] = useState(features[0].id);
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleScroll = () => {
      if (!containerRef.current) return;
      
      const sections = containerRef.current.querySelectorAll('.feature-block');
      let currentId = features[0].id;

      sections.forEach((section) => {
        const rect = section.getBoundingClientRect();
        // If the top of the section is somewhat near the middle of the screen
        if (rect.top <= window.innerHeight / 2 && rect.bottom >= window.innerHeight / 2) {
          currentId = section.id;
        }
      });

      if (currentId !== activeFeature) {
        setActiveFeature(currentId);
      }
    };

    window.addEventListener("scroll", handleScroll);
    return () => window.removeEventListener("scroll", handleScroll);
  }, [activeFeature]);

  const activeImage = features.find((f) => f.id === activeFeature)?.image || features[0].image;

  return (
    <section className="py-24 md:py-32 bg-[#050816] relative border-t border-white/5" id="sticky-features">
      <div className="container mx-auto px-6 md:px-12">
        <div className="flex flex-col md:flex-row items-start justify-between gap-12 md:gap-24" ref={containerRef}>
          
          {/* Scrolling Content (Left Side) */}
          <div className="w-full md:w-1/2 flex flex-col pt-10 md:pt-32 pb-10 md:pb-32 gap-32 md:gap-64">
            {features.map((feature, idx) => (
              <div 
                key={feature.id} 
                id={feature.id} 
                className={`feature-block flex flex-col transition-opacity duration-500 ${activeFeature === feature.id ? 'opacity-100' : 'opacity-30'}`}
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
                
                {/* Mobile Image Display (Only shows on mobile, static) */}
                <div className="block md:hidden mt-10 w-full aspect-[4/5] relative rounded-3xl overflow-hidden border border-white/10">
                  <Image src={feature.image} alt={feature.title} fill className="object-cover" />
                </div>
              </div>
            ))}
          </div>

          {/* Sticky Visual (Right Side - Desktop Only) */}
          <div className="hidden md:flex w-full md:w-1/2 sticky top-32 h-[600px] items-center justify-center">
            <div className="relative w-full max-w-[400px] aspect-[4/5] rounded-[2.5rem] overflow-hidden border border-white/10 shadow-[0_0_50px_rgba(34,197,94,0.1)]">
              <AnimatePresence mode="wait">
                <motion.div
                  key={activeFeature}
                  initial={{ opacity: 0, scale: 0.95 }}
                  animate={{ opacity: 1, scale: 1 }}
                  exit={{ opacity: 0, scale: 1.05 }}
                  transition={{ duration: 0.4, ease: "easeInOut" }}
                  className="absolute inset-0"
                >
                  <Image
                    src={activeImage}
                    alt="Feature Mockup"
                    fill
                    className="object-cover"
                    priority
                  />
                </motion.div>
              </AnimatePresence>
            </div>
          </div>

        </div>
      </div>
    </section>
  );
}
