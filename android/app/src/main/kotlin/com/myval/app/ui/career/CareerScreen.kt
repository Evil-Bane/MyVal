package com.myval.app.ui.career

import androidx.compose.animation.AnimatedContent
import java.util.Locale
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.LocalFireDepartment
import androidx.compose.material.icons.filled.Pause
import androidx.compose.material.icons.filled.PersonAdd
import androidx.compose.material.icons.filled.PersonSearch
import androidx.compose.material.icons.filled.PlayArrow
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material.icons.filled.SkipNext
import androidx.compose.material.icons.filled.SkipPrevious
import androidx.compose.material.icons.filled.Timeline
import androidx.compose.material.icons.filled.Warning
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.FilterChip
import androidx.compose.material3.FilterChipDefaults
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.PathEffect
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil.compose.AsyncImage
import com.myval.app.ui.MyValUiState
import com.myval.app.ui.components.AccountSwitcherChip
import com.myval.app.ui.components.AuthRequiredCard
import com.myval.app.ui.components.GlossyCardBrush
import com.myval.app.ui.components.GlossyElevatedBrush
import com.myval.app.ui.components.LiveNetworkStatusBar
import com.myval.app.ui.components.MyValPullToRefreshBox
import com.myval.app.ui.components.SharpCardShape
import com.myval.app.ui.components.SharpChipShape
import com.myval.app.ui.components.TacticalBadge
import com.myval.app.ui.components.parseHexColor
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
import com.myval.app.ui.theme.ValGold
import com.myval.app.ui.theme.ValGreen
import com.myval.app.ui.theme.ValPurple
import com.myval.app.ui.theme.ValRed
import java.time.Instant
import java.time.LocalDate
import java.time.ZoneId
import kotlinx.coroutines.delay
import uniffi.myval_ffi.CareerSnapshot
import uniffi.myval_ffi.CompetitiveMatchUpdate
import uniffi.myval_ffi.MatchDetailSnapshot
import uniffi.myval_ffi.MatchHistoryItem
import uniffi.myval_ffi.MissionProgress
import uniffi.myval_ffi.ReplayPlayerSnapshot
import uniffi.myval_ffi.RoundKillEvent
import kotlin.math.PI

@Composable
fun CareerScreen(
    uiState: MyValUiState,
    onOpenLogin: () -> Unit,
    onOpenAccounts: () -> Unit,
    onOpenPlayerSearch: () -> Unit,
    onRefreshCareer: () -> Unit,
    onSelectMatch: (String) -> Unit,
) {
    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(ObsidianBg),
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .background(SurfaceCard)
                .padding(horizontal = 16.dp, vertical = 12.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.SpaceBetween,
        ) {
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = "CAREER",
                    fontWeight = FontWeight.Black,
                    fontSize = 18.sp,
                    letterSpacing = 1.sp,
                )
                Spacer(modifier = Modifier.height(4.dp))
                val session = uiState.session
                if (session != null) {
                    AccountSwitcherChip(
                        riotId = session.riotId,
                        shard = session.shard,
                        accountCount = uiState.savedAccounts.size,
                        onClick = onOpenAccounts,
                    )
                }
            }
            if (uiState.session != null) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    IconButton(onClick = onOpenPlayerSearch) {
                        Icon(
                            Icons.Default.PersonSearch,
                            contentDescription = "Search player",
                            tint = ValCyan,
                        )
                    }
                }
            }
        }

        LiveNetworkStatusBar(
            message = uiState.networkStatusMessage,
            isSlow = uiState.isNetworkSlow,
        )

        if (uiState.session == null) {
            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .padding(16.dp),
                contentAlignment = Alignment.TopCenter,
            ) {
                AuthRequiredCard(
                    title = "SIGN IN TO VIEW CAREER & 2D REPLAYS",
                    subtitle = "Connect your Riot Games account to inspect your live RR sparkline, agent win rates, 10-player scoreboards, and interactive 2D minimap kill replays.",
                    onOpenLogin = onOpenLogin,
                    savedAccountCount = uiState.savedAccounts.size,
                    onOpenAccounts = onOpenAccounts,
                )
            }
            return
        }

        val career = uiState.career
        if (career == null) {
            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .padding(24.dp),
                contentAlignment = Alignment.Center,
            ) {
                val err = uiState.careerError
                if (err != null && !uiState.isLoadingCareer) {
                    Column(
                        horizontalAlignment = Alignment.CenterHorizontally,
                        verticalArrangement = Arrangement.spacedBy(12.dp),
                    ) {
                        Text(
                            text = "Failed to load Career data",
                            color = TextPrimary,
                            fontWeight = FontWeight.Bold,
                            fontSize = 15.sp,
                        )
                        Text(
                            text = err,
                            color = TextSecondary,
                            fontSize = 12.sp,
                        )
                        Button(
                            onClick = onRefreshCareer,
                            colors = ButtonDefaults.buttonColors(containerColor = ValRed),
                        ) {
                            Text("Retry", fontWeight = FontWeight.Bold)
                        }
                    }
                } else {
                    CircularProgressIndicator(color = ValRed)
                }
            }
            return
        }

        MyValPullToRefreshBox(
            isRefreshing = uiState.isLoadingCareer,
            onRefresh = onRefreshCareer,
            accent = ValCyan,
            modifier = Modifier
                .fillMaxWidth()
                .weight(1f),
        ) {
            CareerContent(
                career = career,
                onSelectMatch = onSelectMatch,
            )
        }
    }
}

private fun formatHoursMinutes(totalSecs: Long): String {
    if (totalSecs <= 0L) return "0m"
    val hours = totalSecs / 3600L
    val mins = (totalSecs % 3600L) / 60L
    return if (hours > 0L) "${hours}h ${mins}m" else "${mins}m"
}

private fun friendlyQueueName(rawQueue: String): String = when (rawQueue.trim().lowercase()) {
    "competitive" -> "Competitive"
    "unrated" -> "Unrated"
    "swiftplay" -> "Swiftplay"
    "deathmatch" -> "Deathmatch"
    "spikerush" -> "Spike Rush"
    "hurm" -> "TDM"
    "ggteam" -> "Escalation"
    "premier" -> "Premier"
    "custom" -> "Custom"
    else -> rawQueue.replaceFirstChar { it.uppercase() }.ifBlank { "Match" }
}

