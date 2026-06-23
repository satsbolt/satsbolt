"use client";

import { motion } from "framer-motion";
import { Terminal, ArrowRight } from "lucide-react";
import { Button } from "@/components/ui/button";

export function DeveloperSection() {
  return (
    <section className="py-24 md:py-32 bg-black relative overflow-hidden" id="developers">
      {/* Abstract Code Watermark */}
      <div className="absolute inset-0 opacity-[0.03] pointer-events-none flex items-center justify-center overflow-hidden">
        <pre className="text-[#22C55E] text-xs sm:text-sm font-mono leading-loose whitespace-pre select-none">
          {`
impl LightningPayment for SatboldNode {
    fn process_payment(&self, invoice: &Invoice) -> Result<PaymentHash, Error> {
        let channel_manager = self.channel_manager.read().unwrap();
        let payment_id = PaymentId(generate_random_bytes());
        
        match channel_manager.send_payment(invoice.route(), payment_id) {
            Ok(_) => {
                log::info!("Payment routed successfully across LDK.");
                Ok(invoice.payment_hash())
            },
            Err(e) => {
                log::error!("Failed to route payment: {:?}", e);
                Err(Error::RoutingFailed)
            }
        }
    }
}
          `.repeat(4)}
        </pre>
      </div>

      {/* Subtle Glow */}
      <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-full max-w-4xl h-[400px] bg-[#22C55E]/5 blur-[120px] rounded-full pointer-events-none" />

      <div className="container mx-auto px-6 md:px-12 relative z-10">
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.8, ease: "easeOut" }}
          className="max-w-4xl mx-auto text-center border border-white/10 bg-[#050816]/80 backdrop-blur-xl p-10 md:p-16 rounded-[2.5rem]"
        >
          <div className="w-16 h-16 rounded-2xl bg-white/5 flex items-center justify-center mx-auto mb-8 border border-white/10">
            <Terminal className="w-8 h-8 text-white" />
          </div>
          
          <h2 className="text-4xl md:text-5xl lg:text-6xl font-bold text-white tracking-tight mb-8">
            Built on LDK & Rust. <br />
            <span className="text-[#94A3B8]">Open APIs for African developers.</span>
          </h2>
          
          <p className="text-[#94A3B8] text-lg md:text-xl max-w-2xl mx-auto mb-12">
            Powerful, well-documented endpoints. Webhooks for real-time state changes. Engineered for massive scale and perfect uptime.
          </p>
          
          <Button
            variant="outline"
            size="lg"
            className="border-white/20 text-white hover:bg-white hover:text-black font-semibold rounded-full px-10 h-14 text-lg transition-all group"
          >
            Read the Docs
            <ArrowRight className="ml-2 w-5 h-5 group-hover:translate-x-1 transition-transform" />
          </Button>
        </motion.div>
      </div>
    </section>
  );
}
