package io.quicklan.android

import android.Manifest
import android.annotation.SuppressLint
import android.app.AlertDialog
import android.content.Intent
import android.content.pm.PackageManager
import android.net.Uri
import android.os.Build
import android.os.Bundle
import android.provider.Settings
import android.webkit.CookieManager
import android.webkit.WebChromeClient
import android.webkit.WebResourceRequest
import android.webkit.WebResourceResponse
import android.webkit.WebSettings
import android.webkit.WebView
import android.webkit.WebViewClient
import android.widget.FrameLayout
import androidx.activity.ComponentActivity
import androidx.activity.OnBackPressedCallback
import androidx.activity.result.contract.ActivityResultContracts
import androidx.core.content.ContextCompat
import androidx.core.view.ViewCompat
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.updatePadding
import androidx.webkit.WebViewAssetLoader
import androidx.webkit.WebViewCompat
import androidx.webkit.WebViewFeature
import org.json.JSONObject
import java.io.ByteArrayInputStream

class MainActivity : ComponentActivity() {
    private lateinit var webView: WebView
    private var vpnResult: ((Boolean) -> Unit)? = null
    private var vpnTimeout: Runnable? = null

    private val vpnLauncher =
        registerForActivityResult(ActivityResultContracts.StartActivityForResult()) { result ->
            completeVpnConsent(result.resultCode == RESULT_OK)
        }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        QuickLanApplication.instance.activity = this
        WindowCompat.setDecorFitsSystemWindows(window, false)
        window.statusBarColor = ContextCompat.getColor(this, R.color.brand)
        maybeRequestNotifications()
        webView = WebView(this)
        webView.setBackgroundColor(ContextCompat.getColor(this, R.color.brand))
        // WebView padding does not resize its CSS viewport. Inset a parent so
        // fixed navigation and modal actions remain above system bars and IME.
        val content = FrameLayout(this)
        content.addView(webView, FrameLayout.LayoutParams(
            FrameLayout.LayoutParams.MATCH_PARENT,
            FrameLayout.LayoutParams.MATCH_PARENT,
        ))
        setContentView(content)
        ViewCompat.setOnApplyWindowInsetsListener(content) { view, insets ->
            val bars =
                insets.getInsets(
                    WindowInsetsCompat.Type.systemBars() or
                        WindowInsetsCompat.Type.ime() or
                        WindowInsetsCompat.Type.displayCutout(),
                )
            view.updatePadding(bars.left, bars.top, bars.right, bars.bottom)
            WindowInsetsCompat.CONSUMED
        }
        if (!WebViewFeature.isFeatureSupported(WebViewFeature.WEB_MESSAGE_LISTENER)) {
            AlertDialog.Builder(this)
                .setTitle(R.string.webview_update_title)
                .setMessage(R.string.webview_update_body)
                .setPositiveButton(R.string.webview_settings) { _, _ ->
                    val provider = WebView.getCurrentWebViewPackage()?.packageName
                    if (provider != null) {
                        startActivity(Intent(Settings.ACTION_APPLICATION_DETAILS_SETTINGS,
                            Uri.parse("package:$provider")))
                    }
                    finish()
                }
                .setNegativeButton(R.string.close_app) { _, _ -> finish() }
                .setCancelable(false)
                .show()
            return
        }
        configureWebView()
        onBackPressedDispatcher.addCallback(
            this,
            object : OnBackPressedCallback(true) {
                override fun handleOnBackPressed() {
                    dispatchBack()
                }
            },
        )
        webView.loadUrl(TRUSTED_PAGE)
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        setIntent(intent)
    }

    override fun onDestroy() {
        completeVpnConsent(false)
        if (QuickLanApplication.instance.activity === this) {
            QuickLanApplication.instance.activity = null
        }
        webView.loadUrl("about:blank")
        webView.removeAllViews()
        webView.destroy()
        super.onDestroy()
    }

    fun requestVpnConsent(consent: Intent, result: (Boolean) -> Unit) {
        runOnUiThread {
            if (isDestroyed || isFinishing || vpnResult != null) {
                result(false)
                return@runOnUiThread
            }
            vpnResult = result
            vpnTimeout = Runnable { completeVpnConsent(false) }.also {
                QuickLanApplication.instance.mainHandler.postDelayed(it, 120_000)
            }
            vpnLauncher.launch(consent)
        }
    }

    fun cancelVpnConsent() {
        runOnUiThread { completeVpnConsent(false) }
    }

    private fun completeVpnConsent(approved: Boolean) {
        val result = vpnResult
        vpnResult = null
        vpnTimeout?.let { QuickLanApplication.instance.mainHandler.removeCallbacks(it) }
        vpnTimeout = null
        result?.invoke(approved)
    }

    fun deliver(payload: JSONObject) {
        if (isDestroyed || !::webView.isInitialized) return
        val quoted = JSONObject.quote(payload.toString())
        webView.evaluateJavascript(
            "window.__QUICKLAN_ANDROID_RECEIVE(JSON.parse($quoted))",
            null,
        )
    }

    private fun dispatchBack() {
        if (!::webView.isInitialized) {
            finish()
            return
        }
        webView.evaluateJavascript(
            "(function(){try{var e=new Event('quicklan:back',{cancelable:true,bubbles:true});return !window.dispatchEvent(e);}catch(x){return false;}})()",
        ) { result ->
            if (result != "true") {
                finish()
            }
        }
    }

    private fun maybeRequestNotifications() {
        if (Build.VERSION.SDK_INT < 33) return
        if (ContextCompat.checkSelfPermission(this, Manifest.permission.POST_NOTIFICATIONS) ==
            PackageManager.PERMISSION_GRANTED
        ) {
            return
        }
        requestPermissions(arrayOf(Manifest.permission.POST_NOTIFICATIONS), 1)
    }

    @SuppressLint("SetJavaScriptEnabled")
    private fun configureWebView() {
        WebView.setWebContentsDebuggingEnabled(BuildConfig.DEBUG)
        CookieManager.getInstance().setAcceptCookie(false)
        val settings = webView.settings
        settings.javaScriptEnabled = true
        settings.domStorageEnabled = false
        settings.databaseEnabled = false
        settings.allowFileAccess = false
        settings.allowContentAccess = false
        settings.allowFileAccessFromFileURLs = false
        settings.allowUniversalAccessFromFileURLs = false
        settings.javaScriptCanOpenWindowsAutomatically = false
        settings.setSupportMultipleWindows(false)
        settings.mixedContentMode = WebSettings.MIXED_CONTENT_NEVER_ALLOW
        settings.mediaPlaybackRequiresUserGesture = true
        settings.cacheMode = WebSettings.LOAD_NO_CACHE
        if (Build.VERSION.SDK_INT >= 26) {
            settings.safeBrowsingEnabled = true
        }
        webView.setDownloadListener { _, _, _, _, _ -> }
        val assetLoader =
            WebViewAssetLoader.Builder()
                .setDomain(ASSET_HOST)
                .setHttpAllowed(false)
                .addPathHandler("/assets/", WebViewAssetLoader.AssetsPathHandler(this))
                .build()
        webView.webViewClient =
            object : WebViewClient() {
                override fun shouldOverrideUrlLoading(
                    view: WebView,
                    request: WebResourceRequest,
                ): Boolean = !isTrustedPage(request.url)

                override fun shouldInterceptRequest(
                    view: WebView,
                    request: WebResourceRequest,
                ): WebResourceResponse {
                    val url = request.url
                    if (!isTrustedAsset(url)) {
                        return deny()
                    }
                    val loaded = assetLoader.shouldInterceptRequest(url) ?: return deny()
                    return withSecurityHeaders(loaded, url)
                }

                override fun onReceivedSslError(
                    view: android.webkit.WebView?,
                    handler: android.webkit.SslErrorHandler?,
                    error: android.net.http.SslError?,
                ) {
                    handler?.cancel()
                }
            }
        webView.webChromeClient =
            object : WebChromeClient() {
                override fun onCreateWindow(
                    view: WebView?,
                    isDialog: Boolean,
                    isUserGesture: Boolean,
                    resultMsg: android.os.Message?,
                ): Boolean = false

                override fun onPermissionRequest(request: android.webkit.PermissionRequest?) {
                    request?.deny()
                }

                override fun onGeolocationPermissionsShowPrompt(
                    origin: String?,
                    callback: android.webkit.GeolocationPermissions.Callback?,
                ) {
                    callback?.invoke(origin, false, false)
                }

                override fun onShowFileChooser(
                    webView: WebView?,
                    filePathCallback: android.webkit.ValueCallback<Array<Uri>>?,
                    fileChooserParams: FileChooserParams?,
                ): Boolean {
                    filePathCallback?.onReceiveValue(null)
                    return true
                }
            }
        if (WebViewFeature.isFeatureSupported(WebViewFeature.WEB_MESSAGE_LISTENER)) {
            WebViewCompat.addWebMessageListener(
                webView,
                "QuickLANAndroid",
                setOf(ASSET_ORIGIN),
            ) { view, message, sourceOrigin, isMainFrame, _ ->
                if (!isMainFrame) return@addWebMessageListener
                if (sourceOrigin.scheme != "https" || sourceOrigin.host != ASSET_HOST) {
                    return@addWebMessageListener
                }
                val page = view.url ?: return@addWebMessageListener
                if (!isTrustedPage(Uri.parse(page))) return@addWebMessageListener
                val data = message.data ?: return@addWebMessageListener
                if (data.length > MAX_MESSAGE) return@addWebMessageListener
                QuickLanApplication.instance.executor.submitJs(data) { payload ->
                    deliver(payload)
                }
            }
        }
    }

    private companion object {
        const val ASSET_HOST = "appassets.androidplatform.net"
        const val ASSET_ORIGIN = "https://$ASSET_HOST"
        const val TRUSTED_PAGE = "https://$ASSET_HOST/assets/index.html"
        const val MAX_MESSAGE = 65_536
        const val CSP =
            "default-src 'self' https://appassets.androidplatform.net; " +
                "script-src 'self' https://appassets.androidplatform.net; " +
                "style-src 'self' 'unsafe-inline' https://appassets.androidplatform.net; " +
                "img-src 'self' data: https://appassets.androidplatform.net; " +
                "font-src 'self' https://appassets.androidplatform.net; " +
                "connect-src 'none'; frame-src 'none'; object-src 'none'; base-uri 'none'; form-action 'none'"

        fun isTrustedPage(url: Uri): Boolean {
            return url.scheme == "https" &&
                url.host == ASSET_HOST &&
                (url.port == -1 || url.port == 443) &&
                url.path == "/assets/index.html" &&
                url.query.isNullOrEmpty()
        }

        fun isTrustedAsset(url: Uri): Boolean {
            if (url.scheme != "https" || url.host != ASSET_HOST) return false
            if (url.port != -1 && url.port != 443) return false
            val path = url.path ?: return false
            if (!path.startsWith("/assets/")) return false
            if (path.contains("..") || path.contains('\\')) return false
            return true
        }

        fun deny(): WebResourceResponse {
            return WebResourceResponse(
                "text/plain",
                "utf-8",
                403,
                "Forbidden",
                mapOf("Cache-Control" to "no-store"),
                ByteArrayInputStream(ByteArray(0)),
            )
        }

        fun withSecurityHeaders(response: WebResourceResponse, url: Uri): WebResourceResponse {
            val headers = HashMap<String, String>()
            response.responseHeaders?.let { headers.putAll(it) }
            headers["X-Content-Type-Options"] = "nosniff"
            headers["Cache-Control"] = "no-store"
            if (url.path == "/assets/index.html") {
                headers["Content-Security-Policy"] = CSP
            }
            val status = response.statusCode.let { if (it >= 100) it else 200 }
            val reason = response.reasonPhrase?.takeIf { it.isNotBlank() } ?: "OK"
            return WebResourceResponse(
                response.mimeType ?: "application/octet-stream",
                response.encoding ?: "utf-8",
                status,
                reason,
                headers,
                response.data,
            )
        }
    }
}
