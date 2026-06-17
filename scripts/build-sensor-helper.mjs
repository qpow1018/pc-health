import { mkdir, rename } from "node:fs/promises";
import { spawnSync } from "node:child_process";

const outputDir = "src-tauri/binaries";
const project = "src-tauri/helpers/sensor-helper/PcHealth.SensorHelper.csproj";
const source = `${outputDir}/pc-health-sensor-helper.exe`;
const target = `${outputDir}/pc-health-sensor-helper-x86_64-pc-windows-msvc.exe`;

await mkdir(outputDir, { recursive: true });

const publish = spawnSync(
  "dotnet",
  [
    "publish",
    project,
    "-c",
    "Release",
    "-r",
    "win-x64",
    "--self-contained",
    "true",
    "/p:PublishSingleFile=true",
    "/p:IncludeNativeLibrariesForSelfExtract=true",
    "-o",
    outputDir,
  ],
  { stdio: "inherit" },
);

if (publish.error) {
  throw publish.error;
}

if (publish.status !== 0) {
  process.exit(publish.status ?? 1);
}

await rename(source, target);
