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
    if (reference == nullptr) {
        throw std::runtime_error("The persistent_object is not initialized");
    } else {
        return reference->Value();
    }
}
