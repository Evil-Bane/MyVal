package com.myval.app.ui.career

import androidx.compose.animation.AnimatedVisibility
import java.util.Locale
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
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
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.ContentCopy
import androidx.compose.material.icons.filled.Key
import androidx.compose.material.icons.filled.PersonSearch
import androidx.compose.material.icons.filled.Public
import androidx.compose.material.icons.filled.Search
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
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
import com.myval.app.ui.components.MyValPullToRefreshBox
import com.myval.app.ui.components.TacticalBadge
import com.myval.app.ui.components.parseHexColor
import com.myval.app.ui.theme.ObsidianBg
import com.myval.app.ui.theme.SurfaceCard
import com.myval.app.ui.theme.SurfaceElevated
import com.myval.app.ui.theme.SurfaceStroke
import com.myval.app.ui.theme.TextMuted
import com.myval.app.ui.theme.TextPrimary
import com.myval.app.ui.theme.TextSecondary
import com.myval.app.ui.theme.ValAmber
import com.myval.app.ui.theme.ValCyan
import com.myval.app.ui.theme.ValGold
import com.myval.app.ui.theme.ValGreen
import com.myval.app.ui.theme.ValPurple
import com.myval.app.ui.theme.ValRed
import uniffi.myval_ffi.AgentPerformanceStat
import uniffi.myval_ffi.KnownPlayerSummary
import uniffi.myval_ffi.LiveState
import uniffi.myval_ffi.PlayerProfileSnapshot

