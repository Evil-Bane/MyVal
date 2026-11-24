package com.myval.app.ui.live

import android.os.SystemClock
import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ExitToApp
import androidx.compose.material.icons.automirrored.filled.Login
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.ContentCopy
import androidx.compose.material.icons.filled.EmojiEvents
import androidx.compose.material.icons.filled.ExpandLess
import androidx.compose.material.icons.filled.ExpandMore
import androidx.compose.material.icons.filled.Favorite
import androidx.compose.material.icons.filled.Groups
import androidx.compose.material.icons.filled.Info
import androidx.compose.material.icons.filled.Lock
import androidx.compose.material.icons.filled.LockOpen
import androidx.compose.material.icons.filled.Person
import androidx.compose.material.icons.filled.PersonAdd
import androidx.compose.material.icons.filled.PersonOutline
import androidx.compose.material.icons.filled.PersonSearch
import androidx.compose.material.icons.filled.PlayArrow
import androidx.compose.material.icons.filled.QueryStats
import androidx.compose.material.icons.filled.Radar
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material.icons.filled.Shield
import androidx.compose.material.icons.filled.Stop
import androidx.compose.material.icons.filled.Storefront
import androidx.compose.material.icons.filled.VisibilityOff
import androidx.compose.material.icons.filled.Warning
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.FilterChip
import androidx.compose.material3.FilterChipDefaults
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableLongStateOf
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import kotlinx.coroutines.delay
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.draw.scale
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil.compose.AsyncImage
import java.util.Locale
import com.myval.app.ui.AppTab
import com.myval.app.ui.MyValUiState
import com.myval.app.ui.career.MatchHistoryRowCard
import com.myval.app.ui.components.AccountSwitcherChip
import com.myval.app.ui.components.AuthRequiredCard
import com.myval.app.ui.components.GlossyCardBrush
import com.myval.app.ui.components.GlossyElevatedBrush
import com.myval.app.ui.components.InnerCardShape
import com.myval.app.ui.components.LiveNetworkStatusBar
import com.myval.app.ui.components.MyValPullToRefreshBox
import com.myval.app.ui.components.SharpCardShape
import com.myval.app.ui.components.SharpChipShape
import com.myval.app.ui.components.TacticalBadge
import com.myval.app.ui.components.FAIR_PLAY_LOCK_DELAY_SECS
import com.myval.app.ui.components.fairPlayLockRemainingSecs
import com.myval.app.ui.components.formatDuration
import com.myval.app.ui.components.parseHexColor
import com.myval.app.ui.components.stackBadgeColor
import com.myval.app.ui.components.stackTintedCardBrush
import com.myval.app.ui.components.tactileClickable
import com.myval.app.ui.theme.ObsidianBg
import com.myval.app.ui.theme.SurfaceCard
import com.myval.app.ui.theme.SurfaceElevated
import com.myval.app.ui.theme.SurfaceStroke
import com.myval.app.ui.theme.TextMuted
import com.myval.app.ui.theme.TextPrimary
import com.myval.app.ui.theme.TextSecondary
import com.myval.app.ui.theme.ValAmber
import com.myval.app.ui.theme.ValCyan
import com.myval.app.ui.theme.ValDrawSlate
import com.myval.app.ui.theme.ValGold
import com.myval.app.ui.theme.ValGreen
import com.myval.app.ui.theme.ValLossRed
import com.myval.app.ui.theme.ValPurple
import com.myval.app.ui.theme.ValRed
import com.myval.app.ui.theme.ValWinGreen
import uniffi.myval_ffi.Agent
import uniffi.myval_ffi.CoreGameSnapshot
import uniffi.myval_ffi.EquippedWeaponSkin
import uniffi.myval_ffi.LiveState
import uniffi.myval_ffi.PartySnapshot
import uniffi.myval_ffi.PreGameSnapshot
import uniffi.myval_ffi.ScoutedPlayer

@Composable
fun LiveHubScreen(
    uiState: MyValUiState,
    onOpenLogin: () -> Unit,
    onOpenAccounts: () -> Unit,
    onOpenPlayerSearch: () -> Unit,
    onOpenAbout: () -> Unit,
    onInspectPlayer: (puuid: String, displayName: String) -> Unit,
    onNavigateToTab: (AppTab) -> Unit,
    onSelectMatch: (String) -> Unit,
    onManualPoll: () -> Unit,
    onDismissError: () -> Unit,
    onToggleReady: () -> Unit,
    onToggleQueue: () -> Unit,
    onChangeQueue: (String) -> Unit,
    onToggleAccessibility: () -> Unit,
    onGenerateInvite: () -> Unit,
    onDisableInvite: () -> Unit,
    onJoinPartyCode: (String) -> Unit,
    onSelectPreGameAgent: (String) -> Unit,
    onLockPreGameAgent: (String) -> Unit,
    onQuitPreGameMatch: () -> Unit,
) {
    var showJoinDialog by remember { mutableStateOf(false) }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(ObsidianBg),
    ) {
        SessionHeaderBar(
            uiState = uiState,
            onOpenLogin = onOpenLogin,
            onOpenAccounts = onOpenAccounts,
            onOpenPlayerSearch = onOpenPlayerSearch,
            onOpenAbout = onOpenAbout,
            onManualPoll = onManualPoll,
        )

        LiveNetworkStatusBar(
            message = uiState.networkStatusMessage,
            isSlow = uiState.isNetworkSlow,
        )

        if (uiState.liveError != null) {
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .background(ValRed.copy(alpha = 0.14f))
                    .padding(start = 16.dp, end = 6.dp, top = 6.dp, bottom = 6.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(
                    text = uiState.liveError,
                    color = ValRed,
                    fontSize = 12.sp,
                    modifier = Modifier.weight(1f),
                )
                IconButton(onClick = onDismissError, modifier = Modifier.size(28.dp)) {
                    Icon(
                        imageVector = Icons.Default.Close,
                        contentDescription = "Dismiss error",
                        tint = ValRed,
                        modifier = Modifier.size(16.dp),
                    )
                }
            }
        }

        if (uiState.session == null) {
            Box(modifier = Modifier.padding(16.dp)) {
                AuthRequiredCard(
                    title = "Connect Your Riot Account",
                    subtitle = "Sign in directly on auth.riotgames.com to stream live Party remote controls, Ally Pre-Game lobbies, and 10-player Core-Game Rank Scout & Stack Detection.",
                    onOpenLogin = onOpenLogin,
                    savedAccountCount = uiState.savedAccounts.size,
                    onOpenAccounts = onOpenAccounts,
                )
            }
        } else {
            MyValPullToRefreshBox(
                isRefreshing = uiState.isManualPollingLive,
                onRefresh = onManualPoll,
                accent = ValCyan,
                modifier = Modifier
                    .fillMaxWidth()
                    .weight(1f),
            ) {
                AnimatedContent(
                    targetState = uiState.liveState,
                    contentKey = { it::class },
                    transitionSpec = {
                        (fadeIn(tween(220)) + slideInVertically(tween(220)) { it / 24 })
                            .togetherWith(fadeOut(tween(130)))
                    },
                    label = "liveStateTransition",
                ) { live ->
                    when (live) {
                        is LiveState.Idle -> IdleView(
                            uiState = uiState,
                            shard = uiState.session.shard.uppercase(),
                            isPolling = uiState.isPollingLive,
                            onManualPoll = onManualPoll,
                            onOpenPlayerSearch = onOpenPlayerSearch,
                            onNavigateToTab = onNavigateToTab,
                            onSelectMatch = onSelectMatch,
                            onInspectSelf = { onInspectPlayer(uiState.session.puuid, uiState.session.riotId) },
                        )

                        is LiveState.InParty -> PartyView(
                            party = live.snapshot,
                            isMutating = uiState.isMutatingLiveAction,
                            onToggleReady = onToggleReady,
                            onToggleQueue = onToggleQueue,
                            onChangeQueue = onChangeQueue,
                            onToggleAccessibility = onToggleAccessibility,
                            onGenerateInvite = onGenerateInvite,
                            onDisableInvite = onDisableInvite,
                            onOpenJoinDialog = { showJoinDialog = true },
                            onInspectPlayer = onInspectPlayer,
                        )

                        is LiveState.PreGame -> PreGameView(
                            snapshot = live.snapshot,
                            agents = uiState.agents,
                            ownedAgentUuids = uiState.ownedAgentUuids,
                            pregameFirstSeenAtMs = uiState.pregameFirstSeenAtMs,
                            isMutating = uiState.isMutatingLiveAction,
                            onSelectAgent = onSelectPreGameAgent,
                            onLockAgent = onLockPreGameAgent,
                            onQuitMatch = onQuitPreGameMatch,
                            onInspectPlayer = onInspectPlayer,
                        )

                        is LiveState.CoreGame -> CoreGameView(
                            snapshot = live.snapshot,
                            onInspectPlayer = onInspectPlayer,
                        )
                    }
                }
            }
        }
    }

    if (showJoinDialog) {
        JoinPartyCodeDialog(
            onDismiss = { showJoinDialog = false },
            onJoin = { code ->
                showJoinDialog = false
                onJoinPartyCode(code)
            },
        )
    }
}

@Composable
private fun SessionHeaderBar(
    uiState: MyValUiState,
    onOpenLogin: () -> Unit,
    onOpenAccounts: () -> Unit,
    onOpenPlayerSearch: () -> Unit,
    onOpenAbout: () -> Unit,
    onManualPoll: () -> Unit,
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .background(SurfaceCard)
            .padding(horizontal = 16.dp, vertical = 12.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = "MYVAL",
                color = ValRed,
                fontWeight = FontWeight.Black,
                fontSize = 18.sp,
                letterSpacing = 1.5.sp,
            )
            Spacer(modifier = Modifier.height(6.dp))
            val session = uiState.session
            if (session != null) {
                AccountSwitcherChip(
                    riotId = session.riotId,
                    shard = session.shard,
                    accountCount = uiState.savedAccounts.size,
                    onClick = onOpenAccounts,
                )
            } else {
                Text(
                    text = "Not Signed In",
                    color = TextSecondary,
                    fontSize = 12.sp,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }
        }

        if (uiState.session != null) {
            IconButton(onClick = onOpenPlayerSearch, modifier = Modifier.size(38.dp)) {
                Icon(
                    imageVector = Icons.Default.PersonSearch,
                    contentDescription = "Search player",
                    tint = ValCyan,
                    modifier = Modifier.size(20.dp),
                )
            }
        } else {
            Button(
                onClick = onOpenLogin,
                colors = ButtonDefaults.buttonColors(containerColor = ValRed),
                contentPadding = PaddingValues(horizontal = 12.dp, vertical = 6.dp),
            ) {
                Icon(Icons.AutoMirrored.Filled.Login, contentDescription = null, modifier = Modifier.size(16.dp))
                Spacer(modifier = Modifier.width(6.dp))
                Text("Sign In", fontSize = 13.sp, fontWeight = FontWeight.Bold)
            }
            Spacer(modifier = Modifier.width(4.dp))
        }

        Box {
            IconButton(onClick = onOpenAbout, modifier = Modifier.size(36.dp)) {
                Icon(
                    imageVector = Icons.Default.Info,
                    contentDescription = "About & updates",
                    tint = if (uiState.availableUpdate != null) ValWinGreen else TextSecondary,
                    modifier = Modifier.size(19.dp),
                )
            }
            if (uiState.availableUpdate != null) {
                Box(
                    modifier = Modifier
                        .align(Alignment.TopEnd)
                        .padding(top = 6.dp, end = 6.dp)
                        .size(7.dp)
                        .background(ValWinGreen, CircleShape),
                )
            }
        }
    }
}

