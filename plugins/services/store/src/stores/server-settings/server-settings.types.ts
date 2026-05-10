export interface ServerEndpointState {
  defaultBaseUrl: string;
  customBaseUrl?: string | null;
  effectiveBaseUrl: string;
  usingCustomBaseUrl: boolean;
}
