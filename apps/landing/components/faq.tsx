"use client";

import { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { Plus, Minus } from "lucide-react";

const faqs = [
  {
    question: "Do my customers need to own Bitcoin to pay me?",
    answer: "No. Your customers can pay using any Lightning wallet, but we are also integrating local fiat options so they can pay exactly how they do today.",
  },
  {
    question: "How do I get my money?",
    answer: "Funds are instantly settled to your registered Nigerian bank account in Naira. We handle the Lightning-to-Fiat conversion instantly in the background.",
  },
  {
    question: "Are there really zero fees?",
    answer: "Yes. Lightning network routing fees are fractions of a cent, which we absorb. You get 100% of the sale amount.",
  },
  {
    question: "How fast is integration?",
    answer: "Our API is designed to be integrated in less than an hour. We also provide ready-to-use plugins for Shopify, WooCommerce, and custom checkout links.",
  },
];

export function FAQ() {
  const [openIndex, setOpenIndex] = useState<number | null>(0);

  return (
    <section className="py-24 md:py-32 bg-[#050816] relative border-t border-white/5" id="faq">
      <div className="container mx-auto px-6 md:px-12 max-w-4xl">
        <div className="text-center mb-16 md:mb-24">
          <h2 className="text-4xl md:text-5xl font-bold text-white tracking-tight mb-6">
            Frequently Asked <br className="hidden sm:block" />
            <span className="text-[#F7931A]">Questions</span>
          </h2>
        </div>

        <div className="flex flex-col gap-4">
          {faqs.map((faq, idx) => {
            const isOpen = openIndex === idx;
            return (
              <motion.div
                key={idx}
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.4, delay: idx * 0.1 }}
                className={`border rounded-2xl overflow-hidden transition-colors duration-300 ${
                  isOpen ? "bg-white/5 border-white/20" : "bg-transparent border-white/10 hover:border-white/20"
                }`}
              >
                <button
                  onClick={() => setOpenIndex(isOpen ? null : idx)}
                  className="w-full flex items-center justify-between p-6 text-left focus:outline-none"
                >
                  <span className="text-lg font-semibold text-white">{faq.question}</span>
                  <div className={`w-8 h-8 rounded-full flex items-center justify-center transition-colors ${isOpen ? "bg-[#F7931A] text-black" : "bg-white/10 text-white"}`}>
                    {isOpen ? <Minus className="w-4 h-4" /> : <Plus className="w-4 h-4" />}
                  </div>
                </button>
                <AnimatePresence>
                  {isOpen && (
                    <motion.div
                      initial={{ height: 0, opacity: 0 }}
                      animate={{ height: "auto", opacity: 1 }}
                      exit={{ height: 0, opacity: 0 }}
                      transition={{ duration: 0.3, ease: "easeInOut" }}
                    >
                      <div className="p-6 pt-0 text-[#94A3B8] leading-relaxed border-t border-white/5 mt-2">
                        {faq.answer}
                      </div>
                    </motion.div>
                  )}
                </AnimatePresence>
              </motion.div>
            );
          })}
        </div>
      </div>
    </section>
  );
}
