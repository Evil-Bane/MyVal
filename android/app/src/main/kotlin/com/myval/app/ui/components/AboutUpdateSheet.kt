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
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.OpenInNew
import androidx.compose.material.icons.filled.BugReport
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.filled.Download
import androidx.compose.material.icons.filled.NewReleases
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material.icons.filled.Security
import androidx.compose.material.icons.filled.SystemUpdate
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.LinearProgressIndicator
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
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.myval.app.BuildConfig
import com.myval.app.data.AppReleaseInfo
import com.myval.app.data.AppUpdateChecker
import com.myval.app.ui.theme.ObsidianBg
import com.myval.app.ui.theme.SurfaceCard
import com.myval.app.ui.theme.SurfaceElevated
import com.myval.app.ui.theme.SurfaceStroke
import com.myval.app.ui.theme.TextMuted
import com.myval.app.ui.theme.TextPrimary
import com.myval.app.ui.theme.TextSecondary
import com.myval.app.ui.theme.ValCyan
import com.myval.app.ui.theme.ValGold
import com.myval.app.ui.theme.ValRed
import com.myval.app.ui.theme.ValWinGreen

@Composable
private fun UpdateDownloadProgressStrip(
    isDownloadingUpdate: Boolean,
    updateDownloadProgress: Float?,
    updateDownloadLabel: String?,
) {
    if (!isDownloadingUpdate && updateDownloadLabel.isNullOrBlank()) return
    Column(modifier = Modifier.fillMaxWidth()) {
        Spacer(modifier = Modifier.height(8.dp))
        if (isDownloadingUpdate) {
            if (updateDownloadProgress != null) {
                LinearProgressIndicator(
                    progress = { updateDownloadProgress.coerceIn(0f, 1f) },
                    color = ValCyan,
                    trackColor = SurfaceElevated,
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(4.dp)
                        .clip(SharpChipShape),
                )
            } else {
                LinearProgressIndicator(
                    color = ValCyan,
                    trackColor = SurfaceElevated,
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(4.dp)
                        .clip(SharpChipShape),
                )
            }
            Spacer(modifier = Modifier.height(5.dp))
        }
        if (!updateDownloadLabel.isNullOrBlank()) {
            Text(
                text = updateDownloadLabel,
                color = if (isDownloadingUpdate) ValCyan else TextSecondary,
                fontSize = 11.sp,
                fontWeight = FontWeight.SemiBold,
            )
        }
    }
}

