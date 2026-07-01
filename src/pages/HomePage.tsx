import { useState } from "react";
import ProductHome from "@/features/product-home/ProductHome";
import NetworkIncidentHistoryPage from "@/features/network-incidents/NetworkIncidentHistoryPage";
import styles from "./HomePage.module.css";

type HomeView = "home" | "incident_history";

export default function HomePage() {
  const [view, setView] = useState<HomeView>("home");

  return (
    <>
      <header className={styles["app-header"]}>
        <div className={styles["app-header-inner"]}>
          <strong className={styles["brand"]}>PC Health</strong>
          <nav className={styles["nav"]} aria-label="주요 화면">
            <button
              type="button"
              aria-current={view === "home" ? "page" : undefined}
              onClick={() => setView("home")}
            >
              현재 진단
            </button>
            <button
              type="button"
              aria-current={view === "incident_history" ? "page" : undefined}
              onClick={() => setView("incident_history")}
            >
              장애 이력
            </button>
          </nav>
        </div>
      </header>
      {view === "incident_history" ? (
        <NetworkIncidentHistoryPage />
      ) : (
        <ProductHome onOpenIncidentHistory={() => setView("incident_history")} />
      )}
    </>
  );
}
