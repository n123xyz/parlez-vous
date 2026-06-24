package com.plugin.supertonic

import android.app.Activity
import android.util.Log
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import app.tauri.plugin.Invoke
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import java.io.File

@InvokeArg
class IsSupertonicReadyArgs {
  var appDataDir: String = ""
}

@InvokeArg
class DownloadSupertonicArgs {
  var token: String? = null
  var modelPath: String = "" // In download models, this gets mapped to appDataDir
}

@TauriPlugin
class SupertonicPlugin(private val activity: Activity): Plugin(activity) {
    private val scope = CoroutineScope(Dispatchers.IO)

    init {
        try {
            Log.i("SupertonicPlugin", "Loading onnxruntime...")
            System.loadLibrary("onnxruntime")
            Log.i("SupertonicPlugin", "Successfully loaded onnxruntime.")
        } catch (e: UnsatisfiedLinkError) {
            Log.e("SupertonicPlugin", "Failed to load onnxruntime: \${e.message}")
        }
    }

    @Command
    fun isSupertonicReady(invoke: Invoke) {
        val args = invoke.parseArgs(IsSupertonicReadyArgs::class.java)
        scope.launch {
            try {
                val internalFilesDir = args.appDataDir
                val filesToCheck = listOf(
                    "onnx/duration_predictor.onnx",
                    "onnx/text_encoder.onnx",
                    "onnx/tts.json",
                    "onnx/unicode_indexer.json",
                    "onnx/vector_estimator.onnx",
                    "onnx/vocoder.onnx",
                    "voice_styles/F1.json"
                )
                
                var exists = true
                for (file in filesToCheck) {
                    val f = File("$internalFilesDir/$file")
                    if (!f.exists() || f.length() == 0L) {
                        exists = false
                        break
                    }
                }

                val workManager = androidx.work.WorkManager.getInstance(activity.applicationContext)
                val workInfos = workManager.getWorkInfosForUniqueWork("supertonic_download_all").get()
                var isDownloading = false
                if (workInfos.isNotEmpty()) {
                    val state = workInfos.first().state
                    if (state == androidx.work.WorkInfo.State.RUNNING || state == androidx.work.WorkInfo.State.ENQUEUED) {
                        isDownloading = true
                        pollDownloadProgress("supertonic_download_all")
                    }
                }
                
                val ret = JSObject()
                ret.put("exists", exists)
                ret.put("isDownloading", isDownloading)
                invoke.resolve(ret)
            } catch (e: Exception) {
                invoke.reject(e.message)
            }
        }
    }

    @Command
    fun downloadSupertonicModels(invoke: Invoke) {
        val args = invoke.parseArgs(DownloadSupertonicArgs::class.java)
        
        if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.TIRAMISU) {
            if (androidx.core.content.ContextCompat.checkSelfPermission(activity, android.Manifest.permission.POST_NOTIFICATIONS) != android.content.pm.PackageManager.PERMISSION_GRANTED) {
                androidx.core.app.ActivityCompat.requestPermissions(activity, arrayOf(android.Manifest.permission.POST_NOTIFICATIONS), 1003)
            }
        }

        scope.launch {
            try {
                val workData = androidx.work.workDataOf(
                    "token" to args.token,
                    "appDataDir" to args.modelPath
                )

                val workRequest = androidx.work.OneTimeWorkRequestBuilder<SupertonicDownloadWorker>()
                    .setInputData(workData)
                    .build()

                androidx.work.WorkManager.getInstance(activity.applicationContext)
                    .enqueueUniqueWork(
                        "supertonic_download_all",
                        androidx.work.ExistingWorkPolicy.KEEP,
                        workRequest
                    )
                
                pollDownloadProgress("supertonic_download_all")

                val ret = JSObject()
                ret.put("success", true)
                invoke.resolve(ret)
            } catch (e: Exception) {
                invoke.reject(e.message)
            }
        }
    }

    @Command
    fun purgeSupertonicModels(invoke: Invoke) {
        val args = invoke.parseArgs(IsSupertonicReadyArgs::class.java)
        try {
            val appDataDir = args.appDataDir
            File("$appDataDir/onnx").deleteRecursively()
            File("$appDataDir/voice_styles").deleteRecursively()

            // Also clean up the old incorrect path just in case!
            val oldFilesDir = activity.filesDir.absolutePath
            if (oldFilesDir != appDataDir) {
                File("$oldFilesDir/onnx").deleteRecursively()
                File("$oldFilesDir/voice_styles").deleteRecursively()
            }
            
            val ret = JSObject()
            ret.put("success", true)
            invoke.resolve(ret)
        } catch (e: Exception) {
            invoke.reject(e.message)
        }
    }

    private fun pollDownloadProgress(uniqueWorkName: String) {
        scope.launch {
            val workManager = androidx.work.WorkManager.getInstance(activity.applicationContext)
            var isCompleted = false
            
            while (!isCompleted) {
                val workInfos = workManager.getWorkInfosForUniqueWork(uniqueWorkName).get()
                if (workInfos.isNotEmpty()) {
                    val info = workInfos.first()
                    
                    val progress = info.progress
                    val downloaded = progress.getLong("downloaded", -1L)
                    val total = progress.getLong("total", -1L)
                    
                    if (downloaded >= 0) {
                        val eventData = JSObject()
                        eventData.put("downloaded", downloaded)
                        eventData.put("total", total)
                        trigger("supertonic_download_progress", eventData)
                    }
                    
                    if (info.state == androidx.work.WorkInfo.State.SUCCEEDED || 
                        info.state == androidx.work.WorkInfo.State.FAILED ||
                        info.state == androidx.work.WorkInfo.State.CANCELLED) {
                        isCompleted = true
                        
                        val finalEvent = JSObject()
                        finalEvent.put("downloaded", total)
                        finalEvent.put("total", total)
                        finalEvent.put("state", info.state.name)
                        trigger("supertonic_download_progress", finalEvent)
                    }
                }
                if (!isCompleted) {
                    kotlinx.coroutines.delay(500)
                }
            }
        }
    }
}
