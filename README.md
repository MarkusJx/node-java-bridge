# java-bridge

[![Test](https://github.com/MarkusJx/node-java-bridge/actions/workflows/test.yml/badge.svg)](https://github.com/MarkusJx/node-java-bridge/actions/workflows/test.yml)
[![Check-style](https://github.com/MarkusJx/node-java-bridge/actions/workflows/check-style.yml/badge.svg)](https://github.com/MarkusJx/node-java-bridge/actions/workflows/check-style.yml)

<!--[![SystemTest](https://github.com/MarkusJx/node-java-bridge/actions/workflows/system_test.yml/badge.svg)](https://github.com/MarkusJx/node-java-bridge/actions/workflows/system_test.yml)-->

A bridge between Node.js programs and Java APIs written in Rust
using [napi-rs](https://napi.rs/)
to provide a fast and memory-safe interface between the two languages.

The pre-compiled binaries will be provided with the package, the only thing
you need to do on your machine is install a Java Runtime Environment (JRE)
for this package to use. In contrast to other `node.js <-> java` interfaces,
the binary is not hard linked to the JDK it has been compiled with but rather
loads the jvm native library dynamically when the program first starts up.

The full documentation of this package is
available [here](https://markusjx.github.io/node-java-bridge/).

**NOTE: As of version `2.1.0`, this package has been renamed from `@markusjx/java`
to `java-bridge`.**

## Installation

```shell
npm i java-bridge
```

_Note: In order to use this package on windows, you'll need to install
the [Visual C++ Redistributable for Visual Studio 2015](https://www.microsoft.com/en-gb/download/details.aspx?id=48145)._

## Command line interface

This module also provides a command line interface that allows you to generate typescript
definitions for your java classes.
The command line interface is called `java-ts-definition-generator` and can be installed
using `npm install -g java-ts-definition-generator`.
The full documentation can be
found [here](https://github.com/MarkusJx/java-ts-definition-generator).

## Build instructions

_This is only required for development purposes. When installing the package
using `npm i`, you can skip this._

In order to build this project, you should install

- Node.js
- npm
- rustc, the rust compiler
- cargo
- Java JDK 8+
- clang

Then, to build the project, run:

```sh
npm install
npm run build
```

## Support Matrix

> _✅ = Pre-compiled binaries are available_<br> _`-` = Pre-compiled binaries are not
> available_

| Operating System | i686 | x64 | arm | arm64 |
| ---------------- | :--: | :-: | :-: | :---: |
| Linux            |  -   | ✅  |  -  |  ✅   |
| Windows          |  ✅  | ✅  |  -  |   -   |
| macOS            |  -   | ✅  |  -  |  ✅   |

### Known working linux distros

| Distro |    Version    |
| :----: | :-----------: |
| Ubuntu |  `>= 20.04`   |
| Debian | `>= bullseye` |

## Usage

### Example: Hello world from Java

```ts
import { importClass } from './java-bridge';

const System = importClass('java.lang.System');
System.out.println('Hello world!');
```

### Create the JVM

Create a new Java VM using
the [`ensureJvm`](https://markusjx.github.io/node-java-bridge/functions/ensureJvm.html)
method.
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

You can pass extra options to the jvm when creating it, for example requesting a specific
jvm version,
specifying the location of the jvm native library or passing additional arguments to the
jvm.

```ts
import { ensureJvm, JavaVersion } from 'java-bridge';

ensureJvm({
    libPath: 'path/to/jvm.dll',
    version: JavaVersion.VER_9,
    opts: ['-Xms512m', '-Xmx512m'],
});
```

All threads will be attached as daemon threads, allowing the jvm to exit when the main
thread exits.
This behaviour can not be changed, as it may introduce undefined behaviour.

Important note on jvm options: Different arguments must be parsed as separate strings in
the `opts` array.
Otherwise, the jvm will not be able to parse the arguments correctly.

#### Notes on electron

When using this package in a packaged electron application, you should unpack this package
and
the appropriate binaries for your platform into the `app.asar.unpacked` folder. When using
electron-builder, you can do this by adding the following to your `package.json`:

```json
{
    "build": {
        "asarUnpack": [
            "node_modules/java-bridge/**",
            "node_modules/java-bridge-*/**"
        ]
    }
}
```

Additionally, you should set the `isPackagedElectron` option to `true` when creating the
jvm:

```ts
ensureJvm({
    isPackagedElectron: true,
});
```

This option _should_ not have any effect when not using electron or not having the
application packaged.

### Inject a JAR into the class path

In order to import your own classes into the node environment, you need
to add the JAR file to the class path. You can do that with the
[`appendClasspath`](https://markusjx.github.io/node-java-bridge/functions/appendClasspath.html)
or [`classpath.append`](https://markusjx.github.io/node-java-bridge/functions/classpath.append.html)
methods. After loading a JAR, you can import classes from it like any other class
from the JVM using [`importClass`](#synchronous-calls)
or [`importClassAsync`](#asynchronous-calls).

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

If you want to use Java APIs in a synchronous way, you can use the synchronous API of this
module.
Any call to the Java API will be executed in the same thread as your node process so this
may cause your program to hang until the execution is finished. But - in contrast to the
asynchronous API -
these calls are a lot faster as no extra threads need to be created/attached to the JVM.

All synchronous java methods are proceeded with the postfix `Sync`.
This means, all methods of a class (static and non-static) are generated twice,
once as a synchronous call and once as an asynchronous call.

If you are looking for asynchronous calls, take a look at the next section.
In order to import a class synchronously, you can use
the [`importClass`](https://markusjx.github.io/node-java-bridge/functions/importClass.html)
function.
Using this method does not affect your ability to call any method of the class
asynchronously.

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

If you want to use Java APIs in an asynchronous way, you can use the asynchronous API of
this module.
Any call to the Java API will be executed in a separate thread and the execution will not
block your program.
This is in general a lot slower as the synchronous API but allows the program to run more
smoothly.

If you want to improve the performance of the asynchronous API, you can force the module
to attach
any thread as a daemon thread to the JVM. This allows the program to not constantly attach
new threads
to the JVM as the old ones can be reused and thus improves the performance.

In order to import a class asynchronously, you can use the
[`importClassAsync`](https://markusjx.github.io/node-java-bridge/functions/importClassAsync.html)
function.

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
and can't wait for the java method to return while calling the proxy method at the same
time.

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

## Errors

Errors thrown in the java process are returned as `JavaError` objects.
These objects contain the error message, the full stack trace (including the java, node
and rust process) and the java throwable that caused
the error. The throwable is only available when the error was thrown in the java process
and not in the node process and if the call was a synchronous call.

The throwable can be accessed using the `cause` property of the
`JavaError` object.

```ts
import type { JavaError } from 'java-bridge';

try {
    // Call a method that throws an error
    someInstance.someMethodSync();
} catch (e: unknown) {
    const throwable = (e as JavaError).cause;
    throwable.printStackTraceSync();
}
```

If you want to access the Java throwable from an asynchronous call, you
need to enable
the `asyncJavaExceptionObjects` [config option](https://markusjx.github.io/node-java-bridge/variables/config.html)
before or while importing the class.
Enabling this will cause the stack trace of the JavaScript error to be lost.

```ts
import { importClass } from 'java-bridge';

const SomeClass = importClass('path.to.SomeClass', {
    asyncJavaExceptionObjects: true,
});

try {
    await SomeClass.someMethod();
} catch (e: unknown) {
    const throwable = (e as JavaError).cause;
    throwable.printStackTraceSync();
}
```

## Logging

If you want to enable logging for this module, you need to re-compile the module
with the `log` feature. Please install the dependencies listed in the
[build section](#build-instructions) and run `npm run build:all` to build the module with
all features enabled.

Logged events include:

- Class loading
- Method calls
- Class instance creation
- Method and class lookup

**Note:** Logging affects the performance of the module. Thus, it is recommended
to only enable logging when debugging.

For further information on how to use the logging feature, please take a look at the
[logging module documentation](https://markusjx.github.io/node-java-bridge/modules/internal.logging.html).

## Value conversion rules

1. Any basic value such as `string`, `number`, `boolean` or `BigInt` may be passed to
   methods accepting matching
   types
2. `string` values will always be converted to `java.lang.String`
3. `string` values with just one character may be converted to `char` or `java.lang.Char`
   if required
4. Thus, in order to pass a `char` to a java method, use a `string` containing just one
   character
5. `number` values will be converted
   to `int`, `long`, `double`, `float`, `java.lang.Integer`,
   `java.lang.Long`, `java.lang.Double` or `java.lang.Float` depending on the type the
   java function to call requires
6. `boolean` values will be converted to either `boolean` or `java.lang.Boolean`
7. `BigInt` values will be converted to either `long` or `java.lang.Long`
8. Arrays will be converted to java arrays. Java arrays may only contain a single value
   type, therefore the type of
   the first element in the array will be chosen as the array type, empty arrays need no
   conversions.
9. `java.lang.String` values will be converted to `string`
10. `int`, `double`, `float`, `java.lang.Integer`, `java.lang.Double` or `java.lang.Float`
    values will be converted to `number`
11. `long` or `java.lang.Long` values will always be converted to `BigInt`
12. `boolean` or `java.lang.Boolean` values will be converted to `boolean`
13. `char` or `java.lang.Character` values will be converted to `string`
14. Java arrays will be converted to javascript arrays, applying the rules mentioned above
    except
15. Byte arrays will be converted to `Buffer` and vice-versa
