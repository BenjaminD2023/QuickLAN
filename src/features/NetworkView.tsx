import {
  Copy,
  UserRoundPlus,
  Shield,
  ChevronRight,
  MoreHorizontal,
  Monitor,
  Link,
  Power,
} from "lucide-react";
import type { AppView, Peer, SavedNetwork } from "../lib/types";
import { useText } from "../lib/i18n";
import { Notice } from "../components/ui";
export function Welcome({
  onCreate,
  onJoin,
  onHelp,
}: {
  onCreate: () => void;
  onJoin: () => void;
  onHelp: () => void;
}) {
  const t = useText();
  return (
    <div className="welcome">
      <div className="welcome-main">
        <h1>{t("headline")}</h1>
        <p className="intro">{t("intro")}</p>
        <div className="welcome-actions">
          <button className="primary" onClick={onCreate}>
            {t("createLong")}
          </button>
          <button onClick={onJoin}>{t("join")}</button>
        </div>
        <p className="welcome-note">
          {t("welcomeNote")}{" "}
          <button className="text-button" onClick={onHelp}>
            {t("learnMore")}
          </button>
        </p>
      </div>
    </div>
  );
}
export function NetworkView({
  network,
  view,
  busy,
  onConnect,
  onDisconnect,
  onInvite,
  onMore,
  onDiagnostics,
  onPolicy,
  onPeer,
  onCopy,
}: {
  network: SavedNetwork;
  view: AppView;
  busy: boolean;
  onConnect: () => void;
  onDisconnect: () => void;
  onInvite: () => void;
  onMore: () => void;
  onDiagnostics: () => void;
  onPolicy: () => void;
  onPeer: (p: Peer) => void;
  onCopy: (ip: string) => void;
}) {
  const t = useText();
  const current = view.connection.network_id === network.id;
  const connection = current ? view.connection : null;
  const phase = connection?.phase ?? "disconnected";
  const active = !["disconnected", "failed"].includes(phase);
  const peers = connection?.peers ?? [];
  const ip = connection?.virtual_ip;
  return (
    <section className="network-view">
      <header className="page-header">
        <div>
          <h1>{network.label}</h1>
          <div className={`connection-status ${phase}`} aria-live="polite">
            <span />
            {t(phase)}
          </div>
        </div>
        <div className="header-actions">
          <button onClick={onInvite}>
            <UserRoundPlus />
            {t("invite")}
          </button>
          <button
            className={active ? "" : "primary"}
            disabled={busy}
            onClick={active ? onDisconnect : onConnect}
          >
            {active ? <Power /> : <Link />}
            {active ? t("disconnect") : t("connect")}
          </button>
          <button
            className="icon-button"
            aria-label={t("more")}
            onClick={onMore}
          >
            <MoreHorizontal />
          </button>
        </div>
      </header>
      <div className="network-summary">
        <div className="address">
          <h2>{t("virtualAddress")}</h2>
          <div className="virtual-ip">
            {ip ?? "—"}
            {ip && (
              <button
                className="icon-button"
                onClick={() => onCopy(ip)}
                aria-label={t("copyIp")}
              >
                <Copy />
              </button>
            )}
          </div>
          <p>{ip ? network.subnet : t("availableAfter")}</p>
        </div>
        <div className="devices">
          <h2>
            {t("devices")}
            {peers.length > 0 && <span className="count">{peers.length}</span>}
          </h2>
          {peers.length === 0 ? (
            <div className="empty-peers">
              <strong>{t("noDevices")}</strong>
              <p>{t("noDevicesHint")}</p>
            </div>
          ) : (
            <ul className="peer-list">
              {peers.map((p) => (
                <li key={p.id}>
                  <button onClick={() => onPeer(p)} className="peer-row">
                    <Monitor />
                    <span>
                      <strong>{p.nickname}</strong>
                      <small>{p.virtual_ip ?? t("unavailable")}</small>
                    </span>
                    <span className="peer-path">{t(p.path)}</span>
                    <ChevronRight />
                  </button>
                </li>
              ))}
            </ul>
          )}
        </div>
      </div>
      <button className="policy-row" onClick={onPolicy}>
        <Shield />
        <span>
          <strong>{t("policy")}</strong>
          <small>
            {t(
              network.policy === "manual"
                ? "manualDetail"
                : network.policy === "direct_only"
                  ? "directOnlyDetail"
                  : "assistedDetail",
            )}
          </small>
          <small>
            {t(
              network.policy === "manual"
                ? "manual"
                : network.policy === "direct_only"
                  ? "directOnly"
                  : "assisted",
            )}
          </small>
        </span>
        <ChevronRight />
      </button>
      {!view.helper.connection_enabled && (
        <Notice title={t("helperTitle")}>
          <span>{t("helperBody")}</span>{" "}
          <button className="text-button" onClick={onDiagnostics}>
            {t("details")}
          </button>
        </Notice>
      )}
      {network.bootstrap.length === 0 && (
        <p className="subtle-note">{t("noEndpoints")}</p>
      )}
    </section>
  );
}
