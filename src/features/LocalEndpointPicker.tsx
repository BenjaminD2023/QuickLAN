import { useState } from "react";
import { request } from "../lib/bridge";
import { useText } from "../lib/i18n";

interface LocalEndpoint {
  interface: string;
  endpoint: string;
}

export function LocalEndpointPicker({
  onSelect,
}: {
  onSelect: (endpoint: string) => void;
}) {
  const t = useText();
  const [choices, setChoices] = useState<LocalEndpoint[] | null>(null);
  const [loading, setLoading] = useState(false);
  const [failed, setFailed] = useState(false);
  const [selected, setSelected] = useState("");
  return (
    <div className="local-endpoint-picker">
      <button
        type="button"
        disabled={loading}
        onClick={async () => {
          setLoading(true);
          setFailed(false);
          try {
            setChoices(await request<LocalEndpoint[]>("get_local_endpoints"));
          } catch {
            setFailed(true);
          } finally {
            setLoading(false);
          }
        }}
      >
        {t(loading ? "working" : "hostLocally")}
      </button>
      {choices && choices.length > 0 && (
        <label>
          {t("localAddress")}
          <select
            value={selected}
            onChange={(event) => {
              setSelected(event.target.value);
              if (event.target.value) onSelect(event.target.value);
            }}
          >
            <option value="">{t("chooseAddress")}</option>
            {choices.map((choice) => (
              <option value={choice.endpoint} key={choice.endpoint}>
                {choice.interface} · {choice.endpoint}
              </option>
            ))}
          </select>
        </label>
      )}
      {(failed || choices?.length === 0) && (
        <p role="status" className="field-hint">
          {t("localUnavailable")}
        </p>
      )}
      <p className="field-hint">{t("localEndpointHint")}</p>
    </div>
  );
}
