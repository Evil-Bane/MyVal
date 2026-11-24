package com.myval.app.data

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Test

class AppUpdateCheckerTest {

    @Test
    fun versionComparisonHandlesSemverAndPrefixes() {
        assertTrue(AppUpdateChecker.isNewerVersion("v1.0.1", "1.0.0"))
        assertTrue(AppUpdateChecker.isNewerVersion("1.1.0", "1.0.9"))
        assertTrue(AppUpdateChecker.isNewerVersion("2.0.0", "1.9.9"))
        assertFalse(AppUpdateChecker.isNewerVersion("v1.0.0", "1.0.0"))
        assertFalse(AppUpdateChecker.isNewerVersion("0.9.9", "1.0.0"))
        assertFalse(AppUpdateChecker.isNewerVersion("1.0.0-rc1", "1.0.0"))
    }

    @Test
    fun parseReleaseJsonSelectsArm64ApkAndMetadata() {
        val sampleJson = """
            {
              "tag_name": "v1.1.0",
              "name": "MyVal v1.1.0 — Tactical Polish",
              "draft": false,
              "html_url": "https://github.com/Evil-Bane/MyVal/releases/tag/v1.1.0",
              "body": "- Live Match last 5 games fix\n- Faster OLED rendering",
              "published_at": "2026-11-22T00:00:00Z",
              "assets": [
                {
                  "name": "SHA256SUMS.txt",
                  "size": 128,
                  "browser_download_url": "https://github.com/Evil-Bane/MyVal/releases/download/v1.1.0/SHA256SUMS.txt"
                },
                {
                  "name": "MyVal-v1.1.0-arm64-v8a.apk",
                  "size": 14680064,
                  "browser_download_url": "https://github.com/Evil-Bane/MyVal/releases/download/v1.1.0/MyVal-v1.1.0-arm64-v8a.apk"
                }
              ]
            }
        """.trimIndent()

        val parsed = AppUpdateChecker.parseReleaseJson(sampleJson)
        assertNotNull(parsed)
        assertEquals("v1.1.0", parsed!!.tagName)
        assertEquals("1.1.0", parsed.versionName)
        assertEquals("MyVal-v1.1.0-arm64-v8a.apk", parsed.apkFileName)
        assertEquals("14.0 MB", AppUpdateChecker.formatBytes(parsed.apkSizeBytes))
        assertTrue(parsed.apkDownloadUrl!!.endsWith("MyVal-v1.1.0-arm64-v8a.apk"))
    }
}
