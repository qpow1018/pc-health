import { useState } from "react";
import Dashboard from "@/features/dashboard/Dashboard";
import type { MockScenario } from "@/features/sensors/types";
import { useSensorSnapshot } from "@/features/sensors/useSensorSnapshot";

export default function DashboardPage() {
  const [scenario, setScenario] = useState<MockScenario>("normal");
  const { snapshot, error } = useSensorSnapshot(scenario);

  return (
    <Dashboard
      snapshot={snapshot}
      error={error}
      scenario={scenario}
      onScenarioChange={setScenario}
    />
  );
}
