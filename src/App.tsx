import "./App.css";
import { useState } from "react";
import Dashboard from "./components/Dashboard";
import type { MockScenario } from "./sensors/types";
import { useSensorSnapshot } from "./sensors/useSensorSnapshot";

function App() {
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

export default App;