@Composable
private fun CareerContent(
    career: CareerSnapshot,
    onSelectMatch: (String) -> Unit,
) {
    val rankColor = parseHexColor(career.currentRank.colorHex, ValCyan)
    val peakColor = parseHexColor(career.peakRank.colorHex, ValGold)
    val rawTargetProgress = if (career.currentRank.tier >= 24) {
        (career.currentRank.rr.coerceIn(0, 500)) / 500f
    } else if (career.currentRank.tier >= 3) {
        (career.currentRank.rr.coerceIn(0, 100)) / 100f
    } else {
        0f
    }
    val animatedRrProgress by animateFloatAsState(
        targetValue = rawTargetProgress,
        animationSpec = tween(750),
        label = "rrProgress",
    )

    var selectedQueue by remember(career.puuid) { mutableStateOf("ALL") }
    val availableQueues = remember(career.recentMatches) {
        career.recentMatches
            .map { it.queueId.trim().lowercase() }
            .filter { it.isNotEmpty() }
            .distinct()
    }
    val activeQueue = if (selectedQueue == "ALL" || selectedQueue in availableQueues) selectedQueue else "ALL"
    val filteredMatches = remember(career.recentMatches, activeQueue) {
        if (activeQueue == "ALL") {
            career.recentMatches
        } else {
            career.recentMatches.filter { it.queueId.trim().equals(activeQueue, ignoreCase = true) }
        }
    }

    val playtimeStats = remember(career.recentMatches) {
        val zone = ZoneId.systemDefault()
        val today = LocalDate.now(zone)
        val weekStart = today.minusDays(6)
        var todaySecs = 0L
        var todayGames = 0
        var weekSecs = 0L
        var weekGames = 0
        for (m in career.recentMatches) {
            if (m.startTimeEpochSecs <= 0L) continue
            val d = Instant.ofEpochSecond(m.startTimeEpochSecs).atZone(zone).toLocalDate()
            if (d == today) {
                todaySecs += m.durationSecs
                todayGames += 1
            }
            if (!d.isBefore(weekStart)) {
                weekSecs += m.durationSecs
                weekGames += 1
            }
        }
        listOf(
            "Today" to "${formatHoursMinutes(todaySecs)} ($todayGames ${if (todayGames == 1) "game" else "games"})",
            "This Week" to "${formatHoursMinutes(weekSecs)} ($weekGames ${if (weekGames == 1) "game" else "games"})",
        )
    }

    LazyColumn(
        modifier = Modifier.fillMaxSize(),
        contentPadding = PaddingValues(16.dp),
        verticalArrangement = Arrangement.spacedBy(14.dp),
    ) {
        // 1. Dual Circular Rank & Peak Hero Card
        item {
            Surface(
                color = SurfaceCard,
                shape = RoundedCornerShape(10.dp),
                modifier = Modifier
                    .fillMaxWidth()
                    .border(1.dp, rankColor.copy(alpha = 0.55f), RoundedCornerShape(10.dp)),
            ) {
                Column(
                    modifier = Modifier
                        .background(
                            Brush.horizontalGradient(
                                listOf(rankColor.copy(alpha = 0.14f), peakColor.copy(alpha = 0.08f), SurfaceCard),
                            ),
                        )
                        .padding(16.dp),
                ) {
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.spacedBy(12.dp),
                        verticalAlignment = Alignment.Top,
                    ) {
                        // Left: Current Act Progress Ring
                        Column(
                            modifier = Modifier.weight(1f),
                            horizontalAlignment = Alignment.CenterHorizontally,
                        ) {
                            Text(
                                text = "Current Act",
                                color = TextSecondary,
                                fontSize = 11.sp,
                                fontWeight = FontWeight.SemiBold,
                            )
                            Text(
                                text = career.currentActName.ifBlank { "Competitive" },
                                color = rankColor,
                                fontSize = 12.5.sp,
                                fontWeight = FontWeight.Black,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis,
                            )
                            if (career.currentActEndLabel.isNotBlank()) {
                                Text(
                                    text = career.currentActEndLabel,
                                    color = TextMuted,
                                    fontSize = 9.5.sp,
                                    maxLines = 1,
                                    overflow = TextOverflow.Ellipsis,
                                )
                            } else {
                                Spacer(modifier = Modifier.height(12.dp))
                            }
                            Spacer(modifier = Modifier.height(10.dp))
                            Box(
                                contentAlignment = Alignment.Center,
                                modifier = Modifier.size(90.dp),
                            ) {
                                Canvas(modifier = Modifier.fillMaxSize()) {
                                    val strokePx = 6.dp.toPx()
                                    drawArc(
                                        color = SurfaceElevated,
                                        startAngle = 0f,
                                        sweepAngle = 360f,
                                        useCenter = false,
                                        style = Stroke(width = strokePx, cap = StrokeCap.Round),
                                    )
                                    if (animatedRrProgress > 0f) {
                                        drawArc(
                                            color = rankColor,
                                            startAngle = -90f,
                                            sweepAngle = 360f * animatedRrProgress,
                                            useCenter = false,
                                            style = Stroke(width = strokePx, cap = StrokeCap.Round),
                                        )
                                    }
                                }
                                if (career.currentRank.iconUrl != null) {
                                    AsyncImage(
                                        model = career.currentRank.iconUrl,
                                        contentDescription = career.currentRank.tierName,
                                        modifier = Modifier.size(48.dp),
                                    )
                                }
                            }
                            Spacer(modifier = Modifier.height(8.dp))
                            Text(
                                text = career.currentRank.tierName,
                                fontWeight = FontWeight.Black,
                                fontSize = 14.sp,
                                color = rankColor,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis,
                                textAlign = TextAlign.Center,
                            )
                            val rrLabel = when {
                                career.currentRank.tier >= 24 || career.currentRank.rr > 100 -> "${career.currentRank.rr} RR"
                                career.currentRank.tier >= 3 -> "${career.currentRank.rr} / 100 RR"
                                else -> "Placement"
                            }
                            Text(
                                text = rrLabel,
                                color = TextSecondary,
                                fontSize = 11.sp,
                                fontWeight = FontWeight.SemiBold,
                            )
                        }

                        // Right: Peak Rank Full Ring
                        Column(
                            modifier = Modifier.weight(1f),
                            horizontalAlignment = Alignment.CenterHorizontally,
                        ) {
                            Text(
                                text = "Your Peak Rank",
                                color = TextSecondary,
                                fontSize = 11.sp,
                                fontWeight = FontWeight.SemiBold,
                            )
                            Text(
                                text = career.peakRankActName.ifBlank { "All-Time Peak" },
                                color = peakColor,
                                fontSize = 12.5.sp,
                                fontWeight = FontWeight.Black,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis,
                            )
                            Text(
                                text = "Prev: ${career.previousActRank.tierName}",
                                color = TextMuted,
                                fontSize = 9.5.sp,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis,
                            )
                            Spacer(modifier = Modifier.height(10.dp))
                            Box(
                                contentAlignment = Alignment.Center,
                                modifier = Modifier.size(90.dp),
                            ) {
                                Canvas(modifier = Modifier.fillMaxSize()) {
                                    val strokePx = 6.dp.toPx()
                                    drawArc(
                                        color = peakColor.copy(alpha = 0.88f),
                                        startAngle = -90f,
                                        sweepAngle = 360f,
                                        useCenter = false,
                                        style = Stroke(width = strokePx, cap = StrokeCap.Round),
                                    )
                                }
                                if (career.peakRank.iconUrl != null) {
                                    AsyncImage(
                                        model = career.peakRank.iconUrl,
                                        contentDescription = career.peakRank.tierName,
                                        modifier = Modifier.size(48.dp),
                                    )
                                }
                            }
                            Spacer(modifier = Modifier.height(8.dp))
                            Text(
                                text = career.peakRank.tierName,
                                fontWeight = FontWeight.Black,
                                fontSize = 14.sp,
                                color = peakColor,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis,
                                textAlign = TextAlign.Center,
                            )
                            val lastDelta = career.rrTrend.firstOrNull()?.rrDelta
                            if (lastDelta != null) {
                                val sign = if (lastDelta >= 0) "+" else ""
                                Text(
                                    text = "Last: $sign$lastDelta RR",
                                    color = if (lastDelta >= 0) ValGreen else ValRed,
                                    fontSize = 11.sp,
                                    fontWeight = FontWeight.Bold,
                                )
                            } else {
                                Text(
                                    text = career.currentStreakLabel,
                                    color = TextMuted,
                                    fontSize = 11.sp,
                                )
                            }
                        }
                    }

                    Spacer(modifier = Modifier.height(12.dp))
                    HorizontalDivider(thickness = 0.6.dp, color = SurfaceStroke.copy(alpha = 0.7f))
                    Spacer(modifier = Modifier.height(12.dp))

                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween,
                    ) {
                        val sign = if (career.netRrLast10 >= 0) "+" else ""
                        CareerMetricPill(
                            label = "NET RR (10G)",
                            value = "$sign${career.netRrLast10} RR",
                            color = if (career.netRrLast10 >= 0) ValGreen else ValRed,
                        )
                        CareerMetricPill(
                            label = "WIN RATE",
                            value = "${career.winRateLast10}%",
                            color = ValCyan,
                        )
                        CareerMetricPill(
                            label = "BATTLEPASS",
                            value = "Tier ${career.contract.currentLevel}/${career.contract.maxLevel}",
                            color = ValGold,
                        )
                    }
                }
            }
        }

        // Active Riot Account Penalties / Restrictions Card
        if (career.penalties.isNotEmpty()) {
            item {
                Surface(
                    color = ValAmber.copy(alpha = 0.12f),
                    shape = RoundedCornerShape(10.dp),
                    modifier = Modifier
                        .fillMaxWidth()
                        .border(1.dp, ValAmber.copy(alpha = 0.55f), RoundedCornerShape(10.dp)),
                ) {
                    Column(
                        modifier = Modifier.padding(14.dp),
                        verticalArrangement = Arrangement.spacedBy(8.dp),
                    ) {
                        Row(verticalAlignment = Alignment.CenterVertically) {
                            Icon(
                                imageVector = Icons.Default.Warning,
                                contentDescription = null,
                                tint = ValAmber,
                                modifier = Modifier.size(18.dp),
                            )
                            Spacer(modifier = Modifier.width(8.dp))
                            Text(
                                text = "ACTIVE PENALTIES (${career.penalties.size})",
                                color = ValAmber,
                                fontWeight = FontWeight.Black,
                                fontSize = 12.sp,
                                letterSpacing = 0.8.sp,
                            )
                        }
                        career.penalties.forEach { p ->
                            Column(
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .background(ObsidianBg.copy(alpha = 0.5f), RoundedCornerShape(8.dp))
                                    .padding(10.dp),
                            ) {
                                Text(
                                    text = "Reason: ${p.reason}",
                                    color = TextPrimary,
                                    fontWeight = FontWeight.Bold,
                                    fontSize = 12.5.sp,
                                )
                                p.expiresAt?.let { exp ->
                                    Text(
                                        text = "Expires: $exp",
                                        color = TextSecondary,
                                        fontSize = 11.sp,
                                    )
                                }
                                p.gamesRemaining?.let { games ->
                                    Text(
                                        text = "Games Remaining: $games",
                                        color = ValRed,
                                        fontSize = 11.sp,
                                        fontWeight = FontWeight.Bold,
                                    )
                                }
                            }
                        }
                    }
                }
            }
        }

        // Playtime Summary Card (Today & This Week)
        if (career.recentMatches.isNotEmpty()) {
            item {
                Surface(
                    color = SurfaceCard,
                    shape = RoundedCornerShape(10.dp),
                    modifier = Modifier
                        .fillMaxWidth()
                        .border(1.dp, SurfaceStroke, RoundedCornerShape(10.dp)),
                ) {
                    Column(
                        modifier = Modifier.padding(14.dp),
                        verticalArrangement = Arrangement.spacedBy(8.dp),
                    ) {
                        Row(
                            modifier = Modifier.fillMaxWidth(),
                            horizontalArrangement = Arrangement.SpaceBetween,
                            verticalAlignment = Alignment.CenterVertically,
                        ) {
                            Text(
                                text = "PLAYTIME",
                                color = TextSecondary,
                                fontWeight = FontWeight.Black,
                                fontSize = 12.sp,
                                letterSpacing = 1.sp,
                            )
                            Text(
                                text = "Last ${career.recentMatches.size} matches",
                                color = TextMuted,
                                fontSize = 10.5.sp,
                            )
                        }
                        Row(
                            modifier = Modifier.fillMaxWidth(),
                            horizontalArrangement = Arrangement.spacedBy(10.dp),
                        ) {
                            playtimeStats.forEach { (label, valText) ->
                                Column(
                                    modifier = Modifier
                                        .weight(1f)
                                        .background(SurfaceElevated, RoundedCornerShape(8.dp))
                                        .padding(10.dp),
                                ) {
                                    Text(
                                        text = label,
                                        color = TextMuted,
                                        fontSize = 10.5.sp,
                                        fontWeight = FontWeight.Bold,
                                    )
                                    Spacer(modifier = Modifier.height(2.dp))
                                    Text(
                                        text = valText,
                                        color = TextPrimary,
                                        fontSize = 12.sp,
                                        fontWeight = FontWeight.Bold,
                                        maxLines = 1,
                                        overflow = TextOverflow.Ellipsis,
                                    )
                                }
                            }
                        }
                    }
                }
            }
        }

        // Active Missions Card
        if (career.missions.isNotEmpty()) {
            item {
                ActiveMissionsCard(missions = career.missions)
            }
        }

        // 2. RR Sparkline Canvas Card
        if (career.rrTrend.isNotEmpty()) {
            item {
                RrSparklineCard(updates = career.rrTrend)
            }
        }

        // 3. Agent Performance Breakdown
        if (career.agentStats.isNotEmpty()) {
            item {
                Text(
                    text = "AGENT PERFORMANCE",
                    fontWeight = FontWeight.Black,
                    fontSize = 13.sp,
                    letterSpacing = 1.sp,
                    color = TextSecondary,
                )
            }
            item {
                LazyRow(horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                    items(career.agentStats, key = { it.agentName }) { ag ->
                        Surface(
                            color = SurfaceCard,
                            shape = RoundedCornerShape(10.dp),
                            modifier = Modifier
                                .width(224.dp)
                                .border(1.dp, SurfaceStroke, RoundedCornerShape(10.dp)),
                        ) {
                            Row(
                                modifier = Modifier.padding(12.dp),
                                verticalAlignment = Alignment.CenterVertically,
                            ) {
                                if (ag.agentIcon != null) {
                                    AsyncImage(
                                        model = ag.agentIcon,
                                        contentDescription = ag.agentName,
                                        modifier = Modifier
                                            .size(44.dp)
                                            .clip(RoundedCornerShape(8.dp))
                                            .background(SurfaceElevated),
                                    )
                                    Spacer(modifier = Modifier.width(10.dp))
                                }
                                Column(modifier = Modifier.weight(1f)) {
                                    Row(
                                        modifier = Modifier.fillMaxWidth(),
                                        horizontalArrangement = Arrangement.SpaceBetween,
                                        verticalAlignment = Alignment.CenterVertically,
                                    ) {
                                        Text(
                                            text = ag.agentName,
                                            fontWeight = FontWeight.Bold,
                                            fontSize = 14.sp,
                                            color = TextPrimary,
                                            maxLines = 1,
                                            overflow = TextOverflow.Ellipsis,
                                        )
                                        Text(
                                            text = "${ag.matchesPlayed}G",
                                            color = TextMuted,
                                            fontSize = 10.5.sp,
                                            fontWeight = FontWeight.Bold,
                                        )
                                    }
                                    Text(
                                        text = "${"%.1f".format(Locale.US, ag.winRatePercent)}% WR • ${"%.2f".format(Locale.US, ag.kdRatio)} KD",
                                        color = ValCyan,
                                        fontSize = 11.5.sp,
                                        fontWeight = FontWeight.Bold,
                                        maxLines = 1,
                                        softWrap = false,
                                        overflow = TextOverflow.Ellipsis,
                                    )
                                    Text(
                                        text = "${ag.avgAcs.toInt()} ACS • ${"%.1f".format(Locale.US, ag.hsPercent)}% HS",
                                        color = TextMuted,
                                        fontSize = 11.sp,
                                        maxLines = 1,
                                        softWrap = false,
                                        overflow = TextOverflow.Ellipsis,
                                    )
                                }
                            }
                        }
                    }
                }
            }
        }

        // 4. Recent Matches Header + Queue Filter Chips
        item {
            Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                Text(
                    text = "RECENT MATCHES (${filteredMatches.size})",
                    fontWeight = FontWeight.Black,
                    fontSize = 13.sp,
                    letterSpacing = 1.sp,
                    color = TextSecondary,
                )
                if (availableQueues.size > 1) {
                    LazyRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        item(key = "q-all") {
                            FilterChip(
                                selected = activeQueue == "ALL",
                                onClick = { selectedQueue = "ALL" },
                                label = { Text("All", fontSize = 11.5.sp, fontWeight = FontWeight.Bold) },
                                colors = FilterChipDefaults.filterChipColors(
                                    selectedContainerColor = ValCyan.copy(alpha = 0.2f),
                                    selectedLabelColor = ValCyan,
                                ),
                            )
                        }
                        items(availableQueues, key = { "q-$it" }) { q ->
                            FilterChip(
                                selected = activeQueue == q,
                                onClick = { selectedQueue = q },
                                label = { Text(friendlyQueueName(q), fontSize = 11.5.sp, fontWeight = FontWeight.Bold) },
                                colors = FilterChipDefaults.filterChipColors(
                                    selectedContainerColor = ValRed.copy(alpha = 0.2f),
                                    selectedLabelColor = ValRed,
                                ),
                            )
                        }
                    }
                }
            }
        }

        items(filteredMatches, key = { it.matchId }) { item ->
            MatchHistoryRowCard(
                match = item,
                onClick = { onSelectMatch(item.matchId) },
            )
        }
    }
}

