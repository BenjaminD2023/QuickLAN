import { useState } from "react";
import { ChevronDown, Copy, Settings, Shield } from "lucide-react";
import type {
  Bootstrap,
  JoinPreview,
  Policy,
  Preferences,
  SavedNetwork,
} from "../lib/types";
import { useText } from "../lib/i18n";
import { Modal, ModalFooter, Notice } from "../components/ui";
import { LocalEndpointPicker } from "./LocalEndpointPicker";
export interface DialogProps {
  onClose: () => void;
  busy: boolean;
  error: string | null;
}
export function CreateDialog({
  preferences,
  onCreate,
  ...props
}: DialogProps & {
  preferences: Preferences;
  onCreate: (data: {
    label: string;
    nickname: string;
    subnet: string;
    policy: Policy;
    bootstrap: Bootstrap[];
    assistanceAccepted: boolean;
  }) => void;
}) {
  const t = useText();
  const [label, setLabel] = useState("");
  const [nickname, setNickname] = useState(preferences.nickname);
  const [subnet, setSubnet] = useState("");
  const [policy, setPolicy] = useState<Policy>("manual");
  const [endpoint, setEndpoint] = useState("");
  const [operator, setOperator] = useState("");
  const [consent, setConsent] = useState(false);
  return (
    <Modal title={t("createTitle")} {...props}>
      <form
        onSubmit={(e) => {
          e.preventDefault();
          onCreate({
            label: label.trim(),
            nickname: nickname.trim(),
            subnet: subnet.trim(),
            policy,
            bootstrap: endpoint.trim()
              ? [{ endpoint: endpoint.trim(), operator: operator.trim() }]
              : [],
            assistanceAccepted: consent,
          });
        }}
      >
        <label>
          {t("label")}
          <input
            autoFocus
            value={label}
            maxLength={64}
            required
            placeholder="Friday night"
            onChange={(e) => setLabel(e.target.value)}
          />
        </label>
        <label>
          {t("nickname")}
          <input
            value={nickname}
            maxLength={64}
            required
            onChange={(e) => setNickname(e.target.value)}
          />
        </label>
        <p className="muted">{t("chooseConnection")}</p>
        <LocalEndpointPicker
          onSelect={(value) => {
            setEndpoint(value);
            setOperator(nickname.trim() || preferences.nickname);
            setConsent(false);
          }}
        />
        {endpoint && (
          <p className="field-hint">
            <code>{endpoint}</code>
          </p>
        )}
        <details className="advanced">
          <summary>
            <Settings />
            {t("settings")}
            <ChevronDown />
          </summary>
          <div className="advanced-body">
            <label>
              {t("subnet")}
              <input
                value={subnet}
                placeholder={t("automaticSubnet")}
                onChange={(e) => setSubnet(e.target.value)}
                spellCheck={false}
              />
            </label>
            <p className="field-hint">{t("subnetHelp")}</p>
            <label>
              {t("policy")}
              <select
                value={policy}
                onChange={(e) => {
                  setPolicy(e.target.value as Policy);
                  setConsent(false);
                }}
              >
                <option value="manual">{t("manual")}</option>
                <option value="assisted">{t("assisted")}</option>
                <option value="direct_only">{t("directOnly")}</option>
              </select>
            </label>
            <label>
              {t("endpoint")}
              <input
                value={endpoint}
                onChange={(e) => setEndpoint(e.target.value)}
                placeholder="tcp://192.168.1.12:11010"
                maxLength={256}
                required={policy !== "manual"}
                spellCheck={false}
              />
            </label>
            <p className="field-hint">{t("endpointHelp")}</p>
            {endpoint && (
              <label>
                {t("operator")}
                <input
                  value={operator}
                  onChange={(e) => setOperator(e.target.value)}
                  placeholder={t("operatorPlaceholder")}
                  maxLength={64}
                  required
                />
              </label>
            )}
            {policy !== "manual" && (
              <>
                <p className="field-hint">{t("noBuiltins")}</p>
                <label className="checkbox">
                  <input
                    type="checkbox"
                    checked={consent}
                    onChange={(e) => setConsent(e.target.checked)}
                    required
                  />
                  {t(
                    policy === "direct_only"
                      ? "directConsent"
                      : "assistedConsent",
                  )}
                </label>
              </>
            )}
          </div>
        </details>
        <ModalFooter
          {...props}
          submit={t("createAndConnect")}
          disabled={
            !label.trim() ||
            !nickname.trim() ||
            !endpoint.trim() ||
            (policy !== "manual" && !consent)
          }
        />
      </form>
    </Modal>
  );
}
export function JoinDialog({
  preview,
  onReview,
  onAccept,
  ...props
}: DialogProps & {
  preview: JoinPreview | null;
  onReview: (token: string) => void;
  onAccept: (ticket: string, consent: boolean) => void;
}) {
  const t = useText();
  const [token, setToken] = useState("");
  const [trusted, setTrusted] = useState(false);
  const [consent, setConsent] = useState(false);
  return (
    <Modal title={t(preview ? "reviewTitle" : "pasteTitle")} {...props}>
      {!preview ? (
        <form
          onSubmit={(e) => {
            e.preventDefault();
            onReview(token);
          }}
        >
          <p className="dialog-intro">{t("pasteHint")}</p>
          <label>
            {t("pasteLabel")}
            <textarea
              aria-label={t("pasteLabel")}
              autoFocus
              rows={6}
              spellCheck={false}
              autoComplete="off"
              value={token}
              maxLength={8192}
              required
              placeholder="quicklan1:…"
              onChange={(e) => setToken(e.target.value)}
            />
          </label>
          <ModalFooter
            {...props}
            submit={t("review")}
            disabled={!token.trim()}
          />
        </form>
      ) : (
        <form
          onSubmit={(e) => {
            e.preventDefault();
            if (trusted) onAccept(preview.ticket, consent);
          }}
        >
          <dl className="facts">
            <div>
              <dt>{t("label")}</dt>
              <dd>{preview.network.label}</dd>
            </div>
            <div>
              <dt>{t("subnet")}</dt>
              <dd>
                <code>{preview.network.subnet}</code>
              </dd>
            </div>
            <div>
              <dt>{t("policy")}</dt>
              <dd>
                {t(
                  preview.network.policy === "manual"
                    ? "manual"
                    : preview.network.policy === "direct_only"
                      ? "directOnly"
                      : "assisted",
                )}
              </dd>
            </div>
            {preview.network.bootstrap.map((n) => (
              <div key={n.endpoint}>
                <dt>{n.operator}</dt>
                <dd>
                  <code>{n.endpoint}</code>
                </dd>
              </div>
            ))}
          </dl>
          <Notice title={t("bearer")}>{t("bearerHint")}</Notice>
          {preview.network.bootstrap.length === 0 && (
            <p className="field-hint">{t("noEndpoints")}</p>
          )}
          <label className="checkbox">
            <input
              autoFocus
              type="checkbox"
              checked={trusted}
              required
              onChange={(e) => setTrusted(e.target.checked)}
            />
            {t("trust")}
          </label>
          {preview.network.policy !== "manual" && (
            <label className="checkbox">
              <input
                type="checkbox"
                checked={consent}
                required
                onChange={(e) => setConsent(e.target.checked)}
              />
              {t(
                preview.network.policy === "direct_only"
                  ? "directConsent"
                  : "assistedConsent",
              )}
            </label>
          )}
          <ModalFooter
            {...props}
            submit={t("joinAndConnect")}
            disabled={
              !trusted || (preview.network.policy !== "manual" && !consent)
            }
          />
        </form>
      )}
    </Modal>
  );
}
export function InviteDialog({
  network,
  onCopy,
  ...props
}: DialogProps & { network: SavedNetwork; onCopy: () => void }) {
  const t = useText();
  return (
    <Modal title={t("inviteTitle")} {...props}>
      <div className="invite-network">
        <Shield />
        <div>
          <strong>{network.label}</strong>
          <small>{network.subnet}</small>
        </div>
      </div>
      <p>{t("inviteHint")}</p>
      <Notice title={t("bearer")}>{t("bearerHint")}</Notice>
      <button
        className="primary full-width"
        disabled={props.busy}
        onClick={onCopy}
      >
        <Copy />
        {t("copyInvite")}
      </button>
      <p className="field-hint">{t("clipboardHint")}</p>
    </Modal>
  );
}
export function NetworkActions({
  network,
  onAction,
  ...props
}: DialogProps & {
  network: SavedNetwork;
  onAction: (kind: "rename" | "replace" | "forget", value?: string) => void;
}) {
  const t = useText();
  const [mode, setMode] = useState<"menu" | "rename" | "replace" | "forget">(
    "menu",
  );
  const [label, setLabel] = useState(network.label);
  return (
    <Modal
      title={t(
        mode === "menu"
          ? "more"
          : mode === "rename"
            ? "rename"
            : mode === "replace"
              ? "replaceTitle"
              : "forgetTitle",
      )}
      {...props}
    >
      {mode === "menu" ? (
        <div className="action-list">
          <button onClick={() => setMode("rename")}>{t("rename")}</button>
          <button onClick={() => setMode("replace")}>{t("replace")}</button>
          <button className="danger-text" onClick={() => setMode("forget")}>
            {t("forget")}
          </button>
        </div>
      ) : (
        <form
          onSubmit={(e) => {
            e.preventDefault();
            onAction(mode, label.trim());
          }}
        >
          {mode === "rename" ? (
            <label>
              {t("label")}
              <input
                autoFocus
                value={label}
                maxLength={64}
                required
                onChange={(e) => setLabel(e.target.value)}
              />
            </label>
          ) : (
            <p>{t(mode === "replace" ? "replaceHint" : "forgetHint")}</p>
          )}
          <ModalFooter
            {...props}
            danger={mode === "forget"}
            submit={t(
              mode === "rename"
                ? "save"
                : mode === "replace"
                  ? "confirmReplace"
                  : "confirmForget",
            )}
          />
        </form>
      )}
    </Modal>
  );
}
export function SettingsDialog({
  network,
  onSave,
  ...props
}: DialogProps & {
  network: SavedNetwork;
  onSave: (policy: Policy, bootstrap: Bootstrap[], consent: boolean) => void;
}) {
  const t = useText();
  const [policy, setPolicy] = useState(network.policy);
  const [nodes, setNodes] = useState(network.bootstrap);
  const [consent, setConsent] = useState(false);
  return (
    <Modal title={t("editSettings")} {...props}>
      <form
        onSubmit={(e) => {
          e.preventDefault();
          onSave(
            policy,
            nodes.map((n) => ({
              endpoint: n.endpoint.trim(),
              operator: n.operator.trim(),
            })),
            consent,
          );
        }}
      >
        <LocalEndpointPicker
          onSelect={(endpoint) => {
            setPolicy("manual");
            setNodes([{ endpoint, operator: network.label }]);
            setConsent(false);
          }}
        />
        <label>
          {t("policy")}
          <select
            value={policy}
            onChange={(e) => {
              setPolicy(e.target.value as Policy);
              setConsent(false);
            }}
          >
            <option value="manual">{t("manual")}</option>
            <option value="assisted">{t("assisted")}</option>
            <option value="direct_only">{t("directOnly")}</option>
          </select>
        </label>
        <LocalEndpointPicker
          onSelect={(endpoint) => {
            setPolicy("manual");
            setNodes([{ endpoint, operator: network.label }]);
            setConsent(false);
          }}
        />
        {nodes.map((node, i) => (
          <fieldset className="node-fields" key={i}>
            <label>
              {t("endpoint")}
              <input
                value={node.endpoint}
                maxLength={256}
                spellCheck={false}
                required
                onChange={(e) =>
                  setNodes(
                    nodes.map((n, j) =>
                      j === i ? { ...n, endpoint: e.target.value } : n,
                    ),
                  )
                }
              />
            </label>
            <label>
              {t("operator")}
              <input
                value={node.operator}
                maxLength={64}
                required
                onChange={(e) =>
                  setNodes(
                    nodes.map((n, j) =>
                      j === i ? { ...n, operator: e.target.value } : n,
                    ),
                  )
                }
              />
            </label>
            <button
              type="button"
              className="text-button"
              onClick={() => setNodes(nodes.filter((_, j) => j !== i))}
            >
              {t("removeNode")}
            </button>
          </fieldset>
        ))}
        <button
          type="button"
          disabled={nodes.length >= 4}
          onClick={() => setNodes([...nodes, { endpoint: "", operator: "" }])}
        >
          {t("addNode")}
        </button>
        <p className="field-hint">{t("endpointHelp")}</p>
        {policy !== "manual" && (
          <label className="checkbox">
            <input
              type="checkbox"
              checked={consent}
              required
              onChange={(e) => setConsent(e.target.checked)}
            />
            {t(policy === "direct_only" ? "directConsent" : "assistedConsent")}
          </label>
        )}
        <Notice>{t("reshare")}</Notice>
        <ModalFooter
          {...props}
          submit={t("save")}
          disabled={policy !== "manual" && (!consent || nodes.length === 0)}
        />
      </form>
    </Modal>
  );
}
