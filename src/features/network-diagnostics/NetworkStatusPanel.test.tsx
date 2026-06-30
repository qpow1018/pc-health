import { act, render, screen, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import NetworkStatusPanel from "./NetworkStatusPanel";
import {
  incidentStatusFixture,
  normalStatusFixture,
  startingStatusFixture,
  unavailableStatusFixture,
} from "./fixture";
import type { NetworkDiagnosticStatus } from "./types";
import styles from "./NetworkStatusPanel.module.css";

const { canUseMock, getStatusMock } = vi.hoisted(() => ({
  canUseMock: vi.fn(),
  getStatusMock: vi.fn(),
}));

vi.mock("./api", () => ({
  canUseNetworkDiagnostics: canUseMock,
  getNetworkDiagnosticStatus: getStatusMock,
}));

const deferred = <T,>() => {
  let resolve!: (value: T) => void;
  let reject!: (reason: unknown) => void;
  const promise = new Promise<T>((promiseResolve, promiseReject) => {
    resolve = promiseResolve;
    reject = promiseReject;
  });
  return { promise, resolve, reject };
};

function expectAreaLabel(label: string) {
  const areaRow = screen.getByText("추정 구간").closest("div");
  expect(areaRow).not.toBeNull();
  expect(within(areaRow as HTMLElement).getByText(label)).toBeInTheDocument();
}

describe("NetworkStatusPanel", () => {
  beforeEach(() => {
    canUseMock.mockReset();
    canUseMock.mockReturnValue(true);
    getStatusMock.mockReset();
    getStatusMock.mockResolvedValue(startingStatusFixture);
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("loads immediately and polls memory status every second", async () => {
    vi.useFakeTimers();
    getStatusMock.mockResolvedValue(normalStatusFixture);

    render(<NetworkStatusPanel />);
    expect(getStatusMock).toHaveBeenCalledTimes(1);

    await act(() => vi.advanceTimersByTimeAsync(1000));
    expect(getStatusMock).toHaveBeenCalledTimes(2);
  });

  it("does not overlap status calls while a previous request is pending", async () => {
    vi.useFakeTimers();
    const pending = deferred<NetworkDiagnosticStatus>();
    getStatusMock.mockReturnValue(pending.promise);

    render(<NetworkStatusPanel />);
    await act(() => vi.advanceTimersByTimeAsync(3000));

    expect(getStatusMock).toHaveBeenCalledTimes(1);
    pending.resolve(normalStatusFixture);
    await act(async () => pending.promise);
    await act(() => vi.advanceTimersByTimeAsync(1000));
    expect(getStatusMock).toHaveBeenCalledTimes(2);
  });

  it("clears polling and ignores a late result after unmount", async () => {
    vi.useFakeTimers();
    const pending = deferred<NetworkDiagnosticStatus>();
    getStatusMock.mockReturnValue(pending.promise);
    const consoleError = vi.spyOn(console, "error").mockImplementation(() => {});
    const { unmount } = render(<NetworkStatusPanel />);

    unmount();
    pending.resolve(normalStatusFixture);
    await act(async () => pending.promise);
    await act(() => vi.advanceTimersByTimeAsync(2000));

    expect(getStatusMock).toHaveBeenCalledTimes(1);
    expect(consoleError).not.toHaveBeenCalled();
    consoleError.mockRestore();
  });

  it("ignores a late command rejection after unmount", async () => {
    const pending = deferred<NetworkDiagnosticStatus>();
    getStatusMock.mockReturnValue(pending.promise);
    const consoleError = vi.spyOn(console, "error").mockImplementation(() => {});
    const { unmount } = render(<NetworkStatusPanel />);

    unmount();
    pending.reject(new Error("late failure"));
    await expect(pending.promise).rejects.toThrow("late failure");

    expect(consoleError).not.toHaveBeenCalled();
    consoleError.mockRestore();
  });

  it("shows browser availability without invoking or starting a timer", () => {
    vi.useFakeTimers();
    canUseMock.mockReturnValue(false);
    render(<NetworkStatusPanel />);

    expect(screen.getByText("Windows 앱에서 확인 가능")).toBeInTheDocument();
    expect(screen.getByText("확인 불가", { selector: "strong" })).toBeInTheDocument();
    expect(getStatusMock).not.toHaveBeenCalled();
    expect(vi.getTimerCount()).toBe(0);
  });

  it("shows first-check and unavailable states quietly", async () => {
    getStatusMock.mockResolvedValue(unavailableStatusFixture);
    render(<NetworkStatusPanel />);
    expect(screen.getByRole("status")).toHaveTextContent("첫 확인 중");
    expectAreaLabel("첫 확인 중");

    await screen.findByText("Windows 네트워크 진단을 사용할 수 없습니다.");
    expect(screen.getByText("확인 불가", { selector: "strong" })).toBeInTheDocument();
    expectAreaLabel("확인 불가");
  });

  it.each([
    ["normal", "정상"],
    ["suspected", "장애 의심"],
    ["incident", "장애 확인"],
    ["recovering", "복구 확인 중"],
    ["resolved", "복구됨"],
  ] as const)("maps %s lifecycle to %s", async (lifecycle, label) => {
    getStatusMock.mockResolvedValue({
      ...normalStatusFixture,
      lifecycle,
      suspectedArea: lifecycle === "normal" ? null : "unknown",
    });

    render(<NetworkStatusPanel />);
    expect(await screen.findByText(label)).toBeInTheDocument();
  });

  it.each([
    ["normal", null, "현재 확인된 구간에서 반복 이상이 없습니다."],
    [
      "suspected",
      "gateway_or_local",
      "이상이 감지됐지만 장애로 확정하려면 추가 근거가 필요합니다.",
    ],
    [
      "incident",
      "gateway_or_local",
      "호환되는 이상이 반복 확인되었습니다.",
    ],
    [
      "recovering",
      "gateway_or_local",
      "정상 근거가 확인되어 추가 확인 중입니다.",
    ],
    ["resolved", null, "현재 연결은 복구된 상태입니다."],
  ] as const)("shows direct summary copy for %s", async (
    lifecycle,
    suspectedArea,
    description,
  ) => {
    getStatusMock.mockResolvedValue({
      ...normalStatusFixture,
      lifecycle,
      suspectedArea,
    });

    render(<NetworkStatusPanel />);

    expect(await screen.findByText(description)).toBeInTheDocument();
  });

  it("shows quiet unavailable summary copy without claiming an outage", async () => {
    getStatusMock.mockResolvedValue(unavailableStatusFixture);

    render(<NetworkStatusPanel />);

    expect(
      await screen.findAllByText(
        "진단 근거가 부족하거나 Windows 앱에서만 확인할 수 있습니다.",
      ),
    ).toHaveLength(2);
    expect(screen.queryByText("호환되는 이상이 반복 확인되었습니다.")).not.toBeInTheDocument();
  });

  it("shows incident area and evidence without claiming an exact device cause", async () => {
    getStatusMock.mockResolvedValue(incidentStatusFixture);
    render(<NetworkStatusPanel />);

    expect(
      await screen.findByText("공유기 또는 로컬 연결 구간"),
    ).toBeInTheDocument();
    expect(screen.getByText("장애 확인")).toBeInTheDocument();
    expect(screen.getByText("기본 게이트웨이 응답 시간 초과")).toBeInTheDocument();
    expect(screen.queryByText(/케이블|포트|NIC/)).not.toBeInTheDocument();
  });

  it("shows a bounded reason and progress state for confirmed incidents", async () => {
    getStatusMock.mockResolvedValue(incidentStatusFixture);
    render(<NetworkStatusPanel />);

    expect(
      await screen.findByText(
        "공유기 또는 로컬 연결 구간에서 시간 초과 근거가 반복 확인되었습니다.",
      ),
    ).toBeInTheDocument();
    expect(screen.getByText("장애 근거 확인됨")).toBeInTheDocument();
    expect(screen.queryByText(/공유기 고장|케이블 불량|자동 복구/)).not.toBeInTheDocument();
  });

  it("shows a waiting progress state before the first observation", () => {
    render(<NetworkStatusPanel />);

    expect(screen.getByText("첫 확인 대기")).toBeInTheDocument();
    expect(
      screen.getAllByText("진단 근거가 부족하거나 Windows 앱에서만 확인할 수 있습니다."),
    ).toHaveLength(2);
  });

  it("shows the observation and latest evidence timestamps and details", async () => {
    getStatusMock.mockResolvedValue(normalStatusFixture);
    render(<NetworkStatusPanel />);

    expect(await screen.findByText("정상")).toBeInTheDocument();
    expectAreaLabel("해당 없음");
    expect(screen.getByText("최근 확인된 주요 구간이 정상입니다.")).toBeInTheDocument();
    expect(screen.getByText("기본 확인 완료")).toBeInTheDocument();
    expect(screen.getByText("192.168.0.1 응답")).toBeInTheDocument();
    expect(screen.getByText("connecttest.txt 응답 일치")).toBeInTheDocument();
    expect(screen.getByText("HTTP 204")).toBeInTheDocument();
    expect(
      document.querySelector('time[datetime="2026-06-23T09:00:04Z"]'),
    ).toBeInTheDocument();
    expect(
      document.querySelector('time[datetime="2026-06-23T09:00:03Z"]'),
    ).toBeInTheDocument();
  });

  it("shows the diagnostic path from PC to external connectivity", async () => {
    getStatusMock.mockResolvedValue(incidentStatusFixture);

    render(<NetworkStatusPanel />);

    expect(await screen.findByLabelText("구간별 진단 경로")).toBeInTheDocument();
    expect(screen.getByTestId("path-pc")).toHaveTextContent("PC/어댑터");
    expect(screen.getByTestId("path-pc")).toHaveTextContent("확인하지 않음");
    expect(screen.getByTestId("path-route")).toHaveTextContent("IPv4/Route");
    expect(screen.getByTestId("path-gateway")).toHaveTextContent("게이트웨이");
    expect(screen.getByTestId("path-gateway")).toHaveTextContent("시간 초과");
    expect(screen.getByTestId("path-dns")).toHaveTextContent("DNS");
    expect(screen.getByTestId("path-external")).toHaveTextContent("외부 연결");
  });

  it("renders latest evidence rows for PC, route, gateway, DNS, Microsoft and Google", async () => {
    getStatusMock.mockResolvedValue(normalStatusFixture);

    render(<NetworkStatusPanel />);

    await screen.findByText("정상");
    for (const testId of [
      "evidence-pc",
      "evidence-route",
      "evidence-gateway",
      "evidence-dns",
      "evidence-microsoft",
      "evidence-google",
    ]) {
      expect(screen.getByTestId(testId)).toBeInTheDocument();
    }
    expect(screen.getByTestId("evidence-pc")).toHaveTextContent("Ethernet adapter 감지");
    expect(screen.getByTestId("evidence-route")).toHaveTextContent("default route 선택됨");
    expect(screen.getByTestId("evidence-gateway")).toHaveTextContent("3ms");
    expect(screen.getByTestId("evidence-dns")).toHaveTextContent("5ms");
    expect(screen.queryByText("network health score")).not.toBeInTheDocument();
  });

  it("announces polling changes as a polite atomic status", async () => {
    getStatusMock.mockResolvedValue(normalStatusFixture);
    render(<NetworkStatusPanel />);

    const currentStatus = await screen.findByRole("status");
    expect(currentStatus).toHaveTextContent("정상");
    expect(currentStatus).toHaveAttribute("aria-live", "polite");
    expect(currentStatus).toHaveAttribute("aria-atomic", "true");
  });

  it("renders stable gateway, DNS, Microsoft and Google rows when not checked", async () => {
    getStatusMock.mockResolvedValue({
      ...normalStatusFixture,
      evidence: [],
    });
    render(<NetworkStatusPanel />);

    await screen.findByText("정상");
    for (const testId of [
      "evidence-gateway",
      "evidence-dns",
      "evidence-microsoft",
      "evidence-google",
    ]) {
      expect(screen.getByTestId(testId)).toHaveTextContent("확인하지 않음");
    }
  });

  it("maps evidence outcomes to direct Korean text and aggregates DNS conservatively", async () => {
    getStatusMock.mockResolvedValue({
      ...normalStatusFixture,
      evidence: [
        { source: "gateway", status: "success", checkedAt: null, durationMs: null, detail: null },
        { source: "dns_microsoft", status: "success", checkedAt: null, durationMs: null, detail: null },
        { source: "dns_google", status: "timeout", checkedAt: null, durationMs: null, detail: null },
        { source: "http_microsoft", status: "failure", checkedAt: null, durationMs: null, detail: null },
        { source: "http_google", status: "unavailable", checkedAt: null, durationMs: null, detail: null },
      ],
    });
    render(<NetworkStatusPanel />);

    await screen.findByText("정상");
    expect(screen.getByTestId("evidence-gateway")).toHaveTextContent("성공");
    expect(screen.getByTestId("evidence-dns")).toHaveTextContent("시간 초과");
    expect(screen.getByTestId("evidence-microsoft")).toHaveTextContent("실패");
    expect(screen.getByTestId("evidence-google")).toHaveTextContent("확인 불가");
  });

  it("shows a command error message as unavailable", async () => {
    getStatusMock.mockRejectedValue(new Error("status command failed"));
    render(<NetworkStatusPanel />);

    const errorDetail = await screen.findByText("status command failed");
    expect(errorDetail).toBeVisible();
    expect(errorDetail).not.toHaveAttribute("role", "alert");
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
    expect(screen.getByText("확인 불가", { selector: "strong" })).toBeInTheDocument();
  });

  it("mutes an error status even when the last lifecycle was incident", async () => {
    getStatusMock.mockResolvedValue({
      ...incidentStatusFixture,
      availability: "error",
      error: {
        stage: "runtime",
        code: "runtime_failed",
        message: "runtime stopped",
        nativeCode: null,
      },
    });
    render(<NetworkStatusPanel />);

    const statusLabel = await screen.findByText("확인 불가", {
      selector: "strong",
    });
    const panel = statusLabel.closest("section");
    expect(panel).toHaveClass(styles["state-unknown"]);
    expect(panel).not.toHaveClass(styles["state-incident"]);
  });
});
