/**
 * The native addon type.
 * Documentation for the implemented methods
 * is provided in the native C++ code.
 */
export interface nativeAddon {
    readonly java: typeof JavaVMInstance;

    readonly java_function_caller: typeof JavaInterfaceProxy;

    setLoggerMode(mode: LogLevel | number): void;

    setNativeLibraryPath(path: string, rootDir: string): void;

    readonly stdout_redirect: {
        setCallbacks(stdout?: StdoutCallback | null, stderr?: StdoutCallback | null): void;

        reset(): void;
    };
}

/**
 * A log level to set
 */
export enum LogLevel {
    // Show debug, warning and error messages
    DEBUG = 0,
    // Show warning and error messages
    WARNING = 1,
    // Only show error messages
    ERROR = 2,
    // Log nothing at all
    NONE = 3,
}

Object.freeze(LogLevel);

/**
 * The supported java versions.
 * Your list of supported versions
 * may differ if you use a different
 * version of the jvm shared library.
 */
export enum JavaVersion {
    VER_1_1 = '1.1',
    VER_1_2 = '1.2',
    VER_1_4 = '1.4',
    VER_1_6 = '1.6',
    VER_1_8 = '1.8',
    VER_9 = '9',
    VER_10 = '10',
}

Object.freeze(JavaVersion);

declare type BasicType = string | number | boolean | BigInt | null;
declare type BasicOrJavaType = BasicType | JavaObject | JavaConstructor;

/**
 * All types accepted by java
 */
export type JavaType = BasicOrJavaType | BasicOrJavaType[];
export type StdoutCallback = (line: string) => void;

export interface JavaConfig {
    useDaemonThreads?: boolean;
}

/**
 * The java instance.
 * This holds the vm instance
 * and may only exist once in any context.
 */
export declare class JavaVMInstance {
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
    public constructor(jvmPath: string, jvmVersion: JavaVersion | string | null);

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
    public static getClass(classname: string): JavaClassProxy;

    /**
     * Get a class proxy. Async version
     *
     * @param classname the name of the class to search for
     * @return the created java class proxy instance
     */
    public static getClassAsync(classname: string): Promise<JavaClassProxy>;

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
    public appendToClasspath(path: string | string[]): void;

    /**
     * Append a jar to the classpath.
     * Async version.
     *
     * @param path the path to append
     */
    public appendToClasspathAsync(path: string | string[]): Promise<void>;

    /**
     * Set the config for the java instance.
     * This may be set at runtime or anytime during execution.
     * It is recommended to set this before any other calls to the jvm.
     */
    public static set config(config: JavaConfig);

    /**
     * Get the config for the java instance.
     */
    public static get config(): JavaConfig;
}

/**
 * A dummy java object class
 */
export abstract class JavaObject {}

/**
 * A java class proxy class.
 * This only exists for temporarily storing
 * the class name and the java instance
 * to create the actual class from using the
 * {@link JavaClassProxy.getClassConstructor()}
 * function.
 */
declare class JavaClassProxy {
    /**
     * The class name
     */
    public 'class.name': string;

    /**
     * Get the class's constructor
     *
     * @return the java instance proxy constructor
     */
    public getClassConstructor<T extends JavaClassType = JavaClassType>(): JavaConstructor<T>;
}

export type JavaClassType = typeof JavaClassInstance;

/**
 * A java class's constructor
 */
export type JavaConstructor<T extends JavaClassType = JavaClassType> = T & ImportedMembers;

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
export declare class JavaClassInstance extends JavaObject implements ImportedMembers {
    /**
     * The class proxy class instance
     */
    public static readonly 'class.proxy.instance': JavaClassProxy;

    /**
     * Create a new java class instance.
     * Async version.
     *
     * @template T the type of this class as a new instance of this class will be returned
     * @param args the arguments to create the instance
     * @return the java_instance_proxy instance
     */
    public static newInstance<T extends JavaClassInstance = JavaClassInstance>(...args: BasicOrJavaType[]): Promise<T>;

    /**
     * Create a new java instance of type
     * java_instance_proxy["class.proxy.instance"]
     *
     * @param args the arguments to create the instance
     */
    public constructor(...args: BasicOrJavaType[]);

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
    public equals(o: JavaClassInstance): Promise<boolean>;

    /**
     * Default java equals implementation.
     * Sync call.
     *
     * @param o the object to compare this to
     * @returns true if this matches o
     */
    public equalsSync(o: JavaClassInstance): boolean;

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

/**
 * An interface defining the proxy function object layout.
 * See: https://stackoverflow.com/a/56217448
 */
export interface ProxyMethods {
    [key: string]: (...args: JavaType[]) => JavaType | void;
}

/**
 * The class for implementing java interfaces
 */
export declare class JavaInterfaceProxy extends JavaObject {
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
