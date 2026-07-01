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
        <div>
          <p className={styles["eyebrow"]}>READ-ONLY PC DIAGNOSTICS</p>
          <h1>PC Health</h1>
          <p>
            인터넷 장애의 원인 구간을 근거와 함께 구분하는 Windows
            유틸리티입니다.
          </p>
        </div>
        <div className={styles["product-status"]}>
          <span>활성</span>
          <strong>인터넷 장애 진단</strong>
        </div>
      </header>
      <NetworkStatusPanel />
      <RecentIncidentsPanel onOpenHistory={onOpenIncidentHistory} />
      <NetworkProbePanel />
    </main>
  );
}
