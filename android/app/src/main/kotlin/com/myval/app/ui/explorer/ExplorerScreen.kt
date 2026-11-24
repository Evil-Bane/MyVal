package com.myval.app.ui.explorer

import androidx.compose.animation.AnimatedVisibility
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
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ExpandLess
import androidx.compose.material.icons.filled.ExpandMore
import androidx.compose.material.icons.filled.Favorite
import androidx.compose.material.icons.filled.FavoriteBorder
import androidx.compose.material.icons.filled.Info
import androidx.compose.material.icons.filled.Search
import androidx.compose.material.icons.filled.Sync
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.FilterChip
import androidx.compose.material3.FilterChipDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil.compose.AsyncImage
import com.myval.app.ui.MyValUiState
import com.myval.app.ui.components.InnerCardShape
import com.myval.app.ui.components.SharpCardShape
import com.myval.app.ui.components.SharpChipShape
import com.myval.app.ui.components.TacticalBadge
import com.myval.app.ui.components.parseHexColor
import com.myval.app.ui.theme.ObsidianBg
import com.myval.app.ui.theme.SurfaceCard
import com.myval.app.ui.theme.SurfaceElevated
import com.myval.app.ui.theme.SurfaceStroke
import com.myval.app.ui.theme.TextMuted
import com.myval.app.ui.theme.TextPrimary
import com.myval.app.ui.theme.TextSecondary
import com.myval.app.ui.theme.ValCyan
import com.myval.app.ui.theme.ValGold
import com.myval.app.ui.theme.ValPurple
import com.myval.app.ui.theme.ValRed
import com.myval.app.ui.theme.ValWinGreen
import uniffi.myval_ffi.Agent
import uniffi.myval_ffi.MapInfo
import uniffi.myval_ffi.Skin

private enum class ExploreSection(val label: String) {
    SKINS("Skins"),
    AGENTS("Agents"),
    MAPS("Maps"),
}

private val WEAPON_FILTERS = listOf(
    "All",
    "Vandal",
    "Phantom",
    "Melee",
    "Operator",
    "Sheriff",
    "Ghost",
    "Classic",
    "Spectre",
)

private val ROLE_FILTERS = listOf(
    "All",
    "Duelist",
    "Initiator",
    "Controller",
    "Sentinel",
)

/**
 * Parses Riot ability description markup (`<whitebold>`, `<debuffred>`, `<teal>`, `</>`)
 * into clean styled [AnnotatedString] spans without exposing raw tags to players.
 */
fun parseRiotRichText(raw: String): AnnotatedString {
    if (!raw.contains('<')) return AnnotatedString(raw)

    val whiteBold = SpanStyle(color = TextPrimary, fontWeight = FontWeight.Bold)
    val debuffRed = SpanStyle(color = ValRed, fontWeight = FontWeight.Bold)
    val tealHighlight = SpanStyle(color = ValCyan, fontWeight = FontWeight.SemiBold)
    val goldHighlight = SpanStyle(color = ValGold, fontWeight = FontWeight.SemiBold)

    return buildAnnotatedString {
        val stack = ArrayDeque<SpanStyle?>()
        var idx = 0
        while (idx < raw.length) {
            val openIdx = raw.indexOf('<', startIndex = idx)
            if (openIdx < 0) {
                append(raw.substring(idx))
                break
            }
            if (openIdx > idx) {
                append(raw.substring(idx, openIdx))
            }
            val closeIdx = raw.indexOf('>', startIndex = openIdx + 1)
            if (closeIdx < 0) {
                append(raw.substring(openIdx))
                break
            }
            val tag = raw.substring(openIdx + 1, closeIdx).trim().lowercase()
            if (tag == "/" || tag.startsWith("/")) {
                val popped = if (stack.isNotEmpty()) stack.removeLast() else null
                if (popped != null) {
                    pop()
                }
            } else {
                val style = when (tag) {
                    "whitebold", "b", "strong" -> whiteBold
                    "debuffred", "warning", "red" -> debuffRed
                    "teal", "cyan", "buffteal" -> tealHighlight
                    "gold", "yellow" -> goldHighlight
                    else -> null
                }
                stack.addLast(style)
                if (style != null) {
                    pushStyle(style)
                }
            }
            idx = closeIdx + 1
        }
        while (stack.isNotEmpty()) {
            val remaining = stack.removeLast()
            if (remaining != null) {
                pop()
            }
        }
    }
}

