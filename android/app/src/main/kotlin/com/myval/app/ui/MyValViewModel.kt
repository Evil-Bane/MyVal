package com.myval.app.ui

import android.app.Application
import android.os.SystemClock
import android.util.Log
import android.webkit.CookieManager
import android.webkit.WebStorage
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import com.myval.app.BuildConfig
import com.myval.app.data.ApkInstallOutcome
import com.myval.app.data.AppReleaseInfo
import com.myval.app.data.AppUpdateChecker
import com.myval.app.data.EngineHolder
import com.myval.app.data.SecureSessionStore
import com.myval.app.data.StoredAccountRecord
import java.io.File
import java.util.Locale
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.myval_ffi.Agent
import uniffi.myval_ffi.AssetSummary
import uniffi.myval_ffi.CareerSnapshot
import uniffi.myval_ffi.CollectionSummary
import uniffi.myval_ffi.FfiException
import uniffi.myval_ffi.KnownPlayerSummary
import uniffi.myval_ffi.LiveState
import uniffi.myval_ffi.LoadoutPreset
import uniffi.myval_ffi.MapInfo
import uniffi.myval_ffi.MatchDetailSnapshot
import uniffi.myval_ffi.PartySnapshot
import uniffi.myval_ffi.PlayerCard
import uniffi.myval_ffi.PlayerLoadoutSnapshot
import uniffi.myval_ffi.PlayerProfileSnapshot
import uniffi.myval_ffi.PlayerTitle
import uniffi.myval_ffi.PreGameSnapshot
import uniffi.myval_ffi.SessionSummary
import uniffi.myval_ffi.Skin
import uniffi.myval_ffi.Spray
import uniffi.myval_ffi.StorefrontSnapshot

enum class AppTab {
    LIVE,
    STORE,
    CAREER,
    LOADOUT,
    EXPLORER,
}

data class MyValUiState(
    val isBootstrapping: Boolean = true,
    val startupStatus: String = "Loading...",
    val networkStatusMessage: String? = null,
    val isNetworkSlow: Boolean = false,
    val selectedTab: AppTab = AppTab.LIVE,
    val isAppForegrounded: Boolean = true,
    val session: SessionSummary? = null,
    val savedAccounts: List<StoredAccountRecord> = emptyList(),
    val showAccountsSheet: Boolean = false,
    val isSigningIn: Boolean = false,
    val showLoginWebView: Boolean = false,
    val isAddingAnotherAccount: Boolean = false,
    val authError: String? = null,
    val assetSummary: AssetSummary? = null,
    val isSyncingAssets: Boolean = false,
    val assetError: String? = null,
    val liveState: LiveState = LiveState.Idle(
        message = "Sign in with Riot to see your party & live match.",
        recommendedPollSecs = 10u,
    ),
    val isPollingLive: Boolean = false,
    val isManualPollingLive: Boolean = false,
    val isMutatingLiveAction: Boolean = false,
    val pregameFirstSeenAtMs: Long? = null,
    val liveError: String? = null,
    val storefront: StorefrontSnapshot? = null,
    val collectionSummary: CollectionSummary? = null,
    val ownedSkinUuids: Set<String> = emptySet(),
    val ownedChromaUuids: Set<String> = emptySet(),
    val ownedLevelUuids: Set<String> = emptySet(),
    val ownedSprayUuids: Set<String> = emptySet(),
    val ownedCardUuids: Set<String> = emptySet(),
    val ownedTitleUuids: Set<String> = emptySet(),
    val ownedAgentUuids: Set<String> = emptySet(),
    val isLoadingStore: Boolean = false,
    val storeError: String? = null,
    val career: CareerSnapshot? = null,
    val isLoadingCareer: Boolean = false,
    val careerError: String? = null,
    val selectedMatchDetail: MatchDetailSnapshot? = null,
    val returnToMatchDetail: MatchDetailSnapshot? = null,
    val isLoadingMatchDetail: Boolean = false,
    val inspectedPlayerPuuid: String? = null,
    val inspectedPlayerHintName: String? = null,
    val inspectedPlayerProfile: PlayerProfileSnapshot? = null,
    val isLoadingPlayerProfile: Boolean = false,
    val playerProfileError: String? = null,
    val showPlayerSearchSheet: Boolean = false,
    val playerSearchQuery: String = "",
    val knownPlayers: List<KnownPlayerSummary> = emptyList(),
    val isLookingUpRiotId: Boolean = false,
    val riotIdLookupError: String? = null,
    val hasLookupApiKey: Boolean = false,
    val loadout: PlayerLoadoutSnapshot? = null,
    val isLoadingLoadout: Boolean = false,
    val loadoutError: String? = null,
    val loadoutPresets: List<LoadoutPreset> = emptyList(),
    val agents: List<Agent> = emptyList(),
    val maps: List<MapInfo> = emptyList(),
    val skins: List<Skin> = emptyList(),
    val allWeaponSkins: List<Skin> = emptyList(),
    val playerCards: List<PlayerCard> = emptyList(),
    val playerTitles: List<PlayerTitle> = emptyList(),
    val sprays: List<Spray> = emptyList(),
    val skinQuery: String = "",
    val wishlistOnly: Boolean = false,
    val actionBanner: String? = null,
    val availableUpdate: AppReleaseInfo? = null,
    val showUpdatePopup: Boolean = false,
    val showAboutSheet: Boolean = false,
    val isCheckingUpdate: Boolean = false,
    val updateCheckStatus: String? = null,
    val isDownloadingUpdate: Boolean = false,
    val updateDownloadProgress: Float? = null,
    val updateDownloadLabel: String? = null,
    val downloadedUpdateApkPath: String? = null,
)

class MyValViewModel(application: Application) : AndroidViewModel(application) {
    private val engine = EngineHolder.get(application)
    private val sessionStore = SecureSessionStore(application)

    private val _uiState = MutableStateFlow(MyValUiState())
    val uiState: StateFlow<MyValUiState> = _uiState.asStateFlow()

    private var livePollingJob: Job? = null
    private var countdownJob: Job? = null
    private var slowNetworkWatchdogJob: Job? = null
    private var activeNetworkOps: Int = 0
    private var explorerLoaded: Boolean = false
    private var dismissedUpdateTag: String? = null
    private var lastStoreTickElapsedMs: Long = 0L
    private var lastLoadoutFetchElapsedMs: Long = 0L
    private var lastCoreGameMatchId: String? = null
    private var pregameFirstSeen: Pair<String, Long>? = null

    private fun resolvePreGameFirstSeenMs(matchId: String): Long {
        val current = pregameFirstSeen
        if (current != null && current.first.equals(matchId, ignoreCase = true)) {
            return current.second
        }
        val now = SystemClock.elapsedRealtime()
        pregameFirstSeen = matchId to now
        return now
    }

    private data class LoadoutFetchBundle(
        val collection: CollectionSummary?,
        val ownedSkins: Set<String>,
        val ownedChromas: Set<String>,
        val ownedLevels: Set<String>,
        val ownedSprays: Set<String>,
        val ownedCards: Set<String>,
        val ownedTitles: Set<String>,
        val loadout: PlayerLoadoutSnapshot,
    )

    private data class CollectionOwnershipBundle(
        val collection: CollectionSummary,
        val ownedSkins: Set<String>,
        val ownedChromas: Set<String>,
        val ownedLevels: Set<String>,
        val ownedSprays: Set<String>,
        val ownedCards: Set<String>,
        val ownedTitles: Set<String>,
    )

    init {
        bootstrap()
    }

    private fun beginNetworkOp(label: String) {
        activeNetworkOps += 1
        _uiState.update {
            it.copy(
                networkStatusMessage = label,
                isNetworkSlow = false,
            )
        }
        slowNetworkWatchdogJob?.cancel()
        slowNetworkWatchdogJob = viewModelScope.launch {
            delay(2_400L)
            if (activeNetworkOps > 0) {
                _uiState.update {
                    it.copy(
                        networkStatusMessage = "Slow network — waiting for Riot server...",
                        isNetworkSlow = true,
                    )
                }
            }
        }
    }

    private fun endNetworkOp() {
        activeNetworkOps = (activeNetworkOps - 1).coerceAtLeast(0)
        if (activeNetworkOps == 0) {
            slowNetworkWatchdogJob?.cancel()
            slowNetworkWatchdogJob = null
            _uiState.update {
                it.copy(
                    networkStatusMessage = null,
                    isNetworkSlow = false,
                )
            }
        }
    }

    private fun bootstrap() {
        viewModelScope.launch {
            _uiState.update { it.copy(isBootstrapping = true, startupStatus = "Loading...") }

            // Single parallel local I/O burst: SQLite metadata + encrypted account vault (~50-150ms, ZERO network wait!)
            val (summary, agents, hasKey, initialVault, needsResync) = withContext(Dispatchers.IO) {
                val sum = runCatching { engine.getAssetSummary() }.getOrNull()
                val ags = runCatching { engine.listAgents() }.getOrDefault(emptyList())
                val key = runCatching { engine.hasLookupApiKey() }.getOrDefault(false)
                val v = sessionStore.loadVault()
                val resync = runCatching { engine.needsAssetResync() }.getOrDefault(true)
                BootstrapLocalBundle(sum, ags, key, v, resync)
            }

            var vault = initialVault
            _uiState.update {
                it.copy(
                    assetSummary = summary,
                    agents = agents,
                    hasLookupApiKey = hasKey,
                    savedAccounts = vault.accounts,
                )
            }

            val activeRecord = vault.activePuuid?.let { activeId ->
                vault.accounts.firstOrNull { it.puuid.equals(activeId, ignoreCase = true) }
            } ?: vault.accounts.firstOrNull()

            var restoredSummary: SessionSummary? = null
            var latestRecordForRefresh: StoredAccountRecord? = null

            if (activeRecord != null) {
                _uiState.update { it.copy(startupStatus = "Restoring ${activeRecord.riotId}...") }
                if (activeRecord.ssoCookies.isNullOrBlank() && vault.accounts.size == 1) {
                    val webCookie = runCatching {
                        CookieManager.getInstance().getCookie("https://auth.riotgames.com")
                    }.getOrNull()
                    if (!webCookie.isNullOrBlank()) {
                        vault = withContext(Dispatchers.IO) {
                            sessionStore.attachCookiesIfBlank(activeRecord.puuid, webCookie)
                        }
                    }
                }

                val latestRecord = vault.accounts.firstOrNull {
                    it.puuid.equals(activeRecord.puuid, ignoreCase = true)
                } ?: activeRecord
                latestRecordForRefresh = latestRecord

                val (restored, cachedStore, cachedCareer) = withContext(Dispatchers.IO) {
                    val r = runCatching { engine.restoreSessionJson(latestRecord.sessionJson) }.getOrNull()
                    val st = if (r != null) runCatching { engine.getCachedStorefront() }.getOrNull() else null
                    val ca = if (r != null) runCatching { engine.getCachedCareer() }.getOrNull() else null
                    Triple(r, st, ca)
                }

                if (restored != null) {
                    restoredSummary = restored
                    if (cachedStore != null) {
                        lastStoreTickElapsedMs = SystemClock.elapsedRealtime()
                    }
                    _uiState.update {
                        it.copy(
                            session = restored,
                            savedAccounts = vault.accounts,
                            storefront = cachedStore ?: it.storefront,
                            career = cachedCareer ?: it.career,
                            liveState = LiveState.Idle(
                                message = "Checking game status...",
                                recommendedPollSecs = 10u,
                            ),
                            isBootstrapping = false,
                        )
                    }
                } else {
                    _uiState.update { it.copy(savedAccounts = vault.accounts, isBootstrapping = false) }
                }
            } else {
                _uiState.update { it.copy(isBootstrapping = false) }
            }

            // Everything below runs asynchronously AFTER the populated UI is already visible!
            startCountdownTicker()
            checkForUpdates(manual = false)
            if (needsResync) {
                syncAssetsNow()
            }

            if (restoredSummary != null && latestRecordForRefresh != null) {
                val nowEpochSecs = System.currentTimeMillis() / 1000L
                if (restoredSummary.expiresAtEpochSecs <= nowEpochSecs + 300L) {
                    beginNetworkOp("Refreshing Riot session token...")
                    try {
                        val refreshed = trySilentCookieRefresh(latestRecordForRefresh)
                        if (refreshed != null && _uiState.value.session?.puuid.equals(refreshed.puuid, ignoreCase = true)) {
                            val refreshedAccounts = withContext(Dispatchers.IO) { sessionStore.loadVault().accounts }
                            _uiState.update {
                                it.copy(
                                    session = refreshed,
                                    savedAccounts = refreshedAccounts,
                                )
                            }
                        }
                    } finally {
                        endNetworkOp()
                    }
                }
                refreshAllAuthenticatedData()
            }
        }
    }

