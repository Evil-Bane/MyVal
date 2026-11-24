package com.myval.app

import android.app.Application
import coil.ImageLoader
import coil.ImageLoaderFactory
import coil.disk.DiskCache
import coil.memory.MemoryCache
import okhttp3.Dispatcher
import okhttp3.OkHttpClient

class MyValApplication : Application(), ImageLoaderFactory {
    override fun newImageLoader(): ImageLoader {
        val httpClient = OkHttpClient.Builder()
            .dispatcher(
                Dispatcher().apply {
                    maxRequests = 32
                    maxRequestsPerHost = 12
                },
            )
            .retryOnConnectionFailure(true)
            .build()

        return ImageLoader.Builder(this)
            .okHttpClient(httpClient)
            .memoryCache {
                MemoryCache.Builder(this)
                    .maxSizePercent(0.25)
                    .build()
            }
            .diskCache {
                DiskCache.Builder()
                    .directory(cacheDir.resolve("coil_image_cache"))
                    .maxSizeBytes(128L * 1024L * 1024L)
                    .build()
            }
            .respectCacheHeaders(false)
            .crossfade(160)
            .build()
    }
}
