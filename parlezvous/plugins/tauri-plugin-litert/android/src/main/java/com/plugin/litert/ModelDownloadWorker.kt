package com.plugin.litert

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

class ModelDownloadWorker(
    appContext: Context,
    workerParams: WorkerParameters
) : CoroutineWorker(appContext, workerParams) {

    private val notificationManager =
        appContext.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
    private val channelId = "litert_download_channel"
    private val notificationId = 1001

    override suspend fun doWork(): Result = withContext(Dispatchers.IO) {
        val modelPath = inputData.getString("modelPath") ?: return@withContext Result.failure()
        val downloadUrl = inputData.getString("downloadUrl") ?: return@withContext Result.failure()
        val token = inputData.getString("token") // Optional

        val internalFilesDir = applicationContext.filesDir.absolutePath
        val absoluteModelPath = "$internalFilesDir/$modelPath"
        val tmpModelPath = "$absoluteModelPath.tmp"

        val targetFile = File(absoluteModelPath)
        val tmpFile = File(tmpModelPath)

        // If target file already exists and is large, we assume it's downloaded
        if (targetFile.exists() && targetFile.length() > 100000000L) {
            return@withContext Result.success()
        }

        try {
            createNotificationChannel()
            setForeground(createForegroundInfo(0, 100, "Initializing..."))

            downloadFileWithResume(downloadUrl, tmpFile, token)

            val totalBytesExpected = inputData.getLong("expectedSize", -1L) // If we knew it beforehand, but we don't.
            // downloadFileWithResume will now throw if the stream ends prematurely.

            // Rename tmp file to target file on success
            if (tmpFile.exists()) {
                tmpFile.renameTo(targetFile)
                Log.i("ModelDownloadWorker", "Download successfully completed and file renamed")
            }
            
            Result.success()
        } catch (e: Exception) {
            Log.e("ModelDownloadWorker", "Download failed", e)
            Result.failure(workDataOf("error" to e.message))
        }
    }

    private suspend fun downloadFileWithResume(urlStr: String, destFile: File, token: String?) {
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
                currentUrl = URL(location)
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
                if (totalBytes != -1L && totalBytes < 100000000L) {
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
                                setProgress(workDataOf("downloaded" to downloadedBytes, "total" to totalBytes))
                                
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
                setProgress(workDataOf("downloaded" to downloadedBytes, "total" to totalBytes))
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
                "LiteRT Model Download",
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