    private suspend fun trySilentCookieRefresh(targetRecord: StoredAccountRecord? = null): SessionSummary? {
        val activePuuid = targetRecord?.puuid ?: _uiState.value.session?.puuid ?: return null
        val vault = withContext(Dispatchers.IO) { sessionStore.loadVault() }
        val record = targetRecord ?: vault.accounts.firstOrNull { it.puuid.equals(activePuuid, ignoreCase = true) }
        val storedCookie = record?.ssoCookies?.takeIf { it.isNotBlank() }
        val webViewFallback = if (storedCookie == null && vault.accounts.size <= 1) {
            runCatching { CookieManager.getInstance().getCookie("https://auth.riotgames.com") }.getOrNull()
        } else {
            null
        }
        val cookie = storedCookie ?: webViewFallback
        if (cookie.isNullOrBlank()) return null

        return withContext(Dispatchers.IO) {
            runCatching {
                val refreshed = engine.refreshSessionFromCookie(cookie)
                if (!refreshed.puuid.equals(activePuuid, ignoreCase = true)) {
                    record?.sessionJson?.let { runCatching { engine.restoreSessionJson(it) } }
                    return@runCatching null
                }
                val exported = engine.exportSessionJson()
                if (exported != null) {
                    val updatedVault = sessionStore.upsertAccount(refreshed, exported, cookie, setActive = true)
                    _uiState.update { it.copy(savedAccounts = updatedVault.accounts) }
                }
                Log.d("MyVal", "Silently refreshed Riot token for ${refreshed.riotId}")
                refreshed
            }.onFailure { err ->
                Log.d("MyVal", "Silent cookie refresh skipped: ${err.message}")
            }.getOrNull()
        }
    }

    private fun refreshAllAuthenticatedData() {
        // Prioritize Live Hub polling + instant SQLite cache hydration for the Home Dashboard.
        // Heavy tab-specific payloads (Loadout, Owned Agents, Collection Summary, 1400-skin DB) load lazily on demand.
        restartPollingLoopIfNeeded()
        loadCachedOrLiveStorefront()
        loadCachedOrLiveCareer()
    }

    private fun loadCachedOrLiveCareer() {
        val requestPuuid = _uiState.value.session?.puuid ?: return
        viewModelScope.launch {
            val cached = withContext(Dispatchers.IO) {
                runCatching { engine.getCachedCareer() }.getOrNull()
            }
            if (cached != null && _uiState.value.session?.puuid.equals(requestPuuid, ignoreCase = true)) {
                _uiState.update { it.copy(career = cached) }
            }
            refreshCareer()
        }
    }

    private fun refreshOwnedAgents() {
        val requestPuuid = _uiState.value.session?.puuid ?: return
        viewModelScope.launch {
            val agents = executeWithAutoRefresh(requestPuuid) {
                engine.getOwnedAgentUuids()
                    .map { it.lowercase() }
                    .toSet()
            }.getOrDefault(emptySet())
            if (!_uiState.value.session?.puuid.equals(requestPuuid, ignoreCase = true)) return@launch
            if (agents.isNotEmpty()) {
                _uiState.update { it.copy(ownedAgentUuids = agents) }
            }
        }
    }

    private suspend fun <T> executeWithAutoRefresh(
        requestPuuid: String,
        statusLabel: String? = null,
        block: suspend () -> T,
    ): Result<T> {
        if (statusLabel != null) beginNetworkOp(statusLabel)
        return try {
            val first = withContext(Dispatchers.IO) {
                try {
                    Result.success(block())
                } catch (c: CancellationException) {
                    throw c
                } catch (t: Throwable) {
                    Result.failure(t)
                }
            }
            val err = first.exceptionOrNull()
            if (err is FfiException.Unauthorized && _uiState.value.session?.puuid.equals(requestPuuid, ignoreCase = true)) {
                val refreshed = trySilentCookieRefresh()
                if (refreshed != null && refreshed.puuid.equals(requestPuuid, ignoreCase = true)) {
                    _uiState.update { it.copy(session = refreshed) }
                    return withContext(Dispatchers.IO) {
                        try {
                            Result.success(block())
                        } catch (c: CancellationException) {
                            throw c
                        } catch (t: Throwable) {
                            Result.failure(t)
                        }
                    }
                }
            }
            first
        } finally {
            if (statusLabel != null) endNetworkOp()
        }
    }

    fun selectTab(tab: AppTab) {
        if (_uiState.value.selectedTab == tab) return
        _uiState.update { it.copy(selectedTab = tab) }
        restartPollingLoopIfNeeded()
        when (tab) {
            AppTab.LOADOUT -> ensureWeaponSkinsLoaded()
            AppTab.EXPLORER -> {
                ensureExplorerCatalogLoaded()
                ensureWeaponSkinsLoaded()
            }
            else -> Unit
        }
        if (_uiState.value.session != null) {
            when (tab) {
                AppTab.STORE -> {
                    if (_uiState.value.storefront == null && !_uiState.value.isLoadingStore) {
                        refreshStorefront()
                    } else if (_uiState.value.collectionSummary == null) {
                        refreshCollectionSummaryInBackground()
                    }
                }
                AppTab.CAREER -> if (_uiState.value.career == null && !_uiState.value.isLoadingCareer) refreshCareer()
                AppTab.LOADOUT -> if (!_uiState.value.isLoadingLoadout) refreshLoadout(force = false)
                else -> Unit
            }
        }
    }

    fun onForegroundChanged(foregrounded: Boolean) {
        _uiState.update { it.copy(isAppForegrounded = foregrounded) }
        restartPollingLoopIfNeeded()
        if (foregrounded &&
            _uiState.value.selectedTab == AppTab.LOADOUT &&
            _uiState.value.session != null &&
            !_uiState.value.isLoadingLoadout
        ) {
            refreshLoadout(force = false)
        }
    }

    fun openAccountsSheet() {
        _uiState.update { it.copy(showAccountsSheet = true) }
    }

    fun closeAccountsSheet() {
        _uiState.update { it.copy(showAccountsSheet = false) }
    }

    fun openRiotLogin() {
        val hasSavedAccounts = _uiState.value.savedAccounts.isNotEmpty()
        _uiState.update {
            it.copy(
                showAccountsSheet = false,
                showLoginWebView = true,
                isAddingAnotherAccount = hasSavedAccounts,
                authError = null,
            )
        }
    }

    fun refreshActiveAccount() {
        val currentSession = _uiState.value.session
        if (currentSession == null) {
            openRiotLogin()
            return
        }
        viewModelScope.launch {
            _uiState.update { it.copy(showAccountsSheet = false) }
            val refreshed = trySilentCookieRefresh()
            if (refreshed != null && refreshed.puuid.equals(currentSession.puuid, ignoreCase = true)) {
                val updatedAccounts = withContext(Dispatchers.IO) { sessionStore.loadVault().accounts }
                _uiState.update {
                    it.copy(
                        session = refreshed,
                        savedAccounts = updatedAccounts,
                        liveError = null,
                        storeError = null,
                        careerError = null,
                        loadoutError = null,
                        actionBanner = "Refreshed session for ${refreshed.riotId}",
                    )
                }
                refreshAllAuthenticatedData()
            } else {
                _uiState.update {
                    it.copy(
                        showLoginWebView = true,
                        isAddingAnotherAccount = true,
                        authError = null,
                        actionBanner = "Sign in to refresh ${currentSession.riotId}",
                    )
                }
            }
        }
    }

    fun openAddAccountLogin() {
        viewModelScope.launch {
            // Back up active account's WebView SSO cookie before opening a clean cookie session for the new account
            // Only read WebView cookies when at most 1 account is saved so another account's later cookies cannot overwrite an earlier account.
            val activePuuid = _uiState.value.session?.puuid
            if (activePuuid != null && _uiState.value.savedAccounts.size <= 1) {
                val currentCookie = runCatching {
                    CookieManager.getInstance().getCookie("https://auth.riotgames.com")
                }.getOrNull()
                if (!currentCookie.isNullOrBlank()) {
                    val updated = withContext(Dispatchers.IO) {
                        sessionStore.attachCookiesIfBlank(activePuuid, currentCookie)
                    }
                    _uiState.update { it.copy(savedAccounts = updated.accounts) }
                }
            }
            _uiState.update {
                it.copy(
                    showAccountsSheet = false,
                    showLoginWebView = true,
                    isAddingAnotherAccount = true,
                    authError = null,
                )
            }
        }
    }

    fun closeRiotLogin() {
        _uiState.update {
            it.copy(
                showLoginWebView = false,
                isAddingAnotherAccount = false,
                isSigningIn = false,
            )
        }
    }

    fun getRiotLoginUrl(): String = engine.getRiotLoginUrl()