@Composable
private fun IdleView(
    uiState: MyValUiState,
    shard: String,
    isPolling: Boolean,
    onManualPoll: () -> Unit,
    onOpenPlayerSearch: () -> Unit,
    onNavigateToTab: (AppTab) -> Unit,
    onSelectMatch: (String) -> Unit,
    onInspectSelf: () -> Unit,
) {
    val infinite = rememberInfiniteTransition(label = "radarPulse")
    val pulseScale by infinite.animateFloat(
        initialValue = 0.88f,
        targetValue = 1.24f,
        animationSpec = infiniteRepeatable(tween(1500), RepeatMode.Reverse),
        label = "pulseScale",
    )

    val career = uiState.career
    val store = uiState.storefront
    val lastMatch = career?.recentMatches?.firstOrNull()

    LazyColumn(
        modifier = Modifier.fillMaxSize(),
        contentPadding = PaddingValues(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        // 1. Lightweight Live Telemetry Status Strip directly on the OLED canvas (no heavy outer card box)
        item {
            Column(modifier = Modifier.fillMaxWidth()) {
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(horizontal = 2.dp, vertical = 4.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Box(contentAlignment = Alignment.Center, modifier = Modifier.size(34.dp)) {
                        Box(
                            modifier = Modifier
                                .size(26.dp)
                                .scale(pulseScale)
                                .background(ValCyan.copy(alpha = 0.16f), CircleShape),
                        )
                        Icon(
                            imageVector = Icons.Default.Radar,
                            contentDescription = null,
                            tint = ValCyan,
                            modifier = Modifier.size(18.dp),
                        )
                    }
                    Spacer(modifier = Modifier.width(10.dp))
                    Column(modifier = Modifier.weight(1f)) {
                        Text(
                            text = "Valorant Offline",
                            fontSize = 14.sp,
                            fontWeight = FontWeight.Black,
                            color = TextPrimary,
                        )
                        Text(
                            text = if (isPolling) {
                                "Checking PC session..."
                            } else {
                                "Open Valorant on PC • $shard"
                            },
                            fontSize = 11.5.sp,
                            color = ValCyan,
                            fontWeight = FontWeight.SemiBold,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                        )
                    }
                    Spacer(modifier = Modifier.width(8.dp))
                    OutlinedButton(
                        onClick = onManualPoll,
                        shape = InnerCardShape,
                        contentPadding = PaddingValues(horizontal = 12.dp, vertical = 6.dp),
                    ) {
                        Icon(
                            imageVector = Icons.Default.Refresh,
                            contentDescription = "Check now",
                            tint = ValCyan,
                            modifier = Modifier.size(15.dp),
                        )
                        Spacer(modifier = Modifier.width(4.dp))
                        Text("Check", color = ValCyan, fontSize = 12.sp, fontWeight = FontWeight.Bold)
                    }
                }
                Spacer(modifier = Modifier.height(6.dp))
                HorizontalDivider(thickness = 0.6.dp, color = SurfaceStroke.copy(alpha = 0.7f))
            }
        }

        // 2. Account Rank & Wallet Snapshot Card
        if (career != null || store != null) {
            item {
                val rankColor = career?.let { parseHexColor(it.currentRank.colorHex, ValCyan) } ?: ValCyan
                Surface(
                    color = SurfaceCard,
                    shape = RoundedCornerShape(10.dp),
                    modifier = Modifier
                        .fillMaxWidth()
                        .border(1.dp, rankColor.copy(alpha = 0.45f), RoundedCornerShape(10.dp))
                        .clickable(onClick = onInspectSelf),
                ) {
                    Column(
                        modifier = Modifier
                            .background(
                                Brush.horizontalGradient(
                                    listOf(rankColor.copy(alpha = 0.14f), SurfaceCard),
                                ),
                            )
                            .padding(14.dp),
                    ) {
                        if (career != null) {
                            Row(verticalAlignment = Alignment.CenterVertically) {
                                if (career.currentRank.iconUrl != null) {
                                    AsyncImage(
                                        model = career.currentRank.iconUrl,
                                        contentDescription = career.currentRank.tierName,
                                        modifier = Modifier.size(48.dp),
                                    )
                                    Spacer(modifier = Modifier.width(12.dp))
                                }
                                Column(modifier = Modifier.weight(1f)) {
                                    Row(
                                        modifier = Modifier.fillMaxWidth(),
                                        horizontalArrangement = Arrangement.SpaceBetween,
                                        verticalAlignment = Alignment.CenterVertically,
                                    ) {
                                        Text(
                                            text = career.currentRank.tierName,
                                            color = rankColor,
                                            fontWeight = FontWeight.Black,
                                            fontSize = 17.sp,
                                            maxLines = 1,
                                            overflow = TextOverflow.Ellipsis,
                                            modifier = Modifier.weight(1f, fill = false),
                                        )
                                        if (career.currentStreakLabel != "EVEN") {
                                            Spacer(modifier = Modifier.width(8.dp))
                                            TacticalBadge(
                                                text = career.currentStreakLabel,
                                                accent = if (career.currentStreakLabel.contains("W")) ValGreen else ValRed,
                                            )
                                        }
                                    }
                                    Spacer(modifier = Modifier.height(3.dp))
                                    val netSign = if (career.netRrLast10 >= 0) "+" else ""
                                    Text(
                                        text = "${career.currentRank.rr} RR • Peak ${career.peakRank.tierName} • $netSign${career.netRrLast10} RR (10G)",
                                        color = TextSecondary,
                                        fontSize = 11.5.sp,
                                        fontWeight = FontWeight.SemiBold,
                                        maxLines = 1,
                                        overflow = TextOverflow.Ellipsis,
                                    )
                                    if (career.currentRank.tier in 3..23) {
                                        Spacer(modifier = Modifier.height(6.dp))
                                        LinearProgressIndicator(
                                            progress = { (career.currentRank.rr.coerceIn(0, 100)) / 100f },
                                            modifier = Modifier
                                                .fillMaxWidth()
                                                .height(5.dp)
                                                .clip(RoundedCornerShape(8.dp)),
                                            color = rankColor,
                                            trackColor = SurfaceElevated,
                                        )
                                    }
                                }
                            }
                        }

                        if (career != null && store != null) {
                            Spacer(modifier = Modifier.height(12.dp))
                            HorizontalDivider(color = SurfaceStroke.copy(alpha = 0.6f))
                            Spacer(modifier = Modifier.height(10.dp))
                        }

                        if (store != null) {
                            Row(
                                modifier = Modifier.fillMaxWidth(),
                                horizontalArrangement = Arrangement.SpaceBetween,
                            ) {
                                HomeWalletMetric("VP", "${store.wallet.vp}", ValCyan)
                                HomeWalletMetric("RADIANITE", "${store.wallet.radianite}", ValGold)
                                HomeWalletMetric("KINGDOM", "${store.wallet.kingdomCredits}", ValPurple)
                            }
                        }
                    }
                }
            }
        }

        // 3. Quick Actions Row (Global Player Search + Arsenal)
        item {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(10.dp),
            ) {
                HomeQuickActionCard(
                    title = "Player Lookup",
                    subtitle = "Search any Riot ID",
                    icon = Icons.Default.PersonSearch,
                    accent = ValCyan,
                    onClick = onOpenPlayerSearch,
                    modifier = Modifier.weight(1f),
                )
                HomeQuickActionCard(
                    title = "Arsenal",
                    subtitle = "Equipped Skins",
                    icon = Icons.Default.Shield,
                    accent = ValGold,
                    onClick = { onNavigateToTab(AppTab.LOADOUT) },
                    modifier = Modifier.weight(1f),
                )
            }
        }

        // 4. Today's Daily Shop Quick-Glance Strip
        if (store != null && store.dailyOffers.isNotEmpty()) {
            item {
                Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Row(verticalAlignment = Alignment.CenterVertically) {
                            Icon(
                                imageVector = Icons.Default.Storefront,
                                contentDescription = null,
                                tint = ValRed,
                                modifier = Modifier.size(15.dp),
                            )
                            Spacer(modifier = Modifier.width(6.dp))
                            Text(
                                text = "TODAY'S SHOP",
                                color = TextSecondary,
                                fontSize = 11.5.sp,
                                fontWeight = FontWeight.Black,
                                letterSpacing = 1.sp,
                            )
                            Spacer(modifier = Modifier.width(8.dp))
                            TacticalBadge(formatDuration(store.dailyRemainingSecs), ValAmber)
                            if (store.nightMarket != null) {
                                Spacer(modifier = Modifier.width(6.dp))
                                TacticalBadge("NIGHT MKT", ValPurple)
                            }
                        }
                        Text(
                            text = "Open →",
                            color = ValCyan,
                            fontSize = 11.5.sp,
                            fontWeight = FontWeight.Bold,
                            modifier = Modifier
                                .clip(RoundedCornerShape(6.dp))
                                .clickable { onNavigateToTab(AppTab.STORE) }
                                .padding(horizontal = 6.dp, vertical = 2.dp),
                        )
                    }

                    LazyRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        items(store.dailyOffers, key = { it.offerId }) { offer ->
                            val tierColor = parseHexColor(offer.tierColor, ValCyan)
                            Surface(
                                color = SurfaceCard,
                                shape = RoundedCornerShape(10.dp),
                                modifier = Modifier
                                    .width(156.dp)
                                    .border(1.dp, tierColor.copy(alpha = 0.5f), RoundedCornerShape(10.dp))
                                    .clickable { onNavigateToTab(AppTab.STORE) },
                            ) {
                                Column(modifier = Modifier.padding(10.dp)) {
                                    Box(modifier = Modifier.fillMaxWidth()) {
                                        if (offer.displayIcon != null) {
                                            AsyncImage(
                                                model = offer.displayIcon,
                                                contentDescription = offer.skinName,
                                                contentScale = ContentScale.Fit,
                                                modifier = Modifier
                                                    .fillMaxWidth()
                                                    .height(48.dp),
                                            )
                                        }
                                        if (offer.isWishlisted) {
                                            Icon(
                                                imageVector = Icons.Default.Favorite,
                                                contentDescription = "Wishlisted",
                                                tint = ValRed,
                                                modifier = Modifier
                                                    .size(15.dp)
                                                    .align(Alignment.TopEnd),
                                            )
                                        }
                                    }
                                    Spacer(modifier = Modifier.height(6.dp))
                                    Text(
                                        text = offer.skinName,
                                        color = TextPrimary,
                                        fontWeight = FontWeight.Bold,
                                        fontSize = 11.5.sp,
                                        maxLines = 1,
                                        overflow = TextOverflow.Ellipsis,
                                    )
                                    Spacer(modifier = Modifier.height(2.dp))
                                    Text(
                                        text = "${offer.costVp} VP",
                                        color = tierColor,
                                        fontWeight = FontWeight.Black,
                                        fontSize = 11.sp,
                                    )
                                }
                            }
                        }
                    }
                }
            }
        }

        // 5. Last Match Quick Card (Tappable directly into 2D Replay & Scoreboard)
        if (lastMatch != null) {
            item {
                Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Row(verticalAlignment = Alignment.CenterVertically) {
                            Icon(
                                imageVector = Icons.Default.EmojiEvents,
                                contentDescription = null,
                                tint = ValGold,
                                modifier = Modifier.size(15.dp),
                            )
                            Spacer(modifier = Modifier.width(6.dp))
                            Text(
                                text = "LAST MATCH",
                                color = TextSecondary,
                                fontSize = 11.5.sp,
                                fontWeight = FontWeight.Black,
                                letterSpacing = 1.sp,
                            )
                        }
                        Text(
                            text = "Career →",
                            color = ValCyan,
                            fontSize = 11.5.sp,
                            fontWeight = FontWeight.Bold,
                            modifier = Modifier
                                .clip(RoundedCornerShape(6.dp))
                                .clickable { onNavigateToTab(AppTab.CAREER) }
                                .padding(horizontal = 6.dp, vertical = 2.dp),
                        )
                    }

                    MatchHistoryRowCard(
                        match = lastMatch,
                        onClick = { onSelectMatch(lastMatch.matchId) },
                    )
                }
            }
        }
    }
}