fun formatAbilitySlot(rawSlot: String): String = when (rawSlot.trim().lowercase()) {
    "ability1" -> "Q"
    "ability2" -> "E"
    "grenade" -> "C"
    "ultimate" -> "X"
    "passive" -> "PASSIVE"
    else -> rawSlot.trim().uppercase().ifEmpty { "SKILL" }
}

private fun skinTierSortWeight(tierName: String?): Int = when (tierName) {
    "Ultra Edition", "Ultra" -> 0
    "Exclusive Edition", "Exclusive" -> 1
    "Premium Edition", "Premium" -> 2
    "Deluxe Edition", "Deluxe" -> 3
    "Select Edition", "Select" -> 4
    else -> 5
}

@Composable
fun ExplorerScreen(
    uiState: MyValUiState,
    onSyncAssets: () -> Unit,
    onOpenAbout: () -> Unit,
    onSearchSkins: (String, Boolean) -> Unit,
    onToggleWishlist: (String) -> Unit,
) {
    var section by remember { mutableStateOf(ExploreSection.SKINS) }
    var selectedWeapon by remember { mutableStateOf("All") }
    var selectedRole by remember { mutableStateOf("All") }
    var expandedAgentId by remember { mutableStateOf<String?>(null) }
    var selectedMapForPreview by remember { mutableStateOf<MapInfo?>(null) }

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
            Text(
                text = "EXPLORE",
                fontWeight = FontWeight.Black,
                fontSize = 18.sp,
                letterSpacing = 1.sp,
                color = TextPrimary,
            )
            Row(verticalAlignment = Alignment.CenterVertically) {
                if (uiState.isSyncingAssets) {
                    CircularProgressIndicator(
                        modifier = Modifier.size(20.dp),
                        color = ValCyan,
                        strokeWidth = 2.dp,
                    )
                } else {
                    IconButton(onClick = onSyncAssets, modifier = Modifier.size(36.dp)) {
                        Icon(
                            Icons.Default.Sync,
                            contentDescription = "Refresh catalog",
                            tint = ValCyan,
                            modifier = Modifier.size(20.dp),
                        )
                    }
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

        // 3 Clean Sections: Skins / Agents / Maps
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 16.dp, vertical = 8.dp),
            horizontalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            ExploreSection.entries.forEach { sec ->
                FilterChip(
                    selected = section == sec,
                    onClick = { section = sec },
                    shape = SharpChipShape,
                    label = { Text(sec.label, fontWeight = FontWeight.Bold, fontSize = 12.5.sp) },
                    colors = FilterChipDefaults.filterChipColors(
                        selectedContainerColor = ValCyan.copy(alpha = 0.2f),
                        selectedLabelColor = ValCyan,
                    ),
                )
            }
        }

        when (section) {
            ExploreSection.SKINS -> {
                val queryTrimmed = uiState.skinQuery.trim().lowercase()
                val displayedSkins = remember(
                    uiState.skins,
                    uiState.allWeaponSkins,
                    selectedWeapon,
                    queryTrimmed,
                    uiState.wishlistOnly,
                ) {
                    if (selectedWeapon == "All" && queryTrimmed.isEmpty() && !uiState.wishlistOnly) {
                        uiState.skins
                    } else if (selectedWeapon == "All") {
                        uiState.skins
                    } else {
                        val pool = if (uiState.allWeaponSkins.isNotEmpty()) {
                            uiState.allWeaponSkins
                        } else {
                            uiState.skins
                        }
                        pool.filter { skin ->
                            val matchesWeapon = skin.weaponName.equals(selectedWeapon, ignoreCase = true)
                            val matchesWish = !uiState.wishlistOnly || skin.isWishlisted
                            val matchesQuery = queryTrimmed.isEmpty() ||
                                skin.displayName.lowercase().contains(queryTrimmed)
                            val hideStandard = queryTrimmed.isEmpty() && !uiState.wishlistOnly &&
                                (skin.displayName.startsWith("Standard ") || skin.displayName.contains("Random Favorite"))
                            matchesWeapon && matchesWish && matchesQuery && !hideStandard
                        }.sortedWith(
                            compareBy<Skin> { skinTierSortWeight(it.contentTierName) }
                                .thenBy { it.displayName },
                        )
                    }
                }

                Column(modifier = Modifier.fillMaxSize()) {
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(horizontal = 16.dp, vertical = 4.dp),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(8.dp),
                    ) {
                        OutlinedTextField(
                            value = uiState.skinQuery,
                            onValueChange = { onSearchSkins(it, uiState.wishlistOnly) },
                            placeholder = { Text("Search Kuronami, Vandal, Phantom...", fontSize = 13.sp) },
                            leadingIcon = { Icon(Icons.Default.Search, contentDescription = null, tint = ValCyan) },
                            singleLine = true,
                            shape = SharpCardShape,
                            modifier = Modifier.weight(1f),
                        )
                        FilterChip(
                            selected = uiState.wishlistOnly,
                            onClick = { onSearchSkins(uiState.skinQuery, !uiState.wishlistOnly) },
                            shape = SharpChipShape,
                            label = {
                                Icon(
                                    imageVector = Icons.Default.Favorite,
                                    contentDescription = "Wishlist only",
                                    tint = if (uiState.wishlistOnly) ValRed else TextMuted,
                                    modifier = Modifier.size(18.dp),
                                )
                            },
                        )
                    }

                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .horizontalScroll(rememberScrollState())
                            .padding(horizontal = 16.dp, vertical = 6.dp),
                        horizontalArrangement = Arrangement.spacedBy(6.dp),
                    ) {
                        WEAPON_FILTERS.forEach { weapon ->
                            FilterChip(
                                selected = selectedWeapon == weapon,
                                onClick = { selectedWeapon = weapon },
                                shape = SharpChipShape,
                                label = {
                                    Text(
                                        text = weapon,
                                        fontSize = 11.5.sp,
                                        fontWeight = FontWeight.SemiBold,
                                    )
                                },
                                colors = FilterChipDefaults.filterChipColors(
                                    selectedContainerColor = ValRed.copy(alpha = 0.2f),
                                    selectedLabelColor = ValRed,
                                ),
                            )
                        }
                    }

                    LazyColumn(
                        modifier = Modifier.fillMaxSize(),
                        contentPadding = PaddingValues(16.dp),
                        verticalArrangement = Arrangement.spacedBy(10.dp),
                    ) {
                        items(displayedSkins, key = { it.uuid }) { skin ->
                            SkinRowCard(
                                skin = skin,
                                onToggleWishlist = { onToggleWishlist(skin.uuid) },
                            )
                        }
                    }
                }
            }

            ExploreSection.AGENTS -> {
                val filteredAgents = remember(uiState.agents, selectedRole) {
                    uiState.agents
                        .filter { agent ->
                            selectedRole == "All" ||
                                agent.roleName.equals(selectedRole, ignoreCase = true)
                        }
                        .sortedBy { it.displayName }
                }

                Column(modifier = Modifier.fillMaxSize()) {
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .horizontalScroll(rememberScrollState())
                            .padding(horizontal = 16.dp, vertical = 4.dp),
                        horizontalArrangement = Arrangement.spacedBy(6.dp),
                    ) {
                        ROLE_FILTERS.forEach { role ->
                            FilterChip(
                                selected = selectedRole == role,
                                onClick = { selectedRole = role },
                                shape = SharpChipShape,
                                label = {
                                    Text(
                                        text = role,
                                        fontSize = 11.5.sp,
                                        fontWeight = FontWeight.SemiBold,
                                    )
                                },
                                colors = FilterChipDefaults.filterChipColors(
                                    selectedContainerColor = ValCyan.copy(alpha = 0.2f),
                                    selectedLabelColor = ValCyan,
                                ),
                            )
                        }
                    }

                    LazyColumn(
                        modifier = Modifier.fillMaxSize(),
                        contentPadding = PaddingValues(16.dp),
                        verticalArrangement = Arrangement.spacedBy(10.dp),
                    ) {
                        items(filteredAgents, key = { it.uuid }) { agent ->
                            AgentCard(
                                agent = agent,
                                expanded = expandedAgentId == agent.uuid,
                                onClick = {
                                    expandedAgentId = if (expandedAgentId == agent.uuid) null else agent.uuid
                                },
                            )
                        }
                    }
                }
            }

            ExploreSection.MAPS -> {
                val playableMaps = remember(uiState.maps) {
                    uiState.maps
                        .filter { it.displayIcon != null && it.calloutCount > 0u }
                        .sortedBy { it.displayName }
                }

                LazyColumn(
                    modifier = Modifier.fillMaxSize(),
                    contentPadding = PaddingValues(16.dp),
                    verticalArrangement = Arrangement.spacedBy(10.dp),
                ) {
                    items(playableMaps, key = { it.uuid }) { map ->
                        MapCard(
                            map = map,
                            onClick = { selectedMapForPreview = map },
                        )
                    }
                }
            }
        }
    }

    selectedMapForPreview?.let { map ->
        AlertDialog(
            onDismissRequest = { selectedMapForPreview = null },
            containerColor = SurfaceCard,
            shape = SharpCardShape,
            title = {
                Column {
                    Text(
                        text = map.displayName.uppercase(),
                        fontWeight = FontWeight.Black,
                        fontSize = 17.sp,
                        color = TextPrimary,
                    )
                    Text(
                        text = "${map.calloutCount} callouts",
                        color = ValCyan,
                        fontSize = 12.sp,
                        fontWeight = FontWeight.SemiBold,
                    )
                }
            },
            text = {
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .aspectRatio(1f)
                        .clip(InnerCardShape)
                        .background(ObsidianBg)
                        .border(1.dp, SurfaceStroke, InnerCardShape)
                        .padding(8.dp),
                    contentAlignment = Alignment.Center,
                ) {
                    if (map.displayIcon != null) {
                        AsyncImage(
                            model = map.displayIcon,
                            contentDescription = "${map.displayName} minimap",
                            contentScale = ContentScale.Fit,
                            modifier = Modifier.fillMaxSize(),
                        )
                    }
                }
            },
            confirmButton = {
                TextButton(onClick = { selectedMapForPreview = null }) {
                    Text("Close", color = ValCyan, fontWeight = FontWeight.Bold)
                }
            },
        )
    }
}

