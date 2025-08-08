package one.rachelt.rust_saf

import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Bundle
import android.view.LayoutInflater
import android.view.ViewGroup
import android.widget.BaseAdapter
import android.widget.ImageView
import android.widget.ListView
import android.widget.TextView
import androidx.appcompat.app.AppCompatActivity
import androidx.documentfile.provider.DocumentFile
import androidx.core.net.toUri
import androidx.documentfile.provider.TreeDocumentFileWrapper

class FileListActivity : AppCompatActivity() {
    companion object {
        fun start(context: Context, folderUri: Uri) {
            val intent = Intent(context, FileListActivity::class.java).apply {
                data = folderUri
            }
            context.startActivity(intent)
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_file_list)

        val folderUri = intent.data
        if (folderUri == null) {
            finish()
            return
        }
        val documentFile = TreeDocumentFileWrapper.fromTreeUri(this, folderUri)
        if (documentFile == null) {
            finish()
            return
        }

        title = documentFile.name ?: "Files"
        val listView = findViewById<ListView>(R.id.file_list_view)
        val files = documentFile.listFiles().toList()
        if (files.isEmpty()) {
            // Show a message
            findViewById<TextView>(R.id.empty_view).visibility = android.view.View.VISIBLE
        }
        listView.adapter = FileListAdapter(this, files)
    }

    private class FileListAdapter(
        private val context: Context,
        private val files: List<DocumentFile>
    ) : BaseAdapter() {

        override fun getCount(): Int = files.size

        override fun getItem(position: Int): DocumentFile = files[position]

        override fun getItemId(position: Int): Long = position.toLong()

        override fun getView(position: Int, convertView: android.view.View?, parent: ViewGroup): android.view.View {
            val itemView = convertView ?: LayoutInflater.from(context)
                .inflate(R.layout.item_file_list, parent, false)

            val file = getItem(position)

            itemView.findViewById<ImageView>(R.id.file_icon).setImageResource(
                if (file.isDirectory) R.drawable.ic_folder else R.drawable.ic_file
            )

            itemView.findViewById<TextView>(R.id.file_title).text = file.name
            itemView.findViewById<TextView>(R.id.file_path).text = file.uri.toString()

            itemView.setOnClickListener {
                if (file.isDirectory) {
                    // Open the folder in a new FileListActivity
                    start(context, file.uri)
                } else {
                    // Handle file click (e.g., open the file)
                    // This can be customized based on your requirements
                    // For example, you might want to open the file with an appropriate app
                    val intent = Intent(Intent.ACTION_VIEW).apply {
                        setDataAndType(file.uri, context.contentResolver.getType(file.uri) ?: "*/*")
                        addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
                        addFlags(Intent.FLAG_GRANT_WRITE_URI_PERMISSION)
                    }
                    context.startActivity(Intent.createChooser(intent, "Open file with"))
                }
            }

            return itemView
        }
    }
}
