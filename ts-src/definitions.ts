import { inspect } from 'util';

/**
 * The supported java versions.
 * Your list of supported versions
 * may differ if you use a different
 * version of the jvm shared library.
 */
export enum JavaVersion {
    /** Java version 1.1 */
    VER_1_1 = '1.1',
    /** Java version 1.2 */
    VER_1_2 = '1.2',
    /** Java version 1.4 */
    VER_1_4 = '1.4',
    /** Java version 1.6 */
    VER_1_6 = '1.6',
    /** Java version 1.8 */
    VER_1_8 = '1.8',
    /** Java version 9 */
    VER_9 = '9',
    /** Java version 10 */
    VER_10 = '10',
}

Object.freeze(JavaVersion);

/**
 * Any basic javascript type accepted by this library.
 */
export declare type BasicType = string | number | boolean | BigInt | null;

/**
 * Any java type accepted by this library, except arrays.
 */
export type BasicOrJavaType =
    | BasicType
    | JavaObject
    | JavaClass
    | JavaClassType;

/**
 * All types accepted by java
 */
export type JavaType = BasicOrJavaType | BasicOrJavaType[];

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
export declare class JavaClassProxy {
    /**
     * The class name
     */
    public 'class.name': string;

    /**
     * Get the class's constructor
     *
     * @return the java instance proxy constructor
     */
    public getClassConstructor<
        T extends JavaClassType = UnknownJavaClassType
    >(): T;
}

export type JavaClassType = typeof JavaClass;
export type UnknownJavaClassType = typeof UnknownJavaClass;
export type JavaClassConstructorType = typeof JavaClassConstructor;

/**
 * @inheritDoc UnknownJavaClass
 */
export declare class JavaClassInstance extends UnknownJavaClass {}

/**
 * A java class constructor class
 *
 * @see JavaClass
 */
export declare class JavaClassConstructor extends JavaClass {
    public constructor(...args: BasicOrJavaType[]);
}

/**
 * A constructor type.
 */
export type Constructor<T> = { new (): T };

/**
 * A class to be extended for custom class definitions.
 * This does not allow for any methods to be called if not
 * defined in the class definition.
 *
 * ## Example
 * ```ts
 * import { importClass } from 'java-bridge';
 *
 * declare class PersonClass extends JavaClass {
 *     public constructor(name: string, age: number);
 *     public newInstanceAsync(name: string, age: number): Promise<Person>;
 *
 *     public getName(): Promise<string>;
 *     public getNameSync(): string;
 *     public getAge(): Promise<number>;
 *     public getAgeSync(): number;
 * }
 *
 * class Person extends importClass<typeof PersonClass>('com.test.Person') {}
 *
 * const person = new Person('John', 20);
 * console.log(person.getNameSync()); // John
 * console.log(person.getAgeSync()); // 20
 * ```
 */
export declare class JavaClass extends JavaObject {
    /**
     * Get the java class instance
     */
    public static get class(): UnknownJavaClass;

    /**
     * The class proxy class instance
     */
    public static readonly 'class.proxy': JavaClassProxy;

    /**
     * Check if this is an instance of another class.
     * Pass either the name of the other class or the class itself
     * to check if this is an instance of it.
     * Does not overwrite any existing instanceof operator.
     * This uses the native java instanceof operator.
     *
     * ## Example
     * ```ts
     * import { importClass } from 'java-bridge';
     *
     * const JavaString = importClass('java.lang.String');
     * const str = new JavaString('Hello World');
     *
     * str.instanceOf(JavaString); // true
     * str.instanceOf('java.lang.String'); // true
     * str.instanceOf('java.lang.Object'); // true
     * str.instanceOf('java.lang.Integer'); // false
     * ```
     *
     * @param other the class to check if this is an instance of
     * @return true if this is instance of `other`
     */
    public instanceOf<T extends object>(other: string | T): boolean;

    /**
     * Default java equals implementation.
     * Async call.
     *
     * @param o the object to compare this to
     * @returns true if this matches o
     */
    public equals(o: JavaClass): Promise<boolean>;

    /**
     * Default java equals implementation.
     * Sync call.
     *
     * @param o the object to compare this to
     * @returns true if this matches o
     */
    public equalsSync(o: JavaClass): boolean;

    /**
     * Java default toString method.
     * Sync call.
     *
     * @returns this as a string
     */
    public toString(): string;

    /**
     * Java default toString method.
     * Sync call.
     *
     * @deprecated use {@link toString} instead
     * @returns this as a string
     */
    public toStringSync(): string;

    /**
     * Java default toString method.
     * Async call.
     *
     * @since 2.4.0
     * @returns this as a string
     */
    public toStringAsync(): Promise<string>;

    /**
     * Java default hashCode method.
     * Async call.
     */
    public hashCode(): Promise<number>;

    /**
     * Java default hashCode method.
     * Sync call.
     */
    public hashCodeSync(): number;

    /**
     * Custom inspect method for an improved console.log output.
     * This will return the output of {@link toString}.
     * Will only be defined if {@link JavaConfig.customInspect} is true.
     *
     * @since 2.4.0
     * @returns this as a string
     */
    public [inspect.custom]?(): string;
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
export declare class UnknownJavaClass extends JavaClass {
    /**
     * Create a new java class instance.
     * Async version.
     *
     * @template T the type of this class as a new instance of this class will be returned
     * @param args the arguments to create the instance
     * @return the java_instance_proxy instance
     */
    public static newInstanceAsync(
        this: never,
        ...args: BasicOrJavaType[]
    ): Promise<unknown>;
    public static newInstanceAsync<T extends JavaClass>(
        this: Constructor<T>,
        ...args: BasicOrJavaType[]
    ): Promise<T>;

    /**
     * Create a new java instance of type
     * java_instance_proxy["class.proxy.instance"]
     *
     * @param args the arguments to create the instance
     */
    public constructor(...args: BasicOrJavaType[]);

    /**
     * Any class member imported.
     * We'll need to use 'any' as any is callable.
     * The actual type would be JavaType | ((...args: JavaType[]) => JavaType | Promise<JavaType>).
     * Just throwing it out there.
     */
    [member: string]: any;

    /**
     * Any static class member imported.
     * We'll need to use `any` as `any` is callable.
     * The actual type would be JavaType | ((...args: JavaType[]) => JavaType | Promise<JavaType>)
     */
    static [member: string]: any;
}
