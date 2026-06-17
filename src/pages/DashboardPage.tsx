import { useState } from "react";
import Dashboard from "@/features/dashboard/Dashboard";
import type { MockScenario, SensorMode } from "@/features/sensors/types";
import { useSensorSnapshot } from "@/features/sensors/useSensorSnapshot";

export default function DashboardPage() {
  const [mode, setMode] = useState<SensorMode>("live");
  const [scenario, setScenario] = useState<MockScenario>("normal");
  const { snapshot, error } = useSensorSnapshot(mode, scenario);

  return (
    <Dashboard
      mode={mode}
      snapshot={snapshot}
      error={error}
      scenario={scenario}
      onModeChange={setMode}
      onScenarioChange={setScenario}
    />
  );
}
