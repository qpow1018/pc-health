export type ProbeStatus =
  | "success"
  | "timeout"
  | "not_run"
  | "unsupported"
  | "error";

export interface ProbeError {
  stage: string;
  code: string;
  message: string;
  nativeCode: number | null;
}

export interface AdapterSnapshot {
  name: string;
  friendlyName: string;
  interfaceIndex: number;
  ifType: number;
  operationalStatus: string;
  macAddress: string | null;
  ipv4Addresses: string[];
}

export interface RouteSnapshot {
  interfaceIndex: number;
  gateway: string;
  routeMetric: number;
  interfaceMetric: number;
  combinedMetric: number;
  adapterIsEthernet: boolean;
  adapterIsUp: boolean;
}

export interface GatewayCheck {
  status: ProbeStatus;
  durationMs: number;
  replyAddress: string | null;
  roundTripMs: number | null;
  error: ProbeError | null;
}

export interface DnsCheck {
  hostname: string;
  status: ProbeStatus;
  durationMs: number;
  addresses: string[];
  error: ProbeError | null;
}

export interface HttpCheck {
  url: string;
  status: ProbeStatus;
  durationMs: number;
  statusCode: number | null;
  bodyMatches: boolean | null;
  error: ProbeError | null;
}

export interface NetworkProbeSnapshot {
  collectedAt: string;
  collector: string;
  durationMs: number;
  adapters: AdapterSnapshot[];
  defaultRoutes: RouteSnapshot[];
  selectedRoute: RouteSnapshot | null;
  gatewayCheck: GatewayCheck;
  dnsChecks: DnsCheck[];
  httpChecks: HttpCheck[];
  errors: ProbeError[];
}
