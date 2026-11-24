package com.myval.app.ui.loadout

import androidx.compose.animation.AnimatedContent
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
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Casino
import androidx.compose.material.icons.filled.Check
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.filled.Edit
import androidx.compose.material.icons.filled.GridView
import androidx.compose.material.icons.filled.Info
import androidx.compose.material.icons.filled.Lock
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material.icons.filled.Search
import androidx.compose.material.icons.filled.ViewAgenda
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.FilterChip
import androidx.compose.material3.FilterChipDefaults
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Surface
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
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
import com.myval.app.ui.components.InnerCardShape
import com.myval.app.ui.components.LiveNetworkStatusBar
import com.myval.app.ui.components.MyValPullToRefreshBox
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
import uniffi.myval_ffi.LiveState
import uniffi.myval_ffi.LoadoutGunSlot
import uniffi.myval_ffi.LoadoutPreset
import uniffi.myval_ffi.LoadoutSpraySlot
import uniffi.myval_ffi.PlayerCard
import uniffi.myval_ffi.PlayerLoadoutSnapshot
import uniffi.myval_ffi.PlayerTitle
import uniffi.myval_ffi.SessionSummary
import uniffi.myval_ffi.Skin
import uniffi.myval_ffi.Spray

private enum class ArsenalTab(val label: String) {
    LIVE("Live"),
    PRESETS("Presets"),
    IDENTITY("Sprays & Card"),
}

private val FALLBACK_SPRAY_SLOTS: List<LoadoutSpraySlot> = listOf(
    LoadoutSpraySlot("0814b2fe-4512-60a4-5288-1fbdcec6ca48", "Pre-Round", "", "Empty", null, false),
    LoadoutSpraySlot("04af080a-4071-487b-61c0-5b9c0cfaac74", "Mid-Round", "", "Empty", null, false),
    LoadoutSpraySlot("5863985e-43ac-b05d-cb2d-139e72970014", "Post-Round", "", "Empty", null, false),
    LoadoutSpraySlot("7cdc908e-4f69-9140-a604-899bd879eed1", "Extra", "", "Empty", null, false),
)

@Composable
fun LoadoutScreen(
    uiState: MyValUiState,
    onOpenLogin: () -> Unit,
    onOpenAccounts: () -> Unit,
    onRefreshLoadout: () -> Unit,
    onEquipWeaponSkin: (String, String, String?, String?) -> Unit,
    onEquipSpray: (String, String) -> Unit,
    onEquipPlayerCard: (String) -> Unit,
    onEquipPlayerTitle: (String) -> Unit,
    onSetIdentityPrivacy: (Boolean, Boolean) -> Unit,
    onRandomizeLoadout: () -> Unit,
    onSavePreset: (String, String) -> Unit,
    onApplyPreset: (String) -> Unit,
    onDeletePreset: (String) -> Unit,
) {
    var activeTab by remember { mutableStateOf(ArsenalTab.LIVE) }
    var selectedSlotForPicker by remember { mutableStateOf<LoadoutGunSlot?>(null) }
    var selectedSpraySlot by remember { mutableStateOf<LoadoutSpraySlot?>(null) }
    var showCardPicker by remember { mutableStateOf(false) }
    var showTitlePicker by remember { mutableStateOf(false) }
    var showSavePresetDialog by remember { mutableStateOf(false) }
    var useThreeColumnGrid by remember { mutableStateOf(true) }
    var diceSpinTarget by remember { mutableFloatStateOf(0f) }
    val diceRotation by animateFloatAsState(
        targetValue = diceSpinTarget,
        animationSpec = tween(550),
        label = "diceSpin",
    )

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(ObsidianBg),
    ) {
        ArsenalHeader(
            uiState = uiState,
            activeTab = activeTab,
            useThreeColumnGrid = useThreeColumnGrid,
            onToggleGrid = { useThreeColumnGrid = !useThreeColumnGrid },
            onOpenAccounts = onOpenAccounts,
            onRefreshLoadout = onRefreshLoadout,
        )

        LiveNetworkStatusBar(
            message = uiState.networkStatusMessage,
            isSlow = uiState.isNetworkSlow,
        )

        val session = uiState.session
        if (session == null) {
            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .padding(16.dp),
                contentAlignment = Alignment.TopCenter,
            ) {
                AuthRequiredCard(
                    title = "SIGN IN TO MANAGE YOUR ARSENAL",
                    subtitle = "Connect your Riot Games account to inspect live-match equipped skins, switch curated presets, and customize your sprays, player card, and title.",
                    onOpenLogin = onOpenLogin,
                    savedAccountCount = uiState.savedAccounts.size,
                    onOpenAccounts = onOpenAccounts,
                )
            }
            return
        }

        val loadout = uiState.loadout
        if (loadout == null) {
            ArsenalEmptyOrErrorState(
                loadoutError = uiState.loadoutError,
                isLoading = uiState.isLoadingLoadout,
                onRetry = onRefreshLoadout,
            )
            return
        }

        ArsenalDivisionBar(
            activeTab = activeTab,
            onSelectTab = { activeTab = it },
        )

        MyValPullToRefreshBox(
            isRefreshing = uiState.isLoadingLoadout,
            onRefresh = onRefreshLoadout,
            accent = ValPurple,
            modifier = Modifier
                .fillMaxWidth()
                .weight(1f),
        ) {
            AnimatedContent(
                targetState = activeTab,
                transitionSpec = {
                    (fadeIn(tween(180)) + slideInVertically(tween(180)) { it / 28 })
                        .togetherWith(fadeOut(tween(120)))
                },
                label = "arsenalTabTransition",
            ) { tab ->
                when (tab) {
                    ArsenalTab.LIVE -> ArsenalLiveDivision(
                        uiState = uiState,
                        loadout = loadout,
                        useThreeColumnGrid = useThreeColumnGrid,
                        onSelectGunSlot = { selectedSlotForPicker = it },
                    )

                    ArsenalTab.PRESETS -> ArsenalPresetsDivision(
                        uiState = uiState,
                        diceRotation = diceRotation,
                        onOpenSavePreset = { showSavePresetDialog = true },
                        onRandomize = {
                            diceSpinTarget += 360f
                            onRandomizeLoadout()
                        },
                        onApplyPreset = onApplyPreset,
                        onDeletePreset = onDeletePreset,
                    )

                    ArsenalTab.IDENTITY -> ArsenalIdentityDivision(
                        session = session,
                        loadout = loadout,
                        isLoading = uiState.isLoadingLoadout,
                        onEditCard = { showCardPicker = true },
                        onEditTitle = { showTitlePicker = true },
                        onSelectSpraySlot = { selectedSpraySlot = it },
                        onSetIdentityPrivacy = onSetIdentityPrivacy,
                    )
                }
            }
        }
    }

    selectedSlotForPicker?.let { slot ->
        WeaponSkinPickerDialog(
            slot = slot,
            allSkins = uiState.allSkinsForPicker(slot.weaponName),
            ownedSkinUuids = uiState.ownedSkinUuids,
            ownedChromaUuids = uiState.ownedChromaUuids,
            ownedLevelUuids = uiState.ownedLevelUuids,
            onDismiss = { selectedSlotForPicker = null },
            onSelectSkin = { skin, chromaUuid, levelUuid ->
                selectedSlotForPicker = null
                onEquipWeaponSkin(slot.weaponUuid, skin.uuid, chromaUuid, levelUuid)
            },
        )
    }

    selectedSpraySlot?.let { spraySlot ->
        SprayPickerDialog(
            slot = spraySlot,
            allSprays = uiState.sprays,
            ownedSprayUuids = uiState.ownedSprayUuids,
            onDismiss = { selectedSpraySlot = null },
            onSelectSpray = { sprayUuid ->
                selectedSpraySlot = null
                onEquipSpray(spraySlot.slotUuid, sprayUuid)
            },
        )
    }

    if (showCardPicker && uiState.loadout != null) {
        PlayerCardPickerDialog(
            currentCardUuid = uiState.loadout.playerCardUuid,
            allCards = uiState.playerCards,
            ownedCardUuids = uiState.ownedCardUuids,
            onDismiss = { showCardPicker = false },
            onSelectCard = { cardUuid ->
                showCardPicker = false
                onEquipPlayerCard(cardUuid)
            },
        )
    }

    if (showTitlePicker && uiState.loadout != null) {
        PlayerTitlePickerDialog(
            currentTitleUuid = uiState.loadout.playerTitleUuid,
            allTitles = uiState.playerTitles,
            ownedTitleUuids = uiState.ownedTitleUuids,
            onDismiss = { showTitlePicker = false },
            onSelectTitle = { titleUuid ->
                showTitlePicker = false
                onEquipPlayerTitle(titleUuid)
            },
        )
    }

    if (showSavePresetDialog) {
        SavePresetDialog(
            onDismiss = { showSavePresetDialog = false },
            onSave = { name, colorHex ->
                showSavePresetDialog = false
                onSavePreset(name, colorHex)
            },
        )
    }
}

