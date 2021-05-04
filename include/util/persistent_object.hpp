#ifndef NODE_JAVA_BRIDGE_PERSISTENT_OBJECT_HPP
#define NODE_JAVA_BRIDGE_PERSISTENT_OBJECT_HPP

#include <napi.h>

#include "shared_releaser.hpp"

namespace util {
    class persistent_object {
    public:
        persistent_object() noexcept;

        explicit persistent_object(const Napi::Object &object);

        [[nodiscard]] Napi::Object value() const;

        void reset();

    private:
        shared_releaser releaser;
        Napi::ObjectReference *reference;
    };
}

#endif //NODE_JAVA_BRIDGE_PERSISTENT_OBJECT_HPP
