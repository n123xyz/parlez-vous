package com.plugin.litert

import android.app.Activity
import android.util.Log
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import app.tauri.plugin.Invoke
import androidx.activity.ComponentActivity
import androidx.activity.result.contract.ActivityResultContracts
import androidx.activity.result.ActivityResultLauncher
import androidx.activity.result.ActivityResult
import com.google.ai.edge.litertlm.*
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.io.File
import java.io.FileOutputStream
import java.net.URL
import android.content.Intent
import android.net.Uri

@InvokeArg
class InitModelArgs {
  var modelPath: String = ""
  var accelerator: String = "Auto"
  var maxTokens: Int = 5000
}

@InvokeArg
class CheckModelArgs {
  var modelPath: String = ""
}

@InvokeArg
class DownloadModelArgs {
  var modelPath: String = ""
  var token: String? = null
}

@InvokeArg
class PurgeModelArgs {
  var modelPath: String = ""
}

@InvokeArg
class GenerateChatArgs {
  var prompt: String = ""
  var reset: Boolean = false
  var audioBase64: String? = null
  var imageUri: String? = null
}

@TauriPlugin
class LitertPlugin(private val activity: Activity): Plugin(activity) {
    companion object {
        init {
            try {
                System.loadLibrary("LiteRt")
                Log.i("LitertPlugin", "Loaded libLiteRt.so into global namespace to fix OpenCL sampler linker issue")
            } catch (e: UnsatisfiedLinkError) {
                Log.w("LitertPlugin", "Could not pre-load libLiteRt.so, namespace isolation fix may not apply", e)
            }
        }
    }

    private var engine: Engine? = null
    private var conversation: Conversation? = null
    private var conversationConfig: ConversationConfig? = null
    private val scope = CoroutineScope(Dispatchers.IO)

    // Manual ActivityResultRegistry registration to bypass Tauri's broken lateinit launcher bug
    // AND bypass the LifecycleOwner STARTED requirement
    private var pendingImageInvoke: Invoke? = null
    private val pickerLauncher: ActivityResultLauncher<Intent> = 
        (activity as ComponentActivity).activityResultRegistry.register(
            "litert_gallery_picker",
            ActivityResultContracts.StartActivityForResult()
        ) { result ->
            if (result.resultCode == Activity.RESULT_OK && result.data != null && result.data!!.data != null) {
                val uri: Uri = result.data!!.data!!
                
                try {
                    val fileName = "vision_target_${System.currentTimeMillis()}.jpg"
                    val cacheFile = File(activity.cacheDir, fileName)
                    
                    activity.contentResolver.openInputStream(uri)?.use { input ->
                        FileOutputStream(cacheFile).use { output ->
                            input.copyTo(output)
                        }
                    }
                    
                    val ret = JSObject()
                    ret.put("path", cacheFile.absolutePath)
                    pendingImageInvoke?.resolve(ret)
                    
                } catch (e: Exception) {
                    pendingImageInvoke?.reject("Failed to process image: ${e.message}")
                }
            } else {
                pendingImageInvoke?.reject("Image selection cancelled by user")
            }
            pendingImageInvoke = null
        }

    private fun downloadModelWithRedirects(urlStr: String, destFile: File) {
        var currentUrl = URL(urlStr)
        var connection = currentUrl.openConnection() as java.net.HttpURLConnection
        var redirectCount = 0
        val maxRedirects = 5

        while (redirectCount < maxRedirects) {
            connection.instanceFollowRedirects = false
            connection.connect()
            val responseCode = connection.responseCode

            if (responseCode in 300..399) {
                val location = connection.getHeaderField("Location") ?: throw Exception("Redirect without location header")
                currentUrl = URL(location)
                connection = currentUrl.openConnection() as java.net.HttpURLConnection
                redirectCount++
            } else if (responseCode in 200..299) {
                connection.inputStream.use { input ->
                    FileOutputStream(destFile).use { output ->
                        input.copyTo(output)
                    }
                }
                break
            } else {
                throw Exception("Failed to download file: HTTP $responseCode")
            }
        }
    }

