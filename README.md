A bridge between Node.js programs and Java APIs.

## Installation

```shell
npm i @markusjx/java
```

## Usage

### Create the JVM

#### Create the JVM with no extra options

This will first search for a suitable `jvm` native library on the system and then
start the jvm with no extra options. This is also called when any call to the jvm is made
but the jvm is not yet started. Calling this after the jvm has already been created will do nothing.

```ts
import { ensureJvm } from '@markusjx/java';

ensureJvm();
```

#### Create the JVM with extra options

You can pass extra options to the jvm when creating it, for example requesting a specific jvm version,
specifying the location of the jvm native library or passing additional arguments to the jvm.

```ts
import { ensureJvm, JavaVersion } from '@markusjx/java';

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

### Synchronous calls

If you want to use Java APIs in a synchronous way, you can use the synchronous API of this module.
Any call to the Java API will be executed in the same thread as your node process so this
may cause your program to hang until the execution is finished. But - in contrast to the asynchronous API -
these calls are a lot faster as no extra threads need to be created/attached to the JVM.

All synchronous java methods are proceeded with the postfix `Sync`.
This means, all methods of a class (static and non-static) are generated twice,
once as a synchronous call and once as an asynchronous call.

If you are looking for asynchronous calls, take a look at the next section.

```ts
import { importClass } from '@markusjx/java';

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

```ts
import { importClassAsync } from '@markusjx/java';

const JString = await importClassAsync('java.lang.String');

// Create a new instance asynchrnously using 'newInstanceAsync'
const str = await JString.newInstanceAsync('Hello World');

// Call methods asynchronously
await str.length(); // 11
await str.toString(); // 'Hello World'
```

### Implement a Java interface

```ts
import { newProxy } from '@markusjx/java';

const proxy = newProxy('path.to.MyInterface', {
    // Define methods...
});

// Do something with the proxy
instance.someMethod(proxy);

// Destroy the proxy
proxy.reset();
```

### Redirect the stdout and stderr from the java process

```ts
import { stdout } from '@markusjx/java';

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
14. Java arrays will be converted to javascript arrays, applying the rules mentioned above