@Composable
fun PlayerProfileScreen(
    hintName: String?,
    profile: PlayerProfileSnapshot?,
    isLoading: Boolean,
    error: String?,
    onClose: () -> Unit,
    onRefresh: () -> Unit,
    onOpenSearch: () -> Unit,
    onSelectMatch: (matchId: String, perspectivePuuid: String) -> Unit,
) {
    val clipboard = LocalClipboardManager.current
    val titleName = profile?.displayName?.takeIf { it.isNotBlank() }
        ?: hintName?.takeIf { it.isNotBlank() }
        ?: "Player Stats"

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(ObsidianBg),
    ) {
        // Top Header Bar
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .background(SurfaceCard)
                .padding(horizontal = 8.dp, vertical = 10.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            IconButton(onClick = onClose) {
                Icon(
                    imageVector = Icons.AutoMirrored.Filled.ArrowBack,
                    contentDescription = "Back",
                    tint = TextPrimary,
                )
            }
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = titleName,
                    color = TextPrimary,
                    fontWeight = FontWeight.Black,
                    fontSize = 16.sp,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }
            IconButton(onClick = onOpenSearch) {
                Icon(
                    imageVector = Icons.Default.PersonSearch,
                    contentDescription = "Search Riot ID",
                    tint = ValCyan,
                )
            }
        }

        if (profile == null) {
            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .padding(24.dp),
                contentAlignment = Alignment.Center,
            ) {
                if (error != null && !isLoading) {
                    Column(
                        horizontalAlignment = Alignment.CenterHorizontally,
                        verticalArrangement = Arrangement.spacedBy(12.dp),
                    ) {
                        Text(
                            text = "Unable to load player stats",
                            color = TextPrimary,
                            fontWeight = FontWeight.Bold,
                            fontSize = 15.sp,
                        )
                        Text(
                            text = error,
                            color = TextSecondary,
                            fontSize = 12.sp,
                        )
                        Button(
                            onClick = onRefresh,
                            colors = ButtonDefaults.buttonColors(containerColor = ValRed),
                        ) {
                            Text("Retry", fontWeight = FontWeight.Bold)
                        }
                    }
                } else {
                    Column(
                        horizontalAlignment = Alignment.CenterHorizontally,
                        verticalArrangement = Arrangement.spacedBy(12.dp),
                    ) {
                        CircularProgressIndicator(color = ValCyan)
                        Text(
                            text = "Scouting $titleName...",
                            color = TextSecondary,
                            fontSize = 13.sp,
                            fontWeight = FontWeight.SemiBold,
                        )
                    }
                }
            }
            return
        }

        val rankColor = parseHexColor(profile.currentRank.colorHex, ValCyan)

        MyValPullToRefreshBox(
            isRefreshing = isLoading,
            onRefresh = onRefresh,
            accent = ValCyan,
            modifier = Modifier
                .fillMaxWidth()
                .weight(1f),
        ) {
            LazyColumn(
                modifier = Modifier.fillMaxSize(),
                contentPadding = PaddingValues(16.dp),
                verticalArrangement = Arrangement.spacedBy(14.dp),
            ) {
            // 1. Player Identity & Banner Card
            item {
                Surface(
                    color = SurfaceCard,
                    shape = RoundedCornerShape(10.dp),
                    modifier = Modifier
                        .fillMaxWidth()
                        .border(1.dp, rankColor.copy(alpha = 0.55f), RoundedCornerShape(10.dp)),
                ) {
                    Box(modifier = Modifier.fillMaxWidth()) {
                        if (profile.playerCardWide != null) {
                            AsyncImage(
                                model = profile.playerCardWide,
                                contentDescription = "Player banner",
                                contentScale = ContentScale.Crop,
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .height(142.dp),
                            )
                        }
                        Column(
                            modifier = Modifier
                                .fillMaxWidth()
                                .background(
                                    Brush.verticalGradient(
                                        listOf(
                                            ObsidianBg.copy(alpha = 0.68f),
                                            SurfaceCard.copy(alpha = 0.96f),
                                        ),
                                    ),
                                )
                                .padding(14.dp),
                        ) {
                            Row(verticalAlignment = Alignment.CenterVertically) {
                                val avatarUrl = profile.playerCardSmall ?: profile.currentRank.iconUrl
                                if (avatarUrl != null) {
                                    AsyncImage(
                                        model = avatarUrl,
                                        contentDescription = profile.displayName,
                                        contentScale = ContentScale.Crop,
                                        modifier = Modifier
                                            .size(56.dp)
                                            .clip(RoundedCornerShape(10.dp))
                                            .background(SurfaceElevated)
                                            .border(1.5.dp, rankColor.copy(alpha = 0.7f), RoundedCornerShape(10.dp)),
                                    )
                                    Spacer(modifier = Modifier.width(12.dp))
                                }
                                Column(modifier = Modifier.weight(1f)) {
                                    Row(verticalAlignment = Alignment.CenterVertically) {
                                        Text(
                                            text = profile.displayName,
                                            color = TextPrimary,
                                            fontWeight = FontWeight.Black,
                                            fontSize = 18.sp,
                                            maxLines = 1,
                                            overflow = TextOverflow.Ellipsis,
                                            modifier = Modifier.weight(1f, fill = false),
                                        )
                                        Spacer(modifier = Modifier.width(6.dp))
                                        IconButton(
                                            onClick = { clipboard.setText(AnnotatedString(profile.displayName)) },
                                            modifier = Modifier.size(26.dp),
                                        ) {
                                            Icon(
                                                imageVector = Icons.Default.ContentCopy,
                                                contentDescription = "Copy Riot ID",
                                                tint = ValCyan,
                                                modifier = Modifier.size(15.dp),
                                            )
                                        }
                                    }
                                    profile.playerTitle?.let { title ->
                                        Text(
                                            text = title,
                                            color = TextSecondary,
                                            fontSize = 11.5.sp,
                                            maxLines = 1,
                                            overflow = TextOverflow.Ellipsis,
                                        )
                                    }
                                    Spacer(modifier = Modifier.height(6.dp))
                                    Row(
                                        horizontalArrangement = Arrangement.spacedBy(6.dp),
                                        verticalAlignment = Alignment.CenterVertically,
                                    ) {
                                        profile.accountLevel?.let { lvl ->
                                            TacticalBadge("LVL $lvl", TextSecondary)
                                        }
                                        profile.leaderboardRank?.let { lb ->
                                            TacticalBadge("#$lb", ValGold)
                                        }
                                        if (profile.currentStreakLabel != "EVEN") {
                                            TacticalBadge(
                                                text = profile.currentStreakLabel,
                                                accent = if (profile.currentStreakLabel.contains("W")) ValGreen else ValRed,
                                            )
                                        }
                                    }
                                }
                            }

                            Spacer(modifier = Modifier.height(14.dp))

                            // Rank Trio Bar
                            Row(
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .background(ObsidianBg.copy(alpha = 0.55f), RoundedCornerShape(10.dp))
                                    .padding(horizontal = 12.dp, vertical = 10.dp),
                                verticalAlignment = Alignment.CenterVertically,
                                horizontalArrangement = Arrangement.SpaceBetween,
                            ) {
                                Row(
                                    verticalAlignment = Alignment.CenterVertically,
                                    modifier = Modifier.weight(1f),
                                ) {
                                    if (profile.currentRank.iconUrl != null) {
                                        AsyncImage(
                                            model = profile.currentRank.iconUrl,
                                            contentDescription = profile.currentRank.tierName,
                                            modifier = Modifier.size(38.dp),
                                        )
                                        Spacer(modifier = Modifier.width(8.dp))
                                    }
                                    Column {
                                        Text("CURRENT", color = TextMuted, fontSize = 9.5.sp, fontWeight = FontWeight.Bold)
                                        Text(
                                            text = profile.currentRank.tierName,
                                            color = rankColor,
                                            fontWeight = FontWeight.Black,
                                            fontSize = 13.5.sp,
                                            maxLines = 1,
                                            overflow = TextOverflow.Ellipsis,
                                        )
                                        if (profile.currentRank.tier >= 3) {
                                            Text(
                                                text = "${profile.currentRank.rr} RR",
                                                color = TextSecondary,
                                                fontSize = 11.sp,
                                                fontWeight = FontWeight.SemiBold,
                                            )
                                        }
                                    }
                                }

                                Column(horizontalAlignment = Alignment.End) {
                                    Text("PEAK RANK", color = TextMuted, fontSize = 9.5.sp, fontWeight = FontWeight.Bold)
                                    Text(
                                        text = profile.peakRank.tierName,
                                        color = parseHexColor(profile.peakRank.colorHex, ValGold),
                                        fontWeight = FontWeight.Black,
                                        fontSize = 13.sp,
                                    )
                                    Text(
                                        text = "Prev Act: ${profile.previousActRank.tierName}",
                                        color = TextSecondary,
                                        fontSize = 10.5.sp,
                                    )
                                }
                            }
                        }
                    }
                }
            }

            // 2. Core Stats KPI Grid
            if (profile.matchesAnalyzed > 0u) {
                item {
                    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                        Row(
                            modifier = Modifier.fillMaxWidth(),
                            horizontalArrangement = Arrangement.SpaceBetween,
                            verticalAlignment = Alignment.CenterVertically,
                        ) {
                            Text(
                                text = "PERFORMANCE OVERVIEW",
                                fontWeight = FontWeight.Black,
                                fontSize = 12.sp,
                                letterSpacing = 1.sp,
                                color = TextSecondary,
                            )
                            Text(
                                text = "Last ${profile.matchesAnalyzed} Matches • ${profile.wins}W-${profile.losses}L",
                                color = TextMuted,
                                fontSize = 11.sp,
                                fontWeight = FontWeight.SemiBold,
                            )
                        }

                        Row(
                            modifier = Modifier.fillMaxWidth(),
                            horizontalArrangement = Arrangement.spacedBy(8.dp),
                        ) {
                            StatKpiCard(
                                label = "K/D RATIO",
                                value = "%.2f".format(Locale.US, profile.kdRatio),
                                accent = when {
                                    profile.kdRatio >= 1.15 -> ValGreen
                                    profile.kdRatio >= 0.95 -> ValCyan
                                    else -> ValRed
                                },
                                modifier = Modifier.weight(1f),
                            )
                            StatKpiCard(
                                label = "WIN RATE",
                                value = "${"%.1f".format(Locale.US, profile.winRatePercent)}%",
                                accent = if (profile.winRatePercent >= 50.0) ValGreen else ValAmber,
                                modifier = Modifier.weight(1f),
                            )
                            StatKpiCard(
                                label = "HEADSHOT",
                                value = "${"%.1f".format(Locale.US, profile.hsPercent)}%",
                                accent = ValGold,
                                modifier = Modifier.weight(1f),
                            )
                        }

                        Row(
                            modifier = Modifier.fillMaxWidth(),
                            horizontalArrangement = Arrangement.spacedBy(8.dp),
                        ) {
                            StatKpiCard(
                                label = "AVG ACS",
                                value = "${profile.avgAcs.toInt()}",
                                accent = ValPurple,
                                modifier = Modifier.weight(1f),
                            )
                            StatKpiCard(
                                label = "AVG ADR",
                                value = "${profile.avgAdr.toInt()}",
                                accent = ValCyan,
                                modifier = Modifier.weight(1f),
                            )
                            StatKpiCard(
                                label = "AVG K/D/A",
                                value = "${"%.1f".format(Locale.US, profile.avgKills)}/${"%.1f".format(Locale.US, profile.avgDeaths)}/${"%.1f".format(Locale.US, profile.avgAssists)}",
                                accent = TextPrimary,
                                modifier = Modifier.weight(1f),
                            )
                        }
                    }
                }
            }

            // 3. Competitive RR Sparkline
            if (profile.rrTrend.isNotEmpty()) {
                item {
                    RrSparklineCard(updates = profile.rrTrend)
                }
            }

            // 4. Top Agents
            if (profile.agentStats.isNotEmpty()) {
                item {
                    Text(
                        text = "TOP AGENTS (RECENT)",
                        fontWeight = FontWeight.Black,
                        fontSize = 12.sp,
                        letterSpacing = 1.sp,
                        color = TextSecondary,
                    )
                }
                item {
                    LazyRow(horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                        items(profile.agentStats, key = { it.agentUuid }) { stat ->
                            ProfileAgentStatCard(stat)
                        }
                    }
                }
            }

            // 5. Recent Matches (Tappable for Scoreboard & Replay)
            item {
                Text(
                    text = "RECENT MATCHES",
                    fontWeight = FontWeight.Black,
                    fontSize = 12.sp,
                    letterSpacing = 1.sp,
                    color = TextSecondary,
                )
            }

            if (profile.recentMatches.isEmpty()) {
                item {
                    Surface(
                        color = SurfaceCard,
                        shape = RoundedCornerShape(10.dp),
                        modifier = Modifier
                            .fillMaxWidth()
                            .border(1.dp, SurfaceStroke, RoundedCornerShape(10.dp)),
                    ) {
                        Text(
                            text = "No recent completed matches found for this player on the current shard.",
                            color = TextSecondary,
                            fontSize = 12.5.sp,
                            modifier = Modifier.padding(16.dp),
                        )
                    }
                }
            } else {
                itemsIndexed(profile.recentMatches, key = { idx, item -> "pm-$idx-${item.matchId}" }) { _, item ->
                    MatchHistoryRowCard(
                        match = item,
                        onClick = { onSelectMatch(item.matchId, profile.puuid) },
                    )
                }
            }
        }
    }
}
}

