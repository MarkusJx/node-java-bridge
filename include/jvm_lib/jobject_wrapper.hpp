#ifndef NODE_JAVA_BRIDGE_JOBJECT_WRAPPER_HPP
#define NODE_JAVA_BRIDGE_JOBJECT_WRAPPER_HPP

#include <util/shared_releaser.hpp>

#include "jvm_env.hpp"

namespace jni {
    /**
     * A wrapper around any jobject object.
     * Will manage the destruction of the managed element.
     * May be the poor man's version of some kind of shared_ptr.
     *
     * @tparam T the java type. Must be a pointer and
     * either extend jobject or be equal to jobject
     */
    template<class T>
    class jobject_wrapper {
    public:
        static_assert(std::is_pointer_v<T>, "T must be a pointer");
        static_assert(std::is_base_of_v<std::remove_pointer_t<jobject>, std::remove_pointer_t<T>> ||
                      std::is_same_v<T, jobject>, "T must extend jobject");

        /**
         * Create an empty jobject_wrapper.
         * Can't do shit, keep that in mind.
         */
        jobject_wrapper() noexcept: obj(nullptr), releaser(nullptr) {}

        /**
         * Store any jni object
         *
         * @param object the object to store
         * @param env the environment to delete the object with
         */
        jobject_wrapper(T object, const jvm_env &env) : obj(object), releaser(nullptr) {
            obj = reinterpret_cast<T>(env->NewGlobalRef(object));
            T obj_cpy = obj;

            releaser = shared_releaser([obj_cpy, env] {
                deleteRef(obj_cpy, env);
            });
        }

        /**
         * Wrap around a jobject.
         * Will cast the jobject to the
         * required type, defined using T.
         * Only available if T != jobject.
         * Using SFINAE to disable this statement if required.
         *
         * @param object the object to wrap around
         * @param env the environment to work in
         */
        template<class = int, class = typename std::enable_if_t<std::negation_v<std::is_same<T, jobject>>, int>>
        jobject_wrapper(jobject object, const jvm_env &env) : releaser(nullptr) {
            // Cast object to T
            obj = reinterpret_cast<T>(env->NewGlobalRef(object));
            T obj_cpy = obj;

            releaser = shared_releaser([obj_cpy, env] {
                deleteRef(obj_cpy, env);
            });
        }

        /**
         * Awful copy constructor
         *
         * @param object the object to store
         * @param releaser the releaser used to delete the object
         */
        jobject_wrapper(jobject object, shared_releaser releaser) : releaser(std::move(releaser)) {
            obj = reinterpret_cast<T>(object);
        }

        /**
         * Cast this jobject to another type
         *
         * @tparam U the type to cast to
         * @return the casted version of this
         */
        template<class U>
        jobject_wrapper<U> as() const {
            return jobject_wrapper<U>(this->obj, releaser);
        }

        /**
         * Assign another jobject to this wrapper.
         * Destroys the old object if required.
         *
         * @param other the jobject_wrapper to assign
         */
        void assign(const jobject_wrapper<T> &other) {
            releaser.assign(other.releaser);
            obj = other.obj;
        }

        /**
         * Assign a new object to this and release
         * the old one into the wild by taking it
         * into the woods. We all know what happens next.
         *
         * @param newObject the object to copy
         * @param env the environment to work in
         */
        void assign(jobject newObject, const jvm_env &env) {
            obj = reinterpret_cast<T>(env->NewGlobalRef(newObject));
            T object = obj;

            releaser.reset([object, env] {
                deleteRef(object, env);
            });
        }

        /**
         * Get the raw value
         *
         * @return the raw value
         */
        operator T() const {
            return obj;
        }

        /**
         * Check if the stored object is not null.
         * Should also check if the called java function
         * returned a null value as java's null values
         * are equal to C++'s nullptr.
         *
         * See: https://stackoverflow.com/a/2364405
         *
         * @return true if obj is not nullptr
         */
        [[nodiscard]] bool ok() const {
            return obj != nullptr && releaser;
        }

        // The stored value
        T obj;

    private:
        shared_releaser releaser;

        /**
         * Delete the reference to the object
         *
         * @param object the object to dereference
         * @param env the environment the object was created in
         */
        static void deleteRef(jobject object, const jvm_env &env) {
            if (object != nullptr && env.valid()) {
                env.attach_env()->DeleteGlobalRef(object);
            }
        }
    };
}

#endif //NODE_JAVA_BRIDGE_JOBJECT_WRAPPER_HPP
