package com.myval.app.ui.main

import com.myval.app.data.AccountVault
import com.myval.app.data.StoredAccountRecord
import com.myval.app.ui.components.FAIR_PLAY_LOCK_DELAY_SECS
import com.myval.app.ui.components.fairPlayLockRemainingSecs
import com.myval.app.ui.components.formatDuration
import com.myval.app.ui.components.parseHexColor
import com.myval.app.ui.components.stackBadgeColor
import com.myval.app.ui.theme.ValCyan
import com.myval.app.ui.theme.ValRed
import junit.framework.TestCase.assertEquals
import junit.framework.TestCase.assertFalse
import junit.framework.TestCase.assertNotSame
import org.junit.Test

class MainScreenViewModelTest {
    @Test
    fun formatDuration_formatsHoursAndDaysCleanly() {
        assertEquals("00:00:00", formatDuration(0))
        assertEquals("00:00:00", formatDuration(-5))
        assertEquals("01:02:03", formatDuration(3723))
        assertEquals("1d 2h 3m", formatDuration(86400 + 7200 + 180))
    }

    @Test
    fun parseHexColor_handlesValidAndInvalidHex() {
        assertEquals(ValCyan, parseHexColor("invalid", ValCyan))
        val parsed = parseHexColor("#FF4655", ValCyan)
        assertNotSame(ValCyan, parsed)
    }

    @Test
    fun accountVault_holdsMultipleAccountsCleanly() {
        val acc1 = StoredAccountRecord(
            puuid = "p1",
            riotId = "PlayerOne#AP",
            region = "ap",
            shard = "ap",
            expiresAtEpochSecs = 1000L,
            sessionJson = "{}",
        )
        val acc2 = StoredAccountRecord(
            puuid = "p2",
            riotId = "PlayerTwo#AP",
            region = "ap",
            shard = "ap",
            expiresAtEpochSecs = 2000L,
            sessionJson = "{}",
        )
        val vault = AccountVault(activePuuid = "p2", accounts = listOf(acc1, acc2))
        assertEquals(2, vault.accounts.size)
        assertEquals("p2", vault.activePuuid)
    }

    @Test
    fun stackBadgeColor_neverCollidesWithSelfOrEnemyColors() {
        val groupColors = listOf("A", "B", "C", "D", "E").map { stackBadgeColor(it) }
        assertEquals(5, groupColors.toSet().size)
        groupColors.forEach { color ->
            assertFalse("Stack color must not collide with ValCyan (YOU/YOUR TEAM)", color == ValCyan)
            assertFalse("Stack color must not collide with ValRed (ENEMY TEAM)", color == ValRed)
        }
    }

    @Test
    fun fairPlayLockRemainingSecs_enforces15SecondAntiInstalockDelay() {
        assertEquals(15L, FAIR_PLAY_LOCK_DELAY_SECS)
        assertEquals(15, fairPlayLockRemainingSecs(firstSeenElapsedMs = 0L, nowElapsedMs = 5_000L))
        assertEquals(15, fairPlayLockRemainingSecs(firstSeenElapsedMs = 1_000L, nowElapsedMs = 1_000L))
        assertEquals(10, fairPlayLockRemainingSecs(firstSeenElapsedMs = 1_000L, nowElapsedMs = 6_000L))
        assertEquals(1, fairPlayLockRemainingSecs(firstSeenElapsedMs = 1_000L, nowElapsedMs = 15_999L))
        assertEquals(0, fairPlayLockRemainingSecs(firstSeenElapsedMs = 1_000L, nowElapsedMs = 16_000L))
        assertEquals(0, fairPlayLockRemainingSecs(firstSeenElapsedMs = 1_000L, nowElapsedMs = 45_000L))
    }
}
