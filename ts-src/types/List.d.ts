import { JavaClassInstance, JavaType } from '../.';

/**
 * A java list.
 * @see https://docs.oracle.com/javase/8/docs/api/java/util/List.html
 */
export default class List<T extends JavaType> extends JavaClassInstance {
    /**
     * Returns the number of elements in this list.
     * If this list contains more than Integer.MAX_VALUE elements, returns Integer.MAX_VALUE.
     * Async call.
     *
     * @returns the number of elements in this list
     */
    size(): Promise<number>;

    /**
     * Returns the number of elements in this list.
     * If this list contains more than Integer.MAX_VALUE elements, returns Integer.MAX_VALUE.
     * Sync call.
     *
     * @returns the number of elements in this list
     */
    sizeSync(): number;

    /**
     * Appends the specified element to the end of this list (optional operation).
     * Lists that support this operation may place limitations on what elements may be added
     * to this list. In particular, some lists will refuse to add null elements, and others
     * will impose restrictions on the type of elements that may be added.
     * List classes should clearly specify in their documentation any restrictions on what elements may be added.
     * Async call.
     *
     * @param e element to be appended to this list
     */
    add(e: T): Promise<void>;

    /**
     * Appends the specified element to the end of this list (optional operation).
     * Lists that support this operation may place limitations on what elements may be added
     * to this list. In particular, some lists will refuse to add null elements, and others
     * will impose restrictions on the type of elements that may be added.
     * List classes should clearly specify in their documentation any restrictions on what elements may be added.
     * Sync call.
     *
     * @param e element to be appended to this list
     */
    addSync(e: T): void;

    /**
     * Returns the element at the specified position in this list.
     * Async call.
     *
     * @param index index of the element to return
     * @returns the element at the specified position in this list
     */
    get(index: number): Promise<T>;

    /**
     * Returns the element at the specified position in this list.
     * Sync call.
     *
     * @param index index of the element to return
     * @returns the element at the specified position in this list
     */
    getSync(index: number): T;

    /**
     * Returns an array containing all of the elements in this list
     * in proper sequence (from first to last element).
     * Async call.
     *
     * @returns an array containing all of the elements in this list in proper sequence
     */
    toArray(): Promise<T[]>;

    /**
     * Returns an array containing all of the elements in this list
     * in proper sequence (from first to last element).
     * Sync call.
     *
     * @returns an array containing all of the elements in this list in proper sequence
     */
    toArraySync(): T[];

    /**
     * Returns true if this list contains no elements.
     * Async call.
     *
     * @returns true if this list contains no elements
     */
    isEmpty(): Promise<boolean>;

    /**
     * Returns true if this list contains no elements.
     * Sync call.
     *
     * @returns true if this list contains no elements
     */
    isEmptySync(): boolean;

    /**
     * Returns true if this list contains the specified element.
     * More formally, returns true if and only if this list contains at least one element
     * e such that (o==null ? e==null : o.equals(e)).
     * Async call.
     *
     * @param o element whose presence in this list is to be tested
     * @returns true if this list contains the specified element
     */
    contains(o: T): Promise<boolean>;

    /**
     * Returns true if this list contains the specified element.
     * More formally, returns true if and only if this list contains at least one element
     * e such that (o==null ? e==null : o.equals(e)).
     * Sync call.
     *
     * @param o element whose presence in this list is to be tested
     * @returns true if this list contains the specified element
     */
    containsSync(o: T): boolean;

    /**
     * Removes all of the elements from this list (optional operation).
     * The list will be empty after this call returns.
     * Async call.
     */
    clear(): Promise<void>;

    /**
     * Removes all of the elements from this list (optional operation).
     * The list will be empty after this call returns.
     * Sync call.
     */
    clearSync(): void;

    /**
     * Returns the index of the last occurrence of the specified element in this list,
     * or -1 if this list does not contain the element. More formally,
     * returns the highest index i such that
     * (o==null ? get(i)==null : o.equals(get(i))), or -1 if there is no such index.
     * Async call.
     *
     * @param o element to search for
     */
    lastIndexOf(o: T): Promise<number>;

    /**
     * Returns the index of the last occurrence of the specified element in this list,
     * or -1 if this list does not contain the element. More formally,
     * returns the highest index i such that
     * (o==null ? get(i)==null : o.equals(get(i))), or -1 if there is no such index.
     * Sync call.
     *
     * @param o element to search for
     */
    lastIndexOfSync(o: T): number;

    /**
     * Removes the element at the specified position in this list (optional operation).
     * Shifts any subsequent elements to the left (subtracts one from their indices).
     * Returns the element that was removed from the list.
     * Async call.
     *
     * @param index the index of the element to be removed
     */
    remove(index: number): Promise<T>;

    /**
     * Removes the element at the specified position in this list (optional operation).
     * Shifts any subsequent elements to the left (subtracts one from their indices).
     * Returns the element that was removed from the list.
     * Sync call.
     *
     * @param index the index of the element to be removed
     */
    removeSync(index: number): T;
}
