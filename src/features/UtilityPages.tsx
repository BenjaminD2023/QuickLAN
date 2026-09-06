import { useEffect, useState } from "react";
import { Copy, RefreshCw, Shield, Network } from "lucide-react";
import type { AppView, Diagnostics, Preferences } from "../lib/types";
import { useText } from "../lib/i18n";
import { BRAND } from "../lib/brand";
import { request } from "../lib/bridge";
import { Notice } from "../components/ui";
export function PreferencesPage({
  preferences,
  busy,
  onSave,
}: {
  preferences: Preferences;
  busy: boolean;
  onSave: (p: Preferences) => void;
}) {
  const t = useText();
  const [draft, setDraft] = useState(preferences);
  return (
    <section className="utility-page">
      <header className="page-header">
        <h1>{t("preferences")}</h1>
      </header>
      <form
        className="preferences-form"
        onSubmit={(e) => {
          e.preventDefault();
          onSave(draft);
        }}
      >
        <section>
          <h2>{t("nickname")}</h2>
          <input
            aria-label={t("nickname")}
            value={draft.nickname}
            onChange={(e) => setDraft({ ...draft, nickname: e.target.value })}
            maxLength={64}
            required
          />
        </section>
        <section>
          <h2>{t("appearance")}</h2>
          <label>
            {t("theme")}
            <select
              aria-label={t("theme")}
              value={draft.theme}
              onChange={(e) =>
                setDraft({
                  ...draft,
                  theme: e.target.value as Preferences["theme"],
                })
              }
            >
              <option value="system">{t("system")}</option>
              <option value="light">{t("light")}</option>
              <option value="dark">{t("dark")}</option>
            </select>
          </label>
          <label>
            {t("language")}
            <select
              aria-label={t("language")}
              value={draft.language}
              onChange={(e) =>
                setDraft({
                  ...draft,
                  language: e.target.value as Preferences["language"],
                })
              }
            >
              <option value="en">English</option>
              <option value="zh-CN">简体中文</option>
            </select>
          </label>
        </section>
        <section>
          <h2>{t("behavior")}</h2>
          <p>{t("closeBehavior")}</p>
        </section>
        <section>
          <h2>{t("privacy")}</h2>
          <p>{t("privacyBody")}</p>
        </section>
        <button className="primary" disabled={busy}>
          {busy ? t("working") : t("savePreferences")}
        </button>
      </form>
    </section>
  );
}
export function DiagnosticsPage({
  view,
  onCopy,
  onError,
}: {
  view: AppView;
  onCopy: () => void;
  onError: (e: unknown) => void;
}) {
  const t = useText();
  const [report, setReport] = useState<Diagnostics | null>(null);
  const [loading, setLoading] = useState(false);
  async function refresh() {
    setLoading(true);
    try {
      setReport(await request<Diagnostics>("get_diagnostics"));
    } catch (e) {
      onError(e);
    } finally {
      setLoading(false);
    }
  }
  useEffect(() => {
    let alive = true;
    request<Diagnostics>("get_diagnostics")
      .then((r) => {
        if (alive) setReport(r);
      })
      .catch((e) => {
        if (alive) onError(e);
      });
    return () => {
      alive = false;
    };
  }, [onError]);
  return (
    <section className="utility-page">
      <header className="page-header">
        <h1>{t("diagnosticsTitle")}</h1>
        <button
          className="icon-button"
          onClick={() => void refresh()}
          disabled={loading}
          aria-label={t("refresh")}
        >
          <RefreshCw />
        </button>
      </header>
      <Notice
        kind={view.helper.connection_enabled ? "info" : "warning"}
        title={t(
          view.helper.connection_enabled ? "helperReady" : "helperTitle",
        )}
      >
        {t(view.helper.connection_enabled ? "helperReadyBody" : "helperBody")}
      </Notice>
      <h2 className="section-title">{t("releaseChecks")}</h2>
      <ul className="checklist">
        {view.helper.release_gaps.length ? (
          view.helper.release_gaps.map((g) => <li key={g}>{g}</li>)
        ) : (
          <li>{t("nativeFooter")}</li>
        )}
      </ul>
      <p>{t("diagnosticsHint")}</p>
      <pre
        className="diagnostic-preview"
        tabIndex={0}
        aria-label={t("diagnosticsTitle")}
      >
        {report ? JSON.stringify(report, null, 2) : t("unavailable")}
      </pre>
      <button onClick={onCopy} disabled={!report}>
        <Copy />
        {t("copyReport")}
      </button>
    </section>
  );
}
export function AboutPage({ available }: { available: boolean }) {
  const t = useText();
  return (
    <section className="utility-page about-page">
      <Network className="about-icon" size={46} />
      <h1>{BRAND.name}</h1>
      <p className="version">
        {BRAND.version} · {BRAND.core}
      </p>
      <h2>{t("aboutTitle")}</h2>
      <p>{t("aboutBody")}</p>
      <Notice
        kind={available ? "info" : "warning"}
        title={t(available ? "helperReady" : "engineering")}
      >
        {t(available ? "helperReadyBody" : "aboutGap")}
      </Notice>
      <p>{t("aboutEvidence")}</p>
      <div className="license-copy">
        <h2>{t("about")}</h2>
        <p>{t("wrapperLicense")}</p>
        <p>{t("coreLicense")}</p>
        <p className="muted">Tauri · React · TypeScript · Rust · lucide</p>
      </div>
      <p>
        <Shield className="inline-icon" />
        {t("privacyBody")}
      </p>
    </section>
  );
}
export function HelpPage() {
  const t = useText();
  return (
    <section className="utility-page help-page">
      <header className="page-header">
        <h1>{t("gameTitle")}</h1>
      </header>
      <p className="lead">{t("gameIntro")}</p>
      <ol className="guide-steps">
        {(["gameOne", "gameTwo", "gameThree", "gameFour"] as const).map(
          (k, i) => (
            <li key={k}>
              <span>{i + 1}</span>
              <p>{t(k)}</p>
            </li>
          ),
        )}
      </ol>
      <Notice>{t("gameGap")}</Notice>
      <h2 className="section-title">{t("troubleshooting")}</h2>
      <p>{t("troubleBody")}</p>
    </section>
  );
}
