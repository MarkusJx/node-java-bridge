import { JavaType, JavaClassInstance } from '../.';

/**
 * A java collection
 * @see https://docs.oracle.com/javase/8/docs/api/java/util/Collection.html
 */
export default class Collection<E extends JavaType> extends JavaClassInstance {
    /**
     * Ensures that this collection contains the specified element (optional operation).
     * Returns true if this collection changed as a result of the call.
     * (Returns false if this collection does not permit duplicates and already contains the specified element.)
     * Collections that support this operation may place limitations on what elements may
     * be added to this collection. In particular, some collections will refuse to add null elements,
     * and others will impose restrictions on the type of elements that may be added. Collection
     * classes should clearly specify in their documentation any restrictions on what elements may be added.
     *
     * If a collection refuses to add a particular element for any reason other than that
     * it already contains the element, it must throw an exception (rather than returning false).
     * This preserves the invariant that a collection always contains the specified element after
     * this call returns.
     *
     * Async call.
     *
     * @param e element whose presence in this collection is to be ensured
     * @returns true if this collection changed as a result of the call
     */
    add(e: E): Promise<boolean>;

    /**
     * Ensures that this collection contains the specified element (optional operation).
     * Returns true if this collection changed as a result of the call.
     * (Returns false if this collection does not permit duplicates and already contains the specified element.)
     * Collections that support this operation may place limitations on what elements may
     * be added to this collection. In particular, some collections will refuse to add null elements,
     * and others will impose restrictions on the type of elements that may be added. Collection
     * classes should clearly specify in their documentation any restrictions on what elements may be added.
     *
     * If a collection refuses to add a particular element for any reason other than that
     * it already contains the element, it must throw an exception (rather than returning false).
     * This preserves the invariant that a collection always contains the specified element after
     * this call returns.
     *
     * Sync call.
     *
     * @param e element whose presence in this collection is to be ensured
     * @returns true if this collection changed as a result of the call
     */
    addSync(e: E): Promise<boolean>;

    /**
     * Adds all of the elements in the specified collection to this collection (optional operation).
     * The behavior of this operation is undefined if the specified collection is modified while
     * the operation is in progress. (This implies that the behavior of this call is undefined
     * if the specified collection is this collection, and this collection is nonempty.)
     * Async call.
     *
     * @param c collection containing elements to be added to this collection
     * @returns true if this collection changed as a result of the call
     */
    addAll<T extends E>(c: Collection<T>): Promise<boolean>;

    /**
     * Adds all of the elements in the specified collection to this collection (optional operation).
     * The behavior of this operation is undefined if the specified collection is modified while
     * the operation is in progress. (This implies that the behavior of this call is undefined
     * if the specified collection is this collection, and this collection is nonempty.)
     * Sync call.
     *
     * @param c collection containing elements to be added to this collection
     * @returns true if this collection changed as a result of the call
     */
    addAllSync<T extends E>(c: Collection<T>): Promise<boolean>;

    /**
     * Removes all of the elements from this collection (optional operation).
     * The collection will be empty after this method returns.
     * Async call.
     */
    clear(): Promise<void>;

    /**
     * Removes all of the elements from this collection (optional operation).
     * The collection will be empty after this method returns.
     * Sync call.
     */
    clearSync(): void;

    /**
     * Returns true if this collection contains the specified element.
     * More formally, returns true if and only if this collection contains at
     * least one element e such that (o==null ? e==null : o.equals(e)).
     * Async call.
     *
     * @param o element whose presence in this collection is to be tested
     * @returns true if this collection contains the specified element
     */
    contains(o: E): Promise<boolean>;

    /**
     * Returns true if this collection contains the specified element.
     * More formally, returns true if and only if this collection contains at
     * least one element e such that (o==null ? e==null : o.equals(e)).
     * Sync call.
     *
     * @param o element whose presence in this collection is to be tested
     * @returns true if this collection contains the specified element
     */
    containsSync(o: E): boolean;

    /**
     * Returns true if this collection contains all of the elements in the specified collection.
     * Async call.
     *
     * @param c collection to be checked for containment in this collection
     * @returns true if this collection contains all of the elements in the specified collection
     */
    containsAll<T extends JavaType>(c: Collection<T>): Promise<boolean>;

