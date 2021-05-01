#include <stdexcept>
#include <util.hpp>

#include "jvm_lib/jvm_env.hpp"

using namespace jni;

jvm_env::jvm_env() noexcept: env(nullptr), jvm(nullptr), shared_releaser(nullptr), version(0) {}

jvm_env::jvm_env(JavaVM *vm, JNIEnv *environment, jint version, bool detachThread) : env(environment), jvm(vm),
                                                                                     shared_releaser(
                                                                                             [vm, detachThread] {
                                                                                                 if (detachThread) {
                                                                                                     vm->DetachCurrentThread();
                                                                                                 } else {
                                                                                                     vm->DestroyJavaVM();
                                                                                                 }
                                                                                             }), version(version) {}

jvm_env jvm_env::attach_env() const {
    if (jvm == nullptr || env == nullptr) {
        throw std::runtime_error("Tried attaching a new env to a non-existent jvm");
    }

    JNIEnv *environment = nullptr;
    jint create_result = jvm->GetEnv(reinterpret_cast<void **>(&environment), version);

    if (create_result == JNI_EDETACHED) {
        JavaVMInitArgs vm_args;

        vm_args.version = version;
        vm_args.nOptions = 0;
        vm_args.options = nullptr;
        vm_args.ignoreUnrecognized = false;

        create_result = jvm->AttachCurrentThread(reinterpret_cast<void **>(&environment), &vm_args);
        if (create_result == JNI_OK) {
            return jvm_env(jvm, environment, true);
        } else {
            throw std::runtime_error("AttachCurrentThread failed: " + util::jni_error_to_string(create_result));
        }
    } else if (create_result == JNI_OK) {
        return *this;
    } else {
        throw std::runtime_error("GetEnv failed: " + util::jni_error_to_string(create_result));
    }
}

JNIEnv *jvm_env::operator->() const {
    return env;
}