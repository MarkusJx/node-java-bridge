#include <jni.h>
#include <stdexcept>
#include <logger.hpp>

#include "jvm_lib/jvm_jvm.hpp"

using namespace jni;

jvm_jvm::jvm_jvm(JavaVM *vm) : jvm(vm), mtx() {}

jint jvm_jvm::GetEnv(void **env, jint version) {
    if (!valid()) throw std::runtime_error("The vm was destroyed");
    std::unique_lock<std::mutex> lock(mtx);
    return jvm->GetEnv(env, version);
}

jint jvm_jvm::AttachCurrentThread(void **env, void *options) {
    if (!valid()) throw std::runtime_error("The vm was destroyed");

    std::unique_lock<std::mutex> lock(mtx);
    return jvm->AttachCurrentThread(env, options);
}

jint jvm_jvm::DetachCurrentThread() {
    if (!valid()) throw std::runtime_error("The vm was destroyed");

    //std::unique_lock<std::mutex> lock(mtx);
    return jvm->DetachCurrentThread();
}

void jvm_jvm::forceReset() {
    if (!valid()) throw std::runtime_error("The vm was destroyed");

    std::unique_lock<std::mutex> lock(mtx);
    jvm->DestroyJavaVM();
    jvm = nullptr;
}

bool jvm_jvm::valid() {
    //std::unique_lock<std::mutex> lock(mtx);
    return jvm != nullptr;
}

std::mutex &jvm_jvm::mutex() {
    return mtx;
}

jvm_jvm::~jvm_jvm() {
    if (valid()) {
        std::unique_lock<std::mutex> lock(mtx);
        markusjx::logging::StaticLogger::debug("Destroying the java vm");
        jvm->DestroyJavaVM();
    }
}