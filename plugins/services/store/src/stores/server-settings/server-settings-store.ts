import { invoke } from '@/lib/tauri';
import type { ServerEndpointState } from './server-settings.types';

const TAURI_COMMANDS = {
  GET_SERVER_ENDPOINT_STATE: 'get_server_endpoint_state',
  SET_SERVER_BASE_URL: 'set_server_base_url',
  RESET_SERVER_BASE_URL: 'reset_server_base_url',
} as const;

export async function getServerEndpointState(): Promise<ServerEndpointState> {
  return invoke<ServerEndpointState>(TAURI_COMMANDS.GET_SERVER_ENDPOINT_STATE);
}

export async function setServerBaseUrl(baseUrl: string): Promise<ServerEndpointState> {
  return invoke<ServerEndpointState>(TAURI_COMMANDS.SET_SERVER_BASE_URL, { baseUrl });
}

export async function resetServerBaseUrl(): Promise<ServerEndpointState> {
  return invoke<ServerEndpointState>(TAURI_COMMANDS.RESET_SERVER_BASE_URL);
}
