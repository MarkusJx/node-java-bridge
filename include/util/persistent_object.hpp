#ifndef NODE_JAVA_BRIDGE_PERSISTENT_OBJECT_HPP
#define NODE_JAVA_BRIDGE_PERSISTENT_OBJECT_HPP

#include <napi.h>

#include "shared_releaser.hpp"

namespace util {
    /**
     * A wrapper around Napi::ObjectReference
     * and Napi::Persistent. This is basically a
     * shared_ptr for Napi::Objects.
     */
    class persistent_object {
    public:
        /**
         * Create a null persistent object.
         * Can't do anything.
         */
        persistent_object() noexcept;

        /**
         * Create a persistent object from a Napi::Object
         *
         * @param object the object to store
         */
        explicit persistent_object(const Napi::Object &object);

        /**
         * Get the stored object value.
         * Will throw an std::runtime_error
         * if the instance was never initialized.
         *
         * @return the stored object value
         */
        [[nodiscard]] Napi::Object value() const;

    private:
        // The releaser responsible for releasing the object
        shared_releaser releaser;

        // The reference to the object
        Napi::ObjectReference *reference;
    };
}

#endif //NODE_JAVA_BRIDGE_PERSISTENT_OBJECT_HPP