@Composable
private fun AgentCard(agent: Agent, expanded: Boolean, onClick: () -> Unit) {
    Surface(
        color = SurfaceCard,
        shape = SharpCardShape,
        modifier = Modifier
            .fillMaxWidth()
            .border(
                width = if (expanded) 1.25.dp else 1.dp,
                color = if (expanded) ValCyan.copy(alpha = 0.55f) else SurfaceStroke,
                shape = SharpCardShape,
            )
            .clickable(onClick = onClick),
    ) {
        Column(modifier = Modifier.padding(12.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                if (agent.displayIcon != null) {
                    AsyncImage(
                        model = agent.displayIcon,
                        contentDescription = agent.displayName,
                        modifier = Modifier
                            .size(52.dp)
                            .clip(InnerCardShape)
                            .background(SurfaceElevated),
                    )
                    Spacer(modifier = Modifier.width(12.dp))
                }
                Column(modifier = Modifier.weight(1f)) {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Text(
                            text = agent.displayName,
                            fontWeight = FontWeight.Bold,
                            fontSize = 15.sp,
                            color = TextPrimary,
                        )
                        agent.roleName?.let { role ->
                            Spacer(modifier = Modifier.width(8.dp))
                            TacticalBadge(role, ValCyan)
                        }
                    }
                    Spacer(modifier = Modifier.height(2.dp))
                    Text(
                        text = agent.description,
                        color = TextSecondary,
                        fontSize = 12.sp,
                        lineHeight = 16.sp,
                        maxLines = if (expanded) Int.MAX_VALUE else 2,
                        overflow = TextOverflow.Ellipsis,
                    )
                }
                Spacer(modifier = Modifier.width(6.dp))
                Icon(
                    imageVector = if (expanded) Icons.Default.ExpandLess else Icons.Default.ExpandMore,
                    contentDescription = if (expanded) "Collapse" else "Expand abilities",
                    tint = if (expanded) ValCyan else TextMuted,
                    modifier = Modifier.size(20.dp),
                )
            }

            AnimatedVisibility(visible = expanded && agent.abilities.isNotEmpty()) {
                Column(
                    modifier = Modifier.padding(top = 10.dp),
                    verticalArrangement = Arrangement.spacedBy(8.dp),
                ) {
                    agent.abilities.forEach { ab ->
                        val slotLabel = formatAbilitySlot(ab.slot)
                        val slotAccent = if (slotLabel == "X") ValRed else if (slotLabel == "PASSIVE") ValPurple else ValCyan
                        Row(
                            modifier = Modifier
                                .fillMaxWidth()
                                .background(SurfaceElevated, InnerCardShape)
                                .padding(10.dp),
                            verticalAlignment = Alignment.Top,
                        ) {
                            if (ab.displayIcon != null) {
                                AsyncImage(
                                    model = ab.displayIcon,
                                    contentDescription = ab.displayName,
                                    modifier = Modifier
                                        .padding(top = 2.dp)
                                        .size(28.dp),
                                )
                                Spacer(modifier = Modifier.width(10.dp))
                            }
                            Column(modifier = Modifier.weight(1f)) {
                                Row(
                                    verticalAlignment = Alignment.CenterVertically,
                                    horizontalArrangement = Arrangement.spacedBy(6.dp),
                                ) {
                                    TacticalBadge(slotLabel, slotAccent)
                                    Text(
                                        text = ab.displayName,
                                        fontWeight = FontWeight.Bold,
                                        fontSize = 12.5.sp,
                                        color = TextPrimary,
                                    )
                                }
                                Spacer(modifier = Modifier.height(4.dp))
                                Text(
                                    text = parseRiotRichText(ab.description),
                                    fontSize = 11.5.sp,
                                    lineHeight = 16.sp,
                                    color = TextSecondary,
                                )
                            }
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun SkinRowCard(skin: Skin, onToggleWishlist: () -> Unit) {
    val accent = parseHexColor(skin.contentTierColor, ValCyan)
    Surface(
        color = SurfaceCard,
        shape = SharpCardShape,
        modifier = Modifier
            .fillMaxWidth()
            .border(1.dp, accent.copy(alpha = 0.45f), SharpCardShape),
    ) {
        Row(
            modifier = Modifier.padding(12.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Box(
                modifier = Modifier
                    .width(104.dp)
                    .height(48.dp)
                    .background(SurfaceElevated, InnerCardShape),
                contentAlignment = Alignment.Center,
            ) {
                if (skin.displayIcon != null) {
                    AsyncImage(
                        model = skin.displayIcon,
                        contentDescription = skin.displayName,
                        contentScale = ContentScale.Fit,
                        modifier = Modifier
                            .fillMaxSize()
                            .padding(4.dp),
                    )
                }
            }
            Spacer(modifier = Modifier.width(12.dp))
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = skin.displayName,
                    fontWeight = FontWeight.Bold,
                    fontSize = 13.5.sp,
                    color = TextPrimary,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                val tier = skin.contentTierName?.let { " • $it" } ?: ""
                Text(
                    text = "${skin.weaponName}$tier",
                    color = accent,
                    fontSize = 11.5.sp,
                )
            }
            IconButton(onClick = onToggleWishlist) {
                Icon(
                    imageVector = if (skin.isWishlisted) Icons.Default.Favorite else Icons.Default.FavoriteBorder,
                    contentDescription = "Toggle Wishlist",
                    tint = if (skin.isWishlisted) ValRed else TextMuted,
                )
            }
        }
    }
}

@Composable
private fun MapCard(map: MapInfo, onClick: () -> Unit) {
    val bannerUrl = map.listViewIcon ?: map.splash
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .height(96.dp)
            .clip(SharpCardShape)
            .background(SurfaceCard)
            .border(1.dp, SurfaceStroke, SharpCardShape)
            .clickable(onClick = onClick),
    ) {
        if (bannerUrl != null) {
            AsyncImage(
                model = bannerUrl,
                contentDescription = map.displayName,
                contentScale = ContentScale.Crop,
                modifier = Modifier.fillMaxSize(),
            )
        }
        Box(
            modifier = Modifier
                .fillMaxSize()
                .background(
                    Brush.horizontalGradient(
                        listOf(ObsidianBg.copy(alpha = 0.92f), ObsidianBg.copy(alpha = 0.42f)),
                    ),
                ),
        )
        Row(
            modifier = Modifier
                .fillMaxSize()
                .padding(14.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.SpaceBetween,
        ) {
            Column {
                Text(
                    text = map.displayName.uppercase(),
                    fontWeight = FontWeight.Black,
                    fontSize = 18.sp,
                    color = TextPrimary,
                )
                Text(
                    text = "${map.calloutCount} callouts • Tap for minimap",
                    color = TextSecondary,
                    fontSize = 11.5.sp,
                )
            }
            if (map.displayIcon != null) {
                AsyncImage(
                    model = map.displayIcon,
                    contentDescription = "Minimap",
                    modifier = Modifier.size(64.dp),
                    contentScale = ContentScale.Fit,
                )
            }
        }
    }
}
