import { useState } from "react";
import type { Peer } from "../lib/types";
import { request } from "../lib/bridge";
import { errorText, useText } from "../lib/i18n";
import { Modal } from "../components/ui";

type ProbeResult = "reachable" | "refused" | "timed_out" | "unreachable";
export function PeerDialog({
  peer,
  onClose,
  onError,
  onCopy,
}: {
  peer: Peer | undefined;
  onClose: () => void;
  onError: (error: unknown) => void;
  onCopy: (ip: string) => void;
}) {
  const t = useText();
  const [error, setError] = useState<string | null>(null);
  const [port, setPort] = useState("");
  const [checking, setChecking] = useState(false);
  const [result, setResult] = useState<ProbeResult | null>(null);
  async function check() {
    if (!peer) return;
    setChecking(true);
    setError(null);
    setResult(null);
    try {
      setResult(
        await request<ProbeResult>("probe_service", {
          peerId: peer.id,
          port: Number(port),
        }),
      );
    } catch (error) {
      setError(errorText(error));
      onError(error);
    } finally {
      setChecking(false);
    }
  }
  return (
    <Modal title={t("peerDetail")} onClose={onClose} error={error}>
      {peer ? (
        <>
          <dl className="facts">
            <div>
              <dt>{t("nickname")}</dt>
              <dd>{peer.nickname}</dd>
            </div>
            <div>
              <dt>{t("virtualAddress")}</dt>
              <dd>{peer.virtual_ip ?? t("unavailable")}</dd>
            </div>
            <div>
              <dt>{t("policy")}</dt>
              <dd>{t(peer.path)}</dd>
            </div>
            <div>
              <dt>{t("latency")}</dt>
              <dd>
                {peer.latency_ms === null
                  ? t("unavailable")
                  : `${peer.latency_ms} ms`}
              </dd>
            </div>
          </dl>
          <p>{t("selfReported")}</p>
          {peer.virtual_ip && (
            <button onClick={() => onCopy(peer.virtual_ip!)}>
              {t("copyIp")}
            </button>
          )}
          <form
            className="service-probe"
            onSubmit={(event) => {
              event.preventDefault();
              void check();
            }}
          >
            <h3>{t("checkService")}</h3>
            <p>{t("serviceHint")}</p>
            <label>
              {t("tcpPort")}
              <input
                type="number"
                min="1"
                max="65535"
                step="1"
                required
                value={port}
                disabled={checking}
                onChange={(event) => {
                  setPort(event.target.value);
                  setResult(null);
                }}
              />
            </label>
            <button disabled={checking || !peer.virtual_ip || !port}>
              {checking ? t("checkingService") : t("checkService")}
            </button>
            <p role="status">
              {result
                ? t(
                    (
                      {
                        reachable: "serviceReachable",
                        refused: "serviceRefused",
                        timed_out: "serviceTimedOut",
                        unreachable: "serviceUnreachable",
                      } as const
                    )[result],
                  )
                : ""}
            </p>
          </form>
        </>
      ) : (
        <p>{t("peerGone")}</p>
      )}
    </Modal>
  );
}
