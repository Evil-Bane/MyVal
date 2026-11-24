package com.myval.app.data

import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.os.SystemClock
import android.provider.Settings
import androidx.core.content.FileProvider
import com.myval.app.BuildConfig
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import okhttp3.OkHttpClient
import okhttp3.Request
import org.json.JSONObject
import java.io.File
import java.io.FileOutputStream
import java.net.HttpURLConnection
import java.net.URL
import java.util.Locale
import java.util.concurrent.TimeUnit

sealed interface ApkInstallOutcome {
    data object InstallerLaunched : ApkInstallOutcome
    data object NeedsInstallPermission : ApkInstallOutcome
    data class Failed(val reason: String) : ApkInstallOutcome
}

data class AppReleaseInfo(
    val tagName: String,
    val versionName: String,
    val title: String,
    val releaseNotes: String,
    val htmlUrl: String,
    val apkDownloadUrl: String?,
    val apkFileName: String?,
    val apkSizeBytes: Long?,
    val publishedAt: String?,
)

object AppUpdateChecker {
    const val REPO_SLUG: String = BuildConfig.GITHUB_REPO
    const val REPO_URL: String = "https://github.com/${BuildConfig.GITHUB_REPO}"
    const val RELEASES_URL: String = "https://github.com/${BuildConfig.GITHUB_REPO}/releases"
    const val ISSUES_URL: String = "https://github.com/${BuildConfig.GITHUB_REPO}/issues"
    const val OBTAINIUM_URL: String = "https://obtainium.imranr.dev/"

    private const val LATEST_API_URL: String =
        "https://api.github.com/repos/${BuildConfig.GITHUB_REPO}/releases/latest"

    /**
     * Fetches the latest published GitHub Release for Evil-Bane/MyVal.
     * Returns null when the repo has no releases yet (HTTP 404) or is unreachable on silent check.
     */
    fun fetchLatestRelease(): Result<AppReleaseInfo?> = runCatching {
        val conn = (URL(LATEST_API_URL).openConnection() as HttpURLConnection).apply {
            requestMethod = "GET"
            connectTimeout = 7_000
            readTimeout = 8_000
            setRequestProperty("Accept", "application/vnd.github+json")
            setRequestProperty("User-Agent", "MyVal-Android/${BuildConfig.VERSION_NAME}")
        }
        try {
            val code = conn.responseCode
            if (code == HttpURLConnection.HTTP_NOT_FOUND) {
                return@runCatching null
            }
            if (code !in 200..299) {
                error("GitHub Releases returned HTTP $code")
            }
            val body = conn.inputStream.bufferedReader(Charsets.UTF_8).use { it.readText() }
            parseReleaseJson(body)
        } finally {
            conn.disconnect()
        }
    }

    internal fun parseReleaseJson(rawJson: String): AppReleaseInfo? {
        val obj = JSONObject(rawJson)
        if (obj.optBoolean("draft", false)) return null
        val tag = obj.optString("tag_name", "").trim()
        if (tag.isEmpty()) return null

        val cleanVersion = normalizeVersion(tag)
        val title = obj.optString("name", "").trim().ifEmpty { "MyVal v$cleanVersion" }
        val notes = obj.optString("body", "").trim()
        val htmlUrl = obj.optString("html_url", RELEASES_URL).trim().ifEmpty { RELEASES_URL }
        val publishedAt = obj.optString("published_at", "").trim().takeIf { it.isNotEmpty() }

        var apkUrl: String? = null
        var apkName: String? = null
        var apkSize: Long? = null

        val assets = obj.optJSONArray("assets")
        if (assets != null) {
            for (i in 0 until assets.length()) {
                val asset = assets.optJSONObject(i) ?: continue
                val name = asset.optString("name", "").trim()
                val dl = asset.optString("browser_download_url", "").trim()
                if (name.endsWith(".apk", ignoreCase = true) && dl.isNotEmpty()) {
                    // Prefer arm64-v8a or universal APK when multiple APKs exist
                    val isArm64 = name.contains("arm64", ignoreCase = true)
                    if (apkUrl == null || isArm64) {
                        apkUrl = dl
                        apkName = name
                        apkSize = asset.optLong("size", 0L).takeIf { it > 0L }
                    }
                }
            }
        }

        return AppReleaseInfo(
            tagName = tag,
            versionName = cleanVersion,
            title = title,
            releaseNotes = notes,
            htmlUrl = htmlUrl,
            apkDownloadUrl = apkUrl,
            apkFileName = apkName,
            apkSizeBytes = apkSize,
            publishedAt = publishedAt,
        )
    }

    fun normalizeVersion(raw: String): String =
        raw.trim().removePrefix("v").removePrefix("V").substringBefore("-").substringBefore("+").trim()

    /**
     * Semantic version comparison: returns true strictly when [remoteVersion] > [currentVersion].
     */
    fun isNewerVersion(remoteVersion: String, currentVersion: String = BuildConfig.VERSION_NAME): Boolean {
        val rParts = normalizeVersion(remoteVersion).split(".").mapNotNull { it.toIntOrNull() }
        val cParts = normalizeVersion(currentVersion).split(".").mapNotNull { it.toIntOrNull() }
        if (rParts.isEmpty() || cParts.isEmpty()) return false
        val maxLen = maxOf(rParts.size, cParts.size)
        for (i in 0 until maxLen) {
            val r = rParts.getOrElse(i) { 0 }
            val c = cParts.getOrElse(i) { 0 }
            if (r != c) return r > c
        }
        return false
    }

