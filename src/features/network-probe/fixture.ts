import type { NetworkProbeSnapshot } from "./types";

export const networkProbeFixture: NetworkProbeSnapshot = {
  collectedAt: "2026-06-22T00:00:00Z",
  collector: "windows-native",
  durationMs: 42,
  adapters: [
    {
      name: "ethernet",
      friendlyName: "Ethernet",
      interfaceIndex: 7,
      ifType: 6,
      operationalStatus: "up",
      macAddress: "00:11:22:33:44:55",
      ipv4Addresses: ["192.168.0.2"],
    },
  ],
  defaultRoutes: [
    {
      interfaceIndex: 7,
      gateway: "192.168.0.1",
      routeMetric: 10,
      interfaceMetric: 5,
      combinedMetric: 15,
      adapterIsEthernet: true,
      adapterIsUp: true,
    },
  ],
  selectedRoute: {
    interfaceIndex: 7,
    gateway: "192.168.0.1",
    routeMetric: 10,
    interfaceMetric: 5,
    combinedMetric: 15,
    adapterIsEthernet: true,
    adapterIsUp: true,
  },
  gatewayCheck: {
    status: "success",
    durationMs: 3,
    replyAddress: "192.168.0.1",
    roundTripMs: 1,
    error: null,
  },
  dnsChecks: [
    {
      hostname: "www.msftconnecttest.com",
      status: "success",
      durationMs: 4,
      addresses: ["13.107.4.52"],
      error: null,
    },
  ],
  httpChecks: [
    {
      url: "https://connectivitycheck.gstatic.com/generate_204",
      status: "success",
      durationMs: 12,
      statusCode: 204,
      bodyMatches: true,
      error: null,
    },
  ],
  errors: [],
};
