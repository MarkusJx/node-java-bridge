/**
 * The native addon type
 */
declare namespace native {
    const java: typeof java_instance;
}

declare type basic_type = string | number | boolean;
declare type basic_or_java = basic_type | java_object;
declare type any_type = basic_or_java | basic_or_java[];

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
declare class java_class_proxy extends java_object {
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

/**
 * A java class's constructor
 */
declare type java_instance_proxy_constructor = typeof java_instance_proxy;

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
export class java_instance_proxy extends java_object {
    /**
     * The class proxy class instance
     */
    public static readonly 'class.proxy.instance': java_class_proxy;

    /**
     * Create a new java class instance.
     * Async version.
     *
     * @param args the arguments to create the instance
     * @return the java_instance_proxy instance
     */
    public static newInstance(...args: any_type[]): Promise<java_instance_proxy>;

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
}

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
     *
     * @param jvmPath the path to the jvm shared library
     * @param version the version to use
     */
    function createJVM(jvmPath?: string | null, version?: java_version | string | null): void;

    /**
     * Import a class
     *
     * @param classname the name of the class to resolve
     * @return the java_instance_proxy constructor
     */
    function importClass(classname: string): java_instance_proxy_constructor;

    /**
     * Import a class. Async version.
     *
     * @param classname the name of the class to resolve
     * @return the java_instance_proxy constructor
     */
    function importClassAsync(classname: string): Promise<java_instance_proxy_constructor>;

    /**
     * Get the java class instance
     *
     * @return the java class instance
     */
    function getJavaInstance(): java_instance;

    /**
     * Ensure that the jvm exists
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
     * A namespace for logging utilities
     */
    namespace logging {
        /**
         * Set the log level for the java module
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
}

export default java;