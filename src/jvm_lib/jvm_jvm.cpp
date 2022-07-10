#include <jni.h>
#include <stdexcept>
#ifdef ENABLE_LOGGING
#   include <logger.hpp>
#endif //ENABLE_LOGGING

#include "jvm_lib/jvm_jvm.hpp"

using namespace jni;

jvm_jvm::jvm_jvm(JavaVM *vm) : jvm(vm), mtx() {}

jint jvm_jvm::GetEnv(void **env, jint version) {
    if (!valid())
        throw std::runtime_error("The vm was destroyed");

    std::unique_lock<std::mutex> lock(mtx);
    return jvm->GetEnv(env, version);
}

jint jvm_jvm::AttachCurrentThread(void **env, void *options, bool create_daemon) {
    if (!valid())
        throw std::runtime_error("The vm was destroyed");

    std::unique_lock<std::mutex> lock(mtx);
    if (create_daemon) {
        return jvm->AttachCurrentThreadAsDaemon(env, options);
    } else {
        return jvm->AttachCurrentThread(env, options);
    }
}

jint jvm_jvm::DetachCurrentThread() {
    if (!valid())
        throw std::runtime_error("The vm was destroyed");

    std::unique_lock<std::mutex> lock(mtx);
#ifdef ENABLE_LOGGING
    markusjx::logging::StaticLogger::debugStream << "Detaching thread: " << std::this_thread::get_id();
#endif //ENABLE_LOGGING

    return jvm->DetachCurrentThread();
}

void jvm_jvm::forceReset() {
    if (!valid())
        throw std::runtime_error("The vm was destroyed");

    std::unique_lock<std::mutex> lock(mtx);

#ifdef ENABLE_LOGGING
    markusjx::logging::StaticLogger::debug("Destroying the java vm");;
#endif //ENABLE_LOGGING

    jvm->DestroyJavaVM();
#ifdef ENABLE_LOGGING
    markusjx::logging::StaticLogger::debug("Jvm destroyed");
#endif //ENABLE_LOGGING
    jvm = nullptr;
}

bool jvm_jvm::valid() {
    std::unique_lock<std::mutex> lock(mtx);
    return jvm != nullptr;
}

jvm_jvm::~jvm_jvm() {
    if (valid()) {
        std::unique_lock<std::mutex> lock(mtx);
#ifdef ENABLE_LOGGING
        markusjx::logging::StaticLogger::debug("Destroying the java vm");
#endif //ENABLE_LOGGING
        jvm->DestroyJavaVM();
    }
}