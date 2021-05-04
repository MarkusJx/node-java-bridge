#include "util/persistent_object.hpp"

using namespace util;

persistent_object::persistent_object() noexcept: releaser(nullptr), reference(nullptr) {}

persistent_object::persistent_object(const Napi::Object &object) : releaser(nullptr) {
    reference = new Napi::ObjectReference();
    *reference = Napi::Persistent(object);
    auto ref = reference;

    releaser = shared_releaser([ref] {
        ref->Unref();
        delete ref;
    });
}

Napi::Object persistent_object::value() const {
    return reference->Value();
}

void persistent_object::reset() {
    releaser.reset();
}