@Composable
private fun ActiveMissionsCard(missions: List<MissionProgress>) {
    Surface(
        color = SurfaceCard,
        shape = RoundedCornerShape(10.dp),
        modifier = Modifier
            .fillMaxWidth()
            .border(1.dp, SurfaceStroke, RoundedCornerShape(10.dp)),
    ) {
        Column(
            modifier = Modifier.padding(14.dp),
            verticalArrangement = Arrangement.spacedBy(10.dp),
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(
                    text = "ACTIVE MISSIONS",
                    color = TextSecondary,
                    fontWeight = FontWeight.Black,
                    fontSize = 12.sp,
                    letterSpacing = 1.sp,
                )
                val doneCount = missions.count { it.completed }
                Text(
                    text = "$doneCount/${missions.size} completed",
                    color = if (doneCount == missions.size) ValGreen else TextMuted,
                    fontSize = 10.5.sp,
                    fontWeight = FontWeight.Bold,
                )
            }
            missions.forEach { mission ->
                val target = mission.targetProgress.coerceAtLeast(1)
                val progressFrac = (mission.currentProgress.toFloat() / target.toFloat()).coerceIn(0f, 1f)
                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .background(SurfaceElevated, RoundedCornerShape(8.dp))
                        .padding(10.dp),
                    verticalArrangement = Arrangement.spacedBy(6.dp),
                ) {
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Text(
                            text = mission.title,
                            color = TextPrimary,
                            fontSize = 12.sp,
                            fontWeight = FontWeight.Bold,
                            maxLines = 2,
                            overflow = TextOverflow.Ellipsis,
                            modifier = Modifier.weight(1f).padding(end = 8.dp),
                        )
                        Row(
                            verticalAlignment = Alignment.CenterVertically,
                            horizontalArrangement = Arrangement.spacedBy(6.dp),
                        ) {
                            if (mission.xpGrant > 0) {
                                Text(
                                    text = "+${mission.xpGrant} XP",
                                    color = ValCyan,
                                    fontSize = 10.sp,
                                    fontWeight = FontWeight.Black,
                                    modifier = Modifier
                                        .background(ValCyan.copy(alpha = 0.14f), RoundedCornerShape(4.dp))
                                        .padding(horizontal = 5.dp, vertical = 2.dp),
                                )
                            }
                            if (mission.completed) {
                                Icon(
                                    imageVector = Icons.Default.CheckCircle,
                                    contentDescription = "Completed",
                                    tint = ValGreen,
                                    modifier = Modifier.size(16.dp),
                                )
                            } else {
                                Text(
                                    text = "${mission.currentProgress} / ${mission.targetProgress}",
                                    color = TextSecondary,
                                    fontSize = 11.sp,
                                    fontWeight = FontWeight.Bold,
                                )
                            }
                        }
                    }
                    LinearProgressIndicator(
                        progress = { if (mission.completed) 1f else progressFrac },
                        modifier = Modifier
                            .fillMaxWidth()
                            .height(5.dp)
                            .clip(RoundedCornerShape(8.dp)),
                        color = if (mission.completed) ValGreen else ValCyan,
                        trackColor = ObsidianBg,
                    )
                }
            }
        }
    }
}

@Composable
private fun ReplayLegendDot(label: String, color: Color) {
    Row(verticalAlignment = Alignment.CenterVertically) {
        Box(
            modifier = Modifier
                .size(7.dp)
                .background(color, CircleShape),
        )
        Spacer(modifier = Modifier.width(4.dp))
        Text(
            text = label,
            color = TextMuted,
            fontSize = 9.5.sp,
            fontWeight = FontWeight.Bold,
        )
    }
}

@Composable
private fun CareerMetricPill(label: String, value: String, color: Color) {
    Column {
        Text(label, color = TextMuted, fontSize = 10.sp, fontWeight = FontWeight.Bold)
        Text(value, color = color, fontSize = 15.sp, fontWeight = FontWeight.Black)
    }
}

@Composable
internal fun RrSparklineCard(updates: List<CompetitiveMatchUpdate>) {
    val chronological = remember(updates) { updates.reversed() }
    Surface(
        color = SurfaceCard,
        shape = RoundedCornerShape(10.dp),
        modifier = Modifier
            .fillMaxWidth()
            .border(1.dp, SurfaceStroke, RoundedCornerShape(10.dp)),
    ) {
        Column(modifier = Modifier.padding(14.dp)) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Icon(Icons.Default.Timeline, contentDescription = null, tint = ValCyan, modifier = Modifier.size(16.dp))
                    Spacer(modifier = Modifier.width(6.dp))
                    Text("RR HISTORY", fontWeight = FontWeight.Black, fontSize = 12.sp, letterSpacing = 1.sp)
                }
                Text("${chronological.size} Comp Matches", color = TextMuted, fontSize = 11.sp)
            }

            Spacer(modifier = Modifier.height(10.dp))
            Canvas(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(82.dp),
            ) {
                if (chronological.isEmpty() || size.width <= 24f || size.height <= 20f) return@Canvas
                val minRr = (chronological.minOf { it.rrAfter } - 10).coerceAtLeast(0)
                val rawMaxRr = chronological.maxOf { it.rrAfter }
                val maxRr = if (rawMaxRr > 100) rawMaxRr + 15 else (rawMaxRr + 10).coerceAtMost(100)
                val range = (maxRr - minRr).coerceAtLeast(10).toFloat()

                val points = chronological.mapIndexed { idx, u ->
                    val x = if (chronological.size == 1) {
                        size.width / 2f
                    } else {
                        (idx.toFloat() / (chronological.size - 1)) * (size.width - 24f) + 12f
                    }
                    val normY = 1f - ((u.rrAfter - minRr).toFloat() / range)
                    Offset(x, (normY * (size.height - 20f)) + 10f)
                }

                val linePath = Path().apply {
                    points.forEachIndexed { index, pt ->
                        if (index == 0) moveTo(pt.x, pt.y) else lineTo(pt.x, pt.y)
                    }
                }
                val fillPath = Path().apply {
                    addPath(linePath)
                    lineTo(points.last().x, size.height)
                    lineTo(points.first().x, size.height)
                    close()
                }

                drawPath(
                    path = fillPath,
                    brush = Brush.verticalGradient(
                        listOf(ValCyan.copy(alpha = 0.28f), Color.Transparent),
                    ),
                )
                drawPath(
                    path = linePath,
                    color = ValCyan,
                    style = Stroke(width = 3.dp.toPx(), cap = StrokeCap.Round),
                )
                points.forEachIndexed { idx, pt ->
                    val won = chronological[idx].won
                    drawCircle(
                        color = if (won) ValGreen else ValRed,
                        radius = 4.5.dp.toPx(),
                        center = pt,
                    )
                }
            }

            Spacer(modifier = Modifier.height(8.dp))
            LazyRow(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                itemsIndexed(updates, key = { idx, u -> "rr-$idx-${u.matchId}" }) { _, u ->
                    val sign = if (u.rrDelta >= 0) "+" else ""
                    TacticalBadge(
                        text = "${u.mapName}: $sign${u.rrDelta} RR",
                        accent = if (u.rrDelta >= 0) ValGreen else ValRed,
                    )
                }
            }
        }
    }
}

