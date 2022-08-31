import {
    getClassFields,
    getField,
    getStaticField,
    Java,
    JavaOptions,
    setField,
    setStaticField,
} from '../native';
import {
    JavaClassInstance,
    JavaClassProxy,
    JavaClassType,
    JavaConstructor,
    JavaVersion,
} from './definitions';
import { getJavaLibPath, getNativeLibPath } from './nativeLib';

/**
 * The static java instance
 */
let javaInstance: Java | null = null;

interface ImportedJavaClass {
    'class.proxy': object;
    new (...args: any[]): any;
}

/**
 * Options for creating the Java VM.
 */
export interface JVMOptions extends JavaOptions {
    /***
     * The path to the native library
     */
    libPath?: string | null;
    /**
     * The version of the jvm to request
     */
    version?: string | JavaVersion | null;
    /**
     * Additional arguments to pass to the JVM
     */
    opts?: Array<string> | null;
}

/**
 * Ensure the java vm is created.
 * If the jvm is already created, this does nothing.
 * If the vm is not created yet, the jvm will be created upon this call.
 * This method is also called every time with no arguments when any call
 * to the jvm is done in another method.
 *
 * ## Examples
 * Specify the path to jvm.(dylib|dll|so) manually,
 * specify the java version to use and set to use daemon threads.
 * ```ts
 * import { ensureJvm, JavaVersion } from 'java-bridge';
 *
 * ensureJvm({
 *     libPath: 'path/to/jvm.dll',
 *     version: JavaVersion.VER_9,
 *     useDaemonThreads: true
 * });
 * ```
 *
 * Let the plugin find the jvm.(dylib|dll|so)
 * ```ts
 * ensureJvm({
 *     JavaVersion.VER_9,
 *     useDaemonThreads: true
 * });
 * ```
 *
 * Let the plugin find the jvm.(dylib|dll|so) and use the default options
 * ```ts
 * ensureJvm();
 * ```
 *
 * @param options the options to use when creating the jvm
 */
export function ensureJvm(options?: JVMOptions): void {
    if (!javaInstance) {
        javaInstance = new Java(
            options?.libPath,
            options?.version,
            options?.opts,
            options,
            getJavaLibPath(),
            getNativeLibPath()
        );
    }
}

