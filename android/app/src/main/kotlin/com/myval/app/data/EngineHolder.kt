package com.myval.app.data

import android.content.Context
import uniffi.myval_ffi.MyValEngine
import java.io.File

object EngineHolder {
    @Volatile
    private var instance: MyValEngine? = null

    fun get(context: Context): MyValEngine {
        return instance ?: synchronized(this) {
            instance ?: run {
                val dbFile = File(context.applicationContext.filesDir, "myval.db")
                MyValEngine(dbFile.absolutePath).also { instance = it }
            }
        }
    }
}
