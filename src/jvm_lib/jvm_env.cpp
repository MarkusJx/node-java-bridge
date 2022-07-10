#include <stdexcept>
#include <util/util.hpp>
#include <iostream>
#include "node_classes/java.hpp"

#ifdef ENABLE_LOGGING
#   include <logger.hpp>
#endif //ENABLE_LOGGING

#include "jvm_lib/jvm_env.hpp"

using namespace jni;

/**
 * Detach a thread from the jvm
 *
 * @param vm the vm to detach the thread from
 * @param detach whether to actually detach the thread (may be false on the default environment)
 */
void detachThread(const std::shared_ptr<jvm_jvm> &vm, bool detach) {
    if (detach && vm && vm->valid()) {
        vm->DetachCurrentThread();
    }
}

jvm_env::jvm_env() noexcept: env(nullptr), envReleaser(nullptr), jvm(nullptr), version(0) {}

jvm_env::jvm_env(const std::shared_ptr<jvm_jvm> &vm, JNIEnv *env, jint version, bool detach) : env(env), jvm(vm),
                                                                                               version(version),
                                                                                               envReleaser(nullptr) {
    envReleaser = shared_releaser([vm, detach] {
        detachThread(vm, detach);
    });
}

jvm_env jvm_env::attach_env() const {
    if (!jvm || env == nullptr || !jvm->valid()) {
        throw std::runtime_error("Tried attaching a new env to a non-existent jvm");
    }

    JNIEnv *environment = nullptr;
    jint create_result = jvm->GetEnv(reinterpret_cast<void **>(&environment), version);

    if (create_result == JNI_EDETACHED) {
        const bool create_daemon = node_classes::java::use_daemon_threads().load();
        create_result = jvm->AttachCurrentThread(reinterpret_cast<void **>(&environment), nullptr, create_daemon);
        if (create_result == JNI_OK) {
            return {jvm, environment, version, !create_daemon};
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
    if (!jvm || !jvm->valid()) throw std::runtime_error("The vm is destroyed");
    return env;
}

[[maybe_unused]] bool jvm_env::valid() const {
    return jvm && jvm->valid() && env != nullptr;
}

void jvm_env::forceReset() const {
    if (!jvm || !jvm->valid()) return;
    try {
        const_cast<shared_releaser &>(envReleaser).reset();
    } catch (...) {}

    try {
        jvm->forceReset();
    } catch (const std::exception &e) {
#ifdef ENABLE_LOGGING
        markusjx::logging::StaticLogger::errorStream << "Could not reset the jvm: " << e.what();
#endif //ENABLE_LOGGING
    }
}