@Composable
internal fun MatchHistoryRowCard(
    match: MatchHistoryItem,
    onClick: () -> Unit,
) {
    val accent = if (match.won) ValGreen else ValRed
    Surface(
        color = Color.Transparent,
        shape = SharpCardShape,
        modifier = Modifier
            .fillMaxWidth()
            .background(GlossyCardBrush, SharpCardShape)
            .border(1.dp, accent.copy(alpha = 0.45f), SharpCardShape)
            .tactileClickable(pressedScale = 0.97f, onClick = onClick),
    ) {
        Box {
            if (match.mapSplash != null) {
                AsyncImage(
                    model = match.mapSplash,
                    contentDescription = match.mapName,
                    contentScale = ContentScale.Crop,
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(76.dp),
                )
            }
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .background(
                        Brush.horizontalGradient(
                            listOf(
                                ObsidianBg.copy(alpha = 0.95f),
                                ObsidianBg.copy(alpha = 0.80f),
                            ),
                        ),
                    )
                    .padding(10.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Box(
                    modifier = Modifier
                        .width(3.dp)
                        .height(48.dp)
                        .background(accent),
                )
                Spacer(modifier = Modifier.width(9.dp))
                if (match.myAgentIcon != null) {
                    AsyncImage(
                        model = match.myAgentIcon,
                        contentDescription = match.myAgentName,
                        modifier = Modifier
                            .size(44.dp)
                            .clip(SharpChipShape)
                            .background(SurfaceElevated)
                            .border(1.dp, SurfaceStroke, SharpChipShape),
                    )
                    Spacer(modifier = Modifier.width(10.dp))
                }
                Column(modifier = Modifier.weight(1f)) {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Text(
                            text = "${match.mapName.uppercase()} • ${match.myTeamRounds}-${match.enemyTeamRounds}",
                            fontWeight = FontWeight.Black,
                            fontSize = 14.sp,
                            color = TextPrimary,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                            modifier = Modifier.weight(1f, fill = false),
                        )
                        Spacer(modifier = Modifier.width(8.dp))
                        match.rrDelta?.let { delta ->
                            val sign = if (delta >= 0) "+" else ""
                            TacticalBadge("$sign$delta RR", accent)
                        }
                    }
                    Spacer(modifier = Modifier.height(3.dp))
                    Text(
                        text = "${match.myKills}/${match.myDeaths}/${match.myAssists} KDA • ${match.myAcs} ACS • ${match.myHsPercent.toInt()}% HS • ${match.myAdr} ADR",
                        color = TextSecondary,
                        fontSize = 10.5.sp,
                        fontWeight = FontWeight.SemiBold,
                        maxLines = 1,
                        softWrap = false,
                        overflow = TextOverflow.Ellipsis,
                    )
                }
                Icon(
                    imageVector = Icons.Default.PlayArrow,
                    contentDescription = "Open 2D Replay",
                    tint = ValCyan,
                    modifier = Modifier.size(20.dp),
                )
            }
        }
    }
}

private enum class HeatmapMetric(val label: String) {
    KILLS("Kills"),
    DEATHS("Deaths"),
    FIRST_BLOODS("First Bloods"),
    PLANTS("Plants"),
}

private enum class HeatmapHalf(val label: String) {
    ALL("All Rounds"),
    FIRST_HALF("1st Half"),
    SECOND_HALF("2nd Half"),
}

private enum class ReplaySpeed(val label: String, val stepDelayMs: Long, val animMs: Int) {
    SLOW("0.7x", 2_400L, 420),
    NORMAL("1x", 1_400L, 280),
    FAST("2x", 750L, 180),
}

private sealed class RoundTimelineEvent(
    val timeSecs: Double,
    val shortTag: String,
    val summary: String,
    val playerPositions: List<ReplayPlayerSnapshot>,
) {
    class Kill(
        val killIndex: Int,
        val kill: RoundKillEvent,
        val isFirstBlood: Boolean,
        val isTrade: Boolean,
    ) : RoundTimelineEvent(
        timeSecs = kill.timeInRoundSecs,
        shortTag = if (killIndex == 0) "FB" else "K${killIndex + 1}",
        summary = "${kill.killerName} [${kill.weaponName}] → ${kill.victimName}",
        playerPositions = kill.playerPositions,
    )

    class Plant(
        timeSecs: Double,
        val site: String?,
        val planterName: String?,
        val normX: Double,
        val normY: Double,
        positions: List<ReplayPlayerSnapshot>,
    ) : RoundTimelineEvent(
        timeSecs = timeSecs,
        shortTag = if (!site.isNullOrBlank()) "PLANT $site" else "PLANT",
        summary = "Spike planted${site?.let { " ($it)" } ?: ""}${planterName?.let { " by $it" } ?: ""}",
        playerPositions = positions,
    )

    class Defuse(
        timeSecs: Double,
        val defuserName: String?,
        val normX: Double,
        val normY: Double,
        positions: List<ReplayPlayerSnapshot>,
    ) : RoundTimelineEvent(
        timeSecs = timeSecs,
        shortTag = "DEFUSE",
        summary = "Spike defused${defuserName?.let { " by $it" } ?: ""}",
        playerPositions = positions,
    )
}