@Composable
private fun HomeWalletMetric(label: String, value: String, accent: Color) {
    Column {
        Text(
            text = label,
            color = TextMuted,
            fontSize = 9.5.sp,
            fontWeight = FontWeight.Bold,
            letterSpacing = 0.6.sp,
        )
        Text(
            text = value,
            color = accent,
            fontSize = 14.sp,
            fontWeight = FontWeight.Black,
        )
    }
}

@Composable
private fun HomeQuickActionCard(
    title: String,
    subtitle: String,
    icon: androidx.compose.ui.graphics.vector.ImageVector,
    accent: Color,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Surface(
        color = SurfaceCard,
        shape = RoundedCornerShape(10.dp),
        modifier = modifier
            .border(1.dp, accent.copy(alpha = 0.35f), RoundedCornerShape(10.dp))
            .clickable(onClick = onClick),
    ) {
        Row(
            modifier = Modifier.padding(horizontal = 12.dp, vertical = 12.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Box(
                modifier = Modifier
                    .size(34.dp)
                    .background(accent.copy(alpha = 0.14f), RoundedCornerShape(8.dp)),
                contentAlignment = Alignment.Center,
            ) {
                Icon(
                    imageVector = icon,
                    contentDescription = null,
                    tint = accent,
                    modifier = Modifier.size(18.dp),
                )
            }
            Spacer(modifier = Modifier.width(10.dp))
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = title,
                    color = TextPrimary,
                    fontSize = 12.5.sp,
                    fontWeight = FontWeight.Bold,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                Text(
                    text = subtitle,
                    color = TextSecondary,
                    fontSize = 10.5.sp,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }
        }
    }
}

private fun formatQueueLabel(queueId: String): String = when (queueId.trim().lowercase()) {
    "competitive" -> "Competitive"
    "unrated" -> "Unrated"
    "swiftplay" -> "Swiftplay"
    "spikerush" -> "Spike Rush"
    "deathmatch" -> "Deathmatch"
    "hurm" -> "Team Deathmatch"
    "ggteam" -> "Escalation"
    "premier" -> "Premier"
    "newmap" -> "New Map"
    "custom" -> "Custom Game"
    "" -> "Match"
    else -> queueId.trim().replaceFirstChar { it.uppercase() }
}

private fun formatQueueElapsed(secs: Long): String {
    val safe = secs.coerceAtLeast(0L)
    val m = safe / 60L
    val s = safe % 60L
    return "%02d:%02d".format(Locale.US, m, s)
}

@Composable
private fun PartyView(
    party: PartySnapshot,
    isMutating: Boolean,
    onToggleReady: () -> Unit,
    onToggleQueue: () -> Unit,
    onChangeQueue: (String) -> Unit,
    onToggleAccessibility: () -> Unit,
    onGenerateInvite: () -> Unit,
    onDisableInvite: () -> Unit,
    onOpenJoinDialog: () -> Unit,
    onInspectPlayer: (puuid: String, displayName: String) -> Unit,
) {
    val clipboard = LocalClipboardManager.current
    val me = party.members.firstOrNull { it.isMe }
    val meReady = me?.isReady ?: false
    val isOpen = party.accessibility.equals("OPEN", ignoreCase = true)
    val queueBtnEnabled = if (party.isInQueue) party.canLeaveQueue else party.canStartQueue

    LazyColumn(
        modifier = Modifier.fillMaxSize(),
        contentPadding = PaddingValues(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        item {
            Surface(
                color = SurfaceCard,
                shape = RoundedCornerShape(10.dp),
                modifier = Modifier
                    .fillMaxWidth()
                    .border(
                        width = if (party.isInQueue || party.isMatchStarting) 1.5.dp else 1.dp,
                        color = when {
                            party.isMatchStarting -> ValGreen
                            party.isInQueue -> ValCyan
                            else -> SurfaceStroke
                        },
                        shape = RoundedCornerShape(10.dp),
                    ),
            ) {
                Column(modifier = Modifier.padding(16.dp)) {
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Column(modifier = Modifier.weight(1f)) {
                            Text(
                                text = formatQueueLabel(party.queueId).uppercase(),
                                fontWeight = FontWeight.Black,
                                fontSize = 18.sp,
                                color = TextPrimary,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis,
                            )
                            val statusText = when {
                                party.isMatchStarting -> "${party.members.size}/5 • MATCH FOUND • LOADING..."
                                party.isInQueue -> "${party.members.size}/5 • SEARCHING (${formatQueueElapsed(party.queueElapsedSecs)})"
                                else -> "${party.members.size}/5 in party • IN LOBBY"
                            }
                            Text(
                                text = statusText,
                                fontSize = 12.sp,
                                fontWeight = if (party.isInQueue || party.isMatchStarting) FontWeight.Bold else FontWeight.Normal,
                                color = when {
                                    party.isMatchStarting -> ValGreen
                                    party.isInQueue -> ValCyan
                                    else -> TextSecondary
                                },
                            )
                        }
                        Spacer(modifier = Modifier.width(8.dp))
                        OutlinedButton(
                            enabled = party.canToggleAccessibility && !isMutating,
                            onClick = onToggleAccessibility,
                            contentPadding = PaddingValues(horizontal = 12.dp, vertical = 6.dp),
                        ) {
                            Icon(
                                imageVector = if (isOpen) Icons.Default.LockOpen else Icons.Default.Lock,
                                contentDescription = null,
                                tint = when {
                                    !party.canToggleAccessibility -> TextMuted
                                    isOpen -> ValGreen
                                    else -> ValAmber
                                },
                                modifier = Modifier.size(15.dp),
                            )
                            Spacer(modifier = Modifier.width(5.dp))
                            Text(
                                text = if (isOpen) "OPEN" else "CLOSED",
                                color = when {
                                    !party.canToggleAccessibility -> TextMuted
                                    isOpen -> ValGreen
                                    else -> ValAmber
                                },
                                fontSize = 12.sp,
                                fontWeight = FontWeight.Bold,
                            )
                        }
                    }

                    party.errorMessage?.let { partyErr ->
                        Spacer(modifier = Modifier.height(10.dp))
                        Row(
                            modifier = Modifier
                                .fillMaxWidth()
                                .background(ValAmber.copy(alpha = 0.14f), RoundedCornerShape(8.dp))
                                .border(1.dp, ValAmber.copy(alpha = 0.45f), RoundedCornerShape(8.dp))
                                .padding(horizontal = 10.dp, vertical = 8.dp),
                            verticalAlignment = Alignment.CenterVertically,
                        ) {
                            Icon(
                                imageVector = Icons.Default.Warning,
                                contentDescription = null,
                                tint = ValAmber,
                                modifier = Modifier.size(16.dp),
                            )
                            Spacer(modifier = Modifier.width(8.dp))
                            Text(
                                text = partyErr,
                                color = ValAmber,
                                fontSize = 11.5.sp,
                                fontWeight = FontWeight.SemiBold,
                            )
                        }
                    }

                    Spacer(modifier = Modifier.height(12.dp))

                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.spacedBy(8.dp),
                    ) {
                        OutlinedButton(
                            enabled = party.canToggleReady && !isMutating,
                            onClick = onToggleReady,
                            modifier = Modifier.weight(1f),
                        ) {
                            Icon(
                                Icons.Default.CheckCircle,
                                contentDescription = null,
                                tint = when {
                                    !party.canToggleReady -> TextMuted
                                    meReady -> ValGreen
                                    else -> TextSecondary
                                },
                                modifier = Modifier.size(16.dp),
                            )
                            Spacer(modifier = Modifier.width(6.dp))
                            Text(
                                text = if (meReady) "Ready" else "Set Ready",
                                color = when {
                                    !party.canToggleReady -> TextMuted
                                    meReady -> ValGreen
                                    else -> TextPrimary
                                },
                                fontWeight = FontWeight.Bold,
                            )
                        }

                        Button(
                            enabled = queueBtnEnabled && !isMutating,
                            onClick = onToggleQueue,
                            modifier = Modifier.weight(1f),
                            colors = ButtonDefaults.buttonColors(
                                containerColor = when {
                                    party.isMatchStarting -> ValGreen
                                    party.isInQueue -> ValAmber
                                    else -> ValRed
                                },
                                contentColor = if (party.isInQueue || party.isMatchStarting) ObsidianBg else TextPrimary,
                                disabledContainerColor = SurfaceElevated,
                                disabledContentColor = TextMuted,
                            ),
                        ) {
                            if (isMutating) {
                                CircularProgressIndicator(
                                    modifier = Modifier.size(16.dp),
                                    color = ValCyan,
                                    strokeWidth = 2.dp,
                                )
                            } else {
                                Icon(
                                    imageVector = when {
                                        party.isMatchStarting -> Icons.Default.CheckCircle
                                        party.isInQueue -> Icons.Default.Stop
                                        else -> Icons.Default.PlayArrow
                                    },
                                    contentDescription = null,
                                    modifier = Modifier.size(18.dp),
                                )
                            }
                            Spacer(modifier = Modifier.width(6.dp))
                            Text(
                                text = when {
                                    party.isMatchStarting -> "Match Found"
                                    party.isInQueue -> "Leave Queue"
                                    else -> "Start Queue"
                                },
                                fontWeight = FontWeight.Bold,
                                maxLines = 1,
                                softWrap = false,
                            )
                        }
                    }

                    val queueBlockReason = party.queueActionDisabledReason
                    if (!queueBtnEnabled && queueBlockReason != null) {
                        Spacer(modifier = Modifier.height(6.dp))
                        Row(verticalAlignment = Alignment.CenterVertically) {
                            Icon(
                                imageVector = Icons.Default.Lock,
                                contentDescription = null,
                                tint = TextMuted,
                                modifier = Modifier.size(12.dp),
                            )
                            Spacer(modifier = Modifier.width(5.dp))
                            Text(
                                text = queueBlockReason,
                                color = TextMuted,
                                fontSize = 11.sp,
                                fontWeight = FontWeight.Medium,
                            )
                        }
                    }

                    Spacer(modifier = Modifier.height(12.dp))
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Text(
                            text = "Switch Queue Mode",
                            color = TextMuted,
                            fontSize = 11.sp,
                            fontWeight = FontWeight.SemiBold,
                        )
                        val changeQueueReason = party.changeQueueDisabledReason
                        if (!party.canChangeQueue && changeQueueReason != null) {
                            Row(verticalAlignment = Alignment.CenterVertically) {
                                Icon(
                                    imageVector = Icons.Default.Lock,
                                    contentDescription = null,
                                    tint = ValAmber,
                                    modifier = Modifier.size(12.dp),
                                )
                                Spacer(modifier = Modifier.width(4.dp))
                                Text(
                                    text = changeQueueReason,
                                    color = ValAmber,
                                    fontSize = 10.5.sp,
                                    fontWeight = FontWeight.SemiBold,
                                    maxLines = 1,
                                    overflow = TextOverflow.Ellipsis,
                                )
                            }
                        }
                    }
                    Spacer(modifier = Modifier.height(6.dp))

                    Row(
                        modifier = Modifier.horizontalScroll(rememberScrollState()),
                        horizontalArrangement = Arrangement.spacedBy(6.dp),
                    ) {
                        party.availableQueues.forEach { q ->
                            val active = q.equals(party.queueId, ignoreCase = true)
                            val isEligible = party.eligibleQueues.isEmpty() ||
                                party.eligibleQueues.any { it.equals(q, ignoreCase = true) }
                            val chipEnabled = party.canChangeQueue && isEligible && !isMutating
                            FilterChip(
                                selected = active,
                                enabled = chipEnabled,
                                onClick = {
                                    if (!active && chipEnabled) onChangeQueue(q)
                                },
                                leadingIcon = if (!isEligible && !active) {
                                    {
                                        Icon(
                                            imageVector = Icons.Default.Lock,
                                            contentDescription = "Ineligible queue",
                                            modifier = Modifier.size(13.dp),
                                        )
                                    }
                                } else {
                                    null
                                },
                                label = { Text(formatQueueLabel(q), fontSize = 12.sp) },
                                colors = FilterChipDefaults.filterChipColors(
                                    selectedContainerColor = ValCyan.copy(alpha = 0.2f),
                                    selectedLabelColor = ValCyan,
                                    disabledSelectedContainerColor = ValCyan.copy(alpha = 0.14f),
                                    disabledLabelColor = if (active) ValCyan.copy(alpha = 0.78f) else TextMuted.copy(alpha = 0.55f),
                                ),
                            )
                        }
                    }

                    Spacer(modifier = Modifier.height(10.dp))
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .background(SurfaceElevated, RoundedCornerShape(8.dp))
                            .padding(horizontal = 12.dp, vertical = 8.dp),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Column(modifier = Modifier.weight(1f)) {
                            Text("Invite Code", color = TextMuted, fontSize = 11.sp)
                            Text(
                                text = party.inviteCode ?: if (!isOpen) "Open party to generate" else "Disabled",
                                color = if (party.inviteCode != null) TextPrimary else TextSecondary,
                                fontWeight = FontWeight.Bold,
                                fontSize = if (party.inviteCode != null) 14.sp else 12.sp,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis,
                            )
                        }
                        val inviteCode = party.inviteCode
                        if (inviteCode != null) {
                            IconButton(onClick = { clipboard.setText(AnnotatedString(inviteCode)) }) {
                                Icon(Icons.Default.ContentCopy, contentDescription = "Copy", tint = ValCyan)
                            }
                            TextButton(
                                enabled = party.canDisableInviteCode && !isMutating,
                                onClick = onDisableInvite,
                            ) {
                                Text(
                                    "Off",
                                    color = if (party.canDisableInviteCode) ValRed else TextMuted,
                                    fontSize = 12.sp,
                                )
                            }
                        } else {
                            TextButton(
                                enabled = party.canGenerateInviteCode && !isMutating,
                                onClick = onGenerateInvite,
                            ) {
                                Text(
                                    "Generate",
                                    color = if (party.canGenerateInviteCode) ValCyan else TextMuted,
                                    fontSize = 12.sp,
                                    fontWeight = FontWeight.Bold,
                                )
                            }
                        }
                        TextButton(
                            enabled = party.canJoinByCode && !isMutating,
                            onClick = onOpenJoinDialog,
                        ) {
                            Text(
                                "Join",
                                color = if (party.canJoinByCode) TextPrimary else TextMuted,
                                fontSize = 12.sp,
                            )
                        }
                    }
                }
            }
        }

        items(party.members, key = { it.puuid }) { member ->
            Surface(
                color = SurfaceCard,
                shape = RoundedCornerShape(10.dp),
                modifier = Modifier
                    .fillMaxWidth()
                    .border(
                        width = if (member.isMe) 1.5.dp else 1.dp,
                        color = if (member.isMe) ValCyan else SurfaceStroke,
                        shape = RoundedCornerShape(10.dp),
                    )
                    .clickable { onInspectPlayer(member.puuid, member.displayName) },
            ) {
                Row(
                    modifier = Modifier.padding(12.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    if (member.currentRank.iconUrl != null) {
                        AsyncImage(
                            model = member.currentRank.iconUrl,
                            contentDescription = member.currentRank.tierName,
                            modifier = Modifier.size(38.dp),
                        )
                    } else {
                        Box(
                            modifier = Modifier
                                .size(38.dp)
                                .background(SurfaceElevated, CircleShape),
                            contentAlignment = Alignment.Center,
                        ) {
                            Text(
                                text = member.display_name_initial(),
                                color = ValCyan,
                                fontWeight = FontWeight.Bold,
                            )
                        }
                    }
                    Spacer(modifier = Modifier.width(12.dp))
                    Column(modifier = Modifier.weight(1f)) {
                        Row(verticalAlignment = Alignment.CenterVertically) {
                            Text(
                                text = member.displayName,
                                fontWeight = FontWeight.Bold,
                                color = TextPrimary,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis,
                                modifier = Modifier.weight(1f, fill = false),
                            )
                            if (member.isOwner) {
                                Spacer(modifier = Modifier.width(6.dp))
                                TacticalBadge("LEADER", ValGold)
                            }
                            if (member.isMe) {
                                Spacer(modifier = Modifier.width(6.dp))
                                TacticalBadge("YOU", ValCyan)
                            }
                        }
                        val lvlText = member.accountLevel?.let { "Lvl $it • " } ?: ""
                        Text(
                            text = "$lvlText${member.currentRank.tierName} • ${member.platform}",
                            color = TextSecondary,
                            fontSize = 12.sp,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                        )
                    }
                    Spacer(modifier = Modifier.width(8.dp))
                    TacticalBadge(
                        text = if (member.isReady) "READY" else "WAITING",
                        accent = if (member.isReady) ValGreen else TextMuted,
                    )
                }
            }
        }
    }
}

