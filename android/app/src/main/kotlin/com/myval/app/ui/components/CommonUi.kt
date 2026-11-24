package com.myval.app.ui.components

import androidx.compose.animation.core.LinearEasing
import java.util.Locale
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.spring
import androidx.compose.animation.core.tween
import androidx.compose.foundation.LocalIndication
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsHoveredAsState
import androidx.compose.foundation.interaction.collectIsPressedAsState
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxScope
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
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.Login
import androidx.compose.material.icons.filled.KeyboardArrowDown
import androidx.compose.material.icons.filled.Radar
import androidx.compose.material.icons.filled.SwitchAccount
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.pulltorefresh.PullToRefreshBox
import androidx.compose.material3.pulltorefresh.PullToRefreshDefaults
import androidx.compose.material3.pulltorefresh.rememberPullToRefreshState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.composed
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.myval.app.ui.theme.ObsidianBg
import com.myval.app.ui.theme.StackOrange
import com.myval.app.ui.theme.StackPink
import com.myval.app.ui.theme.StackSky
import com.myval.app.ui.theme.SurfaceCard
import com.myval.app.ui.theme.SurfaceElevated
import com.myval.app.ui.theme.SurfaceGlossTop
import com.myval.app.ui.theme.SurfaceStroke
import com.myval.app.ui.theme.TextMuted
import com.myval.app.ui.theme.TextPrimary
import com.myval.app.ui.theme.TextSecondary
import com.myval.app.ui.theme.ValAmber
import com.myval.app.ui.theme.ValCyan
import com.myval.app.ui.theme.ValGold
import com.myval.app.ui.theme.ValPurple
import com.myval.app.ui.theme.ValRed

val SharpCardShape = RoundedCornerShape(10.dp)
val InnerCardShape = RoundedCornerShape(8.dp)
val SharpChipShape = RoundedCornerShape(6.dp)
val SharpButtonShape = RoundedCornerShape(8.dp)

val GlossyCardBrush = Brush.verticalGradient(
    colors = listOf(SurfaceGlossTop.copy(alpha = 0.65f), SurfaceCard, Color(0xFF06080B)),
)

val GlossyElevatedBrush = Brush.verticalGradient(
    colors = listOf(Color(0xFF1F2838), SurfaceElevated, Color(0xFF0E131B)),
)

fun stackTintedCardBrush(stackColor: Color): Brush = Brush.horizontalGradient(
    colors = listOf(
        stackColor.copy(alpha = 0.24f),
        stackColor.copy(alpha = 0.08f),
        SurfaceCard,
    ),
)

fun parseHexColor(hex: String, fallback: Color = ValCyan): Color {
    val clean = hex.trim().removePrefix("#")
    if (clean.length != 6 && clean.length != 8) return fallback
    return runCatching {
        val parsed = clean.toLong(16)
        if (clean.length == 6) Color(0xFF000000L or parsed) else Color(parsed)
    }.getOrDefault(fallback)
}

fun formatDuration(totalSecs: Long): String {
    if (totalSecs <= 0) return "00:00:00"
    val days = totalSecs / 86400
    val hours = (totalSecs % 86400) / 3600
    val mins = (totalSecs % 3600) / 60
    val secs = totalSecs % 60
    return if (days > 0) {
        "${days}d ${hours}h ${mins}m"
    } else {
        "%02d:%02d:%02d".format(Locale.US, hours, mins, secs)
    }
}

fun stackBadgeColor(groupId: String?): Color = when (groupId) {
    "A" -> ValGold
    "B" -> ValPurple
    "C" -> StackOrange
    "D" -> StackPink
    "E" -> StackSky
    else -> ValGold
}

