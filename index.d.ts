/**
 * The native addon type.
 * Documentation for the implemented methods
 * is provided in the native C++ code.
 */
declare namespace native {
    const java: typeof java_instance;

    const java_function_caller: typeof java_function_caller_class;

    function setLoggerMode(mode: java.logging.LogLevel | number): void;

    function setNativeLibraryPath(path: string, rootDir: string): void;

    namespace stdout_redirect {
        function setCallbacks(stdout?: StdoutCallback | null, stderr?: StdoutCallback | null): void;

        function reset(): void;
    }
}

declare type basic_type = string | number | boolean | BigInt | null;
declare type basic_or_java = basic_type | java_object | JavaConstructor;
declare type any_type = basic_or_java | basic_or_java[];

/**
 * All types accepted by java
 */
export type JavaType = any_type;
export type StdoutCallback = (line: string) => void;

/**
 * The java instance.
 * This holds the vm instance
 * and may only exist once in any context.
 */
export class java_instance {
    /**
     * The vm version string
     */
    public readonly version: string;

    /**
     * The vm version requested on creation
     */
    public readonly wantedVersion: string;

    /**
     * Create a new java instance
     *
     * @param jvmPath the path to the jvm shared library (jvm.dll/.so/.dylib)
     * @param jvmVersion the jvm version to request
     */
    public constructor(jvmPath: string, jvmVersion: java.java_version | string | null);

    /**
     * Get the list of loaded jar files
     *
     * @return the list of loaded jars
     */
    public get loadedJars(): string[];

    /**
     * Get a class proxy
     *
     * @param classname the name of the class to search for
     * @return the created java class proxy instance
     */
    public static getClass(classname: string): java_class_proxy;

    /**
     * Get a class proxy. Async version
     *
     * @param classname the name of the class to search for
     * @return the created java class proxy instance
     */
    public static getClassAsync(classname: string): Promise<java_class_proxy>;

    /**
     * Destroy the jvm.
     * This will delete the java vm instance
     * and make all subsequent calls to the
     * vm invalid in as they'll throw exceptions.
     */
    public static destroyJVM(): void;

    /**
     * Append a jar to the classpath.
     * This will not actually append anything to
     * the current classpath but rather create a
     * new class loader with the current class
     * loader as its parent class loader. The
     * newly created class loader will contain
     * the jar path in its search path and will
     * be used for all future class resolve operations.
     *
     * @param path the path of the jar to append
     */
    public appendToClasspath(path: string): void;

    /**
     * Append a jar to the classpath.
     * Async version.
     *
     * @param path the path to append
     */
    public appendToClasspathAsync(path: string): Promise<void>;
}

/**
 * A dummy java object class
 */
export abstract class java_object {
}

/**
 * A java class proxy class.
 * This only exists for temporarily storing
 * the class name and the java instance
 * to create the actual class from using the
 * {@link java_class_proxy.getClassConstructor()}
 * function.
 */
declare class java_class_proxy {
    /**
     * The class name
     */
    public 'class.name': string;

    /**
     * Get the class's constructor
     *
     * @return the java instance proxy constructor
     */
    public getClassConstructor(): java_instance_proxy_constructor;
}

type JavaClassType = typeof JavaClass;

/**
 * A java class's constructor
 */
declare type java_instance_proxy_constructor<T extends JavaClassType = JavaClassType> = T & ImportedMembers;

/**
 * A java class's constructor
 */
export type JavaConstructor<T extends JavaClassType = JavaClassType> = java_instance_proxy_constructor<T>;

/**
 * A java class instance
 */
export class JavaClass extends java_instance_proxy {
}

/**
 * Any class member imported from java
 */
export interface ImportedMembers {
    /**
     * Any class member imported.
     * We'll need to use 'any' as any is callable.
     * The actual type would be JavaType | ((...args: JavaType[]) => JavaType | Promise<JavaType>)
     */
    [member: string]: any;
}

/**
 * The java instance proxy class.
 * This class actually does all the magic.
 * After it is created, this will just be a constructor
 * with all static methods and properties (the accessible ones)
 * stored in it and ready for use. Once the actual instance
 * using the new operator is created, a new
 * java_instance_proxy instance is created, containing
 * the actual java instance (that thing isn't visible though)
 * and all (visible) non-static class member methods and properties.
 */
export class java_instance_proxy extends java_object implements ImportedMembers {
    /**
     * The class proxy class instance
     */
    public static readonly 'class.proxy.instance': java_class_proxy;

    /**
     * Create a new java class instance.
     * Async version.
     *
     * @template T the type of this class as a new instance of this class will be returned
     * @param args the arguments to create the instance
     * @return the java_instance_proxy instance
     */
    public static newInstance<T extends java_instance_proxy = java_instance_proxy>(...args: any_type[]): Promise<T>;

    /**
     * Create a new java instance of type
     * java_instance_proxy["class.proxy.instance"]
     *
     * @param args the arguments to create the instance
     */
    public constructor(...args: any_type[]);

