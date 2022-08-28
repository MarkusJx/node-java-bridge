package io.github.markusjx.bridge;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

@SuppressWarnings("unused")
public class Util {
    private static final Pattern STACKTRACE_PATTERN = Pattern.compile("\\t?\\s*at ([a-z\\d._\\-:]+) \\((([a-z\\d._\\\\/()\\s\\-:]+:?([a-z]+\\d*)+)+):(\\d+)(:(\\d+))?\\)", Pattern.MULTILINE | Pattern.CASE_INSENSITIVE);

    public static Exception exceptionFromJsError(String message, String[] stackTrace) {
        JavascriptException ex = new JavascriptException(message);
        StackTraceElement[] elements = ex.getStackTrace();
        ArrayList<StackTraceElement> ownElements = new ArrayList<>(Arrays.asList(elements));
        StackTraceElement thisElement = ownElements.get(0);
        ownElements.remove(0);

        List<StackTraceElement> convertedElements = new ArrayList<>();
        convertedElements.add(thisElement);

        for (String trace : stackTrace) {
            Matcher matcher = STACKTRACE_PATTERN.matcher(trace);
            if (matcher.matches()) {
                convertedElements.add(new StackTraceElement("external", matcher.group(1), matcher.group(2), Integer.parseInt(matcher.group(5))));
            }
        }

        convertedElements.addAll(ownElements);
        ex.setStackTrace(convertedElements.toArray(new StackTraceElement[0]));
        return ex;
    }
}