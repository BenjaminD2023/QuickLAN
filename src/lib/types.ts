export type Policy = "manual" | "assisted" | "direct_only";
export type Phase =
  | "disconnected"
  | "starting"
  | "joining"
  | "connected"
  | "reconnecting"
  | "stopping"
  | "failed";
export type PeerPath =
  | "local"
  | "direct"
  | "relayed"
  | "unreachable"
  | "unknown";
export interface Bootstrap {
  endpoint: string;
  operator: string;
}
export interface SavedNetwork {
  id: string;
  label: string;
  subnet: string;
  policy: Policy;
  bootstrap: Bootstrap[];
}
export interface Preferences {
  nickname: string;
  theme: "system" | "light" | "dark";
  language: "en" | "zh-CN";
  onboarding_complete: boolean;
}
export interface Peer {
  id: string;
  nickname: string;
  virtual_ip: string | null;
  path: PeerPath;
  latency_ms: number | null;
  identity_verified: false;
}
export interface Connection {
  phase: Phase;
  network_id: string | null;
  virtual_ip: string | null;
  peers: Peer[];
  error: string | null;
}
export interface AppView {
  saved: { networks: SavedNetwork[]; preferences: Preferences };
  connection: Connection;
  helper: {
    installed: boolean;
    connection_enabled: boolean;
    code: string;
    release_gaps: string[];
  };
}
export interface JoinPreview {
  ticket: string;
  network: SavedNetwork;
}
export interface Diagnostics {
  product: string;
  app_version: string;
  core_pin: string;
  platform: string;
  architecture: string;
  phase: Phase;
  peer_count: number;
  events: { sequence: number; phase: Phase; code: string | null }[];
  exclusion_notice: string;
}
export const emptyView: AppView = {
  saved: {
    networks: [],
    preferences: {
      nickname: "My computer",
      theme: "system",
      language: "en",
      onboarding_complete: false,
    },
  },
  connection: {
    phase: "disconnected",
    network_id: null,
    virtual_ip: null,
    peers: [],
    error: null,
  },
  helper: {
    installed: false,
    connection_enabled: false,
    code: "helper_unavailable",
    release_gaps: [],
  },
};
