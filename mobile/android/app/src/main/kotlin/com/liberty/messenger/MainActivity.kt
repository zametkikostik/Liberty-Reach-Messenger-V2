package com.liberty.messenger

import android.os.Bundle
import android.view.WindowManager
import io.flutter.embedding.android.FlutterActivity
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.plugin.common.MethodChannel

class MainActivity : FlutterActivity() {
    private val CHANNEL = "liberty/security"

    override fun configureFlutterEngine(flutterEngine: FlutterEngine) {
        super.configureFlutterEngine(flutterEngine)
        MethodChannel(flutterEngine.dartExecutor.binaryMessenger, CHANNEL)
            .setMethodCallHandler { call, result ->
                try {
                    when (call.method) {
                        "setSecureFlag" -> {
                            val enabled = call.argument<Boolean>("enabled") ?: true
                            if (enabled) {
                                window.addFlags(WindowManager.LayoutParams.FLAG_SECURE)
                            } else {
                                window.clearFlags(WindowManager.LayoutParams.FLAG_SECURE)
                            }
                            result.success(null)
                        }
                        "keystoreEnsure" -> result.success(LibertyKeystore.ensureKey())
                        "keystoreWrap" -> {
                            val data = call.argument<ByteArray>("data")
                                ?: return@setMethodCallHandler result.error("ARG", "data required", null)
                            result.success(LibertyKeystore.wrap(data))
                        }
                        "keystoreUnwrap" -> {
                            val wrapped = call.argument<String>("wrapped")
                                ?: return@setMethodCallHandler result.error("ARG", "wrapped required", null)
                            result.success(LibertyKeystore.unwrap(wrapped))
                        }
                        "keystoreDelete" -> {
                            LibertyKeystore.deleteKey()
                            result.success(null)
                        }
                        "keystoreIsHardware" -> result.success(LibertyKeystore.isHardwareBacked())
                        else -> result.notImplemented()
                    }
                } catch (e: Exception) {
                    result.error("KEYSTORE", e.message, null)
                }
            }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        window.addFlags(WindowManager.LayoutParams.FLAG_SECURE)
    }
}