    fun completeLoginFromRedirect(redirectUrl: String, capturedCookies: String? = null) {
        if (_uiState.value.isSigningIn) return
        viewModelScope.launch {
            _uiState.update { it.copy(isSigningIn = true, authError = null) }
            val previousPuuid = _uiState.value.session?.puuid
            withContext(Dispatchers.IO) {
                runCatching {
                    val summary = engine.completeAuthFromRedirectUrl(redirectUrl)
                    val exported = engine.exportSessionJson()
                    val vault = if (exported != null) {
                        sessionStore.upsertAccount(summary, exported, capturedCookies, setActive = true)
                    } else {
                        sessionStore.loadVault()
                    }
                    summary to vault.accounts
                }
            }.onSuccess { (summary, accounts) ->
                livePollingJob?.cancel()
                val switchedUser = previousPuuid == null || !previousPuuid.equals(summary.puuid, ignoreCase = true)
                if (switchedUser) {
                    pregameFirstSeen = null
                    lastLoadoutFetchElapsedMs = 0L
                    lastCoreGameMatchId = null
                    lastStoreTickElapsedMs = SystemClock.elapsedRealtime()
                }
                _uiState.update { state ->
                    state.copy(
                        session = summary,
                        savedAccounts = accounts,
                        isSigningIn = false,
                        showLoginWebView = false,
                        isAddingAnotherAccount = false,
                        storefront = if (switchedUser) null else state.storefront,
                        collectionSummary = if (switchedUser) null else state.collectionSummary,
                        ownedSkinUuids = if (switchedUser) emptySet() else state.ownedSkinUuids,
                        ownedChromaUuids = if (switchedUser) emptySet() else state.ownedChromaUuids,
                        ownedLevelUuids = if (switchedUser) emptySet() else state.ownedLevelUuids,
                        ownedSprayUuids = if (switchedUser) emptySet() else state.ownedSprayUuids,
                        ownedCardUuids = if (switchedUser) emptySet() else state.ownedCardUuids,
                        ownedTitleUuids = if (switchedUser) emptySet() else state.ownedTitleUuids,
                        ownedAgentUuids = if (switchedUser) emptySet() else state.ownedAgentUuids,
                        career = if (switchedUser) null else state.career,
                        selectedMatchDetail = if (switchedUser) null else state.selectedMatchDetail,
                        returnToMatchDetail = if (switchedUser) null else state.returnToMatchDetail,
                        inspectedPlayerPuuid = if (switchedUser) null else state.inspectedPlayerPuuid,
                        inspectedPlayerHintName = if (switchedUser) null else state.inspectedPlayerHintName,
                        inspectedPlayerProfile = if (switchedUser) null else state.inspectedPlayerProfile,
                        isLoadingPlayerProfile = if (switchedUser) false else state.isLoadingPlayerProfile,
                        playerProfileError = if (switchedUser) null else state.playerProfileError,
                        showPlayerSearchSheet = if (switchedUser) false else state.showPlayerSearchSheet,
                        loadout = if (switchedUser) null else state.loadout,
                        loadoutPresets = if (switchedUser) emptyList() else state.loadoutPresets,
                        pregameFirstSeenAtMs = if (switchedUser) null else state.pregameFirstSeenAtMs,
                        isPollingLive = false,
                        isManualPollingLive = false,
                        isMutatingLiveAction = false,
                        isLoadingStore = false,
                        isLoadingCareer = false,
                        isLoadingLoadout = false,
                        isLoadingMatchDetail = if (switchedUser) false else state.isLoadingMatchDetail,
                        isLookingUpRiotId = if (switchedUser) false else state.isLookingUpRiotId,
                        riotIdLookupError = if (switchedUser) null else state.riotIdLookupError,
                        liveError = null,
                        storeError = null,
                        careerError = null,
                        loadoutError = null,
                        liveState = LiveState.Idle(
                            message = "Checking game status...",
                            recommendedPollSecs = 10u,
                        ),
                        actionBanner = "Signed in as ${summary.riotId} (${summary.shard.uppercase()}) • ${accounts.size} account(s)",
                    )
                }
                refreshAllAuthenticatedData()
            }.onFailure { err ->
                _uiState.update {
                    it.copy(
                        isSigningIn = false,
                        authError = err.message ?: "Authentication failed",
                    )
                }
            }
        }
    }

    fun switchAccount(targetPuuid: String) {
        val currentPuuid = _uiState.value.session?.puuid
        if (currentPuuid != null && currentPuuid.equals(targetPuuid, ignoreCase = true)) {
            _uiState.update { it.copy(showAccountsSheet = false) }
            return
        }
        viewModelScope.launch {
            livePollingJob?.cancel()
            val vault = withContext(Dispatchers.IO) { sessionStore.loadVault() }
            val record = vault.accounts.firstOrNull { it.puuid.equals(targetPuuid, ignoreCase = true) }
            if (record == null) {
                _uiState.update {
                    it.copy(showAccountsSheet = false, actionBanner = "Saved account not found")
                }
                return@launch
            }

            pregameFirstSeen = null
            lastLoadoutFetchElapsedMs = 0L
            lastCoreGameMatchId = null
            lastStoreTickElapsedMs = SystemClock.elapsedRealtime()
            _uiState.update {
                it.copy(
                    session = null,
                    showAccountsSheet = false,
                    storefront = null,
                    collectionSummary = null,
                    ownedSkinUuids = emptySet(),
                    ownedChromaUuids = emptySet(),
                    ownedLevelUuids = emptySet(),
                    ownedSprayUuids = emptySet(),
                    ownedCardUuids = emptySet(),
                    ownedTitleUuids = emptySet(),
                    ownedAgentUuids = emptySet(),
                    career = null,
                    selectedMatchDetail = null,
                    returnToMatchDetail = null,
                    inspectedPlayerPuuid = null,
                    inspectedPlayerHintName = null,
                    inspectedPlayerProfile = null,
                    isLoadingPlayerProfile = false,
                    playerProfileError = null,
                    showPlayerSearchSheet = false,
                    loadout = null,
                    loadoutPresets = emptyList(),
                    pregameFirstSeenAtMs = null,
                    isPollingLive = false,
                    isManualPollingLive = false,
                    isLoadingStore = false,
                    isLoadingCareer = false,
                    isLoadingLoadout = false,
                    isLoadingMatchDetail = false,
                    isLookingUpRiotId = false,
                    riotIdLookupError = null,
                    isMutatingLiveAction = false,
                    liveError = null,
                    storeError = null,
                    careerError = null,
                    loadoutError = null,
                )
            }

            val switchedSession = withContext(Dispatchers.IO) {
                runCatching { engine.restoreSessionJson(record.sessionJson) }.getOrNull()
            }
            if (switchedSession == null) {
                _uiState.update {
                    it.copy(
                        liveState = LiveState.Idle(
                            message = "Session expired or corrupted. Sign in with Riot to continue.",
                            recommendedPollSecs = 10u,
                        ),
                        actionBanner = "Failed to restore ${record.riotId} — please re-sign in",
                    )
                }
                return@launch
            }

            withContext(Dispatchers.IO) { sessionStore.setActivePuuid(record.puuid) }
            val nowEpochSecs = System.currentTimeMillis() / 1000L
            val activeSummary = if (switchedSession.expiresAtEpochSecs <= nowEpochSecs + 300L) {
                trySilentCookieRefresh(record) ?: switchedSession
            } else {
                switchedSession
            }

            val latestAccounts = withContext(Dispatchers.IO) { sessionStore.loadVault().accounts }
            _uiState.update {
                it.copy(
                    session = activeSummary,
                    savedAccounts = latestAccounts,
                    liveState = LiveState.Idle(
                        message = "Checking game status...",
                        recommendedPollSecs = 10u,
                    ),
                    actionBanner = "Switched to ${activeSummary.riotId} (${activeSummary.shard.uppercase()})",
                )
            }
            refreshAllAuthenticatedData()
        }
    }

    fun removeAccount(targetPuuid: String) {
        viewModelScope.launch {
            val activePuuid = _uiState.value.session?.puuid
            val updatedVault = withContext(Dispatchers.IO) { sessionStore.removeAccount(targetPuuid) }
            val removedActive = activePuuid != null && activePuuid.equals(targetPuuid, ignoreCase = true)

            if (!removedActive) {
                _uiState.update {
                    it.copy(
                        savedAccounts = updatedVault.accounts,
                        actionBanner = "Removed account from device",
                    )
                }
                return@launch
            }

            livePollingJob?.cancel()
            val fallbackRecord = updatedVault.activePuuid?.let { nextId ->
                updatedVault.accounts.firstOrNull { it.puuid.equals(nextId, ignoreCase = true) }
            }

            if (fallbackRecord == null) {
                withContext(Dispatchers.IO) { engine.clearSession() }
                clearBrowserAuthState()
                clearAuthenticatedUiState(
                    accounts = emptyList(),
                    liveMessage = "Signed out. Sign in with Riot to stream live party, store, career & loadout.",
                    banner = "Signed out",
                )
            } else {
                _uiState.update { it.copy(savedAccounts = updatedVault.accounts) }
                switchAccount(fallbackRecord.puuid)
            }
        }
    }

    fun signOut() {
        viewModelScope.launch {
            livePollingJob?.cancel()
            withContext(Dispatchers.IO) {
                engine.clearSession()
                sessionStore.clearAll()
            }
            clearBrowserAuthState()
            clearAuthenticatedUiState(
                accounts = emptyList(),
                liveMessage = "Signed out of all accounts. Sign in with Riot to continue.",
                banner = "Signed out all Riot accounts",
            )
        }
    }

    private fun clearBrowserAuthState() {
        runCatching {
            WebStorage.getInstance().deleteAllData()
            val cm = CookieManager.getInstance()
            cm.removeAllCookies(null)
            cm.flush()
        }
    }

    private fun clearAuthenticatedUiState(
        accounts: List<StoredAccountRecord>,
        liveMessage: String,
        banner: String?,
    ) {
        lastLoadoutFetchElapsedMs = 0L
        lastCoreGameMatchId = null
        lastStoreTickElapsedMs = SystemClock.elapsedRealtime()
        pregameFirstSeen = null
        _uiState.update {
            it.copy(
                session = null,
                savedAccounts = accounts,
                showAccountsSheet = false,
                storefront = null,
                collectionSummary = null,
                ownedSkinUuids = emptySet(),
                ownedChromaUuids = emptySet(),
                ownedLevelUuids = emptySet(),
                ownedSprayUuids = emptySet(),
                ownedCardUuids = emptySet(),
                ownedTitleUuids = emptySet(),
                ownedAgentUuids = emptySet(),
                career = null,
                selectedMatchDetail = null,
                returnToMatchDetail = null,
                inspectedPlayerPuuid = null,
                inspectedPlayerHintName = null,
                inspectedPlayerProfile = null,
                isLoadingPlayerProfile = false,
                playerProfileError = null,
                showPlayerSearchSheet = false,
                loadout = null,
                loadoutPresets = emptyList(),
                isPollingLive = false,
                isManualPollingLive = false,
                isLoadingStore = false,
                isLoadingCareer = false,
                isLoadingLoadout = false,
                isLoadingMatchDetail = false,
                isLookingUpRiotId = false,
                riotIdLookupError = null,
                isMutatingLiveAction = false,
                pregameFirstSeenAtMs = null,
                liveError = null,
                storeError = null,
                careerError = null,
                loadoutError = null,
                liveState = LiveState.Idle(
                    message = liveMessage,
                    recommendedPollSecs = 10u,
                ),
                actionBanner = banner,
            )
        }
    }