@Composable
fun TacticalSplashScreen(statusText: String) {
    val infinite = rememberInfiniteTransition(label = "splashGlow")
    val glowAlpha by infinite.animateFloat(
        initialValue = 0.45f,
        targetValue = 1f,
        animationSpec = infiniteRepeatable(tween(900, easing = LinearEasing), RepeatMode.Reverse),
        label = "glowAlpha",
    )

    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(ObsidianBg),
        contentAlignment = Alignment.Center,
    ) {
        Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 36.dp),
        ) {
            Box(
                modifier = Modifier
                    .size(72.dp)
                    .background(GlossyElevatedBrush, SharpCardShape)
                    .border(1.5.dp, ValRed.copy(alpha = 0.75f * glowAlpha), SharpCardShape),
                contentAlignment = Alignment.Center,
            ) {
                Icon(
                    imageVector = Icons.Default.Radar,
                    contentDescription = "MyVal",
                    tint = ValRed,
                    modifier = Modifier.size(34.dp),
                )
            }
            Spacer(modifier = Modifier.height(18.dp))
            Text(
                text = "MYVAL",
                color = TextPrimary,
                fontSize = 26.sp,
                fontWeight = FontWeight.Black,
                letterSpacing = 5.sp,
            )
            Spacer(modifier = Modifier.height(26.dp))
            LinearProgressIndicator(
                modifier = Modifier
                    .width(180.dp)
                    .height(2.5.dp)
                    .clip(SharpChipShape),
                color = ValRed,
                trackColor = SurfaceElevated,
            )
            Spacer(modifier = Modifier.height(12.dp))
            Text(
                text = statusText,
                color = TextSecondary,
                fontSize = 11.5.sp,
                fontWeight = FontWeight.Medium,
                modifier = Modifier.alpha(0.9f),
            )
        }
    }
}