@Composable
private fun ArsenalHeader(
    uiState: MyValUiState,
    activeTab: ArsenalTab,
    useThreeColumnGrid: Boolean,
    onToggleGrid: () -> Unit,
    onOpenAccounts: () -> Unit,
    onRefreshLoadout: () -> Unit,
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
                text = "ARSENAL",
                fontWeight = FontWeight.Black,
                fontSize = 18.sp,
                letterSpacing = 1.sp,
                maxLines = 1,
                softWrap = false,
            )
            Spacer(modifier = Modifier.height(4.dp))
            uiState.session?.let { sessionHeader ->
                AccountSwitcherChip(
                    riotId = sessionHeader.riotId,
                    shard = sessionHeader.shard,
                    accountCount = uiState.savedAccounts.size,
                    onClick = onOpenAccounts,
                )
            }
        }
        if (uiState.session != null && activeTab == ArsenalTab.LIVE) {
            IconButton(onClick = onToggleGrid, modifier = Modifier.size(34.dp)) {
                Icon(
                    imageVector = if (useThreeColumnGrid) Icons.Default.GridView else Icons.Default.ViewAgenda,
                    contentDescription = "Toggle 2 or 3 column grid",
                    tint = if (useThreeColumnGrid) ValCyan else TextSecondary,
                    modifier = Modifier.size(18.dp),
                )
            }
        }
    }
}

@Composable
private fun ArsenalDivisionBar(
    activeTab: ArsenalTab,
    onSelectTab: (ArsenalTab) -> Unit,
) {
    Surface(
        color = SurfaceCard,
        modifier = Modifier.fillMaxWidth(),
    ) {
        Column {
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 16.dp, vertical = 8.dp)
                    .background(ObsidianBg, RoundedCornerShape(10.dp))
                    .border(1.dp, SurfaceStroke, RoundedCornerShape(10.dp))
                    .padding(4.dp),
                horizontalArrangement = Arrangement.spacedBy(4.dp),
            ) {
                ArsenalTab.entries.forEach { tab ->
                    val selected = tab == activeTab
                    val accent = when (tab) {
                        ArsenalTab.LIVE -> ValGreen
                        ArsenalTab.PRESETS -> ValPurple
                        ArsenalTab.IDENTITY -> ValCyan
                    }
                    Box(
                        modifier = Modifier
                            .weight(1f)
                            .clip(RoundedCornerShape(7.dp))
                            .background(if (selected) accent.copy(alpha = 0.18f) else Color.Transparent)
                            .then(
                                if (selected) {
                                    Modifier.border(1.dp, accent.copy(alpha = 0.65f), RoundedCornerShape(7.dp))
                                } else {
                                    Modifier
                                },
                            )
                            .tactileClickable(pressedScale = 0.96f) { onSelectTab(tab) }
                            .padding(vertical = 8.dp, horizontal = 4.dp),
                        contentAlignment = Alignment.Center,
                    ) {
                        Text(
                            text = tab.label,
                            color = if (selected) accent else TextSecondary,
                            fontWeight = if (selected) FontWeight.Black else FontWeight.SemiBold,
                            fontSize = 12.sp,
                            maxLines = 1,
                            softWrap = false,
                            overflow = TextOverflow.Ellipsis,
                        )
                    }
                }
            }
            HorizontalDivider(color = SurfaceStroke, thickness = 1.dp)
        }
    }
}

@Composable
private fun ArsenalEmptyOrErrorState(
    loadoutError: String?,
    isLoading: Boolean,
    onRetry: () -> Unit,
) {
    Box(
        modifier = Modifier
            .fillMaxSize()
            .padding(24.dp),
        contentAlignment = Alignment.Center,
    ) {
        if (loadoutError != null && !isLoading) {
            Column(
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.spacedBy(12.dp),
            ) {
                Text(
                    text = "Failed to load player loadout",
                    color = TextPrimary,
                    fontWeight = FontWeight.Bold,
                    fontSize = 15.sp,
                )
                Text(text = loadoutError, color = TextSecondary, fontSize = 12.sp)
                Button(
                    onClick = onRetry,
                    colors = ButtonDefaults.buttonColors(containerColor = ValRed),
                ) {
                    Text("Retry", fontWeight = FontWeight.Bold)
                }
            }
        } else {
            CircularProgressIndicator(color = ValPurple)
        }
    }
}

@Composable
private fun ArsenalLiveDivision(
    uiState: MyValUiState,
    loadout: PlayerLoadoutSnapshot,
    useThreeColumnGrid: Boolean,
    onSelectGunSlot: (LoadoutGunSlot) -> Unit,
) {
    LazyColumn(
        modifier = Modifier.fillMaxSize(),
        contentPadding = PaddingValues(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        if (loadout.isLiveMatchSynced) {
            item {
                ArsenalStatusBanner(
                    title = "LIVE MATCH EQUIPPED",
                    body = "Showing the active skins locked in your match. Any swap saves to Riot and applies next match.",
                    accent = ValGreen,
                    icon = Icons.Default.CheckCircle,
                )
            }
        } else {
            when (uiState.liveState) {
                is LiveState.CoreGame -> item {
                    ArsenalStatusBanner(
                        title = "IN LIVE MATCH",
                        body = "Riot locks live-match skins. Swaps save to Riot and apply next match.",
                        accent = ValAmber,
                        icon = Icons.Default.Lock,
                    )
                }
                is LiveState.PreGame -> item {
                    ArsenalStatusBanner(
                        title = "AGENT SELECT",
                        body = "Swaps sync to Riot before match lock.",
                        accent = ValCyan,
                        icon = Icons.Default.Info,
                    )
                }
                else -> Unit
            }
        }

        item {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(
                    text = "EQUIPPED WEAPONS (${loadout.guns.size})",
                    fontWeight = FontWeight.Black,
                    fontSize = 12.sp,
                    color = TextSecondary,
                    letterSpacing = 1.sp,
                )
                Text(
                    text = "Tap any gun to customize",
                    color = TextMuted,
                    fontSize = 11.sp,
                )
            }
        }

        val colCount = if (useThreeColumnGrid) 3 else 2
        val chunks = loadout.guns.chunked(colCount)
        items(chunks, key = { chunk -> chunk.joinToString("-") { it.weaponUuid } }) { rowSlots ->
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                rowSlots.forEach { slot ->
                    WeaponLoadoutCard(
                        slot = slot,
                        compact = useThreeColumnGrid,
                        modifier = Modifier.weight(1f),
                        onClick = { onSelectGunSlot(slot) },
                    )
                }
                repeat(colCount - rowSlots.size) {
                    Spacer(modifier = Modifier.weight(1f))
                }
            }
        }
    }
}