    fun syncAssetsNow() {
        if (_uiState.value.isSyncingAssets) return
        viewModelScope.launch {
            _uiState.update { it.copy(isSyncingAssets = true, assetError = null) }
            beginNetworkOp("Syncing Valorant assets...")
            try {
                withContext(Dispatchers.IO) { runCatching { engine.syncAssets() } }
                    .onSuccess { summary ->
                        _uiState.update {
                            it.copy(
                                assetSummary = summary,
                                isSyncingAssets = false,
                                actionBanner = "Synced ${summary.skinCount} skins (v${summary.riotClientVersion})",
                            )
                        }
                        explorerLoaded = false
                        refreshStartupMetadataFromCache()
                        if (_uiState.value.selectedTab == AppTab.EXPLORER) {
                            ensureExplorerCatalogLoaded()
                        }
                        if (_uiState.value.selectedTab == AppTab.LOADOUT) {
                            ensureWeaponSkinsLoaded(force = true)
                        }
                        if (_uiState.value.session != null) {
                            refreshAllAuthenticatedData()
                        }
                    }
                    .onFailure { err ->
                        _uiState.update {
                            it.copy(
                                isSyncingAssets = false,
                                assetError = err.message ?: "Failed to sync valorant-api.com",
                            )
                        }
                    }
            } finally {
                endNetworkOp()
            }
        }
    }

    private suspend fun refreshStartupMetadataFromCache() {
        withContext(Dispatchers.IO) {
            val summary = runCatching { engine.getAssetSummary() }.getOrNull()
            val agents = runCatching { engine.listAgents() }.getOrDefault(emptyList())
            val hasKey = runCatching { engine.hasLookupApiKey() }.getOrDefault(false)
            _uiState.update {
                it.copy(
                    assetSummary = summary,
                    agents = agents,
                    hasLookupApiKey = hasKey,
                )
            }
        }
    }

    private fun ensureExplorerCatalogLoaded() {
        if (explorerLoaded && (_uiState.value.maps.isNotEmpty() || _uiState.value.skins.isNotEmpty())) return
        viewModelScope.launch(Dispatchers.IO) {
            val maps = runCatching { engine.listMaps() }.getOrDefault(emptyList())
            val q = _uiState.value.skinQuery
            val wishOnly = _uiState.value.wishlistOnly
            val skins = runCatching { engine.searchSkins(q, wishOnly, 250u) }.getOrDefault(emptyList())
            explorerLoaded = true
            _uiState.update {
                it.copy(
                    maps = maps,
                    skins = skins,
                )
            }
        }
    }

    private fun ensureWeaponSkinsLoaded(force: Boolean = false) {
        val s = _uiState.value
        if (!force &&
            s.allWeaponSkins.isNotEmpty() &&
            s.playerCards.isNotEmpty() &&
            s.playerTitles.isNotEmpty() &&
            s.sprays.isNotEmpty()
        ) {
            return
        }
        viewModelScope.launch(Dispatchers.IO) {
            val allWeaponSkins = runCatching { engine.listAllWeaponSkins() }
                .getOrDefault(emptyList())
                .ifEmpty { runCatching { engine.searchSkins("", false, 2000u) }.getOrDefault(emptyList()) }
            val playerCards = runCatching { engine.listPlayerCards() }.getOrDefault(emptyList())
            val playerTitles = runCatching { engine.listPlayerTitles() }.getOrDefault(emptyList())
            val sprays = runCatching { engine.listSprays() }.getOrDefault(emptyList())
            _uiState.update {
                it.copy(
                    allWeaponSkins = if (allWeaponSkins.isNotEmpty()) allWeaponSkins else it.allWeaponSkins,
                    playerCards = if (playerCards.isNotEmpty()) playerCards else it.playerCards,
                    playerTitles = if (playerTitles.isNotEmpty()) playerTitles else it.playerTitles,
                    sprays = if (sprays.isNotEmpty()) sprays else it.sprays,
                )
            }
        }
    }

    fun updateSkinSearch(query: String, wishlistOnly: Boolean = _uiState.value.wishlistOnly) {
        _uiState.update { it.copy(skinQuery = query, wishlistOnly = wishlistOnly) }
        viewModelScope.launch(Dispatchers.IO) {
            val skins = runCatching { engine.searchSkins(query, wishlistOnly, 250u) }.getOrDefault(emptyList())
            _uiState.update { it.copy(skins = skins) }
        }
    }

    fun toggleWishlist(skinUuid: String) {
        viewModelScope.launch {
            val nowWishlisted = withContext(Dispatchers.IO) {
                runCatching { engine.toggleWishlist(skinUuid) }.getOrNull()
            }
            if (nowWishlisted != null) {
                _uiState.update { state ->
                    val updatedSkins = state.skins.mapNotNull { s ->
                        if (s.uuid.equals(skinUuid, ignoreCase = true)) {
                            if (state.wishlistOnly && !nowWishlisted) null else s.copy(isWishlisted = nowWishlisted)
                        } else {
                            s
                        }
                    }
                    val updatedAllWeaponSkins = state.allWeaponSkins.map { s ->
                        if (s.uuid.equals(skinUuid, ignoreCase = true)) s.copy(isWishlisted = nowWishlisted) else s
                    }
                    val updatedStore = state.storefront?.let { store ->
                        store.copy(
                            dailyOffers = store.dailyOffers.map { offer ->
                                if (offer.skinUuid.equals(skinUuid, ignoreCase = true)) {
                                    offer.copy(isWishlisted = nowWishlisted)
                                } else {
                                    offer
                                }
                            },
                            nightMarket = store.nightMarket?.let { nm ->
                                nm.copy(
                                    offers = nm.offers.map { o ->
                                        if (o.skinUuid.equals(skinUuid, ignoreCase = true)) {
                                            o.copy(isWishlisted = nowWishlisted)
                                        } else {
                                            o
                                        }
                                    },
                                )
                            },
                        )
                    }
                    state.copy(
                        skins = updatedSkins,
                        allWeaponSkins = updatedAllWeaponSkins,
                        storefront = updatedStore,
                    )
                }
            }
        }
    }

    // ---- Career & 2D Match Replay ----

    fun refreshCareer() {
        val requestPuuid = _uiState.value.session?.puuid ?: return
        if (_uiState.value.isLoadingCareer) return
        viewModelScope.launch {
            _uiState.update { it.copy(isLoadingCareer = true, careerError = null) }
            executeWithAutoRefresh(requestPuuid, statusLabel = "Loading Career...") {
                engine.fetchCareer()
            }
                .onSuccess { career ->
                    val updatedVault = withContext(Dispatchers.IO) {
                        sessionStore.updateAccountRank(
                            puuid = requestPuuid,
                            rankTierName = career.currentRank.tierName,
                            rankColorHex = career.currentRank.colorHex,
                            rankIconUrl = career.currentRank.iconUrl,
                        )
                    }
                    if (!_uiState.value.session?.puuid.equals(requestPuuid, ignoreCase = true)) {
                        _uiState.update { it.copy(savedAccounts = updatedVault.accounts) }
                        return@onSuccess
                    }
                    Log.d("MyVal", "Loaded Career: rank=${career.currentRank.tierName} (${career.currentRank.rr} RR), matches=${career.recentMatches.size}")
                    _uiState.update {
                        it.copy(
                            career = career,
                            savedAccounts = updatedVault.accounts,
                            isLoadingCareer = false,
                            careerError = null,
                        )
                    }
                }
                .onFailure { err ->
                    if (!_uiState.value.session?.puuid.equals(requestPuuid, ignoreCase = true)) return@onFailure
                    Log.e("MyVal", "Career fetch failed: ${err.message}", err)
                    _uiState.update {
                        it.copy(
                            isLoadingCareer = false,
                            careerError = err.message ?: "Failed to load career data",
                        )
                    }
                }
        }
    }

    fun openMatchDetail(matchId: String) {
        val requestPuuid = _uiState.value.session?.puuid ?: return
        openMatchDetailForPlayer(matchId, requestPuuid)
    }

    fun openMatchDetailForPlayer(matchId: String, perspectivePuuid: String) {
        val requestPuuid = _uiState.value.session?.puuid ?: return
        if (_uiState.value.isLoadingMatchDetail) return
        viewModelScope.launch {
            _uiState.update { it.copy(isLoadingMatchDetail = true) }
            try {
                executeWithAutoRefresh(requestPuuid, statusLabel = "Loading Match...") {
                    engine.fetchMatchDetailForPlayer(matchId, perspectivePuuid)
                }
                    .onSuccess { detail ->
                        if (!_uiState.value.session?.puuid.equals(requestPuuid, ignoreCase = true)) return@onSuccess
                        _uiState.update {
                            it.copy(
                                selectedMatchDetail = detail,
                                returnToMatchDetail = null,
                                isLoadingMatchDetail = false,
                            )
                        }
                    }
                    .onFailure { err ->
                        if (!_uiState.value.session?.puuid.equals(requestPuuid, ignoreCase = true)) return@onFailure
                        _uiState.update {
                            it.copy(
                                isLoadingMatchDetail = false,
                                actionBanner = err.message ?: "Failed to load match",
                            )
                        }
                    }
            } finally {
                if (_uiState.value.session?.puuid.equals(requestPuuid, ignoreCase = true)) {
                    _uiState.update { it.copy(isLoadingMatchDetail = false) }
                }
            }
        }
    }

    fun closeMatchDetail() {
        _uiState.update { it.copy(selectedMatchDetail = null) }
    }

    // ---- Player Stats Profile & Riot ID Lookup ----

