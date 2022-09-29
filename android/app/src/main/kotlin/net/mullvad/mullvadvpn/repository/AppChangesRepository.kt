package net.mullvad.mullvadvpn.repository

import android.content.SharedPreferences
import android.content.res.AssetManager
import net.mullvad.mullvadvpn.BuildConfig
import java.io.IOException
import java.io.InputStream

private const val SHOWN_CHANGES_KEY = BuildConfig.VERSION_NAME
private const val CHANGES_FILE = "en-US/default.txt"

interface IAppChangesRepository{
    fun shouldShowLastChanges(): Boolean
    fun setShowedLastChanges()
    fun getLastVersionChanges(): List<String>
}

class AppChangesRepository(
    private val preferences: SharedPreferences,
    private val assets: AssetManager
): IAppChangesRepository {


    override fun shouldShowLastChanges(): Boolean {
        return !preferences.getBoolean(SHOWN_CHANGES_KEY, false)
    }

    override fun setShowedLastChanges() {
        preferences.edit().putBoolean(SHOWN_CHANGES_KEY, true).apply()
    }

    override fun getLastVersionChanges(): List<String> {
        return try {
            val inputStream: InputStream = assets.open(CHANGES_FILE)
            val size: Int = inputStream.available()
            val buffer = ByteArray(size)
            inputStream.read(buffer)
            String(buffer).split('\n')
                .filter { !it.isNullOrEmpty() }
                .map { "$it" }
        } catch (e: IOException) {
            e.printStackTrace()
            ArrayList()
        }
    }
}


class AppChangesMockRepository(): IAppChangesRepository {

    override fun shouldShowLastChanges(): Boolean {
        return true
    }

    override fun setShowedLastChanges() {

    }

    override fun getLastVersionChanges(): List<String> {

        var changes = ArrayList<String>()
        changes.add("Change no 1")
        changes.add("Change no 2")
        changes.add("Change no 3")
        changes.add("Change no 4")
        changes.add("Change no 5")
        return changes
    }
}
