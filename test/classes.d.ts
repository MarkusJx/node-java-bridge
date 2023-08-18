import type { JavaClass } from '../ts-src';

export declare class JString extends JavaClass {
    constructor(value: string);

    static newInstanceAsync(value: string | null): Promise<JString>;

    static [`valueOf`](values: string[]): Promise<JString>;

    static [`valueOfSync`](values: string[]): JString;

    equals(other: JString): Promise<boolean>;

    equalsSync(other: JString): boolean;

    toCharArraySync(): string[];

    toCharArray(): Promise<string[]>;

    getBytesSync(): Buffer;

    splitSync(regex: string): string[];
}
