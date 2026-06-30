import { useState } from "react";
import ProductHome from "@/features/product-home/ProductHome";
import NetworkIncidentHistoryPage from "@/features/network-incidents/NetworkIncidentHistoryPage";

type HomeView = "home" | "incident_history";

export default function HomePage() {
  const [view, setView] = useState<HomeView>("home");

  if (view === "incident_history") {
    return <NetworkIncidentHistoryPage onBack={() => setView("home")} />;
  }

  return <ProductHome onOpenIncidentHistory={() => setView("incident_history")} />;
}
