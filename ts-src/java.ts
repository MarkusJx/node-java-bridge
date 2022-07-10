import {
    JavaClassType,
    JavaConfig,
    JavaConstructor,
    JavaInterfaceProxy,
    JavaVersion as JavaVersion_1,
    JavaVMInstance,
    LogLevel as LogLevel_1,
    ProxyMethods,
    StdoutCallback,
} from './definitions';
import { findJavaLibrary } from './findJavaLibrary';
import native, { JVM_PATH } from './native';

let javaInstance: JavaVMInstance | null = null;

/**
 * The main java namespace
 */
namespace java {
    /**
     * Set the config for the java instance.
     * This may be set at runtime or anytime during execution.
     * It is recommended to set this before any other calls to the jvm.
     */
    export function setConfig(config: JavaConfig): void {
        native.java.config = config;
    }

    /**
     * Get the config for the java instance.
     */
    export function getConfig(): JavaConfig {
        return native.java.config;
    }

    /**
     * Functions for altering the class path
     */
    export namespace classpath {
        /**
         * Append a (or multiple) jar(s) to the class path
         *
         * @param path the path(s) to append to the class path
         */
        export function append(path: string | string[]): void {
            ensureJVM();
            javaInstance!.appendToClasspath(path);
        }

        /**
         * Append a (or multiple) jar(s) to the class path.
         * Async version.
         *
         * @param path the path(s) to append to the class path
         */
        export function appendAsync(path: string | string[]): Promise<void> {
            ensureJVM();
            return javaInstance!.appendToClasspathAsync(path);
        }
    }

    export const JavaVersion = JavaVersion_1;

    /**
     * Create a new java instance.
     * This will destroy the old instance.
     * The creation process might take a while.
     *
     * @param jvmPath the path to the jvm shared library
     * @param version the version to use
     */
    export function createJVM(jvmPath?: string | null, version?: JavaVersion_1 | string | null): void {
        native.java.destroyJVM();
        javaInstance = null;
        ensureJVM(jvmPath, version);
    }

    /**
     * Import a class.
     * Returns the constructor of the class to be created.
     * For example, import "java.util.ArrayList" for a java Array List.
     *
     * Define a custom class type for the imported class and pass the
     * constructor type of the class as the template parameter to get
     * the proper type returned. You could also just cast the result.
     *
     * @template T the type of the java class to import as a js type
     * @param classname the name of the class to resolve
     * @return the java_instance_proxy constructor
     */
    export function importClass<T extends JavaClassType = JavaClassType>(classname: string): JavaConstructor<T> {
        ensureJVM();
        return native.java.getClass(classname).getClassConstructor<T>();
    }

    /**
     * Import a class. Async version.
     *
     * @param classname the name of the class to resolve
     * @return the java_instance_proxy constructor
     */
    export async function importClassAsync<T extends JavaClassType = JavaClassType>(
        classname: string
    ): Promise<JavaConstructor<T>> {
        ensureJVM();
        const proxy = await native.java.getClassAsync(classname);
        return proxy.getClassConstructor<T>();
    }

    /**
     * Get the java class instance
     *
     * @return the java class instance
     */
    export function getJavaInstance(): JavaVMInstance | null {
        return javaInstance;
    }

    /**
     * Create a proxy for a java interface.
     * The functions must be supplied in an object with the name
     * of the function to override as a key and the function
     * as a value. Any function called from java will be run
     * in the node main thread (v8 doesn't really support multithreading),
     * therefore a queue is used to call the functions, which may
     * take a while, so keep that in mind.
     *
     * @param name the name of the interface to 'implement'
     * @param methods the methods to implement
     */
    export function newProxy(name: string, methods: ProxyMethods): JavaInterfaceProxy {
        ensureJVM();
        return new native.java_function_caller(name, methods);
    }

    /**
     * Ensure that the jvm exists.
     * If any argument is set to null, it will be ignored,
     * thus, any argument may be omitted. The path to the jvm
     * shared library should contain the full path to the
     * jvm.(dll|so|dylib). If omitted, the path to the shared
     * library will be used that was found on installation.
     *
     * The version argument should be either of type java_version
     * or a string defined in the java_version type. For example,
     * use "1.8" for java version 1.8. If omitted, version 1.8 will
     * be selected if the native binary was compiled with a version
     * greater than or equal to 1.8. If that is not the case,
     * version 1.6 will be selected by default.
     *
     * @param jvmPath the path to the jvm shared library
     * @param version the version to use
     */
    export function ensureJVM(jvmPath?: string | null, version?: JavaVersion_1 | string | null): void {
        if (javaInstance == null) {
            if (!jvmPath) {
                jvmPath = JVM_PATH;
            }

            javaInstance = new native.java(jvmPath, version ?? null);
        }
    }

    /**
     * Destroy the jvm.
     * This will delete the java vm instance
     * and make all subsequent calls to the
     * vm invalid in as they'll throw exceptions.
     */
    export function destroyJVM(): void {
        native.java.destroyJVM();
    }

    /**
     * Find a suitable JVM or JRE and return it.
     * Throws an error if no jvm could be found.
     * Returns the appropriate path to the jvm shared
     * library adapted to the current operating system.
     * This may be passed to {@link ensureJVM} to start
     * a new jvm instance.
     *
     * @param allowJre whether to allow finding jre instances. Defaults to true.
     * @return the path to the jvm shared library
     */
    export function findJVM(allowJre?: boolean): Promise<string> {
        return findJavaLibrary(allowJre);
    }

    /**
     * A namespace for logging utilities
     */
    export namespace logging {
        /**
         * Set the log level for the java module.
         * Only displays logging information in the console.
         * The level should be either one from LogLevel
         * or a number defined in LogLevel.
         *
         * @param level the level to set
         */
        export function setLogLevel(level: LogLevel_1 | number): void {
            native.setLoggerMode(level);
        }

        export const LogLevel = LogLevel_1;
    }

    /**
     * A namespace for redirecting the stdout/stderr
     */
    export namespace stdoutRedirect {
        /**
         * Enable redirecting the stdout/stderr to custom callbacks.
         * If enabled, nothing will be printed to the default stdout/stderr.
         * Call {@link reset} to print to the default stdout/stderr again.
         * This will also be reset every time this is called again.
         * Set any parameter to null or undefined to log to the console.
         * Leave both parameters unset to create a call equal to {@link reset}.
         *
         * @param stdout the custom callback for stdout
         * @param stderr the custom callback for stdout
         */
        export function enableRedirect(stdout?: StdoutCallback | null, stderr?: StdoutCallback | null): void {
            ensureJVM();
            native.stdout_redirect.setCallbacks(stdout, stderr);
        }

        /**
         * Remove any redirects created and print to the console again.
         * Does nothing if no redirects have been created (that is not a no-op).
         */
        export function reset(): void {
            ensureJVM();
            native.stdout_redirect.reset();
        }
    }
}

Object.freeze(java);
export default java;
