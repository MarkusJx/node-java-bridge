# node-java-bridge

A bridge between Node.js programs and Java APIs.

The goal was to create a tool similar to [@joeferner/node-java](https://github.com/joeferner/node-java) since that API
seems to be somewhat inactive.

**This project is still under development, thus, bugs may occur.** Feel free to submit them.

Key differences from [@joeferner/node-java](https://github.com/joeferner/node-java):

-   The java library is loaded dynamically, thus, the compiled binary is not bound to the specific jdk/jre it was compiled
    with, it can use basically any vm as long as the function definitions are equal to the ones from the jdk the binary
    was compiled with. See [notes](#library-paths) for further information.
-   Asynchronous function calls return actual Promises, rather than relying on callbacks
-   This module is actually context-aware (or at least, it should be), making it more compatible with electron

## Installation

```shell
npm i @markusjx/java
```

### Build requirements

-   npm
-   Git
-   CMake
-   Ninja or Make on Unix-based systems
-   Some sort of [cmake-js](https://www.npmjs.com/package/cmake-js) compatible C++ compiler. May be one of:
    -   [Visual C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
    -   Clang or GCC

# Getting started

```js
const java = require('@markusjx/java');
// or
import java from '@markusjx/java';
```

## Managing the jvm

### Creating a new jvm instance

In order to create a Java Virtual Machine instance, you may call `java.ensureJVM()`. This will create a jvm instance
if it doesn't already exist. Calling this isn't required as it is called on every call to any jvm functions.

Example: Creating a jvm with version 1.8 and the default path to the java library

```js
java.ensureJVM(null, java.java_version.VER_1_8);
```

Alternatively, you may want to create a new java instance using `java.createJVM()`. This will destroy the old instance
before creating the new one.

```js
// Someone requested number 10?
java.createJVM(null, java.java_version.VER_10);
```

If you want to find a valid jvm on your machine, you may want to use:

```js
// Find the shared library on this machine
const libPath = await java.findJVM();

// This can be passed directly into ensureJVM() or createJVM():
java.createJVM(libPath, java.java_version.VER_1_8);
```

### Deleting the jvm instance

If you want to delete the current jvm instance, you may want to call `java.destroyJVM()`. The jvm will be destroyed
and all ongoing function calls may throw exceptions as the vm becomes invalid.

The behaviour is not actually _that well-defined_ as the jvm does wait on attached threads to become detached,
therefore, async calls may still succeed, whereas sync calls will likely fail
(if there are no async operations currently running). Though the question still remains why anyone would want to destroy
the vm mid-call.

```js
java.destroyJVM();
```

**General Notes**:

-   There may only exist one jvm per process as the jni documentation states:

> As of JDK/JRE 1.2, creation of multiple VMs in a single process is not supported.

-   The jvm may only be created and destroyed in the main thread (which shouldn't be that hard to achieve)

## Examples

### Import a java class

This imports the java String class:

```js
const JString = java.importClass('java.lang.String');
// or, async version
const JString_async = await java.importClassAsync('java.lang.String');
```

### Create a new class instance

Create a new `java.lang.String` class instance. This is never required as javascript strings are always converted to
java strings and the other way around. Anyways, here it is just for the hell of it:

```js
let str = new JString('some string which will be converted to a java string');
// Async version
str = await JString.createInstance('some string');

// Get the string
let jsString = str.toStringSync();
// or, async
jsString = await str.toString();
```

#### Calling static methods

As JavaScript doesn't permit static methods to be called on class instances, you should always call static methods
directly on the imported class:

```js
// Call static java.lang.String.valueOf(char[])
// Note that javas char translates to a js string containign a single character
let jsString = JString.valueOf(['s', 'o', 'm', 'e', ' ', 's', 't', 'r', 'i', 'n', 'g']);
```

### Lists

When trying to create lists and when using typescript, you may want to:

```ts
declare class List<T extends java_type> extends JavaClassInstance {
    size(): Promise<number>;

    sizeSync(): number;

    add(data: T): Promise<void>;

    addSync(data: T): void;

    get(index: number): Promise<T>;

    getSync(index: number): T;

    toArray(): Promise<T[]>;

    toArraySync(): T[];

    isEmpty(): Promise<boolean>;

    isEmptySync(): boolean;

    contains(value: T): Promise<boolean>;

    containsSync(value: T): boolean;

    clear(): Promise<void>;

    clearSync(): void;

    lastIndexOf(value: T): Promise<number>;

    lastIndexOfSync(value: T): number;

    remove(index: number): Promise<T>;

    removeSync(index: number): T;
}
```

#### Create a new ArrayList

Import the java class:

```ts
const ArrayList = java.importClass('java.util.ArrayList') as typeof List;
```

Create a new ArrayList instance:

```ts
const list: List<number> = new ArrayList();
// Async version
const async_list: List<number> = await ArrayList.createInstance();
```

You may then want to start adding values:

```js
list.addSync(1234);

// Will print out '1'
console.log(list.sizeSync());

// Will print out '1234'
console.log(list.getSync(0));
```

### Floats

Float values are not supported to be passed directly to methods expecting an object (template classes), e.g. Lists,
those values must be wrapped using the `Float` class:

```js
const Float = java.importClass('java.lang.Float');

// Create the actual float value.
// This can be passed to any method accepting objects.
const value = new Float(12.233213);
```

See the [notes on value conversions](#value-conversions) for further information about values.

# API definition

## ensureJVM

Create the jvm instance if it does not exist.

If the vm already exists, nothing is done, if it doesn't it will be created. Calling this is optional as the jvm will be
created with every call to it if it doesn't already exist.

See [notes](#library-paths) for further information on the jvmPath argument.

See [notes](#jvm-versions) on jvm versions for further information on the version argument.

**Arguments**

-   `jvmPath?: string | null` - The path to the jvm.(so|dylib|dll). Must include the library name.
-   `version?: java_version | string | null` - The version of the jvm to use. May be something
    from [java_version](#enum-java_version).

**Returns**

`void`

## createJVM

Create the jvm instance. This will destroy the old instance and make all calls to it invalid. You may don't want to do
this as the create call may fail.

Calling this is optional as the jvm will be created with every call to it if it doesn't already exist.
See [notes](#library-paths) for further information on the jvmPath argument.

See [notes](#jvm-versions) on jvm versions for further information on the version argument.

**Arguments**

-   `jvmPath?: string | null` - The path to the jvm.(so|dylib|dll). Must include the library name.
-   `version?: java_version | string | null` - The version of the jvm to use. May be something
    from [java_version](#enum-java_version).

**Returns**

`void`

## destroyJVM

Destroy the jvm instance and invalidate all calls to the vm. You may not want to call this unless you are planning to
not use the jvm again any time soon.

**Arguments**

none

**Returns**

`void`

## enum java_version

An enum containing all supported java version, which is defined as:

```ts
enum java_version {
    VER_1_1 = '1.1',
    VER_1_2 = '1.2',
    VER_1_4 = '1.4',
    VER_1_6 = '1.6',
    VER_1_8 = '1.8',
    VER_9 = '9',
    VER_10 = '10',
}
```

For what versions are actually available view the notes on [JVM versions](#jvm-versions).

## classpath

Methods for altering the class path.

### append

Append a single or multiple jar file(s) to the class path. The path should be a full path to the file.

This won't actually modify the class path, for further information, take a look at the definition
in [index.d.ts](index.d.ts) and
[jni_wrapper.cpp](src/jvm_lib/jni_wrapper.cpp).

**Arguments**

-   `path: string | string[]` - The path(s) of the file(s) to append

**Returns**

`void`

### appendAsync

Append a single or multiple jar file(s) to the class path. The same as `append` but it's an async call.

**Arguments**

-   `path: string | string[]` - The path(s) of the file(s) to append

**Returns**

`Promise<void>`

## importClass

Import a java class.

Since the class members and their signatures are resolved on import, this call may take some time. To use your time more
efficiently, consider using the async version of this function. If a class of the same type is already imported
(and not destroyed/garbage-collected), the class is retrieved from a cache. Therefore, no members need to be resolved,
reducing the import time. The class is cached with the first import of the class and destroyed when the
last `java_class_proxy` instance, therefore the last imported instance, is garbage-collected.

**Arguments**

-   `classname: string` - The class to import

**Returns**

`JavaConstructor` - The constructor function of the class

## importClassAsync

Import a java class. Async version.

The async call may be required in some cases as the class members and functions plus their signatures are resolved on
import which may take a while. Optimizations apply.

**Arguments**

-   `classname: string` - The class to import

**Returns**

`Promise<JavaConstructor>` - The constructor function of the class wrapped in a Promise

## getJavaInstance

Get the `java_instance` instance. This class is the main instance which creates and destroys the jvm instances, plus
it contains the definitions for many methods exported. Those shouldn't be called using the instance returned by this
method, though.

**Arguments**

none

**Returns**

`java_instance`

## newProxy

Create a proxy class for implementing java interfaces in javascript. Be aware that any methods called from java will be
run in the main thread as v8 doesn't really support multithreading.

**Arguments**

-   `name: string` - The name of the interface to implement, e.g. `java.lang.Runnable`.
-   `functions: object` - An object containing all methods to be overridden, must contain the name of the method as a
    key and the implementation as a value.

**Returns**

`JavaInterfaceProxy` - The created proxy

**Notes**

If you want to destroy the interface proxy, you can use the `destroy` method on the proxy.
Also, it may take a while for your program to exit (or until the garbage collector has been triggered, to be precise)
as the proxy is kept alive until the last instance of the proxy is garbage-collected. So it may be a good idea to
set you proxy instance to null after you are done with it.

When using a proxy, you must call any method using the proxy async, as calling it sync will cause the program to
deadlock, as node.js is single-threaded and your only thread will be waiting on the java method to finish, which will
prevent this thread from running the defined javascript proxy methods.

## Redirecting the program's `stdout`

### stdoutRedirect.enableRedirect

Enable redirecting the program's `stdout`/`stderr`
to a custom method.

**Arguments**

-   `stdout: (line: string) => void` - The custom stdout callback
-   `stderr: (line: string) => void` - The custom stderr callback

**Returns**

`void`

## logging

A namespace for managing logging output

### setLogLevel

Set the log level for the java module

**Arguments**

-   `level: LogLevel | number` - The log level to set

**Returns**

`void`

### LogLevel

The supported log levels to set, which is defined as:

```ts
enum LogLevel {
    // Show debug, warning and error messages
    DEBUG = 0,
    // Show warning and error messages
    WARNING = 1,
    // Only show error messages
    ERROR = 2,
    // Log nothing at all
    NONE = 3,
}
```

## Notes

### Library paths

The module requires a jvm (which may be either a JDK or JRE)
to be installed on the target machine and the java shared library must be found at runtime as it will be loaded
dynamically by the module.

Furthermore, a JDK needs to be installed and found at compile time as the headers are used for type definitions, but the
native module is not linked against any libraries to not bind it to a specific java implementation. The JDK/JRE is found
using [find-java-home](https://github.com/jsdevel/node-find-java-home), read the documentation, to see how it finds the
JDK/JRE. At runtime, the shared library is loaded dynamically.

The path to the `jvm.(so|dylib|dll)` is determined on
`install` and stored in `module_path/jvmLibPath.json`. If you want to use another jvm than the pre-determined one,
you may want to pass a path to `java.ensureJVM` or `java.createJVM`
before calling any java functions. If no path (or `null`)
is passed, the default path will be used.

You may want to pass a path like this:

```js
// A jvm.dll path on a windows system
const jvm_path = 'C:\\Program Files\\AdoptOpenJDK\\jdk-11.0.10.9-hotspot\\bin\\client\\jvm.dll';

// The version argument is optional
java.ensureJVM(jvm_path);

// The same works for
java.createJVM(jvm_path);
```

Again, passing no path will cause the module to use the path provided by `module_path/jvmLibPath.json`:

```js
// Use the default path
java.ensureJVM(null);

// The first argument is optional,
// so you could just write:
java.ensureJVM();
```

#### Notes on the independence of the module regarding the JDK/JRE

> If the module was compiled using the x64 architecture (64bit),
> it is impossible to load a x86 (32bit) jvm shared library.
> The same applies for native modules compiled as a 32 bit library.
> Therefore, it is important to always match the target architecture
> for both @markusjx/java, the JDK it is compiled with and the JDK/JRE
> it will use to create virtual machines.
>
> This also implies that the architecture of the jni headers
> must match the target architecture of the module, as any
> defined types and their lengths may not match the required lengths.

### JVM versions

If no version argument (or null) is passed to either `java.ensureJVM`
or `java.createJVM`, a default version will be chosen, which is one of `1.6` or `1.8`. Version `1.8` is chosen
if the binary was compiled with a jvm supporting version `1.8`, if it does not, version
`1.6` is chosen.

If neither version was defined at compile time, version `1.1` is chosen.

In general, if a version wasn't available at compile time (the found jvm, defined by `JAVA_HOME` had a lower version
than that version, whatever it may be), this version won't be available until the module is compiled with never versions
of the jvm library headers, which define this version.

In general, compiling the module with newer versions of the headers and running it with any older java version may lead
to unexpected results and requesting a newer version will fail (obviously).

### Value conversions

1. Any basic value such as `string`, `number`, `boolean` or `BigInt` may be passed to methods accepting matching
   types
2. `string` values will always be converted to `java.lang.String`
3. `string` values with just one character may be converted to `char` or `java.lang.Char` if required
4. Thus, in order to pass a `char` to a java method, use a `string` containing just one character
5. `number` values will be converted to `int`, `long`, `double`, `float`, `java.lang.Integer`,
   `java.lang.Long`, `java.lang.Double` or `java.lang.Float` depending on the type the java function to call requires
6. `boolean` values will be converted to either `boolean` or `java.lang.Boolean`
7. `BigInt` values will be converted to either `long` or `java.lang.Long`
8. Arrays will be converted to java arrays. Java arrays may only contain a single value type, therefore the type of
   the first element in the array will be chosen as the array type, empty arrays need no conversions.
9. `java.lang.String` values will be converted to `string`
10. `int`, `double`, `float`, `java.lang.Integer`, `java.lang.Double` or `java.lang.Float`
    values will be converted to `number`
11. `long` or `java.lang.Long` values will always be converted to `BigInt`
12. `boolean` or `java.lang.Boolean` values will be converted to `boolean`
13. `char` or `java.lang.Character` values will be converted to `string`
14. Java arrays will be converted to javascript arrays, applying the rules mentioned above

Java objects are stored in an instance of the `java_instance_proxy` class, any of those
instances may be passed to any method accepting the object stored in that instance (and its type).
Those objects usually won't be converted to any native javascript objects. Exceptions: Methods returning
convertible types, such as `java.lang.String.toString()` which returns itself, but the string value will be
converted to a javascript `string`.
