#include "jvm_lib/jobject_wrapper.hpp"
#include "node_classes/jvm_container.hpp"

namespace jni {
    void jobject_wrapper_util::deleteRef(jobject object) {
        if (object != nullptr) {
            try {
                node_classes::jvm_container::attachJvm()->DeleteGlobalRef(object);
            } catch (...) {}
        }
    }
}