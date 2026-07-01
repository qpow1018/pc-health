import NetworkStatusPanel from "@/features/network-diagnostics/NetworkStatusPanel";
import RecentIncidentsPanel from "@/features/network-incidents/RecentIncidentsPanel";
import NetworkProbePanel from "@/features/network-probe/NetworkProbePanel";
import styles from "./ProductHome.module.css";

export default function ProductHome({
  onOpenIncidentHistory,
}: {
  onOpenIncidentHistory: () => void;
}) {
  return (
    <main className={styles["shell"]}>
      <header className={styles["header"]}>
        <h1>PC Health</h1>
      </header>
      <NetworkStatusPanel />
      <RecentIncidentsPanel onOpenHistory={onOpenIncidentHistory} />
      <NetworkProbePanel />
    </main>
  );
}