    fun formatBytes(bytes: Long?): String? {
        if (bytes == null || bytes <= 0L) return null
        val mb = bytes.toDouble() / (1024.0 * 1024.0)
        return "%.1f MB".format(Locale.US, mb)
    }

    private val downloadClient: OkHttpClient by lazy {
        OkHttpClient.Builder()
            .followRedirects(true)
            .followSslRedirects(true)
            .connectTimeout(15, TimeUnit.SECONDS)
            .readTimeout(60, TimeUnit.SECONDS)
            .callTimeout(10, TimeUnit.MINUTES)
            .build()
    }

    /**
     * Downloads the release APK directly via OkHttp into app cache (`updates/`),
     * bypassing Xiaomi/MIUI and Android system DownloadManager stalls on GitHub redirects.
     */
    suspend fun downloadApkToCache(
        context: Context,
        release: AppReleaseInfo,
        onProgress: (bytesRead: Long, totalBytes: Long) -> Unit,
    ): Result<File> = withContext(Dispatchers.IO) {
        runCatching {
            val downloadUrl = release.apkDownloadUrl
                ?: error("No APK asset found on GitHub Release ${release.tagName}")
            val rawName = (release.apkFileName?.takeIf { it.isNotBlank() }
                ?: "MyVal-v${release.versionName}.apk")
                .replace(Regex("[^A-Za-z0-9._-]"), "_")
            val fileName = if (rawName.contains(release.versionName, ignoreCase = true)) {
                rawName
            } else {
                "MyVal-v${release.versionName}-$rawName"
            }

            val updatesDir = File(context.cacheDir, "updates").apply { mkdirs() }
            updatesDir.listFiles()?.forEach { old ->
                if (old.name != fileName) old.delete()
            }

            val finalFile = File(updatesDir, fileName)
            val tempFile = File(updatesDir, "$fileName.part")
            if (tempFile.exists()) tempFile.delete()

            val request = Request.Builder()
                .url(downloadUrl)
                .header("User-Agent", "MyVal-Android/${BuildConfig.VERSION_NAME}")
                .header("Accept", "application/vnd.android.package-archive, */*")
                .get()
                .build()

            downloadClient.newCall(request).execute().use { response ->
                if (!response.isSuccessful) {
                    error("GitHub download returned HTTP ${response.code}")
                }
                val body = response.body ?: error("Empty download response body")
                val totalBytes = body.contentLength().takeIf { it > 0L }
                    ?: release.apkSizeBytes
                    ?: -1L

                var bytesRead = 0L
                var lastReportMs = 0L
                onProgress(0L, totalBytes)

                body.byteStream().use { input ->
                    FileOutputStream(tempFile).use { output ->
                        val buffer = ByteArray(32 * 1024)
                        while (true) {
                            val read = input.read(buffer)
                            if (read <= 0) break
                            output.write(buffer, 0, read)
                            bytesRead += read
                            val now = SystemClock.elapsedRealtime()
                            if (now - lastReportMs >= 100L) {
                                lastReportMs = now
                                onProgress(bytesRead, totalBytes)
                            }
                        }
                        output.flush()
                    }
                }

                if (bytesRead <= 500_000L) {
                    tempFile.delete()
                    error("Downloaded APK too small ($bytesRead bytes)")
                }
                if (totalBytes > 0L && bytesRead != totalBytes) {
                    tempFile.delete()
                    error("Incomplete download ($bytesRead / $totalBytes bytes)")
                }

                if (finalFile.exists()) finalFile.delete()
                if (!tempFile.renameTo(finalFile)) {
                    tempFile.copyTo(finalFile, overwrite = true)
                    tempFile.delete()
                }
                onProgress(bytesRead, totalBytes.coerceAtLeast(bytesRead))
                finalFile
            }
        }.onFailure { err ->
            if (err is CancellationException) throw err
        }
    }

    fun launchApkInstaller(context: Context, apkFile: File): ApkInstallOutcome {
        if (!apkFile.exists()) {
            return ApkInstallOutcome.Failed("Downloaded APK file is missing")
        }
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O &&
            !context.packageManager.canRequestPackageInstalls()
        ) {
            return runCatching {
                val permIntent = Intent(
                    Settings.ACTION_MANAGE_UNKNOWN_APP_SOURCES,
                    Uri.parse("package:${context.packageName}"),
                ).apply {
                    addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                }
                context.startActivity(permIntent)
                ApkInstallOutcome.NeedsInstallPermission
            }.getOrElse { err ->
                ApkInstallOutcome.Failed(err.message ?: "Enable 'Install unknown apps' in Android Settings")
            }
        }

        return runCatching {
            val contentUri = FileProvider.getUriForFile(
                context,
                "${context.packageName}.fileprovider",
                apkFile,
            )
            val installIntent = Intent(Intent.ACTION_VIEW).apply {
                setDataAndType(contentUri, "application/vnd.android.package-archive")
                addFlags(Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_GRANT_READ_URI_PERMISSION)
            }
            context.startActivity(installIntent)
            ApkInstallOutcome.InstallerLaunched
        }.getOrElse { err ->
            ApkInstallOutcome.Failed(err.message ?: "Unable to open APK installer")
        }
    }

    fun openExternalUrl(context: Context, url: String) {
        runCatching {
            val intent = Intent(Intent.ACTION_VIEW, Uri.parse(url)).apply {
                addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
            }
            context.startActivity(intent)
        }
    }
}
