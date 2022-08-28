package io.github.markusjx.bridge;

import java.io.IOException;
import java.io.OutputStream;
import java.io.PrintStream;

@SuppressWarnings("unused")
public class StdoutRedirect {
    private final PrintStream origStdout;
    private final PrintStream origStderr;

    public StdoutRedirect(boolean redirectStdout, boolean redirectStderr) {
        if (redirectStdout) {
            origStdout = System.out;
            System.setOut(new PrintStream(new CallbackOutputStream(true)));
        } else {
            origStdout = null;
        }

        if (redirectStderr) {
            origStderr = System.err;
            System.setErr(new PrintStream(new CallbackOutputStream(false)));
        } else {
            origStderr = null;
        }
    }

    public void reset() {
        if (origStdout != null) System.setOut(origStdout);
        if (origStderr != null) System.setErr(origStderr);
    }

    private static class CallbackOutputStream extends OutputStream {
        private final StringBuffer stringBuffer = new StringBuffer();
        private final boolean isStdOut;

        public CallbackOutputStream(boolean isStdOut) {
            this.isStdOut = isStdOut;
        }

        @Override
        public void write(int b) throws IOException {
            stringBuffer.append((char) b);

            if (b == '\n') {
                this.writeBuffer();
            }
        }

        @Override
        public void flush() {
            if (stringBuffer.length() > 0) {
                this.writeBuffer();
            }
        }

        private synchronized void writeBuffer() {
            this.writeLine(stringBuffer.toString().trim(), this.isStdOut);
            stringBuffer.delete(0, stringBuffer.length());
        }

        private native void writeLine(String line, boolean isStdOut);
    }
}