@Composable
fun UpdateAvailableDialog(
    release: AppReleaseInfo,
    isDownloadingUpdate: Boolean = false,
    updateDownloadProgress: Float? = null,
    updateDownloadLabel: String? = null,
    hasDownloadedApk: Boolean = false,
    onDownloadApk: () -> Unit,
    onOpenReleasePage: () -> Unit,
    onDismiss: () -> Unit,
) {
    val sizeLabel = AppUpdateChecker.formatBytes(release.apkSizeBytes)
    AlertDialog(
        onDismissRequest = { if (!isDownloadingUpdate) onDismiss() },
        containerColor = SurfaceCard,
        shape = SharpCardShape,
        icon = {
            Box(
                modifier = Modifier
                    .size(44.dp)
                    .background(ValCyan.copy(alpha = 0.14f), SharpCardShape)
                    .border(1.dp, ValCyan.copy(alpha = 0.5f), SharpCardShape),
                contentAlignment = Alignment.Center,
            ) {
                Icon(
                    imageVector = Icons.Default.SystemUpdate,
                    contentDescription = "Update available",
                    tint = ValCyan,
                    modifier = Modifier.size(24.dp),
                )
            }
        },
        title = {
            Column(horizontalAlignment = Alignment.CenterHorizontally) {
                Text(
                    text = "UPDATE AVAILABLE",
                    color = TextPrimary,
                    fontWeight = FontWeight.Black,
                    fontSize = 16.sp,
                    letterSpacing = 0.8.sp,
                )
                Spacer(modifier = Modifier.height(6.dp))
                Row(
                    horizontalArrangement = Arrangement.spacedBy(6.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    TacticalBadge("v${BuildConfig.VERSION_NAME}", TextSecondary)
                    Text("→", color = ValCyan, fontWeight = FontWeight.Black, fontSize = 12.sp)
                    TacticalBadge("v${release.versionName}", ValWinGreen)
                    if (sizeLabel != null) {
                        TacticalBadge(sizeLabel, ValGold)
                    }
                }
            }
        },
        text = {
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .heightIn(max = 260.dp)
                    .verticalScroll(rememberScrollState()),
            ) {
                if (release.title.isNotBlank() && !release.title.equals(release.tagName, ignoreCase = true)) {
                    Text(
                        text = release.title,
                        color = TextPrimary,
                        fontWeight = FontWeight.Bold,
                        fontSize = 13.sp,
                    )
                    Spacer(modifier = Modifier.height(6.dp))
                }
                val notes = release.releaseNotes.ifBlank {
                    "Performance improvements, bug fixes, and the latest Valorant telemetry updates."
                }
                Surface(
                    color = SurfaceElevated,
                    shape = InnerCardShape,
                    modifier = Modifier
                        .fillMaxWidth()
                        .border(0.75.dp, SurfaceStroke, InnerCardShape),
                ) {
                    Text(
                        text = notes,
                        color = TextSecondary,
                        fontSize = 12.sp,
                        lineHeight = 17.sp,
                        modifier = Modifier.padding(10.dp),
                    )
                }
                UpdateDownloadProgressStrip(
                    isDownloadingUpdate = isDownloadingUpdate,
                    updateDownloadProgress = updateDownloadProgress,
                    updateDownloadLabel = updateDownloadLabel,
                )
            }
        },
        confirmButton = {
            Button(
                enabled = !isDownloadingUpdate,
                onClick = onDownloadApk,
                shape = SharpButtonShape,
                colors = ButtonDefaults.buttonColors(
                    containerColor = ValRed,
                    contentColor = TextPrimary,
                ),
            ) {
                if (isDownloadingUpdate) {
                    CircularProgressIndicator(
                        modifier = Modifier.size(15.dp),
                        color = TextPrimary,
                        strokeWidth = 2.dp,
                    )
                } else {
                    Icon(
                        imageVector = Icons.Default.Download,
                        contentDescription = null,
                        modifier = Modifier.size(16.dp),
                    )
                }
                Spacer(modifier = Modifier.width(6.dp))
                Text(
                    text = when {
                        isDownloadingUpdate -> "Downloading..."
                        hasDownloadedApk -> "Install APK"
                        release.apkDownloadUrl != null -> "Download APK"
                        else -> "Get Update"
                    },
                    fontWeight = FontWeight.Black,
                    fontSize = 12.5.sp,
                )
            }
        },
        dismissButton = {
            Row(horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                TextButton(onClick = onOpenReleasePage) {
                    Text("Release Page", color = ValCyan, fontSize = 12.sp, fontWeight = FontWeight.SemiBold)
                }
                TextButton(enabled = !isDownloadingUpdate, onClick = onDismiss) {
                    Text("Later", color = TextSecondary, fontSize = 12.sp)
                }
            }
        },
    )
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun AboutUpdateBottomSheet(
    availableUpdate: AppReleaseInfo?,
    isCheckingUpdate: Boolean,
    updateCheckStatus: String?,
    isDownloadingUpdate: Boolean = false,
    updateDownloadProgress: Float? = null,
    updateDownloadLabel: String? = null,
    hasDownloadedApk: Boolean = false,
    onCheckForUpdates: () -> Unit,
    onDownloadUpdate: () -> Unit,
    onOpenUrl: (String) -> Unit,
    onDismiss: () -> Unit,
) {
    val sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)
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
                .verticalScroll(rememberScrollState())
                .padding(horizontal = 18.dp, vertical = 6.dp),
        ) {
            // Identity header
            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.SpaceBetween,
            ) {
                Column {
                    Text(
                        text = "MYVAL",
                        color = ValRed,
                        fontWeight = FontWeight.Black,
                        fontSize = 20.sp,
                        letterSpacing = 2.sp,
                    )
                    Text(
                        text = "Valorant Companion",
                        color = TextSecondary,
                        fontSize = 11.5.sp,
                    )
                }
                TacticalBadge("v${BuildConfig.VERSION_NAME}", ValCyan)
            }

            Spacer(modifier = Modifier.height(14.dp))

            // Update card
            val hasUpdate = availableUpdate != null
            val updateAccent = if (hasUpdate) ValWinGreen else ValCyan
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .clip(SharpCardShape)
                    .background(GlossyElevatedBrush)
                    .border(1.dp, updateAccent.copy(alpha = 0.5f), SharpCardShape)
                    .padding(14.dp),
            ) {
                Column {
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.SpaceBetween,
                    ) {
                        Row(verticalAlignment = Alignment.CenterVertically) {
                            Icon(
                                imageVector = if (hasUpdate) Icons.Default.NewReleases else Icons.Default.CheckCircle,
                                contentDescription = null,
                                tint = updateAccent,
                                modifier = Modifier.size(18.dp),
                            )
                            Spacer(modifier = Modifier.width(8.dp))
                            Text(
                                text = if (availableUpdate != null) {
                                    "New Version Ready: v${availableUpdate.versionName}"
                                } else {
                                    "App Updates"
                                },
                                color = TextPrimary,
                                fontWeight = FontWeight.Bold,
                                fontSize = 13.5.sp,
                            )
                        }
                        if (isCheckingUpdate) {
                            CircularProgressIndicator(
                                modifier = Modifier.size(16.dp),
                                color = ValCyan,
                                strokeWidth = 2.dp,
                            )
                        }
                    }

                    Spacer(modifier = Modifier.height(6.dp))
                    Text(
                        text = when {
                            availableUpdate != null -> availableUpdate.releaseNotes.ifBlank {
                                "Tap below to download the signed v${availableUpdate.versionName} APK directly."
                            }
                            !updateCheckStatus.isNullOrBlank() -> updateCheckStatus
                            else -> "Checks official GitHub Releases at ${AppUpdateChecker.REPO_SLUG} for newer signed builds."
                        },
                        color = TextSecondary,
                        fontSize = 11.5.sp,
                        lineHeight = 16.sp,
                        maxLines = 4,
                    )

                    UpdateDownloadProgressStrip(
                        isDownloadingUpdate = isDownloadingUpdate,
                        updateDownloadProgress = updateDownloadProgress,
                        updateDownloadLabel = updateDownloadLabel,
                    )

                    Spacer(modifier = Modifier.height(10.dp))
                    Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        if (hasUpdate) {
                            Button(
                                enabled = !isDownloadingUpdate,
                                onClick = onDownloadUpdate,
                                shape = SharpButtonShape,
                                contentPadding = PaddingValues(horizontal = 14.dp, vertical = 8.dp),
                                colors = ButtonDefaults.buttonColors(containerColor = ValRed),
                                modifier = Modifier.weight(1f),
                            ) {
                                if (isDownloadingUpdate) {
                                    CircularProgressIndicator(
                                        modifier = Modifier.size(14.dp),
                                        color = TextPrimary,
                                        strokeWidth = 2.dp,
                                    )
                                } else {
                                    Icon(Icons.Default.Download, contentDescription = null, modifier = Modifier.size(15.dp))
                                }
                                Spacer(modifier = Modifier.width(6.dp))
                                Text(
                                    text = when {
                                        isDownloadingUpdate -> "Downloading..."
                                        hasDownloadedApk -> "Install APK"
                                        else -> "Download APK"
                                    },
                                    fontWeight = FontWeight.Bold,
                                    fontSize = 12.sp,
                                )
                            }
                            OutlinedButton(
                                onClick = { onOpenUrl(availableUpdate?.htmlUrl ?: AppUpdateChecker.RELEASES_URL) },
                                shape = SharpButtonShape,
                                contentPadding = PaddingValues(horizontal = 12.dp, vertical = 8.dp),
                            ) {
                                Icon(
                                    Icons.AutoMirrored.Filled.OpenInNew,
                                    contentDescription = null,
                                    tint = ValCyan,
                                    modifier = Modifier.size(14.dp),
                                )
                                Spacer(modifier = Modifier.width(5.dp))
                                Text("GitHub", color = ValCyan, fontWeight = FontWeight.Bold, fontSize = 12.sp)
                            }
                        } else {
                            OutlinedButton(
                                enabled = !isCheckingUpdate,
                                onClick = onCheckForUpdates,
                                shape = SharpButtonShape,
                                contentPadding = PaddingValues(horizontal = 12.dp, vertical = 8.dp),
                                modifier = Modifier.fillMaxWidth(),
                            ) {
                                Icon(
                                    Icons.Default.Refresh,
                                    contentDescription = null,
                                    tint = ValCyan,
                                    modifier = Modifier.size(15.dp),
                                )
                                Spacer(modifier = Modifier.width(6.dp))
                                Text(
                                    text = if (isCheckingUpdate) "Checking..." else "Check for Updates",
                                    color = ValCyan,
                                    fontWeight = FontWeight.Bold,
                                    fontSize = 12.sp,
                                )
                            }
                        }
                    }
                }
            }

            Spacer(modifier = Modifier.height(14.dp))

            // Zero-middleman security section (clean, borderless recessed strip)
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .clip(InnerCardShape)
                    .background(ObsidianBg)
                    .padding(12.dp),
                verticalAlignment = Alignment.Top,
            ) {
                Icon(
                    imageVector = Icons.Default.Security,
                    contentDescription = null,
                    tint = ValWinGreen,
                    modifier = Modifier.size(18.dp),
                )
                Spacer(modifier = Modifier.width(10.dp))
                Column {
                    Text(
                        text = "Fair-Play, Privacy & Voluntary Terms",
                        color = TextPrimary,
                        fontWeight = FontWeight.Bold,
                        fontSize = 12.5.sp,
                    )
                    Spacer(modifier = Modifier.height(3.dp))
                    Text(
                        text = "MyVal is an open-source, voluntary client-side companion — nobody forces sign-in. Your Riot login happens directly on auth.riotgames.com and tokens stay encrypted on-device in Android Keystore. The developer stores zero data. To prevent unfair advantage, Remote Agent Hover & Lock enforce a 15s fair-play wait (instapicking & instalocking disabled) with zero overlays or PC memory access. Use at your own discretion.",
                        color = TextSecondary,
                        fontSize = 11.sp,
                        lineHeight = 16.sp,
                    )
                }
            }

            Spacer(modifier = Modifier.height(12.dp))
            HorizontalDivider(color = SurfaceStroke, thickness = 0.75.dp)
            Spacer(modifier = Modifier.height(6.dp))

            AboutActionRow(
                icon = Icons.Default.SystemUpdate,
                title = "Official Releases & Changelog",
                subtitle = AppUpdateChecker.REPO_SLUG,
                onClick = { onOpenUrl(AppUpdateChecker.RELEASES_URL) },
            )
            AboutActionRow(
                icon = Icons.Default.BugReport,
                title = "Report a Bug or Request a Feature",
                subtitle = "Open an issue on GitHub",
                onClick = { onOpenUrl(AppUpdateChecker.ISSUES_URL) },
            )
            AboutActionRow(
                icon = Icons.AutoMirrored.Filled.OpenInNew,
                title = "Auto-Update with Obtainium",
                subtitle = "Track GitHub Releases automatically on Android",
                onClick = { onOpenUrl(AppUpdateChecker.OBTAINIUM_URL) },
            )

            Spacer(modifier = Modifier.height(12.dp))
            Text(
                text = "MyVal isn't endorsed by Riot Games and doesn't reflect the views or opinions of Riot Games or anyone officially involved in producing or managing Riot Games properties. Riot Games and VALORANT are trademarks or registered trademarks of Riot Games, Inc.",
                color = TextMuted,
                fontSize = 10.sp,
                lineHeight = 14.sp,
            )
            Spacer(modifier = Modifier.height(18.dp))
        }
    }
}

@Composable
private fun AboutActionRow(
    icon: ImageVector,
    title: String,
    subtitle: String,
    onClick: () -> Unit,
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clip(InnerCardShape)
            .clickable(onClick = onClick)
            .padding(horizontal = 6.dp, vertical = 10.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Icon(
            imageVector = icon,
            contentDescription = null,
            tint = ValCyan,
            modifier = Modifier.size(18.dp),
        )
        Spacer(modifier = Modifier.width(12.dp))
        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = title,
                color = TextPrimary,
                fontWeight = FontWeight.SemiBold,
                fontSize = 13.sp,
            )
            Text(
                text = subtitle,
                color = TextMuted,
                fontSize = 11.sp,
            )
        }
        Icon(
            imageVector = Icons.AutoMirrored.Filled.OpenInNew,
            contentDescription = null,
            tint = TextMuted,
            modifier = Modifier.size(15.dp),
        )
    }
}