    @Command
    fun checkModelExists(invoke: Invoke) {
        val args = invoke.parseArgs(CheckModelArgs::class.java)
        scope.launch {
            try {
                val internalFilesDir = activity.filesDir.absolutePath
                val absoluteModelPath = "$internalFilesDir/${args.modelPath}"
                val modelFile = File(absoluteModelPath)
                
                val exists = modelFile.exists() && modelFile.length() > 100000000L

                val workManager = androidx.work.WorkManager.getInstance(activity.applicationContext)
                val workInfos = workManager.getWorkInfosForUniqueWork("litert_download").get()
                var isDownloading = false
                if (workInfos.isNotEmpty()) {
                    val state = workInfos.first().state
                    if (state == androidx.work.WorkInfo.State.RUNNING || state == androidx.work.WorkInfo.State.ENQUEUED) {
                        isDownloading = true
                        pollDownloadProgress()
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
    fun downloadModel(invoke: Invoke) {
        val args = invoke.parseArgs(DownloadModelArgs::class.java)
        
        // Request POST_NOTIFICATIONS permission for Android 13+
        if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.TIRAMISU) {
            if (androidx.core.content.ContextCompat.checkSelfPermission(activity, android.Manifest.permission.POST_NOTIFICATIONS) != android.content.pm.PackageManager.PERMISSION_GRANTED) {
                androidx.core.app.ActivityCompat.requestPermissions(activity, arrayOf(android.Manifest.permission.POST_NOTIFICATIONS), 1002)
            }
        }

        scope.launch {
            try {
                val downloadUrl = "https://huggingface.co/litert-community/gemma-4-E2B-it-litert-lm/resolve/main/gemma-4-E2B-it.litertlm"
                
                val workData = androidx.work.workDataOf(
                    "modelPath" to args.modelPath,
                    "downloadUrl" to downloadUrl,
                    "token" to args.token
                )

                val workRequest = androidx.work.OneTimeWorkRequestBuilder<ModelDownloadWorker>()
                    .setInputData(workData)
                    .build()

                androidx.work.WorkManager.getInstance(activity.applicationContext)
                    .enqueueUniqueWork(
                        "litert_download",
                        androidx.work.ExistingWorkPolicy.KEEP,
                        workRequest
                    )
                
                // Launch progress polling
                pollDownloadProgress()

                val ret = JSObject()
                ret.put("success", true)
                invoke.resolve(ret)
            } catch (e: Exception) {
                invoke.reject(e.message)
            }
        }
    }

    @Command
    fun purgeModel(invoke: Invoke) {
        val args = invoke.parseArgs(PurgeModelArgs::class.java)
        try {
            val internalFilesDir = activity.filesDir.absolutePath
            val absoluteModelPath = "$internalFilesDir/${args.modelPath}"
            val tmpModelPath = "$absoluteModelPath.tmp"
            
            val modelFile = File(absoluteModelPath)
            if (modelFile.exists()) {
                modelFile.delete()
            }
            
            val tmpFile = File(tmpModelPath)
            if (tmpFile.exists()) {
                tmpFile.delete()
            }
            
            invoke.resolve()
        } catch (e: Exception) {
            invoke.reject(e.message)
        }
    }

    private fun pollDownloadProgress() {
        scope.launch {
            val workManager = androidx.work.WorkManager.getInstance(activity.applicationContext)
            var isCompleted = false
            
            while (!isCompleted) {
                val workInfos = workManager.getWorkInfosForUniqueWork("litert_download").get()
                if (workInfos.isNotEmpty()) {
                    val info = workInfos.first()
                    
                    val progress = info.progress
                    val downloaded = progress.getLong("downloaded", -1L)
                    val total = progress.getLong("total", -1L)
                    
                    if (downloaded >= 0) {
                        val eventData = JSObject()
                        eventData.put("downloaded", downloaded)
                        eventData.put("total", total)
                        trigger("download_progress", eventData)
                    }
                    
                    if (info.state == androidx.work.WorkInfo.State.SUCCEEDED || 
                        info.state == androidx.work.WorkInfo.State.FAILED ||
                        info.state == androidx.work.WorkInfo.State.CANCELLED) {
                        isCompleted = true
                        
                        // Emit final event
                        val finalEvent = JSObject()
                        finalEvent.put("downloaded", total)
                        finalEvent.put("total", total)
                        finalEvent.put("state", info.state.name)
                        trigger("download_progress", finalEvent)
                    }
                }
                if (!isCompleted) {
                    kotlinx.coroutines.delay(500)
                }
            }
        }
    }

    @Command
    fun initModel(invoke: Invoke) {
        val args = invoke.parseArgs(InitModelArgs::class.java)
        
        scope.launch {
            try {
                // Determine absolute path
                val internalFilesDir = activity.filesDir.absolutePath
                val absoluteModelPath = "$internalFilesDir/${args.modelPath}"
                
                val modelFile = File(absoluteModelPath)
                if (!modelFile.exists() || modelFile.length() < 100000000L) {
                    invoke.reject("Model file not found or corrupted. Please download it first.")
                    return@launch
                }

                // Explicitly close any existing engine and conversation to prevent overlapping 2GB native memory allocations
                conversation?.close()
                conversation = null
                engine?.close()
                engine = null
                System.gc()

                try {
                    val configBackend = when (args.accelerator) {
                        "CPU" -> Backend.CPU()
                        "GPU" -> Backend.GPU()
                        "NPU" -> Backend.NPU(nativeLibraryDir = activity.applicationInfo.nativeLibraryDir)
                        else -> null
                    }

                    if (configBackend != null) {
                        val engineConfig = EngineConfig(
                            modelPath = absoluteModelPath,
                            maxNumTokens = args.maxTokens,
                            backend = configBackend,
                            visionBackend = Backend.CPU(),
                            audioBackend = Backend.CPU(),
                            maxNumImages = 1,
                            cacheDir = activity.cacheDir.path
                        )
                        engine = Engine(engineConfig)
                        engine!!.initialize()
                        Log.i("LitertPlugin", "Model initialized with explicit ${args.accelerator} backend")
                    } else {
                        val engineConfig = EngineConfig(
                            modelPath = absoluteModelPath,
                            maxNumTokens = args.maxTokens,
                            backend = Backend.NPU(nativeLibraryDir = activity.applicationInfo.nativeLibraryDir),
                            visionBackend = Backend.CPU(),
                            audioBackend = Backend.CPU(),
                            maxNumImages = 1,
                            cacheDir = activity.cacheDir.path
                        )
                        engine = Engine(engineConfig)
                        engine!!.initialize()
                        Log.i("LitertPlugin", "Model initialized with NPU backend")
                    }
                } catch (e: Exception) {
                    if (args.accelerator != "Auto") {
                        throw e // Rethrow if explicit backend fails
                    }
                    Log.w("LitertPlugin", "NPU Backend failed, falling back to GPU", e)
                    // CRITICAL: Prevent memory leak of 2.5GB by explicitly closing failed native engine
                    try {
                        engine?.close()
                    } catch (closeErr: Exception) {
                        Log.w("LitertPlugin", "Error closing uninitialized NPU engine: ${closeErr.message}")
                    }
                    engine = null
                    System.gc()
                    
                    try {
                        val gpuConfig = EngineConfig(
                            modelPath = absoluteModelPath,
                            maxNumTokens = args.maxTokens,
                            backend = Backend.GPU(),
                            visionBackend = Backend.CPU(),
                            audioBackend = Backend.CPU(),
                            maxNumImages = 1,
                            cacheDir = activity.cacheDir.path
                        )
                        engine = Engine(gpuConfig)
                        engine!!.initialize()
                        Log.i("LitertPlugin", "Model initialized with GPU backend")
                    } catch (e2: Exception) {
                        Log.w("LitertPlugin", "GPU Backend failed, falling back to CPU", e2)
                        // CRITICAL: Prevent memory leak again
                        try {
                            engine?.close()
                        } catch (closeErr: Exception) {
                            Log.w("LitertPlugin", "Error closing uninitialized GPU engine: ${closeErr.message}")
                        }
                        engine = null
                        System.gc()
                        
                        val cpuConfig = EngineConfig(
                            modelPath = absoluteModelPath,
                            maxNumTokens = args.maxTokens,
                            backend = Backend.CPU(),
                            visionBackend = Backend.CPU(),
                            audioBackend = Backend.CPU(),
                            maxNumImages = 1,
                            cacheDir = activity.cacheDir.path
                        )
                        engine = Engine(cpuConfig)
                        engine!!.initialize()
                        Log.i("LitertPlugin", "Model initialized with CPU backend")
                    }
                }

                // Store the conversation config for later reuse
                conversationConfig = ConversationConfig(
                    samplerConfig = SamplerConfig(
                        topK = 40,
                        topP = 0.95,
                        temperature = 0.8
                    )
                )
                conversation = engine!!.createConversation(conversationConfig!!)

                val ret = JSObject()
                ret.put("success", true)
                invoke.resolve(ret)
            } catch (e: LiteRtLmJniException) {
                Log.e("LitertPlugin", "JNI Exception initializing model", e)
                invoke.reject("Native failure: ${e.message}")
            } catch (e: Exception) {
                Log.e("LitertPlugin", "Failed to init model", e)
                invoke.reject("Failed to load model: ${e.message}")
            }
        }
    }

    @Command
    fun generateChat(invoke: Invoke) {
        val args = invoke.parseArgs(GenerateChatArgs::class.java)
        
        if (engine == null) {
            invoke.reject("Model is not initialized.")
            return
        }

        scope.launch {
            try {
                if (args.reset || conversation == null) {
                    conversation?.close()
                    conversation = engine!!.createConversation(conversationConfig ?: ConversationConfig())
                }
                
                var responseText = ""
                
                val contents = mutableListOf<Content>()
                
                if (!args.imageUri.isNullOrEmpty()) {
                    val file = File(args.imageUri!!)
                    if (file.exists()) {
                        val rawBitmap = android.graphics.BitmapFactory.decodeFile(file.absolutePath)
                        if (rawBitmap != null) {
                            val originalWidth = rawBitmap.width.toDouble()
                            val originalHeight = rawBitmap.height.toDouble()
                            
                            val targetArea = 645120.0
                            val area = originalWidth * originalHeight
                            
                            var scale = 1.0
                            if (area > targetArea) {
                                scale = Math.sqrt(targetArea / area)
                            }
                            
                            var newWidth = Math.round(originalWidth * scale).toInt()
                            var newHeight = Math.round(originalHeight * scale).toInt()
                            
                            newWidth = (newWidth / 48) * 48
                            newHeight = (newHeight / 48) * 48
                            
                            if (newWidth < 48) newWidth = 48
                            if (newHeight < 48) newHeight = 48
                            
                            val scaledBitmap = android.graphics.Bitmap.createScaledBitmap(rawBitmap, newWidth, newHeight, true)
                            
                            val tempFile = java.io.File(activity.cacheDir, "litert_scaled_temp.jpg")
                            val outStream = java.io.FileOutputStream(tempFile)
                            scaledBitmap.compress(android.graphics.Bitmap.CompressFormat.JPEG, 90, outStream)
                            outStream.flush()
                            outStream.close()
                            
                            contents.add(Content.ImageFile(tempFile.absolutePath))
                            
                            if (scaledBitmap != rawBitmap) {
                                rawBitmap.recycle()
                            }
                        } else {
                            Log.w("LitertPlugin", "Failed to decode bitmap from path: ${args.imageUri}")
                        }
                    }
                }
                
                if (!args.audioBase64.isNullOrEmpty()) {
                    val audioBytes = android.util.Base64.decode(args.audioBase64, android.util.Base64.DEFAULT)
                    contents.add(Content.AudioBytes(audioBytes))
                }
                
                if (args.prompt.trim().isNotEmpty()) {
                    // Try to inject <image> if not present in multimodal?
                    // Actually, just add the text after the media.
                    contents.add(Content.Text(args.prompt))
                }
                
                if (contents.isNotEmpty()) {
                    conversation!!.sendMessageAsync(Contents.of(contents)).collect { token ->
                        responseText += token
                    }
                } else {
                    invoke.reject("No content provided to generate chat.")
                    return@launch
                }
                
                val ret = JSObject()
                ret.put("response", responseText)
                invoke.resolve(ret)
            } catch (e: LiteRtLmJniException) {
                Log.e("LitertPlugin", "JNI Exception during chat", e)
                invoke.reject("Native generation failed: ${e.message}")
            } catch (e: Exception) {
                Log.e("LitertPlugin", "Failed to generate chat", e)
                invoke.reject("Generation failed: ${e.message}")
            }
        }
    }

    @Command
    fun closeModel(invoke: Invoke) {
        scope.launch {
            try {
                conversation?.close()
                conversation = null
                engine?.close()
                engine = null
                
                val ret = JSObject()
                ret.put("success", true)
                invoke.resolve(ret)
            } catch (e: Exception) {
                invoke.reject("Failed to close model: ${e.message}")
            }
        }
    }

    @Command
    fun pickGalleryImage(invoke: Invoke) {
        pendingImageInvoke = invoke
        val intent = Intent(Intent.ACTION_GET_CONTENT).apply {
            type = "image/*"
            addCategory(Intent.CATEGORY_OPENABLE)
        }
        
        pickerLauncher.launch(intent)
    }
}