    fun openPlayerProfile(puuid: String, hintName: String, forceRefresh: Boolean = false) {
        val requestPuuid = _uiState.value.session?.puuid ?: return
        val cleanPuuid = puuid.trim()
        if (cleanPuuid.isEmpty()) return

        val samePlayer = _uiState.value.inspectedPlayerPuuid.equals(cleanPuuid, ignoreCase = true)
        _uiState.update { state ->
            val savedMatch = state.selectedMatchDetail ?: state.returnToMatchDetail
            state.copy(
                selectedMatchDetail = null,
                returnToMatchDetail = savedMatch,
                showPlayerSearchSheet = false,
                inspectedPlayerPuuid = cleanPuuid,
                inspectedPlayerHintName = hintName.takeIf { n -> n.isNotBlank() } ?: state.inspectedPlayerHintName,
                inspectedPlayerProfile = if (samePlayer) state.inspectedPlayerProfile else null,
                isLoadingPlayerProfile = true,
                playerProfileError = null,
            )
        }

        viewModelScope.launch {
            executeWithAutoRefresh(requestPuuid, statusLabel = "Loading profile...") {
                engine.fetchPlayerProfile(cleanPuuid, forceRefresh)
            }
                .onSuccess { profile ->
                    if (!_uiState.value.session?.puuid.equals(requestPuuid, ignoreCase = true)) return@onSuccess
                    if (!_uiState.value.inspectedPlayerPuuid.equals(cleanPuuid, ignoreCase = true)) return@onSuccess
                    _uiState.update {
                        it.copy(
                            inspectedPlayerProfile = profile,
                            inspectedPlayerHintName = profile.displayName,
                            isLoadingPlayerProfile = false,
                            playerProfileError = null,
                        )
                    }
                }
                .onFailure { err ->
                    if (!_uiState.value.session?.puuid.equals(requestPuuid, ignoreCase = true)) return@onFailure
                    if (!_uiState.value.inspectedPlayerPuuid.equals(cleanPuuid, ignoreCase = true)) return@onFailure
                    _uiState.update {
                        it.copy(
                            isLoadingPlayerProfile = false,
                            playerProfileError = err.message ?: "Failed to load player stats",
                        )
                    }
                }
        }
    }

    fun refreshInspectedPlayerProfile() {
        val puuid = _uiState.value.inspectedPlayerPuuid ?: return
        val hint = _uiState.value.inspectedPlayerProfile?.displayName
            ?: _uiState.value.inspectedPlayerHintName
            ?: "Player"
        openPlayerProfile(puuid, hint, forceRefresh = true)
    }

    fun closePlayerProfile() {
        _uiState.update { state ->
            state.copy(
                selectedMatchDetail = state.returnToMatchDetail,
                returnToMatchDetail = null,
                inspectedPlayerPuuid = null,
                inspectedPlayerHintName = null,
                inspectedPlayerProfile = null,
                isLoadingPlayerProfile = false,
                playerProfileError = null,
            )
        }
    }

    fun openPlayerSearch() {
        _uiState.update { it.copy(showPlayerSearchSheet = true, riotIdLookupError = null) }
        updatePlayerSearchQuery(_uiState.value.playerSearchQuery)
    }

    fun closePlayerSearch() {
        _uiState.update { it.copy(showPlayerSearchSheet = false, riotIdLookupError = null) }
    }

    fun updatePlayerSearchQuery(query: String) {
        _uiState.update { it.copy(playerSearchQuery = query, riotIdLookupError = null) }
        viewModelScope.launch(Dispatchers.IO) {
            val found = runCatching { engine.searchKnownPlayers(query, 40u) }.getOrDefault(emptyList())
            _uiState.update { it.copy(knownPlayers = found) }
        }
    }

    fun lookupPlayerByRiotId(riotId: String) {
        val requestPuuid = _uiState.value.session?.puuid ?: return
        val clean = riotId.trim()
        if (clean.isEmpty() || _uiState.value.isLookingUpRiotId) return
        viewModelScope.launch {
            _uiState.update { it.copy(isLookingUpRiotId = true, riotIdLookupError = null) }
            try {
                executeWithAutoRefresh(requestPuuid, statusLabel = "Searching player...") {
                    engine.lookupPlayerProfileByRiotId(clean)
                }
                    .onSuccess { profile ->
                        if (!_uiState.value.session?.puuid.equals(requestPuuid, ignoreCase = true)) return@onSuccess
                        _uiState.update {
                            it.copy(
                                isLookingUpRiotId = false,
                                riotIdLookupError = null,
                                showPlayerSearchSheet = false,
                                selectedMatchDetail = null,
                                inspectedPlayerPuuid = profile.puuid,
                                inspectedPlayerHintName = profile.displayName,
                                inspectedPlayerProfile = profile,
                                isLoadingPlayerProfile = false,
                                playerProfileError = null,
                            )
                        }
                    }
                    .onFailure { err ->
                        if (!_uiState.value.session?.puuid.equals(requestPuuid, ignoreCase = true)) return@onFailure
                        _uiState.update {
                            it.copy(
                                isLookingUpRiotId = false,
                                riotIdLookupError = err.message ?: "Failed to look up $clean",
                            )
                        }
                    }
            } finally {
                if (_uiState.value.session?.puuid.equals(requestPuuid, ignoreCase = true)) {
                    _uiState.update { it.copy(isLookingUpRiotId = false) }
                }
            }
        }
    }

    fun saveLookupApiKey(key: String) {
        viewModelScope.launch(Dispatchers.IO) {
            runCatching { engine.setLookupApiKey(key) }
                .onSuccess {
                    val active = runCatching { engine.hasLookupApiKey() }.getOrDefault(false)
                    _uiState.update {
                        it.copy(
                            hasLookupApiKey = active,
                            riotIdLookupError = null,
                            actionBanner = if (active) "Saved global lookup API key" else "Cleared lookup API key",
                        )
                    }
                }
        }
    }

    // ---- Loadout, Presets & Randomizer ----

    fun refreshLoadout(force: Boolean = false) {
        val requestPuuid = _uiState.value.session?.puuid ?: return
        if (_uiState.value.isLoadingLoadout) return
        val nowElapsed = SystemClock.elapsedRealtime()
        if (!force && _uiState.value.loadout != null && (nowElapsed - lastLoadoutFetchElapsedMs) < 8_000L) {
            return
        }
        viewModelScope.launch {
            _uiState.update { it.copy(isLoadingLoadout = true, loadoutError = null) }
            val presets = withContext(Dispatchers.IO) {
                runCatching { engine.listLoadoutPresets() }.getOrDefault(emptyList())
            }
            val bundleRes = executeWithAutoRefresh(requestPuuid, statusLabel = "Loading Arsenal...") {
                val col = runCatching { engine.fetchCollectionSummary() }.getOrNull()
                val own = runCatching { engine.getOwnedSkinUuids() }
                    .getOrDefault(emptyList())
                    .map { it.lowercase() }
                    .toSet()
                val chromas = runCatching { engine.getOwnedChromaUuids() }
                    .getOrDefault(emptyList())
                    .map { it.lowercase() }
                    .toSet()
                val levels = runCatching { engine.getOwnedLevelUuids() }
                    .getOrDefault(emptyList())
                    .map { it.lowercase() }
                    .toSet()
                val ownedSprays = runCatching { engine.getOwnedSprayUuids() }
                    .getOrDefault(emptyList())
                    .map { it.lowercase() }
                    .toSet()
                val ownedCards = runCatching { engine.getOwnedCardUuids() }
                    .getOrDefault(emptyList())
                    .map { it.lowercase() }
                    .toSet()
                val ownedTitles = runCatching { engine.getOwnedTitleUuids() }
                    .getOrDefault(emptyList())
                    .map { it.lowercase() }
                    .toSet()
                val lo = engine.fetchPlayerLoadout()
                LoadoutFetchBundle(
                    collection = col,
                    ownedSkins = own,
                    ownedChromas = chromas,
                    ownedLevels = levels,
                    ownedSprays = ownedSprays,
                    ownedCards = ownedCards,
                    ownedTitles = ownedTitles,
                    loadout = lo,
                )
            }

            if (!_uiState.value.session?.puuid.equals(requestPuuid, ignoreCase = true)) return@launch

            bundleRes
                .onSuccess { bundle ->
                    lastLoadoutFetchElapsedMs = SystemClock.elapsedRealtime()
                    val topGuns = bundle.loadout.guns.take(5).joinToString { "${it.weaponName}=${it.skinName}" }
                    Log.d("MyVal", "Loaded Real PlayerLoadout: card=${bundle.loadout.playerCardName}, title=${bundle.loadout.playerTitleText}, ownedSkins=${bundle.ownedSkins.size}, ownedChromas=${bundle.ownedChromas.size}, ownedLevels=${bundle.ownedLevels.size}, sprays=${bundle.loadout.sprays.size}, liveSynced=${bundle.loadout.isLiveMatchSynced}, guns=[$topGuns]")
                    _uiState.update {
                        it.copy(
                            loadout = bundle.loadout,
                            isLoadingLoadout = false,
                            loadoutError = null,
                            loadoutPresets = presets,
                            collectionSummary = bundle.collection ?: it.collectionSummary,
                            ownedSkinUuids = if (bundle.ownedSkins.isNotEmpty()) bundle.ownedSkins else it.ownedSkinUuids,
                            ownedChromaUuids = if (bundle.ownedChromas.isNotEmpty()) bundle.ownedChromas else it.ownedChromaUuids,
                            ownedLevelUuids = if (bundle.ownedLevels.isNotEmpty()) bundle.ownedLevels else it.ownedLevelUuids,
                            ownedSprayUuids = if (bundle.ownedSprays.isNotEmpty()) bundle.ownedSprays else it.ownedSprayUuids,
                            ownedCardUuids = if (bundle.ownedCards.isNotEmpty()) bundle.ownedCards else it.ownedCardUuids,
                            ownedTitleUuids = if (bundle.ownedTitles.isNotEmpty()) bundle.ownedTitles else it.ownedTitleUuids,
                        )
                    }
                }
                .onFailure { err ->
                    Log.e("MyVal", "Loadout fetch failed: ${err.message}", err)
                    val friendlyErr = if (err is FfiException.NotInSession) {
                        "Open Valorant on PC once to sync your active loadout"
                    } else {
                        err.message ?: "Failed to load active loadout"
                    }
                    _uiState.update {
                        it.copy(
                            isLoadingLoadout = false,
                            loadoutError = friendlyErr,
                            loadoutPresets = presets,
                        )
                    }
                }
        }
    }

    private fun loadoutSavedBanner(defaultMessage: String): String =
        if (_uiState.value.liveState is LiveState.CoreGame) {
            "Saved! Riot locks live-match skins — applies next match."
        } else {
            defaultMessage
        }

    private fun runLoadoutMutation(
        successMessage: String,
        failureMessage: String,
        mutate: suspend () -> PlayerLoadoutSnapshot,
    ) {
        val requestPuuid = _uiState.value.session?.puuid ?: return
        if (_uiState.value.isLoadingLoadout) return
        viewModelScope.launch {
            _uiState.update { it.copy(isLoadingLoadout = true, loadoutError = null) }
            executeWithAutoRefresh(requestPuuid) { mutate() }
                .onSuccess { updated ->
                    if (!_uiState.value.session?.puuid.equals(requestPuuid, ignoreCase = true)) return@onSuccess
                    lastLoadoutFetchElapsedMs = SystemClock.elapsedRealtime()
                    _uiState.update {
                        it.copy(
                            loadout = updated,
                            isLoadingLoadout = false,
                            actionBanner = loadoutSavedBanner(successMessage),
                        )
                    }
                }
                .onFailure { err ->
                    Log.e("MyVal", "Loadout mutation failed: ${err.message}", err)
                    if (!_uiState.value.session?.puuid.equals(requestPuuid, ignoreCase = true)) return@onFailure
                    _uiState.update {
                        it.copy(
                            isLoadingLoadout = false,
                            loadoutError = err.message ?: failureMessage,
                            actionBanner = err.message ?: failureMessage,
                        )
                    }
                }
        }
    }