@Composable
private fun ArsenalStatusBanner(
    title: String,
    body: String,
    accent: Color,
    icon: androidx.compose.ui.graphics.vector.ImageVector,
) {
    Surface(
        color = accent.copy(alpha = 0.13f),
        shape = InnerCardShape,
        modifier = Modifier
            .fillMaxWidth()
            .border(1.dp, accent.copy(alpha = 0.55f), InnerCardShape),
    ) {
        Row(
            modifier = Modifier.padding(horizontal = 12.dp, vertical = 10.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Icon(
                imageVector = icon,
                contentDescription = null,
                tint = accent,
                modifier = Modifier.size(18.dp),
            )
            Spacer(modifier = Modifier.width(10.dp))
            Column {
                Text(
                    text = title,
                    color = accent,
                    fontSize = 11.sp,
                    fontWeight = FontWeight.Black,
                    letterSpacing = 0.6.sp,
                )
                Text(
                    text = body,
                    color = TextPrimary.copy(alpha = 0.9f),
                    fontSize = 11.5.sp,
                )
            }
        }
    }
}

@Composable
private fun ArsenalPresetsDivision(
    uiState: MyValUiState,
    diceRotation: Float,
    onOpenSavePreset: () -> Unit,
    onRandomize: () -> Unit,
    onApplyPreset: (String) -> Unit,
    onDeletePreset: (String) -> Unit,
) {
    LazyColumn(
        modifier = Modifier.fillMaxSize(),
        contentPadding = PaddingValues(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        item {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(10.dp),
            ) {
                Button(
                    onClick = onOpenSavePreset,
                    enabled = !uiState.isLoadingLoadout,
                    colors = ButtonDefaults.buttonColors(
                        containerColor = ValCyan,
                        contentColor = ObsidianBg,
                    ),
                    modifier = Modifier.weight(1f),
                    contentPadding = PaddingValues(horizontal = 10.dp, vertical = 10.dp),
                ) {
                    Icon(Icons.Default.Add, contentDescription = null, modifier = Modifier.size(16.dp))
                    Spacer(modifier = Modifier.width(6.dp))
                    Text(
                        text = "Save Current Loadout",
                        fontWeight = FontWeight.Black,
                        fontSize = 11.5.sp,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                }

                Button(
                    onClick = onRandomize,
                    enabled = !uiState.isLoadingLoadout && uiState.ownedSkinUuids.isNotEmpty(),
                    colors = ButtonDefaults.buttonColors(
                        containerColor = ValPurple,
                        contentColor = ObsidianBg,
                        disabledContainerColor = SurfaceElevated,
                        disabledContentColor = TextMuted,
                    ),
                    modifier = Modifier.weight(1f),
                    contentPadding = PaddingValues(horizontal = 10.dp, vertical = 10.dp),
                ) {
                    Icon(
                        Icons.Default.Casino,
                        contentDescription = null,
                        modifier = Modifier
                            .size(16.dp)
                            .rotate(diceRotation),
                    )
                    Spacer(modifier = Modifier.width(6.dp))
                    Text(
                        text = "Randomize Owned",
                        fontWeight = FontWeight.Black,
                        fontSize = 11.5.sp,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                }
            }
        }

        item {
            Text(
                text = "SAVED PRESETS (${uiState.loadoutPresets.size})",
                fontWeight = FontWeight.Black,
                fontSize = 12.sp,
                color = TextSecondary,
                letterSpacing = 1.sp,
            )
        }

        if (uiState.loadoutPresets.isEmpty()) {
            item {
                Surface(
                    color = SurfaceCard,
                    shape = RoundedCornerShape(10.dp),
                    modifier = Modifier
                        .fillMaxWidth()
                        .border(1.dp, SurfaceStroke, RoundedCornerShape(10.dp)),
                ) {
                    Column(
                        modifier = Modifier.padding(18.dp),
                        verticalArrangement = Arrangement.spacedBy(6.dp),
                    ) {
                        Text(
                            text = "No saved presets yet",
                            color = TextPrimary,
                            fontWeight = FontWeight.Bold,
                            fontSize = 15.sp,
                        )
                        Text(
                            text = "Build a loadout you love in Live, then tap Save Current Loadout to snapshot all 21 weapons, variants, buddies, sprays, and card for instant 1-tap switching.",
                            color = TextSecondary,
                            fontSize = 12.sp,
                        )
                    }
                }
            }
        } else {
            items(uiState.loadoutPresets, key = { it.presetId }) { preset ->
                PresetFullCard(
                    preset = preset,
                    enabled = !uiState.isLoadingLoadout,
                    onEquipAll = { onApplyPreset(preset.presetId) },
                    onDelete = { onDeletePreset(preset.presetId) },
                )
            }
        }
    }
}

@Composable
private fun PresetFullCard(
    preset: LoadoutPreset,
    enabled: Boolean,
    onEquipAll: () -> Unit,
    onDelete: () -> Unit,
) {
    val accent = parseHexColor(preset.accentColorHex, ValPurple)
    Surface(
        color = SurfaceCard,
        shape = RoundedCornerShape(10.dp),
        modifier = Modifier
            .fillMaxWidth()
            .border(1.dp, accent.copy(alpha = 0.5f), RoundedCornerShape(10.dp)),
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .height(IntrinsicSize.Min)
                .background(Brush.horizontalGradient(listOf(accent.copy(alpha = 0.14f), SurfaceCard))),
        ) {
            Box(
                modifier = Modifier
                    .width(5.dp)
                    .fillMaxHeight()
                    .background(accent),
            )
            Column(
                modifier = Modifier
                    .weight(1f)
                    .padding(14.dp),
                verticalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Column(modifier = Modifier.weight(1f)) {
                        Text(
                            text = preset.name,
                            fontWeight = FontWeight.Black,
                            fontSize = 15.sp,
                            color = TextPrimary,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                        )
                        Text(
                            text = "${preset.weaponCount} weapons saved",
                            color = accent,
                            fontSize = 11.sp,
                            fontWeight = FontWeight.Bold,
                        )
                    }
                    IconButton(onClick = onDelete, modifier = Modifier.size(28.dp)) {
                        Icon(
                            Icons.Default.Delete,
                            contentDescription = "Delete preset",
                            tint = TextMuted,
                            modifier = Modifier.size(18.dp),
                        )
                    }
                }
                Text(
                    text = preset.previewSkins.joinToString(" • ").ifEmpty { "Full Custom Loadout" },
                    color = TextSecondary,
                    fontSize = 11.5.sp,
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis,
                )
                Button(
                    onClick = onEquipAll,
                    enabled = enabled,
                    colors = ButtonDefaults.buttonColors(
                        containerColor = accent,
                        contentColor = ObsidianBg,
                    ),
                    modifier = Modifier.fillMaxWidth(),
                    contentPadding = PaddingValues(vertical = 8.dp),
                ) {
                    Text(
                        text = "EQUIP ALL",
                        fontWeight = FontWeight.Black,
                        fontSize = 12.sp,
                        letterSpacing = 0.8.sp,
                    )
                }
            }
        }
    }
}

@Composable
private fun ArsenalIdentityDivision(
    session: SessionSummary,
    loadout: PlayerLoadoutSnapshot,
    isLoading: Boolean,
    onEditCard: () -> Unit,
    onEditTitle: () -> Unit,
    onSelectSpraySlot: (LoadoutSpraySlot) -> Unit,
    onSetIdentityPrivacy: (Boolean, Boolean) -> Unit,
) {
    val spraySlots = loadout.sprays.ifEmpty { FALLBACK_SPRAY_SLOTS }
    LazyColumn(
        modifier = Modifier.fillMaxSize(),
        contentPadding = PaddingValues(16.dp),
        verticalArrangement = Arrangement.spacedBy(14.dp),
    ) {
        // Editable Player Card & Title Card
        item {
            Surface(
                color = SurfaceCard,
                shape = RoundedCornerShape(10.dp),
                modifier = Modifier
                    .fillMaxWidth()
                    .border(1.dp, SurfaceStroke, RoundedCornerShape(10.dp)),
            ) {
                Column {
                    Box(
                        modifier = Modifier
                            .fillMaxWidth()
                            .height(104.dp)
                            .clip(RoundedCornerShape(topStart = 10.dp, topEnd = 10.dp)),
                    ) {
                        if (loadout.playerCardArt != null) {
                            AsyncImage(
                                model = loadout.playerCardArt,
                                contentDescription = loadout.playerCardName,
                                contentScale = ContentScale.Crop,
                                modifier = Modifier.fillMaxSize(),
                            )
                        }
                        Box(
                            modifier = Modifier
                                .fillMaxSize()
                                .background(
                                    Brush.horizontalGradient(
                                        listOf(ObsidianBg.copy(alpha = 0.94f), ObsidianBg.copy(alpha = 0.42f)),
                                    ),
                                ),
                        )
                        Column(
                            modifier = Modifier
                                .fillMaxSize()
                                .padding(horizontal = 16.dp, vertical = 14.dp),
                            verticalArrangement = Arrangement.Center,
                        ) {
                            Text(
                                text = session.riotId,
                                fontWeight = FontWeight.Black,
                                fontSize = 18.sp,
                                color = TextPrimary,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis,
                            )
                            Spacer(modifier = Modifier.height(2.dp))
                            Text(
                                text = loadout.playerTitleText.ifBlank { "No Title Equipped" },
                                color = ValCyan,
                                fontSize = 12.sp,
                                fontWeight = FontWeight.Bold,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis,
                            )
                            Text(
                                text = loadout.playerCardName,
                                color = TextSecondary,
                                fontSize = 11.sp,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis,
                            )
                        }
                    }
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(12.dp),
                        horizontalArrangement = Arrangement.spacedBy(10.dp),
                    ) {
                        OutlinedButton(
                            onClick = onEditCard,
                            enabled = !isLoading,
                            modifier = Modifier.weight(1f),
                        ) {
                            Icon(Icons.Default.Edit, contentDescription = null, modifier = Modifier.size(15.dp), tint = ValCyan)
                            Spacer(modifier = Modifier.width(6.dp))
                            Text("Change Card", color = ValCyan, fontSize = 12.sp, fontWeight = FontWeight.Bold)
                        }
                        OutlinedButton(
                            onClick = onEditTitle,
                            enabled = !isLoading,
                            modifier = Modifier.weight(1f),
                        ) {
                            Icon(Icons.Default.Edit, contentDescription = null, modifier = Modifier.size(15.dp), tint = ValGold)
                            Spacer(modifier = Modifier.width(6.dp))
                            Text("Change Title", color = ValGold, fontSize = 12.sp, fontWeight = FontWeight.Bold)
                        }
                    }
                }
            }
        }

        // Interactive 4-Slot Spray Wheel + 5th Flex Slot
        item {
            SprayWheelCard(
                sprays = spraySlots,
                onSelectSlot = onSelectSpraySlot,
            )
        }

        // Privacy & Identity Switches
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
                    verticalArrangement = Arrangement.spacedBy(10.dp),
                ) {
                    Text(
                        text = "PRIVACY & IDENTITY",
                        color = TextSecondary,
                        fontSize = 11.5.sp,
                        fontWeight = FontWeight.Black,
                        letterSpacing = 1.sp,
                    )
                    PrivacySwitchRow(
                        title = "Streamer Mode (Hide Name)",
                        subtitle = "Masks your Riot ID during queue, agent select, and scoreboard",
                        checked = loadout.incognito,
                        enabled = !isLoading,
                        onCheckedChange = { checked ->
                            onSetIdentityPrivacy(checked, loadout.hideAccountLevel)
                        },
                    )
                    HorizontalDivider(color = SurfaceStroke, thickness = 1.dp)
                    PrivacySwitchRow(
                        title = "Hide Account Level",
                        subtitle = "Hides your account level border from teammates and opponents",
                        checked = loadout.hideAccountLevel,
                        enabled = !isLoading,
                        onCheckedChange = { checked ->
                            onSetIdentityPrivacy(loadout.incognito, checked)
                        },
                    )
                }
            }
        }
    }
}