@Composable
internal fun MatchReplayAndScoreboardView(
    detail: MatchDetailSnapshot,
    activeSessionPuuid: String? = null,
    onInspectPlayer: (puuid: String, displayName: String) -> Unit = { _, _ -> },
    onClose: () -> Unit,
) {
    val viewedPuuid = remember(detail.scoreboard, activeSessionPuuid) {
        detail.scoreboard.firstOrNull { it.isMe }?.puuid
            ?: activeSessionPuuid
            ?: detail.scoreboard.firstOrNull { it.teamId.equals(detail.myTeamId, ignoreCase = true) }?.puuid
    }

    var selectedRoundIdx by remember(detail.matchId) { mutableIntStateOf(0) }
    var heatmapMode by remember(detail.matchId) { mutableStateOf(false) }
    var heatmapMetric by remember(detail.matchId) { mutableStateOf(HeatmapMetric.KILLS) }
    var heatmapHalf by remember(detail.matchId) { mutableStateOf(HeatmapHalf.ALL) }
    var heatmapScope by remember(detail.matchId, viewedPuuid) {
        mutableStateOf(viewedPuuid?.lowercase() ?: "ALL")
    }
    var replaySpeed by remember(detail.matchId) { mutableStateOf(ReplaySpeed.NORMAL) }
    var selectedEventIdx by remember(detail.matchId, selectedRoundIdx) { mutableIntStateOf(0) }
    var isAutoPlaying by remember(detail.matchId, selectedRoundIdx) { mutableStateOf(false) }

    val activeRound = detail.rounds.getOrNull(selectedRoundIdx)
    val timelineEvents = remember(activeRound) {
        if (activeRound == null) {
            emptyList()
        } else {
            buildList {
                activeRound.kills.forEachIndexed { idx, k ->
                    val isFirstBlood = idx == 0
                    val isTrade = (0 until idx).any { prevIdx ->
                        val prev = activeRound.kills[prevIdx]
                        val dt = k.timeInRoundSecs - prev.timeInRoundSecs
                        dt in 0.0..5.0 &&
                            prev.killerPuuid.equals(k.victimPuuid, ignoreCase = true) &&
                            !prev.killerTeam.equals(k.killerTeam, ignoreCase = true)
                    }
                    add(RoundTimelineEvent.Kill(idx, k, isFirstBlood = isFirstBlood, isTrade = isTrade))
                }
                val pt = activeRound.plantTimeSecs
                val px = activeRound.plantNormX
                val py = activeRound.plantNormY
                if (pt != null && px != null && py != null) {
                    add(
                        RoundTimelineEvent.Plant(
                            timeSecs = pt,
                            site = activeRound.plantSite,
                            planterName = activeRound.planterName,
                            normX = px,
                            normY = py,
                            positions = activeRound.plantPlayerPositions,
                        ),
                    )
                }
                val dt = activeRound.defuseTimeSecs
                val dx = activeRound.defuseNormX
                val dy = activeRound.defuseNormY
                if (dt != null && dx != null && dy != null) {
                    add(
                        RoundTimelineEvent.Defuse(
                            timeSecs = dt,
                            defuserName = activeRound.defuserName,
                            normX = dx,
                            normY = dy,
                            positions = activeRound.defusePlayerPositions,
                        ),
                    )
                }
            }.sortedBy { it.timeSecs }
        }
    }

    val safeEventIdx = if (timelineEvents.isEmpty()) 0 else selectedEventIdx.coerceIn(0, timelineEvents.lastIndex)
    val activeEvent = timelineEvents.getOrNull(safeEventIdx)

    LaunchedEffect(isAutoPlaying, safeEventIdx, timelineEvents.size, heatmapMode, replaySpeed) {
        if (isAutoPlaying && !heatmapMode && timelineEvents.isNotEmpty()) {
            delay(replaySpeed.stepDelayMs)
            if (safeEventIdx < timelineEvents.lastIndex) {
                selectedEventIdx = safeEventIdx + 1
            } else {
                isAutoPlaying = false
            }
        }
    }

    val enemyDuelRecords = remember(detail.rounds, viewedPuuid) {
        val me = viewedPuuid?.trim()?.lowercase()
        if (me.isNullOrEmpty()) {
            emptyMap<String, Pair<Int, Int>>()
        } else {
            val map = linkedMapOf<String, Pair<Int, Int>>()
            detail.rounds.forEach { round ->
                round.kills.forEach { k ->
                    val killer = k.killerPuuid.trim().lowercase()
                    val victim = k.victimPuuid.trim().lowercase()
                    if (killer == me && victim.isNotEmpty() && victim != me) {
                        val cur = map[victim] ?: (0 to 0)
                        map[victim] = (cur.first + 1) to cur.second
                    } else if (victim == me && killer.isNotEmpty() && killer != me) {
                        val cur = map[killer] ?: (0 to 0)
                        map[killer] = cur.first to (cur.second + 1)
                    }
                }
            }
            map
        }
    }

    val matchMvpPuuid = remember(detail.scoreboard) { detail.scoreboard.firstOrNull()?.puuid }
    val topTeamId = remember(detail.scoreboard) { detail.scoreboard.firstOrNull()?.teamId }
    val teamMvpPuuid = remember(detail.scoreboard, topTeamId) {
        detail.scoreboard.firstOrNull { it.teamId != topTeamId }?.puuid
    }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(ObsidianBg),
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .background(SurfaceCard)
                .border(0.5.dp, SurfaceStroke)
                .padding(horizontal = 10.dp, vertical = 8.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            IconButton(onClick = onClose) {
                Icon(Icons.Default.Close, contentDescription = "Close", tint = TextPrimary)
            }
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = "${detail.mapName.uppercase()} • ${detail.myTeamScore} - ${detail.enemyTeamScore}",
                    fontWeight = FontWeight.Black,
                    fontSize = 16.sp,
                    color = if (detail.won) ValGreen else ValRed,
                )
            }
            FilterChip(
                selected = heatmapMode,
                onClick = {
                    heatmapMode = !heatmapMode
                    isAutoPlaying = false
                },
                shape = SharpChipShape,
                leadingIcon = {
                    Icon(
                        Icons.Default.LocalFireDepartment,
                        contentDescription = null,
                        tint = if (heatmapMode) ValRed else ValCyan,
                        modifier = Modifier.size(15.dp),
                    )
                },
                label = {
                    Text(
                        if (heatmapMode) "Heatmap" else "Replay",
                        fontSize = 11.sp,
                        fontWeight = FontWeight.Bold,
                    )
                },
            )
        }

        LazyColumn(
            modifier = Modifier.fillMaxSize(),
            contentPadding = PaddingValues(12.dp),
            verticalArrangement = Arrangement.spacedBy(10.dp),
        ) {
            // 2D Tactical Minimap Snapshot Card
            item {
                Surface(
                    color = Color.Transparent,
                    shape = SharpCardShape,
                    modifier = Modifier
                        .fillMaxWidth()
                        .background(GlossyCardBrush, SharpCardShape)
                        .border(1.dp, SurfaceStroke, SharpCardShape),
                ) {
                    Column(modifier = Modifier.padding(10.dp)) {
                        // Round selector strip (Replay) OR Precision Heatmap Filter Bars
                        if (!heatmapMode && detail.rounds.isNotEmpty()) {
                            Row(
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .horizontalScroll(rememberScrollState()),
                                horizontalArrangement = Arrangement.spacedBy(5.dp),
                            ) {
                                detail.rounds.forEachIndexed { idx, r ->
                                    val wonRound = r.winningTeam.equals(detail.myTeamId, ignoreCase = true)
                                    val selected = selectedRoundIdx == idx
                                    val accent = if (wonRound) ValGreen else ValRed
                                    Box(
                                        modifier = Modifier
                                            .clip(SharpChipShape)
                                            .background(if (selected) accent.copy(alpha = 0.24f) else SurfaceElevated)
                                            .border(
                                                width = if (selected) 1.5.dp else 1.dp,
                                                color = if (selected) accent else SurfaceStroke,
                                                shape = SharpChipShape,
                                            )
                                            .clickable {
                                                selectedRoundIdx = idx
                                                selectedEventIdx = 0
                                                isAutoPlaying = false
                                            }
                                            .padding(horizontal = 9.dp, vertical = 5.dp),
                                    ) {
                                        Text(
                                            text = "R${r.roundNum}",
                                            color = if (selected) accent else TextSecondary,
                                            fontWeight = FontWeight.Black,
                                            fontSize = 11.sp,
                                        )
                                    }
                                }
                            }
                            Spacer(modifier = Modifier.height(8.dp))
                        } else if (heatmapMode) {
                            // Row 1: Metric (Kills / Deaths / First Bloods / Plants)
                            Row(
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .horizontalScroll(rememberScrollState()),
                                horizontalArrangement = Arrangement.spacedBy(5.dp),
                            ) {
                                HeatmapMetric.entries.forEach { metric ->
                                    val selected = heatmapMetric == metric
                                    val accent = when (metric) {
                                        HeatmapMetric.KILLS -> ValCyan
                                        HeatmapMetric.DEATHS -> ValRed
                                        HeatmapMetric.FIRST_BLOODS -> ValGold
                                        HeatmapMetric.PLANTS -> ValAmber
                                    }
                                    Box(
                                        modifier = Modifier
                                            .clip(SharpChipShape)
                                            .background(if (selected) accent.copy(alpha = 0.24f) else SurfaceElevated)
                                            .border(
                                                width = if (selected) 1.4.dp else 1.dp,
                                                color = if (selected) accent else SurfaceStroke,
                                                shape = SharpChipShape,
                                            )
                                            .clickable { heatmapMetric = metric }
                                            .padding(horizontal = 9.dp, vertical = 5.dp),
                                    ) {
                                        Text(
                                            text = metric.label,
                                            color = if (selected) accent else TextSecondary,
                                            fontWeight = FontWeight.Black,
                                            fontSize = 11.sp,
                                        )
                                    }
                                }
                            }
                            Spacer(modifier = Modifier.height(6.dp))
                            // Row 2: Half filter + Team / Player scope
                            Row(
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .horizontalScroll(rememberScrollState()),
                                horizontalArrangement = Arrangement.spacedBy(5.dp),
                                verticalAlignment = Alignment.CenterVertically,
                            ) {
                                HeatmapHalf.entries.forEach { half ->
                                    val selected = heatmapHalf == half
                                    Box(
                                        modifier = Modifier
                                            .clip(SharpChipShape)
                                            .background(if (selected) ValPurple.copy(alpha = 0.24f) else SurfaceElevated)
                                            .border(
                                                width = if (selected) 1.25.dp else 1.dp,
                                                color = if (selected) ValPurple else SurfaceStroke,
                                                shape = SharpChipShape,
                                            )
                                            .clickable { heatmapHalf = half }
                                            .padding(horizontal = 8.dp, vertical = 4.dp),
                                    ) {
                                        Text(
                                            text = half.label,
                                            color = if (selected) ValPurple else TextSecondary,
                                            fontWeight = FontWeight.Bold,
                                            fontSize = 10.sp,
                                        )
                                    }
                                }

                                if (heatmapMetric != HeatmapMetric.PLANTS) {
                                    Box(
                                        modifier = Modifier
                                            .width(1.dp)
                                            .height(16.dp)
                                            .background(SurfaceStroke),
                                    )
                                    val scopes = buildList {
                                        add("ALL" to "All Players")
                                        add("ALLY" to "Allies")
                                        add("ENEMY" to "Enemies")
                                        detail.scoreboard
                                            .sortedByDescending { it.puuid.equals(viewedPuuid, ignoreCase = true) }
                                            .forEach { sp ->
                                                val shortLabel = if (sp.puuid.equals(viewedPuuid, ignoreCase = true)) {
                                                    "You (${sp.agentName})"
                                                } else {
                                                    "${sp.agentName} · ${sp.displayName.substringBefore("#")}"
                                                }
                                                add(sp.puuid.lowercase() to shortLabel)
                                            }
                                    }
                                    scopes.forEach { (key, label) ->
                                        val selected = heatmapScope.equals(key, ignoreCase = true)
                                        val accent = when (key) {
                                            "ENEMY" -> ValRed
                                            else -> ValCyan
                                        }
                                        Box(
                                            modifier = Modifier
                                                .clip(SharpChipShape)
                                                .background(if (selected) accent.copy(alpha = 0.22f) else SurfaceElevated)
                                                .border(
                                                    width = if (selected) 1.25.dp else 1.dp,
                                                    color = if (selected) accent else SurfaceStroke,
                                                    shape = SharpChipShape,
                                                )
                                                .clickable { heatmapScope = key }
                                                .padding(horizontal = 8.dp, vertical = 4.dp),
                                        ) {
                                            Text(
                                                text = label,
                                                color = if (selected) accent else TextSecondary,
                                                fontWeight = FontWeight.Bold,
                                                fontSize = 10.sp,
                                                maxLines = 1,
                                            )
                                        }
                                    }
                                }
                            }
                            Spacer(modifier = Modifier.height(8.dp))
                        }

                        val filteredRounds = remember(detail.rounds, heatmapHalf) {
                            detail.rounds.filter { r ->
                                when (heatmapHalf) {
                                    HeatmapHalf.ALL -> true
                                    HeatmapHalf.FIRST_HALF -> r.roundNum <= 12u
                                    HeatmapHalf.SECOND_HALF -> r.roundNum > 12u
                                }
                            }
                        }

                        // Square 2D Minimap + Exact Player Positions + FOV Cones + Filtered Thermal Heatmap
                        BoxWithConstraints(
                            modifier = Modifier
                                .fillMaxWidth()
                                .aspectRatio(1f)
                                .clip(SharpCardShape)
                                .background(ObsidianBg)
                                .border(1.dp, SurfaceStroke, SharpCardShape),
                        ) {
                            val mapW = maxWidth
                            val mapH = maxHeight

                            if (detail.mapMinimapIcon != null) {
                                AsyncImage(
                                    model = detail.mapMinimapIcon,
                                    contentDescription = "Tactical Minimap",
                                    modifier = Modifier.fillMaxSize(),
                                    contentScale = ContentScale.Fit,
                                )
                            }

                            val activeKill = (activeEvent as? RoundTimelineEvent.Kill)?.kill
                            val activeTime = activeEvent?.timeSecs ?: 0.0
                            val aliveSnapshots = activeEvent?.playerPositions.orEmpty()
                            val priorKills = if (!heatmapMode && activeRound != null) {
                                activeRound.kills.filter { k ->
                                    k.timeInRoundSecs < activeTime - 0.001 &&
                                        (activeKill == null || k !== activeKill)
                                }
                            } else {
                                emptyList()
                            }

                            val heatmapKills = remember(filteredRounds, heatmapMetric, heatmapScope, detail.myTeamId) {
                                val raw = if (heatmapMetric == HeatmapMetric.FIRST_BLOODS) {
                                    filteredRounds.mapNotNull { it.kills.firstOrNull() }
                                } else {
                                    filteredRounds.flatMap { it.kills }
                                }
                                raw.filter { k ->
                                    val puuid = if (heatmapMetric == HeatmapMetric.DEATHS) k.victimPuuid else k.killerPuuid
                                    val team = if (heatmapMetric == HeatmapMetric.DEATHS) k.victimTeam else k.killerTeam
                                    when {
                                        heatmapScope == "ALL" -> true
                                        heatmapScope == "ALLY" -> team.equals(detail.myTeamId, ignoreCase = true)
                                        heatmapScope == "ENEMY" -> !team.equals(detail.myTeamId, ignoreCase = true)
                                        else -> puuid.equals(heatmapScope, ignoreCase = true)
                                    }
                                }
                            }
                            val heatmapPlants = remember(filteredRounds) {
                                filteredRounds.filter { it.plantNormX != null && it.plantNormY != null }
                            }

                            Canvas(modifier = Modifier.fillMaxSize()) {
                                if (size.width <= 24f || size.height <= 24f) return@Canvas
                                fun safeNorm(v: Double): Float {
                                    val f = v.toFloat()
                                    return if (f.isFinite()) f else 0.5f
                                }
                                // Subtle tactical grid
                                val step = size.width / 8f
                                for (i in 1..7) {
                                    drawLine(
                                        color = SurfaceStroke.copy(alpha = 0.22f),
                                        start = Offset(step * i, 0f),
                                        end = Offset(step * i, size.height),
                                    )
                                    drawLine(
                                        color = SurfaceStroke.copy(alpha = 0.22f),
                                        start = Offset(0f, step * i),
                                        end = Offset(size.width, step * i),
                                    )
                                }

                                if (heatmapMode) {
                                    if (heatmapMetric == HeatmapMetric.PLANTS) {
                                        heatmapPlants.forEach { r ->
                                            val px = (safeNorm(r.plantNormX ?: 0.5) * size.width).coerceIn(10f, size.width - 10f)
                                            val py = (safeNorm(r.plantNormY ?: 0.5) * size.height).coerceIn(10f, size.height - 10f)
                                            val pt = Offset(px, py)
                                            drawCircle(
                                                brush = Brush.radialGradient(
                                                    colors = listOf(ValGold.copy(alpha = 0.45f), ValAmber.copy(alpha = 0.14f), Color.Transparent),
                                                    center = pt,
                                                    radius = 28.dp.toPx(),
                                                ),
                                                radius = 28.dp.toPx(),
                                                center = pt,
                                            )
                                            drawCircle(
                                                color = ValGold,
                                                radius = 4.dp.toPx(),
                                                center = pt,
                                            )
                                        }
                                    } else {
                                        heatmapKills.forEach { k ->
                                            val kx = (safeNorm(k.killerNormX) * size.width).coerceIn(10f, size.width - 10f)
                                            val ky = (safeNorm(k.killerNormY) * size.height).coerceIn(10f, size.height - 10f)
                                            val vx = (safeNorm(k.victimNormX) * size.width).coerceIn(10f, size.width - 10f)
                                            val vy = (safeNorm(k.victimNormY) * size.height).coerceIn(10f, size.height - 10f)
                                            val isDeath = heatmapMetric == HeatmapMetric.DEATHS
                                            val hotspot = if (isDeath) Offset(vx, vy) else Offset(kx, ky)
                                            val (outerColor, coreColor) = when (heatmapMetric) {
                                                HeatmapMetric.DEATHS -> ValRed to ValAmber
                                                HeatmapMetric.FIRST_BLOODS -> ValGold to ValAmber
                                                else -> if (k.killerTeam.equals(detail.myTeamId, ignoreCase = true)) ValCyan to ValGreen else ValRed to ValAmber
                                            }
                                            if (heatmapMetric == HeatmapMetric.FIRST_BLOODS) {
                                                drawLine(
                                                    color = outerColor.copy(alpha = 0.35f),
                                                    start = Offset(kx, ky),
                                                    end = Offset(vx, vy),
                                                    strokeWidth = 1.4.dp.toPx(),
                                                )
                                            }
                                            drawCircle(
                                                brush = Brush.radialGradient(
                                                    colors = listOf(coreColor.copy(alpha = 0.32f), outerColor.copy(alpha = 0.14f), Color.Transparent),
                                                    center = hotspot,
                                                    radius = 26.dp.toPx(),
                                                ),
                                                radius = 26.dp.toPx(),
                                                center = hotspot,
                                            )
                                            drawCircle(
                                                color = outerColor.copy(alpha = 0.9f),
                                                radius = 3.2.dp.toPx(),
                                                center = hotspot,
                                            )
                                        }
                                    }
                                } else if (activeRound != null) {
                                    // 1. Spike plant beacon (visible from plant timestamp onward)
                                    val px = activeRound.plantNormX
                                    val py = activeRound.plantNormY
                                    val plantTime = activeRound.plantTimeSecs
                                    val plantVisible = px != null && py != null &&
                                        (plantTime == null || activeTime >= plantTime - 0.01)
                                    if (plantVisible && px != null && py != null) {
                                        val plantPt = Offset(safeNorm(px) * size.width, safeNorm(py) * size.height)
                                        drawCircle(
                                            brush = Brush.radialGradient(
                                                colors = listOf(ValGold.copy(alpha = 0.55f), Color.Transparent),
                                                center = plantPt,
                                                radius = 22.dp.toPx(),
                                            ),
                                            radius = 22.dp.toPx(),
                                            center = plantPt,
                                        )
                                        drawCircle(
                                            color = ValGold,
                                            radius = 5.dp.toPx(),
                                            center = plantPt,
                                            style = Stroke(width = 2.dp.toPx()),
                                        )
                                    }

                                    // 2. Prior deaths earlier in this round (faded X marks so you see who already fell)
                                    priorKills.forEach { pk ->
                                        val vx = (safeNorm(pk.victimNormX) * size.width).coerceIn(12f, size.width - 12f)
                                        val vy = (safeNorm(pk.victimNormY) * size.height).coerceIn(12f, size.height - 12f)
                                        val victimAlly = pk.victimTeam.equals(detail.myTeamId, ignoreCase = true)
                                        val c = (if (victimAlly) ValCyan else ValRed).copy(alpha = 0.42f)
                                        val r = 4.5.dp.toPx()
                                        drawLine(c, Offset(vx - r, vy - r), Offset(vx + r, vy + r), strokeWidth = 1.6.dp.toPx())
                                        drawLine(c, Offset(vx + r, vy - r), Offset(vx - r, vy + r), strokeWidth = 1.6.dp.toPx())
                                    }

                                    // 3. FOV Vision Cones for EVERY alive player at the exact selected event timestamp
                                    val coneRadius = 24.dp.toPx()
                                    aliveSnapshots.forEach { ps ->
                                        val cx = (safeNorm(ps.normX) * size.width).coerceIn(12f, size.width - 12f)
                                        val cy = (safeNorm(ps.normY) * size.height).coerceIn(12f, size.height - 12f)
                                        val isAlly = ps.teamId.equals(detail.myTeamId, ignoreCase = true)
                                        val coneColor = (if (isAlly) ValCyan else ValRed).copy(alpha = 0.28f)
                                        val angleDeg = ((ps.viewAngleRad.takeIf { it.isFinite() } ?: 0.0) * 180.0 / PI).toFloat()
                                        drawArc(
                                            color = coneColor,
                                            startAngle = angleDeg - 26f,
                                            sweepAngle = 52f,
                                            useCenter = true,
                                            topLeft = Offset(cx - coneRadius, cy - coneRadius),
                                            size = Size(coneRadius * 2f, coneRadius * 2f),
                                        )
                                    }

                                    // 4. ONLY the SINGLE selected kill's engagement line (Killer -> Victim) + elimination X
                                    if (activeKill != null) {
                                        val start = Offset(
                                            (safeNorm(activeKill.killerNormX) * size.width).coerceIn(12f, size.width - 12f),
                                            (safeNorm(activeKill.killerNormY) * size.height).coerceIn(12f, size.height - 12f),
                                        )
                                        val end = Offset(
                                            (safeNorm(activeKill.victimNormX) * size.width).coerceIn(12f, size.width - 12f),
                                            (safeNorm(activeKill.victimNormY) * size.height).coerceIn(12f, size.height - 12f),
                                        )
                                        val killerIsAlly = activeKill.killerTeam.equals(detail.myTeamId, ignoreCase = true)
                                        val lineColor = if (killerIsAlly) ValCyan else ValRed

                                        drawLine(
                                            color = lineColor.copy(alpha = 0.92f),
                                            start = start,
                                            end = end,
                                            strokeWidth = 2.4.dp.toPx(),
                                            cap = StrokeCap.Round,
                                            pathEffect = PathEffect.dashPathEffect(floatArrayOf(12f, 7f), 0f),
                                        )
                                        // Killer highlight halo
                                        drawCircle(
                                            color = ValGold,
                                            radius = 14.dp.toPx(),
                                            center = start,
                                            style = Stroke(width = 2.dp.toPx()),
                                        )
                                        // Victim elimination X marker
                                        val xr = 7.dp.toPx()
                                        drawCircle(
                                            color = ObsidianBg.copy(alpha = 0.85f),
                                            radius = 10.dp.toPx(),
                                            center = end,
                                        )
                                        drawLine(ValRed, Offset(end.x - xr, end.y - xr), Offset(end.x + xr, end.y + xr), strokeWidth = 2.6.dp.toPx(), cap = StrokeCap.Round)
                                        drawLine(ValRed, Offset(end.x + xr, end.y - xr), Offset(end.x - xr, end.y + xr), strokeWidth = 2.6.dp.toPx(), cap = StrokeCap.Round)
                                    }
                                }
                            }

                            // 5. Real Agent Portrait Markers for EVERY alive player at this exact timestamp + Victim & Killer fallback
                            val iconSize = 22.dp
                            if (!heatmapMode && mapW >= iconSize && mapH >= iconSize) {
                                val halfIcon = iconSize / 2
                                val maxX = (mapW - halfIcon).coerceAtLeast(halfIcon)
                                val maxY = (mapH - halfIcon).coerceAtLeast(halfIcon)
                                if (aliveSnapshots.isNotEmpty()) {
                                    aliveSnapshots.forEach { ps ->
                                        key(ps.puuid) {
                                            val isAlly = ps.teamId.equals(detail.myTeamId, ignoreCase = true)
                                            val isKiller = activeKill != null && ps.puuid.equals(activeKill.killerPuuid, ignoreCase = true)
                                            val ringColor = when {
                                                isKiller -> ValGold
                                                isAlly -> ValCyan
                                                else -> ValRed
                                            }
                                            val animX by animateFloatAsState(
                                                targetValue = ps.normX.toFloat().takeIf { it.isFinite() } ?: 0.5f,
                                                animationSpec = tween(durationMillis = replaySpeed.animMs),
                                                label = "posX",
                                            )
                                            val animY by animateFloatAsState(
                                                targetValue = ps.normY.toFloat().takeIf { it.isFinite() } ?: 0.5f,
                                                animationSpec = tween(durationMillis = replaySpeed.animMs),
                                                label = "posY",
                                            )
                                            val xDp = (mapW * (animX.takeIf { it.isFinite() } ?: 0.5f)).coerceIn(halfIcon, maxX) - halfIcon
                                            val yDp = (mapH * (animY.takeIf { it.isFinite() } ?: 0.5f)).coerceIn(halfIcon, maxY) - halfIcon
                                            Box(
                                                modifier = Modifier
                                                    .offset(x = xDp, y = yDp)
                                                    .size(iconSize)
                                                    .clip(CircleShape)
                                                    .background(ObsidianBg)
                                                    .border(if (isKiller) 2.dp else 1.5.dp, ringColor, CircleShape),
                                                contentAlignment = Alignment.Center,
                                            ) {
                                                if (ps.agentIcon != null) {
                                                    AsyncImage(
                                                        model = ps.agentIcon,
                                                        contentDescription = ps.displayName,
                                                        modifier = Modifier.fillMaxSize(),
                                                        contentScale = ContentScale.Crop,
                                                    )
                                                } else {
                                                    Box(
                                                        modifier = Modifier
                                                            .size(8.dp)
                                                            .background(ringColor, CircleShape),
                                                    )
                                                }
                                            }
                                        }
                                    }
                                } else if (activeKill != null) {
                                    // Fallback killer marker if Riot omitted playerLocations on an edge-case kill
                                    val kx = activeKill.killerNormX.toFloat().takeIf { it.isFinite() } ?: 0.5f
                                    val ky = activeKill.killerNormY.toFloat().takeIf { it.isFinite() } ?: 0.5f
                                    val kxDp = (mapW * kx).coerceIn(halfIcon, maxX) - halfIcon
                                    val kyDp = (mapH * ky).coerceIn(halfIcon, maxY) - halfIcon
                                    Box(
                                        modifier = Modifier
                                            .offset(x = kxDp, y = kyDp)
                                            .size(iconSize)
                                            .clip(CircleShape)
                                            .background(ObsidianBg)
                                            .border(2.dp, ValGold, CircleShape),
                                        contentAlignment = Alignment.Center,
                                    ) {
                                        if (activeKill.killerAgentIcon != null) {
                                            AsyncImage(
                                                model = activeKill.killerAgentIcon,
                                                contentDescription = activeKill.killerName,
                                                modifier = Modifier.fillMaxSize(),
                                                contentScale = ContentScale.Crop,
                                            )
                                        }
                                    }
                                }

                                // Victim Agent Portrait with red elimination ring + X overlay at exact death position
                                if (activeKill != null) {
                                    val vx = activeKill.victimNormX.toFloat().takeIf { it.isFinite() } ?: 0.5f
                                    val vy = activeKill.victimNormY.toFloat().takeIf { it.isFinite() } ?: 0.5f
                                    val vxDp = (mapW * vx).coerceIn(halfIcon, maxX) - halfIcon
                                    val vyDp = (mapH * vy).coerceIn(halfIcon, maxY) - halfIcon
                                    Box(
                                        modifier = Modifier
                                            .offset(x = vxDp, y = vyDp)
                                            .size(iconSize)
                                            .clip(CircleShape)
                                            .background(ObsidianBg)
                                            .border(1.8.dp, ValRed, CircleShape),
                                        contentAlignment = Alignment.Center,
                                    ) {
                                        if (activeKill.victimAgentIcon != null) {
                                            AsyncImage(
                                                model = activeKill.victimAgentIcon,
                                                contentDescription = activeKill.victimName,
                                                modifier = Modifier.fillMaxSize(),
                                                contentScale = ContentScale.Crop,
                                                alpha = 0.55f,
                                            )
                                        }
                                        Icon(
                                            imageVector = Icons.Default.Close,
                                            contentDescription = "Eliminated",
                                            tint = ValRed,
                                            modifier = Modifier.size(14.dp),
                                        )
                                    }
                                }
                            }

                            // 6. Compact Live Snapshot HUD overlay at bottom of minimap
                            if (heatmapMode) {
                                val pointCount = if (heatmapMetric == HeatmapMetric.PLANTS) heatmapPlants.size else heatmapKills.size
                                val scopeName = when (heatmapScope) {
                                    "ALL" -> "All Players"
                                    "ALLY" -> "Allies"
                                    "ENEMY" -> "Enemies"
                                    else -> detail.scoreboard.firstOrNull { it.puuid.equals(heatmapScope, ignoreCase = true) }?.agentName ?: "Player"
                                }
                                val siteSplit = if (heatmapMetric == HeatmapMetric.PLANTS && heatmapPlants.isNotEmpty()) {
                                    val grouped = heatmapPlants.groupingBy { (it.plantSite ?: "?").uppercase() }.eachCount()
                                    " • " + grouped.entries.sortedBy { it.key }.joinToString(" / ") { "${it.key}:${it.value}" }
                                } else {
                                    ""
                                }
                                Row(
                                    modifier = Modifier
                                        .align(Alignment.BottomCenter)
                                        .fillMaxWidth()
                                        .background(ObsidianBg.copy(alpha = 0.86f))
                                        .padding(horizontal = 8.dp, vertical = 5.dp),
                                    verticalAlignment = Alignment.CenterVertically,
                                    horizontalArrangement = Arrangement.SpaceBetween,
                                ) {
                                    Text(
                                        text = "${heatmapMetric.label} • ${if (heatmapMetric == HeatmapMetric.PLANTS) heatmapHalf.label else "$scopeName • ${heatmapHalf.label}"}$siteSplit",
                                        color = TextPrimary,
                                        fontSize = 10.5.sp,
                                        fontWeight = FontWeight.Bold,
                                        maxLines = 1,
                                        overflow = TextOverflow.Ellipsis,
                                        modifier = Modifier.weight(1f),
                                    )
                                    Spacer(modifier = Modifier.width(6.dp))
                                    TacticalBadge("$pointCount pts", if (heatmapMetric == HeatmapMetric.DEATHS) ValRed else ValCyan)
                                }
                            } else if (activeEvent != null) {
                                val alliesAlive = aliveSnapshots.count { it.teamId.equals(detail.myTeamId, ignoreCase = true) }
                                val enemiesAlive = aliveSnapshots.size - alliesAlive
                                Row(
                                    modifier = Modifier
                                        .align(Alignment.BottomCenter)
                                        .fillMaxWidth()
                                        .background(ObsidianBg.copy(alpha = 0.84f))
                                        .padding(horizontal = 8.dp, vertical = 5.dp),
                                    verticalAlignment = Alignment.CenterVertically,
                                    horizontalArrangement = Arrangement.SpaceBetween,
                                ) {
                                    Text(
                                        text = "${"%.1f".format(Locale.US, activeEvent.timeSecs)}s • ${activeEvent.summary}",
                                        color = TextPrimary,
                                        fontSize = 10.5.sp,
                                        fontWeight = FontWeight.Bold,
                                        maxLines = 1,
                                        overflow = TextOverflow.Ellipsis,
                                        modifier = Modifier.weight(1f),
                                    )
                                    if (aliveSnapshots.isNotEmpty()) {
                                        Spacer(modifier = Modifier.width(6.dp))
                                        TacticalBadge("${alliesAlive}v${enemiesAlive}", ValCyan)
                                    }
                                }
                            }
                        }

                        if (!heatmapMode && activeRound != null) {
                            Spacer(modifier = Modifier.height(6.dp))
                            Row(
                                verticalAlignment = Alignment.CenterVertically,
                                horizontalArrangement = Arrangement.spacedBy(12.dp),
                                modifier = Modifier.fillMaxWidth(),
                            ) {
                                ReplayLegendDot("Ally", ValCyan)
                                ReplayLegendDot("Enemy", ValRed)
                                ReplayLegendDot("Killer", ValGold)
                                Spacer(modifier = Modifier.weight(1f))
                                Text(
                                    text = "R${activeRound.roundNum} • ${activeRound.endType}",
                                    color = TextSecondary,
                                    fontSize = 10.5.sp,
                                    fontWeight = FontWeight.Bold,
                                    maxLines = 1,
                                    overflow = TextOverflow.Ellipsis,
                                )
                                TacticalBadge(
                                    text = if (activeRound.winningTeam.equals(detail.myTeamId, ignoreCase = true)) "WON" else "LOST",
                                    accent = if (activeRound.winningTeam.equals(detail.myTeamId, ignoreCase = true)) ValGreen else ValRed,
                                )
                            }

                            // Playback controls + Speed + Clickable Chronological Event Timeline
                            if (timelineEvents.isNotEmpty()) {
                                Spacer(modifier = Modifier.height(8.dp))
                                Row(
                                    modifier = Modifier.fillMaxWidth(),
                                    verticalAlignment = Alignment.CenterVertically,
                                    horizontalArrangement = Arrangement.spacedBy(6.dp),
                                ) {
                                    // Prev / Play-Pause / Next / Speed sharp square controls
                                    Row(horizontalArrangement = Arrangement.spacedBy(4.dp), verticalAlignment = Alignment.CenterVertically) {
                                        Box(
                                            modifier = Modifier
                                                .size(30.dp)
                                                .clip(SharpChipShape)
                                                .background(SurfaceElevated)
                                                .border(1.dp, SurfaceStroke, SharpChipShape)
                                                .clickable(enabled = safeEventIdx > 0) {
                                                    isAutoPlaying = false
                                                    selectedEventIdx = (safeEventIdx - 1).coerceAtLeast(0)
                                                },
                                            contentAlignment = Alignment.Center,
                                        ) {
                                            Icon(
                                                Icons.Default.SkipPrevious,
                                                contentDescription = "Previous event",
                                                tint = if (safeEventIdx > 0) TextPrimary else TextMuted,
                                                modifier = Modifier.size(16.dp),
                                            )
                                        }
                                        Box(
                                            modifier = Modifier
                                                .size(30.dp)
                                                .clip(SharpChipShape)
                                                .background(if (isAutoPlaying) ValRed.copy(alpha = 0.25f) else ValCyan.copy(alpha = 0.2f))
                                                .border(1.dp, if (isAutoPlaying) ValRed else ValCyan, SharpChipShape)
                                                .clickable {
                                                    if (!isAutoPlaying && safeEventIdx >= timelineEvents.lastIndex) {
                                                        selectedEventIdx = 0
                                                    }
                                                    isAutoPlaying = !isAutoPlaying
                                                },
                                            contentAlignment = Alignment.Center,
                                        ) {
                                            Icon(
                                                imageVector = if (isAutoPlaying) Icons.Default.Pause else Icons.Default.PlayArrow,
                                                contentDescription = "Play/Pause timeline",
                                                tint = if (isAutoPlaying) ValRed else ValCyan,
                                                modifier = Modifier.size(16.dp),
                                            )
                                        }
                                        Box(
                                            modifier = Modifier
                                                .size(30.dp)
                                                .clip(SharpChipShape)
                                                .background(SurfaceElevated)
                                                .border(1.dp, SurfaceStroke, SharpChipShape)
                                                .clickable(enabled = safeEventIdx < timelineEvents.lastIndex) {
                                                    isAutoPlaying = false
                                                    selectedEventIdx = (safeEventIdx + 1).coerceAtMost(timelineEvents.lastIndex)
                                                },
                                            contentAlignment = Alignment.Center,
                                        ) {
                                            Icon(
                                                Icons.Default.SkipNext,
                                                contentDescription = "Next event",
                                                tint = if (safeEventIdx < timelineEvents.lastIndex) TextPrimary else TextMuted,
                                                modifier = Modifier.size(16.dp),
                                            )
                                        }
                                        Box(
                                            modifier = Modifier
                                                .height(30.dp)
                                                .clip(SharpChipShape)
                                                .background(SurfaceElevated)
                                                .border(1.dp, SurfaceStroke, SharpChipShape)
                                                .clickable {
                                                    val next = (ReplaySpeed.entries.indexOf(replaySpeed) + 1) % ReplaySpeed.entries.size
                                                    replaySpeed = ReplaySpeed.entries[next]
                                                }
                                                .padding(horizontal = 7.dp),
                                            contentAlignment = Alignment.Center,
                                        ) {
                                            Text(
                                                text = replaySpeed.label,
                                                color = ValGold,
                                                fontSize = 10.sp,
                                                fontWeight = FontWeight.Black,
                                            )
                                        }
                                    }

                                    // Scrollable clickable event chips (FB, K2, PLANT, DEFUSE...)
                                    Row(
                                        modifier = Modifier
                                            .weight(1f)
                                            .horizontalScroll(rememberScrollState()),
                                        horizontalArrangement = Arrangement.spacedBy(5.dp),
                                    ) {
                                        timelineEvents.forEachIndexed { idx, ev ->
                                            val isSelected = idx == safeEventIdx
                                            val chipAccent = when (ev) {
                                                is RoundTimelineEvent.Plant -> ValGold
                                                is RoundTimelineEvent.Defuse -> ValGreen
                                                is RoundTimelineEvent.Kill -> when {
                                                    ev.isFirstBlood -> ValGold
                                                    ev.kill.killerTeam.equals(detail.myTeamId, ignoreCase = true) -> ValCyan
                                                    else -> ValRed
                                                }
                                            }
                                            Box(
                                                modifier = Modifier
                                                    .clip(SharpChipShape)
                                                    .background(if (isSelected) chipAccent.copy(alpha = 0.25f) else SurfaceElevated)
                                                    .border(
                                                        width = if (isSelected) 1.5.dp else 1.dp,
                                                        color = if (isSelected) chipAccent else SurfaceStroke,
                                                        shape = SharpChipShape,
                                                    )
                                                    .clickable {
                                                        isAutoPlaying = false
                                                        selectedEventIdx = idx
                                                    }
                                                    .padding(horizontal = 8.dp, vertical = 5.dp),
                                            ) {
                                                Text(
                                                    text = "${ev.shortTag} ${"%.0f".format(Locale.US, ev.timeSecs)}s",
                                                    color = if (isSelected) chipAccent else TextSecondary,
                                                    fontSize = 10.5.sp,
                                                    fontWeight = FontWeight.Black,
                                                )
                                            }
                                        }
                                    }
                                }

                                Spacer(modifier = Modifier.height(8.dp))

                                // Interactive Kill & Objective Feed (tap ANY row to jump map positions to that exact moment!)
                                timelineEvents.forEachIndexed { idx, ev ->
                                    val isSelected = idx == safeEventIdx
                                    when (ev) {
                                        is RoundTimelineEvent.Kill -> {
                                            val k = ev.kill
                                            val allyKill = k.killerTeam.equals(detail.myTeamId, ignoreCase = true)
                                            val killerColor = if (allyKill) ValCyan else ValRed
                                            val victimColor = if (allyKill) ValRed else ValCyan
                                            Row(
                                                modifier = Modifier
                                                    .fillMaxWidth()
                                                    .padding(vertical = 2.dp)
                                                    .clip(SharpChipShape)
                                                    .background(
                                                        if (isSelected) GlossyElevatedBrush else Brush.verticalGradient(listOf(Color.Transparent, Color.Transparent)),
                                                    )
                                                    .then(
                                                        if (isSelected) {
                                                            Modifier.border(1.25.dp, killerColor, SharpChipShape)
                                                        } else {
                                                            Modifier
                                                        },
                                                    )
                                                    .clickable {
                                                        isAutoPlaying = false
                                                        selectedEventIdx = idx
                                                    }
                                                    .padding(horizontal = 8.dp, vertical = 6.dp),
                                                verticalAlignment = Alignment.CenterVertically,
                                            ) {
                                                Text(
                                                    text = "${"%.1f".format(Locale.US, k.timeInRoundSecs)}s",
                                                    color = if (isSelected) ValGold else TextMuted,
                                                    fontSize = 10.5.sp,
                                                    fontWeight = FontWeight.Bold,
                                                    modifier = Modifier.width(34.dp),
                                                )
                                                if (ev.isFirstBlood) {
                                                    TacticalBadge("FB", ValGold)
                                                    Spacer(modifier = Modifier.width(4.dp))
                                                } else if (ev.isTrade) {
                                                    TacticalBadge("TRD", ValPurple)
                                                    Spacer(modifier = Modifier.width(4.dp))
                                                }
                                                Row(
                                                    modifier = Modifier.weight(1f),
                                                    verticalAlignment = Alignment.CenterVertically,
                                                    horizontalArrangement = Arrangement.End,
                                                ) {
                                                    Text(
                                                        text = k.killerName,
                                                        color = killerColor,
                                                        fontSize = 11.sp,
                                                        fontWeight = FontWeight.Bold,
                                                        maxLines = 1,
                                                        overflow = TextOverflow.Ellipsis,
                                                        modifier = Modifier.weight(1f, fill = false),
                                                    )
                                                    Spacer(modifier = Modifier.width(6.dp))
                                                    if (!k.weaponIcon.isNullOrBlank()) {
                                                        Box(
                                                            modifier = Modifier
                                                                .clip(SharpChipShape)
                                                                .background(SurfaceElevated)
                                                                .border(0.6.dp, SurfaceStroke, SharpChipShape)
                                                                .padding(horizontal = 5.dp, vertical = 2.dp),
                                                            contentAlignment = Alignment.Center,
                                                        ) {
                                                            AsyncImage(
                                                                model = k.weaponIcon,
                                                                contentDescription = k.weaponName,
                                                                modifier = Modifier
                                                                    .height(14.dp)
                                                                    .widthIn(min = 16.dp, max = 46.dp),
                                                                contentScale = ContentScale.Fit,
                                                            )
                                                        }
                                                    } else {
                                                        TacticalBadge(k.weaponName, TextSecondary)
                                                    }
                                                    Spacer(modifier = Modifier.width(6.dp))
                                                    Text(
                                                        text = k.victimName,
                                                        color = victimColor,
                                                        fontSize = 11.sp,
                                                        fontWeight = FontWeight.Bold,
                                                        maxLines = 1,
                                                        overflow = TextOverflow.Ellipsis,
                                                        modifier = Modifier.weight(1f, fill = false),
                                                    )
                                                }
                                            }
                                        }

                                        is RoundTimelineEvent.Plant, is RoundTimelineEvent.Defuse -> {
                                            val accent = if (ev is RoundTimelineEvent.Plant) ValGold else ValGreen
                                            Row(
                                                modifier = Modifier
                                                    .fillMaxWidth()
                                                    .padding(vertical = 2.dp)
                                                    .clip(SharpChipShape)
                                                    .background(accent.copy(alpha = if (isSelected) 0.18f else 0.08f))
                                                    .border(
                                                        width = if (isSelected) 1.25.dp else 0.5.dp,
                                                        color = if (isSelected) accent else accent.copy(alpha = 0.35f),
                                                        shape = SharpChipShape,
                                                    )
                                                    .clickable {
                                                        isAutoPlaying = false
                                                        selectedEventIdx = idx
                                                    }
                                                    .padding(horizontal = 8.dp, vertical = 6.dp),
                                                verticalAlignment = Alignment.CenterVertically,
                                            ) {
                                                Text(
                                                    text = "${"%.1f".format(Locale.US, ev.timeSecs)}s",
                                                    color = accent,
                                                    fontSize = 10.5.sp,
                                                    fontWeight = FontWeight.Bold,
                                                    modifier = Modifier.width(36.dp),
                                                )
                                                TacticalBadge(ev.shortTag, accent)
                                                Spacer(modifier = Modifier.width(8.dp))
                                                Text(
                                                    text = ev.summary,
                                                    color = TextPrimary,
                                                    fontSize = 11.sp,
                                                    fontWeight = FontWeight.SemiBold,
                                                    maxLines = 1,
                                                    overflow = TextOverflow.Ellipsis,
                                                )
                                            }
                                        }
                                    }
                                }
                            } else {
                                Spacer(modifier = Modifier.height(6.dp))
                                Text(
                                    text = "No kill or spike events recorded in Round ${activeRound.roundNum}.",
                                    color = TextMuted,
                                    fontSize = 11.sp,
                                )
                            }
                        }
                    }
                }
            }

            // Full 10-player Scoreboard sorted by ACS
            item {
                Text(
                    text = "SCOREBOARD",
                    fontWeight = FontWeight.Black,
                    fontSize = 12.sp,
                    color = TextSecondary,
                    letterSpacing = 1.sp,
                )
            }

            itemsIndexed(detail.scoreboard, key = { _, p -> p.puuid }) { _, p ->
                val isAlly = p.teamId == detail.myTeamId
                val teamColor = if (isAlly) ValCyan else ValRed
                val isMatchMvp = p.puuid == matchMvpPuuid
                val isTeamMvp = !isMatchMvp && p.puuid == teamMvpPuuid
                val isActualSelf = activeSessionPuuid != null && p.puuid.equals(activeSessionPuuid, ignoreCase = true)
                val duelVsViewed = if (!isAlly) enemyDuelRecords[p.puuid.trim().lowercase()] else null
                Surface(
                    color = SurfaceCard,
                    shape = RoundedCornerShape(10.dp),
                    modifier = Modifier
                        .fillMaxWidth()
                        .border(
                            width = if (p.isMe || isMatchMvp) 1.5.dp else 1.dp,
                            color = when {
                                isMatchMvp -> ValGold
                                p.isMe -> ValCyan
                                else -> teamColor.copy(alpha = 0.35f)
                            },
                            shape = RoundedCornerShape(10.dp),
                        )
                        .clickable { onInspectPlayer(p.puuid, p.displayName) },
                ) {
                    Row(
                        modifier = Modifier.padding(10.dp),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        if (p.agentIcon != null) {
                            AsyncImage(
                                model = p.agentIcon,
                                contentDescription = p.agentName,
                                modifier = Modifier
                                    .size(38.dp)
                                    .clip(RoundedCornerShape(8.dp))
                                    .background(SurfaceElevated),
                            )
                            Spacer(modifier = Modifier.width(10.dp))
                        }
                        Column(modifier = Modifier.weight(1f)) {
                            Row(
                                modifier = Modifier.fillMaxWidth(),
                                horizontalArrangement = Arrangement.SpaceBetween,
                                verticalAlignment = Alignment.CenterVertically,
                            ) {
                                Text(
                                    text = p.displayName,
                                    fontWeight = FontWeight.Bold,
                                    fontSize = 13.sp,
                                    color = TextPrimary,
                                    maxLines = 1,
                                    overflow = TextOverflow.Ellipsis,
                                    modifier = Modifier.weight(1f),
                                )
                                Spacer(modifier = Modifier.width(8.dp))
                                Text(
                                    text = "${p.kills}/${p.deaths}/${p.assists} • ${p.acs} ACS",
                                    fontWeight = FontWeight.Black,
                                    fontSize = 12.5.sp,
                                    color = teamColor,
                                    maxLines = 1,
                                    softWrap = false,
                                )
                            }
                            Spacer(modifier = Modifier.height(2.dp))
                            Row(
                                modifier = Modifier.fillMaxWidth(),
                                horizontalArrangement = Arrangement.SpaceBetween,
                                verticalAlignment = Alignment.CenterVertically,
                            ) {
                                Text(
                                    text = "${p.agentName} • ${p.rankName} • ${p.firstBloods} FB",
                                    color = parseHexColor(p.rankColor, teamColor),
                                    fontSize = 11.sp,
                                    maxLines = 1,
                                    overflow = TextOverflow.Ellipsis,
                                    modifier = Modifier.weight(1f),
                                )
                                Spacer(modifier = Modifier.width(8.dp))
                                Text(
                                    text = "${p.adr} ADR • ${p.hsPercent.toInt()}% HS",
                                    color = TextSecondary,
                                    fontSize = 10.5.sp,
                                    maxLines = 1,
                                    softWrap = false,
                                )
                            }
                            if (isMatchMvp || isTeamMvp || p.isMe || (duelVsViewed != null && (duelVsViewed.first > 0 || duelVsViewed.second > 0))) {
                                Spacer(modifier = Modifier.height(4.dp))
                                Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                                    if (isMatchMvp) {
                                        TacticalBadge("MATCH MVP", ValGold)
                                    } else if (isTeamMvp) {
                                        TacticalBadge("TEAM MVP", ValPurple)
                                    }
                                    if (isActualSelf) {
                                        TacticalBadge("YOU", ValCyan)
                                    } else if (p.isMe) {
                                        TacticalBadge("VIEWED", ValCyan)
                                    }
                                    if (duelVsViewed != null && (duelVsViewed.first > 0 || duelVsViewed.second > 0)) {
                                        val duelAccent = when {
                                            duelVsViewed.first > duelVsViewed.second -> ValGreen
                                            duelVsViewed.first < duelVsViewed.second -> ValRed
                                            else -> TextSecondary
                                        }
                                        TacticalBadge("DUEL ${duelVsViewed.first}-${duelVsViewed.second}", duelAccent)
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
