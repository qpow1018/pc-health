import { useState } from "react";
import ProductHome from "@/features/product-home/ProductHome";
import NetworkIncidentDetailPage from "@/features/network-incidents/NetworkIncidentDetailPage";
import NetworkIncidentHistoryPage from "@/features/network-incidents/NetworkIncidentHistoryPage";
import type { NetworkIncident } from "@/features/network-incidents/types";
import styles from "./HomePage.module.css";

type HomeView = "home" | "incident_history";

export default function HomePage() {
  const [view, setView] = useState<HomeView>("home");
  const [selectedIncident, setSelectedIncident] = useState<NetworkIncident | null>(
    null,
  );
  const isIncidentSection = view === "incident_history" || selectedIncident !== null;

  const openHistory = () => {
    setSelectedIncident(null);
    setView("incident_history");
  };

  const openHome = () => {
    setSelectedIncident(null);
    setView("home");
  };

  return (
    <>
      <header className={styles["app-header"]}>
        <div className={styles["app-header-inner"]}>
          <strong className={styles["brand"]}>Net Checker</strong>
          <nav className={styles["nav"]} aria-label="주요 화면">
            <button
              type="button"
              aria-current={!isIncidentSection ? "page" : undefined}
              onClick={openHome}
            >
              현재 진단
            </button>
            <button
              type="button"
              aria-current={isIncidentSection ? "page" : undefined}
              onClick={openHistory}
            >
              장애 이력
            </button>
          </nav>
        </div>
      </header>
      {selectedIncident ? (
        <NetworkIncidentDetailPage
          incident={selectedIncident}
          onBack={openHistory}
        />
      ) : view === "incident_history" ? (
        <NetworkIncidentHistoryPage onOpenDetail={setSelectedIncident} />
      ) : (
        <ProductHome onOpenIncidentHistory={openHistory} />
      )}
    </>
  );
}
