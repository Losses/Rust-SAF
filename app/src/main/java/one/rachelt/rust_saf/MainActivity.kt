package one.rachelt.rust_saf

import android.content.Context
import android.content.Intent
import android.os.Bundle
import android.os.ParcelFileDescriptor
import android.widget.Button
import android.widget.TextView
import androidx.activity.enableEdgeToEdge
import androidx.activity.result.contract.ActivityResultContracts
import androidx.appcompat.app.AppCompatActivity
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsCompat
import androidx.documentfile.provider.DocumentFile

class MainActivity : AppCompatActivity() {
    companion object {
        init {
            System.loadLibrary("main")
        }
    }
    lateinit var textView: TextView
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        initializeContext(this)
        enableEdgeToEdge()
        setContentView(R.layout.activity_main)
        ViewCompat.setOnApplyWindowInsetsListener(findViewById(R.id.main)) { v, insets ->
            val systemBars = insets.getInsets(WindowInsetsCompat.Type.systemBars())
            v.setPadding(systemBars.left, systemBars.top, systemBars.right, systemBars.bottom)
            insets
        }
        val fileLauncher = registerForActivityResult(ActivityResultContracts.OpenDocument()) {
            textView.text = it.toString()
            if (it != null) {
                contentResolver.takePersistableUriPermission(it, Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_GRANT_WRITE_URI_PERMISSION)
                contentResolver.openFileDescriptor(it, "w")?.use { pfd ->
                    val fd = pfd.detachFd()
                    textView.text = textView.text as String + fd.toString()
                    ParcelFileDescriptor.adoptFd(fd).close()
                }
            }
        }
        val treeLauncher = registerForActivityResult(ActivityResultContracts.OpenDocumentTree()) {
            textView.text = it.toString()
            if (it != null) {
                contentResolver.takePersistableUriPermission(it, Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_GRANT_WRITE_URI_PERMISSION)
                val documentFile = DocumentFile.fromTreeUri(this, it)
                textView.text = textView.text as String + "\n" + documentFile?.name + "\n" + documentFile?.exists()
                listUriFiles(it.toString())
            }
        }
        textView = findViewById(R.id.text_view)
        findViewById<Button>(R.id.select_file).setOnClickListener {
            fileLauncher.launch(arrayOf("*/*"))
        }
        findViewById<Button>(R.id.select_dir).setOnClickListener {
            treeLauncher.launch(null)
        }
    }

    override fun onDestroy() {
        super.onDestroy()
        releaseContext()
    }

    private external fun initializeContext(context: Context)
    private external fun releaseContext()
    external fun listUriFiles(uri: String)

}