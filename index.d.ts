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
 * The supported java versions.
 * Your list of supported versions
 * may differ if you use a different
 * version of the jvm shared library.
 */
export enum java_version {
    VER_1_1 = "1.1",
    VER_1_2 = "1.2",
    VER_1_4 = "1.4",
    VER_1_6 = "1.6",
    VER_1_8 = "1.8",
    VER_9 = "9",
    VER_10 = "10"
}

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
    public constructor(jvmPath: string, jvmVersion: java_version | string | null);

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
    public getClass(classname: string): java_class_proxy;

    /**
     * Get a class proxy
     *
     * @param classname the name of the class to resolve
     * @return the created java class proxy instance (as a promise)
     */
    public getClassAsync(classname: string): Promise<java_class_proxy>;

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
     * The java vm instance
     */
    public 'java.instance': java_instance;

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
     * The java class instance
     */
    public readonly 'java.instance': java_instance;

    public static newInstance(...args: any_type[]): Promise<java_instance_proxy>;

    /**
     * Create a new java instance of type
     * java_instance_proxy["class.proxy.instance"]
     *
     * @param args the arguments to create the instance
     */
    public constructor(...args: any_type[]);
}

declare namespace java {
    namespace classpath {
        function append(path: string): void;

        function appendAsync(path: string): Promise<void>;
    }

    function createJVM(jvmPath?: string | null, version?: java_version | string | null): void;

    function importClass(classname: string): java_instance_proxy_constructor;

    function getJavaInstance(): java_instance;

    function ensureJVM(jvmPath?: string | null, version?: java_version | string | null): void;
}

export default java;