    fun equipWeaponSkin(
        weaponUuid: String,
        skinUuid: String,
        chromaUuid: String? = null,
        levelUuid: String? = null,
    ) {
        runLoadoutMutation(
            successMessage = "Equipped & synced to Riot!",
            failureMessage = "Failed to equip skin",
        ) {
            engine.equipWeaponSkinFull(weaponUuid, skinUuid, chromaUuid, levelUuid)
        }
    }

    fun equipSpray(slotUuid: String, sprayUuid: String) {
        runLoadoutMutation(
            successMessage = "Spray equipped & synced to Riot!",
            failureMessage = "Failed to equip spray",
        ) {
            engine.equipSpray(slotUuid, sprayUuid)
        }
    }

    fun equipPlayerCard(cardUuid: String) {
        runLoadoutMutation(
            successMessage = "Player Card equipped & synced to Riot!",
            failureMessage = "Failed to equip player card",
        ) {
            engine.equipPlayerCard(cardUuid)
        }
    }

    fun equipPlayerTitle(titleUuid: String) {
        runLoadoutMutation(
            successMessage = if (titleUuid.isBlank()) "Cleared player title!" else "Player Title updated!",
            failureMessage = "Failed to update player title",
        ) {
            engine.equipPlayerTitle(titleUuid)
        }
    }

    fun setIdentityPrivacy(incognito: Boolean, hideAccountLevel: Boolean) {
        runLoadoutMutation(
            successMessage = "Privacy settings updated on Riot!",
            failureMessage = "Failed to update privacy settings",
        ) {
            engine.setIdentityPrivacy(incognito, hideAccountLevel)
        }
    }

    fun randomizeLoadout() {
        if (_uiState.value.ownedSkinUuids.isEmpty()) {
            _uiState.update { it.copy(actionBanner = "No owned weapon skins found to randomize") }
            return
        }
        val seed = System.currentTimeMillis().toULong()
        runLoadoutMutation(
            successMessage = "Randomized across owned skins!",
            failureMessage = "Failed to randomize loadout",
        ) {
            engine.randomizeLoadout(seed)
        }
    }

    fun saveLoadoutPreset(name: String, accentHex: String) {
        val requestPuuid = _uiState.value.session?.puuid ?: return
        viewModelScope.launch {
            withContext(Dispatchers.IO) { runCatching { engine.saveCurrentLoadoutPreset(name, accentHex) } }
                .onSuccess { preset ->
                    val presets = withContext(Dispatchers.IO) {
                        runCatching { engine.listLoadoutPresets() }.getOrDefault(emptyList())
                    }
                    if (!_uiState.value.session?.puuid.equals(requestPuuid, ignoreCase = true)) return@onSuccess
                    _uiState.update {
                        it.copy(
                            loadoutPresets = presets,
                            actionBanner = "Saved preset '${preset.name}'",
                        )
                    }
                }
                .onFailure { err ->
                    _uiState.update {
                        it.copy(actionBanner = err.message ?: "Failed to save preset")
                    }
                }
        }
    }

    fun applyLoadoutPreset(presetId: String) {
        val requestPuuid = _uiState.value.session?.puuid ?: return
        if (_uiState.value.isLoadingLoadout) return
        viewModelScope.launch {
            _uiState.update { it.copy(isLoadingLoadout = true, loadoutError = null) }
            executeWithAutoRefresh(requestPuuid) { engine.applyLoadoutPreset(presetId) }
                .onSuccess { updated ->
                    if (!_uiState.value.session?.puuid.equals(requestPuuid, ignoreCase = true)) return@onSuccess
                    lastLoadoutFetchElapsedMs = SystemClock.elapsedRealtime()
                    _uiState.update {
                        it.copy(
                            loadout = updated,
                            isLoadingLoadout = false,
                            actionBanner = loadoutSavedBanner("Applied loadout preset!"),
                        )
                    }
                }
                .onFailure { err ->
                    if (!_uiState.value.session?.puuid.equals(requestPuuid, ignoreCase = true)) return@onFailure
                    _uiState.update {
                        it.copy(
                            isLoadingLoadout = false,
                            loadoutError = err.message ?: "Failed to apply preset",
                            actionBanner = err.message ?: "Failed to apply preset",
                        )
                    }
                }
        }
    }

    fun deleteLoadoutPreset(presetId: String) {
        viewModelScope.launch(Dispatchers.IO) {
            runCatching { engine.deleteLoadoutPreset(presetId) }
            val presets = runCatching { engine.listLoadoutPresets() }.getOrDefault(emptyList())
            _uiState.update { it.copy(loadoutPresets = presets) }
        }
    }

    // ---- Live Hub ----

    private fun restartPollingLoopIfNeeded() {
        livePollingJob?.cancel()
        val s = _uiState.value
        if (!s.isAppForegrounded || s.selectedTab != AppTab.LIVE || s.session == null) {
            return
        }
        livePollingJob = viewModelScope.launch {
            while (isActive) {
                val delayMs = pollLiveOnce()
                delay(delayMs)
            }
        }
    }

    fun manualPollLive() {
        viewModelScope.launch { pollLiveOnce(isManual = true) }
    }

    private suspend fun pollLiveOnce(isManual: Boolean = false): Long {
        val requestPuuid = _uiState.value.session?.puuid ?: return 10_000L
        _uiState.update {
            it.copy(
                isPollingLive = true,
                isManualPollingLive = if (isManual) true else it.isManualPollingLive,
            )
        }
        return try {
            val next = withContext(Dispatchers.IO) { engine.pollLiveState() }
            if (!_uiState.value.session?.puuid.equals(requestPuuid, ignoreCase = true)) return 10_000L
            // Keep pregameFirstSeen across transient non-pregame polls so fair-play elapsed time is preserved
            // if a momentary Idle/InParty transition resolves back to the same match; only session change or quit resets it.
            val pregameFirstSeenMs = when (next) {
                is LiveState.PreGame -> resolvePreGameFirstSeenMs(next.snapshot.matchId)
                else -> null
            }
            _uiState.update {
                it.copy(
                    liveState = next,
                    isPollingLive = false,
                    isManualPollingLive = if (isManual) false else it.isManualPollingLive,
                    pregameFirstSeenAtMs = pregameFirstSeenMs,
                    liveError = null,
                )
            }
            when (next) {
                is LiveState.CoreGame -> {
                    val mid = next.snapshot.matchId
                    if (mid.isNotEmpty() && mid != lastCoreGameMatchId) {
                        lastCoreGameMatchId = mid
                        lastLoadoutFetchElapsedMs = 0L
                        if (_uiState.value.loadout != null || _uiState.value.selectedTab == AppTab.LOADOUT) {
                            refreshLoadout(force = true)
                        }
                    }
                }
                else -> {
                    if (lastCoreGameMatchId != null) {
                        lastCoreGameMatchId = null
                        lastLoadoutFetchElapsedMs = 0L
                    }
                }
            }
            if (next is LiveState.PreGame && _uiState.value.ownedAgentUuids.isEmpty()) {
                refreshOwnedAgents()
            }
            when (next) {
                is LiveState.Idle -> (next.recommendedPollSecs.toLong().coerceIn(5L, 15L)) * 1000L
                is LiveState.PreGame -> 2_500L
                else -> 4_000L
            }
        } catch (ex: FfiException.Unauthorized) {
            if (!_uiState.value.session?.puuid.equals(requestPuuid, ignoreCase = true)) return 10_000L
            val refreshed = trySilentCookieRefresh()
            if (refreshed != null && refreshed.puuid.equals(requestPuuid, ignoreCase = true)) {
                _uiState.update { it.copy(session = refreshed, isPollingLive = false, liveError = null) }
                return 2_000L
            }
            _uiState.update {
                it.copy(
                    isPollingLive = false,
                    liveError = null,
                    liveState = LiveState.Idle(
                        message = "Valorant Offline",
                        recommendedPollSecs = 15u,
                    ),
                )
            }
            15_000L
        } catch (c: CancellationException) {
            if (isManual) _uiState.update { it.copy(isManualPollingLive = false) }
            throw c
        } catch (ex: Throwable) {
            if (!_uiState.value.session?.puuid.equals(requestPuuid, ignoreCase = true)) {
                if (isManual) _uiState.update { it.copy(isManualPollingLive = false) }
                return 10_000L
            }
            val msg = ex.message.orEmpty()
            if (msg.contains("was cancelled", ignoreCase = true)) {
                _uiState.update {
                    it.copy(
                        isPollingLive = false,
                        isManualPollingLive = if (isManual) false else it.isManualPollingLive,
                    )
                }
                return 8_000L
            }
            _uiState.update {
                it.copy(
                    isPollingLive = false,
                    isManualPollingLive = if (isManual) false else it.isManualPollingLive,
                    liveError = msg.ifBlank { "Live poll error" },
                )
            }
            8_000L
        } finally {
            _uiState.update {
                it.copy(
                    isPollingLive = false,
                    isManualPollingLive = if (isManual) false else it.isManualPollingLive,
                )
            }
        }
    }

    fun dismissLiveError() {
        _uiState.update { it.copy(liveError = null) }
    }

    private fun runPartyMutation(
        guard: (PartySnapshot) -> String? = { null },
        action: suspend (PartySnapshot) -> PartySnapshot,
    ) {
        val requestPuuid = _uiState.value.session?.puuid ?: return
        val current = (_uiState.value.liveState as? LiveState.InParty)?.snapshot ?: return
        if (_uiState.value.isMutatingLiveAction) return
        val blockedReason = guard(current)
        if (blockedReason != null) {
            _uiState.update { it.copy(actionBanner = blockedReason) }
            return
        }
        viewModelScope.launch {
            _uiState.update { it.copy(isMutatingLiveAction = true, liveError = null) }
            executeWithAutoRefresh(requestPuuid) { action(current) }
                .onSuccess { updated ->
                    if (!_uiState.value.session?.puuid.equals(requestPuuid, ignoreCase = true)) return@onSuccess
                    _uiState.update {
                        it.copy(
                            liveState = LiveState.InParty(updated),
                            isMutatingLiveAction = false,
                            liveError = null,
                        )
                    }
                }
                .onFailure { err ->
                    if (!_uiState.value.session?.puuid.equals(requestPuuid, ignoreCase = true)) return@onFailure
                    _uiState.update {
                        it.copy(
                            isMutatingLiveAction = false,
                            liveError = err.message ?: "Party action failed",
                        )
                    }
                }
        }
    }