private fun uniffi.myval_ffi.PartyMemberInfo.display_name_initial(): String =
    displayName.firstOrNull()?.uppercase() ?: "?"

@Composable
private fun PreGameView(
    snapshot: PreGameSnapshot,
    agents: List<Agent>,
    ownedAgentUuids: Set<String>,
    pregameFirstSeenAtMs: Long?,
    isMutating: Boolean,
    onSelectAgent: (String) -> Unit,
    onLockAgent: (String) -> Unit,
    onQuitMatch: () -> Unit,
    onInspectPlayer: (puuid: String, displayName: String) -> Unit,
) {
    var showAgentPicker by remember { mutableStateOf(false) }
    var showQuitConfirm by remember { mutableStateOf(false) }
    LaunchedEffect(snapshot.canSelectAgent) {
        if (!snapshot.canSelectAgent) {
            showAgentPicker = false
        }
    }
    val resolvedFirstSeenMs = remember(snapshot.matchId, pregameFirstSeenAtMs) {
        pregameFirstSeenAtMs ?: SystemClock.elapsedRealtime()
    }
    val expandedAllies = remember { mutableStateMapOf<String, Boolean>() }
    val lockedCount = snapshot.allies.count { it.selectionState.equals("locked", ignoreCase = true) }
    val queueLabel = formatQueueLabel(snapshot.queueId).uppercase()
    val sideText = if (snapshot.allySide.isNotBlank()) {
        "YOU ARE ${snapshot.allySide.uppercase()}"
    } else {
        "AGENT SELECT"
    }
    val rightBadge = if (snapshot.phaseTimeRemainingSecs > 0L) {
        "${snapshot.phaseTimeRemainingSecs}s"
    } else {
        "$lockedCount/${snapshot.allies.size} LOCKED"
    }
    val timerColor = when {
        snapshot.phaseTimeRemainingSecs in 1L..10L -> ValRed
        snapshot.phaseTimeRemainingSecs > 10L -> ValCyan
        else -> ValGold
    }

    LazyColumn(
        modifier = Modifier.fillMaxSize(),
        contentPadding = PaddingValues(horizontal = 14.dp, vertical = 10.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        item {
            MapHeroBanner(
                mapName = snapshot.mapName,
                mapSplash = snapshot.mapSplash,
                subtitle = "$queueLabel • $sideText",
                rightBadge = rightBadge,
                rightBadgeColor = timerColor,
            )
        }

        item {
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 2.dp),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(
                    text = "YOUR TEAM ($lockedCount/${snapshot.allies.size} LOCKED)",
                    color = TextSecondary,
                    fontSize = 11.sp,
                    fontWeight = FontWeight.Black,
                    letterSpacing = 1.sp,
                )
                if (snapshot.enemyTeamSize > 0) {
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(2.dp),
                    ) {
                        Text(
                            text = "ENEMY ",
                            color = TextMuted,
                            fontSize = 10.sp,
                            fontWeight = FontWeight.Bold,
                        )
                        val total = snapshot.enemyTeamSize.coerceIn(1, 5)
                        val locked = snapshot.enemyTeamLockCount.coerceIn(0, total)
                        repeat(total) { idx ->
                            val isLocked = idx < locked
                            Icon(
                                imageVector = if (isLocked) Icons.Default.Person else Icons.Default.PersonOutline,
                                contentDescription = null,
                                tint = if (isLocked) ValRed else TextMuted,
                                modifier = Modifier.size(16.dp),
                            )
                        }
                    }
                }
            }
        }

        items(snapshot.allies, key = { it.puuid }) { ally ->
            val isLocked = ally.selectionState.equals("locked", ignoreCase = true)
            val isHovering = ally.agentUuid.isNotBlank() && !isLocked
            val rankColor = parseHexColor(ally.currentRank.colorHex, ValCyan)
            val hasStack = ally.partyGroupId != null
            val stackColor = stackBadgeColor(ally.partyGroupId)
            val hasSkins = ally.equippedSkins.isNotEmpty()
            val isExpanded = expandedAllies[ally.puuid] == true
            val cardBrush = when {
                hasStack -> stackTintedCardBrush(stackColor)
                ally.isMe -> GlossyElevatedBrush
                else -> GlossyCardBrush
            }
            val borderColor = when {
                hasStack -> stackColor.copy(alpha = 0.78f)
                ally.isMe -> ValCyan.copy(alpha = 0.65f)
                else -> SurfaceStroke
            }
            val borderWidth = if (hasStack || ally.isMe) 1.35.dp else 1.dp

            Surface(
                color = Color.Transparent,
                shape = SharpCardShape,
                modifier = Modifier
                    .fillMaxWidth()
                    .clip(SharpCardShape)
                    .background(cardBrush, SharpCardShape)
                    .border(borderWidth, borderColor, SharpCardShape)
                    .clickable {
                        if (hasSkins) {
                            expandedAllies[ally.puuid] = !isExpanded
                        } else {
                            onInspectPlayer(ally.puuid, ally.displayName)
                        }
                    },
            ) {
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .then(
                            if (hasStack) {
                                Modifier.drawBehind {
                                    drawRect(
                                        color = stackColor,
                                        topLeft = Offset.Zero,
                                        size = Size(4.dp.toPx(), size.height),
                                    )
                                }
                            } else {
                                Modifier
                            },
                        ),
                ) {
                    Column(
                        modifier = Modifier.padding(
                            start = if (hasStack) 13.dp else 10.dp,
                            top = 9.dp,
                            end = 10.dp,
                            bottom = 9.dp,
                        ),
                    ) {
                        Row(verticalAlignment = Alignment.CenterVertically) {
                            Box(
                                modifier = Modifier
                                    .size(44.dp)
                                    .clickable { onInspectPlayer(ally.puuid, ally.displayName) },
                            ) {
                                AgentAvatar(
                                    iconUrl = ally.agentIcon,
                                    agentName = ally.agentName,
                                    dimmed = isHovering,
                                    borderColor = when {
                                        hasStack -> stackColor.copy(alpha = 0.85f)
                                        isLocked -> ValCyan.copy(alpha = 0.65f)
                                        isHovering -> ValAmber.copy(alpha = 0.65f)
                                        else -> SurfaceStroke
                                    },
                                )
                                // Statics-style compact premade-party icon pinned to avatar corner right in Agent Select
                                if (hasStack) {
                                    Box(
                                        modifier = Modifier
                                            .align(Alignment.BottomEnd)
                                            .size(18.dp)
                                            .background(ObsidianBg, SharpChipShape)
                                            .border(1.dp, stackColor, SharpChipShape),
                                        contentAlignment = Alignment.Center,
                                    ) {
                                        Icon(
                                            imageVector = Icons.Default.Groups,
                                            contentDescription = "Premade party",
                                            tint = stackColor,
                                            modifier = Modifier.size(11.dp),
                                        )
                                    }
                                }
                            }
                            Spacer(modifier = Modifier.width(10.dp))
                            Column(modifier = Modifier.weight(1f)) {
                                Row(verticalAlignment = Alignment.CenterVertically) {
                                    Text(
                                        text = ally.displayName,
                                        fontWeight = FontWeight.Bold,
                                        color = TextPrimary,
                                        fontSize = 14.sp,
                                        maxLines = 1,
                                        overflow = TextOverflow.Ellipsis,
                                        modifier = Modifier.weight(1f, fill = false),
                                    )
                                    if (ally.isMe) {
                                        Spacer(modifier = Modifier.width(6.dp))
                                        TacticalBadge("YOU", ValCyan)
                                    }
                                    if (ally.incognito) {
                                        Spacer(modifier = Modifier.width(5.dp))
                                        Icon(
                                            Icons.Default.VisibilityOff,
                                            contentDescription = "Streamer mode enabled",
                                            tint = TextMuted,
                                            modifier = Modifier.size(13.dp),
                                        )
                                    }
                                }
                                Spacer(modifier = Modifier.height(3.dp))
                                Row(
                                    modifier = Modifier.fillMaxWidth(),
                                    horizontalArrangement = Arrangement.SpaceBetween,
                                    verticalAlignment = Alignment.CenterVertically,
                                ) {
                                    val statusText = when {
                                        ally.agentUuid.isBlank() -> "Selecting agent..."
                                        isLocked -> "${ally.agentName} • LOCKED"
                                        else -> "${ally.agentName} • HOVERING"
                                    }
                                    Text(
                                        text = statusText,
                                        color = if (isLocked) ValCyan else ValAmber,
                                        fontSize = 11.5.sp,
                                        fontWeight = FontWeight.SemiBold,
                                        maxLines = 1,
                                        overflow = TextOverflow.Ellipsis,
                                        modifier = Modifier.weight(1f, fill = false),
                                    )
                                    Spacer(modifier = Modifier.width(8.dp))
                                    val rankText = if (ally.currentRank.tier >= 3) {
                                        "${ally.currentRank.tierName} • ${ally.currentRank.rr} RR"
                                    } else {
                                        ally.currentRank.tierName
                                    }
                                    Text(
                                        text = rankText,
                                        color = rankColor,
                                        fontSize = 11.sp,
                                        fontWeight = FontWeight.Bold,
                                        maxLines = 1,
                                        softWrap = false,
                                    )
                                }
                                Spacer(modifier = Modifier.height(3.dp))
                                Row(
                                    modifier = Modifier.fillMaxWidth(),
                                    horizontalArrangement = Arrangement.SpaceBetween,
                                    verticalAlignment = Alignment.CenterVertically,
                                ) {
                                    Row(
                                        verticalAlignment = Alignment.CenterVertically,
                                        horizontalArrangement = Arrangement.spacedBy(6.dp),
                                    ) {
                                        ally.accountLevel?.let { lvl ->
                                            Text(
                                                text = "Lvl $lvl",
                                                color = TextMuted,
                                                fontSize = 10.sp,
                                                maxLines = 1,
                                            )
                                        }
                                        RecentFormDots(outcomes = ally.recentOutcomes)
                                    }
                                    Text(
                                        text = "Peak: ${ally.peakRank.tierName}",
                                        color = TextMuted,
                                        fontSize = 10.sp,
                                        maxLines = 1,
                                        softWrap = false,
                                    )
                                }
                            }
                            if (ally.currentRank.iconUrl != null) {
                                Spacer(modifier = Modifier.width(8.dp))
                                AsyncImage(
                                    model = ally.currentRank.iconUrl,
                                    contentDescription = ally.currentRank.tierName,
                                    modifier = Modifier
                                        .size(32.dp)
                                        .clickable { onInspectPlayer(ally.puuid, ally.displayName) },
                                )
                            }
                            if (hasSkins) {
                                Spacer(modifier = Modifier.width(4.dp))
                                Column(horizontalAlignment = Alignment.CenterHorizontally) {
                                    Text(
                                        text = "${ally.equippedSkins.size}",
                                        color = ValGold,
                                        fontSize = 9.5.sp,
                                        fontWeight = FontWeight.Black,
                                    )
                                    Icon(
                                        imageVector = if (isExpanded) Icons.Default.ExpandLess else Icons.Default.ExpandMore,
                                        contentDescription = "Inspect skins",
                                        tint = TextSecondary,
                                        modifier = Modifier.size(18.dp),
                                    )
                                }
                            }
                        }

                        AnimatedVisibility(visible = isExpanded && hasSkins) {
                            EquippedSkinsDrawer(
                                skins = ally.equippedSkins,
                                onInspectPlayer = { onInspectPlayer(ally.puuid, ally.displayName) },
                            )
                        }
                    }
                }
            }
        }

        item {
            Spacer(modifier = Modifier.height(4.dp))
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(10.dp),
            ) {
                Button(
                    enabled = snapshot.canSelectAgent && !isMutating && agents.isNotEmpty(),
                    onClick = { showAgentPicker = true },
                    modifier = Modifier.weight(1f),
                    colors = ButtonDefaults.buttonColors(
                        containerColor = ValCyan,
                        contentColor = ObsidianBg,
                        disabledContainerColor = SurfaceElevated,
                        disabledContentColor = TextMuted,
                    ),
                ) {
                    Icon(
                        imageVector = if (snapshot.canSelectAgent) Icons.Default.LockOpen else Icons.Default.Lock,
                        contentDescription = null,
                        modifier = Modifier.size(16.dp),
                    )
                    Spacer(modifier = Modifier.width(6.dp))
                    Text(
                        text = if (snapshot.canSelectAgent) "Select Agent" else "Agent Locked",
                        fontWeight = FontWeight.Black,
                        fontSize = 13.sp,
                        maxLines = 1,
                        softWrap = false,
                    )
                }

                OutlinedButton(
                    enabled = snapshot.canQuitMatch && !isMutating,
                    onClick = { showQuitConfirm = true },
                    modifier = Modifier.weight(1f),
                ) {
                    Icon(
                        imageVector = Icons.AutoMirrored.Filled.ExitToApp,
                        contentDescription = null,
                        tint = if (snapshot.canQuitMatch && !isMutating) ValRed else TextMuted,
                        modifier = Modifier.size(16.dp),
                    )
                    Spacer(modifier = Modifier.width(6.dp))
                    Text(
                        text = "Quit Match",
                        color = if (snapshot.canQuitMatch && !isMutating) ValRed else TextMuted,
                        fontWeight = FontWeight.Bold,
                        fontSize = 13.sp,
                        maxLines = 1,
                        softWrap = false,
                    )
                }
            }
        }
    }

    if (showAgentPicker) {
        PreGameAgentPickerDialog(
            agents = agents,
            ownedAgentUuids = ownedAgentUuids,
            lockedAgentUuids = snapshot.lockedAgentUuids,
            myCurrentAgentUuid = snapshot.myAgentUuid,
            matchId = snapshot.matchId,
            firstSeenAtMs = resolvedFirstSeenMs,
            onDismiss = { showAgentPicker = false },
            onHover = { agentUuid ->
                showAgentPicker = false
                onSelectAgent(agentUuid)
            },
            onLock = { agentUuid ->
                showAgentPicker = false
                onLockAgent(agentUuid)
            },
        )
    }

    if (showQuitConfirm) {
        AlertDialog(
            onDismissRequest = { showQuitConfirm = false },
            containerColor = SurfaceCard,
            title = { Text("Quit Agent Select?", fontWeight = FontWeight.Black, color = TextPrimary) },
            text = {
                Text(
                    "Leaving Agent Select counts as a match dodge and may apply a Riot queue penalty or RR loss.",
                    color = TextSecondary,
                    fontSize = 13.sp,
                )
            },
            confirmButton = {
                Button(
                    onClick = {
                        showQuitConfirm = false
                        onQuitMatch()
                    },
                    colors = ButtonDefaults.buttonColors(containerColor = ValRed),
                ) {
                    Text("Quit Match", fontWeight = FontWeight.Bold)
                }
            },
            dismissButton = {
                TextButton(onClick = { showQuitConfirm = false }) { Text("Cancel") }
            },
        )
    }
}