@Composable
fun LiveNetworkStatusBar(
    message: String?,
    isSlow: Boolean = false,
) {
    if (message.isNullOrBlank()) return
    val accent = if (isSlow) ValAmber else ValCyan
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .background(accent.copy(alpha = 0.11f))
            .border(0.5.dp, accent.copy(alpha = 0.3f))
            .padding(horizontal = 14.dp, vertical = 5.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        CircularProgressIndicator(
            modifier = Modifier.size(11.dp),
            color = accent,
            strokeWidth = 1.6.dp,
        )
        Spacer(modifier = Modifier.width(8.dp))
        Text(
            text = message,
            color = accent,
            fontSize = 11.sp,
            fontWeight = FontWeight.SemiBold,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
    }
}

@Composable
fun TacticalBadge(
    text: String,
    accent: Color,
    modifier: Modifier = Modifier,
) {
    Box(
        modifier = modifier
            .background(accent.copy(alpha = 0.14f), SharpChipShape)
            .border(1.dp, accent.copy(alpha = 0.5f), SharpChipShape)
            .padding(horizontal = 6.dp, vertical = 2.dp),
    ) {
        Text(
            text = text,
            color = accent,
            fontSize = 10.sp,
            fontWeight = FontWeight.ExtraBold,
            letterSpacing = 0.4.sp,
            maxLines = 1,
            softWrap = false,
            overflow = TextOverflow.Ellipsis,
        )
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MyValPullToRefreshBox(
    isRefreshing: Boolean,
    onRefresh: () -> Unit,
    accent: Color = ValCyan,
    modifier: Modifier = Modifier,
    content: @Composable BoxScope.() -> Unit,
) {
    val state = rememberPullToRefreshState()
    PullToRefreshBox(
        isRefreshing = isRefreshing,
        onRefresh = onRefresh,
        state = state,
        modifier = modifier,
        indicator = {
            PullToRefreshDefaults.Indicator(
                state = state,
                isRefreshing = isRefreshing,
                containerColor = SurfaceElevated,
                color = accent,
                modifier = Modifier.align(Alignment.TopCenter),
            )
        },
        content = content,
    )
}

fun Modifier.tactileScale(
    interactionSource: MutableInteractionSource,
    enabled: Boolean = true,
    pressedScale: Float = 0.96f,
): Modifier = composed {
    val isPressed by interactionSource.collectIsPressedAsState()
    val isHovered by interactionSource.collectIsHoveredAsState()
    val scale by animateFloatAsState(
        targetValue = when {
            isPressed && enabled -> pressedScale
            isHovered && enabled -> 1.015f
            else -> 1f
        },
        animationSpec = spring(dampingRatio = 0.62f, stiffness = 540f),
        label = "tactileScale",
    )
    this.graphicsLayer {
        scaleX = scale
        scaleY = scale
    }
}

fun Modifier.tactileClickable(
    enabled: Boolean = true,
    pressedScale: Float = 0.96f,
    onClick: () -> Unit,
): Modifier = composed {
    val interactionSource = remember { MutableInteractionSource() }
    this
        .tactileScale(
            interactionSource = interactionSource,
            enabled = enabled,
            pressedScale = pressedScale,
        )
        .clickable(
            interactionSource = interactionSource,
            indication = LocalIndication.current,
            enabled = enabled,
            onClick = onClick,
        )
}

@Composable
fun AccountSwitcherChip(
    riotId: String,
    shard: String,
    accountCount: Int,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val countSuffix = if (accountCount > 1) " ($accountCount)" else ""
    Row(
        modifier = modifier
            .clip(SharpChipShape)
            .background(GlossyElevatedBrush)
            .border(1.dp, SurfaceStroke, SharpChipShape)
            .tactileClickable(pressedScale = 0.95f, onClick = onClick)
            .padding(horizontal = 8.dp, vertical = 4.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(
            text = "$riotId • ${shard.uppercase()}$countSuffix",
            color = TextPrimary,
            fontSize = 11.sp,
            fontWeight = FontWeight.Bold,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            modifier = Modifier.weight(1f, fill = false),
        )
        Spacer(modifier = Modifier.width(4.dp))
        Icon(
            imageVector = Icons.Default.KeyboardArrowDown,
            contentDescription = "Switch Riot account",
            tint = ValCyan,
            modifier = Modifier.size(14.dp),
        )
    }
}

@Composable
fun AuthRequiredCard(
    title: String,
    subtitle: String,
    onOpenLogin: () -> Unit,
    savedAccountCount: Int = 0,
    onOpenAccounts: (() -> Unit)? = null,
    modifier: Modifier = Modifier,
) {
    Surface(
        color = Color.Transparent,
        shape = SharpCardShape,
        modifier = modifier
            .fillMaxWidth()
            .background(GlossyCardBrush, SharpCardShape)
            .border(1.dp, SurfaceStroke, SharpCardShape),
    ) {
        Column(modifier = Modifier.padding(18.dp)) {
            Text(
                text = title,
                fontSize = 16.sp,
                fontWeight = FontWeight.Black,
                letterSpacing = 0.6.sp,
                color = TextPrimary,
            )
            Spacer(modifier = Modifier.height(6.dp))
            Text(
                text = subtitle,
                fontSize = 12.5.sp,
                color = TextSecondary,
            )
            Spacer(modifier = Modifier.height(14.dp))
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                val loginInteraction = remember { MutableInteractionSource() }
                Button(
                    onClick = onOpenLogin,
                    interactionSource = loginInteraction,
                    modifier = Modifier.tactileScale(loginInteraction, pressedScale = 0.95f),
                    shape = SharpButtonShape,
                    colors = ButtonDefaults.buttonColors(containerColor = ValRed),
                    contentPadding = PaddingValues(horizontal = 14.dp, vertical = 10.dp),
                ) {
                    Icon(
                        Icons.AutoMirrored.Filled.Login,
                        contentDescription = null,
                        modifier = Modifier.size(16.dp),
                    )
                    Spacer(modifier = Modifier.width(6.dp))
                    Text("Sign In with Riot", fontWeight = FontWeight.Bold, fontSize = 12.5.sp)
                }
                if (savedAccountCount > 0 && onOpenAccounts != null) {
                    val acctInteraction = remember { MutableInteractionSource() }
                    OutlinedButton(
                        onClick = onOpenAccounts,
                        interactionSource = acctInteraction,
                        modifier = Modifier.tactileScale(acctInteraction, pressedScale = 0.95f),
                        shape = SharpButtonShape,
                        contentPadding = PaddingValues(horizontal = 12.dp, vertical = 10.dp),
                    ) {
                        Icon(
                            Icons.Default.SwitchAccount,
                            contentDescription = null,
                            modifier = Modifier.size(15.dp),
                        )
                        Spacer(modifier = Modifier.width(6.dp))
                        Text("Accounts ($savedAccountCount)", fontWeight = FontWeight.Bold, fontSize = 12.sp)
                    }
                }
            }
        }
    }
}

const val FAIR_PLAY_LOCK_DELAY_SECS: Long = 15L

fun fairPlayLockRemainingSecs(firstSeenElapsedMs: Long, nowElapsedMs: Long): Int {
    if (firstSeenElapsedMs <= 0L || nowElapsedMs < firstSeenElapsedMs) {
        return FAIR_PLAY_LOCK_DELAY_SECS.toInt()
    }
    val delayMs = FAIR_PLAY_LOCK_DELAY_SECS * 1_000L
    val elapsedMs = nowElapsedMs - firstSeenElapsedMs
    if (elapsedMs >= delayMs) return 0
    return ((delayMs - elapsedMs + 999L) / 1_000L).toInt().coerceAtLeast(1)
}
