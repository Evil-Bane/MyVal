package com.myval.app

import android.app.Activity
import android.view.WindowManager
import androidx.activity.compose.BackHandler
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.spring
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideOutHorizontally
import androidx.compose.foundation.background
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.systemBarsPadding
import androidx.compose.foundation.pager.HorizontalPager
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.EmojiEvents
import androidx.compose.material.icons.filled.Explore
import androidx.compose.material.icons.filled.Radar
import androidx.compose.material.icons.filled.Shield
import androidx.compose.material.icons.filled.Storefront
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.NavigationBarItemDefaults
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleEventObserver
import androidx.lifecycle.compose.LocalLifecycleOwner
import androidx.lifecycle.viewmodel.compose.viewModel
import com.myval.app.ui.AppTab
import com.myval.app.ui.MyValViewModel
import com.myval.app.ui.auth.RiotLoginScreen
import com.myval.app.ui.career.CareerScreen
import com.myval.app.ui.career.MatchReplayAndScoreboardView
import com.myval.app.ui.career.PlayerProfileScreen
import com.myval.app.ui.career.PlayerSearchBottomSheet
import com.myval.app.ui.components.AboutUpdateBottomSheet
import com.myval.app.ui.components.AccountsManagerBottomSheet
import com.myval.app.ui.components.TacticalSplashScreen
import com.myval.app.ui.components.UpdateAvailableDialog
import com.myval.app.ui.components.tactileScale
import com.myval.app.ui.explorer.ExplorerScreen
import com.myval.app.ui.live.LiveHubScreen
import com.myval.app.ui.loadout.LoadoutScreen
import com.myval.app.ui.store.StoreScreen
import com.myval.app.ui.theme.ObsidianBg
import com.myval.app.ui.theme.SurfaceCard
import com.myval.app.ui.theme.SurfaceElevated
import com.myval.app.ui.theme.SurfaceStroke
import com.myval.app.ui.theme.TextSecondary
import com.myval.app.ui.theme.ValRed
import kotlinx.coroutines.launch
import uniffi.myval_ffi.LiveState