    fun toggleReady() = runPartyMutation(
        guard = { p ->
            if (!p.canToggleReady) {
                if (p.isMatchStarting) "Match found — cannot change ready state"
                else "Leave queue before changing ready state"
            } else {
                null
            }
        },
    ) { p ->
        val meReady = p.members.firstOrNull { it.isMe }?.isReady ?: false
        engine.partySetReady(p.partyId, !meReady)
    }

    fun toggleMatchmakingQueue() = runPartyMutation(
        guard = { p ->
            if (p.isInQueue) {
                if (!p.canLeaveQueue) "Match found — cannot cancel queue" else null
            } else {
                if (!p.canStartQueue) {
                    p.queueActionDisabledReason ?: "Cannot start matchmaking right now"
                } else {
                    null
                }
            }
        },
    ) { p ->
        if (p.isInQueue) {
            engine.partyLeaveQueue(p.partyId)
        } else {
            engine.partyJoinQueue(p.partyId)
        }
    }

    fun changePartyQueue(queueId: String) = runPartyMutation(
        guard = { p ->
            when {
                !p.canChangeQueue -> p.changeQueueDisabledReason ?: "Leave queue first to switch mode"
                p.queueId.equals(queueId, ignoreCase = true) -> ""
                p.eligibleQueues.isNotEmpty() && p.eligibleQueues.none { it.equals(queueId, ignoreCase = true) } ->
                    "${queueId.uppercase()} is not eligible for your party right now"
                else -> null
            }?.takeIf { it.isNotEmpty() }
        },
    ) { p ->
        if (p.queueId.equals(queueId, ignoreCase = true)) return@runPartyMutation p
        engine.partyChangeQueue(p.partyId, queueId)
    }

    fun togglePartyAccessibility() = runPartyMutation(
        guard = { p ->
            if (!p.canToggleAccessibility) {
                if (p.isInQueue || p.isMatchStarting) "Leave queue before changing party privacy"
                else "Only the party leader can change party privacy"
            } else {
                null
            }
        },
    ) { p ->
        val makeOpen = !p.accessibility.equals("OPEN", ignoreCase = true)
        engine.partySetAccessibility(p.partyId, makeOpen)
    }

    fun generateInviteCode() = runPartyMutation(
        guard = { p ->
            if (!p.canGenerateInviteCode) {
                when {
                    !p.accessibility.equals("OPEN", ignoreCase = true) -> "Set party to OPEN before generating an invite code"
                    p.isInQueue || p.isMatchStarting -> "Leave queue before managing invite codes"
                    else -> "Only the party leader can generate an invite code"
                }
            } else {
                null
            }
        },
    ) { p ->
        engine.partyGenerateInviteCode(p.partyId)
    }

    fun disableInviteCode() = runPartyMutation(
        guard = { p ->
            if (!p.canDisableInviteCode) {
                if (p.isInQueue || p.isMatchStarting) "Leave queue before disabling invite code"
                else "Only the party leader can disable invite codes"
            } else {
                null
            }
        },
    ) { p ->
        engine.partyDisableInviteCode(p.partyId)
    }

    fun joinPartyByCode(code: String) {
        val requestPuuid = _uiState.value.session?.puuid ?: return
        if (code.isBlank() || _uiState.value.isMutatingLiveAction) return
        val currentParty = (_uiState.value.liveState as? LiveState.InParty)?.snapshot
        if (currentParty != null && !currentParty.canJoinByCode) {
            _uiState.update { it.copy(actionBanner = "Leave queue before joining another party") }
            return
        }
        viewModelScope.launch {
            _uiState.update { it.copy(isMutatingLiveAction = true, liveError = null) }
            executeWithAutoRefresh(requestPuuid) { engine.partyJoinByCode(code.trim()) }
                .onSuccess { updated ->
                    if (!_uiState.value.session?.puuid.equals(requestPuuid, ignoreCase = true)) return@onSuccess
                    _uiState.update {
                        it.copy(
                            liveState = LiveState.InParty(updated),
                            isMutatingLiveAction = false,
                            liveError = null,
                            actionBanner = "Joined party!",
                        )
                    }
                }
                .onFailure { err ->
                    if (!_uiState.value.session?.puuid.equals(requestPuuid, ignoreCase = true)) return@onFailure
                    _uiState.update {
                        it.copy(
                            isMutatingLiveAction = false,
                            liveError = err.message ?: "Failed to join party",
                        )
                    }
                }
        }
    }

    fun selectPreGameAgent(agentUuid: String) = runPreGameAgentAction(agentUuid, lock = false)

    fun lockPreGameAgent(agentUuid: String) = runPreGameAgentAction(agentUuid, lock = true)

    private fun runPreGameAgentAction(agentUuid: String, lock: Boolean) {
        val requestPuuid = _uiState.value.session?.puuid ?: return
        val current = (_uiState.value.liveState as? LiveState.PreGame)?.snapshot ?: return
        if (_uiState.value.isMutatingLiveAction) return
        if (!current.canSelectAgent) {
            _uiState.update {
                it.copy(
                    actionBanner = if (current.mySelectionState.equals("locked", ignoreCase = true)) {
                        "You already locked your agent"
                    } else {
                        "Agent select is no longer accepting picks"
                    },
                )
            }
            return
        }
        if (current.lockedAgentUuids.any { it.equals(agentUuid, ignoreCase = true) }) {
            _uiState.update { it.copy(actionBanner = "That agent is already locked by a teammate") }
            return
        }
        val firstSeenMs = resolvePreGameFirstSeenMs(current.matchId)
        val waitSecs = com.myval.app.ui.components.fairPlayLockRemainingSecs(
            firstSeenElapsedMs = firstSeenMs,
            nowElapsedMs = SystemClock.elapsedRealtime(),
        )
        if (waitSecs > 0) {
            val actionVerb = if (lock) "locking" else "hovering"
            _uiState.update {
                it.copy(
                    pregameFirstSeenAtMs = firstSeenMs,
                    actionBanner = "Wait ${waitSecs}s before $actionVerb — app strictly disables instapicking and instalocking for any unfair advantage.",
                )
            }
            return
        }
        viewModelScope.launch {
            _uiState.update { it.copy(isMutatingLiveAction = true, liveError = null) }
            executeWithAutoRefresh(requestPuuid) {
                if (lock) {
                    engine.pregameLockAgent(current.matchId, agentUuid)
                } else {
                    engine.pregameSelectAgent(current.matchId, agentUuid)
                }
            }.onSuccess { updated ->
                if (!_uiState.value.session?.puuid.equals(requestPuuid, ignoreCase = true)) return@onSuccess
                _uiState.update {
                    it.copy(
                        liveState = LiveState.PreGame(updated),
                        isMutatingLiveAction = false,
                        liveError = null,
                        actionBanner = if (lock) "Agent locked!" else "Agent hovered",
                    )
                }
            }.onFailure { err ->
                if (!_uiState.value.session?.puuid.equals(requestPuuid, ignoreCase = true)) return@onFailure
                _uiState.update {
                    it.copy(
                        isMutatingLiveAction = false,
                        liveError = err.message ?: "Agent select action failed",
                    )
                }
            }
        }
    }

    fun quitPreGameMatch() {
        val requestPuuid = _uiState.value.session?.puuid ?: return
        val current = (_uiState.value.liveState as? LiveState.PreGame)?.snapshot ?: return
        if (_uiState.value.isMutatingLiveAction) return
        if (!current.canQuitMatch) {
            _uiState.update { it.copy(actionBanner = "Agent select has already ended") }
            return
        }
        viewModelScope.launch {
            _uiState.update { it.copy(isMutatingLiveAction = true, liveError = null) }
            executeWithAutoRefresh(requestPuuid) { engine.pregameQuitMatch(current.matchId) }
                .onSuccess {
                    if (!_uiState.value.session?.puuid.equals(requestPuuid, ignoreCase = true)) return@onSuccess
                    pregameFirstSeen = null
                    _uiState.update {
                        it.copy(
                            isMutatingLiveAction = false,
                            pregameFirstSeenAtMs = null,
                            liveError = null,
                            actionBanner = "Left Agent Select",
                        )
                    }
                    pollLiveOnce()
                }
                .onFailure { err ->
                    if (!_uiState.value.session?.puuid.equals(requestPuuid, ignoreCase = true)) return@onFailure
                    _uiState.update {
                        it.copy(
                            isMutatingLiveAction = false,
                            liveError = err.message ?: "Failed to quit match",
                        )
                    }
                }
        }
    }

    // ---- Storefront ----

    private fun loadCachedOrLiveStorefront() {
        val requestPuuid = _uiState.value.session?.puuid ?: return
        viewModelScope.launch {
            val cached = withContext(Dispatchers.IO) {
                runCatching { engine.getCachedStorefront() }.getOrNull()
            }
            if (cached != null && _uiState.value.session?.puuid.equals(requestPuuid, ignoreCase = true)) {
                lastStoreTickElapsedMs = SystemClock.elapsedRealtime()
                _uiState.update { it.copy(storefront = cached) }
            }
            refreshStorefront()
        }
    }

    fun refreshStorefront() {
        val requestPuuid = _uiState.value.session?.puuid ?: return
        if (_uiState.value.isLoadingStore) return
        viewModelScope.launch {
            _uiState.update { it.copy(isLoadingStore = true, storeError = null) }
            executeWithAutoRefresh(requestPuuid, statusLabel = "Loading Store...") {
                engine.fetchStorefront()
            }
                .onSuccess { snap ->
                    if (!_uiState.value.session?.puuid.equals(requestPuuid, ignoreCase = true)) return@onSuccess
                    lastStoreTickElapsedMs = SystemClock.elapsedRealtime()
                    _uiState.update {
                        it.copy(
                            storefront = snap,
                            isLoadingStore = false,
                        )
                    }
                    if (_uiState.value.selectedTab == AppTab.STORE || _uiState.value.selectedTab == AppTab.LOADOUT) {
                        refreshCollectionSummaryInBackground()
                    }
                }
                .onFailure { err ->
                    if (!_uiState.value.session?.puuid.equals(requestPuuid, ignoreCase = true)) return@onFailure
                    _uiState.update {
                        it.copy(
                            isLoadingStore = false,
                            storeError = err.message ?: "Failed to load Riot store",
                        )
                    }
                }
        }
    }

