package com.myval.app.ui.explorer

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Test

class ExplorerScreenRichTextTest {

    @Test
    fun parseRiotRichText_stripsMarkupAndPreservesSpans() {
        val raw = "EQUIP <whitebold>Wingman</>. FIRE to send it forward. " +
            "<debuffred>Chases</> the first enemy, <teal>concussing</> on impact."
        val annotated = parseRiotRichText(raw)
        assertEquals(
            "EQUIP Wingman. FIRE to send it forward. Chases the first enemy, concussing on impact.",
            annotated.text,
        )
        assertFalse(annotated.text.contains('<'))
        assertFalse(annotated.text.contains('>'))
        assertEquals(3, annotated.spanStyles.size)
    }

    @Test
    fun formatAbilitySlot_mapsStandardKeybindsCleanly() {
        assertEquals("Q", formatAbilitySlot("Ability1"))
        assertEquals("E", formatAbilitySlot("Ability2"))
        assertEquals("C", formatAbilitySlot("Grenade"))
        assertEquals("X", formatAbilitySlot("Ultimate"))
        assertEquals("PASSIVE", formatAbilitySlot("Passive"))
    }
}
