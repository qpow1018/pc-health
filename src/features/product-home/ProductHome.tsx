import NetworkStatusPanel from "@/features/network-diagnostics/NetworkStatusPanel";
import NetworkProbePanel from "@/features/network-probe/NetworkProbePanel";
import styles from "./ProductHome.module.css";

const areas = [
  {
    title: "인터넷 장애 진단",
    status: "활성",
    description: "PC, 로컬 연결, 공유기, DNS와 외부 회선 상태를 구분합니다.",
  },
  {
    title: "드라이버 관리",
    status: "기획 중",
    description:
      "설치 버전, Windows Update 후보와 처리 이력을 읽기 전용으로 확인합니다.",
  },
];

export default function ProductHome() {
  return (
    <main className={styles["shell"]}>
      <header className={styles["header"]}>
        <p className={styles["eyebrow"]}>READ-ONLY PC DIAGNOSTICS</p>
        <h1>PC Health</h1>
        <p>
          인터넷 장애 진단과 드라이버 상태 확인을 위한 Windows
          유틸리티입니다.
        </p>
      </header>
      <section className={styles["areas"]} aria-label="제품 영역">
        {areas.map((area) => (
          <article className={styles["area"]} key={area.title}>
            <span>{area.status}</span>
            <h2>{area.title}</h2>
            <p>{area.description}</p>
          </article>
        ))}
      </section>
      <NetworkStatusPanel />
      <NetworkProbePanel />
    </main>
  );
}
