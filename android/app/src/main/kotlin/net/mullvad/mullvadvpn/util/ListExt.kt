package net.mullvad.mullvadvpn.util

fun List<String>.toBulletList(): String  {
    var sb = StringBuilder()
    sb.append("<ul>")
    this.forEach {
        sb.append("<li><h4>&nbsp; $it</h4></li>\n<li></li>\n")
    }
    sb.append("</ul>")
    return sb.toString()
}
