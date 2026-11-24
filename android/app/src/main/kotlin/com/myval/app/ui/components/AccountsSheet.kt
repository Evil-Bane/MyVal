package com.myval.app.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.DeleteOutline
import androidx.compose.material.icons.filled.PersonAdd
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil.compose.AsyncImage
import com.myval.app.data.StoredAccountRecord
import com.myval.app.ui.theme.ObsidianBg
import com.myval.app.ui.theme.SurfaceCard
import com.myval.app.ui.theme.SurfaceElevated
import com.myval.app.ui.theme.SurfaceStroke
import com.myval.app.ui.theme.TextMuted
import com.myval.app.ui.theme.TextPrimary
import com.myval.app.ui.theme.TextSecondary
import com.myval.app.ui.theme.ValAmber
import com.myval.app.ui.theme.ValCyan
import com.myval.app.ui.theme.ValGreen
import com.myval.app.ui.theme.ValRed

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun AccountsManagerBottomSheet(
    accounts: List<StoredAccountRecord>,
    activePuuid: String?,
    onDismiss: () -> Unit,
    onSwitchAccount: (String) -> Unit,
    onAddAnotherAccount: () -> Unit,
    onRefreshCurrentAccount: () -> Unit,
    onRemoveAccount: (String) -> Unit,
    onSignOutAll: () -> Unit,
) {
    val sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)
    val nowEpochSecs = System.currentTimeMillis() / 1000L

    ModalBottomSheet(
        onDismissRequest = onDismiss,
        sheetState = sheetState,
        containerColor = SurfaceCard,
        contentColor = TextPrimary,
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .navigationBarsPadding()
                .padding(horizontal = 16.dp, vertical = 8.dp),
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        text = "ACCOUNTS (${accounts.size})",
                        fontWeight = FontWeight.Black,
                        fontSize = 17.sp,
                        letterSpacing = 1.sp,
                        color = TextPrimary,
                    )
                }
            }

            Spacer(modifier = Modifier.height(14.dp))

            if (accounts.isEmpty()) {
                Surface(
                    color = SurfaceElevated,
                    shape = RoundedCornerShape(10.dp),
                    modifier = Modifier
                        .fillMaxWidth()
                        .border(1.dp, SurfaceStroke, RoundedCornerShape(10.dp)),
                ) {
                    Text(
                        text = "No Riot accounts saved yet. Tap below to add your first or secondary Valorant account.",
                        color = TextSecondary,
                        fontSize = 12.5.sp,
                        modifier = Modifier.padding(16.dp),
                    )
                }
            } else {
                LazyColumn(
                    modifier = Modifier
                        .fillMaxWidth()
                        .heightIn(max = 340.dp),
                    verticalArrangement = Arrangement.spacedBy(10.dp),
                ) {
                    items(accounts, key = { it.puuid }) { acc ->
                        val isActive = activePuuid != null && acc.puuid.equals(activePuuid, ignoreCase = true)
                        val isExpired = acc.expiresAtEpochSecs > 0L && nowEpochSecs >= acc.expiresAtEpochSecs
                        val rankColor = acc.rankColorHex?.let { parseHexColor(it, ValCyan) } ?: ValCyan
                        val borderColor = if (isActive) ValCyan else SurfaceStroke

                        Surface(
                            color = SurfaceElevated,
                            shape = RoundedCornerShape(10.dp),
                            modifier = Modifier
                                .fillMaxWidth()
                                .border(
                                    width = if (isActive) 1.5.dp else 1.dp,
                                    color = borderColor,
                                    shape = RoundedCornerShape(10.dp),
                                )
                                .clickable {
                                    if (!isActive) onSwitchAccount(acc.puuid)
                                },
                        ) {
                            Row(
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .padding(horizontal = 12.dp, vertical = 11.dp),
                                verticalAlignment = Alignment.CenterVertically,
                            ) {
                                if (acc.rankIconUrl != null) {
                                    AsyncImage(
                                        model = acc.rankIconUrl,
                                        contentDescription = acc.rankTierName,
                                        modifier = Modifier.size(38.dp),
                                    )
                                } else {
                                    Box(
                                        modifier = Modifier
                                            .size(38.dp)
                                            .clip(CircleShape)
                                            .background(ObsidianBg)
                                            .border(1.dp, rankColor.copy(alpha = 0.5f), CircleShape),
                                        contentAlignment = Alignment.Center,
                                    ) {
                                        Text(
                                            text = acc.riotId.take(2).uppercase(),
                                            color = rankColor,
                                            fontWeight = FontWeight.Black,
                                            fontSize = 13.sp,
                                        )
                                    }
                                }
                                Spacer(modifier = Modifier.width(11.dp))

                                Column(modifier = Modifier.weight(1f)) {
                                    Text(
                                        text = acc.riotId,
                                        color = TextPrimary,
                                        fontWeight = FontWeight.Bold,
                                        fontSize = 14.sp,
                                        maxLines = 1,
                                        overflow = TextOverflow.Ellipsis,
                                    )
                                    Spacer(modifier = Modifier.height(2.dp))
                                    val subParts = buildList {
                                        add(acc.shard.uppercase())
                                        acc.rankTierName?.let { add(it) }
                                        if (isExpired && acc.ssoCookies.isNullOrBlank()) {
                                            add("Re-auth needed")
                                        }
                                    }
                                    Text(
                                        text = subParts.joinToString(" • "),
                                        color = if (isExpired && acc.ssoCookies.isNullOrBlank()) ValAmber else rankColor,
                                        fontSize = 11.5.sp,
                                        fontWeight = FontWeight.SemiBold,
                                        maxLines = 1,
                                        overflow = TextOverflow.Ellipsis,
                                    )
                                }

                                Spacer(modifier = Modifier.width(6.dp))
                                if (isActive) {
                                    TacticalBadge("ACTIVE", ValGreen)
                                    Spacer(modifier = Modifier.width(2.dp))
                                    IconButton(
                                        onClick = onRefreshCurrentAccount,
                                        modifier = Modifier.size(32.dp),
                                    ) {
                                        Icon(
                                            imageVector = Icons.Default.Refresh,
                                            contentDescription = "Re-authenticate active account",
                                            tint = ValCyan,
                                            modifier = Modifier.size(17.dp),
                                        )
                                    }
                                } else {
                                    OutlinedButton(
                                        onClick = { onSwitchAccount(acc.puuid) },
                                        contentPadding = PaddingValues(horizontal = 10.dp, vertical = 4.dp),
                                    ) {
                                        Text(
                                            text = "Switch",
                                            color = ValCyan,
                                            fontSize = 11.5.sp,
                                            fontWeight = FontWeight.Bold,
                                        )
                                    }
                                }

                                IconButton(
                                    onClick = { onRemoveAccount(acc.puuid) },
                                    modifier = Modifier.size(32.dp),
                                ) {
                                    Icon(
                                        imageVector = Icons.Default.DeleteOutline,
                                        contentDescription = "Remove ${acc.riotId}",
                                        tint = TextMuted,
                                        modifier = Modifier.size(18.dp),
                                    )
                                }
                            }
                        }
                    }
                }
            }

            Spacer(modifier = Modifier.height(14.dp))

            Button(
                onClick = onAddAnotherAccount,
                colors = ButtonDefaults.buttonColors(containerColor = ValRed),
                modifier = Modifier.fillMaxWidth(),
                contentPadding = PaddingValues(vertical = 12.dp),
            ) {
                Icon(
                    imageVector = Icons.Default.PersonAdd,
                    contentDescription = null,
                    modifier = Modifier.size(18.dp),
                )
                Spacer(modifier = Modifier.width(8.dp))
                Text(
                    text = if (accounts.isEmpty()) "Sign In with Riot (RSO)" else "Add Another Riot Account",
                    fontWeight = FontWeight.Black,
                    fontSize = 13.5.sp,
                )
            }

            if (accounts.size > 1 || (accounts.isNotEmpty() && activePuuid != null)) {
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(top = 4.dp),
                    horizontalArrangement = Arrangement.Center,
                ) {
                    TextButton(onClick = onSignOutAll) {
                        Text(
                            text = if (accounts.size > 1) "Sign Out All Accounts" else "Sign Out",
                            color = TextMuted,
                            fontSize = 12.sp,
                        )
                    }
                }
            }
            Spacer(modifier = Modifier.height(8.dp))
        }
    }
}
