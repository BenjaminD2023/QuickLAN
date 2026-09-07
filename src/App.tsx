import { useCallback, useEffect, useState } from "react";
import type {
  Bootstrap,
  JoinPreview,
  Peer,
  Policy,
  SavedNetwork,
  AppView,
  Preferences,
} from "./lib/types";
import { emptyView } from "./lib/types";
import { request, nativeAvailable, testMode, isAndroid } from "./lib/bridge";
import { LocaleContext, errorText, useText } from "./lib/i18n";
import { BottomNav, NetworkSwitcher, Sidebar } from "./components/Sidebar";
import type { Page } from "./components/Sidebar";
import { Notice } from "./components/ui";
import { PeerDialog } from "./features/PeerDialog";
import { NetworkView, Welcome } from "./features/NetworkView";
import {
  CreateDialog,
  InviteDialog,
  JoinDialog,
  NetworkActions,
  SettingsDialog,
} from "./features/NetworkDialogs";
import {
  AboutPage,
  DiagnosticsPage,
  HelpPage,
  PreferencesPage,
} from "./features/UtilityPages";

type Dialog = "create" | "join" | "invite" | "actions" | "policy" | null;
export default function App() {
  const [view, setView] = useState<AppView>(emptyView);
  return (
    <LocaleContext.Provider value={view.saved.preferences.language}>
      <Application view={view} setView={setView} />
    </LocaleContext.Provider>
  );
}
function Application({
  view,
  setView,
}: {
  view: AppView;
  setView: (v: AppView) => void;
}) {
  const t = useText();
  const [page, setPage] = useState<Page>("networks");
  const [selected, setSelected] = useState<string>();
  const [dialog, setDialog] = useState<Dialog>(null);
  const [preview, setPreview] = useState<JoinPreview | null>(null);
  const [peer, setPeer] = useState<Peer | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [toast, setToast] = useState<string | null>(null);
  const [loadFailed, setLoadFailed] = useState(false);
  const [loaded, setLoaded] = useState(false);
  const native = nativeAvailable();
  const network =
    view.saved.networks.find((n) => n.id === selected) ??
    view.saved.networks[0];
  const showError = useCallback((e: unknown) => setError(errorText(e)), []);
  const refresh = useCallback(async () => {
    const next = await request<AppView>("get_state");
    setView(next);
    setLoadFailed(false);
    setLoaded(true);
  }, [setView]);
  useEffect(() => {
    if (!native) {
      setLoaded(true);
      return;
    }
    let active = true;
    const poll = () =>
      request<AppView>("get_state")
        .then((v) => {
          if (active) {
            setView(v);
            setLoaded(true);
            setLoadFailed(false);
          }
        })
        .catch((e) => {
          if (active) {
            showError(e);
            setLoaded(true);
            setLoadFailed(true);
          }
        });
    void poll();
    const id = setInterval(() => void poll(), 3000);
    return () => {
      active = false;
      clearInterval(id);
    };
  }, [native, setView, showError]);
  useEffect(() => {
    const theme = view.saved.preferences.theme;
    document.documentElement.dataset.theme = theme;
    document.documentElement.lang = view.saved.preferences.language;
  }, [view.saved.preferences]);
  useEffect(() => {
    document.documentElement.classList.toggle("android", isAndroid());
    return () => document.documentElement.classList.remove("android");
  }, []);
  useEffect(() => {
    const seen = new WeakSet<Event>();
    function onBack(event: Event) {
      if (seen.has(event)) return;
      seen.add(event);
      const openDialog = document.querySelector("dialog[open]");
      if (openDialog) {
        event.preventDefault();
        if (!busy)
          openDialog.dispatchEvent(new Event("cancel", { cancelable: true }));
        return;
      }
      if (page !== "networks") {
        event.preventDefault();
        setPage("networks");
        setError(null);
      }
    }
    window.addEventListener("quicklan:back", onBack);
    document.addEventListener("quicklan:back", onBack);
    return () => {
      window.removeEventListener("quicklan:back", onBack);
      document.removeEventListener("quicklan:back", onBack);
    };
  }, [busy, page]);
  useEffect(() => {
    if (!toast) return;
    const id = setTimeout(() => setToast(null), 3500);
    return () => clearTimeout(id);
  }, [toast]);
  async function act(work: () => Promise<unknown>, after?: () => void) {
    if (busy) return;
    setBusy(true);
    setError(null);
    try {
      await work();
      if (native) await refresh();
      after?.();
    } catch (e) {
      showError(e);
    } finally {
      setBusy(false);
    }
  }
  function open(d: Dialog) {
    setDialog(d);
    setPreview(null);
    setError(null);
  }
  function close() {
    if (busy) return;
    if (dialog === "join") void request("cancel_invitation").catch(() => {});
    setDialog(null);
    setPeer(null);
    setPreview(null);
    setError(null);
  }
  function chosen(n: SavedNetwork) {
    setSelected(n.id);
    setPage("networks");
    setDialog(null);
    setPreview(null);
    setToast(t("saved"));
  }
  function create(data: {
    label: string;
    nickname: string;
    subnet: string;
    policy: Policy;
    bootstrap: Bootstrap[];
    assistanceAccepted: boolean;
  }) {
    void act(async () => {
      const n = await request<SavedNetwork>("create_network", {
        nickname: data.nickname,
        label: data.label,
        subnet: data.subnet,
        policy: data.policy,
        bootstrap: data.bootstrap,
        assistanceAccepted: data.assistanceAccepted,
      });
      chosen(n);
      await refresh();
      await request("connect_network", { id: n.id });
    });
  }
  const common = { onClose: close, busy, error };
  const copy = (command: string, args?: Record<string, unknown>) =>
    void act(
      () => request(command, args),
      () => setToast(t("copied")),
    );
  return (
    <div className={isAndroid() ? "app-shell android" : "app-shell"}>
      {testMode && (
        <div className="test-banner">TEST SIMULATION — no networking</div>
      )}
      <Sidebar
        networks={view.saved.networks}
        selected={network?.id}
        page={page}
        onPage={(p) => {
          setPage(p);
          setError(null);
        }}
        onSelect={(id) => {
          setSelected(id);
          setPage("networks");
          setError(null);
        }}
        onCreate={() => open("create")}
        onJoin={() => open("join")}
        onQuit={() => void act(() => request("quit_app"))}
      />
      <main>
        <div className="main-scroll">
          {isAndroid() && page === "networks" && view.saved.networks.length > 0 && (
            <NetworkSwitcher
              networks={view.saved.networks}
              selected={network?.id}
              onSelect={(id) => {
                setSelected(id);
                setPage("networks");
                setError(null);
              }}
              onCreate={() => open("create")}
              onJoin={() => open("join")}
            />
          )}
          {!native && (
            <div className="runtime-notice">
              <Notice kind="info" title={t("browserTitle")}>
                {t("browserBody")}
              </Notice>
            </div>
          )}
          {loadFailed && (
            <div className="runtime-notice">
              <Notice kind="error" title={t("storageTitle")}>
                {error}
                <button
                  className="text-button"
                  onClick={() => void act(refresh)}
                >
                  {t("retry")}
                </button>
              </Notice>
            </div>
          )}
          {error && !dialog && !peer && !loadFailed && (
            <div className="runtime-notice">
              <Notice kind="error">{error}</Notice>
            </div>
          )}
          {!loaded ? (
            <div className="loading" role="status">
              {t("working")}
            </div>
          ) : page === "networks" ? (
            network ? (
              <NetworkView
                network={network}
                view={view}
                busy={busy}
                onConnect={() =>
                  void act(() => request("connect_network", { id: network.id }))
                }
                onDisconnect={() =>
                  void act(() => request("disconnect_network"))
                }
                onInvite={() => open("invite")}
                onMore={() => open("actions")}
                onDiagnostics={() => setPage("diagnostics")}
                onPolicy={() => open("policy")}
                onPeer={setPeer}
                onCopy={(ip) => copy("copy_virtual_ip", { ip })}
              />
            ) : (
              <Welcome
                onCreate={() => open("create")}
                onJoin={() => open("join")}
                onHelp={() => setPage("help")}
              />
            )
          ) : page === "preferences" ? (
            <PreferencesPage
              key={view.saved.preferences.language}
              preferences={view.saved.preferences}
              busy={busy}
              onSave={(preferences: Preferences) =>
                void act(
                  () => request("save_preferences", { preferences }),
                  () => setToast(t("saved")),
                )
              }
              onQuit={() => void act(() => request("quit_app"))}
            />
          ) : page === "diagnostics" ? (
            <DiagnosticsPage
              view={view}
              onCopy={() => copy("copy_diagnostics")}
              onError={showError}
            />
          ) : page === "about" ? (
            <AboutPage available={view.helper.connection_enabled} />
          ) : (
            <HelpPage />
          )}
        </div>
        <footer className="app-footer">
          {t(view.helper.connection_enabled ? "nativeFooter" : "engineering")}
          {isAndroid() && (
            <button
              type="button"
              className="text-button"
              onClick={() => void act(() => request("quit_app"))}
            >
              {t("quit")}
            </button>
          )}
        </footer>
      </main>
      {isAndroid() && (
        <BottomNav
          page={page}
          onPage={(p) => {
            setPage(p);
            setError(null);
          }}
        />
      )}
      {toast && (
        <div className="toast" role="status">
          {toast}
        </div>
      )}
      {dialog === "create" && (
        <CreateDialog
          {...common}
          preferences={view.saved.preferences}
          onCreate={create}
        />
      )}{" "}
      {dialog === "join" && (
        <JoinDialog
          {...common}
          preview={preview}
          onReview={(token) =>
            void act(async () =>
              setPreview(
                await request<JoinPreview>("preview_invitation", { token }),
              ),
            )
          }
          onAccept={(ticket, consent) =>
            void act(async () => {
              const n = await request<SavedNetwork>("accept_invitation", {
                ticket,
                trusted: true,
                assistanceAccepted: consent,
              });
              chosen(n);
              await refresh();
              await request("connect_network", { id: n.id });
            })
          }
        />
      )}{" "}
      {dialog === "invite" && network && (
        <InviteDialog
          {...common}
          network={network}
          onCopy={() => copy("copy_invitation", { id: network.id })}
        />
      )}{" "}
      {dialog === "actions" && network && (
        <NetworkActions
          {...common}
          network={network}
          onAction={(kind, label) =>
            void act(
              async () => {
                if (kind === "rename")
                  await request("rename_network", { id: network.id, label });
                if (kind === "forget")
                  await request("forget_network", { id: network.id });
                if (kind === "replace")
                  chosen(
                    await request<SavedNetwork>("replace_credentials", {
                      id: network.id,
                    }),
                  );
              },
              () => setDialog(null),
            )
          }
        />
      )}{" "}
      {dialog === "policy" && network && (
        <SettingsDialog
          {...common}
          network={network}
          onSave={(policy, bootstrap, assistanceAccepted) =>
            void act(
              () =>
                request("update_settings", {
                  id: network.id,
                  policy,
                  bootstrap,
                  assistanceAccepted,
                }),
              () => {
                setDialog(null);
                setToast(t("saved"));
              },
            )
          }
        />
      )}
      {peer && (
        <PeerDialog
          key={`${peer.id}:${view.connection.peers.find((item) => item.id === peer.id)?.virtual_ip ?? "gone"}`}
          peer={view.connection.peers.find((item) => item.id === peer.id)}
          onClose={() => setPeer(null)}
          onError={showError}
          onCopy={(ip) => copy("copy_virtual_ip", { ip })}
        />
      )}
    </div>
  );
}