    private fun refreshCollectionSummaryInBackground() {
        val requestPuuid = _uiState.value.session?.puuid ?: return
        viewModelScope.launch {
            val res = executeWithAutoRefresh(requestPuuid) {
                val col = engine.fetchCollectionSummary()
                val own = runCatching { engine.getOwnedSkinUuids() }
                    .getOrDefault(emptyList())
                    .map { it.lowercase() }
                    .toSet()
                val chromas = runCatching { engine.getOwnedChromaUuids() }
                    .getOrDefault(emptyList())
                    .map { it.lowercase() }
                    .toSet()
                val levels = runCatching { engine.getOwnedLevelUuids() }
                    .getOrDefault(emptyList())
                    .map { it.lowercase() }
                    .toSet()
                val ownedSprays = runCatching { engine.getOwnedSprayUuids() }
                    .getOrDefault(emptyList())
                    .map { it.lowercase() }
                    .toSet()
                val ownedCards = runCatching { engine.getOwnedCardUuids() }
                    .getOrDefault(emptyList())
                    .map { it.lowercase() }
                    .toSet()
                val ownedTitles = runCatching { engine.getOwnedTitleUuids() }
                    .getOrDefault(emptyList())
                    .map { it.lowercase() }
                    .toSet()
                CollectionOwnershipBundle(
                    collection = col,
                    ownedSkins = own,
                    ownedChromas = chromas,
                    ownedLevels = levels,
                    ownedSprays = ownedSprays,
                    ownedCards = ownedCards,
                    ownedTitles = ownedTitles,
                )
            }
            if (!_uiState.value.session?.puuid.equals(requestPuuid, ignoreCase = true)) return@launch
            res.onSuccess { b ->
                _uiState.update {
                    it.copy(
                        collectionSummary = b.collection,
                        ownedSkinUuids = if (b.ownedSkins.isNotEmpty()) b.ownedSkins else it.ownedSkinUuids,
                        ownedChromaUuids = if (b.ownedChromas.isNotEmpty()) b.ownedChromas else it.ownedChromaUuids,
                        ownedLevelUuids = if (b.ownedLevels.isNotEmpty()) b.ownedLevels else it.ownedLevelUuids,
                        ownedSprayUuids = if (b.ownedSprays.isNotEmpty()) b.ownedSprays else it.ownedSprayUuids,
                        ownedCardUuids = if (b.ownedCards.isNotEmpty()) b.ownedCards else it.ownedCardUuids,
                        ownedTitleUuids = if (b.ownedTitles.isNotEmpty()) b.ownedTitles else it.ownedTitleUuids,
                    )
                }
            }
        }
    }

    fun dismissBanner() {
        _uiState.update { it.copy(actionBanner = null) }
    }

    fun openAboutSheet() {
        _uiState.update { it.copy(showAboutSheet = true) }
    }

    fun closeAboutSheet() {
        _uiState.update { it.copy(showAboutSheet = false) }
    }

    fun dismissUpdatePopup() {
        dismissedUpdateTag = _uiState.value.availableUpdate?.tagName
        _uiState.update { it.copy(showUpdatePopup = false) }
    }

    fun checkForUpdates(manual: Boolean = true) {
        if (_uiState.value.isCheckingUpdate) return
        viewModelScope.launch {
            _uiState.update {
                it.copy(
                    isCheckingUpdate = true,
                    updateCheckStatus = if (manual) "Checking GitHub Releases..." else it.updateCheckStatus,
                )
            }
            val result = withContext(Dispatchers.IO) { AppUpdateChecker.fetchLatestRelease() }
            result.onSuccess { release ->
                if (release != null && AppUpdateChecker.isNewerVersion(release.versionName, BuildConfig.VERSION_NAME)) {
                    val shouldPopup = manual || !release.tagName.equals(dismissedUpdateTag, ignoreCase = true)
                    _uiState.update {
                        it.copy(
                            isCheckingUpdate = false,
                            availableUpdate = release,
                            showUpdatePopup = shouldPopup,
                            updateCheckStatus = "Update v${release.versionName} is available!",
                        )
                    }
                } else {
                    _uiState.update {
                        it.copy(
                            isCheckingUpdate = false,
                            availableUpdate = null,
                            showUpdatePopup = false,
                            updateCheckStatus = "You're on the latest version (v${BuildConfig.VERSION_NAME}).",
                            actionBanner = if (manual) "MyVal v${BuildConfig.VERSION_NAME} is up to date" else it.actionBanner,
                        )
                    }
                }
            }.onFailure { err ->
                _uiState.update {
                    it.copy(
                        isCheckingUpdate = false,
                        updateCheckStatus = if (manual) {
                            "Could not check updates: ${err.message ?: "Network error"}"
                        } else {
                            it.updateCheckStatus
                        },
                    )
                }
            }
        }
    }

    fun downloadAvailableUpdate() {
        val release = _uiState.value.availableUpdate ?: return
        if (_uiState.value.isDownloadingUpdate) return

        val appContext = getApplication<Application>()
        val existingApk = _uiState.value.downloadedUpdateApkPath
            ?.let(::File)
            ?.takeIf { it.exists() && it.name.contains(release.versionName) && it.length() > 500_000L }
        if (existingApk != null) {
            handleApkInstallOutcome(appContext, existingApk, release.versionName)
            return
        }

        viewModelScope.launch {
            _uiState.update {
                it.copy(
                    isDownloadingUpdate = true,
                    updateDownloadProgress = 0f,
                    updateDownloadLabel = "Connecting to GitHub...",
                    downloadedUpdateApkPath = null,
                )
            }
            val result = AppUpdateChecker.downloadApkToCache(appContext, release) { bytesRead, totalBytes ->
                val readMb = bytesRead.toDouble() / (1024.0 * 1024.0)
                val progress = if (totalBytes > 0L) (bytesRead.toFloat() / totalBytes.toFloat()).coerceIn(0f, 1f) else null
                val label = if (totalBytes > 0L) {
                    val totalMb = totalBytes.toDouble() / (1024.0 * 1024.0)
                    val pct = ((bytesRead * 100L) / totalBytes).coerceIn(0L, 100L)
                    "Downloading v${release.versionName} • ${"%.1f".format(Locale.US, readMb)} / ${"%.1f".format(Locale.US, totalMb)} MB ($pct%)"
                } else {
                    "Downloading v${release.versionName} • ${"%.1f".format(Locale.US, readMb)} MB"
                }
                _uiState.update {
                    it.copy(
                        updateDownloadProgress = progress,
                        updateDownloadLabel = label,
                    )
                }
            }
            result.onSuccess { apkFile ->
                _uiState.update {
                    it.copy(
                        isDownloadingUpdate = false,
                        updateDownloadProgress = 1f,
                        downloadedUpdateApkPath = apkFile.absolutePath,
                    )
                }
                handleApkInstallOutcome(appContext, apkFile, release.versionName)
            }.onFailure { err ->
                val msg = err.message?.ifBlank { "Network error" } ?: "Network error"
                _uiState.update {
                    it.copy(
                        isDownloadingUpdate = false,
                        updateDownloadProgress = null,
                        updateDownloadLabel = "Download failed ($msg) — tap Release Page",
                        actionBanner = "Update download failed — tap Release Page to download in Chrome",
                    )
                }
            }
        }
    }

    private fun handleApkInstallOutcome(appContext: Application, apkFile: File, versionName: String) {
        when (val outcome = AppUpdateChecker.launchApkInstaller(appContext, apkFile)) {
            is ApkInstallOutcome.InstallerLaunched -> {
                _uiState.update {
                    it.copy(
                        showUpdatePopup = false,
                        updateDownloadLabel = "v$versionName ready — confirm Update on screen",
                        actionBanner = "Opening v$versionName installer...",
                    )
                }
            }
            is ApkInstallOutcome.NeedsInstallPermission -> {
                _uiState.update {
                    it.copy(
                        updateDownloadLabel = "Allow 'Install unknown apps' for MyVal, then tap Install APK",
                        actionBanner = "Allow 'Install unknown apps' for MyVal, then tap Install APK",
                    )
                }
            }
            is ApkInstallOutcome.Failed -> {
                _uiState.update {
                    it.copy(
                        updateDownloadLabel = outcome.reason,
                        actionBanner = outcome.reason,
                    )
                }
            }
        }
    }

    fun openAvailableReleasePage() {
        val url = _uiState.value.availableUpdate?.htmlUrl ?: AppUpdateChecker.RELEASES_URL
        AppUpdateChecker.openExternalUrl(getApplication(), url)
        _uiState.update { it.copy(showUpdatePopup = false) }
    }

    fun openExternalUrl(url: String) {
        AppUpdateChecker.openExternalUrl(getApplication(), url)
    }

    private fun startCountdownTicker() {
        countdownJob?.cancel()
        lastStoreTickElapsedMs = SystemClock.elapsedRealtime()
        countdownJob = viewModelScope.launch {
            while (isActive) {
                delay(1_000L)
                val nowMs = SystemClock.elapsedRealtime()
                _uiState.update { state ->
                    // Zero background CPU wakeups or hidden-tab recompositions when paused
                    if (!state.isAppForegrounded) return@update state

                    val storeVisible = state.selectedTab == AppTab.STORE ||
                        (state.selectedTab == AppTab.LIVE && state.liveState is LiveState.Idle)
                    val nextStore = if (storeVisible && state.storefront != null) {
                        val deltaSecs = ((nowMs - lastStoreTickElapsedMs) / 1_000L).coerceAtLeast(1L)
                        lastStoreTickElapsedMs = nowMs
                        val current = state.storefront
                        current.copy(
                            dailyRemainingSecs = (current.dailyRemainingSecs - deltaSecs).coerceAtLeast(0),
                            accessoryRemainingSecs = (current.accessoryRemainingSecs - deltaSecs).coerceAtLeast(0),
                            featuredBundles = current.featuredBundles.map { b ->
                                b.copy(remainingSecs = (b.remainingSecs - deltaSecs).coerceAtLeast(0))
                            },
                            nightMarket = current.nightMarket?.let { nm ->
                                nm.copy(remainingSecs = (nm.remainingSecs - deltaSecs).coerceAtLeast(0))
                            },
                        )
                    } else {
                        state.storefront
                    }

                    val liveVisible = state.selectedTab == AppTab.LIVE
                    val nextLive = if (liveVisible) {
                        when (val live = state.liveState) {
                            is LiveState.PreGame -> {
                                val rem = live.snapshot.phaseTimeRemainingSecs
                                if (rem > 0L) {
                                    LiveState.PreGame(live.snapshot.copy(phaseTimeRemainingSecs = rem - 1L))
                                } else {
                                    live
                                }
                            }
                            is LiveState.InParty -> {
                                if (live.snapshot.isInQueue && !live.snapshot.isMatchStarting) {
                                    LiveState.InParty(live.snapshot.copy(queueElapsedSecs = live.snapshot.queueElapsedSecs + 1L))
                                } else {
                                    live
                                }
                            }
                            else -> live
                        }
                    } else {
                        state.liveState
                    }

                    if (nextStore === state.storefront && nextLive === state.liveState) {
                        state
                    } else {
                        state.copy(storefront = nextStore, liveState = nextLive)
                    }
                }
            }
        }
    }
}

private data class Quadruple<A, B, C, D>(
    val first: A,
    val second: B,
    val third: C,
    val fourth: D,
)

private data class BootstrapLocalBundle(
    val summary: AssetSummary?,
    val agents: List<Agent>,
    val hasKey: Boolean,
    val vault: com.myval.app.data.AccountVault,
    val needsResync: Boolean,
)
