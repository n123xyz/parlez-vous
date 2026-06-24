package com.plugin.supertonic

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.content.Context
import android.content.pm.ServiceInfo
import android.os.Build
import android.util.Log
import androidx.core.app.NotificationCompat
import androidx.work.CoroutineWorker
import androidx.work.ForegroundInfo
import androidx.work.WorkerParameters
import androidx.work.workDataOf
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.withContext
import java.io.File
import java.io.FileOutputStream
import java.io.RandomAccessFile
import java.net.HttpURLConnection
import java.net.URL

class SupertonicDownloadWorker(
    appContext: Context,
    workerParams: WorkerParameters
) : CoroutineWorker(appContext, workerParams) {

    private val notificationManager =
        appContext.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
    private val channelId = "supertonic_download_channel"
    private val notificationId = 1002

    override suspend fun doWork(): Result = withContext(Dispatchers.IO) {
        val token = inputData.getString("token") // Optional
        val repoUrl = "https://huggingface.co/Supertone/supertonic-3/resolve/main"
        val internalFilesDir = inputData.getString("appDataDir") ?: applicationContext.filesDir.absolutePath

        val filesToDownload = listOf(
            "onnx/duration_predictor.onnx",
            "onnx/text_encoder.onnx",
            "onnx/tts.json",
            "onnx/unicode_indexer.json",
            "onnx/vector_estimator.onnx",
            "onnx/vocoder.onnx",
            "voice_styles/F1.json",
            "voice_styles/F2.json",
            "voice_styles/F3.json",
            "voice_styles/F4.json",
            "voice_styles/F5.json",
            "voice_styles/M1.json",
            "voice_styles/M2.json",
            "voice_styles/M3.json",
            "voice_styles/M4.json",
            "voice_styles/M5.json"
        )

        try {
            createNotificationChannel()
            
            for ((index, filePath) in filesToDownload.withIndex()) {
                val absoluteModelPath = "$internalFilesDir/$filePath"
                val tmpModelPath = "$absoluteModelPath.tmp"
                val targetFile = File(absoluteModelPath)
                val tmpFile = File(tmpModelPath)

                targetFile.parentFile?.mkdirs()

                // Skip if already exists
                if (targetFile.exists() && targetFile.length() > 0) {
                    continue
                }

                val downloadUrl = "$repoUrl/$filePath"
                setForeground(createForegroundInfo(0, 100, "Downloading ${filePath.substringAfterLast("/")} (${index + 1}/${filesToDownload.size})"))

                downloadFileWithResume(downloadUrl, tmpFile, token) { downloadedBytes, totalBytes ->
                    val baseProgress = (index * 100L) / filesToDownload.size
                    val fileProgress = if (totalBytes > 0) ((downloadedBytes * 100L) / totalBytes) / filesToDownload.size else 0L
                    val overallProgress = baseProgress + fileProgress
                    setProgress(workDataOf("downloaded" to overallProgress, "total" to 100L))
                }

                if (tmpFile.exists()) {
                    tmpFile.renameTo(targetFile)
                }
            }

            Log.i("SupertonicDownloadWorker", "All Supertonic models downloaded successfully")
            Result.success()
        } catch (e: Exception) {
            Log.e("SupertonicDownloadWorker", "Download failed", e)
            Result.failure(workDataOf("error" to e.message))
        }
    }

    private suspend fun downloadFileWithResume(urlStr: String, destFile: File, token: String?, onProgress: suspend (Long, Long) -> Unit) {
        var currentUrl = URL(urlStr)
        var connection = currentUrl.openConnection() as HttpURLConnection
        var redirectCount = 0
        val maxRedirects = 5

        while (redirectCount < maxRedirects) {
            connection.instanceFollowRedirects = false
            
            // Authorization if provided, but ONLY to Hugging Face to prevent S3 HTTP 400 errors
            if (!token.isNullOrBlank() && currentUrl.host.endsWith("huggingface.co")) {
                connection.setRequestProperty("Authorization", "Bearer $token")
            }

            // Resumability support
            var downloadedBytes = 0L
            if (destFile.exists()) {
                downloadedBytes = destFile.length()
                if (downloadedBytes > 0) {
                    connection.setRequestProperty("Range", "bytes=$downloadedBytes-")
                }
            }
            
            connection.setRequestProperty("Accept-Encoding", "identity") // Crucial for range headers

            connection.connect()
            val responseCode = connection.responseCode

            if (responseCode in 300..399) {
                val location = connection.getHeaderField("Location") ?: throw Exception("Redirect without location header")
                currentUrl = URL(currentUrl, location)
                connection = currentUrl.openConnection() as HttpURLConnection
                redirectCount++
            } else if (responseCode in 200..299) {
                val isPartial = responseCode == 206 // Partial Content
                val contentLength = connection.contentLengthCompat
                val totalBytes = if (contentLength != -1L) downloadedBytes + contentLength else -1L

                // If not partial content and we requested range, it means server doesn't support resuming or range is invalid.
                if (!isPartial && downloadedBytes > 0) {
                    destFile.delete()
                    downloadedBytes = 0L
                }

                // If the total size is tiny, it's an LFS pointer, which means the token was invalid or missing
                if (totalBytes != -1L && totalBytes < 300L) {
                    throw Exception("Downloaded file is too small ($totalBytes bytes). It is likely an LFS pointer. Please ensure your HuggingFace token is correct and has access.")
                }

                connection.inputStream.use { input ->
                    RandomAccessFile(destFile, "rw").use { output ->
                        output.seek(downloadedBytes)
                        val buffer = ByteArray(8192)
                        var bytesRead: Int
                        var lastProgressUpdate = System.currentTimeMillis()

                        while (input.read(buffer).also { bytesRead = it } != -1) {
                            output.write(buffer, 0, bytesRead)
                            downloadedBytes += bytesRead

                            val currentTime = System.currentTimeMillis()
                            if (currentTime - lastProgressUpdate > 500) {
                                lastProgressUpdate = currentTime
                                onProgress(downloadedBytes, totalBytes)
                                
                                val progressPercent = if (totalBytes > 0) ((downloadedBytes * 100) / totalBytes).toInt() else -1
                                setForeground(createForegroundInfo(downloadedBytes, totalBytes, "Downloading: $progressPercent%"))
                            }
                        }
                        
                        // Prevent premature EOF truncation
                        if (totalBytes != -1L && downloadedBytes < totalBytes) {
                            throw Exception("Stream ended prematurely: downloaded $downloadedBytes out of $totalBytes bytes")
                        }
                    }
                }
                onProgress(downloadedBytes, totalBytes)
                break
            } else {
                throw Exception("Failed to download file: HTTP $responseCode")
            }
        }
    }

    private val HttpURLConnection.contentLengthCompat: Long
        get() = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.N) contentLengthLong else contentLength.toLong()

    private fun createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(
                channelId,
                "Supertonic Model Download",
                NotificationManager.IMPORTANCE_LOW
            )
            notificationManager.createNotificationChannel(channel)
        }
    }

    private fun createForegroundInfo(downloaded: Long, total: Long, status: String): ForegroundInfo {
        val max = if (total > 0) total.toInt() else 100
        val progress = if (total > 0) downloaded.toInt() else 0
        
        val notification = NotificationCompat.Builder(applicationContext, channelId)
            .setContentTitle("Downloading Local Model")
            .setContentText(status)
            .setSmallIcon(android.R.drawable.stat_sys_download)
            .setOngoing(true)
            .setProgress(max, progress, total <= 0)
            .build()

        return if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
            ForegroundInfo(
                notificationId, 
                notification, 
                ServiceInfo.FOREGROUND_SERVICE_TYPE_DATA_SYNC
            )
        } else {
            ForegroundInfo(notificationId, notification)
        }
    }
}
