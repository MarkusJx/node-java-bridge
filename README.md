# node-java

[![Test](https://github.com/MarkusJx/node-java-bridge/actions/workflows/test.yml/badge.svg)](https://github.com/MarkusJx/node-java-bridge/actions/workflows/test.yml)
[![Check-style](https://github.com/MarkusJx/node-java-bridge/actions/workflows/check-style.yml/badge.svg)](https://github.com/MarkusJx/node-java-bridge/actions/workflows/check-style.yml)
[![SystemTest](https://github.com/MarkusJx/node-java-bridge/actions/workflows/system_test.yml/badge.svg)](https://github.com/MarkusJx/node-java-bridge/actions/workflows/system_test.yml)

A bridge between Node.js programs and Java APIs written in Rust using [napi-rs](https://napi.rs/)
to provide a fast and memory-safe interface between the two languages.

The pre-compiled binaries will be provided with the package, the only thing
you need to do on your machine is install a Java Runtime Environment (JRE)
for this package to use. In contrast to other `node.js <-> java` interfaces,
the binary is not hard linked to the JDK it has been compiled with but rather
loads the jvm native library dynamically when the program first starts up.

The full documentation of this package is available [here](https://markusjx.github.io/node-java-bridge/).

**NOTE: As of version `2.1.0`, this package has been renamed from `@markusjx/java` to `java-bridge`.**

## Installation

```shell
npm i java-bridge
```

## Build instructions

In order to build this project, you should install

-   Node.js
-   npm
-   rustc, the rust compiler
-   cargo
-   Java JDK 8+
-   clang

Then, to build the project, run:

```sh
npm install
npm run build
```

## Support Matrix

> _✅ = Pre-compiled binaries are available_<br> _`-` = Pre-compiled binaries are not available_

| Operating System | i686 | x64 | arm | arm64 |
| ---------------- | ---- | --- | --- | ----- |
| Linux            | -    | ✅  | -   | ✅    |
| Windows          | ✅   | ✅  | -   | -     |
| macOS            | -    | ✅  | -   | ✅    |

## Usage

### Example: Hello world from Java

```ts
import { importClass } from './java-bridge';

const System = importClass('java.lang.System');
System.out.println('Hello world!');
```

### Create the JVM

Create a new Java VM using the [`ensureJvm`](https://markusjx.github.io/node-java-bridge/functions/ensureJvm.html) method.
Calling this after the jvm has already been created will do nothing.
Destroying the jvm manually is not (yet) supported.

#### Create the JVM with no extra options

This will first search for a suitable `jvm` native library on the system and then
start the jvm with no extra options. This is also called when any call to the jvm is made
but the jvm is not yet started.

```ts
import { ensureJvm } from 'java-bridge';

ensureJvm();
```

#### Create the JVM with extra options

You can pass extra options to the jvm when creating it, for example requesting a specific jvm version,
specifying the location of the jvm native library or passing additional arguments to the jvm.

```ts
import { ensureJvm, JavaVersion } from 'java-bridge';

ensureJvm({
    libPath: 'path/to/jvm.dll',
    version: JavaVersion.VER_9,
    opts: '-Xms512m -Xmx512m',
});
```

#### Use daemon threads

By default, new threads will **not** be attached as daemon threads, meaning any thread attached to the
jvm will be automatically detached once it is not required anymore. This slows down asynchronous calls
but will prevent the jvm from being terminated if any thread is still running.

If you want to change this behaviour, you can pass the `useDaemonThreads` option to the `ensureJvm` function.
This will make the jvm attach threads as daemon threads causing those threads to not be detached
once not required anymore.

```ts
ensureJvm({
    useDaemonThreads: true,
});
```

### Inject a JAR into the class path

In order to import your own classes into the node environment, you need
to add the JAR file to the class path. You can do that with the
[`appendClasspath`](https://markusjx.github.io/node-java-bridge/functions/appendClasspath.html)
or [`classpath.append`](https://markusjx.github.io/node-java-bridge/functions/classpath.append.html)
methods. After loading a JAR, you can import classes from it like any other class
from the JVM using [`importClass`](#synchronous-calls) or [`importClassAsync`](#asynchronous-calls).

```ts
import { appendClasspath } from 'java-bridge';

// Append a single jar to the class path
appendClasspath('/path/to/jar.jar');

// Append multiple jars to the class path
appendClasspath(['/path/to/jar1.jar', '/path/to/jar2.jar']);
```

or

```ts
import { classpath } from 'java-bridge';

// Append a single jar to the class path
classpath.append('/path/to/jar.jar');
```

### Synchronous calls

If you want to use Java APIs in a synchronous way, you can use the synchronous API of this module.
Any call to the Java API will be executed in the same thread as your node process so this
may cause your program to hang until the execution is finished. But - in contrast to the asynchronous API -
these calls are a lot faster as no extra threads need to be created/attached to the JVM.

All synchronous java methods are proceeded with the postfix `Sync`.
This means, all methods of a class (static and non-static) are generated twice,
once as a synchronous call and once as an asynchronous call.

If you are looking for asynchronous calls, take a look at the next section.
In order to import a class synchronously, you can use the [`importClass`](https://markusjx.github.io/node-java-bridge/functions/importClass.html) function.
Using this method does not affect your ability to call any method of the class asynchronously.

```ts
import { importClass } from 'java-bridge';

// Import a class
const JString = importClass('java.lang.String');

// Create a new instance of the class
const str = new JString('Hello World');

// Call a method on the instance
str.lengthSync(); // 11

// Supported native types will be automatically converted
// to the corresponding type in the other language
str.toStringSync(); // 'Hello World'
```

### Asynchronous calls

If you want to use Java APIs in an asynchronous way, you can use the asynchronous API of this module.
Any call to the Java API will be executed in a separate thread and the execution will not block your program.
This is in general a lot slower as the synchronous API but allows the program to run more smoothly.

If you want to improve the performance of the asynchronous API, you can force the module to attach
any thread as a daemon thread to the JVM. This allows the program to not constantly attach new threads
to the JVM as the old ones can be reused and thus improves the performance.

In order to import a class asynchronously, you can use the
[`importClassAsync`](https://markusjx.github.io/node-java-bridge/functions/importClassAsync.html) function.

```ts
import { importClassAsync } from 'java-bridge';

const JString = await importClassAsync('java.lang.String');

// Create a new instance asynchrnously using 'newInstanceAsync'
const str = await JString.newInstanceAsync('Hello World');

// Call methods asynchronously
await str.length(); // 11
await str.toString(); // 'Hello World'
```

### Implement a Java interface

You can also implement a Java interface in node.js using the
[`newProxy`](https://markusjx.github.io/node-java-bridge/functions/newProxy.html) method.
Please note that when calling a java method that uses an interface defined by this method,
you must call that method using the interface asynchronously as Node.js is single threaded
and can't wait for the java method to return while calling the proxy method at the same time.

```ts
import { newProxy } from 'java-bridge';

const proxy = newProxy('path.to.MyInterface', {
    // Define methods...
});

// Do something with the proxy
instance.someMethod(proxy);

// Destroy the proxy
proxy.reset();
```

### Redirect the stdout and stderr from the java process

If you want to redirect the stdout and/or stderr from the java
process to the node.js process, you can use the
[`enableRedirect`](https://markusjx.github.io/node-java-bridge/functions/stdout.enableRedirect.html)
method.

```ts
import { stdout } from 'java-bridge';

const guard = stdout.enableRedirect(
    (_, data) => {
        console.log('Stdout:', data);
    },
    (_, data) => {
        console.error('Stderr:', data);
    }
);
```

## Value conversion rules

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
14. Java arrays will be converted to javascript arrays, applying the rules mentioned above except
15. Byte arrays will be converted to `Buffer` and vice-versa

## Command line interface

This module also ships with a command line interface called `java-ts-gen` for creating typescript definitions for Java classes.
The command line interface will create typescript definitions for all specified classes and their dependencies.

### Installation

```bash
npm install -g java-bridge
```

### Usage

```
java-ts-gen <output> <classnames..>

Positionals:
  classnames  The fully qualified class name(s) to convert              [string]
  output      The output file                                           [string]

Options:
  --help             Show help                                         [boolean]
  --version          Show version number                               [boolean]
  --classpath, --cp  The classpath to use                               [string]
```

### Notes

-   The classpath argument can be supplied multiple times to add multiple jars to the classpath
-   Multiple class names can be supplied to generate definitions for multiple classes
-   The generated typescript files will automatically import all classes once the module is loaded.

### Examples

#### Generate definitions for a single class

Generate definitions for the `java.lang.String` class and all its referenced classes and save them to `./project`:

```bash
java-ts-gen ./project java.lang.String
```

This will create a directory called `java` containing the definitions for the `java.lang.String` class and all its
dependencies all inside subdirectories. The `java.lang.String` class will be saved to `./project/java/lang/String.ts`.
Thus, the folder structure of `project` will look something like this:

```
.
├── ...
├── java
│   ├── lang
│   │   ├── String.ts
│   │   ├── Object.ts
│   │   └── ...
│   ├── util
│   │   └── ...
│   └── ...
└── ...
```

#### Generate definitions for multiple classes

Generate definitions for the `java.lang.String` and `java.util.ArrayList` classes and all of their dependencies
and save them to `./project`:

```bash
java-ts-gen ./project java.lang.String java.util.ArrayList
```