@Composable
private fun PreGameAgentPickerDialog(
    agents: List<Agent>,
    ownedAgentUuids: Set<String>,
    lockedAgentUuids: List<String>,
    myCurrentAgentUuid: String?,
    matchId: String,
    firstSeenAtMs: Long,
    onDismiss: () -> Unit,
    onHover: (String) -> Unit,
    onLock: (String) -> Unit,
) {
    val lockedSet = remember(lockedAgentUuids) { lockedAgentUuids.map { it.lowercase() }.toSet() }
    var pickedAgent by remember(myCurrentAgentUuid) {
        mutableStateOf(
            agents.firstOrNull { ag ->
                val lc = ag.uuid.lowercase()
                ag.uuid.equals(myCurrentAgentUuid, ignoreCase = true) &&
                    (ownedAgentUuids.isEmpty() || ownedAgentUuids.contains(lc)) &&
                    !lockedSet.contains(lc)
            },
        )
    }
    LaunchedEffect(lockedSet, ownedAgentUuids) {
        val cur = pickedAgent ?: return@LaunchedEffect
        val lc = cur.uuid.lowercase()
        val isOwned = ownedAgentUuids.isEmpty() || ownedAgentUuids.contains(lc)
        if (!isOwned || lockedSet.contains(lc)) {
            pickedAgent = null
        }
    }
    var nowElapsedMs by remember(matchId, firstSeenAtMs) { mutableLongStateOf(SystemClock.elapsedRealtime()) }
    var earlyLockWarned by remember(matchId) { mutableStateOf(false) }
    var earlyActionVerb by remember(matchId) { mutableStateOf("locking") }
    LaunchedEffect(matchId, firstSeenAtMs) {
        while (true) {
            nowElapsedMs = SystemClock.elapsedRealtime()
            if (fairPlayLockRemainingSecs(firstSeenAtMs, nowElapsedMs) == 0) break
            delay(250L)
        }
    }
    val lockWaitSecs = fairPlayLockRemainingSecs(firstSeenAtMs, nowElapsedMs)
    val sortedAgents = remember(agents, ownedAgentUuids, lockedSet) {
        agents.sortedWith(
            compareByDescending<Agent> {
                ownedAgentUuids.isEmpty() || ownedAgentUuids.contains(it.uuid.lowercase())
            }.thenBy { lockedSet.contains(it.uuid.lowercase()) }
                .thenBy { it.displayName },
        )
    }

    AlertDialog(
        onDismissRequest = onDismiss,
        containerColor = SurfaceCard,
        title = {
            Column {
                Text("Select Agent", fontWeight = FontWeight.Black, fontSize = 17.sp)
                Text(
                    text = pickedAgent?.let {
                        "Selected: ${it.displayName} (${FAIR_PLAY_LOCK_DELAY_SECS}s fair-play hover & lock wait)"
                    } ?: "Hover & Lock wait ${FAIR_PLAY_LOCK_DELAY_SECS}s (fair play — no instapicking)",
                    color = ValCyan,
                    fontSize = 11.5.sp,
                )
            }
        },
        text = {
            Column {
                LazyColumn(
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(310.dp),
                    verticalArrangement = Arrangement.spacedBy(8.dp),
                ) {
                items(sortedAgents, key = { it.uuid }) { agent ->
                    val isLockedByAlly = lockedSet.contains(agent.uuid.lowercase())
                    val isOwned = ownedAgentUuids.isEmpty() || ownedAgentUuids.contains(agent.uuid.lowercase())
                    val canPick = isOwned && !isLockedByAlly
                    val isSelected = pickedAgent?.uuid.equals(agent.uuid, ignoreCase = true)
                    Surface(
                        color = if (canPick) SurfaceElevated else ObsidianBg.copy(alpha = 0.6f),
                        shape = RoundedCornerShape(8.dp),
                        modifier = Modifier
                            .fillMaxWidth()
                            .border(
                                width = if (isSelected) 1.5.dp else 1.dp,
                                color = when {
                                    isSelected -> ValCyan
                                    canPick -> SurfaceStroke
                                    else -> SurfaceStroke.copy(alpha = 0.4f)
                                },
                                shape = RoundedCornerShape(8.dp),
                            )
                            .clickable(enabled = canPick) { pickedAgent = agent },
                    ) {
                        Row(
                            modifier = Modifier.padding(9.dp),
                            verticalAlignment = Alignment.CenterVertically,
                        ) {
                            AgentAvatar(
                                iconUrl = agent.displayIcon,
                                agentName = agent.displayName,
                                dimmed = !canPick,
                            )
                            Spacer(modifier = Modifier.width(10.dp))
                            Column(modifier = Modifier.weight(1f)) {
                                Text(
                                    text = agent.displayName,
                                    color = if (canPick) TextPrimary else TextMuted,
                                    fontWeight = FontWeight.Bold,
                                    fontSize = 13.5.sp,
                                    maxLines = 1,
                                    overflow = TextOverflow.Ellipsis,
                                )
                                agent.roleName?.let { role ->
                                    Text(
                                        text = role,
                                        color = if (canPick) TextSecondary else TextMuted,
                                        fontSize = 11.sp,
                                    )
                                }
                            }
                            when {
                                isLockedByAlly -> TacticalBadge("TAKEN", ValRed)
                                !isOwned -> TacticalBadge("UNOWNED", TextMuted)
                                isSelected -> TacticalBadge("PICKED", ValCyan)
                            }
                        }
                    }
                }
            }
            if (earlyLockWarned && lockWaitSecs > 0) {
                Spacer(modifier = Modifier.height(10.dp))
                Surface(
                    color = ValRed.copy(alpha = 0.14f),
                    shape = RoundedCornerShape(8.dp),
                    modifier = Modifier
                        .fillMaxWidth()
                        .border(1.dp, ValRed.copy(alpha = 0.45f), RoundedCornerShape(8.dp)),
                ) {
                    Text(
                        text = "Wait ${lockWaitSecs}s before $earlyActionVerb — MyVal strictly disables instapicking and instalocking for any unfair advantage.",
                        color = ValGold,
                        fontSize = 11.5.sp,
                        fontWeight = FontWeight.SemiBold,
                        lineHeight = 16.sp,
                        modifier = Modifier.padding(10.dp),
                    )
                }
            } else if (lockWaitSecs == 0 && earlyLockWarned) {
                Spacer(modifier = Modifier.height(10.dp))
                Surface(
                    color = ValCyan.copy(alpha = 0.12f),
                    shape = RoundedCornerShape(8.dp),
                    modifier = Modifier
                        .fillMaxWidth()
                        .border(1.dp, ValCyan.copy(alpha = 0.4f), RoundedCornerShape(8.dp)),
                ) {
                    Text(
                        text = "15s Fair-Play delay complete — you can hover or lock your agent now.",
                        color = ValCyan,
                        fontSize = 11.5.sp,
                        fontWeight = FontWeight.SemiBold,
                        modifier = Modifier.padding(10.dp),
                    )
                }
            }
            }
        },
        confirmButton = {
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                val chosen = pickedAgent
                val attemptPickAction: (Boolean, Agent) -> Unit = { isLock, agent ->
                    val liveNowMs = SystemClock.elapsedRealtime()
                    nowElapsedMs = liveNowMs
                    if (fairPlayLockRemainingSecs(firstSeenAtMs, liveNowMs) > 0) {
                        earlyActionVerb = if (isLock) "locking" else "hovering"
                        earlyLockWarned = true
                    } else if (isLock) {
                        onLock(agent.uuid)
                    } else {
                        onHover(agent.uuid)
                    }
                }
                OutlinedButton(
                    enabled = chosen != null,
                    onClick = { chosen?.let { attemptPickAction(false, it) } },
                ) {
                    Text(
                        text = if (lockWaitSecs > 0) "Hover (${lockWaitSecs}s)" else "Hover",
                        fontWeight = FontWeight.Bold,
                        color = if (lockWaitSecs > 0) ValGold else TextPrimary,
                    )
                }
                Button(
                    enabled = chosen != null,
                    onClick = { chosen?.let { attemptPickAction(true, it) } },
                    colors = ButtonDefaults.buttonColors(
                        containerColor = if (lockWaitSecs > 0) SurfaceElevated else ValRed,
                        contentColor = if (lockWaitSecs > 0) ValGold else TextPrimary,
                    ),
                ) {
                    Text(
                        text = if (lockWaitSecs > 0) "Lock (${lockWaitSecs}s)" else "Lock Agent",
                        fontWeight = FontWeight.Black,
                    )
                }
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) { Text("Close") }
        },
    )
}

