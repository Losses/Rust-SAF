package one.rachelt.rust_saf;

import android.content.ContentProvider;
import android.content.ContentValues;
import android.database.Cursor;
import android.database.MatrixCursor;
import android.net.Uri;
import android.os.Bundle;
import android.provider.DocumentsContract;

public class ListingProvider extends ContentProvider {
    @Override public boolean onCreate() { return true; }

    @Override public Cursor query(Uri uri, String[] columns, String selection, String[] args, String sort) {
        String id = DocumentsContract.getDocumentId(uri);
        if ("missing-cursor".equals(id)) return null;
        MatrixCursor cursor = new MatrixCursor(columns);
        if (!"empty".equals(id)) {
            cursor.newRow()
                .add(DocumentsContract.Document.COLUMN_DOCUMENT_ID, "track")
                .add(DocumentsContract.Document.COLUMN_DISPLAY_NAME, "track.ogg")
                .add(DocumentsContract.Document.COLUMN_SIZE, 10)
                .add(DocumentsContract.Document.COLUMN_MIME_TYPE, "audio/ogg");
        }
        Bundle extras = new Bundle();
        if ("loading".equals(id)) extras.putBoolean(DocumentsContract.EXTRA_LOADING, true);
        if ("error".equals(id)) extras.putString(DocumentsContract.EXTRA_ERROR, "unavailable");
        cursor.setExtras(extras);
        return cursor;
    }

    @Override public String getType(Uri uri) { return null; }
    @Override public Uri insert(Uri uri, ContentValues values) { throw new UnsupportedOperationException(); }
    @Override public int delete(Uri uri, String selection, String[] args) { throw new UnsupportedOperationException(); }
    @Override public int update(Uri uri, ContentValues values, String selection, String[] args) { throw new UnsupportedOperationException(); }
}