@Composable
private fun StatKpiCard(
    label: String,
    value: String,
    accent: Color,
    modifier: Modifier = Modifier,
) {
    Surface(
        color = SurfaceCard,
        shape = RoundedCornerShape(10.dp),
        modifier = modifier.border(1.dp, SurfaceStroke, RoundedCornerShape(10.dp)),
    ) {
        Column(
            modifier = Modifier.padding(horizontal = 10.dp, vertical = 10.dp),
            horizontalAlignment = Alignment.Start,
        ) {
            Text(
                text = label,
                color = TextMuted,
                fontSize = 9.5.sp,
                fontWeight = FontWeight.Bold,
                letterSpacing = 0.6.sp,
                maxLines = 1,
            )
            Spacer(modifier = Modifier.height(4.dp))
            Text(
                text = value,
                color = accent,
                fontSize = if (value.length > 9) 12.5.sp else 15.sp,
                fontWeight = FontWeight.Black,
                maxLines = 1,
                softWrap = false,
                overflow = TextOverflow.Ellipsis,
            )
        }
    }
}

@Composable
private fun ProfileAgentStatCard(stat: AgentPerformanceStat) {
    Surface(
        color = SurfaceCard,
        shape = RoundedCornerShape(10.dp),
        modifier = Modifier
            .width(166.dp)
            .border(1.dp, SurfaceStroke, RoundedCornerShape(10.dp)),
    ) {
        Column(modifier = Modifier.padding(12.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                if (stat.agentIcon != null) {
                    AsyncImage(
                        model = stat.agentIcon,
                        contentDescription = stat.agentName,
                        modifier = Modifier
                            .size(34.dp)
                            .clip(RoundedCornerShape(8.dp))
                            .background(SurfaceElevated),
                    )
                    Spacer(modifier = Modifier.width(8.dp))
                }
                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        text = stat.agentName,
                        fontWeight = FontWeight.Bold,
                        fontSize = 13.sp,
                        color = TextPrimary,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                    Text(
                        text = "${stat.matchesPlayed}G • ${stat.winRatePercent.toInt()}% WR",
                        color = if (stat.winRatePercent >= 50.0) ValGreen else ValAmber,
                        fontSize = 11.sp,
                        fontWeight = FontWeight.SemiBold,
                    )
                }
            }
            Spacer(modifier = Modifier.height(8.dp))
            Text(
                text = "${stat.avgAcs.toInt()} ACS • ${"%.2f".format(Locale.US, stat.kdRatio)} K/D",
                color = ValCyan,
                fontSize = 11.5.sp,
                fontWeight = FontWeight.Bold,
            )
            Text(
                text = "${"%.1f".format(Locale.US, stat.hsPercent)}% HS",
                color = TextSecondary,
                fontSize = 10.5.sp,
            )
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun PlayerSearchBottomSheet(
    query: String,
    knownPlayers: List<KnownPlayerSummary>,
    liveState: LiveState,
    isLookingUpRiotId: Boolean,
    lookupError: String?,
    hasLookupApiKey: Boolean,
    onQueryChange: (String) -> Unit,
    onLookupRiotId: (String) -> Unit,
    onSaveLookupApiKey: (String) -> Unit,
    onSelectPlayer: (puuid: String, displayName: String) -> Unit,
    onDismiss: () -> Unit,
) {
    val sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)
    var showApiKeyEditor by remember { mutableStateOf(false) }
    var apiKeyDraft by remember { mutableStateOf("") }

    val trimmedQuery = query.trim()
    val looksLikeFullRiotId = trimmedQuery.contains('#') &&
        trimmedQuery.substringBeforeLast('#').isNotBlank() &&
        trimmedQuery.substringAfterLast('#').isNotBlank()

    val lobbyPlayers: List<Pair<String, String>> = when (liveState) {
        is LiveState.CoreGame -> (liveState.snapshot.myTeam + liveState.snapshot.enemyTeam)
            .map { it.puuid to it.displayName }
        is LiveState.PreGame -> liveState.snapshot.allies.map { it.puuid to it.displayName }
        is LiveState.InParty -> liveState.snapshot.members.map { it.puuid to it.displayName }
        is LiveState.Idle -> emptyList()
    }
        .filter { it.first.isNotBlank() }
        .distinctBy { it.first.lowercase() }

    ModalBottomSheet(
        onDismissRequest = onDismiss,
        sheetState = sheetState,
        containerColor = SurfaceCard,
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 16.dp)
                .padding(bottom = 24.dp),
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        text = "PLAYER SEARCH",
                        color = TextPrimary,
                        fontWeight = FontWeight.Black,
                        fontSize = 16.sp,
                        letterSpacing = 1.sp,
                    )
                }
                IconButton(onClick = { showApiKeyEditor = !showApiKeyEditor }) {
                    Icon(
                        imageVector = Icons.Default.Key,
                        contentDescription = "Global Lookup API Key",
                        tint = if (hasLookupApiKey) ValGreen else TextSecondary,
                    )
                }
                IconButton(onClick = onDismiss) {
                    Icon(Icons.Default.Close, contentDescription = "Close", tint = TextSecondary)
                }
            }

            Spacer(modifier = Modifier.height(10.dp))

            OutlinedTextField(
                value = query,
                onValueChange = onQueryChange,
                placeholder = { Text("Enter Riot ID (e.g. TenZ#SEN)...", fontSize = 13.sp) },
                leadingIcon = { Icon(Icons.Default.Search, contentDescription = null, tint = ValCyan) },
                singleLine = true,
                keyboardOptions = KeyboardOptions(imeAction = ImeAction.Search),
                keyboardActions = KeyboardActions(
                    onSearch = {
                        if (looksLikeFullRiotId && !isLookingUpRiotId) {
                            onLookupRiotId(trimmedQuery)
                        }
                    },
                ),
                modifier = Modifier.fillMaxWidth(),
            )

            if (looksLikeFullRiotId) {
                Spacer(modifier = Modifier.height(8.dp))
                Button(
                    enabled = !isLookingUpRiotId,
                    onClick = { onLookupRiotId(trimmedQuery) },
                    modifier = Modifier.fillMaxWidth(),
                    colors = ButtonDefaults.buttonColors(
                        containerColor = ValCyan,
                        contentColor = ObsidianBg,
                    ),
                ) {
                    if (isLookingUpRiotId) {
                        CircularProgressIndicator(
                            modifier = Modifier.size(16.dp),
                            color = ObsidianBg,
                            strokeWidth = 2.dp,
                        )
                        Spacer(modifier = Modifier.width(8.dp))
                        Text("Searching $trimmedQuery...", fontWeight = FontWeight.Black)
                    } else {
                        Icon(Icons.Default.Public, contentDescription = null, modifier = Modifier.size(16.dp))
                        Spacer(modifier = Modifier.width(8.dp))
                        Text("Search \"$trimmedQuery\" Globally", fontWeight = FontWeight.Black)
                    }
                }
            }

            if (lookupError != null) {
                Spacer(modifier = Modifier.height(8.dp))
                Surface(
                    color = ValRed.copy(alpha = 0.14f),
                    shape = RoundedCornerShape(8.dp),
                    modifier = Modifier
                        .fillMaxWidth()
                        .border(1.dp, ValRed.copy(alpha = 0.4f), RoundedCornerShape(8.dp)),
                ) {
                    Column(modifier = Modifier.padding(10.dp)) {
                        Text(
                            text = lookupError,
                            color = ValRed,
                            fontSize = 11.5.sp,
                            fontWeight = FontWeight.SemiBold,
                        )
                        if (lookupError.contains("API key", ignoreCase = true) && !showApiKeyEditor) {
                            Spacer(modifier = Modifier.height(4.dp))
                            Text(
                                text = "Tap here to add a free HenrikDev / Riot API key →",
                                color = ValCyan,
                                fontSize = 11.5.sp,
                                fontWeight = FontWeight.Bold,
                                modifier = Modifier.clickable { showApiKeyEditor = true },
                            )
                        }
                    }
                }
            }

            AnimatedVisibility(visible = showApiKeyEditor) {
                Surface(
                    color = SurfaceElevated,
                    shape = RoundedCornerShape(10.dp),
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(top = 8.dp)
                        .border(1.dp, SurfaceStroke, RoundedCornerShape(10.dp)),
                ) {
                    Column(modifier = Modifier.padding(12.dp)) {
                        Row(
                            modifier = Modifier.fillMaxWidth(),
                            horizontalArrangement = Arrangement.SpaceBetween,
                            verticalAlignment = Alignment.CenterVertically,
                        ) {
                            Text(
                                text = "GLOBAL LOOKUP API KEY",
                                color = TextPrimary,
                                fontSize = 11.sp,
                                fontWeight = FontWeight.Black,
                                letterSpacing = 0.8.sp,
                            )
                            if (hasLookupApiKey) {
                                TacticalBadge("KEY ACTIVE", ValGreen)
                            }
                        }
                        Spacer(modifier = Modifier.height(4.dp))
                        Text(
                            text = "Optional: paste a free HenrikDev key (api.henrikdev.xyz/dashboard) or Riot Dev key (RGAPI-...) to look up any player you haven't played with yet.",
                            color = TextSecondary,
                            fontSize = 11.sp,
                        )
                        Spacer(modifier = Modifier.height(8.dp))
                        OutlinedTextField(
                            value = apiKeyDraft,
                            onValueChange = { apiKeyDraft = it },
                            placeholder = { Text("HDEV-... or RGAPI-...", fontSize = 12.sp) },
                            singleLine = true,
                            modifier = Modifier.fillMaxWidth(),
                        )
                        Spacer(modifier = Modifier.height(8.dp))
                        Row(
                            modifier = Modifier.fillMaxWidth(),
                            horizontalArrangement = Arrangement.spacedBy(8.dp),
                        ) {
                            Button(
                                enabled = apiKeyDraft.isNotBlank(),
                                onClick = {
                                    onSaveLookupApiKey(apiKeyDraft.trim())
                                    apiKeyDraft = ""
                                    showApiKeyEditor = false
                                },
                                colors = ButtonDefaults.buttonColors(containerColor = ValCyan, contentColor = ObsidianBg),
                                modifier = Modifier.weight(1f),
                            ) {
                                Text("Save Key", fontWeight = FontWeight.Bold, fontSize = 12.sp)
                            }
                            if (hasLookupApiKey) {
                                OutlinedButton(
                                    onClick = {
                                        onSaveLookupApiKey("")
                                        apiKeyDraft = ""
                                    },
                                ) {
                                    Text("Clear", color = ValRed, fontSize = 12.sp)
                                }
                            }
                        }
                    }
                }
            }

            Spacer(modifier = Modifier.height(12.dp))

            LazyColumn(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(380.dp),
                verticalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                if (query.isBlank() && lobbyPlayers.isNotEmpty()) {
                    item {
                        Text(
                            text = "CURRENT LOBBY",
                            color = ValCyan,
                            fontSize = 11.sp,
                            fontWeight = FontWeight.Black,
                            letterSpacing = 1.sp,
                        )
                    }
                    items(lobbyPlayers, key = { "lobby-${it.first}" }) { (puuid, name) ->
                        Surface(
                            color = SurfaceElevated,
                            shape = RoundedCornerShape(8.dp),
                            modifier = Modifier
                                .fillMaxWidth()
                                .border(1.dp, ValCyan.copy(alpha = 0.3f), RoundedCornerShape(8.dp))
                                .clickable { onSelectPlayer(puuid, name) },
                        ) {
                            Row(
                                modifier = Modifier.padding(horizontal = 12.dp, vertical = 10.dp),
                                verticalAlignment = Alignment.CenterVertically,
                                horizontalArrangement = Arrangement.SpaceBetween,
                            ) {
                                Text(
                                    text = name,
                                    color = TextPrimary,
                                    fontWeight = FontWeight.Bold,
                                    fontSize = 13.5.sp,
                                    maxLines = 1,
                                    overflow = TextOverflow.Ellipsis,
                                    modifier = Modifier.weight(1f),
                                )
                                TacticalBadge("LIVE", ValCyan)
                            }
                        }
                    }
                }

                item {
                    Text(
                        text = if (query.isBlank()) "RECENTLY SEEN PLAYERS" else "MATCHING PLAYERS (${knownPlayers.size})",
                        color = TextSecondary,
                        fontSize = 11.sp,
                        fontWeight = FontWeight.Black,
                        letterSpacing = 1.sp,
                        modifier = Modifier.padding(top = 4.dp),
                    )
                }

                if (knownPlayers.isEmpty()) {
                    item {
                        Text(
                            text = if (query.isBlank()) {
                                "Play or inspect matches to build your Riot ID lookup list."
                            } else {
                                "No encountered players match \"$query\"."
                            },
                            color = TextMuted,
                            fontSize = 12.5.sp,
                            modifier = Modifier.padding(vertical = 12.dp),
                        )
                    }
                } else {
                    items(knownPlayers, key = { "known-${it.puuid}" }) { p ->
                        val rank = p.currentRank
                        Surface(
                            color = ObsidianBg.copy(alpha = 0.7f),
                            shape = RoundedCornerShape(8.dp),
                            modifier = Modifier
                                .fillMaxWidth()
                                .border(1.dp, SurfaceStroke, RoundedCornerShape(8.dp))
                                .clickable { onSelectPlayer(p.puuid, p.displayName) },
                        ) {
                            Row(
                                modifier = Modifier.padding(horizontal = 12.dp, vertical = 10.dp),
                                verticalAlignment = Alignment.CenterVertically,
                            ) {
                                Text(
                                    text = p.displayName,
                                    color = TextPrimary,
                                    fontWeight = FontWeight.Bold,
                                    fontSize = 13.5.sp,
                                    maxLines = 1,
                                    overflow = TextOverflow.Ellipsis,
                                    modifier = Modifier.weight(1f),
                                )
                                if (rank != null) {
                                    Spacer(modifier = Modifier.width(8.dp))
                                    Text(
                                        text = rank.tierName,
                                        color = parseHexColor(rank.colorHex, ValCyan),
                                        fontSize = 11.sp,
                                        fontWeight = FontWeight.Bold,
                                    )
                                    if (rank.iconUrl != null) {
                                        Spacer(modifier = Modifier.width(6.dp))
                                        AsyncImage(
                                            model = rank.iconUrl,
                                            contentDescription = rank.tierName,
                                            modifier = Modifier.size(22.dp),
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
}