    /**
     * Check if this is an instance of another class
     *
     * @param classname the class to check if this is an instance of
     * @return true if this is instance of classname
     */
    public instanceOf(classname: string): boolean;

    /**
     * Default java equals implementation.
     * Async call.
     *
     * @param o the object to compare this to
     * @returns true if this matches o
     */
    public equals(o: java_instance_proxy): Promise<boolean>;

    /**
     * Default java equals implementation.
     * Sync call.
     *
     * @param o the object to compare this to
     * @returns true if this matches o
     */
    public equalsSync(o: java_instance_proxy): boolean;

    /**
     * Java default toString method.
     * Async call.
     *
     * @returns this as a string
     */
    public toString(): Promise<string>;

    /**
     * Java default toString method.
     * Sync call.
     *
     * @returns this as a string
     */
    public toStringSync(): string;

    /**
     * Any class member imported.
     * We'll need to use 'any' as any is callable.
     * The actual type would be JavaType | ((...args: JavaType[]) => JavaType | Promise<JavaType>).
     * Just throwing it out there.
     */
    [member: string]: any;
}

export class JavaClassInstance extends java_instance_proxy {
}

/**
 * An interface defining the proxy function object layout.
 * See: https://stackoverflow.com/a/56217448
 */
interface ProxyMethods {
    [key: string]: (...args: JavaType[]) => JavaType | void;
}

/**
 * The class for implementing java interfaces
 */
declare class java_function_caller_class extends java_object {
    /**
     * Create a function_caller_class instance
     *
     * @param name the name if the interface to implement
     * @param methods the methods to override
     */
    public constructor(name: string, methods: ProxyMethods);

    /**
     * Destroy the proxy class.
     * Throws an error if the proxy has already been destroyed.
     */
    public destroy(): Promise<void>;
}

/**
 * The class for implementing java interfaces
 */
export type JavaInterfaceProxy = java_function_caller_class;

/**
 * The main java namespace
 */
declare namespace java {
    /**
     * Functions for altering the class path
     */
    namespace classpath {
        /**
         * Append a (or multiple) jar(s) to the class path
         *
         * @param path the path(s) to append to the class path
         */
        function append(path: string | string[]): void;

        /**
         * Append a (or multiple) jar(s) to the class path.
         * Async version.
         *
         * @param path the path(s) to append to the class path
         */
        function appendAsync(path: string | string[]): Promise<void>;
    }

    /**
     * The supported java versions.
     * Your list of supported versions
     * may differ if you use a different
     * version of the jvm shared library.
     */
    enum java_version {
        VER_1_1 = "1.1",
        VER_1_2 = "1.2",
        VER_1_4 = "1.4",
        VER_1_6 = "1.6",
        VER_1_8 = "1.8",
        VER_9 = "9",
        VER_10 = "10"
    }

    /**
     * Create a new java instance.
     * This will destroy the old instance.
     * The creation process might take a while.
     *
     * @param jvmPath the path to the jvm shared library
     * @param version the version to use
     */
    function createJVM(jvmPath?: string | null, version?: java_version | string | null): void;

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
    function importClass<T extends JavaClassType = JavaClassType>(classname: string): JavaConstructor<T>;

    /**
     * Import a class. Async version.
     *
     * @param classname the name of the class to resolve
     * @return the java_instance_proxy constructor
     */
    function importClassAsync<T extends JavaClassType = JavaClassType>(classname: string): Promise<JavaConstructor<T>>;

    /**
     * Get the java class instance
     *
     * @return the java class instance
     */
    function getJavaInstance(): java_instance;

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
    function newProxy(name: string, methods: ProxyMethods): JavaInterfaceProxy;

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
    function ensureJVM(jvmPath?: string | null, version?: java_version | string | null): void;

    /**
     * Destroy the jvm.
     * This will delete the java vm instance
     * and make all subsequent calls to the
     * vm invalid in as they'll throw exceptions.
     */
    function destroyJVM(): void;

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
    function findJVM(allowJre?: boolean): Promise<string>;

    /**
     * A namespace for logging utilities
     */
    namespace logging {
        /**
         * Set the log level for the java module.
         * Only displays logging information in the console.
         * The level should be either one from LogLevel
         * or a number defined in LogLevel.
         *
         * @param level the level to set
         */
        function setLogLevel(level: LogLevel | number): void;

        /**
         * A log level to set
         */
        enum LogLevel {
            // Show debug, warning and error messages
            DEBUG = 0,
            // Show warning and error messages
            WARNING = 1,
            // Only show error messages
            ERROR = 2,
            // Log nothing at all
            NONE = 3
        }
    }

    /**
     * A namespace for redirecting the stdout/stderr
     */
    namespace stdoutRedirect {
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
        function enableRedirect(stdout?: StdoutCallback | null, stderr?: StdoutCallback | null): void;

        /**
         * Remove any redirects created and print to the console again.
         * Does nothing if no redirects have been created (that is not a no-op).
         */
        function reset(): void;
    }
}

export default java;