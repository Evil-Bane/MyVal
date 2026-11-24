package com.myval.app.ui.store

import androidx.compose.animation.core.animateFloatAsState
import java.util.Locale
import androidx.compose.animation.core.tween
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
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.CardGiftcard
import androidx.compose.material.icons.filled.Favorite
import androidx.compose.material.icons.filled.FavoriteBorder
import androidx.compose.material.icons.filled.NotificationsActive
import androidx.compose.material.icons.filled.PersonAdd
import androidx.compose.material.icons.filled.Visibility
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil.compose.AsyncImage
import com.myval.app.ui.MyValUiState
import com.myval.app.ui.components.AccountSwitcherChip
import com.myval.app.ui.components.AuthRequiredCard
import com.myval.app.ui.components.LiveNetworkStatusBar
import com.myval.app.ui.components.MyValPullToRefreshBox
import com.myval.app.ui.components.SharpChipShape
import com.myval.app.ui.components.TacticalBadge
import com.myval.app.ui.components.formatDuration
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
import com.myval.app.ui.theme.ValGreen
import com.myval.app.ui.theme.ValPurple
import com.myval.app.ui.theme.ValRed
import uniffi.myval_ffi.CollectionSummary
import uniffi.myval_ffi.DailyOffer
import uniffi.myval_ffi.NightMarketOffer
import uniffi.myval_ffi.StorefrontSnapshot

