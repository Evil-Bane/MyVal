package com.myval.app.ui.theme

import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Shapes
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.unit.dp

private val MyValDarkScheme = darkColorScheme(
    primary = ValRed,
    onPrimary = TextPrimary,
    primaryContainer = SurfaceElevated,
    onPrimaryContainer = TextPrimary,
    secondary = ValCyan,
    onSecondary = ObsidianBg,
    tertiary = ValGold,
    background = ObsidianBg,
    onBackground = TextPrimary,
    surface = SurfaceCard,
    onSurface = TextPrimary,
    surfaceVariant = SurfaceElevated,
    onSurfaceVariant = TextSecondary,
    outline = SurfaceStroke,
)

// Balanced crisp-yet-smooth tactical geometry (never overly boxy, never bubbly pills)
private val MyValBalancedShapes = Shapes(
    extraSmall = RoundedCornerShape(4.dp),
    small = RoundedCornerShape(6.dp),
    medium = RoundedCornerShape(10.dp),
    large = RoundedCornerShape(12.dp),
    extraLarge = RoundedCornerShape(14.dp),
)

@Composable
fun MyValTheme(
    content: @Composable () -> Unit,
) {
    MaterialTheme(
        colorScheme = MyValDarkScheme,
        typography = Typography,
        shapes = MyValBalancedShapes,
        content = content,
    )
}
