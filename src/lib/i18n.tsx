import { createContext, useContext } from "react";
import { isAndroid } from "./bridge";
export const en = {
  hostLocally: "Host on the same Wi-Fi or LAN",
  localAddress: "This computer’s local endpoint",
  chooseAddress: "Choose your Wi-Fi or Ethernet address",
  localUnavailable:
    "No local address could be read. Enter a reachable endpoint in connection settings.",
  localEndpointHint:
    "For friends on the same router, choose your Wi-Fi or Ethernet address. Keep this computer connected while friends join. VPN/container addresses may also appear; remote friends need a reachable endpoint in connection settings.",
  localEndpointHintAndroid:
    "For friends on the same router, choose your Wi-Fi or Ethernet address. Keep this device connected while friends join. VPN/container addresses may also appear; remote friends need a reachable endpoint in connection settings.",
  helpNav: "Help",
  androidStack: "Android WebView · React · TypeScript · Rust · lucide",
  nativeFooter: "QuickLAN · One active network per device",
  helperReady: "Networking engine available",
  helperReadyBody:
    "Connect from your saved network. The operating system will ask for permission to create its virtual adapter.",
  helperReadyBodyAndroid:
    "Connect from your saved network. Android will ask for VPN permission to create the virtual adapter. After you leave the app, the VPN continues under a notification until you disconnect or quit.",
  peerName: "Device name",
  peerPath: "Connection path",
  directConsent:
    "I accept the listed nodes for discovery and connection setup. Application traffic must use a direct connection and will stop if none is available.",
  checkService: "Check application port",
  serviceHint:
    "Enter the TCP port your friend’s app uses. This opens one connection without sending data. UDP-only games need an in-game connection test.",
  tcpPort: "TCP port",
  checkingService: "Checking…",
  serviceReachable:
    "TCP connection accepted. The service is listening; this does not verify the application protocol.",
  serviceRefused:
    "Connection refused. Ask your friend to start the app and confirm its port and bind address; a firewall may also reject it.",
  serviceTimedOut:
    "No response within 3 seconds. Check the peer path, app port and firewall permissions.",
  serviceUnreachable:
    "The connection could not be established. Check the peer connection and application settings.",
  peerGone: "This peer is no longer in the current network state.",
  editSettings: "Edit connection settings",
  reshare:
    "After changing connection settings, copy a fresh invitation and share it with every member.",
  addNode: "Add node",
  removeNode: "Remove node",
  networks: "Networks",
  create: "Create network",
  createLong: "Create a network",
  join: "Join with invitation",
  firewallTitle: "System firewall",
  firewallScope:
    "Controls all Windows Defender Firewall profiles or the macOS application firewall. Third-party firewalls and macOS packet-filter rules are outside this control.",
  firewallUnsupported: "System firewall control is not available on Android",
  firewallUnsupportedBody:
    "Android does not allow an unprivileged app to enable or disable the OS firewall. QuickLAN will not pretend to change it. Review firewall or Private DNS options in system settings if needed, and check that the host app may accept connections.",
  firewallOn: "Enabled",
  firewallOff: "Disabled",
  firewallEnable: "Enable firewall",
  firewallDisable: "Disable firewall",
  firewallUnavailable:
    "Firewall state is unavailable. Refresh to retry; no state is assumed.",
  firewallPersistence:
    "Changes apply to this whole computer and stay in effect after disconnecting or quitting QuickLAN. Use Enable firewall to turn protection back on. Existing application rules are retained.",
  firewallWarning:
    "Disabling the system firewall affects all applications and networks, including public Wi-Fi. Listening services may become reachable. This does not fix every connection problem. Administrator authorization is required.",
  firewallAccept:
    "I understand this affects the whole computer and persists until I enable the firewall again.",
  firewallWaiting:
    "Waiting for system authorization and checking the firewall…",
  firewallVerified:
    "The operating system reports the requested firewall state.",
  firewallPolicyBlocked:
    "The requested state was not reached. A managed policy may override local changes. The current state is shown below; QuickLAN does not bypass organization policies.",
  firewallFailed:
    "The change was cancelled or could not be completed. Check the current state and retry with administrator authorization.",
  firewallBusy:
    "Another firewall change is still running. Wait for it to finish, then refresh.",
  preferences: "Preferences",
  about: "About",
  diagnostics: "Diagnostics",
  help: "Connect to a game or app",
  quit: "Quit and disconnect",
  headline: "Your friends. Your network.",
  intro:
    "Create a private network, share an invitation, and connect by virtual IP.",
  engineering: "Networking helper unavailable in this build",
  welcomeNote:
    "QuickLAN works with people you trust. An invitation is a shared credential, and peers may reach listening services allowed by your firewall.",
  learnMore: "How it works",
  connect: "Connect",
  disconnect: "Disconnect",
  invite: "Invite",
  virtualAddress: "Virtual address",
  availableAfter: "Available after connecting",
  devices: "Devices",
  noDevices: "No connected devices",
  noDevicesHint: "Connect to see devices on this network.",
  policy: "Connection policy",
  manual: "No public assistance",
  manualDetail: "Manual endpoints",
  assisted: "P2P preferred, relay permitted",
  assistedDetail: "Custom assistance nodes",
  directOnly: "Direct-only application traffic",
  directOnlyDetail: "Discovery assistance; no relayed application data",
  directOnlyGap: "This connection policy is not supported by this engine.",
  helperTitle: "Networking helper unavailable",
  helperBody:
    "Your networks are saved. Connection requires the verified system helper.",
  details: "View details",
  createTitle: "Create a network",
  label: "Network label",
  nickname: "Your device name",
  singleActive: "One active network at a time.",
  settings: "Connection settings",
  subnet: "Shared address range",
  subnetHelp:
    "A private /24 used by every device. A conflict needs a coordinated change.",
  endpoint: "Reachable endpoint",
  operator: "Node operator",
  endpointHelp:
    "Optional for saving. To connect remotely, your friends need a shared reachable endpoint. Manual mode requires an IP address.",
  operatorPlaceholder: "You or your node operator",
  assistedConsent:
    "I permit discovery assistance and relay fallback through the configured operator. They may see public IPs, timing and traffic volume.",
  noBuiltins:
    "No public nodes are built in. Add only a node whose operator permits your use.",
  cancel: "Cancel",
  save: "Save",
  close: "Close",
  working: "Working…",
  pasteTitle: "Paste an invitation",
  pasteLabel: "Invitation token",
  pasteHint:
    "Paste a QuickLAN invitation from someone you trust. Nothing connects until you choose Connect.",
  review: "Review invitation",
  reviewTitle: "Review invitation",
  saveNetwork: "Save network",
  bearer: "Anyone with this invitation may join.",
  bearerHint:
    "Only accept invitations from people you trust. Invitations do not expire and cannot be revoked per device.",
  trust:
    "I trust this invitation and understand peers may reach permitted listening services.",
  noEndpoints:
    "No reachable endpoint included. Configure a shared endpoint before remote use.",
  inviteTitle: "Invite your friends",
  inviteHint: "Send this invitation privately using your existing messenger.",
  copyInvite: "Copy invitation",
  copied: "Copied to clipboard",
  clipboardHint:
    "The clipboard may be visible to other apps or clipboard history. QuickLAN only writes when you ask.",
  unknown: "Unknown",
  unreachable: "Unreachable",
  local: "This device",
  direct: "Connected directly",
  relayed: "Using a relay",
  latency: "Measured latency",
  unavailable: "Unavailable",
  peerDetail: "Device details",
  selfReported:
    "Device names and numeric IDs are self-reported. They are not verified identities.",
  copyIp: "Copy virtual IP",
  disconnected: "Disconnected",
  starting: "Starting",
  joining: "Joining network",
  connected: "Connected",
  reconnecting: "Reconnecting",
  stopping: "Stopping",
  failed: "Connection unavailable",
  appearance: "Appearance",
  theme: "Theme",
  system: "System",
  light: "Light",
  dark: "Dark",
  language: "Language",
  behavior: "Window behavior",
  behaviorAndroid: "App behavior",
  closeBehaviorAndroid:
    "Leaving QuickLAN keeps the VPN running under a notification until you disconnect or quit. The app does not connect automatically after a process restart.",
  closeBehavior:
    "Closing the window disconnects. No background networking or automatic startup connection is enabled.",
  privacy: "Privacy",
  privacyBody:
    "No account, analytics, automatic update checks or remote crash uploads. Diagnostics stay on your device.",
  savePreferences: "Save preferences",
  saved: "Saved",
  diagnosticsTitle: "Connection diagnostics",
  diagnosticsHint:
    "Preview this report before sharing it. Only application state and error codes are included.",
  copyReport: "Copy sanitized report",
  releaseChecks: "Networking availability",
  refresh: "Refresh",
  noEvents: "No connection events yet.",
  aboutTitle: "Small networks. Open source.",
  aboutBody:
    "QuickLAN is an independent desktop application built around EasyTier. It is not affiliated with the EasyTier project.",
  aboutBodyAndroid:
    "QuickLAN is an independent application built around EasyTier. It is not affiliated with the EasyTier project.",
  wrapperLicense:
    "QuickLAN desktop: Apache-2.0. The separate networking engine: GPL-3.0-only.",
  wrapperLicenseAndroid:
    "This Android app is a GPL-3.0 combined work of the QuickLAN interface and networking engine. The desktop GUI is Apache-2.0 with a separately licensed GPL-3.0 engine.",
  coreLicense:
    "EasyTier core: LGPL-3.0. Its original license and attribution are preserved.",
  aboutGap:
    "The packaged networking helper is missing or could not be verified. Reinstall the complete matching package.",
  aboutEvidence:
    "Native CI exercises real virtual adapters and cleanup on Windows and both Mac architectures. Isolated Linux tests exercise virtual-IP TCP and UDP. Physical cross-device and internet NAT testing remain outstanding.",
  gameTitle: "Connect by virtual IP",
  gameIntro:
    "QuickLAN targets games and applications that accept an IP address. Automatic LAN-game discovery is not supported.",
  gameOne:
    "Have a trusted friend host the game or application and confirm it is listening.",
  gameTwo:
    "Connect both devices to the same QuickLAN network, then copy the host’s virtual IP.",
  gameThree:
    "Use the game’s direct-connect option with that IP and the port documented by the game.",
  gameFour:
    "Check the application’s firewall permissions. Preferences also provides explicit system firewall controls with administrator authorization.",
  gameFourAndroid:
    "Check the host application’s network permission. Android cannot toggle the system firewall without root, so QuickLAN does not offer that control.",
  gameGap:
    "If the virtual IP changes after reconnecting, copy the new address and restart any application bound to the old address.",
  troubleshooting: "If a connection fails",
  troubleBody:
    "A peer connection does not prove the game port is open. Check the host application, the chosen port and its scoped firewall rule. An offline bootstrap or restrictive network may prevent peers from connecting at all.",
  more: "Network actions",
  rename: "Rename network",
  forget: "Forget network",
  forgetTitle: "Forget this network locally?",
  forgetHint:
    "This removes this device’s saved credential. Other members can still communicate on the old network.",
  replace: "Create replacement credentials",
  replaceTitle: "Create a replacement network?",
  replaceHint:
    "Share the new invitation with your trusted friends. Every device must migrate. The old network remains usable by anyone who still has its credentials.",
  confirmForget: "Forget locally",
  confirmReplace: "Create replacement",
  browserTitle: "Desktop runtime required",
  androidRuntimeTitle: "QuickLAN Android runtime required",
  androidRuntimeBody:
    "This browser view displays the interface. Secure storage and networking commands are available only in the QuickLAN Android app.",
  browserBody:
    "This browser view displays the interface. Secure storage and networking commands are available only in the native QuickLAN app.",
  storageTitle: "Secure storage unavailable",
  retry: "Try again",
  createAndConnect: "Create and connect",
  joinAndConnect: "Join and connect",
  chooseConnection:
    "On the same Wi-Fi? Use the button below. For friends on another internet connection, add a shared node in Connection settings. This build does not yet include an automatic internet service.",
  setupNeeded: "Finish connection setup",
  setupConnection: "Set up connection",
  connectionFailed: "Connection could not start",
  waitingForFriends:
    "Your computer is ready. Share an invitation and ask your friend to join and connect.",
  automaticSubnet: "Automatic — choose an available range",
};
export type TextKey = keyof typeof en;
const zh: Record<TextKey, string> = {
  ...en,
  hostLocally: "在同一 Wi-Fi 或局域网中创建连接",
  localAddress: "本机的局域网端点",
  chooseAddress: "选择 Wi-Fi 或以太网地址",
  localUnavailable: "无法读取局域网地址，请在连接设置中输入可达端点。",
  localEndpointHint:
    "同一路由器下的朋友可使用 Wi-Fi 或以太网地址。朋友加入时请保持本机连接。列表也可能包含 VPN 或容器地址；远程朋友需要在连接设置中配置可达端点。",
  localEndpointHintAndroid:
    "同一路由器下的朋友可使用 Wi-Fi 或以太网地址。朋友加入时请保持本机连接。列表也可能包含 VPN 或容器地址；远程朋友需要在连接设置中配置可达端点。",
  helpNav: "帮助",
  androidStack: "Android WebView · React · TypeScript · Rust · lucide",
  nativeFooter: "QuickLAN · 每台设备同时连接一个网络",
  helperReady: "组网引擎可用",
  helperReadyBody: "从已保存网络发起连接。操作系统将请求创建虚拟网卡的权限。",
  helperReadyBodyAndroid:
    "从已保存网络发起连接。Android 会请求 VPN 权限以创建虚拟网卡。离开应用后，VPN 会以通知形式继续运行，直到你断开或退出。",
  peerName: "设备名称",
  peerPath: "连接路径",
  directConsent:
    "我同意使用列出的节点进行发现和连接建立。应用流量只能直连，没有直连路径时将停止传输。",
  checkService: "检查应用端口",
  serviceHint:
    "输入朋友应用使用的 TCP 端口。只建立一次连接，不发送数据。仅使用 UDP 的游戏需要在游戏内测试。",
  tcpPort: "TCP 端口",
  checkingService: "正在检查…",
  serviceReachable: "TCP 连接已接受，服务正在监听。这不代表已验证应用协议。",
  serviceRefused:
    "连接被拒绝。请朋友启动应用并确认端口和监听地址；防火墙也可能拒绝连接。",
  serviceTimedOut:
    "3 秒内未收到响应。请检查设备连接路径、应用端口和防火墙权限。",
  serviceUnreachable: "无法建立连接。请检查设备连接和应用设置。",
  peerGone: "当前网络状态中已没有此设备。",
  networks: "网络",
  create: "创建网络",
  createLong: "创建网络",
  join: "通过邀请加入",
  firewallTitle: "系统防火墙",
  firewallScope:
    "控制 Windows Defender 防火墙的所有配置文件或 macOS 应用程序防火墙。不控制第三方防火墙或 macOS 数据包过滤规则。",
  firewallUnsupported: "Android 无法控制系统防火墙",
  firewallUnsupportedBody:
    "未经 root 的 Android 应用不能开关系统防火墙。QuickLAN 不会假装已更改。如需查看防火墙或私人 DNS，请使用系统设置，并确认主机应用允许接受连接。",
  firewallOn: "已启用",
  firewallOff: "已停用",
  firewallEnable: "启用防火墙",
  firewallDisable: "停用防火墙",
  firewallUnavailable: "无法获取防火墙状态。请刷新重试；不会假定当前状态。",
  firewallPersistence:
    "更改影响整台电脑，断开连接或退出 QuickLAN 后仍然有效。点击启用防火墙可恢复保护。现有应用程序规则会保留。",
  firewallWarning:
    "停用系统防火墙会影响所有应用和网络，包括公共 Wi-Fi。正在监听的服务可能被访问。这不能解决所有连接问题，需要管理员授权。",
  firewallAccept: "我了解这会影响整台电脑，并持续到我再次启用防火墙。",
  firewallWaiting: "正在等待系统授权并检查防火墙…",
  firewallVerified: "操作系统已报告所请求的防火墙状态。",
  firewallPolicyBlocked:
    "未达到请求的状态。管理策略可能覆盖本地更改。请查看当前状态；QuickLAN 不会绕过组织策略。",
  firewallFailed:
    "更改已取消或未能完成。请检查当前状态，并使用管理员授权重试。",
  firewallBusy: "另一项防火墙更改仍在进行。请等待完成后刷新。",
  preferences: "偏好设置",
  about: "关于",
  diagnostics: "诊断",
  help: "连接游戏或应用",
  quit: "退出并断开连接",
  headline: "朋友之间，轻松组网。",
  intro: "创建私有网络，分享邀请，通过虚拟 IP 连接。",
  engineering: "此版本的组网引擎不可用",
  welcomeNote:
    "请只与信任的人组网。邀请包含共享凭据；成员可能访问防火墙允许的监听服务。",
  learnMore: "了解使用方式",
  connect: "连接",
  disconnect: "断开",
  invite: "邀请",
  virtualAddress: "虚拟地址",
  availableAfter: "连接后可用",
  devices: "设备",
  noDevices: "没有已连接设备",
  noDevicesHint: "连接网络后查看设备。",
  policy: "连接策略",
  manual: "不使用公共网络协助",
  manualDetail: "手动配置节点",
  assisted: "优先直连，允许中继",
  assistedDetail: "自定义协助节点",
  directOnly: "应用流量仅直连",
  directOnlyDetail: "允许辅助发现；禁止中继应用流量",
  directOnlyGap: "此引擎不支持该连接策略。",
  helperTitle: "组网服务不可用",
  helperBody: "网络已保存。连接需要经过验证的系统组网服务。",
  details: "查看详情",
  createTitle: "创建网络",
  label: "网络名称",
  nickname: "设备名称",
  singleActive: "一次只能连接一个网络。",
  settings: "连接设置",
  subnet: "共享地址段",
  subnetHelp: "所有设备使用同一私有 /24 网段。冲突时需协调迁移。",
  endpoint: "可达节点地址",
  operator: "节点运营者",
  endpointHelp:
    "保存时可选。远程连接需要共同的可达节点；手动模式仅接受 IP 地址。",
  operatorPlaceholder: "你或你的节点运营者",
  assistedConsent:
    "我允许通过已配置节点进行发现协助及中继。运营者可能看到公网 IP、时间和流量大小。",
  noBuiltins: "没有内置公共节点。请仅添加允许你使用的节点。",
  cancel: "取消",
  save: "保存",
  close: "关闭",
  working: "处理中…",
  pasteTitle: "粘贴邀请",
  pasteLabel: "邀请令牌",
  pasteHint: "粘贴可信朋友发送的 QuickLAN 邀请。点击“连接”前不会组网。",
  review: "查看邀请",
  reviewTitle: "确认邀请",
  saveNetwork: "保存网络",
  bearer: "任何持有邀请的人都可能加入。",
  bearerHint: "请只接受可信来源的邀请。邀请不会过期，无法单独撤销某台设备。",
  trust: "我信任此邀请，并理解成员可能访问允许的监听服务。",
  noEndpoints: "邀请未包含可达节点。远程使用前需配置共同节点。",
  inviteTitle: "邀请朋友",
  inviteHint: "通过现有通讯工具私下发送邀请。",
  copyInvite: "复制邀请",
  copied: "已复制到剪贴板",
  clipboardHint:
    "其他应用或剪贴板历史可能读取邀请。QuickLAN 仅在你操作时写入剪贴板。",
  unknown: "未知",
  unreachable: "不可达",
  local: "本机",
  direct: "已直连",
  relayed: "通过中继",
  latency: "实测延迟",
  unavailable: "不可用",
  peerDetail: "设备详情",
  selfReported: "设备名称和数字 ID 由对方自行报告，不代表身份已验证。",
  copyIp: "复制虚拟 IP",
  disconnected: "未连接",
  starting: "启动中",
  joining: "加入网络中",
  connected: "已连接",
  reconnecting: "重新连接中",
  stopping: "断开中",
  failed: "连接不可用",
  appearance: "外观",
  theme: "主题",
  system: "跟随系统",
  light: "浅色",
  dark: "深色",
  language: "语言",
  behavior: "窗口行为",
  behaviorAndroid: "应用行为",
  closeBehavior: "关闭窗口即断开连接。本版本不启用后台组网或启动自动连接。",
  closeBehaviorAndroid:
    "离开 QuickLAN 后，VPN 会以通知形式继续运行，直到你断开或退出。进程重启后不会自动连接。",
  privacy: "隐私",
  privacyBody:
    "无需账号；无分析统计、自动更新检查或远程崩溃上传。诊断保留在本机。",
  savePreferences: "保存偏好",
  saved: "已保存",
  diagnosticsTitle: "连接诊断",
  diagnosticsHint: "分享前请预览报告。仅包含应用状态和错误代码。",
  copyReport: "复制脱敏报告",
  releaseChecks: "开放系统组网前的检查",
  refresh: "刷新",
  noEvents: "暂无连接事件。",
  aboutTitle: "小型网络，开放源码。",
  aboutBody:
    "QuickLAN 是基于 EasyTier 的独立桌面应用，与 EasyTier 项目没有隶属关系。",
  aboutBodyAndroid:
    "QuickLAN 是基于 EasyTier 的独立应用，与 EasyTier 项目没有隶属关系。",
  wrapperLicense: "QuickLAN 桌面：Apache-2.0；独立组网引擎：GPL-3.0-only。",
  wrapperLicenseAndroid:
    "此 Android 应用是 QuickLAN 界面与组网引擎的 GPL-3.0 合并作品。桌面图形界面为 Apache-2.0，组网引擎另行以 GPL-3.0 许可。",
  coreLicense: "EasyTier 核心：LGPL-3.0，保留原始许可证及署名。",
  aboutGap: "组网引擎缺失或验证失败，请重新安装完整的对应安装包。",
  aboutEvidence:
    "原生 CI 验证 Windows 和两种 Mac 架构的真实虚拟网卡及清理。隔离 Linux 测试验证虚拟 IP 的 TCP/UDP 流量。尚未完成跨实体设备和互联网 NAT 测试。",
  gameTitle: "通过虚拟 IP 连接",
  gameIntro: "适用于支持输入 IP 的游戏和应用，不支持自动发现所有局域网游戏。",
  gameOne: "请可信朋友开启游戏主机或应用，确认服务正在监听。",
  gameTwo: "双方连接同一个 QuickLAN 网络，复制主机的虚拟 IP。",
  gameThree: "在游戏直连界面输入该 IP 和游戏文档注明的端口。",
  gameFour:
    "检查应用的防火墙权限。偏好设置中也提供需要管理员授权的系统防火墙控制。",
  gameFourAndroid:
    "请检查主机应用的网络权限。未经 root，Android 不能开关系统防火墙，因此 QuickLAN 不提供该控制。",
  gameGap:
    "若重新连接后虚拟 IP 发生变化，请复制新地址，并重启绑定旧地址的应用。",
  troubleshooting: "连接失败时",
  troubleBody:
    "设备已连接不代表游戏端口开放。请检查服务、端口和限定范围的防火墙规则。节点离线或网络限制也可能导致连接失败。",
  more: "网络操作",
  rename: "重命名网络",
  forget: "忘记网络",
  forgetTitle: "在本机忘记此网络？",
  forgetHint: "移除本机保存的凭据。其他成员仍可在原网络中通信。",
  replace: "创建替换凭据",
  replaceTitle: "创建替换网络？",
  replaceHint:
    "请向可信朋友发送新邀请，每台设备都需迁移。持有旧凭据的人仍可使用原网络。",
  confirmForget: "在本机忘记",
  confirmReplace: "创建替换网络",
  browserTitle: "需要桌面运行环境",
  browserBody:
    "浏览器仅显示界面。安全存储和组网命令只在原生 QuickLAN 应用中可用。",
  androidRuntimeTitle: "需要 QuickLAN Android 运行环境",
  androidRuntimeBody:
    "浏览器仅显示界面。安全存储和组网命令只在 QuickLAN Android 应用中可用。",
  storageTitle: "安全存储不可用",
  retry: "重试",
  createAndConnect: "创建并连接",
  joinAndConnect: "加入并连接",
  chooseConnection:
    "同一 Wi-Fi 的朋友可使用下方按钮。异地朋友需要在连接设置中添加共享节点。此版本尚未提供自动互联网组网服务。",
  setupNeeded: "完成连接设置",
  setupConnection: "设置连接",
  connectionFailed: "无法开始连接",
  waitingForFriends:
    "本机已就绪。请发送邀请，让朋友加入并连接。",
  automaticSubnet: "自动 — 选择可用网段",
};
export const LocaleContext = createContext<"en" | "zh-CN">("en");
export function useText() {
  const locale = useContext(LocaleContext);
  return (key: TextKey): string => (locale === "zh-CN" ? zh[key] : en[key]);
}
const errors: Record<string, string> = {
  desktop_required: en.browserBody,
  helper_unavailable: en.helperBody,
  invalid_invitation: "This invitation is malformed or too large.",
  unsupported_version: "This invitation needs an unsupported QuickLAN version.",
  invalid_label: "Use 1–64 characters without control characters.",
  invalid_subnet: "Use a private IPv4 /24 network such as 10.73.42.0/24.",
  route_conflict:
    "This address range is already used by a LAN, another VPN, or a saved QuickLAN network. Create a new network to choose a free range, then share its new invitation.",
  invalid_endpoint:
    "Use tcp://IP:port or udp://IP:port, without a path or credentials. Manual mode requires an IP address.",
  unsupported_policy: en.directOnlyGap,
  assistance_consent_required:
    "Accept assistance and relay behavior and configure a shared node.",
  confirmation_required: "Review and accept the invitation first.",
  already_saved: "This network is already saved.",
  not_found: "This network could not be found.",
  busy: "Disconnect the current network first.",
  storage_unavailable:
    "Unlock or allow access to the operating system credential store and try again. No plaintext fallback was used.",
  invalid_storage:
    "Saved data is invalid or newer than this app. It has not been overwritten.",
  unsafe_path: "The private application data folder could not be used safely.",
  release_gate: "System networking has not passed its release checks.",
  core_failed:
    "The networking service could not start or stopped unexpectedly. Try Connect again and review Diagnostics if it keeps failing.",
  permission_denied:
    "Allow the administrator prompt to create the virtual network adapter, then try Connect again.",
  unauthorized: "The helper request was not authorized.",
};
const androidErrorsEn: Record<string, string> = {
  permission_denied:
    "VPN permission was refused. QuickLAN cannot create a virtual adapter without it.",
  request_timeout:
    "The request timed out. If Android asked for VPN permission, allow it and try Connect again.",
  firewall_unsupported: en.firewallUnsupportedBody,
  helper_unavailable: en.androidRuntimeBody,
  desktop_required: en.androidRuntimeBody,
};
const androidErrorsZh: Record<string, string> = {
  permission_denied: "已拒绝 VPN 权限。没有虚拟网卡时 QuickLAN 无法连接。",
  request_timeout:
    "请求超时。如果 Android 正在请求 VPN 权限，请允许后再点连接。",
  firewall_unsupported: zh.firewallUnsupportedBody,
  helper_unavailable: zh.androidRuntimeBody,
  desktop_required: zh.androidRuntimeBody,
};
export function errorText(value: unknown) {
  if (typeof value !== "string")
    return "The operation could not be completed. Try again or review Diagnostics.";
  if (isAndroid()) {
    const androidErrors =
      document.documentElement.lang === "zh-CN"
        ? androidErrorsZh
        : androidErrorsEn;
    if (value in androidErrors) return androidErrors[value];
  }
  return value in errors
    ? errors[value]
    : "The operation could not be completed. Try again or review Diagnostics.";
}
