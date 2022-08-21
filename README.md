# @markusjx/java
A bridge between Node.js programs and Java APIs.

## Installation
```shell
npm i @markusjx/java
```

## Usage
### Synchronous calls
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
```ts
import { importClassAsync } from '@markusjx/java';

const JString = await importClassAsync('java.lang.String');

// Create a new instance asynchrnously using 'newInstanceAsync'
const str = await JString.newInstanceAsync('Hello World');

// Call methods asynchronously
await str.length(); // 11
await str.toString(); // 'Hello World'
```

### Value conversion rules

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