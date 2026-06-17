import { useState } from "react";
import Dashboard from "@/features/dashboard/Dashboard";
import type { MockScenario, SensorMode } from "@/features/sensors/types";
import { useSensorDiagnostics } from "@/features/sensors/useSensorDiagnostics";
import { useSensorSnapshot } from "@/features/sensors/useSensorSnapshot";

export default function DashboardPage() {
  const [mode, setMode] = useState<SensorMode>("live");
  const [scenario, setScenario] = useState<MockScenario>("normal");
  const { snapshot, error } = useSensorSnapshot(mode, scenario);
  const diagnostics = useSensorDiagnostics();

  return (
    <Dashboard
      mode={mode}
      snapshot={snapshot}
      error={error}
      diagnostics={diagnostics.diagnostics}
      diagnosticsError={diagnostics.error}
      isDiagnosticsLoading={diagnostics.isLoading}
      scenario={scenario}
      onCaptureDiagnostics={diagnostics.capture}
      onModeChange={setMode}
      onScenarioChange={setScenario}
    />
  );
}
