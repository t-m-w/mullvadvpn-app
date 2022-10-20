package net.mullvad.mullvadvpn.util

import android.Manifest
import android.app.PendingIntent
import android.content.Context
import android.content.pm.PackageManager
import android.net.VpnService
import android.os.Build
import android.provider.Settings
import android.service.quicksettings.Tile
import android.widget.Toast
import net.mullvad.mullvadvpn.BuildConfig

object SdkUtils {
    fun getSupportedPendingIntentFlags(): Int {
        return if (Build.VERSION.SDK_INT > Build.VERSION_CODES.S) {
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_MUTABLE
        } else {
            PendingIntent.FLAG_UPDATE_CURRENT
        }
    }

    fun Context.isNotificationPermissionGranted(): Boolean {
        return (Build.VERSION.SDK_INT < Build.VERSION_CODES.TIRAMISU) ||
            checkSelfPermission(Manifest.permission.POST_NOTIFICATIONS) ==
                PackageManager.PERMISSION_GRANTED
    }

    fun Context.getAlwaysOnVpnAppName(): String?{
            val currentAlwaysOnVpn = Settings.Secure.getString(
                contentResolver,
                "always_on_vpn_app"
            )
            var appName = packageManager.getInstalledPackages(PackageManager.PackageInfoFlags.of(0))
                .filter { it.packageName == currentAlwaysOnVpn }
            if (appName.size == 1 && appName[0].packageName != packageName) {
                return appName[0].applicationInfo.loadLabel(packageManager).toString()
            }

        return null
    }

    fun VpnService.Builder.setMeteredIfSupported(isMetered: Boolean) {
        if (Build.VERSION.SDK_INT > Build.VERSION_CODES.Q) {
            this.setMetered(isMetered)
        }
    }

    fun Tile.setSubtitleIfSupported(subtitleText: CharSequence) {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
            this.subtitle = subtitleText
        }
    }
}
