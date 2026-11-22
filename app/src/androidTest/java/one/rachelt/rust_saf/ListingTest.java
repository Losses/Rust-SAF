package one.rachelt.rust_saf;

import java.io.IOException;
import org.junit.Test;
import static org.junit.Assert.*;

public class ListingTest {
    static { System.loadLibrary("listing_test"); }
    private static native int count(String uri) throws IOException;

    private int list(String id) throws IOException {
        return count("content://one.rachelt.rust_saf.test.listings/tree/" + id + "/document/" + id);
    }

    private void expectFailure(String id, String message) throws IOException {
        try {
            int count = list(id);
            fail(id + ": expected a listing error, got " + count + " entries");
        } catch (IOException expected) {
            assertEquals(message, expected.getMessage());
        }
    }

    @Test public void completeListings() throws IOException {
        assertEquals(1, list("populated"));
        assertEquals(0, list("empty"));
    }

    @Test public void nullCursor() throws IOException {
        expectFailure("missing-cursor", "Document provider returned no directory cursor");
    }
    @Test public void stillLoading() throws IOException {
        expectFailure("loading", "Document provider is still loading the directory");
    }
    @Test public void providerError() throws IOException {
        expectFailure("error", "Document provider reported a directory listing error");
    }
}