    /**
     * Returns true if this collection contains all of the elements in the specified collection.
     * Sync call.
     *
     * @param c collection to be checked for containment in this collection
     * @returns true if this collection contains all of the elements in the specified collection
     */
    containsAllSync<T extends JavaType>(c: Collection<T>): boolean;

    /**
     * Returns true if this collection contains no elements.
     * Async call.
     *
     * @returns true if this collection contains no elements
     */
    isEmpty(): Promise<boolean>;

    /**
     * Returns true if this collection contains no elements.
     * Sync call.
     *
     * @returns true if this collection contains no elements
     */
    isEmptySync(): boolean;

    /**
     * Removes a single instance of the specified element from this collection,
     * if it is present (optional operation). More formally, removes an element e such that
     * (o==null ? e==null : o.equals(e)), if this collection contains one or more such elements.
     * Returns true if this collection contained the specified element
     * (or equivalently, if this collection changed as a result of the call).
     * Async call.
     *
     * @param o element to be removed from this collection, if present
     * @returns true if an element was removed as a result of this call
     */
    remove(o: E): Promise<boolean>;

    /**
     * Removes a single instance of the specified element from this collection,
     * if it is present (optional operation). More formally, removes an element e such that
     * (o==null ? e==null : o.equals(e)), if this collection contains one or more such elements.
     * Returns true if this collection contained the specified element
     * (or equivalently, if this collection changed as a result of the call).
     * Sync call.
     *
     * @param o element to be removed from this collection, if present
     * @returns true if an element was removed as a result of this call
     */
    removeSync(o: E): boolean;

    /**
     * Removes all of this collection's elements that are also contained in
     * the specified collection (optional operation). After this call returns,
     * this collection will contain no elements in common with the specified collection.
     * Async call.
     *
     * @param c collection containing elements to be removed from this collection
     * @returns true if this collection changed as a result of the call
     */
    removeAll<T extends JavaType>(c: Collection<T>): Promise<boolean>;

    /**
     * Removes all of this collection's elements that are also contained in
     * the specified collection (optional operation). After this call returns,
     * this collection will contain no elements in common with the specified collection.
     * Sync call.
     *
     * @param c collection containing elements to be removed from this collection
     * @returns true if this collection changed as a result of the call
     */
    removeAllSync<T extends JavaType>(c: Collection<T>): boolean;

    /**
     * Retains only the elements in this collection that are contained
     * in the specified collection (optional operation). In other words,
     * removes from this collection all of its elements that are not
     * contained in the specified collection.
     * Async call.
     *
     * @param c collection containing elements to be retained in this collection
     * @returns true if this collection changed as a result of the call
     */
    retainAll<T extends JavaType>(c: Collection<T>): Promise<boolean>;

    /**
     * Retains only the elements in this collection that are contained
     * in the specified collection (optional operation). In other words,
     * removes from this collection all of its elements that are not
     * contained in the specified collection.
     * Sync call.
     *
     * @param c collection containing elements to be retained in this collection
     * @returns true if this collection changed as a result of the call
     */
    retainAllSync<T extends JavaType>(c: Collection<T>): boolean;

    /**
     * Returns the number of elements in this collection.
     * If this collection contains more than Integer.MAX_VALUE elements,
     * returns Integer.MAX_VALUE.
     * Async call.
     *
     * @returns the number of elements in this collection
     */
    size(): Promise<number>;

    /**
     * Returns the number of elements in this collection.
     * If this collection contains more than Integer.MAX_VALUE elements,
     * returns Integer.MAX_VALUE.
     * Async call.
     *
     * @returns the number of elements in this collection
     */
    sizeSync(): number;

    /**
     * Returns an array containing all of the elements in this collection.
     * If this collection makes any guarantees as to what order its elements
     * are returned by its iterator, this method must return the elements in the same order.
     * Async call.
     *
     * @returns an array containing all of the elements in this collection
     */
    toArray(): Promise<E[]>;

    /**
     * Returns an array containing all of the elements in this collection.
     * If this collection makes any guarantees as to what order its elements
     * are returned by its iterator, this method must return the elements in the same order.
     * Sync call.
     *
     * @returns an array containing all of the elements in this collection
     */
    toArraySync(): E[];
}
