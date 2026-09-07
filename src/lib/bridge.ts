import { invoke, isTauri } from "@tauri-apps/api/core";
export const testMode = import.meta.env.MODE === "test";
const MAX_PENDING = 24;
const REQUEST_TIMEOUT_MS = 30_000;
const CONSENT_TIMEOUT_MS = 300_000;
interface AndroidPending {
  resolve: (value: unknown) => void;
  reject: (error: string) => void;
  timer: number;
}
const pending = new Map<string, AndroidPending>();
let requestSeq = 0;
export function isAndroid() {
  return typeof window !== "undefined" &&
    typeof window.QuickLANAndroid?.postMessage === "function";
}
export function nativeAvailable() {
  return (
    isTauri() ||
    isAndroid() ||
    (testMode && typeof window.__QUICKLAN_TEST_INVOKE === "function")
  );
}
function settle(id: string, ok: boolean, value: unknown, error: unknown) {
  const item = pending.get(id);
  if (!item) return;
  pending.delete(id);
  clearTimeout(item.timer);
  if (ok) item.resolve(value);
  else item.reject(typeof error === "string" && error ? error : "core_failed");
}
function receive(payload: unknown) {
  try {
    const message =
      typeof payload === "string" ? (JSON.parse(payload) as unknown) : payload;
    if (!message || typeof message !== "object") return;
    const { id, ok, value, error } = message as {
      id?: unknown;
      ok?: unknown;
      value?: unknown;
      error?: unknown;
    };
    if (typeof id !== "string" || !id) return;
    settle(id, ok === true, value, error);
  } catch {
    /* ignore malformed native callbacks */
  }
}
function ensureReceiver() {
  window.__QUICKLAN_ANDROID_RECEIVE = receive;
}
function androidRequest<T>(
  command: string,
  args: Record<string, unknown> = {},
): Promise<T> {
  ensureReceiver();
  if (typeof window.QuickLANAndroid?.postMessage !== "function")
    return Promise.reject("helper_unavailable");
  if (pending.size >= MAX_PENDING) return Promise.reject("busy");
  const id = `ql-${Date.now()}-${++requestSeq}`;
  return new Promise<T>((resolve, reject) => {
    const timer = window.setTimeout(
      () => settle(id, false, undefined, "request_timeout"),
      command === "connect_network" ? CONSENT_TIMEOUT_MS : REQUEST_TIMEOUT_MS,
    );
    pending.set(id, {
      resolve: (value) => resolve(value as T),
      reject,
      timer,
    });
    try {
      window.QuickLANAndroid!.postMessage(
        JSON.stringify({ id, command, args }),
      );
    } catch {
      settle(id, false, undefined, "helper_unavailable");
    }
  });
}
export async function request<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  if (testMode && window.__QUICKLAN_TEST_INVOKE)
    return window.__QUICKLAN_TEST_INVOKE(command, args) as Promise<T>;
  if (isAndroid()) return androidRequest<T>(command, args);
  if (!isTauri()) throw "desktop_required";
  return invoke<T>(command, args);
}
declare global {
  interface Window {
    QuickLANAndroid?: { postMessage: (json: string) => void };
    __QUICKLAN_ANDROID_RECEIVE?: (payload: unknown) => void;
    __QUICKLAN_TEST_INVOKE?: (
      command: string,
      args?: Record<string, unknown>,
    ) => Promise<unknown>;
  }
}