function defineFields(object: Record<string, any>, getStatic: boolean): void {
    for (const field of getClassFields(object['class.proxy'], getStatic)) {
        const getter = (): any =>
            getStatic
                ? getStaticField(object, field.name)
                : getField(object, field.name);
        if (field.isFinal) {
            Object.defineProperty(object, field.name, {
                get: getter,
                enumerable: true,
            });
        } else {
            Object.defineProperty(object, field.name, {
                get: getter,
                set: (value: any) =>
                    getStatic
                        ? setStaticField(object, field.name, value)
                        : setField(object, field.name, value),
                enumerable: true,
            });
        }
    }
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
 * ## Examples
 * ### Import ``java.util.ArrayList`` and create a new instance of it
 * ```ts
 * import { importClass } from 'java-bridge';
 *
 * // Import java.util.ArrayList
 * const ArrayList = importClass('java.util.ArrayList');
 *
 * // Create a new instance of ArrayList
 * const list = new ArrayList();
 * ```
 *
 * ### Import ``java.util.ArrayList`` with types
 * ```ts
 * import { importClass, JavaClassInstance, JavaType } from 'java-bridge';
 *
 * /**
 *  * Definitions for class java.util.List
 *  *\/
 * declare class List <T extends JavaType> extends JavaClassInstance {
 *     size(): Promise<number>;
 *     sizeSync(): number;
 *     add(e: T): Promise<void>;
 *     addSync(e: T): void;
 *     get(index: number): Promise<T>;
 *     getSync(index: number): T;
 *     toArray(): Promise<T[]>;
 *     toArraySync(): T[];
 *     isEmpty(): Promise<boolean>;
 *     isEmptySync(): boolean;
 * }
 *
 * /**
 *  * Definitions for class java.util.ArrayList
 *  *\/
 * declare class ArrayListClass<T extends JavaType> extends List<T> {
 *     public constructor(other: ArrayListClass<T>);
 *     public constructor();
 * }
 *
 * // This causes the class to be import when the module is loaded.
 * class ArrayList<T> extends importClass<typeof ArrayListClass>('java.util.ArrayList')<T> {}
 *
 * // Create a new ArrayList instance
 * const list = new ArrayList<string>();
 *
 * // Add some contents to the list
 * list.add('Hello');
 * list.add('World');
 *
 * // Check the list contents
 * assert.equals(list.sizeSync(), 2);
 * assert.equals(list.getSync(0), 'Hello');
 * assert.equals(list.getSync(1), 'World');
 * ```
 *
 * @template T the type of the java class to import as a js type
 * @param classname the name of the class to resolve
 * @return the java class constructor
 */
export function importClass<T extends JavaClassType = JavaClassType>(
    classname: string
): JavaConstructor<T> {
    ensureJvm();
    const constructor = javaInstance!.importClass(
        classname
    ) as ImportedJavaClass;
    defineFields(constructor, true);

    constructor.constructor = function (...args: any[]) {
        const object = new constructor.prototype.constructor(...args);
        defineFields(object, false);

        return object;
    };

    return constructor as unknown as JavaConstructor<T>;
}

/**
 * @inheritDoc importClass
 */
export async function importClassAsync<T extends JavaClassType = JavaClassType>(
    classname: string
): Promise<JavaConstructor<T>> {
    ensureJvm();
    const constructor = (await javaInstance!.importClassAsync(
        classname
    )) as ImportedJavaClass;
    defineFields(constructor, true);

    constructor.constructor = function (...args: any[]) {
        const object = new constructor.prototype.constructor(...args);
        defineFields(object, false);

        return object;
    };

    return constructor as unknown as JavaConstructor<T>;
}

/**
 * Append a single or multiple jars to the class path.
 *
 * Just replaces the old internal class loader with a new one containing the new jars.
 * This doesn't check if the jars are valid and/or even exist.
 * The new classpath will be available to all classes imported after this call.
 *
 * ## Example
 * ```ts
 * import { appendClasspath } from 'java-bridge';
 *
 * // Append a single jar to the class path
 * appendClasspath('/path/to/jar.jar');
 *
 * // Append multiple jars to the class path
 * appendClasspath(['/path/to/jar1.jar', '/path/to/jar2.jar']);
 * ```
 * or
 * ```ts
 * import { classpath } from 'java-bridge';
 *
 * // Append a single jar to the class path
 * classpath.append('/path/to/jar.jar');
 * ```
 *
 * @param path the path(s) to add
 */
export function appendClasspath(path: string | string[]): void {
    ensureJvm();
    javaInstance!.appendClasspath(path);
}

/**
 * Check if `this_obj` is instance of `other`.
 * This uses the native java `instanceof` operator.
 * You may want to use this if {@link JavaClassInstance.instanceOf}
 * is overridden, as that method itself does not override
 * any method defined in the specific java class named 'instanceOf'.
 *
 * ## Example
 * ```ts
 * import { instanceOf, importClass } from 'java-bridge';
 *
 * const ArrayList = importClass('java.util.ArrayList');
 * const list = new ArrayList();
 *
 * isInstanceOf(list, ArrayList); // true
 * isInstanceOf(list, 'java.util.ArrayList'); // true
 * isInstanceOf(list, 'java.util.List'); // true
 * isInstanceOf(list, 'java.util.Collection'); // true
 * isInstanceOf(list, 'java.lang.Object'); // true
 * isInstanceOf(list, 'java.lang.String'); // false
 *
 * // You can also use the instanceOf method (if not overridden)
 * list.instanceOf(ArrayList); // true
 * list.instanceOf('java.util.ArrayList'); // true
 * list.instanceOf('java.util.List'); // true
 * list.instanceOf('java.util.Collection'); // true
 * list.instanceOf('java.lang.Object'); // true
 * list.instanceOf('java.lang.String'); // false
 * ```
 *
 * @param this_obj the object to check
 * @param other the class or class name to check against
 * @return true if `this_obj` is an instance of `other`
 */
export function isInstanceOf<T extends typeof JavaClassInstance>(
    this_obj: JavaClassInstance,
    other: string | T
): boolean {
    ensureJvm();
    return javaInstance!.isInstanceOf(this_obj, other);
}

/**
 * Methods for altering and querying the class path.
 * @example
 * import { classpath } from 'java-bridge';
 *
 * // Append a jar to the class path
 * classpath.append('/path/to/jar.jar');
 *
 * assert.equal(classpath.get().length, 1);
 * assert.equal(classpath.get()[0], '/path/to/jar.jar');
 */
export namespace classpath {
    /**
     * @inheritDoc appendClasspath
     */
    export function append(path: string | string[]): void {
        appendClasspath(path);
    }

    /**
     * Get the loaded jars in the class path
     *
     * @returns a list of the loaded jars
     */
    export function get(): string[] {
        ensureJvm();
        return javaInstance!.loadedJars;
    }
}

/**
 * A callback for any output redirected from stdout/stderr from the java process.
 *
 * @param err an error if the conversion of the output failed.
 *            This is null if the output was valid. This will probably never be set.
 * @param data the data that was converted. This is unset if <code>err</code> is set.
 */
export type StdoutCallback = (err: Error | null, data?: string) => void;

/**
 * The class guarding the stdout redirect.
 * Keep this instance in scope to not lose the redirect.
 * As soon as this gets garbage collected, the redirection
 * of the stdout/stderr will be stopped. Only one instance
 * of this can exist at a time. Call {@link reset} to stop
 * redirecting the program output and release this class
 * instance early.
 *
 * This can be created by calling {@link stdout.enableRedirect}.
 *
 * ## Example
 * ```ts
 * import { stdout } from 'java-bridge';
 *
 * const guard = stdout.enableRedirect((_, data) => {
 *     console.log('Stdout:', data);
 * }, (_, data) => {
 *     console.error('Stderr:', data);
 * });
 *
 * // Change the receiver method
 * guard.on('stderr', (_, data) => {
 *     console.warn('Stderr:', data);
 * });
 *
 * // Disable a receiver
 * guard.on('stdout', null);
 *
 * // Disable stdout redirect
 * guard.reset();
 * ```
 *
 * ## See also
 * * {@link stdout.enableRedirect}
 */
export interface StdoutRedirectGuard {
    /**
     * Set the stdout/stderr event handler.
     * Pass <code>null</code> to disable this specific handler.
     * Only accepts 'stdout' and 'stderr' as the <code>event</code>
     * argument. Overwrites the previous handler.
     *
     * @param event the event to listen on
     * @param callback the callback
     */
    on(event: 'stdout' | 'stderr', callback: StdoutCallback | null): void;

    /**
     * Reset this <code>StdoutRedirectGuard</code> instance.
     * After this call, the stdout/stderr will no longer
     * be redirected to the specified methods and any call
     * to this class will throw an error as this counts as destroyed.
     */
    reset(): void;
}

/**
 * A namespace containing methods for redirecting the stdout/stderr of the java process.
 *
 * ## See also
 * * {@link StdoutRedirectGuard}
 * * {@link stdout.enableRedirect}
 */
export namespace stdout {
    /**
     * Enable stdout/stderr redirection.
     *
     * Pass methods for the stdout and stderr output to be redirected to.
     * These methods must accept an error as the first argument,
     * although this will probably never be set and can be ignored.
     * The second argument is the data that was redirected.
     *
     * Setting any method to ``null`` or ``undefined`` will disable the redirect for that method.
     * This also allows you not set any handler which does not make any sense at all.
     *
     * ## Examples
     * ### Redirect all data to the js console
     * ```ts
     * import { stdout } from 'java-bridge';
     *
     * const guard = stdout.enableRedirect((_, data) => {
     *     console.log('Stdout:', data);
     * }, (_, data) => {
     *     console.error('Stderr:', data);
     * });
     * ```
     *
     * ### Redirect stdout to the js console
     * ```ts
     * const guard = stdout.enableRedirect((_, data) => {
     *     console.log('Stdout:', data);
     * });
     * ```
     *
     * ### Redirect stderr to the js console
     * ```ts
     * const guard = stdout.enableRedirect(null, (_, data) => {
     *    console.error('Stderr:', data);
     * });
     * ```
     *
     * ### Redirect nothing to the js console (y tho)
     * This enables you to print nothing to nowhere.
     * ```ts
     * // Why would you do this?
     * const guard = stdout.enableRedirect(null, null);
     *
     * // Or
     * const guard = stdout.enableRedirect();
     * ```
     *
     * @see StdoutRedirectGuard
     * @see StdoutCallback
     * @param stdout the callback to be called when stdout is received
     * @param stderr the callback to be called when stderr is received
     * @returns a <code>StdoutRedirectGuard</code> instance. Keep this instance in scope to not lose the redirect.
     */
    export function enableRedirect(
        stdout?: StdoutCallback | null,
        stderr?: StdoutCallback | null
    ): StdoutRedirectGuard {
        ensureJvm();
        return javaInstance!.setStdoutCallbacks(stdout, stderr);
    }
}

/**
 * The class for implementing java interfaces.
 * Keep this instance in scope to not destroy the java object.
 * Call {@link reset} to instantly destroy this instance.
 *
 * ## Notes
 * Keeping this instance alive may cause your process not to exit
 * early. Thus, you must wait for the javascript garbage collector
 * to destroy this instance even if you called {@link reset}.
 *
 * Once this instance has been destroyed, either by calling {@link reset}
 * or the garbage collector, any call to any method defined earlier
 * by {@link newProxy} will throw an error in the java process.
 *
 * ## Example
 * ```ts
 * import { newProxy } from 'java-bridge';
 *
 * const proxy = newProxy('path.to.MyInterface', {
 *     // Define methods...
 * });
 *
 * // Do something with the proxy
 * instance.someMethod(proxy);
 *
 * // Destroy the proxy
 * proxy.reset();
 * ```
 *
 * ## See also
 * * {@link newProxy}
 */
export interface JavaInterfaceProxy {
    /**
     * Destroy the proxy class.
     * After this call any call to any method defined by the
     * interface will throw an error on the java side. This error
     * may be thrown back to the node process, if you are not
     * specifically implementing methods that will be called
     * from another (java) thread.
     * Throws an error if the proxy has already been destroyed.
     */
    reset(): void;
}

/**
 * An interface proxy method.
 * Any arguments passed to this method are values converted from java values.
 * The return value will be converted back to a java type.
 *
 * @param args the arguments passed from the java process
 * @return the value to pass back to the java process
 */
export type ProxyMethod = (...args: any[]) => any;
type InternalProxyRecord = Parameters<
    typeof Java.prototype.createInterfaceProxy
>[1];

/**
 * Create a new java interface proxy.
 * This allows you to implement java interfaces in javascript.
 *
 * Pass an object as the second argument with the names of the
 * methods you want to implement as keys and the implementations
 * as values in order to expose these methods to the java process.
 * Any arguments will be converted to javascript values and
 * return values will be converted to java values.
 *
 * When the java process tries to call any method which is
 * not implemented by the proxy, an error will be thrown.
 *
 * ## Examples
 * ### Implement ``java.lang.Runnable``
 * ```ts
 * import { newProxy, importClass } from 'java-bridge';
 *
 * // Define the interface
 * const runnable = newProxy('java.lang.Runnable', {
 *     run: (): void => {
 *         console.log('Hello World!');
 *     }
 * });
 *
 * // Note: You can't do something like this:
 * // runnable.run();
 *
 * // Pass the proxy to a java method instead:
 * const Thread = importClass('java.lang.Thread');
 * const thread = new Thread(runnable); // <- Pass the proxy here
 *
 * // NOTE: You don't have to call this asynchronously
 * // as this call instantly returns.
 * thread.startSync();
 * ```
 *
 * ### Implement ``java.util.function.Function`` to transform a string
 * ```ts
 * const func = newProxy('java.util.function.Function', {
 *     // Any parameters and return types will be automatically converted
 *     apply: (str: string): string => {
 *         return str.toUpperCase();
 *     }
 * });
 *
 * // Import the string class
 * const JString = java.importClass('java.lang.String');
 * const str = new JString('hello');
 *
 * // Pass the proxy.
 * // NOTE: You must call this method async otherwise your program will hang.
 * // See notes for more info.
 * const transformed = await str.transform(func);
 *
 * assert.assertEquals(transformed, 'HELLO');
 * ```
 *
 * Which is equivalent to the following java code:
 * ```java
 * Function<String, String> func = new Function<>() {
 *     @Override
 *     public String apply(String str) {
 *         return str.toUpperCase();
 *     }
 * };
 *
 * String str = "hello";
 * String transformed = str.transform(func);
 * assert.assertEquals(transformed, "HELLO");
 * ```
 *
 * #### Throwing exceptions
 * Any exceptions thrown by the proxy will be converted to java exceptions
 * and then rethrown in the java process. This may cause the exception
 * to again be rethrown in the javascript process.
 * ```ts
 * const func = newProxy('java.util.function.Function', {
 *     apply: (str: string): string => {
 *         throw new Error('Something went wrong');
 *     }
 * });
 *
 * const JString = java.importClass('java.lang.String');
 * const str = new JString('hello');
 *
 * // This will re-throw the above error
 * const transformed: never = await str.transform(func);
 * ```
 *
 * ## Notes
 * * Keep this instance in scope to not destroy the interface proxy.
 * * Call {@link JavaInterfaceProxy.reset} to instantly destroy this instance.
 * * If any method is queried by the java process and not implemented in here,
 *   an exception will be thrown in the java process.
 * * Any errors thrown in the javascript process will be rethrown in the java process.
 * * **When calling a java method that uses an interface defined by this, you must call
 *   that method using the interface asynchronously as Node.js is single threaded and can't
 *   wait for the java method to return while calling the proxy method at the same time.**
 *
 * ## See also
 * * {@link JavaInterfaceProxy}
 *
 * @param interfaceName the name of the java interface to implement
 * @param methods the methods to implement.
 * @returns a proxy class to pass back to the java process
 */
export function newProxy(
    interfaceName: string,
    methods: Record<string, ProxyMethod>
): JavaInterfaceProxy {
    ensureJvm();
    const proxyMethods: InternalProxyRecord = Object.create(null);

    for (const [name, method] of Object.entries(methods)) {
        proxyMethods[name] = (
            err: null | Error,
            callback: (err: Error | null, data?: any | null) => void,
            ...args: any[]
        ): void => {
            if (err) {
                throw err;
            }

            try {
                const res = method(...args);
                callback(null, res);
            } catch (e: any) {
                if (e instanceof Error) {
                    callback(e);
                } else {
                    callback(new Error(e.toString()));
                }
            }
        };
    }

    return javaInstance!.createInterfaceProxy(
        interfaceName,
        proxyMethods
    ) as JavaInterfaceProxy;
}

/**
 * Get the static java instance.
 * This has no real use, all important methods are exported explicitly.
 */
export function getJavaInstance(): Java | null {
    return javaInstance;
}