@Composable
private fun CoreGameView(
    snapshot: CoreGameSnapshot,
    onInspectPlayer: (puuid: String, displayName: String) -> Unit,
) {
    val expandedPlayers = remember { mutableStateMapOf<String, Boolean>() }
    var isSplitGridView by remember(snapshot.matchId) { mutableStateOf(true) }
    var quickInspectPuuid by remember { mutableStateOf<String?>(null) }

    val subtitleText = remember(snapshot) {
        buildList {
            add(snapshot.queueId.uppercase())
            if (snapshot.serverName.isNotBlank()) add(snapshot.serverName.uppercase())
            if (snapshot.myStartingSide.isNotBlank()) add("STARTED ${snapshot.myStartingSide.uppercase()}")
            if (snapshot.serverName.isBlank() && snapshot.myStartingSide.isBlank()) {
                add("${snapshot.myTeam.size}v${snapshot.enemyTeam.size} LIVE MATCH")
            }
        }.joinToString(" • ")
    }

    val quickInspectPlayer = remember(snapshot, quickInspectPuuid) {
        quickInspectPuuid?.let { target ->
            snapshot.myTeam.firstOrNull { it.puuid == target }
                ?: snapshot.enemyTeam.firstOrNull { it.puuid == target }
        }
    }

    LazyColumn(
        modifier = Modifier.fillMaxSize(),
        contentPadding = PaddingValues(horizontal = 12.dp, vertical = 8.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        item {
            MapHeroBanner(
                mapName = snapshot.mapName,
                mapSplash = snapshot.mapSplash,
                subtitle = subtitleText,
                rightBadge = "AVG: ${snapshot.averageLobbyRankName}",
                rightBadgeColor = ValGold,
            )
        }

        item {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                    LiveViewModeChip(
                        label = "5v5 Grid",
                        selected = isSplitGridView,
                        onClick = { isSplitGridView = true },
                    )
                    LiveViewModeChip(
                        label = "Cards",
                        selected = !isSplitGridView,
                        onClick = { isSplitGridView = false },
                    )
                }
                Text(
                    text = if (isSplitGridView) "Tap player for skins & stats" else "${snapshot.detectedStacks.size} stacks detected",
                    color = TextMuted,
                    fontSize = 10.5.sp,
                    fontWeight = FontWeight.SemiBold,
                    maxLines = 1,
                )
            }
        }

        if (isSplitGridView) {
            item {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Box(modifier = Modifier.weight(1f)) {
                        TeamSectionHeader("YOUR TEAM (${snapshot.myTeam.size})", ValCyan)
                    }
                    Box(modifier = Modifier.weight(1f)) {
                        TeamSectionHeader("ENEMY TEAM (${snapshot.enemyTeam.size})", ValRed)
                    }
                }
            }

            val rowCount = maxOf(snapshot.myTeam.size, snapshot.enemyTeam.size)
            items(rowCount, key = { idx -> "grid-row-$idx" }) { idx ->
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(7.dp),
                ) {
                    val ally = snapshot.myTeam.getOrNull(idx)
                    if (ally != null) {
                        CompactLivePlayerTile(
                            player = ally,
                            isEnemy = false,
                            onClick = { quickInspectPuuid = ally.puuid },
                            modifier = Modifier.weight(1f),
                        )
                    } else {
                        Spacer(modifier = Modifier.weight(1f))
                    }

                    val enemy = snapshot.enemyTeam.getOrNull(idx)
                    if (enemy != null) {
                        CompactLivePlayerTile(
                            player = enemy,
                            isEnemy = true,
                            onClick = { quickInspectPuuid = enemy.puuid },
                            modifier = Modifier.weight(1f),
                        )
                    } else {
                        Spacer(modifier = Modifier.weight(1f))
                    }
                }
            }
        } else {
            item {
                TeamSectionHeader("YOUR TEAM", ValCyan)
            }

            items(snapshot.myTeam, key = { "ally-${it.puuid}" }) { player ->
                val isExpanded = expandedPlayers[player.puuid] == true
                ScoutedPlayerCard(
                    player = player,
                    isEnemy = false,
                    expanded = isExpanded,
                    onToggleExpand = { expandedPlayers[player.puuid] = !isExpanded },
                    onInspectPlayer = { onInspectPlayer(player.puuid, player.displayName) },
                )
            }

            item {
                Spacer(modifier = Modifier.height(4.dp))
                TeamSectionHeader("ENEMY TEAM", ValRed)
            }

            items(snapshot.enemyTeam, key = { "enemy-${it.puuid}" }) { player ->
                val isExpanded = expandedPlayers[player.puuid] == true
                ScoutedPlayerCard(
                    player = player,
                    isEnemy = true,
                    expanded = isExpanded,
                    onToggleExpand = { expandedPlayers[player.puuid] = !isExpanded },
                    onInspectPlayer = { onInspectPlayer(player.puuid, player.displayName) },
                )
            }
        }
    }

    quickInspectPlayer?.let { player ->
        val isEnemy = snapshot.enemyTeam.any { it.puuid == player.puuid }
        LivePlayerQuickInspectDialog(
            player = player,
            isEnemy = isEnemy,
            onDismiss = { quickInspectPuuid = null },
            onInspectFullStats = {
                quickInspectPuuid = null
                onInspectPlayer(player.puuid, player.displayName)
            },
        )
    }
}

@Composable
private fun LiveViewModeChip(
    label: String,
    selected: Boolean,
    onClick: () -> Unit,
) {
    Surface(
        color = if (selected) ValCyan.copy(alpha = 0.18f) else SurfaceCard,
        shape = SharpChipShape,
        modifier = Modifier
            .border(
                1.dp,
                if (selected) ValCyan else SurfaceStroke,
                SharpChipShape,
            )
            .clip(SharpChipShape)
            .tactileClickable(pressedScale = 0.94f, onClick = onClick),
    ) {
        Text(
            text = label,
            color = if (selected) ValCyan else TextSecondary,
            fontSize = 11.sp,
            fontWeight = FontWeight.Bold,
            modifier = Modifier.padding(horizontal = 10.dp, vertical = 5.dp),
            maxLines = 1,
            softWrap = false,
        )
    }
}

private fun formatCompactRank(tierName: String, tier: Int, rr: Int): String {
    if (tier < 3) return "Unranked"
    val parts = tierName.trim().split(Regex("\\s+"))
    val base = parts.firstOrNull().orEmpty().uppercase()
    val div = parts.getOrNull(1).orEmpty()
    val shortBase = when (base) {
        "IRON" -> "Irn"
        "BRONZE" -> "Brz"
        "SILVER" -> "Sil"
        "GOLD" -> "Gld"
        "PLATINUM" -> "Plat"
        "DIAMOND" -> "Dia"
        "ASCENDANT" -> "Asc"
        "IMMORTAL" -> "Imm"
        "RADIANT" -> "Rad"
        else -> tierName.take(4)
    }
    val label = if (div.isNotEmpty()) "$shortBase $div" else shortBase
    return "$label · $rr"
}

