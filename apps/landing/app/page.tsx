import { Navbar } from "@/components/navbar";
import { Hero } from "@/components/hero";
import { TrustedBy } from "@/components/trusted-by";
import { Problem } from "@/components/problem";
import { Origins } from "@/components/origins";
import { StickyFeatures } from "@/components/sticky-features";
import { Solution } from "@/components/solution";
import { MerchantSection } from "@/components/merchant-section";
import { DeveloperSection } from "@/components/developer-section";
import { FAQ } from "@/components/faq";
import { CTA } from "@/components/cta";
import { Footer } from "@/components/footer";

export default function Home() {
  return (
    <div className="flex flex-col min-h-screen">
      <Navbar />
      <main className="flex-1">
        <Hero />
        <TrustedBy />
        <Problem />
        <Origins />
        <StickyFeatures />
        <Solution />
        <MerchantSection />
        <DeveloperSection />
        <FAQ />
        <CTA />
      </main>
      <Footer />
    </div>
  );
}
