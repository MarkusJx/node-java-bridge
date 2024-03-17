import type { JavaClass, JavaInterfaceProxy, JavaType } from '../.';

export declare class JString extends JavaClass {
    constructor();
    constructor(value: string);
    constructor(value: Buffer);

    static newInstanceAsync(value: string | null): Promise<JString>;

    static [`valueOf`](values: string[]): Promise<JString>;

    static [`valueOfSync`](values: string[]): JString;

    equals(other: JString): Promise<boolean>;

    equalsSync(other: JString): boolean;

    toCharArraySync(): string[];

    toCharArray(): Promise<string[]>;

    getBytesSync(): Buffer;

    splitSync(regex: string): string[];

    transform(fn: JavaInterfaceProxy<FunctionInterface<string>>): Promise<this>;
}

export interface FunctionInterface<T> {
    apply(value: T): T;
}

export declare class RuntimeClass extends JavaClass {
    public static getRuntimeSync(): RuntimeClass;

    public totalMemorySync(): bigint;

    public freeMemorySync(): bigint;
}

export interface Stream {
    printlnSync(msg: string): void;

    flushSync(): void;
}

export declare class SystemClass extends JavaClass {
    public static readonly out: Stream;
    public static readonly err: Stream;

    public static gcSync(): void;
}

export declare class StreamClass<T extends JavaType> extends JavaClass {
    toListSync(): ListClass<T>;

    toList(): Promise<ListClass<T>>;
}

export declare class ListClass<T extends JavaType> extends JavaClass {
    containsSync(element: T): boolean;

    sizeSync(): number;

    getSync(index: number): T;

    lastIndexOfSync(element: T): number;

    addSync(value: T): void;

    removeSync(index: number): T;

    toArraySync(): T[];

    isEmptySync(): boolean;

    clearSync(): void;

    add(value: T): Promise<void>;

    lastIndexOf(element: T): Promise<number>;

    contains(element: T): Promise<boolean>;

    isEmpty(): Promise<boolean>;

    size(): Promise<number>;

    get(index: number): Promise<T>;

    remove(index: number): Promise<T>;

    clear(): Promise<void>;

    streamSync(): StreamClass<T>;

    stream(): Promise<StreamClass<T>>;
}

export declare class ArrayListClass<T extends JavaType> extends ListClass<T> {
    static newInstanceAsync(): Promise<ArrayListClass<JavaType>>;

    constructor();
    constructor(other: ListClass<T>);
}