@Composable
private fun CompactLivePlayerTile(
    player: ScoutedPlayer,
    isEnemy: Boolean,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val hasStack = player.partyGroupId != null
    val stackColor = stackBadgeColor(player.partyGroupId)
    val cardBrush = when {
        hasStack -> stackTintedCardBrush(stackColor)
        player.isMe -> GlossyElevatedBrush
        else -> GlossyCardBrush
    }
    val borderColor = when {
        hasStack -> stackColor.copy(alpha = 0.78f)
        player.isMe -> ValCyan.copy(alpha = 0.65f)
        else -> SurfaceStroke
    }
    val nameColor = when {
        player.isMe -> ValGold
        isEnemy -> ValRed
        else -> ValCyan
    }
    val rankColor = parseHexColor(player.currentRank.colorHex, ValCyan)

    Surface(
        color = Color.Transparent,
        shape = InnerCardShape,
        modifier = modifier
            .clip(InnerCardShape)
            .background(cardBrush, InnerCardShape)
            .border(if (hasStack || player.isMe) 1.25.dp else 1.dp, borderColor, InnerCardShape)
            .clickable(onClick = onClick),
    ) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .then(
                    if (hasStack || player.isMe) {
                        val stripeColor = if (hasStack) stackColor else ValCyan
                        Modifier.drawBehind {
                            drawRect(
                                color = stripeColor,
                                topLeft = Offset.Zero,
                                size = Size(3.dp.toPx(), size.height),
                            )
                        }
                    } else {
                        Modifier
                    },
                ),
        ) {
            Column(
                modifier = Modifier.padding(
                    start = if (hasStack || player.isMe) 9.dp else 7.dp,
                    end = 7.dp,
                    top = 7.dp,
                    bottom = 7.dp,
                ),
                verticalArrangement = Arrangement.spacedBy(5.dp),
            ) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Box(modifier = Modifier.size(34.dp)) {
                        Box(
                            modifier = Modifier
                                .size(34.dp)
                                .clip(RoundedCornerShape(6.dp))
                                .background(SurfaceElevated)
                                .border(
                                    1.dp,
                                    if (isEnemy) ValRed.copy(alpha = 0.45f) else ValCyan.copy(alpha = 0.45f),
                                    RoundedCornerShape(6.dp),
                                ),
                            contentAlignment = Alignment.Center,
                        ) {
                            if (player.agentIcon != null) {
                                AsyncImage(
                                    model = player.agentIcon,
                                    contentDescription = player.agentName,
                                    modifier = Modifier.fillMaxSize(),
                                    contentScale = ContentScale.Crop,
                                )
                            } else {
                                Icon(
                                    imageVector = Icons.Default.PersonOutline,
                                    contentDescription = player.agentName,
                                    tint = TextMuted,
                                    modifier = Modifier.size(16.dp),
                                )
                            }
                        }
                        if (hasStack) {
                            Box(
                                modifier = Modifier
                                    .align(Alignment.BottomEnd)
                                    .size(14.dp)
                                    .background(ObsidianBg, RoundedCornerShape(4.dp))
                                    .border(1.dp, stackColor, RoundedCornerShape(4.dp)),
                                contentAlignment = Alignment.Center,
                            ) {
                                Icon(
                                    imageVector = Icons.Default.Groups,
                                    contentDescription = "Premade party",
                                    tint = stackColor,
                                    modifier = Modifier.size(9.dp),
                                )
                            }
                        }
                    }

                    Spacer(modifier = Modifier.width(6.dp))

                    Column(modifier = Modifier.weight(1f)) {
                        Text(
                            text = if (player.isMe) "You" else player.displayName,
                            color = nameColor,
                            fontSize = 11.5.sp,
                            fontWeight = FontWeight.Bold,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                        )
                        Text(
                            text = player.agentName,
                            color = TextSecondary,
                            fontSize = 10.sp,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                        )
                    }

                    if (player.currentRank.iconUrl != null) {
                        Spacer(modifier = Modifier.width(4.dp))
                        AsyncImage(
                            model = player.currentRank.iconUrl,
                            contentDescription = player.currentRank.tierName,
                            modifier = Modifier.size(22.dp),
                        )
                    }
                }

                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Text(
                        text = formatCompactRank(
                            player.currentRank.tierName,
                            player.currentRank.tier,
                            player.currentRank.rr,
                        ),
                        color = rankColor,
                        fontSize = 9.5.sp,
                        fontWeight = FontWeight.Bold,
                        maxLines = 1,
                        softWrap = false,
                        overflow = TextOverflow.Ellipsis,
                        modifier = Modifier.weight(1f, fill = false),
                    )
                    Spacer(modifier = Modifier.width(4.dp))
                    RecentFormDots(
                        outcomes = player.recentOutcomes,
                        fallbackWins = player.recentResults,
                        compact = true,
                    )
                }

                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    val peakShort = formatCompactRank(player.peakRank.tierName, player.peakRank.tier, 0)
                        .substringBefore(" · ")
                    Text(
                        text = if (player.partyGroupLabel != null) {
                            "${player.partyGroupLabel} • Pk $peakShort"
                        } else {
                            "Pk $peakShort"
                        },
                        color = if (hasStack) stackColor else TextMuted,
                        fontSize = 9.sp,
                        fontWeight = if (hasStack) FontWeight.Bold else FontWeight.Medium,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                        modifier = Modifier.weight(1f),
                    )
                }
            }
        }
    }
}

@Composable
private fun LivePlayerQuickInspectDialog(
    player: ScoutedPlayer,
    isEnemy: Boolean,
    onDismiss: () -> Unit,
    onInspectFullStats: () -> Unit,
) {
    val tierColor = parseHexColor(player.currentRank.colorHex, ValCyan)
    val stackColor = stackBadgeColor(player.partyGroupId)
    AlertDialog(
        onDismissRequest = onDismiss,
        containerColor = SurfaceCard,
        title = {
            Row(verticalAlignment = Alignment.CenterVertically) {
                AgentAvatar(
                    iconUrl = player.agentIcon,
                    agentName = player.agentName,
                    borderColor = if (isEnemy) ValRed.copy(alpha = 0.65f) else ValCyan.copy(alpha = 0.65f),
                )
                Spacer(modifier = Modifier.width(10.dp))
                Column(modifier = Modifier.weight(1f)) {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Text(
                            text = player.displayName,
                            color = if (player.isMe) ValGold else TextPrimary,
                            fontWeight = FontWeight.Black,
                            fontSize = 15.sp,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                            modifier = Modifier.weight(1f, fill = false),
                        )
                        if (player.isMe) {
                            Spacer(modifier = Modifier.width(6.dp))
                            TacticalBadge("YOU", ValCyan)
                        }
                    }
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Text(
                            text = buildString {
                                append(player.agentName)
                                player.accountLevel?.let { append(" • Lvl $it") }
                            },
                            color = if (isEnemy) ValRed else ValCyan,
                            fontSize = 11.5.sp,
                            fontWeight = FontWeight.SemiBold,
                        )
                        player.partyGroupLabel?.let { label ->
                            Spacer(modifier = Modifier.width(6.dp))
                            TacticalBadge(label, stackColor)
                        }
                    }
                }
            }
        },
        text = {
            Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
                Surface(
                    color = SurfaceElevated,
                    shape = InnerCardShape,
                    modifier = Modifier
                        .fillMaxWidth()
                        .border(1.dp, tierColor.copy(alpha = 0.4f), InnerCardShape),
                ) {
                    Row(
                        modifier = Modifier.padding(10.dp),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        if (player.currentRank.iconUrl != null) {
                            AsyncImage(
                                model = player.currentRank.iconUrl,
                                contentDescription = player.currentRank.tierName,
                                modifier = Modifier.size(34.dp),
                            )
                            Spacer(modifier = Modifier.width(10.dp))
                        }
                        Column(modifier = Modifier.weight(1f)) {
                            Text(
                                text = if (player.currentRank.tier >= 3) {
                                    "${player.currentRank.tierName} • ${player.currentRank.rr} RR"
                                } else {
                                    player.currentRank.tierName
                                },
                                color = tierColor,
                                fontWeight = FontWeight.Bold,
                                fontSize = 13.sp,
                            )
                            Text(
                                text = "Peak: ${player.peakRank.tierName}",
                                color = TextSecondary,
                                fontSize = 11.sp,
                            )
                        }
                        RecentFormDots(
                            outcomes = player.recentOutcomes,
                            fallbackWins = player.recentResults,
                        )
                    }
                }

                EquippedSkinsDrawer(
                    skins = player.equippedSkins,
                    onInspectPlayer = onInspectFullStats,
                    showHeaderStatsLink = false,
                )
            }
        },
        confirmButton = {
            Button(
                onClick = onInspectFullStats,
                colors = ButtonDefaults.buttonColors(
                    containerColor = ValCyan,
                    contentColor = ObsidianBg,
                ),
            ) {
                Icon(Icons.Default.QueryStats, contentDescription = null, modifier = Modifier.size(16.dp))
                Spacer(modifier = Modifier.width(6.dp))
                Text("Full Stats", fontWeight = FontWeight.Black)
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) { Text("Close") }
        },
    )
}

@Composable
private fun TeamSectionHeader(title: String, accent: Color) {
    Row(
        verticalAlignment = Alignment.CenterVertically,
        modifier = Modifier.padding(top = 4.dp),
    ) {
        Box(
            modifier = Modifier
                .width(4.dp)
                .height(16.dp)
                .background(accent, RoundedCornerShape(2.dp)),
        )
        Spacer(modifier = Modifier.width(8.dp))
        Text(
            text = title,
            color = accent,
            fontSize = 12.sp,
            fontWeight = FontWeight.Black,
            letterSpacing = 1.2.sp,
        )
    }
}