@Composable
fun StoreScreen(
    uiState: MyValUiState,
    onOpenLogin: () -> Unit,
    onOpenAccounts: () -> Unit,
    onRefreshStore: () -> Unit,
    onToggleWishlist: (String) -> Unit,
) {
    val revealedNightMarket = remember { mutableStateMapOf<String, Boolean>() }

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
                Text("STORE", fontWeight = FontWeight.Black, fontSize = 18.sp, letterSpacing = 1.sp)
                Spacer(modifier = Modifier.height(4.dp))
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
                        text = "Sign in to load your Daily Shop & Vault",
                        color = TextSecondary,
                        fontSize = 11.5.sp,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                }
            }
            if (uiState.storefront == null && uiState.isLoadingStore && uiState.session != null) {
                CircularProgressIndicator(modifier = Modifier.size(20.dp), color = ValCyan, strokeWidth = 2.dp)
            }
        }

        LiveNetworkStatusBar(
            message = uiState.networkStatusMessage,
            isSlow = uiState.isNetworkSlow,
        )

        if (uiState.storeError != null) {
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .background(ValRed.copy(alpha = 0.14f))
                    .padding(horizontal = 16.dp, vertical = 8.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.SpaceBetween,
            ) {
                Text(
                    text = uiState.storeError,
                    color = ValRed,
                    fontSize = 12.sp,
                    modifier = Modifier.weight(1f),
                )
                Button(
                    onClick = onRefreshStore,
                    colors = ButtonDefaults.buttonColors(containerColor = ValRed),
                    contentPadding = PaddingValues(horizontal = 10.dp, vertical = 4.dp),
                ) {
                    Text("Retry", fontSize = 11.sp)
                }
            }
        }

        if (uiState.session == null) {
            Box(modifier = Modifier.padding(16.dp)) {
                AuthRequiredCard(
                    title = "Sign In to View Your Daily Store",
                    subtitle = "Authenticate with your Riot account to check your Daily 4 Offers, Featured Bundles, 3D Night Market, Accessories, Wallet Balances, and Total Vault VP Value.",
                    onOpenLogin = onOpenLogin,
                    savedAccountCount = uiState.savedAccounts.size,
                    onOpenAccounts = onOpenAccounts,
                )
            }
            return
        }

        val store = uiState.storefront
        if (store == null) {
            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .padding(24.dp),
                contentAlignment = Alignment.Center,
            ) {
                val err = uiState.storeError
                if (err != null && !uiState.isLoadingStore) {
                    Column(
                        horizontalAlignment = Alignment.CenterHorizontally,
                        verticalArrangement = Arrangement.spacedBy(10.dp),
                    ) {
                        Text(
                            text = "Failed to load Riot Store",
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
                            onClick = onRefreshStore,
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
            isRefreshing = uiState.isLoadingStore,
            onRefresh = onRefreshStore,
            accent = ValCyan,
            modifier = Modifier
                .fillMaxWidth()
                .weight(1f),
        ) {
            StoreContentList(
                store = store,
                collection = uiState.collectionSummary,
                revealedNightMarket = revealedNightMarket,
                onRevealNightMarket = { id -> revealedNightMarket[id] = true },
                onToggleWishlist = onToggleWishlist,
            )
        }
    }
}

private enum class ShopSubTab(val label: String) {
    DAILY("Daily Offers"),
    ACCESSORY("Accessory"),
    BUNDLES("Bundles"),
    NIGHT_MARKET("Night Market"),
}

@Composable
private fun StoreContentList(
    store: StorefrontSnapshot,
    collection: CollectionSummary?,
    revealedNightMarket: Map<String, Boolean>,
    onRevealNightMarket: (String) -> Unit,
    onToggleWishlist: (String) -> Unit,
) {
    var selectedSubTab by remember { mutableStateOf(ShopSubTab.DAILY) }
    val wishHits = remember(store) {
        val list = mutableListOf<String>()
        store.dailyOffers.filter { it.isWishlisted }.forEach { list.add("${it.skinName} (Daily)") }
        store.nightMarket?.offers?.filter { it.isWishlisted }
            ?.forEach { list.add("${it.skinName} (-${it.discountPercent}%)") }
        list
    }

    LazyColumn(
        modifier = Modifier.fillMaxSize(),
        contentPadding = PaddingValues(16.dp),
        verticalArrangement = Arrangement.spacedBy(14.dp),
    ) {
        if (wishHits.isNotEmpty()) {
            item {
                Surface(
                    color = ValGreen.copy(alpha = 0.16f),
                    shape = RoundedCornerShape(10.dp),
                    modifier = Modifier
                        .fillMaxWidth()
                        .border(1.5.dp, ValGreen, RoundedCornerShape(10.dp)),
                ) {
                    Row(
                        modifier = Modifier.padding(12.dp),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Icon(Icons.Default.NotificationsActive, contentDescription = null, tint = ValGreen)
                        Spacer(modifier = Modifier.width(10.dp))
                        Column {
                            Text(
                                text = "WISHLIST DROP IN YOUR SHOP!",
                                color = ValGreen,
                                fontWeight = FontWeight.Black,
                                fontSize = 12.sp,
                            )
                            Text(
                                text = wishHits.joinToString(" • "),
                                color = TextPrimary,
                                fontWeight = FontWeight.Bold,
                                fontSize = 13.sp,
                            )
                        }
                    }
                }
            }
        }

        // Clean Borderless Wallet + Collection Value Strip on pitch-black OLED canvas
        item {
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 2.dp, vertical = 2.dp),
            ) {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                ) {
                    WalletStat("VP", store.wallet.vp.toString(), ValCyan)
                    WalletStat("RADIANITE", store.wallet.radianite.toString(), ValGold)
                    WalletStat("KINGDOM", store.wallet.kingdomCredits.toString(), ValPurple)
                    if (collection != null) {
                        WalletStat(
                            "VAULT VALUE",
                            "$${"%.0f".format(Locale.US, collection.estimatedUsd)}",
                            ValGreen,
                        )
                    }
                }
                if (collection != null && collection.rarityBreakdown.isNotEmpty()) {
                    Spacer(modifier = Modifier.height(10.dp))
                    Row(
                        modifier = Modifier.horizontalScroll(rememberScrollState()),
                        horizontalArrangement = Arrangement.spacedBy(6.dp),
                    ) {
                        TacticalBadge(
                            "${collection.ownedSkinCount} Skins (${collection.estimatedTotalVp} VP)",
                            ValGold,
                        )
                        collection.rarityBreakdown.forEach { tier ->
                            TacticalBadge(
                                "${tier.count} ${tier.tierName}",
                                parseHexColor(tier.tierColor, ValCyan),
                            )
                        }
                    }
                }
                Spacer(modifier = Modifier.height(10.dp))
                HorizontalDivider(
                    thickness = 0.6.dp,
                    color = SurfaceStroke.copy(alpha = 0.7f),
                )
            }
        }

        // 4-Tab Shop Selector (Daily | Accessory | Bundles | Night Market)
        item {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(6.dp),
            ) {
                ShopSubTab.entries.forEach { tab ->
                    val isSelected = selectedSubTab == tab
                    val showHotDot = when (tab) {
                        ShopSubTab.DAILY -> store.dailyOffers.any { it.isWishlisted }
                        ShopSubTab.NIGHT_MARKET -> store.nightMarket != null
                        else -> false
                    }
                    val dotColor = if (tab == ShopSubTab.NIGHT_MARKET) ValRed else ValGreen
                    Surface(
                        color = if (isSelected) ValRed.copy(alpha = 0.18f) else SurfaceCard,
                        shape = SharpChipShape,
                        modifier = Modifier
                            .weight(1f)
                            .border(
                                1.dp,
                                if (isSelected) ValRed else SurfaceStroke,
                                SharpChipShape,
                            )
                            .clip(SharpChipShape)
                            .clickable { selectedSubTab = tab },
                    ) {
                        Row(
                            modifier = Modifier.padding(horizontal = 6.dp, vertical = 9.dp),
                            horizontalArrangement = Arrangement.Center,
                            verticalAlignment = Alignment.CenterVertically,
                        ) {
                            if (showHotDot) {
                                Box(
                                    modifier = Modifier
                                        .size(6.dp)
                                        .background(dotColor, CircleShape),
                                )
                                Spacer(modifier = Modifier.width(4.dp))
                            }
                            Text(
                                text = when (tab) {
                                    ShopSubTab.DAILY -> "Daily"
                                    ShopSubTab.ACCESSORY -> "Accessory"
                                    ShopSubTab.BUNDLES -> "Bundles"
                                    ShopSubTab.NIGHT_MARKET -> "Night Mkt"
                                },
                                color = if (isSelected) ValRed else TextSecondary,
                                fontSize = 11.sp,
                                fontWeight = if (isSelected) FontWeight.Black else FontWeight.SemiBold,
                                maxLines = 1,
                                softWrap = false,
                                overflow = TextOverflow.Ellipsis,
                            )
                        }
                    }
                }
            }
        }

        when (selectedSubTab) {
            ShopSubTab.DAILY -> {
                item {
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Text("DAILY OFFERS", fontWeight = FontWeight.Black, fontSize = 14.sp, letterSpacing = 1.sp)
                        TacticalBadge("Resets in ${formatDuration(store.dailyRemainingSecs)}", ValCyan)
                    }
                }

                val rows = store.dailyOffers.chunked(2)
                items(rows, key = { pair -> pair.joinToString("-") { it.offerId } }) { pair ->
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.spacedBy(10.dp),
                    ) {
                        pair.forEach { offer ->
                            DailyOfferCard(
                                offer = offer,
                                modifier = Modifier.weight(1f),
                                onToggleWishlist = { onToggleWishlist(offer.skinUuid) },
                            )
                        }
                        if (pair.size == 1) {
                            Spacer(modifier = Modifier.weight(1f))
                        }
                    }
                }
            }

            ShopSubTab.BUNDLES -> {
                if (store.featuredBundles.isEmpty()) {
                    item {
                        Surface(
                            color = SurfaceCard,
                            shape = RoundedCornerShape(10.dp),
                            modifier = Modifier
                                .fillMaxWidth()
                                .border(1.dp, SurfaceStroke, RoundedCornerShape(10.dp)),
                        ) {
                            Text(
                                text = "No featured bundles are active right now.",
                                color = TextSecondary,
                                fontSize = 13.sp,
                                textAlign = TextAlign.Center,
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .padding(24.dp),
                            )
                        }
                    }
                } else {
                    store.featuredBundles.forEachIndexed { bIdx, bundle ->
                        item(key = "bundle-banner-$bIdx-${bundle.bundleUuid}") {
                            Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                                Row(
                                    modifier = Modifier.fillMaxWidth(),
                                    horizontalArrangement = Arrangement.SpaceBetween,
                                    verticalAlignment = Alignment.CenterVertically,
                                ) {
                                    Text("FEATURED BUNDLE", fontWeight = FontWeight.Black, fontSize = 13.sp, letterSpacing = 1.sp)
                                    TacticalBadge("Ends in ${formatDuration(bundle.remainingSecs)}", ValGold)
                                }
                                Surface(
                                    color = SurfaceCard,
                                    shape = RoundedCornerShape(10.dp),
                                    modifier = Modifier
                                        .fillMaxWidth()
                                        .border(1.dp, SurfaceStroke, RoundedCornerShape(10.dp)),
                                ) {
                                    Column {
                                        if (bundle.displayIcon != null) {
                                            AsyncImage(
                                                model = bundle.displayIcon,
                                                contentDescription = bundle.name,
                                                contentScale = ContentScale.Crop,
                                                modifier = Modifier
                                                    .fillMaxWidth()
                                                    .height(150.dp)
                                                    .clip(RoundedCornerShape(topStart = 10.dp, topEnd = 10.dp)),
                                            )
                                        }
                                        Row(
                                            modifier = Modifier
                                                .fillMaxWidth()
                                                .padding(14.dp),
                                            horizontalArrangement = Arrangement.SpaceBetween,
                                            verticalAlignment = Alignment.CenterVertically,
                                        ) {
                                            Column(modifier = Modifier.weight(1f)) {
                                                Text(bundle.name, fontWeight = FontWeight.Black, fontSize = 16.sp, color = TextPrimary)
                                                Text(
                                                    text = "${bundle.items.size} items",
                                                    fontSize = 12.sp,
                                                    color = TextSecondary,
                                                )
                                            }
                                            Column(horizontalAlignment = Alignment.End) {
                                                TacticalBadge("${bundle.totalDiscountedPriceVp} VP", ValGold)
                                                if (bundle.totalBasePriceVp > bundle.totalDiscountedPriceVp) {
                                                    Spacer(modifier = Modifier.height(2.dp))
                                                    Text(
                                                        text = "${bundle.totalBasePriceVp} VP",
                                                        color = TextMuted,
                                                        fontSize = 10.sp,
                                                        textDecoration = TextDecoration.LineThrough,
                                                    )
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        itemsIndexed(bundle.items, key = { iIdx, item -> "bundle-$bIdx-$iIdx-${item.itemUuid}" }) { _, item ->
                            Surface(
                                color = SurfaceCard,
                                shape = RoundedCornerShape(10.dp),
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .border(1.dp, SurfaceStroke, RoundedCornerShape(10.dp)),
                            ) {
                                Row(
                                    modifier = Modifier.padding(horizontal = 12.dp, vertical = 10.dp),
                                    verticalAlignment = Alignment.CenterVertically,
                                ) {
                                    Box(
                                        modifier = Modifier
                                            .width(72.dp)
                                            .height(42.dp)
                                            .background(SurfaceElevated, RoundedCornerShape(8.dp)),
                                        contentAlignment = Alignment.Center,
                                    ) {
                                        if (item.displayIcon != null) {
                                            AsyncImage(
                                                model = item.displayIcon,
                                                contentDescription = item.name,
                                                contentScale = ContentScale.Fit,
                                                modifier = Modifier
                                                    .fillMaxSize()
                                                    .padding(4.dp),
                                            )
                                        } else {
                                            Icon(
                                                imageVector = Icons.Default.CardGiftcard,
                                                contentDescription = null,
                                                tint = ValGold,
                                                modifier = Modifier.size(18.dp),
                                            )
                                        }
                                    }
                                    Spacer(modifier = Modifier.width(12.dp))
                                    Column(modifier = Modifier.weight(1f)) {
                                        Text(
                                            text = item.name,
                                            color = TextPrimary,
                                            fontWeight = FontWeight.Bold,
                                            fontSize = 13.sp,
                                            maxLines = 1,
                                            overflow = TextOverflow.Ellipsis,
                                        )
                                        Row(verticalAlignment = Alignment.CenterVertically) {
                                            Text(
                                                text = "${item.discountedPriceVp} VP",
                                                color = ValCyan,
                                                fontWeight = FontWeight.Black,
                                                fontSize = 11.5.sp,
                                            )
                                            if (item.basePriceVp > item.discountedPriceVp) {
                                                Spacer(modifier = Modifier.width(6.dp))
                                                Text(
                                                    text = "${item.basePriceVp}",
                                                    color = TextMuted,
                                                    fontSize = 10.sp,
                                                    textDecoration = TextDecoration.LineThrough,
                                                )
                                            }
                                            Spacer(modifier = Modifier.width(8.dp))
                                            Text(
                                                text = item.itemType,
                                                color = TextMuted,
                                                fontSize = 10.5.sp,
                                            )
                                        }
                                    }
                                    if (item.discountPercent > 0) {
                                        TacticalBadge("-${item.discountPercent}%", ValGold)
                                    }
                                }
                            }
                        }
                    }
                }
            }

            ShopSubTab.ACCESSORY -> {
                item {
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Text("ACCESSORIES", fontWeight = FontWeight.Black, fontSize = 14.sp, letterSpacing = 1.sp)
                        TacticalBadge("Resets in ${formatDuration(store.accessoryRemainingSecs)}", ValPurple)
                    }
                }
                if (store.accessoryOffers.isEmpty()) {
                    item {
                        Surface(
                            color = SurfaceCard,
                            shape = RoundedCornerShape(10.dp),
                            modifier = Modifier
                                .fillMaxWidth()
                                .border(1.dp, SurfaceStroke, RoundedCornerShape(10.dp)),
                        ) {
                            Text(
                                text = "No accessory offers available right now.",
                                color = TextSecondary,
                                fontSize = 13.sp,
                                textAlign = TextAlign.Center,
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .padding(24.dp),
                            )
                        }
                    }
                } else {
                    items(store.accessoryOffers, key = { it.offerId }) { acc ->
                        Surface(
                            color = SurfaceCard,
                            shape = RoundedCornerShape(10.dp),
                            modifier = Modifier
                                .fillMaxWidth()
                                .border(1.dp, SurfaceStroke, RoundedCornerShape(10.dp)),
                        ) {
                            Column(
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .padding(14.dp),
                                verticalArrangement = Arrangement.spacedBy(10.dp),
                            ) {
                                Box(
                                    modifier = Modifier
                                        .fillMaxWidth()
                                        .height(82.dp)
                                        .background(SurfaceElevated, RoundedCornerShape(8.dp)),
                                    contentAlignment = Alignment.Center,
                                ) {
                                    if (acc.displayIcon != null) {
                                        AsyncImage(
                                            model = acc.displayIcon,
                                            contentDescription = acc.name,
                                            modifier = Modifier
                                                .fillMaxSize()
                                                .padding(8.dp),
                                            contentScale = ContentScale.Fit,
                                        )
                                    } else {
                                        Row(verticalAlignment = Alignment.CenterVertically) {
                                            Icon(
                                                Icons.Default.CardGiftcard,
                                                contentDescription = null,
                                                tint = ValPurple,
                                                modifier = Modifier.size(20.dp),
                                            )
                                            Spacer(modifier = Modifier.width(6.dp))
                                            Text(
                                                text = acc.itemType.uppercase(),
                                                color = ValPurple,
                                                fontSize = 12.sp,
                                                fontWeight = FontWeight.Black,
                                            )
                                        }
                                    }
                                }
                                Row(
                                    modifier = Modifier.fillMaxWidth(),
                                    horizontalArrangement = Arrangement.SpaceBetween,
                                    verticalAlignment = Alignment.CenterVertically,
                                ) {
                                    Column(modifier = Modifier.weight(1f)) {
                                        Text(
                                            text = acc.name,
                                            fontWeight = FontWeight.Bold,
                                            fontSize = 13.5.sp,
                                            color = TextPrimary,
                                            maxLines = 1,
                                            overflow = TextOverflow.Ellipsis,
                                        )
                                        Text(
                                            text = acc.itemType,
                                            color = TextMuted,
                                            fontSize = 11.sp,
                                        )
                                    }
                                    TacticalBadge("${acc.costKc} KC", ValPurple)
                                }
                            }
                        }
                    }
                }
            }

            ShopSubTab.NIGHT_MARKET -> {
                val nm = store.nightMarket
                if (nm == null) {
                    item {
                        Surface(
                            color = SurfaceCard,
                            shape = RoundedCornerShape(10.dp),
                            modifier = Modifier
                                .fillMaxWidth()
                                .border(1.dp, SurfaceStroke, RoundedCornerShape(10.dp)),
                        ) {
                            Column(
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .padding(24.dp),
                                horizontalAlignment = Alignment.CenterHorizontally,
                                verticalArrangement = Arrangement.spacedBy(8.dp),
                            ) {
                                Icon(
                                    imageVector = Icons.Default.Visibility,
                                    contentDescription = null,
                                    tint = ValRed.copy(alpha = 0.7f),
                                    modifier = Modifier.size(28.dp),
                                )
                                Text(
                                    text = "The Night Market isn't live right now",
                                    color = TextPrimary,
                                    fontWeight = FontWeight.Bold,
                                    fontSize = 14.sp,
                                    textAlign = TextAlign.Center,
                                )
                                Text(
                                    text = "Check back when the next Night Market drops for 6 discounted mystery skins.",
                                    color = TextSecondary,
                                    fontSize = 12.sp,
                                    textAlign = TextAlign.Center,
                                )
                            }
                        }
                    }
                } else {
                    item {
                        Row(
                            modifier = Modifier.fillMaxWidth(),
                            horizontalArrangement = Arrangement.SpaceBetween,
                            verticalAlignment = Alignment.CenterVertically,
                        ) {
                            Text(
                                "NIGHT MARKET (TAP TO 3D FLIP)",
                                fontWeight = FontWeight.Black,
                                fontSize = 13.sp,
                                color = ValRed,
                                letterSpacing = 1.sp,
                            )
                            TacticalBadge("Ends in ${formatDuration(nm.remainingSecs)}", ValRed)
                        }
                    }
                    val nmRows = nm.offers.chunked(2)
                    items(nmRows, key = { pair -> "nm-${pair.joinToString("-") { it.offerId }}" }) { pair ->
                        Row(
                            modifier = Modifier.fillMaxWidth(),
                            horizontalArrangement = Arrangement.spacedBy(10.dp),
                        ) {
                            pair.forEach { offer ->
                                val revealed = offer.isSeen || revealedNightMarket[offer.offerId] == true
                                NightMarket3DFlipCard(
                                    offer = offer,
                                    revealed = revealed,
                                    onReveal = { onRevealNightMarket(offer.offerId) },
                                    onToggleWishlist = { onToggleWishlist(offer.skinUuid) },
                                    modifier = Modifier.weight(1f),
                                )
                            }
                            if (pair.size == 1) {
                                Spacer(modifier = Modifier.weight(1f))
                            }
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun WalletStat(label: String, value: String, accent: androidx.compose.ui.graphics.Color) {
    Column(horizontalAlignment = Alignment.CenterHorizontally) {
        Text(label, color = TextMuted, fontSize = 10.sp, fontWeight = FontWeight.Bold)
        Text(value, color = accent, fontSize = 16.sp, fontWeight = FontWeight.Black)
    }
}

@Composable
private fun DailyOfferCard(
    offer: DailyOffer,
    modifier: Modifier = Modifier,
    onToggleWishlist: () -> Unit,
) {
    val accent = parseHexColor(offer.tierColor, ValCyan)
    Surface(
        color = SurfaceCard,
        shape = RoundedCornerShape(10.dp),
        modifier = modifier.border(1.dp, accent.copy(alpha = 0.55f), RoundedCornerShape(10.dp)),
    ) {
        Column(
            modifier = Modifier
                .background(Brush.verticalGradient(listOf(accent.copy(alpha = 0.16f), SurfaceCard)))
                .padding(11.dp),
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(
                    text = "${offer.costVp} VP",
                    color = ValCyan,
                    fontWeight = FontWeight.Black,
                    fontSize = 12.sp,
                )
                IconButton(onClick = onToggleWishlist, modifier = Modifier.size(24.dp)) {
                    Icon(
                        imageVector = if (offer.isWishlisted) Icons.Default.Favorite else Icons.Default.FavoriteBorder,
                        contentDescription = "Wishlist",
                        tint = if (offer.isWishlisted) ValRed else TextMuted,
                        modifier = Modifier.size(16.dp),
                    )
                }
            }
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(68.dp),
                contentAlignment = Alignment.Center,
            ) {
                if (offer.displayIcon != null) {
                    AsyncImage(
                        model = offer.displayIcon,
                        contentDescription = offer.skinName,
                        contentScale = ContentScale.Fit,
                        modifier = Modifier.fillMaxSize(),
                    )
                }
            }
            Spacer(modifier = Modifier.height(6.dp))
            Text(
                text = offer.skinName,
                color = TextPrimary,
                fontWeight = FontWeight.Bold,
                fontSize = 12.5.sp,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            Text(
                text = offer.weaponName,
                color = TextSecondary,
                fontSize = 11.sp,
            )
        }
    }
}

@Composable
private fun NightMarket3DFlipCard(
    offer: NightMarketOffer,
    revealed: Boolean,
    onReveal: () -> Unit,
    onToggleWishlist: () -> Unit,
    modifier: Modifier = Modifier.width(170.dp),
) {
    val rotation by animateFloatAsState(
        targetValue = if (revealed) 180f else 0f,
        animationSpec = tween(durationMillis = 520),
        label = "nm3dFlip",
    )
    val accent = parseHexColor(offer.tierColor, ValRed)

    Surface(
        color = SurfaceElevated,
        shape = RoundedCornerShape(10.dp),
        modifier = modifier
            .height(152.dp)
            .graphicsLayer {
                rotationY = rotation
                cameraDistance = 12f * density
            }
            .border(
                1.dp,
                if (rotation > 90f) accent else ValRed.copy(alpha = 0.6f),
                RoundedCornerShape(10.dp),
            )
            .clickable { if (!revealed) onReveal() },
    ) {
        if (rotation <= 90f) {
            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .background(Brush.verticalGradient(listOf(ValRed.copy(alpha = 0.24f), SurfaceCard))),
                contentAlignment = Alignment.Center,
            ) {
                Column(horizontalAlignment = Alignment.CenterHorizontally) {
                    Icon(Icons.Default.Visibility, contentDescription = "Tap to flip", tint = ValRed)
                    Spacer(modifier = Modifier.height(6.dp))
                    Text(
                        "-${offer.discountPercent}%",
                        color = ValGold,
                        fontWeight = FontWeight.Black,
                        fontSize = 20.sp,
                    )
                    Text("TAP TO 3D FLIP", color = TextSecondary, fontSize = 10.sp, fontWeight = FontWeight.Bold)
                }
            }
        } else {
            Column(
                modifier = Modifier
                    .fillMaxSize()
                    .graphicsLayer { rotationY = 180f }
                    .background(Brush.verticalGradient(listOf(accent.copy(alpha = 0.16f), SurfaceCard)))
                    .padding(11.dp),
            ) {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    TacticalBadge("-${offer.discountPercent}%", ValGold)
                    IconButton(onClick = onToggleWishlist, modifier = Modifier.size(22.dp)) {
                        Icon(
                            imageVector = if (offer.isWishlisted) Icons.Default.Favorite else Icons.Default.FavoriteBorder,
                            contentDescription = "Wishlist",
                            tint = if (offer.isWishlisted) ValRed else TextMuted,
                            modifier = Modifier.size(15.dp),
                        )
                    }
                }
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(54.dp),
                    contentAlignment = Alignment.Center,
                ) {
                    if (offer.displayIcon != null) {
                        AsyncImage(
                            model = offer.displayIcon,
                            contentDescription = offer.skinName,
                            contentScale = ContentScale.Fit,
                            modifier = Modifier.fillMaxSize(),
                        )
                    }
                }
                Text(
                    text = offer.skinName,
                    fontWeight = FontWeight.Bold,
                    fontSize = 12.sp,
                    color = TextPrimary,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Text(
                        text = "${offer.discountPriceVp} VP",
                        color = ValCyan,
                        fontWeight = FontWeight.Black,
                        fontSize = 12.sp,
                    )
                    Spacer(modifier = Modifier.width(6.dp))
                    Text(
                        text = "${offer.basePriceVp}",
                        fontSize = 10.sp,
                        color = TextMuted,
                        textDecoration = TextDecoration.LineThrough,
                    )
                }
            }
        }
    }
}