@Composable
fun MyValApp(vm: MyValViewModel = viewModel()) {
    val uiState by vm.uiState.collectAsState()
    val snackbarHostState = remember { SnackbarHostState() }
    val lifecycleOwner = LocalLifecycleOwner.current
    val context = LocalContext.current
    val scope = rememberCoroutineScope()
    val tabs = remember { AppTab.entries }
    val navItems = remember {
        listOf(
            Triple(AppTab.LIVE, "Live", Icons.Default.Radar),
            Triple(AppTab.STORE, "Store", Icons.Default.Storefront),
            Triple(AppTab.CAREER, "Career", Icons.Default.EmojiEvents),
            Triple(AppTab.LOADOUT, "Arsenal", Icons.Default.Shield),
            Triple(AppTab.EXPLORER, "Explore", Icons.Default.Explore),
        )
    }
    val pagerState = rememberPagerState(
        initialPage = uiState.selectedTab.ordinal,
        pageCount = { tabs.size },
    )
    val currentPageIndex by remember { derivedStateOf { pagerState.currentPage } }

    // Keep screen awake ONLY during active live Agent Select / In-Game matches to save battery elsewhere
    val keepScreenOn = uiState.liveState is LiveState.PreGame || uiState.liveState is LiveState.CoreGame
    DisposableEffect(context, keepScreenOn) {
        val window = (context as? Activity)?.window
        if (keepScreenOn) {
            window?.addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
        } else {
            window?.clearFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
        }
        onDispose {
            window?.clearFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
        }
    }

    DisposableEffect(lifecycleOwner) {
        val observer = LifecycleEventObserver { _, event ->
            when (event) {
                Lifecycle.Event.ON_RESUME -> vm.onForegroundChanged(true)
                Lifecycle.Event.ON_PAUSE -> vm.onForegroundChanged(false)
                else -> Unit
            }
        }
        lifecycleOwner.lifecycle.addObserver(observer)
        onDispose { lifecycleOwner.lifecycle.removeObserver(observer) }
    }

    LaunchedEffect(pagerState.settledPage) {
        val targetTab = tabs[pagerState.settledPage]
        if (uiState.selectedTab != targetTab) {
            vm.selectTab(targetTab)
        }
    }

    LaunchedEffect(uiState.actionBanner) {
        val banner = uiState.actionBanner ?: return@LaunchedEffect
        snackbarHostState.showSnackbar(banner)
        vm.dismissBanner()
    }

    if (uiState.isBootstrapping) {
        TacticalSplashScreen(statusText = uiState.startupStatus)
        return
    }

    if (uiState.showLoginWebView) {
        BackHandler { vm.closeRiotLogin() }
        RiotLoginScreen(
            loginUrl = vm.getRiotLoginUrl(),
            isAddingAnotherAccount = uiState.isAddingAnotherAccount,
            isCompletingAuth = uiState.isSigningIn,
            authError = uiState.authError,
            onRedirectWithTokens = vm::completeLoginFromRedirect,
            onClose = vm::closeRiotLogin,
        )
        return
    }

    if (uiState.showAccountsSheet) {
        AccountsManagerBottomSheet(
            accounts = uiState.savedAccounts,
            activePuuid = uiState.session?.puuid,
            onDismiss = vm::closeAccountsSheet,
            onSwitchAccount = vm::switchAccount,
            onAddAnotherAccount = vm::openAddAccountLogin,
            onRefreshCurrentAccount = vm::refreshActiveAccount,
            onRemoveAccount = vm::removeAccount,
            onSignOutAll = vm::signOut,
        )
    }

    if (uiState.showPlayerSearchSheet) {
        PlayerSearchBottomSheet(
            query = uiState.playerSearchQuery,
            knownPlayers = uiState.knownPlayers,
            liveState = uiState.liveState,
            isLookingUpRiotId = uiState.isLookingUpRiotId,
            lookupError = uiState.riotIdLookupError,
            hasLookupApiKey = uiState.hasLookupApiKey,
            onQueryChange = vm::updatePlayerSearchQuery,
            onLookupRiotId = vm::lookupPlayerByRiotId,
            onSaveLookupApiKey = vm::saveLookupApiKey,
            onSelectPlayer = { puuid, name -> vm.openPlayerProfile(puuid, name) },
            onDismiss = vm::closePlayerSearch,
        )
    }

    if (uiState.showAboutSheet) {
        AboutUpdateBottomSheet(
            availableUpdate = uiState.availableUpdate,
            isCheckingUpdate = uiState.isCheckingUpdate,
            updateCheckStatus = uiState.updateCheckStatus,
            isDownloadingUpdate = uiState.isDownloadingUpdate,
            updateDownloadProgress = uiState.updateDownloadProgress,
            updateDownloadLabel = uiState.updateDownloadLabel,
            hasDownloadedApk = uiState.downloadedUpdateApkPath != null,
            onCheckForUpdates = { vm.checkForUpdates(manual = true) },
            onDownloadUpdate = vm::downloadAvailableUpdate,
            onOpenUrl = vm::openExternalUrl,
            onDismiss = vm::closeAboutSheet,
        )
    }

    val pendingUpdate = uiState.availableUpdate
    if (uiState.showUpdatePopup && pendingUpdate != null) {
        UpdateAvailableDialog(
            release = pendingUpdate,
            isDownloadingUpdate = uiState.isDownloadingUpdate,
            updateDownloadProgress = uiState.updateDownloadProgress,
            updateDownloadLabel = uiState.updateDownloadLabel,
            hasDownloadedApk = uiState.downloadedUpdateApkPath != null,
            onDownloadApk = vm::downloadAvailableUpdate,
            onOpenReleasePage = vm::openAvailableReleasePage,
            onDismiss = vm::dismissUpdatePopup,
        )
    }

    val showProfileOverlay = uiState.inspectedPlayerPuuid != null && uiState.selectedMatchDetail == null
    val activeMatchDetail = uiState.selectedMatchDetail

    // When on any secondary tab with no modal/overlay open, Back returns cleanly to Live instead of exiting
    val returnHomeOnBack = !showProfileOverlay &&
        activeMatchDetail == null &&
        !uiState.showAccountsSheet &&
        !uiState.showPlayerSearchSheet &&
        !uiState.showAboutSheet &&
        !(uiState.showUpdatePopup && pendingUpdate != null) &&
        currentPageIndex != AppTab.LIVE.ordinal
    BackHandler(enabled = returnHomeOnBack) {
        vm.selectTab(AppTab.LIVE)
        scope.launch { pagerState.animateScrollToPage(AppTab.LIVE.ordinal) }
    }

    // Root Box keeps Scaffold + HorizontalPager alive underneath detail overlays so scroll state
    // and expanded player skin drawers are 100% preserved with smooth native push/pop transitions.
    Box(modifier = Modifier.fillMaxSize().background(ObsidianBg)) {
        Scaffold(
            containerColor = ObsidianBg,
            snackbarHost = { SnackbarHost(snackbarHostState) },
            bottomBar = {
                Column {
                    HorizontalDivider(thickness = 0.5.dp, color = SurfaceStroke)
                    NavigationBar(containerColor = SurfaceCard, tonalElevation = 0.dp) {
                        navItems.forEach { (tab, label, icon) ->
                            val isSelected = currentPageIndex == tab.ordinal
                            val navInteraction = remember { MutableInteractionSource() }
                            val iconScale by animateFloatAsState(
                                targetValue = if (isSelected) 1.15f else 1f,
                                animationSpec = spring(dampingRatio = 0.55f, stiffness = 480f),
                                label = "navIconScale",
                            )
                            NavigationBarItem(
                                selected = isSelected,
                                onClick = {
                                    vm.selectTab(tab)
                                    scope.launch { pagerState.animateScrollToPage(tab.ordinal) }
                                },
                                interactionSource = navInteraction,
                                modifier = Modifier.tactileScale(navInteraction, pressedScale = 0.92f),
                                icon = {
                                    Icon(
                                        icon,
                                        contentDescription = label,
                                        modifier = Modifier.graphicsLayer {
                                            scaleX = iconScale
                                            scaleY = iconScale
                                        },
                                    )
                                },
                                label = { Text(label, fontWeight = FontWeight.Bold, fontSize = 10.5.sp) },
                                colors = NavigationBarItemDefaults.colors(
                                    selectedIconColor = ValRed,
                                    selectedTextColor = ValRed,
                                    indicatorColor = SurfaceElevated,
                                    unselectedIconColor = TextSecondary,
                                    unselectedTextColor = TextSecondary,
                                ),
                            )
                        }
                    }
                }
            },
        ) { innerPadding ->
            HorizontalPager(
                state = pagerState,
                beyondViewportPageCount = 1,
                modifier = Modifier
                    .fillMaxSize()
                    .padding(innerPadding),
            ) { pageIndex ->
                when (tabs[pageIndex]) {
                    AppTab.LIVE -> LiveHubScreen(
                        uiState = uiState,
                        onOpenLogin = vm::openRiotLogin,
                        onOpenAccounts = vm::openAccountsSheet,
                        onOpenPlayerSearch = vm::openPlayerSearch,
                        onOpenAbout = vm::openAboutSheet,
                        onInspectPlayer = { puuid, name -> vm.openPlayerProfile(puuid, name) },
                        onNavigateToTab = { tab ->
                            vm.selectTab(tab)
                            scope.launch { pagerState.animateScrollToPage(tab.ordinal) }
                        },
                        onSelectMatch = vm::openMatchDetail,
                        onManualPoll = vm::manualPollLive,
                        onDismissError = vm::dismissLiveError,
                        onToggleReady = vm::toggleReady,
                        onToggleQueue = vm::toggleMatchmakingQueue,
                        onChangeQueue = vm::changePartyQueue,
                        onToggleAccessibility = vm::togglePartyAccessibility,
                        onGenerateInvite = vm::generateInviteCode,
                        onDisableInvite = vm::disableInviteCode,
                        onJoinPartyCode = vm::joinPartyByCode,
                        onSelectPreGameAgent = vm::selectPreGameAgent,
                        onLockPreGameAgent = vm::lockPreGameAgent,
                        onQuitPreGameMatch = vm::quitPreGameMatch,
                    )

                    AppTab.STORE -> StoreScreen(
                        uiState = uiState,
                        onOpenLogin = vm::openRiotLogin,
                        onOpenAccounts = vm::openAccountsSheet,
                        onRefreshStore = vm::refreshStorefront,
                        onToggleWishlist = vm::toggleWishlist,
                    )

                    AppTab.CAREER -> CareerScreen(
                        uiState = uiState,
                        onOpenLogin = vm::openRiotLogin,
                        onOpenAccounts = vm::openAccountsSheet,
                        onOpenPlayerSearch = vm::openPlayerSearch,
                        onRefreshCareer = vm::refreshCareer,
                        onSelectMatch = vm::openMatchDetail,
                    )

                    AppTab.LOADOUT -> LoadoutScreen(
                        uiState = uiState,
                        onOpenLogin = vm::openRiotLogin,
                        onOpenAccounts = vm::openAccountsSheet,
                        onRefreshLoadout = { vm.refreshLoadout(force = true) },
                        onEquipWeaponSkin = vm::equipWeaponSkin,
                        onEquipSpray = vm::equipSpray,
                        onEquipPlayerCard = vm::equipPlayerCard,
                        onEquipPlayerTitle = vm::equipPlayerTitle,
                        onSetIdentityPrivacy = vm::setIdentityPrivacy,
                        onRandomizeLoadout = vm::randomizeLoadout,
                        onSavePreset = vm::saveLoadoutPreset,
                        onApplyPreset = vm::applyLoadoutPreset,
                        onDeletePreset = vm::deleteLoadoutPreset,
                    )

                    AppTab.EXPLORER -> ExplorerScreen(
                        uiState = uiState,
                        onSyncAssets = vm::syncAssetsNow,
                        onOpenAbout = vm::openAboutSheet,
                        onSearchSkins = vm::updateSkinSearch,
                        onToggleWishlist = vm::toggleWishlist,
                    )
                }
            }
        }

        // Native animated overlay: Inspected Player Profile
        AnimatedVisibility(
            visible = showProfileOverlay,
            enter = slideInHorizontally(initialOffsetX = { it / 5 }, animationSpec = tween(200)) + fadeIn(tween(180)),
            exit = slideOutHorizontally(targetOffsetX = { it / 5 }, animationSpec = tween(180)) + fadeOut(tween(150)),
        ) {
            BackHandler(enabled = showProfileOverlay) { vm.closePlayerProfile() }
            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .background(ObsidianBg)
                    .systemBarsPadding(),
            ) {
                PlayerProfileScreen(
                    hintName = uiState.inspectedPlayerHintName,
                    profile = uiState.inspectedPlayerProfile,
                    isLoading = uiState.isLoadingPlayerProfile,
                    error = uiState.playerProfileError,
                    onClose = vm::closePlayerProfile,
                    onRefresh = vm::refreshInspectedPlayerProfile,
                    onOpenSearch = vm::openPlayerSearch,
                    onSelectMatch = vm::openMatchDetailForPlayer,
                )
            }
        }

        // Native animated overlay: Match Replay & Scoreboard
        AnimatedVisibility(
            visible = activeMatchDetail != null,
            enter = slideInHorizontally(initialOffsetX = { it / 5 }, animationSpec = tween(200)) + fadeIn(tween(180)),
            exit = slideOutHorizontally(targetOffsetX = { it / 5 }, animationSpec = tween(180)) + fadeOut(tween(150)),
        ) {
            if (activeMatchDetail != null) {
                BackHandler { vm.closeMatchDetail() }
                Box(
                    modifier = Modifier
                        .fillMaxSize()
                        .background(ObsidianBg)
                        .systemBarsPadding(),
                ) {
                    MatchReplayAndScoreboardView(
                        detail = activeMatchDetail,
                        activeSessionPuuid = uiState.session?.puuid,
                        onInspectPlayer = { puuid, name -> vm.openPlayerProfile(puuid, name) },
                        onClose = vm::closeMatchDetail,
                    )
                }
            }
        }
    }
}