@Composable
private fun ScoutedPlayerCard(
    player: ScoutedPlayer,
    isEnemy: Boolean,
    expanded: Boolean,
    onToggleExpand: () -> Unit,
    onInspectPlayer: () -> Unit,
) {
    val hasStack = player.partyGroupId != null
    val stackColor = stackBadgeColor(player.partyGroupId)
    val cardBrush = when {
        hasStack -> stackTintedCardBrush(stackColor)
        player.isMe -> GlossyElevatedBrush
        else -> GlossyCardBrush
    }
    val borderColor = when {
        hasStack -> stackColor.copy(alpha = 0.78f)
        player.isMe -> ValCyan.copy(alpha = 0.55f)
        else -> SurfaceStroke
    }
    val borderWidth = if (hasStack || player.isMe) 1.35.dp else 1.dp

    Surface(
        color = Color.Transparent,
        shape = SharpCardShape,
        modifier = Modifier
            .fillMaxWidth()
            .clip(SharpCardShape)
            .background(cardBrush, SharpCardShape)
            .border(borderWidth, borderColor, SharpCardShape)
            .clickable(onClick = onToggleExpand),
    ) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .then(
                    if (hasStack) {
                        Modifier.drawBehind {
                            drawRect(
                                color = stackColor,
                                topLeft = Offset.Zero,
                                size = Size(4.dp.toPx(), size.height),
                            )
                        }
                    } else {
                        Modifier
                    },
                ),
        ) {
            Column(
                modifier = Modifier.padding(
                    start = if (hasStack) 13.dp else 10.dp,
                    top = 9.dp,
                    end = 10.dp,
                    bottom = 9.dp,
                ),
            ) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Box(
                    modifier = Modifier
                        .size(44.dp)
                        .clickable(onClick = onInspectPlayer),
                ) {
                    AgentAvatar(
                        iconUrl = player.agentIcon,
                        agentName = player.agentName,
                        borderColor = when {
                            hasStack -> stackColor.copy(alpha = 0.85f)
                            isEnemy -> ValRed.copy(alpha = 0.45f)
                            else -> ValCyan.copy(alpha = 0.45f)
                        },
                    )
                    // Statics-style compact premade-party icon pinned to avatar corner
                    if (hasStack) {
                        Box(
                            modifier = Modifier
                                .align(Alignment.BottomEnd)
                                .size(18.dp)
                                .background(ObsidianBg, SharpChipShape)
                                .border(1.dp, stackColor, SharpChipShape),
                            contentAlignment = Alignment.Center,
                        ) {
                            Icon(
                                imageVector = Icons.Default.Groups,
                                contentDescription = "Premade party",
                                tint = stackColor,
                                modifier = Modifier.size(11.dp),
                            )
                        }
                    }
                }
                Spacer(modifier = Modifier.width(10.dp))

                Column(modifier = Modifier.weight(1f)) {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Text(
                            text = player.displayName,
                            color = TextPrimary,
                            fontWeight = FontWeight.Bold,
                            fontSize = 14.sp,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                            modifier = Modifier.weight(1f, fill = false),
                        )
                        if (player.isMe) {
                            Spacer(modifier = Modifier.width(6.dp))
                            TacticalBadge("YOU", ValCyan)
                        }
                        if (player.incognito && !player.isMe) {
                            Spacer(modifier = Modifier.width(5.dp))
                            Icon(
                                Icons.Default.VisibilityOff,
                                contentDescription = "Streamer mode enabled",
                                tint = TextMuted,
                                modifier = Modifier.size(13.dp),
                            )
                        }
                    }

                    Spacer(modifier = Modifier.height(3.dp))
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Row(
                            verticalAlignment = Alignment.CenterVertically,
                            modifier = Modifier.weight(1f, fill = false),
                        ) {
                            Text(
                                text = player.agentName,
                                color = if (isEnemy) ValRed else ValCyan,
                                fontSize = 11.5.sp,
                                fontWeight = FontWeight.SemiBold,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis,
                            )
                            player.accountLevel?.let { lvl ->
                                Text(
                                    text = " • Lvl $lvl",
                                    color = TextSecondary,
                                    fontSize = 11.sp,
                                    maxLines = 1,
                                )
                            }
                        }
                        Spacer(modifier = Modifier.width(8.dp))
                        val tierColor = parseHexColor(player.currentRank.colorHex, ValCyan)
                        val rankText = if (player.currentRank.tier >= 3) {
                            "${player.currentRank.tierName} • ${player.currentRank.rr} RR"
                        } else {
                            player.currentRank.tierName
                        }
                        Text(
                            text = rankText,
                            color = tierColor,
                            fontSize = 11.sp,
                            fontWeight = FontWeight.Bold,
                            maxLines = 1,
                            softWrap = false,
                        )
                    }

                    Spacer(modifier = Modifier.height(3.dp))
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Row(verticalAlignment = Alignment.CenterVertically) {
                            RecentFormDots(
                                outcomes = player.recentOutcomes,
                                fallbackWins = player.recentResults,
                            )
                        }
                        Text(
                            text = "Peak: ${player.peakRank.tierName}",
                            color = TextMuted,
                            fontSize = 10.sp,
                            maxLines = 1,
                            softWrap = false,
                        )
                    }
                }

                if (player.currentRank.iconUrl != null) {
                    Spacer(modifier = Modifier.width(8.dp))
                    AsyncImage(
                        model = player.currentRank.iconUrl,
                        contentDescription = player.currentRank.tierName,
                        modifier = Modifier
                            .size(32.dp)
                            .clickable(onClick = onInspectPlayer),
                    )
                }

                // Always keep the Loadout expand chevron visible so clicking any player reliably reveals their equipped skins
                Spacer(modifier = Modifier.width(4.dp))
                Column(horizontalAlignment = Alignment.CenterHorizontally) {
                    if (player.equippedSkins.isNotEmpty()) {
                        Text(
                            text = "${player.equippedSkins.size}",
                            color = ValGold,
                            fontSize = 9.5.sp,
                            fontWeight = FontWeight.Black,
                        )
                    }
                    Icon(
                        imageVector = if (expanded) Icons.Default.ExpandLess else Icons.Default.ExpandMore,
                        contentDescription = "Inspect equipped loadout",
                        tint = if (player.equippedSkins.isNotEmpty()) TextSecondary else TextMuted,
                        modifier = Modifier.size(18.dp),
                    )
                }
            }

            AnimatedVisibility(visible = expanded) {
                EquippedSkinsDrawer(
                    skins = player.equippedSkins,
                    onInspectPlayer = onInspectPlayer,
                )
            }
            }
        }
    }
}

@Composable
private fun RecentFormDots(
    outcomes: List<Byte>,
    fallbackWins: List<Boolean> = emptyList(),
    compact: Boolean = false,
) {
    val resolved: List<Int> = when {
        outcomes.isNotEmpty() -> outcomes.take(5).map { it.toInt() }
        fallbackWins.isNotEmpty() -> fallbackWins.take(5).map { if (it) 1 else -1 }
        else -> emptyList()
    }
    if (resolved.isEmpty()) return

    val dotSize = if (compact) 5.dp else 7.dp
    val dotGap = if (compact) 2.dp else 3.5.dp
    // Dark obsidian backing pill keeps Win/Loss/Draw pips crisp even over colored party-stack cards
    Row(
        modifier = Modifier
            .background(ObsidianBg.copy(alpha = 0.80f), RoundedCornerShape(4.dp))
            .border(0.75.dp, SurfaceStroke.copy(alpha = 0.75f), RoundedCornerShape(4.dp))
            .padding(
                horizontal = if (compact) 3.5.dp else 5.dp,
                vertical = if (compact) 2.dp else 2.5.dp,
            ),
        horizontalArrangement = Arrangement.spacedBy(dotGap),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        resolved.forEach { result ->
            val dotColor = when {
                result > 0 -> ValWinGreen
                result < 0 -> ValLossRed
                else -> ValDrawSlate
            }
            Box(
                modifier = Modifier
                    .size(dotSize)
                    .background(dotColor, RoundedCornerShape(2.dp)),
            )
        }
    }
}

@Composable
private fun EquippedSkinsDrawer(
    skins: List<EquippedWeaponSkin>,
    onInspectPlayer: () -> Unit,
    showHeaderStatsLink: Boolean = true,
) {
    val hasAnimatedOrTopTier = skins.any {
        it.isAnimated || it.tierName in setOf("Premium", "Exclusive", "Ultra")
    }
    val headerLabel = when {
        skins.isEmpty() -> "EQUIPPED LOADOUT"
        hasAnimatedOrTopTier -> "ANIMATED & PREMIUM SKINS (${skins.size})"
        else -> "EQUIPPED SKINS (${skins.size})"
    }

    Column(modifier = Modifier.padding(top = 10.dp)) {
        HorizontalDivider(thickness = 0.5.dp, color = SurfaceStroke.copy(alpha = 0.7f))
        Spacer(modifier = Modifier.height(8.dp))
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text(
                text = headerLabel,
                color = TextMuted,
                fontSize = 10.sp,
                fontWeight = FontWeight.Bold,
                letterSpacing = 1.sp,
            )
            if (showHeaderStatsLink) {
                Row(
                    modifier = Modifier
                        .clip(SharpChipShape)
                        .clickable(onClick = onInspectPlayer)
                        .padding(horizontal = 6.dp, vertical = 2.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Icon(
                        imageVector = Icons.Default.QueryStats,
                        contentDescription = "View player stats",
                        tint = ValCyan,
                        modifier = Modifier.size(13.dp),
                    )
                    Spacer(modifier = Modifier.width(4.dp))
                    Text(
                        text = "View Stats",
                        color = ValCyan,
                        fontSize = 10.5.sp,
                        fontWeight = FontWeight.Bold,
                    )
                }
            }
        }
        Spacer(modifier = Modifier.height(6.dp))
        if (skins.isEmpty()) {
            Text(
                text = "Standard / Default loadout equipped in this match.",
                color = TextSecondary,
                fontSize = 11.5.sp,
                modifier = Modifier.padding(vertical = 4.dp),
            )
        } else {
            LazyRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                items(skins, key = { "${it.weaponName}-${it.skinName}" }) { skin ->
                    val c = parseHexColor(skin.tierColor, ValCyan)
                    Surface(
                        color = SurfaceElevated,
                        shape = InnerCardShape,
                        modifier = Modifier
                            .width(148.dp)
                            .border(1.dp, c.copy(alpha = 0.55f), InnerCardShape),
                    ) {
                        Column(modifier = Modifier.padding(8.dp)) {
                            if (skin.skinIcon != null) {
                                AsyncImage(
                                    model = skin.skinIcon,
                                    contentDescription = skin.skinName,
                                    modifier = Modifier
                                        .fillMaxWidth()
                                        .height(42.dp),
                                    contentScale = ContentScale.Fit,
                                )
                                Spacer(modifier = Modifier.height(4.dp))
                            }
                            Text(
                                text = skin.skinName,
                                color = TextPrimary,
                                fontSize = 11.sp,
                                fontWeight = FontWeight.SemiBold,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis,
                            )
                            Spacer(modifier = Modifier.height(2.dp))
                            Row(
                                modifier = Modifier.fillMaxWidth(),
                                horizontalArrangement = Arrangement.SpaceBetween,
                                verticalAlignment = Alignment.CenterVertically,
                            ) {
                                Text(
                                    text = skin.weaponName,
                                    color = c,
                                    fontSize = 10.sp,
                                    fontWeight = FontWeight.Bold,
                                    maxLines = 1,
                                    overflow = TextOverflow.Ellipsis,
                                    modifier = Modifier.weight(1f, fill = false),
                                )
                                skin.tierName?.let { tier ->
                                    Spacer(modifier = Modifier.width(4.dp))
                                    Text(
                                        text = tier.uppercase(),
                                        color = c.copy(alpha = 0.85f),
                                        fontSize = 8.5.sp,
                                        fontWeight = FontWeight.Black,
                                        maxLines = 1,
                                        softWrap = false,
                                    )
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun MapHeroBanner(
    mapName: String,
    mapSplash: String?,
    subtitle: String,
    rightBadge: String,
    rightBadgeColor: Color,
) {
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .height(82.dp)
            .clip(SharpCardShape)
            .background(SurfaceCard)
            .border(1.dp, SurfaceStroke, SharpCardShape),
    ) {
        if (mapSplash != null) {
            AsyncImage(
                model = mapSplash,
                contentDescription = mapName,
                contentScale = ContentScale.Crop,
                modifier = Modifier.fillMaxSize(),
            )
        }
        Box(
            modifier = Modifier
                .fillMaxSize()
                .background(
                    Brush.horizontalGradient(
                        listOf(ObsidianBg.copy(alpha = 0.92f), ObsidianBg.copy(alpha = 0.55f)),
                    ),
                ),
        )
        Row(
            modifier = Modifier
                .fillMaxSize()
                .padding(horizontal = 16.dp, vertical = 14.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.SpaceBetween,
        ) {
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = mapName.uppercase(),
                    color = TextPrimary,
                    fontSize = 22.sp,
                    fontWeight = FontWeight.Black,
                    letterSpacing = 1.sp,
                )
                Text(
                    text = subtitle,
                    color = TextSecondary,
                    fontSize = 11.sp,
                    fontWeight = FontWeight.SemiBold,
                )
            }
            TacticalBadge(text = rightBadge, accent = rightBadgeColor)
        }
    }
}

@Composable
private fun AgentAvatar(
    iconUrl: String?,
    agentName: String,
    dimmed: Boolean = false,
    borderColor: Color = SurfaceStroke,
) {
    Box(
        modifier = Modifier
            .size(44.dp)
            .clip(InnerCardShape)
            .background(SurfaceElevated)
            .border(1.dp, borderColor, InnerCardShape),
        contentAlignment = Alignment.Center,
    ) {
        if (iconUrl != null) {
            AsyncImage(
                model = iconUrl,
                contentDescription = agentName,
                modifier = Modifier
                    .fillMaxSize()
                    .alpha(if (dimmed) 0.58f else 1f),
                contentScale = ContentScale.Crop,
            )
        } else {
            Icon(
                imageVector = Icons.Default.PersonOutline,
                contentDescription = agentName,
                tint = TextMuted,
                modifier = Modifier.size(22.dp),
            )
        }
    }
}

@Composable
private fun JoinPartyCodeDialog(
    onDismiss: () -> Unit,
    onJoin: (String) -> Unit,
) {
    var code by remember { mutableStateOf("") }
    AlertDialog(
        onDismissRequest = onDismiss,
        containerColor = SurfaceCard,
        title = { Text("Join Party by Invite Code", fontWeight = FontWeight.Bold) },
        text = {
            OutlinedTextField(
                value = code,
                onValueChange = { code = it.uppercase() },
                label = { Text("Invite Code (e.g. VAL-123)") },
                singleLine = true,
            )
        },
        confirmButton = {
            Button(
                onClick = { onJoin(code) },
                colors = ButtonDefaults.buttonColors(containerColor = ValRed),
            ) {
                Text("Join")
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) { Text("Cancel") }
        },
    )
}