@Composable
private fun PrivacySwitchRow(
    title: String,
    subtitle: String,
    checked: Boolean,
    enabled: Boolean,
    onCheckedChange: (Boolean) -> Unit,
) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.SpaceBetween,
    ) {
        Column(modifier = Modifier.weight(1f).padding(end = 12.dp)) {
            Text(
                text = title,
                color = TextPrimary,
                fontWeight = FontWeight.Bold,
                fontSize = 13.5.sp,
            )
            Text(
                text = subtitle,
                color = TextMuted,
                fontSize = 11.sp,
            )
        }
        Switch(
            checked = checked,
            enabled = enabled,
            onCheckedChange = onCheckedChange,
            colors = SwitchDefaults.colors(
                checkedThumbColor = ObsidianBg,
                checkedTrackColor = ValCyan,
                uncheckedThumbColor = TextSecondary,
                uncheckedTrackColor = SurfaceElevated,
            ),
        )
    }
}

@Composable
private fun SprayWheelCard(
    sprays: List<LoadoutSpraySlot>,
    onSelectSlot: (LoadoutSpraySlot) -> Unit,
) {
    val wheelSlots = sprays.take(4)
    val extraSlots = sprays.drop(4)
    val topSpray = wheelSlots.getOrNull(0)
    val leftSpray = wheelSlots.getOrNull(1)
    val rightSpray = wheelSlots.getOrNull(2)
    val bottomSpray = wheelSlots.getOrNull(3)

    Surface(
        color = SurfaceCard,
        shape = RoundedCornerShape(10.dp),
        modifier = Modifier
            .fillMaxWidth()
            .border(1.dp, SurfaceStroke, RoundedCornerShape(10.dp)),
    ) {
        Column(modifier = Modifier.padding(12.dp)) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(
                    text = "SPRAY WHEEL",
                    color = TextSecondary,
                    fontSize = 11.5.sp,
                    fontWeight = FontWeight.Black,
                    letterSpacing = 1.sp,
                )
                Text(
                    text = "Tap any slot to swap",
                    color = ValCyan,
                    fontSize = 10.5.sp,
                    fontWeight = FontWeight.Bold,
                )
            }
            Spacer(modifier = Modifier.height(8.dp))
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(190.dp),
                contentAlignment = Alignment.Center,
            ) {
                Canvas(modifier = Modifier.fillMaxSize()) {
                    val cx = size.width / 2f
                    val cy = size.height / 2f
                    val r = 24.dp.toPx()
                    val armX = size.width * 0.25f
                    val armY = size.height * 0.34f
                    val lineColor = ValRed.copy(alpha = 0.42f)
                    val stroke = 1.3.dp.toPx()
                    drawLine(lineColor, Offset(cx - armX, cy - armY), Offset(cx + armX, cy + armY), strokeWidth = stroke)
                    drawLine(lineColor, Offset(cx + armX, cy - armY), Offset(cx - armX, cy + armY), strokeWidth = stroke)
                    drawCircle(color = ObsidianBg, radius = r, center = Offset(cx, cy))
                    drawCircle(
                        color = ValRed.copy(alpha = 0.7f),
                        radius = r,
                        center = Offset(cx, cy),
                        style = Stroke(width = 1.5.dp.toPx()),
                    )
                }

                topSpray?.let { sp ->
                    SpraySlotNode(
                        spray = sp,
                        onClick = { onSelectSlot(sp) },
                        modifier = Modifier.align(Alignment.TopCenter),
                    )
                }
                leftSpray?.let { sp ->
                    SpraySlotNode(
                        spray = sp,
                        onClick = { onSelectSlot(sp) },
                        modifier = Modifier
                            .align(Alignment.CenterStart)
                            .padding(start = 14.dp),
                    )
                }
                rightSpray?.let { sp ->
                    SpraySlotNode(
                        spray = sp,
                        onClick = { onSelectSlot(sp) },
                        modifier = Modifier
                            .align(Alignment.CenterEnd)
                            .padding(end = 14.dp),
                    )
                }
                bottomSpray?.let { sp ->
                    SpraySlotNode(
                        spray = sp,
                        onClick = { onSelectSlot(sp) },
                        modifier = Modifier.align(Alignment.BottomCenter),
                    )
                }
            }

            if (extraSlots.isNotEmpty()) {
                Spacer(modifier = Modifier.height(8.dp))
                HorizontalDivider(color = SurfaceStroke, thickness = 1.dp)
                Spacer(modifier = Modifier.height(8.dp))
                extraSlots.forEach { slot ->
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .clip(RoundedCornerShape(8.dp))
                            .background(SurfaceElevated)
                            .border(1.dp, ValPurple.copy(alpha = 0.45f), RoundedCornerShape(8.dp))
                            .clickable { onSelectSlot(slot) }
                            .padding(horizontal = 10.dp, vertical = 8.dp),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Box(
                            modifier = Modifier
                                .size(34.dp)
                                .clip(RoundedCornerShape(6.dp))
                                .background(ObsidianBg),
                            contentAlignment = Alignment.Center,
                        ) {
                            if (slot.sprayIcon != null) {
                                AsyncImage(
                                    model = slot.sprayIcon,
                                    contentDescription = slot.sprayName,
                                    contentScale = ContentScale.Fit,
                                    modifier = Modifier.fillMaxSize().padding(3.dp),
                                )
                            } else {
                                Icon(Icons.Default.Add, contentDescription = null, tint = TextMuted, modifier = Modifier.size(16.dp))
                            }
                        }
                        Spacer(modifier = Modifier.width(10.dp))
                        Column(modifier = Modifier.weight(1f)) {
                            Text(
                                text = slot.sprayName,
                                color = TextPrimary,
                                fontWeight = FontWeight.Bold,
                                fontSize = 12.5.sp,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis,
                            )
                            Text(
                                text = slot.slotLabel,
                                color = TextMuted,
                                fontSize = 10.5.sp,
                            )
                        }
                        if (slot.isFlex) {
                            TacticalBadge("FLEX", ValPurple)
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun SpraySlotNode(
    spray: LoadoutSpraySlot,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        modifier = modifier
            .width(82.dp)
            .clip(RoundedCornerShape(8.dp))
            .clickable(onClick = onClick)
            .padding(vertical = 2.dp),
    ) {
        Box(
            modifier = Modifier
                .size(50.dp)
                .clip(RoundedCornerShape(8.dp))
                .background(SurfaceElevated.copy(alpha = 0.72f))
                .border(
                    width = 1.dp,
                    color = if (spray.isFlex) ValPurple.copy(alpha = 0.65f) else SurfaceStroke,
                    shape = RoundedCornerShape(8.dp),
                ),
            contentAlignment = Alignment.Center,
        ) {
            if (spray.sprayIcon != null) {
                AsyncImage(
                    model = spray.sprayIcon,
                    contentDescription = spray.sprayName,
                    contentScale = ContentScale.Fit,
                    modifier = Modifier
                        .fillMaxSize()
                        .padding(4.dp),
                )
            } else {
                Icon(
                    Icons.Default.Add,
                    contentDescription = "Equip Spray",
                    tint = TextMuted,
                    modifier = Modifier.size(18.dp),
                )
            }
            if (spray.isFlex) {
                Text(
                    text = "FLEX",
                    color = ObsidianBg,
                    fontSize = 7.5.sp,
                    fontWeight = FontWeight.Black,
                    modifier = Modifier
                        .align(Alignment.TopEnd)
                        .background(ValPurple, RoundedCornerShape(bottomStart = 5.dp))
                        .padding(horizontal = 3.dp, vertical = 1.dp),
                )
            }
        }
        Text(
            text = if (spray.sprayName == "Empty") spray.slotLabel else spray.sprayName,
            color = if (spray.sprayName == "Empty") TextMuted else TextSecondary,
            fontSize = 9.sp,
            fontWeight = FontWeight.SemiBold,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            textAlign = TextAlign.Center,
        )
    }
}

@Composable
private fun WeaponLoadoutCard(
    slot: LoadoutGunSlot,
    compact: Boolean,
    modifier: Modifier = Modifier,
    onClick: () -> Unit,
) {
    val accent = parseHexColor(slot.tierColor, ValCyan)
    Surface(
        color = SurfaceCard,
        shape = RoundedCornerShape(10.dp),
        modifier = modifier
            .border(1.dp, accent.copy(alpha = 0.5f), RoundedCornerShape(10.dp))
            .clickable(onClick = onClick),
    ) {
        if (compact) {
            Column(
                modifier = Modifier
                    .background(Brush.verticalGradient(listOf(accent.copy(alpha = 0.14f), SurfaceCard)))
                    .padding(horizontal = 8.dp, vertical = 9.dp),
                horizontalAlignment = Alignment.CenterHorizontally,
            ) {
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(44.dp),
                    contentAlignment = Alignment.Center,
                ) {
                    if (slot.skinIcon != null) {
                        AsyncImage(
                            model = slot.skinIcon,
                            contentDescription = slot.skinName,
                            contentScale = ContentScale.Fit,
                            modifier = Modifier.fillMaxSize(),
                        )
                    }
                    if (slot.isLiveMatchSkin) {
                        Box(
                            modifier = Modifier
                                .align(Alignment.TopEnd)
                                .size(7.dp)
                                .background(ValGreen, CircleShape),
                        )
                    }
                    slot.buddyIcon?.let { bIcon ->
                        Box(
                            modifier = Modifier
                                .align(Alignment.BottomEnd)
                                .size(17.dp)
                                .clip(RoundedCornerShape(4.dp))
                                .background(ObsidianBg.copy(alpha = 0.85f))
                                .border(0.8.dp, SurfaceStroke, RoundedCornerShape(4.dp))
                                .padding(1.5.dp),
                        ) {
                            AsyncImage(
                                model = bIcon,
                                contentDescription = slot.buddyName,
                                contentScale = ContentScale.Fit,
                                modifier = Modifier.fillMaxSize(),
                            )
                        }
                    }
                }
                Spacer(modifier = Modifier.height(5.dp))
                Text(
                    text = slot.weaponName,
                    color = accent,
                    fontWeight = FontWeight.Black,
                    fontSize = 11.sp,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                Text(
                    text = slot.skinName,
                    color = TextSecondary,
                    fontWeight = FontWeight.SemiBold,
                    fontSize = 9.5.sp,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                slot.nextMatchSkinName?.let { nextName ->
                    Text(
                        text = "Next: $nextName",
                        color = ValAmber,
                        fontSize = 8.sp,
                        fontWeight = FontWeight.Bold,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                }
            }
        } else {
            Column(
                modifier = Modifier
                    .background(Brush.verticalGradient(listOf(accent.copy(alpha = 0.14f), SurfaceCard)))
                    .padding(11.dp),
            ) {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(5.dp)) {
                        Text(
                            text = slot.weaponName.uppercase(),
                            color = accent,
                            fontWeight = FontWeight.Black,
                            fontSize = 11.sp,
                        )
                        if (slot.isLiveMatchSkin) {
                            Box(
                                modifier = Modifier
                                    .size(7.dp)
                                    .background(ValGreen, CircleShape),
                            )
                        }
                    }
                    Text("SWAP", color = TextMuted, fontSize = 10.sp, fontWeight = FontWeight.Bold)
                }
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(60.dp),
                    contentAlignment = Alignment.Center,
                ) {
                    if (slot.skinIcon != null) {
                        AsyncImage(
                            model = slot.skinIcon,
                            contentDescription = slot.skinName,
                            contentScale = ContentScale.Fit,
                            modifier = Modifier.fillMaxSize(),
                        )
                    }
                    slot.buddyIcon?.let { bIcon ->
                        Box(
                            modifier = Modifier
                                .align(Alignment.BottomEnd)
                                .size(22.dp)
                                .clip(RoundedCornerShape(5.dp))
                                .background(ObsidianBg.copy(alpha = 0.85f))
                                .border(0.8.dp, SurfaceStroke, RoundedCornerShape(5.dp))
                                .padding(2.dp),
                        ) {
                            AsyncImage(
                                model = bIcon,
                                contentDescription = slot.buddyName,
                                contentScale = ContentScale.Fit,
                                modifier = Modifier.fillMaxSize(),
                            )
                        }
                    }
                }
                Spacer(modifier = Modifier.height(6.dp))
                Text(
                    text = slot.skinName,
                    color = TextPrimary,
                    fontWeight = FontWeight.Bold,
                    fontSize = 12.sp,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                slot.nextMatchSkinName?.let { nextName ->
                    Spacer(modifier = Modifier.height(3.dp))
                    Text(
                        text = "Next match: $nextName",
                        color = ValAmber,
                        fontSize = 9.5.sp,
                        fontWeight = FontWeight.Bold,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                }
            }
        }
    }
}

@Composable
private fun WeaponSkinPickerDialog(
    slot: LoadoutGunSlot,
    allSkins: List<Skin>,
    ownedSkinUuids: Set<String>,
    ownedChromaUuids: Set<String>,
    ownedLevelUuids: Set<String>,
    onDismiss: () -> Unit,
    onSelectSkin: (Skin, String?, String?) -> Unit,
) {
    val ownedSkins = remember(allSkins, ownedSkinUuids, slot.skinUuid) {
        allSkins.filter { skin ->
            ownedSkinUuids.contains(skin.uuid.lowercase()) ||
                skin.uuid.equals(slot.skinUuid, ignoreCase = true) ||
                skin.displayName.startsWith("Standard ", ignoreCase = true)
        }
    }
    var showOwnedOnly by remember(ownedSkins.size) { mutableStateOf(ownedSkins.isNotEmpty()) }
    val rawVisible = if (showOwnedOnly && ownedSkins.isNotEmpty()) ownedSkins else allSkins
    val visibleSkins = remember(rawVisible, slot.skinUuid) {
        rawVisible.sortedByDescending { it.uuid.equals(slot.skinUuid, ignoreCase = true) }
    }

    val activeSkin = remember(allSkins, slot.skinUuid) {
        allSkins.firstOrNull { it.uuid.equals(slot.skinUuid, ignoreCase = true) }
    }
    val activeChromaName = remember(activeSkin, slot.chromaUuid) {
        activeSkin?.chromas?.firstOrNull { it.uuid.equals(slot.chromaUuid, ignoreCase = true) }
            ?.displayName
            ?.substringAfterLast("(")
            ?.removeSuffix(")")
            ?.trim()
    }
    val activeLevelIdx = remember(activeSkin, slot.levelUuid) {
        activeSkin?.levels?.indexOfFirst { it.uuid.equals(slot.levelUuid, ignoreCase = true) } ?: -1
    }
    val slotAccent = parseHexColor(slot.tierColor, ValCyan)

    AlertDialog(
        onDismissRequest = onDismiss,
        containerColor = SurfaceCard,
        title = {
            Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                Text("Customize ${slot.weaponName}", fontWeight = FontWeight.Black, fontSize = 17.sp)
                // Currently Equipped Hero Summary
                Surface(
                    color = ObsidianBg,
                    shape = RoundedCornerShape(8.dp),
                    modifier = Modifier
                        .fillMaxWidth()
                        .border(1.dp, slotAccent.copy(alpha = 0.55f), RoundedCornerShape(8.dp)),
                ) {
                    Column(
                        modifier = Modifier.padding(10.dp),
                        verticalArrangement = Arrangement.spacedBy(4.dp),
                    ) {
                        Row(verticalAlignment = Alignment.CenterVertically) {
                            if (slot.skinIcon != null) {
                                AsyncImage(
                                    model = slot.skinIcon,
                                    contentDescription = slot.skinName,
                                    contentScale = ContentScale.Fit,
                                    modifier = Modifier
                                        .width(66.dp)
                                        .height(34.dp),
                                )
                                Spacer(modifier = Modifier.width(8.dp))
                            }
                            Column(modifier = Modifier.weight(1f)) {
                                Text(
                                    text = if (slot.isLiveMatchSkin) "LIVE MATCH EQUIPPED" else "CURRENTLY EQUIPPED",
                                    color = if (slot.isLiveMatchSkin) ValGreen else slotAccent,
                                    fontSize = 9.sp,
                                    fontWeight = FontWeight.Black,
                                    letterSpacing = 0.6.sp,
                                )
                                Text(
                                    text = slot.skinName,
                                    color = TextPrimary,
                                    fontSize = 12.5.sp,
                                    fontWeight = FontWeight.Bold,
                                    maxLines = 1,
                                    overflow = TextOverflow.Ellipsis,
                                )
                                val metaParts = buildList {
                                    if (!activeChromaName.isNullOrBlank()) add(activeChromaName)
                                    if (activeLevelIdx >= 0) add("L${activeLevelIdx + 1}")
                                }
                                if (metaParts.isNotEmpty() || slot.buddyName != null) {
                                    Row(
                                        verticalAlignment = Alignment.CenterVertically,
                                        horizontalArrangement = Arrangement.spacedBy(6.dp),
                                    ) {
                                        if (metaParts.isNotEmpty()) {
                                            Text(
                                                text = metaParts.joinToString(" • "),
                                                color = TextSecondary,
                                                fontSize = 10.sp,
                                                maxLines = 1,
                                                overflow = TextOverflow.Ellipsis,
                                            )
                                        }
                                        slot.buddyName?.let { bName ->
                                            Row(verticalAlignment = Alignment.CenterVertically) {
                                                slot.buddyIcon?.let { bIcon ->
                                                    AsyncImage(
                                                        model = bIcon,
                                                        contentDescription = bName,
                                                        modifier = Modifier.size(13.dp),
                                                    )
                                                    Spacer(modifier = Modifier.width(3.dp))
                                                }
                                                Text(
                                                    text = bName,
                                                    color = ValGold,
                                                    fontSize = 10.sp,
                                                    maxLines = 1,
                                                    overflow = TextOverflow.Ellipsis,
                                                )
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        slot.nextMatchSkinName?.let { nextName ->
                            Text(
                                text = "Queued for next match: $nextName",
                                color = ValAmber,
                                fontSize = 10.sp,
                                fontWeight = FontWeight.Bold,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis,
                            )
                        }
                    }
                }
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    FilterChip(
                        selected = showOwnedOnly,
                        onClick = { showOwnedOnly = true },
                        label = {
                            Text("Owned (${ownedSkins.size})", fontSize = 11.5.sp, fontWeight = FontWeight.Bold)
                        },
                        colors = FilterChipDefaults.filterChipColors(
                            selectedContainerColor = ValGreen.copy(alpha = 0.22f),
                            selectedLabelColor = ValGreen,
                        ),
                    )
                    FilterChip(
                        selected = !showOwnedOnly,
                        onClick = { showOwnedOnly = false },
                        label = {
                            Text("All Skins (${allSkins.size})", fontSize = 11.5.sp, fontWeight = FontWeight.Bold)
                        },
                        colors = FilterChipDefaults.filterChipColors(
                            selectedContainerColor = ValCyan.copy(alpha = 0.22f),
                            selectedLabelColor = ValCyan,
                        ),
                    )
                }
            }
        },
        text = {
            LazyColumn(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(330.dp),
                verticalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                items(visibleSkins, key = { it.uuid }) { skin ->
                    val isCurrent = skin.uuid.equals(slot.skinUuid, ignoreCase = true)
                    val isStandard = skin.displayName.startsWith("Standard ", ignoreCase = true)
                    val isOwned = isCurrent ||
                        ownedSkinUuids.contains(skin.uuid.lowercase()) ||
                        isStandard
                    val canTapCard = isOwned && (!isCurrent || slot.nextMatchSkinName != null)
                    val accent = parseHexColor(skin.contentTierColor, ValCyan)
                    val previewIcon = if (isCurrent) {
                        skin.chromas.firstOrNull { it.uuid.equals(slot.chromaUuid, ignoreCase = true) }
                            ?.displayIcon ?: skin.displayIcon
                    } else {
                        skin.displayIcon
                    }

                    Surface(
                        color = if (isOwned) SurfaceElevated else ObsidianBg.copy(alpha = 0.6f),
                        shape = RoundedCornerShape(8.dp),
                        modifier = Modifier
                            .fillMaxWidth()
                            .border(
                                width = if (isCurrent) 1.5.dp else 1.dp,
                                color = when {
                                    isCurrent -> ValGold
                                    isOwned -> accent.copy(alpha = 0.4f)
                                    else -> SurfaceStroke.copy(alpha = 0.5f)
                                },
                                shape = RoundedCornerShape(8.dp),
                            )
                            .clickable(enabled = canTapCard) {
                                onSelectSkin(skin, skin.chromas.firstOrNull()?.uuid, null)
                            },
                    ) {
                        Column(modifier = Modifier.padding(10.dp)) {
                            Row(verticalAlignment = Alignment.CenterVertically) {
                                if (previewIcon != null) {
                                    AsyncImage(
                                        model = previewIcon,
                                        contentDescription = skin.displayName,
                                        modifier = Modifier
                                            .width(64.dp)
                                            .height(34.dp),
                                        contentScale = ContentScale.Fit,
                                        alpha = if (isOwned) 1f else 0.45f,
                                    )
                                    Spacer(modifier = Modifier.width(8.dp))
                                }
                                Column(modifier = Modifier.weight(1f)) {
                                    Text(
                                        text = skin.displayName,
                                        fontWeight = FontWeight.Bold,
                                        fontSize = 13.sp,
                                        color = if (isOwned) TextPrimary else TextMuted,
                                        maxLines = 1,
                                        overflow = TextOverflow.Ellipsis,
                                    )
                                    Text(
                                        text = skin.contentTierName ?: slot.weaponName,
                                        color = if (isOwned) accent else TextMuted,
                                        fontSize = 11.sp,
                                        maxLines = 1,
                                        overflow = TextOverflow.Ellipsis,
                                    )
                                }
                                Spacer(modifier = Modifier.width(6.dp))
                                when {
                                    isCurrent -> TacticalBadge("EQUIPPED", ValGold)
                                    isOwned -> TacticalBadge("OWNED", ValGreen)
                                    else -> Row(verticalAlignment = Alignment.CenterVertically) {
                                        Icon(
                                            Icons.Default.Lock,
                                            contentDescription = "Not owned",
                                            tint = TextMuted,
                                            modifier = Modifier.size(13.dp),
                                        )
                                        Spacer(modifier = Modifier.width(3.dp))
                                        Text("LOCKED", color = TextMuted, fontSize = 10.sp, fontWeight = FontWeight.Bold)
                                    }
                                }
                            }

                            if (isOwned && skin.chromas.size > 1) {
                                Spacer(modifier = Modifier.height(8.dp))
                                Row(
                                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                                    verticalAlignment = Alignment.CenterVertically,
                                ) {
                                    Text(
                                        text = "Variants:",
                                        color = TextMuted,
                                        fontSize = 10.sp,
                                        fontWeight = FontWeight.SemiBold,
                                    )
                                    skin.chromas.forEachIndexed { idx, chroma ->
                                        val isChromaActive = isCurrent &&
                                            chroma.uuid.equals(slot.chromaUuid, ignoreCase = true)
                                        val isChromaOwned = idx == 0 ||
                                            isChromaActive ||
                                            isStandard ||
                                            ownedChromaUuids.isEmpty() ||
                                            ownedChromaUuids.contains(chroma.uuid.lowercase())
                                        val swatchBg = when (idx) {
                                            0 -> accent
                                            1 -> ValCyan
                                            2 -> ValRed
                                            else -> ValGold
                                        }
                                        Box(
                                            modifier = Modifier
                                                .size(24.dp)
                                                .clip(CircleShape)
                                                .alpha(if (isChromaOwned) 1f else 0.3f)
                                                .background(swatchBg.copy(alpha = 0.25f), CircleShape)
                                                .border(
                                                    width = if (isChromaActive) 2.dp else 1.dp,
                                                    color = if (isChromaActive) ValGold else SurfaceStroke,
                                                    shape = CircleShape,
                                                )
                                                .clickable(enabled = isChromaOwned && !isChromaActive) {
                                                    onSelectSkin(skin, chroma.uuid, null)
                                                },
                                            contentAlignment = Alignment.Center,
                                        ) {
                                            if (chroma.swatch != null) {
                                                AsyncImage(
                                                    model = chroma.swatch,
                                                    contentDescription = chroma.displayName,
                                                    contentScale = ContentScale.Crop,
                                                    modifier = Modifier
                                                        .fillMaxSize()
                                                        .clip(CircleShape),
                                                )
                                            } else {
                                                Text(
                                                    text = "${idx + 1}",
                                                    color = TextPrimary,
                                                    fontSize = 9.5.sp,
                                                    fontWeight = FontWeight.Bold,
                                                )
                                            }
                                        }
                                    }
                                }
                            }

                            if (isOwned && skin.levels.size > 1) {
                                Spacer(modifier = Modifier.height(6.dp))
                                Row(
                                    horizontalArrangement = Arrangement.spacedBy(6.dp),
                                    verticalAlignment = Alignment.CenterVertically,
                                ) {
                                    Text(
                                        text = "Levels:",
                                        color = TextMuted,
                                        fontSize = 10.sp,
                                        fontWeight = FontWeight.SemiBold,
                                    )
                                    skin.levels.forEachIndexed { lvlIdx, lvl ->
                                        val isLevelActive = isCurrent &&
                                            lvl.uuid.equals(slot.levelUuid, ignoreCase = true)
                                        val isLevelOwned = lvlIdx == 0 ||
                                            isLevelActive ||
                                            isStandard ||
                                            ownedLevelUuids.isEmpty() ||
                                            ownedLevelUuids.contains(lvl.uuid.lowercase())
                                        Box(
                                            modifier = Modifier
                                                .clip(RoundedCornerShape(6.dp))
                                                .alpha(if (isLevelOwned) 1f else 0.35f)
                                                .background(
                                                    if (isLevelActive) ValGold.copy(alpha = 0.22f) else ObsidianBg,
                                                )
                                                .border(
                                                    width = if (isLevelActive) 1.5.dp else 1.dp,
                                                    color = if (isLevelActive) ValGold else SurfaceStroke,
                                                    shape = RoundedCornerShape(6.dp),
                                                )
                                                .clickable(enabled = isLevelOwned && !isLevelActive) {
                                                    val chromaArg = if (lvlIdx == skin.levels.lastIndex) {
                                                        if (isCurrent) slot.chromaUuid else null
                                                    } else {
                                                        skin.chromas.firstOrNull()?.uuid
                                                    }
                                                    onSelectSkin(skin, chromaArg, lvl.uuid)
                                                }
                                                .padding(horizontal = 8.dp, vertical = 3.dp),
                                        ) {
                                            Text(
                                                text = "L${lvlIdx + 1}",
                                                color = if (isLevelActive) ValGold else TextPrimary,
                                                fontSize = 10.sp,
                                                fontWeight = FontWeight.Bold,
                                            )
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        },
        confirmButton = {
            TextButton(onClick = onDismiss) { Text("Close") }
        },
    )
}

@Composable
private fun PickerFilterHeader(
    query: String,
    onQueryChange: (String) -> Unit,
    placeholder: String,
    showOwnedOnly: Boolean,
    ownedCount: Int,
    allCount: Int,
    onToggleOwnedOnly: (Boolean) -> Unit,
) {
    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
        OutlinedTextField(
            value = query,
            onValueChange = onQueryChange,
            placeholder = { Text(placeholder, fontSize = 12.sp) },
            leadingIcon = { Icon(Icons.Default.Search, contentDescription = null, modifier = Modifier.size(16.dp)) },
            singleLine = true,
            modifier = Modifier.fillMaxWidth(),
        )
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            FilterChip(
                selected = showOwnedOnly,
                onClick = { onToggleOwnedOnly(true) },
                label = { Text("Owned ($ownedCount)", fontSize = 11.5.sp, fontWeight = FontWeight.Bold) },
                colors = FilterChipDefaults.filterChipColors(
                    selectedContainerColor = ValGreen.copy(alpha = 0.22f),
                    selectedLabelColor = ValGreen,
                ),
            )
            FilterChip(
                selected = !showOwnedOnly,
                onClick = { onToggleOwnedOnly(false) },
                label = { Text("All ($allCount)", fontSize = 11.5.sp, fontWeight = FontWeight.Bold) },
                colors = FilterChipDefaults.filterChipColors(
                    selectedContainerColor = ValCyan.copy(alpha = 0.22f),
                    selectedLabelColor = ValCyan,
                ),
            )
        }
    }
}

@Composable
private fun SprayPickerDialog(
    slot: LoadoutSpraySlot,
    allSprays: List<Spray>,
    ownedSprayUuids: Set<String>,
    onDismiss: () -> Unit,
    onSelectSpray: (String) -> Unit,
) {
    var query by remember { mutableStateOf("") }
    val ownedSprays = remember(allSprays, ownedSprayUuids, slot.sprayUuid) {
        allSprays.filter { sp ->
            ownedSprayUuids.contains(sp.uuid.lowercase()) ||
                sp.uuid.equals(slot.sprayUuid, ignoreCase = true)
        }
    }
    var showOwnedOnly by remember(ownedSprays.size) { mutableStateOf(ownedSprays.isNotEmpty()) }
    val baseList = if (showOwnedOnly && ownedSprays.isNotEmpty()) ownedSprays else allSprays
    val filtered = remember(baseList, query) {
        if (query.isBlank()) baseList else baseList.filter { it.displayName.contains(query.trim(), ignoreCase = true) }
    }

    AlertDialog(
        onDismissRequest = onDismiss,
        containerColor = SurfaceCard,
        title = {
            Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                Text("Equip ${slot.slotLabel} Spray", fontWeight = FontWeight.Black, fontSize = 17.sp)
                PickerFilterHeader(
                    query = query,
                    onQueryChange = { query = it },
                    placeholder = "Search sprays...",
                    showOwnedOnly = showOwnedOnly,
                    ownedCount = ownedSprays.size,
                    allCount = allSprays.size,
                    onToggleOwnedOnly = { showOwnedOnly = it },
                )
            }
        },
        text = {
            LazyColumn(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(340.dp),
                verticalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                items(filtered.chunked(2), key = { row -> row.joinToString("-") { it.uuid } }) { rowItems ->
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.spacedBy(8.dp),
                    ) {
                        rowItems.forEach { spray ->
                            val isCurrent = spray.uuid.equals(slot.sprayUuid, ignoreCase = true)
                            val isOwned = isCurrent || ownedSprayUuids.isEmpty() || ownedSprayUuids.contains(spray.uuid.lowercase())
                            Surface(
                                color = if (isOwned) SurfaceElevated else ObsidianBg.copy(alpha = 0.6f),
                                shape = RoundedCornerShape(8.dp),
                                modifier = Modifier
                                    .weight(1f)
                                    .border(
                                        width = if (isCurrent) 1.5.dp else 1.dp,
                                        color = if (isCurrent) ValGold else SurfaceStroke,
                                        shape = RoundedCornerShape(8.dp),
                                    )
                                    .clickable(enabled = isOwned && !isCurrent) {
                                        onSelectSpray(spray.uuid)
                                    },
                            ) {
                                Column(
                                    modifier = Modifier.padding(8.dp),
                                    horizontalAlignment = Alignment.CenterHorizontally,
                                ) {
                                    Box(
                                        modifier = Modifier.size(64.dp),
                                        contentAlignment = Alignment.Center,
                                    ) {
                                        val art = spray.fullTransparentIcon ?: spray.displayIcon
                                        if (art != null) {
                                            AsyncImage(
                                                model = art,
                                                contentDescription = spray.displayName,
                                                contentScale = ContentScale.Fit,
                                                alpha = if (isOwned) 1f else 0.4f,
                                                modifier = Modifier.fillMaxSize(),
                                            )
                                        }
                                        if (!isOwned) {
                                            Icon(
                                                Icons.Default.Lock,
                                                contentDescription = "Locked",
                                                tint = TextMuted,
                                                modifier = Modifier
                                                    .align(Alignment.TopEnd)
                                                    .size(14.dp),
                                            )
                                        } else if (isCurrent) {
                                            Icon(
                                                Icons.Default.Check,
                                                contentDescription = "Equipped",
                                                tint = ValGold,
                                                modifier = Modifier
                                                    .align(Alignment.TopEnd)
                                                    .size(15.dp),
                                            )
                                        }
                                    }
                                    Spacer(modifier = Modifier.height(4.dp))
                                    Text(
                                        text = spray.displayName,
                                        color = if (isOwned) TextPrimary else TextMuted,
                                        fontSize = 10.5.sp,
                                        fontWeight = FontWeight.SemiBold,
                                        maxLines = 1,
                                        overflow = TextOverflow.Ellipsis,
                                        textAlign = TextAlign.Center,
                                    )
                                }
                            }
                        }
                        if (rowItems.size == 1) Spacer(modifier = Modifier.weight(1f))
                    }
                }
            }
        },
        confirmButton = {
            TextButton(onClick = onDismiss) { Text("Close") }
        },
    )
}

@Composable
private fun PlayerCardPickerDialog(
    currentCardUuid: String,
    allCards: List<PlayerCard>,
    ownedCardUuids: Set<String>,
    onDismiss: () -> Unit,
    onSelectCard: (String) -> Unit,
) {
    var query by remember { mutableStateOf("") }
    val ownedCards = remember(allCards, ownedCardUuids, currentCardUuid) {
        allCards.filter { c ->
            ownedCardUuids.contains(c.uuid.lowercase()) ||
                c.uuid.equals(currentCardUuid, ignoreCase = true)
        }
    }
    var showOwnedOnly by remember(ownedCards.size) { mutableStateOf(ownedCards.isNotEmpty()) }
    val baseList = if (showOwnedOnly && ownedCards.isNotEmpty()) ownedCards else allCards
    val filtered = remember(baseList, query) {
        if (query.isBlank()) baseList else baseList.filter { it.displayName.contains(query.trim(), ignoreCase = true) }
    }

    AlertDialog(
        onDismissRequest = onDismiss,
        containerColor = SurfaceCard,
        title = {
            Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                Text("Select Player Card", fontWeight = FontWeight.Black, fontSize = 17.sp)
                PickerFilterHeader(
                    query = query,
                    onQueryChange = { query = it },
                    placeholder = "Search cards...",
                    showOwnedOnly = showOwnedOnly,
                    ownedCount = ownedCards.size,
                    allCount = allCards.size,
                    onToggleOwnedOnly = { showOwnedOnly = it },
                )
            }
        },
        text = {
            LazyColumn(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(340.dp),
                verticalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                items(filtered.chunked(2), key = { row -> row.joinToString("-") { it.uuid } }) { rowItems ->
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.spacedBy(8.dp),
                    ) {
                        rowItems.forEach { card ->
                            val isCurrent = card.uuid.equals(currentCardUuid, ignoreCase = true)
                            val isOwned = isCurrent || ownedCardUuids.isEmpty() || ownedCardUuids.contains(card.uuid.lowercase())
                            Surface(
                                color = if (isOwned) SurfaceElevated else ObsidianBg.copy(alpha = 0.6f),
                                shape = RoundedCornerShape(8.dp),
                                modifier = Modifier
                                    .weight(1f)
                                    .border(
                                        width = if (isCurrent) 1.5.dp else 1.dp,
                                        color = if (isCurrent) ValGold else SurfaceStroke,
                                        shape = RoundedCornerShape(8.dp),
                                    )
                                    .clickable(enabled = isOwned && !isCurrent) {
                                        onSelectCard(card.uuid)
                                    },
                            ) {
                                Column {
                                    Box(
                                        modifier = Modifier
                                            .fillMaxWidth()
                                            .height(64.dp),
                                    ) {
                                        val art = card.wideArt ?: card.smallArt ?: card.largeArt
                                        if (art != null) {
                                            AsyncImage(
                                                model = art,
                                                contentDescription = card.displayName,
                                                contentScale = ContentScale.Crop,
                                                alpha = if (isOwned) 1f else 0.4f,
                                                modifier = Modifier
                                                    .fillMaxSize()
                                                    .clip(RoundedCornerShape(topStart = 8.dp, topEnd = 8.dp)),
                                            )
                                        }
                                        if (!isOwned) {
                                            Icon(
                                                Icons.Default.Lock,
                                                contentDescription = "Locked",
                                                tint = TextPrimary,
                                                modifier = Modifier
                                                    .align(Alignment.TopEnd)
                                                    .padding(4.dp)
                                                    .size(14.dp),
                                            )
                                        } else if (isCurrent) {
                                            Icon(
                                                Icons.Default.Check,
                                                contentDescription = "Equipped",
                                                tint = ValGold,
                                                modifier = Modifier
                                                    .align(Alignment.TopEnd)
                                                    .padding(4.dp)
                                                    .size(15.dp),
                                            )
                                        }
                                    }
                                    Text(
                                        text = card.displayName,
                                        color = if (isOwned) TextPrimary else TextMuted,
                                        fontSize = 10.5.sp,
                                        fontWeight = FontWeight.SemiBold,
                                        maxLines = 1,
                                        overflow = TextOverflow.Ellipsis,
                                        modifier = Modifier.padding(horizontal = 6.dp, vertical = 5.dp),
                                    )
                                }
                            }
                        }
                        if (rowItems.size == 1) Spacer(modifier = Modifier.weight(1f))
                    }
                }
            }
        },
        confirmButton = {
            TextButton(onClick = onDismiss) { Text("Close") }
        },
    )
}

@Composable
private fun PlayerTitlePickerDialog(
    currentTitleUuid: String,
    allTitles: List<PlayerTitle>,
    ownedTitleUuids: Set<String>,
    onDismiss: () -> Unit,
    onSelectTitle: (String) -> Unit,
) {
    var query by remember { mutableStateOf("") }
    val ownedTitles = remember(allTitles, ownedTitleUuids, currentTitleUuid) {
        allTitles.filter { t ->
            ownedTitleUuids.contains(t.uuid.lowercase()) ||
                t.uuid.equals(currentTitleUuid, ignoreCase = true)
        }
    }
    var showOwnedOnly by remember(ownedTitles.size) { mutableStateOf(ownedTitles.isNotEmpty()) }
    val baseList = if (showOwnedOnly && ownedTitles.isNotEmpty()) ownedTitles else allTitles
    val filtered = remember(baseList, query) {
        if (query.isBlank()) {
            baseList
        } else {
            val q = query.trim()
            baseList.filter {
                it.titleText.contains(q, ignoreCase = true) ||
                    it.displayName.contains(q, ignoreCase = true)
            }
        }
    }

    AlertDialog(
        onDismissRequest = onDismiss,
        containerColor = SurfaceCard,
        title = {
            Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                Text("Select Player Title", fontWeight = FontWeight.Black, fontSize = 17.sp)
                PickerFilterHeader(
                    query = query,
                    onQueryChange = { query = it },
                    placeholder = "Search titles...",
                    showOwnedOnly = showOwnedOnly,
                    ownedCount = ownedTitles.size,
                    allCount = allTitles.size,
                    onToggleOwnedOnly = { showOwnedOnly = it },
                )
            }
        },
        text = {
            LazyColumn(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(330.dp),
                verticalArrangement = Arrangement.spacedBy(6.dp),
            ) {
                items(filtered, key = { it.uuid }) { title ->
                    val isCurrent = title.uuid.equals(currentTitleUuid, ignoreCase = true)
                    val isOwned = isCurrent || ownedTitleUuids.isEmpty() || ownedTitleUuids.contains(title.uuid.lowercase())
                    val label = title.titleText.ifBlank { title.displayName }
                    Surface(
                        color = if (isOwned) SurfaceElevated else ObsidianBg.copy(alpha = 0.6f),
                        shape = RoundedCornerShape(8.dp),
                        modifier = Modifier
                            .fillMaxWidth()
                            .border(
                                width = if (isCurrent) 1.5.dp else 1.dp,
                                color = if (isCurrent) ValGold else SurfaceStroke,
                                shape = RoundedCornerShape(8.dp),
                            )
                            .clickable(enabled = isOwned && !isCurrent) {
                                onSelectTitle(title.uuid)
                            },
                    ) {
                        Row(
                            modifier = Modifier
                                .fillMaxWidth()
                                .padding(horizontal = 12.dp, vertical = 10.dp),
                            verticalAlignment = Alignment.CenterVertically,
                            horizontalArrangement = Arrangement.SpaceBetween,
                        ) {
                            Text(
                                text = label,
                                color = if (isOwned) TextPrimary else TextMuted,
                                fontWeight = FontWeight.Bold,
                                fontSize = 13.sp,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis,
                                modifier = Modifier.weight(1f),
                            )
                            Spacer(modifier = Modifier.width(8.dp))
                            when {
                                isCurrent -> TacticalBadge("ACTIVE", ValGold)
                                isOwned -> TacticalBadge("OWNED", ValGreen)
                                else -> Icon(Icons.Default.Lock, contentDescription = "Locked", tint = TextMuted, modifier = Modifier.size(14.dp))
                            }
                        }
                    }
                }
            }
        },
        dismissButton = {
            if (currentTitleUuid.isNotBlank()) {
                TextButton(onClick = { onSelectTitle("") }) {
                    Text("Clear Title", color = ValRed, fontWeight = FontWeight.Bold)
                }
            }
        },
        confirmButton = {
            TextButton(onClick = onDismiss) { Text("Close") }
        },
    )
}

@Composable
private fun SavePresetDialog(
    onDismiss: () -> Unit,
    onSave: (String, String) -> Unit,
) {
    var name by remember { mutableStateOf("") }
    var colorHex by remember { mutableStateOf("#A78BFA") }
    val palette = listOf("#FF4655", "#28D8C8", "#A78BFA", "#F5C451", "#34D399")

    AlertDialog(
        onDismissRequest = onDismiss,
        containerColor = SurfaceCard,
        title = { Text("Save Loadout Preset", fontWeight = FontWeight.Bold) },
        text = {
            Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
                OutlinedTextField(
                    value = name,
                    onValueChange = { name = it },
                    label = { Text("Preset Name (e.g. Kuronami Set)") },
                    singleLine = true,
                )
                Row(horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                    palette.forEach { hex ->
                        val c = parseHexColor(hex, ValCyan)
                        Box(
                            modifier = Modifier
                                .size(30.dp)
                                .clip(RoundedCornerShape(8.dp))
                                .background(c)
                                .border(
                                    width = if (colorHex == hex) 2.dp else 0.dp,
                                    color = TextPrimary,
                                    shape = RoundedCornerShape(8.dp),
                                )
                                .clickable { colorHex = hex },
                        )
                    }
                }
            }
        },
        confirmButton = {
            Button(
                onClick = { if (name.isNotBlank()) onSave(name, colorHex) },
                colors = ButtonDefaults.buttonColors(containerColor = ValRed),
            ) {
                Text("Save Preset")
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) { Text("Cancel") }
        },
    )
}

private fun MyValUiState.allSkinsForPicker(weaponName: String): List<Skin> {
    val source = if (allWeaponSkins.isNotEmpty()) allWeaponSkins else skins
    return source.filter { it.weaponName.equals(weaponName, ignoreCase = true) }
}
