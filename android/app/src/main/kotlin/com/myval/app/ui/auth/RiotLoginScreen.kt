package com.myval.app.ui.auth

import android.annotation.SuppressLint
import android.graphics.Bitmap
import android.view.MotionEvent
import android.view.View
import android.view.ViewGroup
import android.webkit.CookieManager
import android.webkit.WebChromeClient
import android.webkit.WebResourceRequest
import android.webkit.WebSettings
import android.webkit.WebStorage
import android.webkit.WebView
import android.webkit.WebViewClient
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.DeleteSweep
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.viewinterop.AndroidView
import com.myval.app.ui.theme.ObsidianBg
import com.myval.app.ui.theme.SurfaceCard
import com.myval.app.ui.theme.TextSecondary
import com.myval.app.ui.theme.ValCyan
import com.myval.app.ui.theme.ValRed

@SuppressLint("SetJavaScriptEnabled", "ClickableViewAccessibility")
@Composable
fun RiotLoginScreen(
    loginUrl: String,
    isAddingAnotherAccount: Boolean,
    isCompletingAuth: Boolean,
    authError: String?,
    onRedirectWithTokens: (String, String?) -> Unit,
    onClose: () -> Unit,
) {
    var pageLoading by remember { mutableStateOf(true) }
    var loadProgress by remember { mutableIntStateOf(10) }
    var capturedTokenUrl by remember { mutableStateOf(false) }
    var didClearCookiesOnOpen by rememberSaveable { mutableStateOf(false) }
    var webViewRef by remember { mutableStateOf<WebView?>(null) }

    LaunchedEffect(authError) {
        if (authError != null) {
            capturedTokenUrl = false
        }
    }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(ObsidianBg)
            .statusBarsPadding()
            .navigationBarsPadding()
            .imePadding(),
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .background(SurfaceCard)
                .padding(horizontal = 8.dp, vertical = 6.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            IconButton(onClick = onClose) {
                Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = "Back")
            }
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = if (isAddingAnotherAccount) "Add Riot Account" else "Riot Sign-On (RSO)",
                    fontWeight = FontWeight.Bold,
                    fontSize = 15.sp,
                )
                Text(
                    text = if (isAddingAnotherAccount) {
                        "Fresh WebView session • Existing saved accounts stay signed in"
                    } else {
                        "Swipe & scroll freely • Direct auth.riotgames.com"
                    },
                    color = ValCyan,
                    fontSize = 11.sp,
                )
            }
            IconButton(
                onClick = {
                    capturedTokenUrl = false
                    webViewRef?.let { wv ->
                        if (wv.url.isNullOrBlank() || wv.url?.contains("access_token=") == true) {
                            wv.loadUrl(loginUrl)
                        } else {
                            wv.reload()
                        }
                    }
                },
            ) {
                Icon(Icons.Default.Refresh, contentDescription = "Reload login page")
            }
            IconButton(
                onClick = {
                    val wv = webViewRef
                    WebStorage.getInstance().deleteAllData()
                    CookieManager.getInstance().removeAllCookies {
                        CookieManager.getInstance().flush()
                        capturedTokenUrl = false
                        wv?.post {
                            wv.clearCache(true)
                            wv.loadUrl(loginUrl)
                        }
                    }
                },
            ) {
                Icon(Icons.Default.DeleteSweep, contentDescription = "Reset cookies")
            }
            Spacer(modifier = Modifier.width(4.dp))
        }

        if (pageLoading || isCompletingAuth) {
            LinearProgressIndicator(
                progress = { (loadProgress / 100f).coerceIn(0.08f, 1f) },
                modifier = Modifier.fillMaxWidth(),
                color = ValRed,
            )
        }

        if (authError != null) {
            Text(
                text = authError,
                color = ValRed,
                style = MaterialTheme.typography.bodySmall,
                modifier = Modifier
                    .fillMaxWidth()
                    .background(ValRed.copy(alpha = 0.14f))
                    .padding(horizontal = 16.dp, vertical = 8.dp),
            )
        }

        Box(
            modifier = Modifier
                .fillMaxWidth()
                .weight(1f),
        ) {
            AndroidView(
                modifier = Modifier.fillMaxSize(),
                factory = { ctx ->
                    WebView(ctx).apply {
                        webViewRef = this
                        layoutParams = ViewGroup.LayoutParams(
                            ViewGroup.LayoutParams.MATCH_PARENT,
                            ViewGroup.LayoutParams.MATCH_PARENT,
                        )
                        setBackgroundColor(ObsidianBg.toArgb())
                        isFocusable = true
                        isFocusableInTouchMode = true
                        isScrollContainer = true
                        isVerticalScrollBarEnabled = true
                        isHorizontalScrollBarEnabled = true
                        overScrollMode = View.OVER_SCROLL_IF_CONTENT_SCROLLS

                        // Prevent parent Compose containers from intercepting vertical/horizontal swipes
                        setOnTouchListener { v, event ->
                            when (event.actionMasked) {
                                MotionEvent.ACTION_DOWN,
                                MotionEvent.ACTION_MOVE -> v.parent?.requestDisallowInterceptTouchEvent(true)
                            }
                            false
                        }

                        settings.apply {
                            javaScriptEnabled = true
                            domStorageEnabled = true
                            @Suppress("DEPRECATION")
                            databaseEnabled = true
                            useWideViewPort = true
                            loadWithOverviewMode = true
                            setSupportZoom(true)
                            builtInZoomControls = true
                            displayZoomControls = false
                            javaScriptCanOpenWindowsAutomatically = true
                            mixedContentMode = WebSettings.MIXED_CONTENT_COMPATIBILITY_MODE
                            // Strip "; wv" so Riot serves its responsive mobile viewport cleanly
                            userAgentString = userAgentString.replace("; wv", "")
                        }

                        val cookieManager = CookieManager.getInstance()
                        cookieManager.setAcceptCookie(true)
                        cookieManager.setAcceptThirdPartyCookies(this, true)

                        webChromeClient = object : WebChromeClient() {
                            override fun onProgressChanged(view: WebView?, newProgress: Int) {
                                loadProgress = newProgress
                                pageLoading = newProgress < 100
                            }
                        }

                        webViewClient = object : WebViewClient() {
                            private fun inspectUrl(url: String?): Boolean {
                                if (url != null && url.contains("access_token=") && !capturedTokenUrl) {
                                    capturedTokenUrl = true
                                    val freshCookies = cookieManager.getCookie("https://auth.riotgames.com")
                                    cookieManager.flush()
                                    onRedirectWithTokens(url, freshCookies)
                                    return true
                                }
                                return false
                            }

                            override fun shouldOverrideUrlLoading(
                                view: WebView?,
                                request: WebResourceRequest?,
                            ): Boolean = inspectUrl(request?.url?.toString())

                            override fun onPageStarted(
                                view: WebView?,
                                url: String?,
                                favicon: Bitmap?,
                            ) {
                                pageLoading = true
                                inspectUrl(url)
                            }

                            override fun onPageFinished(view: WebView?, url: String?) {
                                pageLoading = false
                                if (!inspectUrl(url)) {
                                    // Ensure mobile responsive viewport meta is active and centered
                                    view?.evaluateJavascript(
                                        """
                                        (function() {
                                          var m = document.querySelector('meta[name="viewport"]');
                                          if (!m) {
                                            m = document.createElement('meta');
                                            m.name = 'viewport';
                                            document.head.appendChild(m);
                                          }
                                          m.content = 'width=device-width, initial-scale=1.0, maximum-scale=3.0, user-scalable=yes';
                                          document.documentElement.style.overflowY = 'auto';
                                          document.body.style.overflowY = 'auto';
                                        })();
                                        """.trimIndent(),
                                        null,
                                    )
                                }
                            }
                        }

                        if (isAddingAnotherAccount && !didClearCookiesOnOpen) {
                            didClearCookiesOnOpen = true
                            // Clear active SSO cookies once when opening the sheet so Riot prompts for the new account
                            WebStorage.getInstance().deleteAllData()
                            clearCache(true)
                            cookieManager.removeAllCookies {
                                cookieManager.flush()
                                post { loadUrl(loginUrl) }
                            }
                        } else {
                            loadUrl(loginUrl)
                        }
                    }
                },
            )

            if (isCompletingAuth) {
                Box(
                    modifier = Modifier
                        .fillMaxSize()
                        .background(ObsidianBg.copy(alpha = 0.85f)),
                    contentAlignment = Alignment.Center,
                ) {
                    Column(horizontalAlignment = Alignment.CenterHorizontally) {
                        CircularProgressIndicator(color = ValRed)
                        Text(
                            text = "Exchanging Riot Entitlements & Region Shard...",
                            color = TextSecondary,
                            modifier = Modifier.padding(top = 12.dp),
                        )
                    }
                }
            }
        }
    }
}
