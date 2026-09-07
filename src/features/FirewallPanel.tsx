import { useEffect, useRef, useState } from "react";
import { request } from "../lib/bridge";
import { useText } from "../lib/i18n";
import { Modal, ModalFooter, Notice } from "../components/ui";

type FirewallStatus = {
  platform: "macos" | "windows";
  profiles: { name: string; enabled: boolean }[];
};
export function FirewallPanel() {
  const t = useText();
  const [status, setStatus] = useState<FirewallStatus | null>(null);
  const [busy, setBusy] = useState(false);
  const [confirming, setConfirming] = useState(false);
  const [accepted, setAccepted] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [changed, setChanged] = useState(false);
  const sequence = useRef(0);
  const changing = useRef(false);
  function explain(e: unknown) {
    if (e === "firewall_policy_blocked") return t("firewallPolicyBlocked");
    if (e === "firewall_unavailable" || e === "firewall_unsupported")
      return t("firewallUnavailable");
    if (e === "firewall_busy") return t("firewallBusy");
    return t("firewallFailed");
  }
  useEffect(() => {
    let alive = true;
    function read() {
      if (changing.current) return;
      const ticket = ++sequence.current;
      request<FirewallStatus>("get_firewall_status")
        .then((s) => {
          if (alive && ticket === sequence.current) {
            setStatus(s);
            setChanged(false);
          }
        })
        .catch(() => {
          if (alive && ticket === sequence.current) setStatus(null);
        });
    }
    read();
    window.addEventListener("focus", read);
    return () => {
      alive = false;
      window.removeEventListener("focus", read);
    };
  }, []);
  async function refresh() {
    if (changing.current) return;
    changing.current = true;
    ++sequence.current;
    setBusy(true);
    setError(null);
    setChanged(false);
    try {
      setStatus(await request<FirewallStatus>("get_firewall_status"));
    } catch (e) {
      setStatus(null);
      setError(explain(e));
    } finally {
      changing.current = false;
      setBusy(false);
    }
  }
  async function change(enabled: boolean) {
    if (changing.current) return;
    changing.current = true;
    ++sequence.current;
    setBusy(true);
    setError(null);
    setChanged(false);
    try {
      const result = await request<FirewallStatus>("set_firewall_enabled", {
        enabled,
        confirmed: !enabled && accepted,
      });
      setStatus(result);
      setChanged(true);
      setConfirming(false);
      setAccepted(false);
    } catch (e) {
      setError(explain(e));
      // Commands can partially apply before a policy/error intervenes. Read the real state.
      try {
        setStatus(await request<FirewallStatus>("get_firewall_status"));
      } catch {
        setStatus(null);
      }
    } finally {
      changing.current = false;
      setBusy(false);
    }
  }
  return (
    <section className="firewall-panel" aria-labelledby="firewall-title">
      <h2 id="firewall-title">{t("firewallTitle")}</h2>
      <p>{t("firewallScope")}</p>
      <div role="status" aria-live="polite">
        {status ? (
          <ul className="firewall-profiles">
            {status.profiles.map((p) => (
              <li key={p.name}>
                <strong>{p.name}</strong>
                <span>{t(p.enabled ? "firewallOn" : "firewallOff")}</span>
              </li>
            ))}
          </ul>
        ) : (
          <p>{t("firewallUnavailable")}</p>
        )}
        {changed && <p>{t("firewallVerified")}</p>}
      </div>
      <p className="muted">{t("firewallPersistence")}</p>
      {error && !confirming && <Notice kind="error">{error}</Notice>}
      <div className="firewall-actions">
        <button type="button" onClick={() => void refresh()} disabled={busy}>
          {t("refresh")}
        </button>
        <button
          type="button"
          onClick={() => void change(true)}
          disabled={busy || !status || status.profiles.every((p) => p.enabled)}
        >
          {t("firewallEnable")}
        </button>
        <button
          type="button"
          onClick={() => {
            setConfirming(true);
            setAccepted(false);
            setError(null);
          }}
          disabled={busy || !status || status.profiles.every((p) => !p.enabled)}
        >
          {t("firewallDisable")}
        </button>
      </div>
      {busy && <p role="status">{t("firewallWaiting")}</p>}
      {confirming && (
        <Modal
          title={t("firewallDisable")}
          busy={busy}
          error={error}
          onClose={() => setConfirming(false)}
        >
          <form
            onSubmit={(e) => {
              e.preventDefault();
              if (accepted) void change(false);
            }}
          >
            <Notice>{t("firewallWarning")}</Notice>
            <label className="checkbox-row">
              <input
                type="checkbox"
                checked={accepted}
                onChange={(e) => setAccepted(e.target.checked)}
                disabled={busy}
              />
              {t("firewallAccept")}
            </label>
            <ModalFooter
              onClose={() => setConfirming(false)}
              busy={busy}
              disabled={!accepted}
              danger
              submit={t("firewallDisable")}
            />
          </form>
        </Modal>
      )}
    </section>
  );
}
