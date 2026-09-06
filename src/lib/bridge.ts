import { invoke, isTauri } from "@tauri-apps/api/core";
export const testMode = import.meta.env.MODE === "test";
export function nativeAvailable() {
  return (
    isTauri() ||
    (testMode && typeof window.__QUICKLAN_TEST_INVOKE === "function")
  );
}
export async function request<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  if (testMode && window.__QUICKLAN_TEST_INVOKE)
    return window.__QUICKLAN_TEST_INVOKE(command, args) as Promise<T>;
  if (!isTauri()) throw "desktop_required";
  return invoke<T>(command, args);
}
declare global {
  interface Window {
    __QUICKLAN_TEST_INVOKE?: (
      command: string,
      args?: Record<string, unknown>,
    ) => Promise<unknown>;
  }
}
