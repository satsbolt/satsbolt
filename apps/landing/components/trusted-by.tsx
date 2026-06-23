import { Bitcoin, Zap, Cpu, TerminalSquare } from "lucide-react";

export function TrustedBy() {
  const stack = [
    { name: "Bitcoin Core", icon: Bitcoin },
    { name: "Lightning Network", icon: Zap },
    { name: "LDK", icon: Cpu },
    { name: "Rust Built", icon: TerminalSquare },
  ];

  return (
    <section className="py-12 border-y border-white/5 bg-[#050816]/50">
      <div className="container mx-auto px-6 md:px-12 text-center">
        <p className="text-sm font-medium text-[#94A3B8] tracking-widest uppercase mb-8">
          Trusted infrastructure powering modern payments
        </p>
        <div className="flex flex-wrap justify-center gap-10 md:gap-24 items-center opacity-70">
          {stack.map((item) => {
            const Icon = item.icon;
            return (
              <div
                key={item.name}
                className="flex items-center gap-3 text-[#94A3B8] hover:text-white transition-colors duration-300"
              >
                <Icon className="w-7 h-7" />
                <span className="font-semibold text-xl tracking-tight">{item.name}</span>
              </div>
            );
          })}
        </div>
      </div>
    </section>
  );
}
