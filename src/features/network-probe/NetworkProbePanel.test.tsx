import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import NetworkProbePanel from "./NetworkProbePanel";
import { networkProbeFixture as snapshot } from "./fixture";

const { getSnapshotMock } = vi.hoisted(() => ({
  getSnapshotMock: vi.fn(),
}));

vi.mock("./api", () => ({
  getNetworkProbeSnapshot: getSnapshotMock,
}));

describe("NetworkProbePanel", () => {
  beforeEach(() => {
    getSnapshotMock.mockReset();
  });

  it("runs once and displays formatted raw JSON", async () => {
    getSnapshotMock.mockResolvedValue(snapshot);
    const user = userEvent.setup();
    render(<NetworkProbePanel />);

    await user.click(screen.getByRole("button", { name: "네트워크 확인 실행" }));

    expect(getSnapshotMock).toHaveBeenCalledTimes(1);
    expect(screen.getByText(/"collector": "windows-native"/)).toBeInTheDocument();
  });

  it("prevents duplicate runs while the probe is pending", async () => {
    getSnapshotMock.mockReturnValue(new Promise(() => {}));
    const user = userEvent.setup();
    render(<NetworkProbePanel />);

    await user.click(screen.getByRole("button", { name: "네트워크 확인 실행" }));

    expect(screen.getByRole("button", { name: "확인 중" })).toBeDisabled();
    expect(getSnapshotMock).toHaveBeenCalledTimes(1);
  });

  it("copies the displayed JSON", async () => {
    getSnapshotMock.mockResolvedValue(snapshot);
    const user = userEvent.setup();
    render(<NetworkProbePanel />);
    await user.click(screen.getByRole("button", { name: "네트워크 확인 실행" }));

    await user.click(screen.getByRole("button", { name: "JSON 복사" }));

    await expect(navigator.clipboard.readText()).resolves.toBe(
      JSON.stringify(snapshot, null, 2),
    );
    expect(screen.getByText("복사했습니다.")).toBeInTheDocument();
  });

  it("shows command errors without inventing a result", async () => {
    getSnapshotMock.mockRejectedValue(new Error("Tauri command unavailable"));
    const user = userEvent.setup();
    render(<NetworkProbePanel />);

    await user.click(screen.getByRole("button", { name: "네트워크 확인 실행" }));

    expect(screen.getByRole("alert")).toHaveTextContent("Tauri command unavailable");
    expect(screen.queryByRole("button", { name: "JSON 복사" })).not.toBeInTheDocument();
  });
});
