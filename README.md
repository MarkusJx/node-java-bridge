# node-java-bridge
A bridge between Node.js programs and Java APIs.

The goal was to create a tool similar to [@joeferner/node-java](https://github.com/joeferner/node-java) since
that API seems to be inactive.

Key differences from [@joeferner/node-java](https://github.com/joeferner/node-java):
* The java library is loaded dynamically, thus, the compiled binary is not bound to the specific jdk/jre it was compiled
  with, it can use basically any vm as long as the function definitions are equal to the ones from the jdk the binary
  was compiled with.
* Asynchronous function calls return actual Promises, rather than relying on callbacks

## Installation
```shell
npm i @markusjx/java
```

### Build requirements
* npm
* Git
* CMake
* Ninja or Make on Unix-based systems
* Some sort of [cmake-js](https://www.npmjs.com/package/cmake-js) compatible C++ compiler. May be one of:
    * [Visual C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
    * Clang or GCC
    
## Usage
```ts
const java = require('@markusjx/java');
// or
import java from "@markusjx/java";
```

### Managing the jvm
#### Creating a new jvm instance
In order to create a Java Virtual Machine instance, you may call ``java.ensureJVM(jvmPath?, version?)``.
This will create a jvm instance if it doesn't already exist. Calling this isn't required as it is called
on every call to any jvm functions.

Example: Creating a jvm with version 1.8 and the default path to the java library
```ts
java.ensureJVM(null, java.java_version.VER_1_8);
```

Alternatively, you may want to create a new java instance using ``java.createJVM(jvmPath?, version?)``.
This will destroy the old instance before creating the new one.

````ts
// Someone requested number 10?
java.createJVM(null, java.java_version.VER_10);
````

#### Deleting the jvm instance
If you want to delete the current jvm instance, you may want to call ``java.destroyJVM()``.
The jvm will be destroyed and all ongoing function calls may throw exceptions as the vm becomes invalid.

The behaviour is not actually *that well-defined* as the jvm does wait on attached threads to become detached,
therefore, async calls may still succeed, whereas sync calls will likely fail
(if there are no async operations currently running). Though the question still remains why anyone would want
to destroy the vm mid-call.

````ts
java.destroyJVM();
````

**General Notes**:
* There may only exist one jvm per process as the jni documentation states:
> As of JDK/JRE 1.2, creation of multiple VMs in a single process is not supported.
* The jvm may only be created and destroyed in the main thread (which shouldn't be that hard to achieve)

### Examples
#### Importing a java class
This imports the java String class:
````ts
const JString = java.importClass('java.lang.String');
// or, async version
const JString_async = await java.importClassAsync('java.lang.String');
````

## API definition
### ensureJVM
Create the jvm instance if it does not exist.

If the vm already exists, nothing is done, if it doesn't it will be created.
Calling this is optional as the jvm will be created with every call to it if it doesn't already exist.

**Arguments**
* ``jvmPath?: string | null`` - The path to the jvm.(so|dylib|dll). Must include the library name.
* ``version?: java_version | string | null`` - The version of the jvm to use

**Return type**

``void``

### importClass
Import a java class.

Since the class members and their signatures are resolved on import, this call may take some time.
To use your time more efficiently, consider using the async version of this function.
If a class of the same type is already imported (and not destroyed/garbage-collected), the class is
retrieved from a cache. Therefore, no members need to be resolved, reducing the import time. The class
is cached with the first import of the class and destroyed when the last ``java_class_proxy`` instance,
therefore the last imported instance, is garbage-collected.

**Arguments**
* ``classname: string`` - The class to import

**Return type**

``java_instance_proxy_constructor`` - The constructor function of the class

### importClassAsync
Import a java class. Async version.

The async call may be required in some cases as the class members and functions plus their signatures are resolved on
import which may take a while. Optimizations apply.

**Arguments**
* ``classname: string`` - The class to import

**Return type**

``Promise<java_instance_proxy_constructor>`` - The constructor function of the class wrapped in a Promise
