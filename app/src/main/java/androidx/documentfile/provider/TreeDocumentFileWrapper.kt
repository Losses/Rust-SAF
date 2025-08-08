package androidx.documentfile.provider

import android.content.Context
import android.net.Uri

object TreeDocumentFileWrapper {
    @JvmStatic
    fun fromTreeUri(parent: DocumentFile?, context: Context, uri: Uri): DocumentFile {
        return TreeDocumentFile(parent, context, uri)
    }

    @JvmStatic
    fun fromTreeUri(context: Context, uri: Uri): DocumentFile? {
        val parent = DocumentFile.fromTreeUri(context, uri)
        if (parent?.uri.toString().startsWith(uri.toString())) {
            return parent
        }
        return fromTreeUri(parent, context, uri)
    }
}