package io.github.markusjx.bridge;

import java.lang.reflect.InvocationHandler;
import java.lang.reflect.Method;
import java.util.Arrays;
import java.util.List;

/**
 * A class for creating java interface proxies using node.js functions.
 * This is mostly based on: https://github.com/joeferner/node-java/blob/master/src-java/node/NodeDynamicProxyClass.java
 */
public class JavaFunctionCaller implements InvocationHandler {
    private static final Method EQUALS;
    private static final Method HASH_CODE;
    private static final Method TO_STRING;

    static {
        try {
            EQUALS = Object.class.getMethod("equals", Object.class);
            HASH_CODE = Object.class.getMethod("hashCode");
            TO_STRING = Object.class.getMethod("toString");
        } catch (NoSuchMethodException e) {
            throw new ExceptionInInitializerError(e);
        }
    }

    /**
     * The functions that were implemented in the node process
     */
    private final List<String> implementedMethods;
    /**
     * The pointer to the native proxy class
     */
    private final long ptr;
    /**
     * Whether this caller instance is still valid
     */
    private boolean valid;

    /**
     * Create a new JavaFunctionCaller instance
     *
     * @param nativePath         the path of the native library
     * @param implementedMethods the form the node process implemented functions
     * @param ptr                the pointer to the native proxy class
     */
    public JavaFunctionCaller(String nativePath, String[] implementedMethods, long ptr) {
        Runtime.getRuntime().load(nativePath.replace('\\', '/'));

        this.implementedMethods = Arrays.asList(implementedMethods);
        this.ptr = ptr;
        this.valid = true;
    }

    /**
     * Call a java function from node.js
     *
     * @param ptr  the pointer to the native proxy class
     * @param m    the method to call
     * @param args the function arguments
     * @return the function return value
     */
    private native Object callNodeFunction(long ptr, Method m, Object[] args);

    /**
     * Mark this caller as invalid
     */
    @SuppressWarnings("unused")
    public void destruct() {
        valid = false;
    }

    @Override
    public Object invoke(Object proxy, Method method, Object[] args) throws Throwable {
        if (!valid) {
            throw new IllegalAccessException("The proxy interface isn't valid anymore");
        } else if (method.equals(EQUALS)) {
            return args[0] == proxy;
        } else if (method.equals(HASH_CODE)) {
            return System.identityHashCode(proxy);
        } else if (method.equals(TO_STRING)) {
            return "JavaFunctionCaller{" +
                    "implementedMethods=" + implementedMethods +
                    ", ptr=" + ptr +
                    '}';
        } else {
            if (implementedMethods.contains(method.getName())) {
                return callNodeFunction(ptr, method, args);
            } else {
                throw new NoSuchMethodException("The requested method was not defined by the javascript process");
            }
        }
    }
}
